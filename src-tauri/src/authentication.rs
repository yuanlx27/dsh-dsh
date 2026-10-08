//! Private, transient native authentication. Never navigate a WebView to the
//! token URL, evaluate index scripts or serialize/debug a session.
use crate::{errors::Failure, runtime::validate_token_url};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{
    Client, StatusCode,
    header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, LOCATION, SET_COOKIE},
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::time::{Instant, timeout_at};
use url::{Position, Url};

const MAX_BOOT_BYTES: usize = 2 * 1024 * 1024;
const GLOBALS: [&str; 6] = [
    "__DSH_BOOT__",
    "__DSH_CONNECTION_RECOVERY__",
    "__DSH_MODELS_ONBOARDING__",
    "__DSH_CONTACT_CONFIG__",
    "__DSH_SHORTCUTS_CONFIG__",
    "__DSH_DOCUMENT_PREVIEW_CONFIG__",
];

#[derive(Serialize)]
pub struct BootData {
    pub globals: BTreeMap<String, Value>,
}

// Deliberately no Debug/Serialize, including on the outer result.
pub struct Authenticated {
    pub session: Arc<ServiceSession>,
    pub boot: BootData,
}
struct Credentials {
    origin: Url,
    cookie: HeaderValue,
}
pub struct ServiceSession {
    generation: u64,
    credentials: Mutex<Option<Credentials>>,
}
impl ServiceSession {
    pub fn is_current(&self, generation: u64) -> bool {
        self.generation == generation && self.credentials.lock().is_ok_and(|c| c.is_some())
    }
    pub fn invalidate(&self) {
        if let Ok(mut credentials) = self.credentials.lock() {
            *credentials = None;
        }
    }
    // Native consumers can construct a request without retaining a mutex guard
    // across I/O. Future forwarding still needs admission/revocation checks.
    pub(crate) fn with_credentials<T>(
        &self,
        generation: u64,
        use_credentials: impl FnOnce(&Url, &HeaderValue) -> T,
    ) -> Result<T, Failure> {
        if generation != self.generation {
            return Err(Failure::Unavailable);
        }
        let credentials = self.credentials.lock().map_err(|_| Failure::Unavailable)?;
        let credentials = credentials.as_ref().ok_or(Failure::Unavailable)?;
        Ok(use_credentials(&credentials.origin, &credentials.cookie))
    }
}

pub async fn authenticate(
    mut token_url: Url,
    generation: u64,
    deadline: Instant,
) -> Result<Authenticated, Failure> {
    if deadline <= Instant::now() {
        return Err(Failure::Timeout);
    }
    validate_token_url(&token_url).map_err(|_| Failure::Connection)?;
    // Literal owned loopback only: never consult inherited/system HTTP proxies
    // or install a cookie jar. Every network error becomes a fixed category.
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .map_err(|_| Failure::Connection)?;
    timeout_at(deadline, async move {
        let mut token = token_url
            .query_pairs()
            .next()
            .ok_or(Failure::Connection)?
            .1
            .into_owned();
        let exchange = client
            .get(token_url.clone())
            .send()
            .await
            .map_err(|_| Failure::Connection)?;
        token_url.set_query(None); // No token-bearing URL retained after exchange.
        if exchange.status() != StatusCode::SEE_OTHER
            || exchange
                .headers()
                .get(LOCATION)
                .and_then(|v| v.to_str().ok())
                != Some("./")
        {
            return Err(Failure::Connection);
        }
        let cookie = service_cookie(exchange.headers(), &token_url)?;
        drop(exchange);
        let cookie_secret = cookie
            .to_str()
            .map_err(|_| Failure::Connection)?
            .split_once('=')
            .ok_or(Failure::Connection)?
            .1
            .to_owned();
        let origin_secret = token_url.origin().ascii_serialization();
        let session = Arc::new(ServiceSession {
            generation,
            credentials: Mutex::new(Some(Credentials {
                origin: token_url,
                cookie,
            })),
        });
        let request = session.with_credentials(generation, |origin, cookie| {
            client.get(origin.clone()).header(COOKIE, cookie.clone())
        })?;
        // R itself validates the signed cookie by admitting this clean index.
        // Native code does not read or duplicate its persistent signing secret.
        let mut index = request.send().await.map_err(|_| Failure::Connection)?;
        if index.status() != StatusCode::OK
            || index
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split(';').next())
                .is_none_or(|v| !v.trim().eq_ignore_ascii_case("text/html"))
        {
            return Err(Failure::Connection);
        }
        if index
            .content_length()
            .is_some_and(|length| length > MAX_BOOT_BYTES as u64)
        {
            return Err(Failure::Connection);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = index.chunk().await.map_err(|_| Failure::Connection)? {
            if chunk.len() > MAX_BOOT_BYTES - bytes.len() {
                return Err(Failure::Connection);
            }
            bytes.extend_from_slice(&chunk);
        }
        drop(index);
        let html = std::str::from_utf8(&bytes).map_err(|_| Failure::Connection)?;
        let boot = extract_boot(html, &[&token, &cookie_secret, &origin_secret])?;
        // Used only to detect accidental auth echoes in extracted JSON; no token
        // goes into the session or returned boot. Clearing is not zeroization.
        token.clear();
        Ok(Authenticated { session, boot })
    })
    .await
    .map_err(|_| Failure::Timeout)?
}

fn service_cookie(headers: &HeaderMap, origin: &Url) -> Result<HeaderValue, Failure> {
    let mut cookies = headers.get_all(SET_COOKIE).iter();
    let raw = cookies.next().ok_or(Failure::Connection)?;
    if cookies.next().is_some() || raw.as_bytes().len() > 8192 {
        return Err(Failure::Connection);
    }
    let raw = raw.to_str().map_err(|_| Failure::Connection)?;
    let mut fields = raw.split(';').map(str::trim);
    let pair = fields.next().ok_or(Failure::Connection)?;
    let (name, value) = pair.split_once('=').ok_or(Failure::Connection)?;
    let authority = &origin[Position::BeforeHost..Position::AfterPort];
    let expected = format!(
        "dsh-auth-{}",
        URL_SAFE_NO_PAD.encode(Sha256::digest(authority.as_bytes()))
    );
    if name != expected {
        return Err(Failure::Connection);
    }
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3 || parts[0] != "v1" {
        return Err(Failure::Connection);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| Failure::Connection)?;
    let signature = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| Failure::Connection)?;
    if payload.is_empty()
        || signature.len() != 32
        || URL_SAFE_NO_PAD.encode(&payload) != parts[1]
        || URL_SAFE_NO_PAD.encode(&signature) != parts[2]
    {
        return Err(Failure::Connection);
    }
    let mut attributes = BTreeMap::new();
    for field in fields {
        let (name, value) = field
            .split_once('=')
            .map_or((field, None), |(n, v)| (n, Some(v)));
        let name = name.to_ascii_lowercase();
        if !["max-age", "path", "expires", "httponly", "samesite"].contains(&name.as_str())
            || attributes.insert(name, value).is_some()
        {
            return Err(Failure::Connection);
        }
    }
    if attributes.get("path") != Some(&Some("/"))
        || attributes.get("httponly") != Some(&None)
        || !attributes
            .get("samesite")
            .is_some_and(|v| v.is_some_and(|v| v.eq_ignore_ascii_case("Strict")))
        || !attributes
            .get("max-age")
            .is_some_and(|v| v.is_some_and(|v| v.parse::<u64>().is_ok_and(|age| age > 0)))
        || attributes
            .get("expires")
            .is_some_and(|v| v.is_none_or(str::is_empty))
    {
        return Err(Failure::Connection);
    }
    let mut header = HeaderValue::from_str(pair).map_err(|_| Failure::Connection)?;
    header.set_sensitive(true);
    Ok(header)
}

fn extract_boot(html: &str, secrets: &[&str]) -> Result<BootData, Failure> {
    let mut globals = BTreeMap::new();
    // Match only R's pinned structured JSON renderer. This is intentionally not
    // a JS evaluator/general HTML scraper. Inline queue/scripts stay packaged.
    for script in html.split("<script>").skip(1) {
        let (script, _) = script.split_once("</script>").ok_or(Failure::Connection)?;
        let Some(row) = script.strip_prefix("globalThis[") else {
            continue;
        };
        let (name, json) = row.split_once("] = ").ok_or(Failure::Connection)?;
        let name: String = serde_json::from_str(name).map_err(|_| Failure::Connection)?;
        if !GLOBALS.contains(&name.as_str()) {
            continue;
        }
        let value: Value = serde_json::from_str(json).map_err(|_| Failure::Connection)?;
        if !value.is_object() || globals.insert(name, value).is_some() {
            return Err(Failure::Connection);
        }
    }
    if !globals.contains_key("__DSH_BOOT__") || !globals.contains_key("__DSH_CONNECTION_RECOVERY__")
    {
        return Err(Failure::Connection);
    }
    let json = serde_json::to_string(&globals).map_err(|_| Failure::Connection)?;
    if secrets
        .iter()
        .any(|secret| !secret.is_empty() && json.contains(secret))
    {
        return Err(Failure::Connection);
    }
    Ok(BootData { globals })
}

//! Fixed owned-service HTTP only. No cookie jar, redirect, proxy, retry or replay.
use crate::{authentication::ServiceSession, bridge::Permit, errors::Failure};
use bytes::Bytes;
use reqwest::{
    Client, Method, Response,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::sync::Arc;
use tokio::sync::watch;
use url::Url;

pub const MAX_BODY_BYTES: usize = 300 * 1024 * 1024;
pub const MAX_CHUNK_BYTES: usize = 64 * 1024;
#[derive(Default)]
pub struct Body {
    bytes: Vec<u8>,
    next: u64,
    poisoned: bool,
}
impl Body {
    pub fn push(&mut self, sequence: u64, bytes: Vec<u8>) -> Result<(), Failure> {
        if self.poisoned
            || sequence != self.next
            || bytes.len() > MAX_CHUNK_BYTES
            || bytes.len() > MAX_BODY_BYTES - self.bytes.len()
        {
            self.poisoned = true;
            return Err(Failure::Unavailable);
        }
        self.next = self.next.checked_add(1).ok_or(Failure::Unavailable)?;
        self.bytes.extend(bytes);
        Ok(())
    }
    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }
}
pub struct HttpRequest {
    pub path: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Body,
}
pub struct Chunk {
    pub sequence: u64,
    pub bytes: Vec<u8>,
}
pub struct Transport {
    session: Arc<ServiceSession>,
    client: Client,
}
pub struct Reply {
    pub status: u16,
    pub headers: HeaderMap,
    response: Response,
    pending: Bytes,
    sequence: u64,
    session: Arc<ServiceSession>,
    generation: u64,
    cancel: watch::Receiver<bool>,
    permit: Option<Permit>,
    ended: bool,
    failed: bool,
}
impl Transport {
    pub fn new(session: Arc<ServiceSession>) -> Self {
        // Fixed options are validated by reqwest; failure is never renderer data.
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Native HTTP client initialization failed.");
        Self { session, client }
    }
    pub async fn fetch_admitted(
        &self,
        permit: &Permit,
        input: HttpRequest,
        cancel: watch::Receiver<bool>,
    ) -> Result<Reply, Failure> {
        if !permit.is_current() {
            return Err(Failure::Unavailable);
        }
        self.fetch_owned(
            permit.runtime_generation(),
            input,
            cancel,
            Some(permit.clone()),
        )
        .await
    }
    // Native-only lower-layer API for isolated IO contracts. Never expose this
    // generation-only entry point as an IPC handler; use fetch_admitted there.
    pub async fn fetch(
        &self,
        generation: u64,
        input: HttpRequest,
        cancel: watch::Receiver<bool>,
    ) -> Result<Reply, Failure> {
        self.fetch_owned(generation, input, cancel, None).await
    }
    async fn fetch_owned(
        &self,
        generation: u64,
        input: HttpRequest,
        cancel: watch::Receiver<bool>,
        permit: Option<Permit>,
    ) -> Result<Reply, Failure> {
        current(&self.session, generation, &cancel, permit.as_ref())?;
        if input.body.poisoned
            || !matches!(
                input.method,
                Method::GET
                    | Method::HEAD
                    | Method::POST
                    | Method::PUT
                    | Method::PATCH
                    | Method::DELETE
                    | Method::OPTIONS
            )
        {
            return Err(Failure::Unavailable);
        }
        let relative = validate_target(&input.path)?;
        let mut headers = clean_headers(&input.headers, true)?;
        let target = self
            .session
            .with_credentials(generation, |origin, cookie| {
                let target = origin.join(&relative).map_err(|_| Failure::Unavailable)?;
                if target.origin() != origin.origin() {
                    return Err(Failure::Unavailable);
                }
                headers.insert("cookie", cookie.clone());
                headers.insert(
                    "origin",
                    HeaderValue::from_str(&origin.origin().ascii_serialization())
                        .map_err(|_| Failure::Unavailable)?,
                );
                Ok::<_, Failure>(target)
            })??;
        current(&self.session, generation, &cancel, permit.as_ref())?;
        let send = self
            .client
            .request(input.method, target)
            .headers(headers)
            .body(input.body.bytes)
            .send();
        let response = tokio::select! { biased;
            _ = cancelled(cancel.clone()) => return Err(Failure::Unavailable),
            _ = self.session.revoked(generation) => return Err(Failure::Unavailable),
            _ = permit_revoked(permit.as_ref()) => return Err(Failure::Unavailable),
            response = send => response.map_err(|_| Failure::Connection)?,
        };
        current(&self.session, generation, &cancel, permit.as_ref())?;
        if response.status().is_redirection() {
            return Err(Failure::Connection);
        }
        let status = response.status().as_u16();
        let headers = clean_headers(response.headers(), false)?;
        Ok(Reply {
            status,
            headers,
            response,
            pending: Bytes::new(),
            sequence: 0,
            session: self.session.clone(),
            generation,
            cancel,
            permit,
            ended: false,
            failed: false,
        })
    }
}
impl Reply {
    // Pull based: no background buffering or unbounded global event stream.
    pub async fn next_chunk(&mut self) -> Result<Option<Chunk>, Failure> {
        if self.failed {
            return Err(Failure::Unavailable);
        }
        let result = self.next_ordered().await;
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    async fn next_ordered(&mut self) -> Result<Option<Chunk>, Failure> {
        current(
            &self.session,
            self.generation,
            &self.cancel,
            self.permit.as_ref(),
        )?;
        if self.ended {
            return Ok(None);
        }
        if self.pending.is_empty() {
            self.pending = tokio::select! { biased;
                _ = cancelled(self.cancel.clone()) => return Err(Failure::Unavailable),
                _ = self.session.revoked(self.generation) => return Err(Failure::Unavailable),
                _ = permit_revoked(self.permit.as_ref()) => return Err(Failure::Unavailable),
                chunk = self.response.chunk() => match chunk.map_err(|_| Failure::Connection)? {
                    Some(bytes) => bytes, None => { self.ended = true; return Ok(None); }
                },
            };
        }
        current(
            &self.session,
            self.generation,
            &self.cancel,
            self.permit.as_ref(),
        )?;
        if self.pending.len() > 1024 * 1024 {
            return Err(Failure::Connection);
        }
        let bytes = self
            .pending
            .split_to(self.pending.len().min(MAX_CHUNK_BYTES))
            .to_vec();
        let sequence = self.sequence;
        self.sequence = self.sequence.checked_add(1).ok_or(Failure::Unavailable)?;
        Ok(Some(Chunk { sequence, bytes }))
    }
}
fn current(
    session: &ServiceSession,
    generation: u64,
    cancel: &watch::Receiver<bool>,
    permit: Option<&Permit>,
) -> Result<(), Failure> {
    if *cancel.borrow()
        || cancel.has_changed().is_err()
        || !session.is_current(generation)
        || permit.is_some_and(|p| !p.is_current())
    {
        return Err(Failure::Unavailable);
    }
    Ok(())
}
pub(crate) async fn cancelled(mut signal: watch::Receiver<bool>) {
    loop {
        if *signal.borrow() || signal.changed().await.is_err() {
            return;
        }
    }
}
pub(crate) async fn permit_revoked(permit: Option<&Permit>) {
    match permit {
        Some(p) => p.revoked().await,
        None => std::future::pending().await,
    }
}
pub(crate) fn validate_target(input: &str) -> Result<String, Failure> {
    if input.is_empty()
        || input.len() > 16384
        || input.contains(['\\', '#'])
        || input.chars().any(char::is_control)
    {
        return Err(Failure::Unavailable);
    }
    let input = if input.starts_with('/') {
        input.to_owned()
    } else {
        format!("/{input}")
    };
    if input.starts_with("//") {
        return Err(Failure::Unavailable);
    }
    let (path, query) = input.split_once('?').unwrap_or((&input, ""));
    for raw in [path, query] {
        let bytes = raw.as_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            if *byte == b'%'
                && (i + 2 >= bytes.len()
                    || !bytes[i + 1].is_ascii_hexdigit()
                    || !bytes[i + 2].is_ascii_hexdigit())
            {
                return Err(Failure::Unavailable);
            }
        }
        let decoded = percent_encoding::percent_decode_str(raw)
            .decode_utf8()
            .map_err(|_| Failure::Unavailable)?;
        if decoded.contains(['%', '\\']) || decoded.chars().any(char::is_control) {
            return Err(Failure::Unavailable);
        }
    }
    let decoded = percent_encoding::percent_decode_str(path)
        .decode_utf8()
        .map_err(|_| Failure::Unavailable)?;
    if path.contains(':')
        || decoded.contains(':')
        || decoded
            .split('/')
            .skip(1)
            .any(|p| matches!(p, "" | "." | ".."))
        || path.to_ascii_lowercase().contains("%2f")
        || path.to_ascii_lowercase().contains("%5c")
    {
        return Err(Failure::Unavailable);
    }
    if path == "/" || url::form_urlencoded::parse(query.as_bytes()).any(|(k, _)| k == "token") {
        return Err(Failure::Unavailable);
    }
    // Use URL's parser only after raw traversal/authority checks, never normalize
    // a hostile input into an allowed target.
    let base = Url::parse("http://127.0.0.1/").map_err(|_| Failure::Unavailable)?;
    let result = base.join(&input).map_err(|_| Failure::Unavailable)?;
    if result.origin() != base.origin() {
        return Err(Failure::Unavailable);
    }
    Ok(input)
}
pub(crate) fn clean_headers(input: &HeaderMap, request: bool) -> Result<HeaderMap, Failure> {
    if input.len() > 128
        || input
            .iter()
            .map(|(k, v)| k.as_str().len() + v.as_bytes().len())
            .sum::<usize>()
            > 65536
    {
        return Err(Failure::Unavailable);
    }
    let mut output = input.clone();
    let connection: Vec<HeaderName> = input
        .get_all("connection")
        .iter()
        .flat_map(|v| v.to_str().unwrap_or("").split(','))
        .filter_map(|s| HeaderName::from_bytes(s.trim().as_bytes()).ok())
        .collect();
    for name in connection {
        output.remove(name);
    }
    for name in [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
        "set-cookie",
    ] {
        output.remove(name);
    }
    if request {
        let names: Vec<_> = output
            .keys()
            .filter(|k| {
                matches!(
                    k.as_str(),
                    "host"
                        | "origin"
                        | "cookie"
                        | "authorization"
                        | "referer"
                        | "content-length"
                        | "forwarded"
                ) || k.as_str().starts_with("sec-")
                    || k.as_str().starts_with("x-forwarded-")
            })
            .cloned()
            .collect();
        for name in names {
            output.remove(name);
        }
    }
    Ok(output)
}

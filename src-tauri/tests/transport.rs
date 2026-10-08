//! T016 contract-first tests. Authentication (T020) and HTTP forwarding (T025)
//! are intentionally absent initially. This is not installed WKWebView evidence.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use deepseek_harness_desktop::{
    authentication::{self, Authenticated},
    errors::Failure,
    http_transport::{Body, HttpRequest, MAX_BODY_BYTES, Transport},
};
use reqwest::{Method, header::HeaderMap};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Notify, watch},
    task::JoinHandle,
    time::{Instant, timeout},
};
use url::Url;

const TOKEN: &str = "disposable-transport-fixture-not-a-real-token";
const BOOT: &str = r#"<html><head><script>globalThis["__DSH_BOOT__"] = {"entries":[],"batches":[]}</script><script>globalThis["__DSH_CONNECTION_RECOVERY__"] = {"retryDelays":[0,100]}</script><script>globalThis["UNTRUSTED_INJECTION"] = "discard-me"</script></head><body>not returned as HTML</body></html>"#;

#[derive(Clone)]
struct Response {
    status: u16,
    headers: Vec<(String, String)>,
    chunks: Vec<Vec<u8>>,
    hold: bool,
}
impl Response {
    fn ok(bytes: &[u8]) -> Self {
        Self {
            status: 200,
            headers: vec![],
            chunks: vec![bytes.to_vec()],
            hold: false,
        }
    }
}
struct RecordedRequest {
    method: String,
    path: String,
    headers: HeaderMap,
    body: Vec<u8>,
}
struct Fixture {
    url: Url,
    cookie: String,
    exchange: Arc<Mutex<Response>>,
    index: Arc<Mutex<Response>>,
    api: Arc<Mutex<Response>>,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    api_entered: Arc<Notify>,
    server: JoinHandle<()>,
}
impl Fixture {
    async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let authority = listener.local_addr().unwrap().to_string();
        // Reproduce R's cookie naming/opaque shape with synthetic, non-signing bytes.
        // The fixture is not upstream BrowserAuth; real-cookie admission is T032.
        let name = URL_SAFE_NO_PAD.encode(Sha256::digest(authority.as_bytes()));
        let payload = URL_SAFE_NO_PAD.encode(b"synthetic-cookie-payload");
        let signature = URL_SAFE_NO_PAD.encode([0_u8; 32]);
        let cookie = format!("dsh-auth-{name}=v1.{payload}.{signature}");
        let exchange = Arc::new(Mutex::new(Response {
            status: 303,
            headers: vec![
                ("location".into(), "./".into()),
                (
                    "set-cookie".into(),
                    format!("{cookie}; Max-Age=2592000; Path=/; HttpOnly; SameSite=Strict"),
                ),
            ],
            chunks: vec![],
            hold: false,
        }));
        let mut index_reply = Response::ok(BOOT.as_bytes());
        index_reply
            .headers
            .push(("content-type".into(), "text/html; charset=utf-8".into()));
        let index = Arc::new(Mutex::new(index_reply));
        let api = Arc::new(Mutex::new(Response::ok(b"opaque response")));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let api_entered = Arc::new(Notify::new());
        let (e, i, a, r, entered) = (
            exchange.clone(),
            index.clone(),
            api.clone(),
            requests.clone(),
            api_entered.clone(),
        );
        let server = tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let header_end = loop {
                    let mut chunk = [0; 8192];
                    let count = socket.read(&mut chunk).await.unwrap();
                    if count == 0 {
                        return;
                    }
                    bytes.extend_from_slice(&chunk[..count]);
                    if let Some(at) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        break at + 4;
                    }
                    assert!(
                        bytes.len() <= 65536,
                        "Fixture request header bound exceeded."
                    );
                };
                let head = std::str::from_utf8(&bytes[..header_end]).unwrap();
                let mut lines = head.split("\r\n");
                let mut start = lines.next().unwrap().split_whitespace();
                let method = start.next().unwrap().to_owned();
                let path = start.next().unwrap().to_owned();
                let mut headers = HeaderMap::new();
                for line in lines.filter(|line| !line.is_empty()) {
                    let (name, value) = line.split_once(':').unwrap();
                    headers.append(
                        reqwest::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                        value.trim().parse().unwrap(),
                    );
                }
                // The forwarding owner buffers R's bounded body and supplies its length.
                assert!(!headers.contains_key("transfer-encoding"));
                let length = headers
                    .get("content-length")
                    .map(|v| v.to_str().unwrap().parse::<usize>().unwrap())
                    .unwrap_or(0);
                while bytes.len() - header_end < length {
                    let mut chunk = [0; 8192];
                    let count = socket.read(&mut chunk).await.unwrap();
                    if count == 0 {
                        return;
                    }
                    bytes.extend_from_slice(&chunk[..count]);
                }
                let response = if path.starts_with("/?token=") {
                    e.lock().unwrap().clone()
                } else if path == "/" {
                    i.lock().unwrap().clone()
                } else {
                    entered.notify_one();
                    a.lock().unwrap().clone()
                };
                r.lock().unwrap().push(RecordedRequest {
                    method,
                    path,
                    headers,
                    body: bytes[header_end..header_end + length].to_vec(),
                });
                if response.hold {
                    std::future::pending::<()>().await;
                }
                let mut head = format!(
                    "HTTP/1.1 {} Fixture\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n",
                    response.status
                );
                for (name, value) in response.headers {
                    head += &format!("{name}: {value}\r\n");
                }
                head += "\r\n";
                if socket.write_all(head.as_bytes()).await.is_err() {
                    continue;
                }
                for chunk in response.chunks.into_iter().filter(|c| !c.is_empty()) {
                    if socket
                        .write_all(format!("{:x}\r\n", chunk.len()).as_bytes())
                        .await
                        .is_err()
                    {
                        break;
                    }
                    if socket.write_all(&chunk).await.is_err() {
                        break;
                    }
                    if socket.write_all(b"\r\n").await.is_err() {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
                let _ = socket.write_all(b"0\r\n\r\n").await;
            }
        });
        Self {
            url: Url::parse(&format!("http://{authority}/?token={TOKEN}")).unwrap(),
            cookie,
            exchange,
            index,
            api,
            requests,
            api_entered,
            server,
        }
    }
    async fn authenticate(&self) -> Result<Authenticated, Failure> {
        authentication::authenticate(self.url.clone(), 7, Instant::now() + Duration::from_secs(2))
            .await
    }
    fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}
fn request(path: &str) -> HttpRequest {
    HttpRequest {
        path: path.into(),
        method: Method::GET,
        headers: HeaderMap::new(),
        body: Body::default(),
    }
}
fn body(bytes: &[u8]) -> Body {
    let mut body = Body::default();
    for (sequence, bytes) in bytes.chunks(65536).enumerate() {
        body.push(sequence as u64, bytes.to_vec()).unwrap();
    }
    body
}
async fn response_bytes(mut reply: deepseek_harness_desktop::http_transport::Reply) -> Vec<u8> {
    let mut bytes = vec![];
    let mut sequence = 0;
    while let Some(chunk) = reply.next_chunk().await.unwrap() {
        assert_eq!(chunk.sequence, sequence);
        sequence += 1;
        bytes.extend_from_slice(&chunk.bytes);
    }
    bytes
}

#[tokio::test]
async fn private_exchange_requires_303_and_yields_only_non_secret_boot() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let boot = serde_json::to_string(&authenticated.boot).unwrap();
    assert!(boot.contains("__DSH_BOOT__"));
    assert!(boot.contains("__DSH_CONNECTION_RECOVERY__"));
    for forbidden in [
        TOKEN,
        &f.cookie,
        "127.0.0.1",
        "UNTRUSTED_INJECTION",
        "not returned as HTML",
        "<script>",
    ] {
        assert!(!boot.contains(forbidden));
    }
    assert!(authenticated.session.is_current(7));
    assert!(!authenticated.session.is_current(8));
    let calls = f.requests.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].method, "GET");
    assert!(!calls[0].headers.contains_key("cookie"));
    assert_eq!(calls[1].path, "/");
    assert_eq!(calls[1].headers["cookie"], f.cookie);
}

#[tokio::test]
async fn unsuccessful_exchange_cannot_follow_any_redirect_or_prepare_boot() {
    for status in [200, 301, 302, 307, 308, 401, 500] {
        let f = Fixture::new().await;
        f.exchange.lock().unwrap().status = status;
        assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
        assert_eq!(f.count(), 1);
    }
    for location in [
        "/redirected",
        "//127.0.0.1:1/",
        "http://127.0.0.1:1/",
        "https://example.invalid/",
    ] {
        let f = Fixture::new().await;
        f.exchange.lock().unwrap().headers[0].1 = location.into();
        assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
        assert_eq!(f.count(), 1);
    }
}

#[tokio::test]
async fn missing_wrong_or_unsafe_cookie_is_rejected_without_boot_request() {
    for attributes in [
        "",
        "; Path=/",
        "; Path=/; HttpOnly",
        "; Path=/; HttpOnly; SameSite=Lax",
        "; Path=/; HttpOnly; SameSite=Strict; Domain=127.0.0.1",
        "; Path=/api; HttpOnly; SameSite=Strict",
        "; Max-Age=0; Path=/; HttpOnly; SameSite=Strict",
    ] {
        let f = Fixture::new().await;
        f.exchange.lock().unwrap().headers[1].1 = format!("{}{attributes}", f.cookie);
        assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
        assert_eq!(f.count(), 1);
    }
    for cookie in [
        None,
        Some("unrelated=opaque; Path=/; HttpOnly; SameSite=Strict"),
        Some("dsh-auth-wrong=opaque; Path=/; HttpOnly; SameSite=Strict"),
    ] {
        let f = Fixture::new().await;
        let mut exchange = f.exchange.lock().unwrap();
        exchange.headers.retain(|(name, _)| name != "set-cookie");
        if let Some(cookie) = cookie {
            exchange.headers.push(("set-cookie".into(), cookie.into()));
        }
        drop(exchange);
        assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
        assert_eq!(f.count(), 1);
    }
}

#[tokio::test]
async fn malformed_authenticated_boot_and_secret_echoes_fail_closed() {
    let f = Fixture::new().await;
    for html in [
        "<html>no boot injection</html>".to_owned(),
        BOOT.replace(r#"{"entries":[],"batches":[]}"#, "not-json()"),
        BOOT.replace(
            r#"{"entries":[],"batches":[]}"#,
            &format!(r#"{{"debug":"{TOKEN}"}}"#),
        ),
        BOOT.replace(
            r#"{"entries":[],"batches":[]}"#,
            &format!(r#"{{"debug":"{}"}}"#, f.cookie),
        ),
        BOOT.replace(
            r#"{"entries":[],"batches":[]}"#,
            &format!(r#"{{"debug":"{}"}}"#, f.url.origin().ascii_serialization()),
        ),
    ] {
        f.index.lock().unwrap().chunks = vec![html.into_bytes()];
        assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
    }
    f.index.lock().unwrap().status = 401;
    assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
}

#[tokio::test]
async fn authentication_uses_the_existing_startup_deadline() {
    let f = Fixture::new().await;
    f.index.lock().unwrap().hold = true;
    let result =
        authentication::authenticate(f.url.clone(), 7, Instant::now() + Duration::from_millis(50))
            .await;
    assert_eq!(result.err(), Some(Failure::Timeout));
    assert_eq!(f.count(), 2);
}

#[tokio::test]
async fn destination_and_token_path_rejection_happens_before_dispatch() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let transport = Transport::new(authenticated.session);
    let (_cancel, signal) = watch::channel(false);
    for path in [
        "http://127.0.0.1:1/api",
        "http://127.0.0.1/api",
        "https://example.invalid/api",
        "//example.invalid/api",
        "///api",
        "\\\\example.invalid\\api",
        "/api\\escape",
        "/../api",
        "/api/../secret",
        "/api/%2e%2e/secret",
        "/api/%252e%252e/secret",
        "/api/%2f%2fexample.invalid",
        "/api/%5cescape",
        "/api/%00",
        "/api/%",
        "/api/%GG",
        "/api\r\ninjected",
        "/api#fragment",
        "/?token=fixture",
        "/api?%74oken=fixture",
        "/api?token",
    ] {
        assert!(
            transport
                .fetch(7, request(path), signal.clone())
                .await
                .is_err()
        );
    }
    for method in [Method::CONNECT, Method::TRACE] {
        let mut input = request("/api/fixture");
        input.method = method;
        assert!(transport.fetch(7, input, signal.clone()).await.is_err());
    }
    assert_eq!(f.count(), 2);
}

#[tokio::test]
async fn auth_trust_and_connection_headers_are_not_caller_controlled() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let transport = Transport::new(authenticated.session);
    let (_cancel, signal) = watch::channel(false);
    let mut input = request("/api/fixture?q=preserved");
    for (name, value) in [
        ("host", "example.invalid"),
        ("origin", "https://example.invalid"),
        ("cookie", "renderer-cookie"),
        ("authorization", "renderer-auth"),
        ("proxy-authorization", "renderer-proxy-auth"),
        ("sec-fetch-site", "cross-site"),
        ("sec-fetch-mode", "navigate"),
        ("forwarded", "host=example.invalid"),
        ("x-forwarded-host", "example.invalid"),
        ("referer", "https://example.invalid/"),
        ("connection", "x-hop, keep-alive"),
        ("x-hop", "must-strip"),
        ("keep-alive", "timeout=20"),
        ("upgrade", "websocket"),
        ("te", "trailers"),
        ("trailer", "x-trailer"),
        ("transfer-encoding", "chunked"),
        ("x-fixture", "preserve"),
    ] {
        input.headers.insert(
            reqwest::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }
    let reply = transport.fetch(7, input, signal).await.unwrap();
    assert_eq!(response_bytes(reply).await, b"opaque response");
    let calls = f.requests.lock().unwrap();
    let headers = &calls[2].headers;
    assert_eq!(
        headers["host"],
        f.url.host_str().unwrap().to_owned() + ":" + &f.url.port().unwrap().to_string()
    );
    assert_eq!(headers["cookie"], f.cookie);
    assert_eq!(headers["origin"], f.url.origin().ascii_serialization());
    assert_eq!(headers["x-fixture"], "preserve");
    assert_eq!(calls[2].path, "/api/fixture?q=preserved");
    for name in [
        "authorization",
        "proxy-authorization",
        "sec-fetch-site",
        "sec-fetch-mode",
        "forwarded",
        "x-forwarded-host",
        "referer",
        "x-hop",
        "keep-alive",
        "upgrade",
        "te",
        "trailer",
    ] {
        assert!(!headers.contains_key(name));
    }
    // Invalid header syntax is rejected by the selected HTTP parser, not repaired.
    assert!(reqwest::header::HeaderValue::from_bytes(b"value\r\ninjected: true").is_err());
}

#[tokio::test]
async fn buffered_input_enforces_the_exact_300_mib_default_and_chunk_order() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let transport = Transport::new(authenticated.session);
    let (_cancel, signal) = watch::channel(false);
    assert_eq!(MAX_BODY_BYTES, 314572800);
    let mut reordered = Body::default();
    assert!(reordered.push(1, vec![0]).is_err());
    let mut input = Body::default();
    for sequence in 0..4800 {
        input.push(sequence, vec![0; 65536]).unwrap();
    }
    assert_eq!(input.byte_len(), MAX_BODY_BYTES);
    assert!(input.push(4800, vec![0]).is_err());
    assert_eq!(input.byte_len(), MAX_BODY_BYTES);
    let mut oversized = request("/api/fixture");
    oversized.method = Method::POST;
    oversized.body = input;
    assert!(transport.fetch(7, oversized, signal).await.is_err());
    assert_eq!(f.count(), 2);
    let mut duplicate = Body::default();
    duplicate.push(0, vec![1]).unwrap();
    assert!(duplicate.push(0, vec![2]).is_err());
    assert_eq!(duplicate.byte_len(), 1);
}

#[tokio::test]
async fn binary_multipart_and_ordered_response_bytes_are_unchanged() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let transport = Transport::new(authenticated.session);
    let (_cancel, signal) = watch::channel(false);
    let binary: Vec<u8> = (0..=255).cycle().take(131089).collect();
    let mut payload = b"--fixture-boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"binary.dat\"\r\nContent-Type: application/octet-stream\r\n\r\n".to_vec();
    payload.extend_from_slice(&binary);
    payload.extend_from_slice(b"\r\n--fixture-boundary--\r\n");
    f.api.lock().unwrap().chunks = binary.chunks(997).map(<[u8]>::to_vec).collect();
    let mut input = request("api/fixture-upload");
    input.method = Method::POST;
    input.headers.insert(
        "content-type",
        "multipart/form-data; boundary=fixture-boundary"
            .parse()
            .unwrap(),
    );
    input.body = body(&payload);
    let reply = transport.fetch(7, input, signal).await.unwrap();
    assert_eq!(response_bytes(reply).await, binary);
    let calls = f.requests.lock().unwrap();
    assert_eq!(calls[2].method, "POST");
    assert_eq!(calls[2].path, "/api/fixture-upload");
    assert_eq!(
        calls[2].headers["content-type"],
        "multipart/form-data; boundary=fixture-boundary"
    );
    assert_eq!(calls[2].body, payload);
}

#[tokio::test]
async fn service_redirects_fail_closed_and_response_auth_is_stripped() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let transport = Transport::new(authenticated.session);
    let (_cancel, signal) = watch::channel(false);
    for status in [301, 302, 303, 307, 308] {
        f.api.lock().unwrap().status = status;
        f.api.lock().unwrap().headers = vec![("location".into(), "/redirected".into())];
        assert!(
            transport
                .fetch(7, request("/api/fixture"), signal.clone())
                .await
                .is_err()
        );
    }
    assert_eq!(f.count(), 7);
    let mut api = f.api.lock().unwrap();
    api.status = 200;
    api.headers = vec![
        ("set-cookie".into(), f.cookie.clone()),
        ("connection".into(), "x-hop".into()),
        ("x-hop".into(), "private".into()),
        ("x-fixture".into(), "preserve".into()),
    ];
    drop(api);
    let reply = transport
        .fetch(7, request("/api/fixture"), signal)
        .await
        .unwrap();
    assert_eq!(reply.headers["x-fixture"], "preserve");
    for name in ["set-cookie", "connection", "transfer-encoding", "x-hop"] {
        assert!(!reply.headers.contains_key(name));
    }
    assert_eq!(response_bytes(reply).await, b"opaque response");
}

#[tokio::test]
async fn cancellation_before_dispatch_and_during_headers_does_not_replay() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let transport = Transport::new(authenticated.session);
    let (cancel, signal) = watch::channel(true);
    assert!(
        transport
            .fetch(7, request("/api/fixture"), signal.clone())
            .await
            .is_err()
    );
    assert_eq!(f.count(), 2);
    cancel.send(false).unwrap();
    f.api.lock().unwrap().hold = true;
    let fetch = transport.fetch(7, request("/api/fixture"), signal);
    tokio::pin!(fetch);
    tokio::select! {
        _ = f.api_entered.notified() => {},
        _ = &mut fetch => panic!("Held response completed before cancellation."),
        _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("Forwarding did not reach fixture."),
    }
    cancel.send(true).unwrap();
    assert!(
        timeout(Duration::from_secs(1), &mut fetch)
            .await
            .unwrap()
            .is_err()
    );
    assert_eq!(f.count(), 3);
}

#[tokio::test]
async fn cancellation_and_generation_revocation_deny_already_queued_replies() {
    let f = Fixture::new().await;
    let authenticated = f.authenticate().await.unwrap();
    let transport = Transport::new(authenticated.session.clone());
    let (cancel, signal) = watch::channel(false);
    let mut reply = transport
        .fetch(7, request("/api/fixture"), signal.clone())
        .await
        .unwrap();
    cancel.send(true).unwrap();
    assert!(reply.next_chunk().await.is_err());
    cancel.send(false).unwrap();
    let mut reply = transport
        .fetch(7, request("/api/fixture"), signal.clone())
        .await
        .unwrap();
    authenticated.session.invalidate();
    assert!(!authenticated.session.is_current(7));
    assert!(reply.next_chunk().await.is_err());
    assert!(
        transport
            .fetch(7, request("/api/fixture"), signal.clone())
            .await
            .is_err()
    );
    assert!(
        transport
            .fetch(8, request("/api/fixture"), signal)
            .await
            .is_err()
    );
    assert_eq!(f.count(), 4);
}

#![allow(dead_code)] // Shared by auth-only and HTTP contract executables.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use deepseek_harness_desktop::{
    authentication::{self, Authenticated},
    errors::Failure,
};
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Notify,
    task::JoinHandle,
    time::Instant,
};
use url::Url;

pub const TOKEN: &str = "disposable-transport-fixture-not-a-real-token";
pub const BOOT: &str = r#"<html><head><script>globalThis["__DSH_BOOT__"] = {"entries":[],"batches":[]}</script><script>globalThis["__DSH_CONNECTION_RECOVERY__"] = {"retryDelays":[0,100]}</script><script>globalThis["UNTRUSTED_INJECTION"] = "discard-me"</script></head><body>not returned as HTML</body></html>"#;

#[derive(Clone)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub chunks: Vec<Vec<u8>>,
    pub hold: bool,
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
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}
pub struct Fixture {
    pub url: Url,
    pub cookie: String,
    pub exchange: Arc<Mutex<Response>>,
    pub index: Arc<Mutex<Response>>,
    pub api: Arc<Mutex<Response>>,
    pub requests: Arc<Mutex<Vec<RecordedRequest>>>,
    pub api_entered: Arc<Notify>,
    server: JoinHandle<()>,
}
impl Fixture {
    pub async fn new() -> Self {
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
    pub async fn authenticate(&self) -> Result<Authenticated, Failure> {
        authentication::authenticate(self.url.clone(), 7, Instant::now() + Duration::from_secs(2))
            .await
    }
    pub fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

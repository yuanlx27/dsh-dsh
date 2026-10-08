//! T016 HTTP contracts; authentication is exercised separately in authentication.rs.
#[path = "transport/fixture.rs"]
mod fixture;
use deepseek_harness_desktop::http_transport::{Body, HttpRequest, MAX_BODY_BYTES, Transport};
use fixture::Fixture;
use reqwest::{Method, header::HeaderMap};
use std::time::Duration;
use tokio::{sync::watch, time::timeout};

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

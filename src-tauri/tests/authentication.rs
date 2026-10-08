//! T016 authentication subset, runnable before T025 exists.
#[path = "transport/fixture.rs"]
mod fixture;
use deepseek_harness_desktop::{authentication, errors::Failure};
use fixture::{BOOT, Fixture, TOKEN};
use std::time::Duration;
use tokio::time::Instant;

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
        {
            let mut exchange = f.exchange.lock().unwrap();
            exchange.headers.retain(|(name, _)| name != "set-cookie");
            if let Some(cookie) = cookie {
                exchange.headers.push(("set-cookie".into(), cookie.into()));
            }
        }
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

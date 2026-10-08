//! Additional bounded-auth/security regressions, before implementing T020.
#[path = "transport/fixture.rs"]
mod fixture;
use deepseek_harness_desktop::{authentication, errors::Failure};
use fixture::{BOOT, Fixture, TOKEN};
use std::{process::Command, time::Duration};
use tokio::time::Instant;
use url::Url;

#[tokio::test]
async fn invalid_auth_destinations_and_expired_deadlines_never_dispatch() {
    let f = Fixture::new().await;
    for raw in [
        "http://localhost:54321/?token=fixture",
        "https://127.0.0.1:54321/?token=fixture",
        "http://192.168.1.2:54321/?token=fixture",
        "http://user:pass@127.0.0.1:54321/?token=fixture",
        "http://127.0.0.1:0/?token=fixture",
        "http://127.0.0.1:54321/other?token=fixture",
        "http://127.0.0.1:54321/?token=fixture&token=other",
        "http://127.0.0.1:54321/?token=fixture#fragment",
    ] {
        assert_eq!(
            authentication::authenticate(
                Url::parse(raw).unwrap(),
                7,
                Instant::now() + Duration::from_secs(1)
            )
            .await
            .err(),
            Some(Failure::Connection)
        );
    }
    assert_eq!(
        authentication::authenticate(f.url.clone(), 7, Instant::now() - Duration::from_secs(1))
            .await
            .err(),
        Some(Failure::Timeout)
    );
    assert_eq!(f.count(), 0);
}

#[tokio::test]
async fn duplicate_cookies_and_invalid_opaque_values_fail_before_boot() {
    for suffix in [
        "; HttpOnly=wrong",
        "; Domain=localhost",
        "; Path=/api",
        "; Max-Age=-1",
    ] {
        let f = Fixture::new().await;
        f.exchange.lock().unwrap().headers[1].1.push_str(suffix);
        assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
        assert_eq!(f.count(), 1);
    }
    let f = Fixture::new().await;
    let second = f.exchange.lock().unwrap().headers[1].clone();
    f.exchange.lock().unwrap().headers.push(second);
    assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
    assert_eq!(f.count(), 1);
    let f = Fixture::new().await;
    let name = f.cookie.split_once('=').unwrap().0;
    f.exchange.lock().unwrap().headers[1].1 =
        format!("{name}=v1.bad.unsigned; Max-Age=30; Path=/; HttpOnly; SameSite=Strict");
    assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
    assert_eq!(f.count(), 1);
}

#[tokio::test]
async fn boot_redirects_duplicates_oversize_and_encoded_secret_echoes_fail_closed() {
    for html in [
        BOOT.repeat(2),
        format!("{BOOT}{}", " ".repeat(2 * 1024 * 1024)),
        BOOT.replace(
            r#"{"entries":[],"batches":[]}"#,
            &format!(r#"{{"debug":"{}"}}"#, TOKEN.replacen('d', "\\u0064", 1)),
        ),
    ] {
        let f = Fixture::new().await;
        f.index.lock().unwrap().chunks = vec![html.into_bytes()];
        assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
        assert_eq!(f.count(), 2);
    }
    let f = Fixture::new().await;
    f.index.lock().unwrap().status = 302;
    f.index
        .lock()
        .unwrap()
        .headers
        .push(("location".into(), "/redirected".into()));
    assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
    assert_eq!(f.count(), 2);
    let f = Fixture::new().await;
    let value = f.cookie.split_once('=').unwrap().1;
    f.index.lock().unwrap().chunks = vec![
        BOOT.replace(
            r#"{"entries":[],"batches":[]}"#,
            &format!(r#"{{"debug":"{value}"}}"#),
        )
        .into_bytes(),
    ];
    assert_eq!(f.authenticate().await.err(), Some(Failure::Connection));
}

#[tokio::test]
async fn all_pinned_public_globals_survive_without_html_or_executable_injections() {
    let f = Fixture::new().await;
    let public = [
        "__DSH_MODELS_ONBOARDING__",
        "__DSH_CONTACT_CONFIG__",
        "__DSH_SHORTCUTS_CONFIG__",
        "__DSH_DOCUMENT_PREVIEW_CONFIG__",
    ];
    let mut html = BOOT.to_owned();
    for name in public {
        html.push_str(&format!(
            r#"<script>globalThis["{name}"] = {{"fixture":true}}</script>"#
        ));
    }
    f.index.lock().unwrap().chunks = vec![html.into_bytes()];
    let authenticated = f.authenticate().await.unwrap();
    let boot = serde_json::to_value(authenticated.boot).unwrap();
    for name in public {
        assert_eq!(boot["globals"][name]["fixture"], true);
    }
    authenticated.session.invalidate();
    authenticated.session.invalidate();
    assert!(!authenticated.session.is_current(7));
}

#[tokio::test]
async fn native_client_ignores_inherited_proxy_without_mutating_process_environment() {
    if std::env::var_os("DSH_AUTH_PROXY_TEST_CHILD").is_some() {
        let f = Fixture::new().await;
        assert!(f.authenticate().await.is_ok());
        assert_eq!(f.count(), 2);
        return;
    }
    // A separate test process avoids unsafe/global environment mutation. A bad
    // proxy targets loopback only; this test cannot leak the fixture to the LAN.
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "native_client_ignores_inherited_proxy_without_mutating_process_environment",
        ])
        .env("DSH_AUTH_PROXY_TEST_CHILD", "1")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("http_proxy", "http://127.0.0.1:1")
        .env("ALL_PROXY", "http://127.0.0.1:1")
        .env("all_proxy", "http://127.0.0.1:1")
        .env("NO_PROXY", "")
        .env("no_proxy", "")
        .status()
        .unwrap();
    assert!(status.success());
}

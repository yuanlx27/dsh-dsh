use super::*;
use std::{fs, os::unix::fs::PermissionsExt};

fn fixture(script: &str) -> (tempfile::TempDir, Runtime) {
    let root = tempfile::tempdir().unwrap();
    let launcher = root.path().join("helper-launcher.sh");
    fs::write(
        &launcher,
        format!("#!/bin/sh\necho $$ >> launch.count\n{script}\n"),
    )
    .unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
    let config = LaunchConfig {
        node: launcher.clone(),
        cli: launcher.clone(),
        overlay: launcher.clone(),
        launcher,
        home: root.path().to_owned(),
        cwd: root.path().to_owned(),
    };
    (root, Runtime::new(config))
}

#[tokio::test]
async fn concurrent_start_joins_one_generation_and_authentication_is_separate() {
    let (root, runtime) =
        fixture("echo 'dsh web: http://127.0.0.1:54321/?token=fixture'\nIFS= read -r stop\nexit 0");
    let (a, b) = tokio::join!(runtime.start(), runtime.start());
    let a = a.unwrap();
    let b = b.unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(runtime.state(), State::Starting(a.generation));
    assert_eq!(
        fs::read_to_string(root.path().join("launch.count"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert_eq!(a.take_token_url().unwrap().query(), Some("token=fixture"));
    assert!(b.take_token_url().is_err());
    assert_eq!(
        runtime.complete(a.generation + 1).await,
        Err(Failure::Unavailable)
    );
    runtime.complete(a.generation).await.unwrap();
    assert_eq!(runtime.state(), State::Ready(a.generation));
    runtime.stop().await.unwrap();
    assert_eq!(runtime.state(), State::Stopped);
    let next = runtime.start().await.unwrap();
    assert!(next.generation > a.generation);
    runtime.stop().await.unwrap();
    assert!(next.take_token_url().is_err());
}

#[tokio::test]
async fn early_exit_does_not_automatically_restart_or_retain_the_token() {
    let (root, runtime) =
        fixture("echo 'dsh web: http://127.0.0.1:54321/?token=fixture'\nsleep 0.1\nexit 2");
    let startup = runtime.start().await.unwrap();
    let mut state = runtime.subscribe();
    tokio::time::timeout(Duration::from_secs(2), async {
        while !matches!(*state.borrow_and_update(), State::Failed(_, _)) {
            state.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert_eq!(
        runtime.state(),
        State::Failed(startup.generation, Failure::Exited)
    );
    assert!(startup.take_token_url().is_err());
    assert_eq!(
        fs::read_to_string(root.path().join("launch.count"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

#[test]
fn inherited_injection_and_model_environment_is_not_the_command_environment() {
    for name in [
        "NODE_OPTIONS",
        "NODE_PATH",
        "NODE_EXTRA_CA_CERTS",
        "DEEPSEEK_API_KEY",
        "OPENAI_API_KEY",
        "GEMINI_API_KEY",
        "DSH_TELEMETRY_ENDPOINT",
        "OTEL_EXPORTER_OTLP_ENDPOINT",
    ] {
        assert!(excluded_environment(OsStr::new(name)), "{name}");
    }
    for name in [
        "PATH",
        "HOME",
        "SHELL",
        "LANG",
        "TERM",
        "SSH_AUTH_SOCK",
        "GITHUB_TOKEN",
        "TMPDIR",
    ] {
        assert!(!excluded_environment(OsStr::new(name)), "{name}");
    }
}

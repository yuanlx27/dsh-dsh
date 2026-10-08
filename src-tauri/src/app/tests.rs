use super::*;
#[path = "../../tests/transport/fixture.rs"]
mod io_fixture;
use crate::runtime::tests::fixture;
#[tokio::test]
async fn joined_start_is_ready_after_auth_and_preparation_but_entry_needs_saved_notice() {
    let io = io_fixture::Fixture::new().await;
    let (root, runtime) = fixture(&format!(
        "echo 'dsh web: {}'\nIFS= read -r stop\nexit 0",
        io.url
    ));
    let app = Application::new(
        runtime.clone(),
        Arc::new(Bridge::new()),
        root.path().into(),
        Arc::new(|boot| Ok(Protocol::fixture(boot))),
    )
    .unwrap();
    let (a, b) = tokio::join!(app.start(), app.start());
    a.unwrap();
    b.unwrap();
    assert!(matches!(runtime.state(), State::Ready(1)));
    assert!(app.enter().is_err());
    assert_eq!(app.status().phase, "available");
    app.acknowledge(NOTICE_REVISION).unwrap();
    assert_eq!(app.enter().unwrap(), 1);
    assert_eq!(
        std::fs::read_to_string(root.path().join("launch.count"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    let protocol = app.protocol().unwrap();
    assert!(!protocol.bootstrap().globals.is_empty());
    app.shutdown().await.unwrap();
    assert!(app.enter().is_err());
    assert!(app.protocol().is_none());
}
#[tokio::test]
async fn preparation_failure_cleans_owned_service_without_granting_entry_or_retrying() {
    let io = io_fixture::Fixture::new().await;
    let (root, runtime) = fixture(&format!(
        "echo 'dsh web: {}'\nIFS= read -r stop\nexit 0",
        io.url
    ));
    acknowledge_notice(root.path(), NOTICE_REVISION).unwrap();
    let app = Application::new(
        runtime.clone(),
        Arc::new(Bridge::new()),
        root.path().into(),
        Arc::new(|_| Err(Failure::InvalidBundle)),
    )
    .unwrap();
    assert_eq!(app.start().await, Err(Failure::InvalidBundle));
    assert!(app.enter().is_err());
    assert!(app.protocol().is_none());
    assert_eq!(app.status().failure, Some("invalidBundle"));
    assert_eq!(
        std::fs::read_to_string(root.path().join("launch.count"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}
#[test]
fn native_notice_matches_ts_and_bad_save_does_not_replace_bounds_or_grant_notice() {
    assert!(
        include_str!("../../../src/shell/state.ts")
            .contains(&format!("NOTICE_REVISION = \"{NOTICE_REVISION}\""))
    );
    let root = tempfile::tempdir().unwrap();
    assert!(!notice_acknowledged(root.path()).unwrap());
    assert!(acknowledge_notice(root.path(), "old").is_err());
    let prefs = preferences::Preferences {
        window_bounds: Some(preferences::Bounds {
            x: 1.,
            y: 2.,
            width: 800.,
            height: 600.,
        }),
        ..Default::default()
    };
    preferences::save(root.path(), &prefs).unwrap();
    acknowledge_notice(root.path(), NOTICE_REVISION).unwrap();
    assert_eq!(
        preferences::load(root.path()).unwrap().window_bounds,
        prefs.window_bounds
    );
    let denied = tempfile::tempdir().unwrap();
    std::fs::create_dir(denied.path().join("preferences.json")).unwrap();
    assert!(acknowledge_notice(denied.path(), NOTICE_REVISION).is_err());
    assert!(notice_acknowledged(denied.path()).is_err());
}

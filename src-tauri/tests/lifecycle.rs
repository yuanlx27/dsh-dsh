//! T018: shell-owned transition contracts, not native DOM/dialog qualification.
//! Identities here are native-side fixture IDs, never renderer admission labels.
use deepseek_harness_desktop::{
    app::{NOTICE_REVISION, acknowledge_notice, notice_acknowledged},
    errors::Failure,
    preferences::{self, Preferences},
    quit::{QuitAction, QuitDecision, QuitResponse},
    window::{WindowAction, WindowState},
};
use std::fs;

#[test]
fn only_the_current_successfully_saved_notice_revision_grants_entry() {
    let root = tempfile::tempdir().unwrap();
    assert!(!notice_acknowledged(root.path()).unwrap());
    let old = Preferences {
        safety_notice_revision: Some("obsolete-fixture-notice".into()),
        ..Preferences::default()
    };
    preferences::save(root.path(), &old).unwrap();
    assert!(!notice_acknowledged(root.path()).unwrap());
    assert!(acknowledge_notice(root.path(), "obsolete-fixture-notice").is_err());
    assert!(!notice_acknowledged(root.path()).unwrap());
    acknowledge_notice(root.path(), NOTICE_REVISION).unwrap();
    assert!(notice_acknowledged(root.path()).unwrap());
    assert_eq!(
        preferences::load(root.path())
            .unwrap()
            .safety_notice_revision
            .as_deref(),
        Some(NOTICE_REVISION)
    );
    // A new invocation reads persisted shell preferences, not a process-only flag.
    assert!(notice_acknowledged(root.path()).unwrap());
}

#[test]
fn notice_write_failure_cannot_grant_entry_or_replace_prior_preferences() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("preferences.json");
    fs::create_dir(&file).unwrap();
    assert!(acknowledge_notice(root.path(), NOTICE_REVISION).is_err());
    assert!(file.is_dir());
    assert!(notice_acknowledged(root.path()).is_err());
    fs::remove_dir(&file).unwrap();
    assert!(!notice_acknowledged(root.path()).unwrap());
    assert!(acknowledge_notice(root.path(), "obsolete-fixture-notice").is_err());
    assert!(!file.exists());
}

#[test]
fn notice_acknowledgement_preserves_bounds_and_stores_no_harness_state() {
    let root = tempfile::tempdir().unwrap();
    let original = Preferences {
        window_bounds: Some(preferences::Bounds {
            x: 50.0,
            y: 50.0,
            width: 800.0,
            height: 600.0,
        }),
        ..Preferences::default()
    };
    preferences::save(root.path(), &original).unwrap();
    acknowledge_notice(root.path(), NOTICE_REVISION).unwrap();
    let saved = preferences::load(root.path()).unwrap();
    assert_eq!(saved.window_bounds, original.window_bounds);
    let json = serde_json::to_value(saved).unwrap();
    let object = json.as_object().unwrap();
    assert_eq!(object.len(), 3);
    for field in [
        "cookie",
        "token",
        "workspace",
        "session",
        "task",
        "credential",
        "draft",
        "scroll",
    ] {
        assert!(!object.contains_key(field));
    }
}

#[test]
fn close_and_repeated_open_retain_one_native_document_identity() {
    let mut window = WindowState::default();
    assert_eq!(window.open(), WindowAction::Create);
    window.opened(41);
    for _ in 0..10 {
        assert_eq!(window.open(), WindowAction::Show(41));
        assert_eq!(window.close(), WindowAction::Hide(41));
        assert_eq!(window.close(), WindowAction::Hide(41));
        assert_eq!(window.open(), WindowAction::Show(41));
    }
    // The policy has no runtime start/stop, task, draft or history operation.
    // Actual retained WKWebView/draft/scroll/transfer identity is T032/T033.
}

#[test]
fn recreate_only_after_absence_or_page_failure_and_ignore_late_old_events() {
    let mut window = WindowState::default();
    window.opened(41);
    window.failed(41);
    assert_eq!(window.open(), WindowAction::Create);
    window.opened(42);
    window.failed(41);
    window.destroyed(41);
    assert_eq!(window.open(), WindowAction::Show(42));
    window.destroyed(42);
    assert_eq!(window.open(), WindowAction::Create);
    assert_eq!(window.close(), WindowAction::None);
}

fn confirming(decision: &mut QuitDecision) -> u64 {
    match decision.request(true) {
        QuitAction::Confirm(id) => id,
        other => panic!("Expected one live-service confirmation, got {other:?}."),
    }
}

#[test]
fn repeated_menu_shortcut_or_ownerless_quit_requests_join_one_decision() {
    let mut decision = QuitDecision::default();
    let id = confirming(&mut decision);
    for _ in 0..20 {
        assert_eq!(decision.request(true), QuitAction::Focus(id));
    }
    assert_eq!(decision.respond(id, QuitResponse::Stay), QuitAction::None);
    let next = confirming(&mut decision);
    assert_ne!(id, next);
    assert_eq!(
        decision.respond(id, QuitResponse::StopAndQuit),
        QuitAction::None
    );
    assert_eq!(decision.request(true), QuitAction::Focus(next));
}

#[test]
fn stay_dismissal_or_dialog_failure_preserves_the_live_service() {
    for response in [
        QuitResponse::Stay,
        QuitResponse::Dismissed,
        QuitResponse::Failed,
    ] {
        let mut decision = QuitDecision::default();
        let id = confirming(&mut decision);
        assert_eq!(decision.respond(id, response), QuitAction::None);
        assert_eq!(decision.shutdown_result(Ok(())), QuitAction::None);
        assert!(matches!(decision.request(true), QuitAction::Confirm(_)));
    }
}

#[test]
fn live_hidden_idle_active_approval_and_unknown_work_share_the_same_confirmation() {
    // Work state/visible session are deliberately not inputs to the policy.
    for _scenario in ["hidden", "idle", "active", "pending-approval", "unknown"] {
        let mut decision = QuitDecision::default();
        let id = confirming(&mut decision);
        assert_eq!(
            decision.respond(id, QuitResponse::StopAndQuit),
            QuitAction::StopOwned
        );
        assert_eq!(decision.request(true), QuitAction::None);
        assert_eq!(decision.respond(id, QuitResponse::Stay), QuitAction::None);
        assert_eq!(
            decision.respond(id, QuitResponse::StopAndQuit),
            QuitAction::None
        );
        // Confirming is not exiting. Only actual owned-shutdown success permits exit.
        assert_eq!(decision.shutdown_result(Ok(())), QuitAction::Exit);
        assert_eq!(decision.shutdown_result(Ok(())), QuitAction::None);
    }
}

#[test]
fn failed_owned_cleanup_never_reports_exit_and_requires_an_explicit_new_decision() {
    let mut decision = QuitDecision::default();
    let id = confirming(&mut decision);
    assert_eq!(
        decision.respond(id, QuitResponse::StopAndQuit),
        QuitAction::StopOwned
    );
    assert_eq!(
        decision.shutdown_result(Err(Failure::Cleanup)),
        QuitAction::None
    );
    let next = confirming(&mut decision);
    assert_ne!(id, next);
    assert_eq!(
        decision.respond(id, QuitResponse::StopAndQuit),
        QuitAction::None
    );
    assert_eq!(decision.request(true), QuitAction::Focus(next));
}

#[test]
fn no_service_can_exit_without_inspecting_tasks_or_opening_a_dialog() {
    let mut decision = QuitDecision::default();
    assert_eq!(decision.request(false), QuitAction::Exit);
}

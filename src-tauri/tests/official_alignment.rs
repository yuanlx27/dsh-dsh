//! A3/A4 core/source contracts, not native event-loop or installed dialog evidence.
//! Always-confirm live/unknown owned service and native Close Window are approved
//! adaptations; official contextual Close Page / inspected-idle parity is not claimed.
use deepseek_harness_desktop::{
    errors::Failure,
    menu::{Command, command},
    quit::{QuitAction, QuitDecision, QuitResponse, service_alive},
    runtime::State,
};

#[test]
fn menu_and_accelerator_share_command_items_without_contextual_page_close() {
    let source = include_str!("../src/menu.rs");
    let close = source
        .split("let close =")
        .nth(1)
        .unwrap()
        .split("let quit =")
        .next()
        .unwrap();
    let quit = source
        .split("let quit =")
        .nth(1)
        .unwrap()
        .split("let open =")
        .next()
        .unwrap();
    assert!(close.contains("\"desktop.close\"") && close.contains("\"Cmd+W\""));
    assert!(quit.contains("\"desktop.quit\"") && quit.contains("\"Cmd+Q\""));
    assert_eq!(command("desktop.close"), Some(Command::Close));
    assert_eq!(command("desktop.quit"), Some(Command::Quit));
    assert_eq!(command("page.close"), None);
    assert!(source.contains("match command(id)"));
}

#[test]
fn late_dialog_and_shutdown_after_disposal_never_authorize_exit() {
    let mut quit = QuitDecision::default();
    let QuitAction::Confirm(id) = quit.request(true) else {
        panic!("Expected confirmation")
    };
    quit.dispose();
    assert_eq!(
        quit.respond(id, QuitResponse::StopAndQuit),
        QuitAction::None
    );
    assert_eq!(quit.shutdown_result(Ok(())), QuitAction::None);
    assert_eq!(quit.request(false), QuitAction::None);
    assert!(!quit.is_exiting());
}

#[test]
fn proven_service_exit_before_quit_differs_from_unconfirmed_cleanup() {
    let mut gone = QuitDecision::default();
    assert!(!service_alive(State::Failed(1, Failure::Exited)));
    assert_eq!(
        gone.request(service_alive(State::Failed(1, Failure::Exited))),
        QuitAction::Exit
    );
    let mut unknown = QuitDecision::default();
    assert!(service_alive(State::Failed(1, Failure::Cleanup)));
    assert!(matches!(
        unknown.request(service_alive(State::Failed(1, Failure::Cleanup))),
        QuitAction::Confirm(_)
    ));
}

#[test]
fn service_exit_does_not_turn_a_pending_dialog_into_an_implicit_grant() {
    let mut quit = QuitDecision::default();
    let QuitAction::Confirm(id) = quit.request(true) else {
        panic!("Expected confirmation")
    };
    assert_eq!(quit.request(false), QuitAction::Focus(id));
    assert_eq!(quit.respond(id, QuitResponse::Dismissed), QuitAction::None);
    assert!(!quit.is_exiting());
    assert_eq!(quit.request(false), QuitAction::Exit);
}

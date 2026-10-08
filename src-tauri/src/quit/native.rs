use super::{QuitAction, QuitDecision, QuitResponse, service_alive};
use crate::{app::Application, window::native::Primary};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::{NSAlert, NSAlertSecondButtonReturn, NSAlertStyle, NSApplication};
use objc2_foundation::NSString;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};
use tauri::AppHandle;
thread_local! {static ALERT:RefCell<Option<(u64,Retained<NSAlert>)>>=const {RefCell::new(None)};}
const WARNING: &str = "The local service may be working in any session or workspace, including one not visible here. Its work state is unknown. Stopping it interrupts active work and pending approvals across all sessions. This cannot be undone. Stay keeps the service running.";
pub struct Quit {
    decision: Mutex<QuitDecision>,
    application: Arc<Application>,
    primary: Arc<Primary>,
    handle: AppHandle,
}
impl Quit {
    pub fn new(
        application: Arc<Application>,
        primary: Arc<Primary>,
        handle: AppHandle,
    ) -> Arc<Self> {
        Arc::new(Self {
            decision: Mutex::new(QuitDecision::default()),
            application,
            primary,
            handle,
        })
    }
    pub fn is_exiting(&self) -> bool {
        self.decision.lock().is_ok_and(|s| s.is_exiting())
    }
    pub fn request(self: &Arc<Self>) {
        let alive = service_alive(self.application.runtime().state());
        let action = self
            .decision
            .lock()
            .map_or(QuitAction::None, |mut s| s.request(alive));
        self.perform(action);
    }
    fn respond(self: &Arc<Self>, id: u64, response: QuitResponse) {
        let action = self
            .decision
            .lock()
            .map_or(QuitAction::None, |mut s| s.respond(id, response));
        self.perform(action);
    }
    fn perform(self: &Arc<Self>, action: QuitAction) {
        match action {
            QuitAction::Confirm(id) => {
                let this = self.clone();
                if self
                    .handle
                    .run_on_main_thread(move || {
                        let response =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| show(id)))
                                .unwrap_or(QuitResponse::Failed);
                        this.respond(id, response);
                    })
                    .is_err()
                {
                    self.respond(id, QuitResponse::Failed);
                }
            }
            QuitAction::Focus(id) => {
                let _ = self.handle.run_on_main_thread(move || focus(id));
            }
            QuitAction::StopOwned => {
                let this = self.clone();
                self.primary.save_bounds(&self.handle);
                tauri::async_runtime::spawn(async move {
                    let result = this.application.shutdown().await;
                    let action = this
                        .decision
                        .lock()
                        .map_or(QuitAction::None, |mut d| d.shutdown_result(result));
                    this.perform(action);
                });
            }
            QuitAction::Exit => {
                if MainThreadMarker::new().is_some() {
                    self.primary.save_bounds(&self.handle);
                }
                self.application.dispose();
                self.handle.exit(0);
            }
            QuitAction::None => {}
        }
    }
    pub fn dispose(&self) {
        if let Ok(mut decision) = self.decision.lock() {
            decision.dispose();
        }
    }
}
fn focus(id: u64) {
    let Some(main) = MainThreadMarker::new() else {
        return;
    };
    NSApplication::sharedApplication(main).activate();
    ALERT.with(|slot| {
        if let Some((current, alert)) = &*slot.borrow()
            && *current == id
        {
            alert.window().makeKeyAndOrderFront(None);
        }
    });
}
fn show(id: u64) -> QuitResponse {
    let Some(main) = MainThreadMarker::new() else {
        return QuitResponse::Failed;
    };
    let alert = NSAlert::new(main);
    alert.setAlertStyle(NSAlertStyle::Warning);
    alert.setMessageText(&NSString::from_str("Stop the service and quit?"));
    alert.setInformativeText(&NSString::from_str(WARNING));
    let stay = alert.addButtonWithTitle(&NSString::from_str("Stay"));
    stay.setKeyEquivalent(&NSString::from_str("\r"));
    let stop = alert.addButtonWithTitle(&NSString::from_str("Stop and Quit"));
    // Never give destructive action Escape or Return. All non-button outcomes
    // below preserve the service; this is deliberately not an rfd Cancel button.
    stop.setKeyEquivalent(&NSString::from_str(""));
    ALERT.with(|slot| *slot.borrow_mut() = Some((id, alert.clone())));
    focus(id);
    let response = alert.runModal(); // ownerless modal, not a sheet on hidden WK
    ALERT.with(|slot| {
        if slot
            .borrow()
            .as_ref()
            .is_some_and(|(current, _)| *current == id)
        {
            slot.borrow_mut().take();
        }
    });
    if response == NSAlertSecondButtonReturn {
        QuitResponse::StopAndQuit
    } else {
        QuitResponse::Stay
    }
}

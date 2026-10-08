//! Always-confirm live owned service; no task/idle inspector or renderer decision.
use crate::errors::Failure;
#[derive(Debug, PartialEq, Eq)]
pub enum QuitAction {
    None,
    Confirm(u64),
    Focus(u64),
    StopOwned,
    Exit,
}
#[derive(Clone, Copy)]
pub enum QuitResponse {
    Stay,
    Dismissed,
    Failed,
    StopAndQuit,
}
#[derive(Default)]
pub struct QuitDecision {
    next: u64,
    pending: Option<u64>,
    stopping: bool,
    exiting: bool,
    disposed: bool,
}
impl QuitDecision {
    pub fn request(&mut self, alive: bool) -> QuitAction {
        if self.disposed || self.exiting || self.stopping {
            return QuitAction::None;
        }
        if let Some(id) = self.pending {
            return QuitAction::Focus(id);
        }
        if !alive {
            self.exiting = true;
            return QuitAction::Exit;
        }
        let Some(id) = self.next.checked_add(1) else {
            return QuitAction::None;
        };
        self.next = id;
        self.pending = Some(id);
        QuitAction::Confirm(id)
    }
    pub fn respond(&mut self, id: u64, response: QuitResponse) -> QuitAction {
        if self.disposed || self.pending != Some(id) {
            return QuitAction::None;
        }
        self.pending = None;
        if matches!(response, QuitResponse::StopAndQuit) {
            self.stopping = true;
            QuitAction::StopOwned
        } else {
            QuitAction::None
        }
    }
    pub fn shutdown_result(&mut self, result: Result<(), Failure>) -> QuitAction {
        if self.disposed || !self.stopping {
            return QuitAction::None;
        }
        self.stopping = false;
        if result.is_ok() {
            self.exiting = true;
            QuitAction::Exit
        } else {
            QuitAction::None
        }
    }
    pub fn is_exiting(&self) -> bool {
        self.exiting
    }
    pub fn dispose(&mut self) {
        self.disposed = true;
        self.pending = None;
        self.stopping = false;
    }
}
pub fn service_alive(state: crate::runtime::State) -> bool {
    matches!(
        state,
        crate::runtime::State::Starting(_)
            | crate::runtime::State::Ready(_)
            | crate::runtime::State::Stopping(_)
            | crate::runtime::State::Failed(_, Failure::Cleanup)
    )
}
#[cfg(feature = "desktop")]
pub mod native;

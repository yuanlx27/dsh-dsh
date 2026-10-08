//! Native orchestration only. Local shell commands get projections, never auth.
use crate::{
    authentication::{self, BootData, ServiceSession},
    bridge::{Bridge, Permit},
    errors::Failure,
    preferences,
    protocol::Protocol,
    runtime::{Runtime, State},
};
use serde::Serialize;
use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[cfg(all(target_os = "macos", feature = "native-bridge"))]
mod failure;
#[cfg(all(target_os = "macos", feature = "native-bridge"))]
pub mod macos;

pub const NOTICE_REVISION: &str = "experimental-2026-10-06";
pub fn notice_acknowledged(root: &Path) -> io::Result<bool> {
    Ok(preferences::load(root)?.safety_notice_revision.as_deref() == Some(NOTICE_REVISION))
}
pub fn acknowledge_notice(root: &Path, revision: &str) -> io::Result<()> {
    if revision != NOTICE_REVISION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unsupported safety notice revision.",
        ));
    }
    let mut saved = preferences::load(root)?;
    saved.safety_notice_revision = Some(NOTICE_REVISION.into());
    preferences::save(root, &saved)
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    phase: &'static str,
    notice_revision: &'static str,
    acknowledged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<&'static str>,
}
pub type Prepare = Arc<dyn Fn(&BootData) -> Result<Arc<Protocol>, Failure> + Send + Sync>;
struct Ready {
    generation: u64,
    session: Arc<ServiceSession>,
    protocol: Arc<Protocol>,
}
struct Local {
    active: Option<Ready>,
    acknowledged: bool,
    preferences_failed: bool,
    generation: u64,
    disposed: bool,
}
pub struct Application {
    runtime: Runtime,
    bridge: Arc<Bridge>,
    root: PathBuf,
    prepare: Prepare,
    local: Mutex<Local>,
    startup: tokio::sync::Mutex<()>,
}
impl Application {
    pub fn new(
        runtime: Runtime,
        bridge: Arc<Bridge>,
        root: PathBuf,
        prepare: Prepare,
    ) -> Result<Arc<Self>, Failure> {
        bridge
            .observe_runtime(runtime.subscribe())
            .map_err(|_| Failure::Unavailable)?;
        let notice = notice_acknowledged(&root);
        let app = Arc::new(Self {
            runtime,
            bridge,
            root,
            prepare,
            local: Mutex::new(Local {
                active: None,
                acknowledged: notice.as_ref().is_ok_and(|v| *v),
                preferences_failed: notice.is_err(),
                generation: 0,
                disposed: false,
            }),
            startup: tokio::sync::Mutex::new(()),
        });
        let weak = Arc::downgrade(&app);
        let mut states = app.runtime.subscribe();
        tokio::spawn(async move {
            loop {
                let Some(app) = weak.upgrade() else {
                    break;
                };
                app.reconcile();
                drop(app);
                if states.changed().await.is_err() {
                    break;
                }
            }
        });
        Ok(app)
    }
    pub fn bridge(&self) -> Arc<Bridge> {
        self.bridge.clone()
    }
    pub fn runtime(&self) -> Runtime {
        self.runtime.clone()
    }
    fn reconcile(&self) {
        let Ok(mut local) = self.local.lock() else {
            return;
        };
        let state = self.runtime.state();
        let generation = match state {
            State::Starting(g) | State::Ready(g) | State::Stopping(g) | State::Failed(g, _) => g,
            State::Stopped => local.generation,
        };
        local.generation = local.generation.max(generation);
        if local.active.as_ref().is_some_and(|a| {
            local.disposed || !matches!(state,State::Starting(g)|State::Ready(g) if g==a.generation)
        }) && let Some(active) = local.active.take()
        {
            active.session.invalidate();
        }
        let ready = !local.disposed
            && matches!(state,State::Ready(g) if local.active.as_ref().is_some_and(|a|a.generation==g&&a.session.is_current(g)));
        let acknowledged = local.acknowledged;
        let generation = local.generation;
        let _ = self.bridge.set_runtime(generation, ready, acknowledged);
        drop(local);
    }
    pub fn status(&self) -> Snapshot {
        self.reconcile();
        let state = self.runtime.state();
        let local = self.local.lock().unwrap_or_else(|e| e.into_inner());
        let ready = matches!(state,State::Ready(g) if !local.disposed&&local.active.as_ref().is_some_and(|a|a.generation==g&&a.session.is_current(g)));
        let failure = if local.preferences_failed {
            Some("preferences")
        } else {
            match state {
                State::Failed(_, f) => Some(match f {
                    Failure::InvalidBundle => "invalidBundle",
                    Failure::Launch => "launch",
                    Failure::Timeout => "timeout",
                    Failure::Exited => "earlyExit",
                    Failure::Cleanup => "cleanup",
                    _ => "connection",
                }),
                _ => None,
            }
        };
        Snapshot {
            phase: if failure.is_some() {
                "failed"
            } else if ready {
                "available"
            } else {
                "starting"
            },
            notice_revision: NOTICE_REVISION,
            acknowledged: local.acknowledged,
            failure,
        }
    }
    pub fn save_bounds(&self, bounds: preferences::Bounds) -> io::Result<()> {
        let local = self
            .local
            .lock()
            .map_err(|_| io::Error::other("Application unavailable."))?;
        if local.disposed {
            return Err(io::Error::other("Application unavailable."));
        }
        let mut saved = preferences::load(&self.root)?;
        saved.window_bounds = Some(bounds);
        preferences::save(&self.root, &saved)
    }
    pub fn acknowledge(&self, revision: &str) -> io::Result<Snapshot> {
        let mut local = self
            .local
            .lock()
            .map_err(|_| io::Error::other("Application unavailable."))?;
        if local.disposed {
            return Err(io::Error::other("Application unavailable."));
        }
        // Serialize shell preference changes and grant only after durable save.
        acknowledge_notice(&self.root, revision)?;
        local.acknowledged = true;
        local.preferences_failed = false;
        drop(local);
        self.reconcile();
        Ok(self.status())
    }
    pub async fn start(&self) -> Result<(), Failure> {
        let _start = self.startup.lock().await;
        {
            let local = self.local.lock().map_err(|_| Failure::Unavailable)?;
            if local.disposed {
                return Err(Failure::Unavailable);
            }
            if matches!(self.runtime.state(),State::Ready(g) if local.active.as_ref().is_some_and(|a|a.generation==g&&a.session.is_current(g)))
            {
                return Ok(());
            }
        }
        let startup = self.runtime.start().await?;
        let generation = startup.generation;
        let deadline = startup.deadline;
        let result = async {
            let token = startup.take_token_url()?;
            let auth = authentication::authenticate(token, generation, deadline).await?;
            let session = auth.session;
            let prepare = self.prepare.clone();
            let prepared = tokio::time::timeout_at(
                deadline,
                tokio::task::spawn_blocking(move || prepare(&auth.boot)),
            )
            .await;
            let protocol = match prepared {
                Ok(Ok(Ok(p))) => p,
                Ok(Ok(Err(e))) => {
                    session.invalidate();
                    return Err(e);
                }
                Ok(Err(_)) => {
                    session.invalidate();
                    return Err(Failure::Connection);
                }
                Err(_) => {
                    session.invalidate();
                    return Err(Failure::Timeout);
                }
            };
            {
                let mut local = self.local.lock().map_err(|_| Failure::Unavailable)?;
                if local.disposed {
                    session.invalidate();
                    return Err(Failure::Unavailable);
                }
                local.active = Some(Ready {
                    generation,
                    session,
                    protocol,
                });
            }
            tokio::time::timeout_at(deadline, self.runtime.complete(generation))
                .await
                .map_err(|_| Failure::Timeout)??;
            self.reconcile();
            Ok(())
        }
        .await;
        if let Err(failure) = result {
            self.invalidate();
            let _ = self.runtime.fail(generation, failure).await;
            self.reconcile();
            return Err(failure);
        }
        Ok(())
    }
    pub fn enter(&self) -> Result<u64, Failure> {
        self.reconcile();
        let local = self.local.lock().map_err(|_| Failure::Unavailable)?;
        if !local.disposed
            && local.acknowledged
            && let State::Ready(g) = self.runtime.state()
            && local
                .active
                .as_ref()
                .is_some_and(|a| a.generation == g && a.session.is_current(g))
        {
            Ok(g)
        } else {
            Err(Failure::Unavailable)
        }
    }
    // A fresh native permit is required even for non-secret bootstrap data.
    pub fn session(&self, permit: &Permit) -> Result<Arc<ServiceSession>, Failure> {
        if !self.bridge.owns(permit) || !permit.is_current() {
            return Err(Failure::Unavailable);
        }
        let local = self.local.lock().map_err(|_| Failure::Unavailable)?;
        let active = local.active.as_ref().ok_or(Failure::Unavailable)?;
        if !local.acknowledged
            || local.disposed
            || !active.session.is_current(permit.runtime_generation())
            || self.runtime.state() != State::Ready(active.generation)
        {
            return Err(Failure::Unavailable);
        }
        Ok(active.session.clone())
    }
    pub fn protocol(&self) -> Option<Arc<Protocol>> {
        self.reconcile();
        self.local
            .lock()
            .ok()?
            .active
            .as_ref()
            .map(|a| a.protocol.clone())
    }
    pub fn invalidate(&self) {
        if let Ok(mut local) = self.local.lock()
            && let Some(active) = local.active.take()
        {
            active.session.invalidate();
        }
        self.reconcile();
    }
    pub async fn shutdown(&self) -> Result<(), Failure> {
        self.invalidate();
        self.runtime.stop().await?;
        self.reconcile();
        Ok(())
    }
    pub fn dispose(&self) {
        if let Ok(mut local) = self.local.lock() {
            local.disposed = true;
        }
        self.invalidate();
    }
}
impl Drop for Application {
    fn drop(&mut self) {
        self.dispose();
    }
}
#[cfg(test)]
mod tests;

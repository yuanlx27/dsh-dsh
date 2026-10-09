//! Frame admission is derived only from native WK metadata. Renderer generations
//! are freshness claims, not origin/window/main-frame proof. No Tauri command
//! accepts a caller-constructed Sender or yields a Permit.
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[cfg(test)]
mod dispatch_tests;
#[cfg(test)]
mod http_tests;
#[cfg(any(test, feature = "native-bridge"))]
pub mod local;
#[cfg(all(target_os = "macos", feature = "native-bridge"))]
pub mod macos;
#[cfg(test)]
mod tests;
#[cfg(any(test, feature = "native-bridge"))]
const MAX_MESSAGE_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Denied;
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Bootstrap,
    Http,
    Stream,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub window_generation: u64,
    pub runtime_generation: u64,
    pub operation: Operation,
    pub payload: Value,
}
#[cfg(any(test, feature = "native-bridge"))]
fn parse_request(body: &str) -> Result<Request, Denied> {
    if body.len() > MAX_MESSAGE_BYTES {
        return Err(Denied);
    }
    let request: Request = serde_json::from_str(body).map_err(|_| Denied)?;
    if matches!(request.operation, Operation::Bootstrap) && !request.payload.is_null() {
        return Err(Denied);
    }
    Ok(request)
}
#[cfg(any(test, feature = "native-bridge"))]
#[derive(Clone)]
struct Sender {
    message_view: usize,
    frame_view: usize,
    main_frame: bool,
    protocol: String,
    host: String,
    port: i64,
    frame_url: Option<String>,
    view_url: Option<String>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Domain {
    App,
    #[cfg(any(test, feature = "native-bridge"))]
    Shell,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Binding {
    view: usize,
    generation: u64,
    domain: Domain,
}
#[derive(Default)]
struct State {
    owner: Option<Binding>,
    epoch: u64,
    #[cfg(any(test, feature = "native-bridge"))]
    next_window: u64,
    runtime_generation: u64,
    ready: bool,
    notice: bool,
    retired: bool,
    observed: Option<watch::Receiver<crate::runtime::State>>,
    #[cfg(any(test, feature = "native-bridge"))]
    document_fragment: Option<String>,
}
pub struct Bridge {
    state: Mutex<State>,
    changes: watch::Sender<u64>,
}
// Non-serializable native capability; scopes every owned transport job/handle
// and final reply to an admitted window/runtime/notice epoch.
#[derive(Clone)]
pub struct Permit {
    bridge: Arc<Bridge>,
    epoch: u64,
    window_generation: u64,
    runtime_generation: u64,
}
impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}
impl Bridge {
    pub fn new() -> Self {
        let (changes, _) = watch::channel(0);
        Self {
            state: Mutex::new(State::default()),
            changes,
        }
    }
    // Native runtime witness is checked at admission/final delivery, not only
    // eventually by the application observer task.
    pub(crate) fn observe_runtime(
        &self,
        runtime: watch::Receiver<crate::runtime::State>,
    ) -> Result<(), Denied> {
        let mut state = self.state.lock().map_err(|_| Denied)?;
        self.change(&mut state)?;
        state.observed = Some(runtime);
        Ok(())
    }
    pub(crate) fn owns(&self, permit: &Permit) -> bool {
        std::ptr::eq(self, Arc::as_ptr(&permit.bridge))
    }
    fn change(&self, state: &mut State) -> Result<(), Denied> {
        state.epoch = state.epoch.checked_add(1).ok_or(Denied)?;
        self.changes.send_replace(state.epoch);
        Ok(())
    }
    // Native lifecycle only: call on authentication completion/invalidation and
    // persisted notice changes, not on renderer-reported readiness or a click.
    pub fn set_runtime(
        &self,
        generation: u64,
        ready: bool,
        saved_notice: bool,
    ) -> Result<(), Denied> {
        let mut state = self.state.lock().map_err(|_| Denied)?;
        if generation < state.runtime_generation
            || ready && generation == state.runtime_generation && state.retired
        {
            return Err(Denied);
        }
        if generation > state.runtime_generation {
            state.retired = false;
        }
        if generation == state.runtime_generation && state.ready && !ready {
            state.retired = true;
        }
        if (state.runtime_generation, state.ready, state.notice)
            != (generation, ready, saved_notice)
        {
            self.change(&mut state)?;
            state.runtime_generation = generation;
            state.ready = ready;
            state.notice = saved_notice;
        }
        Ok(())
    }
    #[cfg(any(test, feature = "native-bridge"))]
    fn bind(&self, view: usize) -> Result<Binding, Denied> {
        self.bind_owned(view, Domain::App)
    }
    #[cfg(any(test, feature = "native-bridge"))]
    fn bind_owned(&self, view: usize, domain: Domain) -> Result<Binding, Denied> {
        if view == 0 {
            return Err(Denied);
        }
        let mut state = self.state.lock().map_err(|_| Denied)?;
        self.change(&mut state)?;
        state.next_window = state.next_window.checked_add(1).ok_or(Denied)?;
        let binding = Binding {
            view,
            generation: state.next_window,
            domain,
        };
        state.owner = Some(binding);
        state.document_fragment = None;
        Ok(binding)
    }
    #[cfg(any(test, feature = "native-bridge"))]
    fn expect_document(&self, binding: Binding, fragment: String) -> Result<(), Denied> {
        let mut state = self.state.lock().map_err(|_| Denied)?;
        if state.owner != Some(binding) || fragment.is_empty() {
            return Err(Denied);
        }
        self.change(&mut state)?;
        state.document_fragment = Some(fragment);
        Ok(())
    }
    #[cfg(feature = "native-bridge")]
    pub(crate) fn detach_view(&self, view: usize) {
        if let Ok(mut state) = self.state.lock()
            && state.owner.is_some_and(|b| b.view == view)
        {
            state.owner = None;
            state.document_fragment = None;
            let _ = self.change(&mut state);
        }
    }
    #[cfg(any(test, feature = "native-bridge"))]
    fn detach(&self, binding: Binding) {
        if let Ok(mut state) = self.state.lock()
            && state.owner == Some(binding)
        {
            state.owner = None;
            state.document_fragment = None;
            let _ = self.change(&mut state);
        }
    }
    #[cfg(any(test, feature = "native-bridge"))]
    fn admit(
        self: &Arc<Self>,
        binding: Binding,
        sender: &Sender,
        request: &Request,
    ) -> Result<Permit, Denied> {
        let state = self.state.lock().map_err(|_| Denied)?;
        validate_sender(&state, binding, sender)?;
        if request.window_generation != binding.generation
            || request.runtime_generation != state.runtime_generation
        {
            return Err(Denied);
        }
        Ok(Permit {
            bridge: Arc::clone(self),
            epoch: state.epoch,
            window_generation: binding.generation,
            runtime_generation: state.runtime_generation,
        })
    }
}
#[cfg(any(test, feature = "native-bridge"))]
fn observed_ready(state: &State) -> bool {
    state
        .observed
        .as_ref()
        .is_none_or(|r| *r.borrow() == crate::runtime::State::Ready(state.runtime_generation))
}
impl Permit {
    pub fn window_generation(&self) -> u64 {
        self.window_generation
    }
    pub fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }
    pub fn is_current(&self) -> bool {
        self.with_current(|| ()).is_ok()
    }
    // Hold the native epoch through final delivery, not merely an earlier check.
    pub(crate) fn with_current<T>(&self, work: impl FnOnce() -> T) -> Result<T, Denied> {
        let state = self.bridge.state.lock().map_err(|_| Denied)?;
        let witness = state.observed.as_ref().map(|r| r.borrow());
        let current = {
            state.epoch == self.epoch
                && state.ready
                && witness
                    .as_ref()
                    .is_none_or(|r| **r == crate::runtime::State::Ready(state.runtime_generation))
                && state.notice
                && state.owner.is_some_and(|b| {
                    b.generation == self.window_generation && b.domain == Domain::App
                })
                && state.runtime_generation == self.runtime_generation
        };
        if !current {
            return Err(Denied);
        }
        let result = work();
        drop(witness);
        Ok(result)
    }
    pub fn same_scope(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.bridge, &other.bridge)
            && self.epoch == other.epoch
            && self.window_generation == other.window_generation
            && self.runtime_generation == other.runtime_generation
    }
    pub async fn revoked(&self) {
        let mut changes = self.bridge.changes.subscribe();
        loop {
            if !self.is_current() || changes.changed().await.is_err() {
                return;
            }
        }
    }
}
#[cfg(any(test, feature = "native-bridge"))]
fn validate_sender(state: &State, binding: Binding, sender: &Sender) -> Result<(), Denied> {
    if binding.domain != Domain::App
        || !state.ready
        || !observed_ready(state)
        || !state.notice
        || state.runtime_generation == 0
    {
        return Err(Denied);
    }
    validate_owned_sender(state, binding, sender, "app")
}
#[cfg(any(test, feature = "native-bridge"))]
fn validate_owned_sender(
    state: &State,
    binding: Binding,
    sender: &Sender,
    host: &str,
) -> Result<(), Denied> {
    if state.owner != Some(binding)
        || sender.message_view != binding.view
        || sender.frame_view != binding.view
        || !sender.main_frame
        || sender.protocol != "dsh-app"
        || sender.host != host
        || sender.port != 0
        || !owned_url(sender.frame_url.as_deref(), host)
        || !owned_url(sender.view_url.as_deref(), host)
        || state.document_fragment.as_ref().is_some_and(|expected| {
            [sender.frame_url.as_deref(), sender.view_url.as_deref()]
                .into_iter()
                .any(|input| {
                    input
                        .and_then(|s| url::Url::parse(s).ok())
                        .is_none_or(|u| u.fragment() != Some(expected.as_str()))
                })
        })
    {
        return Err(Denied);
    }
    Ok(())
}
#[cfg(any(test, feature = "native-bridge"))]
fn owned_url(input: Option<&str>, host: &str) -> bool {
    input
        .and_then(|s| url::Url::parse(s).ok())
        .is_some_and(|u| {
            u.scheme() == "dsh-app"
                && u.host_str() == Some(host)
                && u.port().is_none()
                && u.username().is_empty()
                && u.password().is_none()
        })
}

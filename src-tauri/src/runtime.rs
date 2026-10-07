use crate::{
    bundle::{self, BundlePaths},
    preferences,
};
use std::{
    ffi::OsStr,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin},
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
    time::Instant,
};
use url::Url;

#[cfg(test)]
mod tests;

pub use crate::errors::Failure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Stopped,
    Starting(u64),
    Ready(u64),
    Stopping(u64),
    Failed(u64, Failure),
}

pub fn parse_readiness(line: &str) -> Result<Option<Url>, Failure> {
    let Some(raw) = line.strip_prefix("dsh web: ") else {
        return Ok(None);
    };
    if !raw.starts_with("http://127.0.0.1:") || raw.chars().any(char::is_whitespace) {
        return Err(Failure::Announcement);
    }
    let url = Url::parse(raw).map_err(|_| Failure::Announcement)?;
    let query: Vec<_> = url.query_pairs().collect();
    if url.scheme() != "http"
        || url.host_str() != Some("127.0.0.1")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port_or_known_default().is_none_or(|p| p == 0)
        || url.path() != "/"
        || url.fragment().is_some()
        || query.len() != 1
        || query[0].0 != "token"
        || query[0].1.is_empty()
    {
        return Err(Failure::Announcement);
    }
    Ok(Some(url))
}

pub async fn read_readiness<R: AsyncBufRead + Unpin>(
    reader: R,
    deadline: Instant,
) -> Result<Url, Failure> {
    use tokio::io::AsyncBufReadExt;
    tokio::time::timeout_at(deadline, async {
        let mut reader = reader;
        loop {
            let mut line = Vec::new();
            let size = (&mut reader)
                .take(65537)
                .read_until(b'\n', &mut line)
                .await
                .map_err(|_| Failure::Announcement)?;
            if size == 0 {
                return Err(Failure::Exited);
            }
            if size > 65536 {
                return Err(Failure::Announcement);
            }
            let line = std::str::from_utf8(&line)
                .map_err(|_| Failure::Announcement)?
                .trim_end_matches(['\r', '\n']);
            if let Some(url) = parse_readiness(line)? {
                return Ok(url);
            }
        }
    })
    .await
    .map_err(|_| Failure::Timeout)?
}

pub fn excluded_environment(name: &OsStr) -> bool {
    let name = name.to_string_lossy().to_ascii_uppercase();
    matches!(
        name.as_str(),
        "NODE_OPTIONS"
            | "NODE_PATH"
            | "NODE_EXTRA_CA_CERTS"
            | "NODE_REPL_EXTERNAL_MODULE"
            | "ELECTRON_RUN_AS_NODE"
    ) || name.ends_with("_API_KEY")
        || [
            "DEEPSEEK_",
            "OPENAI_",
            "ANTHROPIC_",
            "GEMINI_",
            "GROQ_",
            "MISTRAL_",
            "COHERE_",
            "DSH_TELEMETRY",
            "DSH_ANALYTICS",
            "SENTRY_",
            "POSTHOG_",
            "SEGMENT_",
            "OTEL_",
        ]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

pub struct LaunchConfig {
    launcher: PathBuf,
    node: PathBuf,
    cli: PathBuf,
    home: PathBuf,
    cwd: PathBuf,
    overlay: PathBuf,
}

impl LaunchConfig {
    // The native app supplies only immutable bundle paths and its own validated overlay.
    pub async fn verified(paths: &BundlePaths, app_data: PathBuf) -> Result<Self, Failure> {
        let manifest = bundle::verify(paths)
            .await
            .map_err(|_| Failure::InvalidBundle)?;
        let overlay = paths.resources.join("desktop-web.patch.yml");
        if !manifest
            .artifacts
            .iter()
            .any(|a| a.path == "src-tauri/resources/desktop-web.patch.yml")
            || !overlay.is_absolute()
            || !overlay.is_file()
        {
            return Err(Failure::InvalidBundle);
        }
        preferences::ensure_private_dir(&app_data).map_err(|_| Failure::Launch)?;
        let home = app_data.join("dsh");
        let cwd = app_data.join("working-directory");
        for path in [&home, &cwd] {
            preferences::ensure_private_dir(path).map_err(|_| Failure::Launch)?;
        }
        Ok(Self {
            launcher: paths.launcher(),
            node: paths.node(),
            cli: paths.cli(),
            home,
            cwd,
            overlay,
        })
    }
}

// No Debug/Serialize implementation: this object must never become UI boot data.
pub struct Startup {
    pub generation: u64,
    pub deadline: Instant,
    token_url: Mutex<Option<Url>>,
}
impl Startup {
    pub fn take_token_url(&self) -> Result<Url, Failure> {
        self.token_url
            .lock()
            .map_err(|_| Failure::Connection)?
            .take()
            .ok_or(Failure::Unavailable)
    }
    fn discard_token(&self) {
        if let Ok(mut token) = self.token_url.lock() {
            *token = None;
        }
    }
}
type StartReply = oneshot::Sender<Result<Arc<Startup>, Failure>>;
enum Request {
    Start(StartReply),
    Complete(u64, oneshot::Sender<Result<(), Failure>>),
    Stop(oneshot::Sender<Result<(), Failure>>),
}
enum Event {
    Announcement(u64, Result<Url, Failure>),
}

#[derive(Clone)]
pub struct Runtime {
    requests: mpsc::Sender<Request>,
    state: watch::Receiver<State>,
}
impl Runtime {
    pub fn new(config: LaunchConfig) -> Self {
        let (requests, receiver) = mpsc::channel(32);
        let (state, status) = watch::channel(State::Stopped);
        tokio::spawn(owner(config, receiver, state));
        Self {
            requests,
            state: status,
        }
    }
    pub fn state(&self) -> State {
        *self.state.borrow()
    }
    pub fn subscribe(&self) -> watch::Receiver<State> {
        self.state.clone()
    }
    pub async fn start(&self) -> Result<Arc<Startup>, Failure> {
        let (tx, rx) = oneshot::channel();
        self.requests
            .send(Request::Start(tx))
            .await
            .map_err(|_| Failure::Unavailable)?;
        rx.await.map_err(|_| Failure::Unavailable)?
    }
    pub async fn complete(&self, generation: u64) -> Result<(), Failure> {
        let (tx, rx) = oneshot::channel();
        self.requests
            .send(Request::Complete(generation, tx))
            .await
            .map_err(|_| Failure::Unavailable)?;
        rx.await.map_err(|_| Failure::Unavailable)?
    }
    pub async fn stop(&self) -> Result<(), Failure> {
        let (tx, rx) = oneshot::channel();
        self.requests
            .send(Request::Stop(tx))
            .await
            .map_err(|_| Failure::Unavailable)?;
        rx.await.map_err(|_| Failure::Unavailable)?
    }
}

struct Process {
    child: Child,
    pipe: Option<ChildStdin>,
    drains: Vec<JoinHandle<()>>,
}
impl Process {
    async fn stop(&mut self) -> Result<(), Failure> {
        if let Some(mut pipe) = self.pipe.take() {
            let _ = pipe.write_all(b"stop\n").await;
        }
        // The launcher performs the seven-second group escalation and stays alive if cleanup fails.
        match tokio::time::timeout(Duration::from_secs(9), self.child.wait()).await {
            Ok(Ok(status)) if status.success() || status.code() == Some(2) => {
                for drain in &self.drains {
                    drain.abort();
                }
                Ok(())
            }
            _ => Err(Failure::Cleanup),
        }
    }
}

fn spawn(
    config: &LaunchConfig,
    generation: u64,
    deadline: Instant,
    events: mpsc::Sender<Event>,
) -> Result<Process, Failure> {
    let mut command = tokio::process::Command::new(&config.launcher);
    command
        .arg("--node")
        .arg(&config.node)
        .arg("--cli")
        .arg(&config.cli)
        .arg("--home")
        .arg(&config.home)
        .arg("--cwd")
        .arg(&config.cwd)
        .arg("--overlay")
        .arg(&config.overlay)
        .current_dir(&config.cwd)
        .env("DSH_HOME", &config.home)
        .env("DO_NOT_TRACK", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, _) in std::env::vars_os() {
        if excluded_environment(&name) {
            command.env_remove(name);
        }
    }
    let mut child = command.spawn().map_err(|_| Failure::Launch)?;
    let pipe = child.stdin.take();
    let stdout = child.stdout.take().ok_or(Failure::Launch)?;
    let stderr = child.stderr.take().ok_or(Failure::Launch)?;
    let first = tokio::spawn(async move {
        let mut output = BufReader::new(stdout);
        let result = read_readiness(&mut output, deadline).await;
        let valid = result.is_ok();
        let _ = events.send(Event::Announcement(generation, result)).await;
        if valid {
            let _ = tokio::io::copy(&mut output, &mut tokio::io::sink()).await;
        }
    });
    let second = tokio::spawn(async move {
        let _ = tokio::io::copy(&mut BufReader::new(stderr), &mut tokio::io::sink()).await;
    });
    Ok(Process {
        child,
        pipe,
        drains: vec![first, second],
    })
}

async fn owner(
    config: LaunchConfig,
    mut requests: mpsc::Receiver<Request>,
    state: watch::Sender<State>,
) {
    let (events, mut announcements) = mpsc::channel(1);
    let mut process: Option<Process> = None;
    let mut startup: Option<Arc<Startup>> = None;
    let mut generation = 0_u64;
    let mut waiters: Vec<StartReply> = Vec::new();
    let mut deadline = Instant::now();
    loop {
        tokio::select! {
            request = requests.recv() => match request {
                None => {
                    if let Some(value) = startup.take() { value.discard_token(); }
                    if let Some(p) = process.as_mut() { let _ = p.stop().await; }
                    break;
                }
                Some(Request::Start(reply)) => {
                    if matches!(*state.borrow(), State::Starting(_) | State::Ready(_)) {
                        if let Some(value) = &startup { let _ = reply.send(Ok(value.clone())); }
                        else { waiters.push(reply); }
                        continue;
                    }
                    if let Some(p) = process.as_mut() {
                        state.send_replace(State::Stopping(generation));
                        if p.stop().await.is_err() {
                            state.send_replace(State::Failed(generation, Failure::Cleanup));
                            let _ = reply.send(Err(Failure::Cleanup)); continue;
                        }
                        process = None;
                    }
                    generation += 1;
                    deadline = Instant::now() + Duration::from_secs(15);
                    state.send_replace(State::Starting(generation));
                    match spawn(&config, generation, deadline, events.clone()) {
                        Ok(value) => { process = Some(value); waiters.push(reply); }
                        Err(error) => { state.send_replace(State::Failed(generation, error)); let _ = reply.send(Err(error)); }
                    }
                }
                Some(Request::Complete(id, reply)) => {
                    let result = if let Some(value) = startup.as_ref().filter(|_| id == generation && matches!(*state.borrow(), State::Starting(_)) && Instant::now() < deadline) {
                        value.discard_token();
                        state.send_replace(State::Ready(generation)); Ok(())
                    } else { Err(Failure::Unavailable) };
                    let _ = reply.send(result);
                }
                Some(Request::Stop(reply)) => {
                    if let Some(value) = startup.take() { value.discard_token(); }
                    state.send_replace(State::Stopping(generation));
                    let result = if let Some(p) = process.as_mut() { p.stop().await } else { Ok(()) };
                    if result.is_ok() { process = None; state.send_replace(State::Stopped); }
                    else { state.send_replace(State::Failed(generation, Failure::Cleanup)); }
                    for waiter in waiters.drain(..) { let _ = waiter.send(Err(Failure::Unavailable)); }
                    let _ = reply.send(result);
                }
            },
            Some(Event::Announcement(id, result)) = announcements.recv() => {
                if id != generation || !matches!(*state.borrow(), State::Starting(_)) { continue; }
                match result {
                    Ok(url) => {
                        let value = Arc::new(Startup { generation, deadline, token_url: Mutex::new(Some(url)) });
                        for waiter in waiters.drain(..) { let _ = waiter.send(Ok(value.clone())); }
                        startup = Some(value);
                    }
                    Err(error) => {
                        state.send_replace(State::Failed(generation, error));
                        let clean = if let Some(p) = process.as_mut() { p.stop().await.is_ok() } else { true };
                        if clean { process = None; }
                        let error = if clean { error } else { Failure::Cleanup };
                        state.send_replace(State::Failed(generation, error));
                        for waiter in waiters.drain(..) { let _ = waiter.send(Err(error)); }
                    }
                }
            },
            _ = tokio::time::sleep_until(deadline), if matches!(*state.borrow(), State::Starting(_)) => {
                if let Some(value) = startup.take() { value.discard_token(); }
                state.send_replace(State::Failed(generation, Failure::Timeout));
                let clean = if let Some(p) = process.as_mut() { p.stop().await.is_ok() } else { true };
                if clean { process = None; }
                let error = if clean { Failure::Timeout } else { Failure::Cleanup };
                state.send_replace(State::Failed(generation, error));
                for waiter in waiters.drain(..) { let _ = waiter.send(Err(error)); }
            },
            status = async { process.as_mut().unwrap().child.wait().await }, if process.is_some() && !matches!(*state.borrow(), State::Failed(_, Failure::Cleanup)) => {
                if let Some(value) = startup.take() { value.discard_token(); }
                let clean = matches!(status, Ok(status) if status.success() || status.code() == Some(2));
                let error = if clean { Failure::Exited } else { Failure::Cleanup };
                if clean && let Some(p) = process.take() { for drain in p.drains { drain.abort(); } }
                state.send_replace(State::Failed(generation, error));
                for waiter in waiters.drain(..) { let _ = waiter.send(Err(error)); }
            }
        }
    }
}

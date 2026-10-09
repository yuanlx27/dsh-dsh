//! Native application entry. No general invoke handler or renderer plugin rights.
use crate::{
    app::{self, Application, macos::Document},
    bridge::{
        Bridge, Operation,
        local::{self, macos::Dispatch as LocalDispatch},
        macos::{Dispatch, Reply},
    },
    bundle::{BundlePaths, Manifest},
    dispatch::Dispatcher,
    errors::Failure,
    menu,
    protocol::{Asset, Protocol},
    quit::native::Quit,
    runtime::{LaunchConfig, Runtime},
    window::native::Primary,
};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_web_kit::WKWebView;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use tauri::{AppHandle, Manager, RunEvent};
struct Core {
    application: Arc<Application>,
    dispatcher: Arc<Dispatcher>,
    quit: Arc<Quit>,
    manifest: Manifest,
}
struct Desktop {
    bridge: Arc<Bridge>,
    primary: OnceLock<Arc<Primary>>,
    core: Mutex<Option<Arc<Core>>>,
    failure: Mutex<Option<Failure>>,
    root: PathBuf,
    disposed: AtomicBool,
    next: AtomicU64,
    init: tokio::sync::Mutex<()>,
}
struct Page {
    id: u64,
    document: Document,
}
thread_local! {
    static PAGE:RefCell<Option<Page>>=const {RefCell::new(None)};
    static REPLIES:RefCell<HashMap<u64,Reply>>=RefCell::new(HashMap::new());
}
fn shell() -> BTreeMap<String, Vec<u8>> {
    BTreeMap::from([
        (
            "index.html".into(),
            include_bytes!("../../dist/shell/index.html").to_vec(),
        ),
        (
            "main.js".into(),
            include_bytes!("../../dist/shell/main.js").to_vec(),
        ),
        (
            "state.js".into(),
            include_bytes!("../../dist/shell/state.js").to_vec(),
        ),
        (
            "native.js".into(),
            include_bytes!("../../dist/shell/native.js").to_vec(),
        ),
        ("wk.js".into(), include_bytes!("../../dist/wk.js").to_vec()),
        (
            "styles.css".into(),
            include_bytes!("../../dist/shell/styles.css").to_vec(),
        ),
    ])
}
fn modules() -> BTreeMap<String, Vec<u8>> {
    BTreeMap::from([
        (
            "harness.js".into(),
            include_bytes!("../../dist/harness.js").to_vec(),
        ),
        ("wk.js".into(), include_bytes!("../../dist/wk.js").to_vec()),
        (
            "transport.js".into(),
            include_bytes!("../../dist/transport.js").to_vec(),
        ),
        (
            "native-transport.js".into(),
            include_bytes!("../../dist/native-transport.js").to_vec(),
        ),
    ])
}
impl Desktop {
    fn core(&self) -> Option<Arc<Core>> {
        self.core.lock().ok()?.clone()
    }
    fn request_quit(&self) -> bool {
        let core = self.core.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(current) = core.as_ref() {
            let quit = current.quit.clone();
            drop(core);
            if quit.is_exiting() {
                return false;
            }
            quit.request();
            true
        } else {
            self.disposed.store(true, Ordering::SeqCst);
            false
        }
    }
    fn status(&self) -> Value {
        if let Some(core) = self.core() {
            return serde_json::to_value(core.application.status()).unwrap_or(Value::Null);
        }
        let failure = self.failure.lock().ok().and_then(|v| *v).map(|f| match f {
            Failure::InvalidBundle => "invalidBundle",
            Failure::Launch => "launch",
            Failure::Cleanup => "cleanup",
            _ => "connection",
        });
        json!({"phase":if failure.is_some(){"failed"}else{"starting"},"noticeRevision":app::NOTICE_REVISION,"acknowledged":app::notice_acknowledged(&self.root).unwrap_or(false),"failure":failure}).as_object().map(|row|{let mut row=row.clone();if failure.is_none(){row.remove("failure");}Value::Object(row)}).unwrap_or(Value::Null)
    }
    fn retain(&self, reply: Reply) -> Option<u64> {
        REPLIES.with(|map| {
            let mut map = map.borrow_mut();
            if map.len() >= 64 {
                return None;
            }
            let id = self.next.fetch_add(1, Ordering::Relaxed);
            map.insert(id, reply);
            Some(id)
        })
    }
    fn complete(handle: &AppHandle, id: u64, result: Result<Value, Failure>) {
        let _ = handle.run_on_main_thread(move || {
            REPLIES.with(|map| {
                if let Some(reply) = map.borrow_mut().remove(&id)
                    && let Ok(value) = result
                    && reply.complete(&value).is_ok()
                {
                    fixture_event("original-frame-reply-delivered");
                }
            })
        });
    }
    fn harness(self: &Arc<Self>, handle: AppHandle) -> Dispatch {
        let root = self.clone();
        Rc::new(move |scope, request, reply| {
            let Some(core) = root.core() else {
                return;
            };
            if matches!(request.operation, Operation::Http)
                && request.payload.get("action").and_then(Value::as_str) == Some("begin")
                && let Some(path) = request.payload.get("path").and_then(Value::as_str)
                && [
                    "/api/_desktop_fixture_boot",
                    "/api/_desktop_fixture_queue",
                    "/api/_desktop_fixture_entry",
                    "/api/_desktop_fixture_error",
                    "/api/_desktop_fixture_mux",
                    "/api/_desktop_fixture_mux-error",
                ]
                .contains(&path)
            {
                fixture_event(path);
            }
            fixture_event(match request.operation {
                Operation::Bootstrap => "harness-bootstrap-admitted",
                Operation::Http => "harness-http-admitted",
                Operation::Stream => "harness-stream-admitted",
            });
            if matches!(request.operation, Operation::Bootstrap) {
                if !request.payload.is_null() {
                    return;
                }
                if core.application.session(&scope).is_err() {
                    return;
                }
                if let Some(protocol) = core.application.protocol()
                    && let Ok(value) = serde_json::to_value(protocol.bootstrap())
                {
                    #[cfg(debug_assertions)]
                    let value = {
                        let mut value = value;
                        if std::env::var_os("DSH_DESKTOP_FIXTURE_ROOT").is_some() {
                            value["fixture"] = Value::Bool(true);
                        }
                        value
                    };
                    let _ = reply.complete(&value);
                }
                return;
            }
            let Some(id) = root.retain(reply) else {
                return;
            };
            let handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                let result = core
                    .dispatcher
                    .call(scope, request.operation, request.payload)
                    .await;
                Self::complete(&handle, id, result);
            });
        })
    }
    fn local(self: &Arc<Self>, handle: AppHandle) -> LocalDispatch {
        let root = self.clone();
        Rc::new(move |scope, request, reply| {
            fixture_event("shell-main-frame-admitted");
            if matches!(request, local::Request::Enter {}) {
                let Some(core) = root.core() else {
                    return;
                };
                if core.application.enter().is_err() {
                    return;
                }
                if reply.complete(&Value::Null).is_err() {
                    return;
                }
                let dispatch = root.harness(handle.clone());
                PAGE.with(|page| {
                    if let Some(page) = page.borrow_mut().as_mut() {
                        let _ = page.document.enter(&core.application, dispatch);
                    }
                });
                return;
            }
            let Some(id) = root.retain(reply) else {
                return;
            };
            let root = root.clone();
            let handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                if scope.with_current(|| ()).is_err() {
                    Self::complete(&handle, id, Err(Failure::Unavailable));
                    return;
                }
                let result = match request {
                    local::Request::Status {} => Ok(root.status()),
                    local::Request::Acknowledge { revision } => {
                        if let Some(core) = root.core() {
                            core.application
                                .acknowledge_scoped(&scope, &revision)
                                .map_err(|_| Failure::Unavailable)
                                .and_then(|snapshot| {
                                    serde_json::to_value(snapshot).map_err(|_| Failure::Unavailable)
                                })
                        } else {
                            scope
                                .with_current(|| app::acknowledge_notice(&root.root, &revision))
                                .map_err(|_| Failure::Unavailable)
                                .and_then(|saved| saved.map_err(|_| Failure::Unavailable))
                                .map(|_| {
                                    if let Some(core) = root.core() {
                                        core.application.reload_notice();
                                    }
                                    root.status()
                                })
                                .map_err(|_| Failure::Unavailable)
                        }
                    }
                    local::Request::Retry {} => {
                        let result = root.clone().initialize(handle.clone()).await;
                        if let Err(f) = result
                            && let Ok(mut failure) = root.failure.lock()
                        {
                            *failure = Some(f);
                        }
                        result.map(|_| root.status())
                    }
                    local::Request::Enter {} => Err(Failure::Unavailable),
                };
                Self::complete(&handle, id, result);
            });
        })
    }
    fn created(self: &Arc<Self>, window: tauri::WebviewWindow, id: u64) -> Result<(), Failure> {
        let root = self.clone();
        let handle = window.app_handle().clone();
        #[cfg(debug_assertions)]
        if std::env::var_os("DSH_DESKTOP_FIXTURE_ROOT").is_some()
            && std::env::var("DSH_DESKTOP_FIXTURE_INSPECT").as_deref() == Ok("1")
        {
            window.open_devtools();
        }
        window
            .with_webview(move |view| {
                let Some(main) = MainThreadMarker::new() else {
                    return;
                };
                // Tauri gives the native WKWebView pointer on its UI thread.
                let Some(view) = (unsafe { Retained::retain(view.inner() as *mut WKWebView) })
                else {
                    return;
                };
                let weak = Arc::downgrade(&root);
                let failure_handle = handle.clone();
                let failed = Rc::new(move || {
                    if let Some(root) = weak.upgrade() {
                        if let Some(primary) = root.primary.get() {
                            primary.failed(id);
                        }
                        failure_handle.dialog_fixed_failure();
                    }
                });
                let mut document = Document::new(view, root.bridge.clone(), main, failed);
                if document.local(root.local(handle)).is_err() {
                    return;
                }
                PAGE.with(|page| *page.borrow_mut() = Some(Page { id, document }));
            })
            .map_err(|_| Failure::Connection)
    }
    async fn initialize(self: Arc<Self>, handle: AppHandle) -> Result<(), Failure> {
        let _initialization = self.init.lock().await;
        if self.disposed.load(Ordering::SeqCst) {
            return Err(Failure::Unavailable);
        }
        if let Some(core) = self.core() {
            return core.application.start().await;
        }
        let resources = handle
            .path()
            .resource_dir()
            .map_err(|_| Failure::InvalidBundle)?;
        let binaries = std::env::current_exe()
            .map_err(|_| Failure::InvalidBundle)?
            .parent()
            .ok_or(Failure::InvalidBundle)?
            .to_owned();
        #[cfg(debug_assertions)]
        let paths = if binaries.ends_with("Contents/MacOS") {
            BundlePaths::installed(resources, binaries)
        } else {
            // Cargo tests rebuild the build-only launcher at target/debug/dsh-launcher.
            // Unbundled development uses the already verified staging helper instead.
            BundlePaths::staged(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .ok_or(Failure::InvalidBundle)?,
            )
        };
        #[cfg(not(debug_assertions))]
        let paths = BundlePaths::installed(resources, binaries);
        let paths = Arc::new(paths);
        let (config, manifest) = LaunchConfig::verified_manifest(&paths, self.root.clone()).await?;
        if self.disposed.load(Ordering::SeqCst) {
            return Err(Failure::Unavailable);
        }
        let frozen = manifest.clone();
        let assets = shell();
        let prepare = Arc::new(move |boot: &crate::authentication::BootData| {
            let mut protocol = Protocol::new(&paths, &frozen, boot, assets.clone())?;
            protocol.desktop_modules(modules())?;
            Ok(Arc::new(protocol))
        });
        let application = Application::new(
            Runtime::new(config),
            self.bridge.clone(),
            self.root.clone(),
            prepare,
        )?;
        let app = application.clone();
        let dispatcher = Dispatcher::new(Arc::new(move |scope| app.session(scope)));
        let quit = Quit::new(
            application.clone(),
            self.primary.get().ok_or(Failure::Unavailable)?.clone(),
            handle.clone(),
        );
        {
            let mut current = self.core.lock().map_err(|_| Failure::Unavailable)?;
            if self.disposed.load(Ordering::SeqCst) {
                application.dispose();
                return Err(Failure::Unavailable);
            }
            application.reload_notice();
            *current = Some(Arc::new(Core {
                application: application.clone(),
                dispatcher,
                quit,
                manifest,
            }));
        }
        let weak = Arc::downgrade(&self);
        let observed = Arc::downgrade(&application);
        let mut states = application.runtime().subscribe();
        let ui = handle.clone();
        tauri::async_runtime::spawn(async move {
            while states.changed().await.is_ok() {
                let state = *states.borrow_and_update();
                if matches!(state, crate::runtime::State::Failed(_, _)) {
                    let weak = weak.clone();
                    let observed = observed.clone();
                    let ui_copy = ui.clone();
                    let _ = ui.run_on_main_thread(move || {
                        if observed
                            .upgrade()
                            .is_none_or(|app| app.runtime().state() != state)
                        {
                            return;
                        }
                        if let Some(root) = weak.upgrade() {
                            PAGE.with(|page| {
                                if let Some(page) = page.borrow_mut().as_mut()
                                    && page.document.window_generation().is_some()
                                {
                                    let _ = page.document.local(root.local(ui_copy));
                                }
                            });
                        }
                    });
                }
            }
        });
        application.start().await?;
        #[cfg(debug_assertions)]
        if std::env::var_os("DSH_DESKTOP_FIXTURE_ROOT").is_some()
            && std::env::var("DSH_DESKTOP_FIXTURE_ENTER").as_deref() == Ok("1")
        {
            // Explicit fixture-only native consent seeding; not UI/user acceptance.
            application
                .acknowledge(app::NOTICE_REVISION)
                .map_err(|_| Failure::Unavailable)?;
            let native = self.clone();
            let entered = application.clone();
            let ui = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                PAGE.with(|page| {
                    if let Some(page) = page.borrow_mut().as_mut()
                        && page.document.enter(&entered, native.harness(ui)).is_ok()
                    {
                        fixture_event("fixture-native-entered");
                    }
                })
            });
        }
        Ok(())
    }
    fn asset(&self, method: &str, uri: &str) -> Asset {
        if let Some(core) = self.core()
            && let Some(protocol) = core.application.protocol()
        {
            return protocol.respond(method, uri);
        }
        if (method == "GET" || method == "HEAD")
            && let Some(path) = uri.strip_prefix("dsh-app://shell/")
            && let Some(bytes) = shell().get(if path.is_empty() { "index.html" } else { path })
        {
            return Asset {
                status: 200,
                content_type: if path.ends_with(".js") {
                    "text/javascript; charset=utf-8"
                } else if path.ends_with(".css") {
                    "text/css; charset=utf-8"
                } else {
                    "text/html; charset=utf-8"
                },
                content_length: bytes.len(),
                body: if method == "HEAD" {
                    vec![]
                } else {
                    bytes.clone()
                },
            };
        }
        Asset {
            status: 404,
            content_type: "application/octet-stream",
            content_length: 0,
            body: vec![],
        }
    }
}
// Native-only fixed failure; no renderer failure/body/URL is interpolated.
trait FixedDialog {
    fn dialog_fixed_failure(&self);
}
impl FixedDialog for AppHandle {
    fn dialog_fixed_failure(&self) {
        use tauri_plugin_dialog::DialogExt;
        self.dialog().message("The Harness page stopped. Open Workspace Window to recreate it. The local service was not restarted.").title("Harness page unavailable").show(|_|{});
    }
}
pub(crate) fn fixture_event(event: &str) {
    #[cfg(debug_assertions)]
    if let Some(root) = std::env::var_os("DSH_DESKTOP_FIXTURE_ROOT") {
        use std::{io::Write, os::unix::fs::OpenOptionsExt};
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(PathBuf::from(root).join("native-smoke.txt"))
        {
            let _ = writeln!(file, "{event}");
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = event;
}
fn data_root(app: &tauri::App) -> tauri::Result<PathBuf> {
    #[cfg(debug_assertions)]
    if let Some(value) = std::env::var_os("DSH_DESKTOP_FIXTURE_ROOT") {
        let root = PathBuf::from(value);
        if !root.is_absolute() {
            return Err(tauri::Error::Io(std::io::Error::other(
                "Fixture root must be absolute.",
            )));
        }
        return Ok(root);
    }
    app.path().app_data_dir()
}
pub fn run() {
    let root_slot: Arc<OnceLock<Arc<Desktop>>> = Arc::new(OnceLock::new());
    let scheme = root_slot.clone();
    let setup = root_slot.clone();
    let events = root_slot.clone();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(
            move |app, _args, _cwd| {
                if let Some(root) = events.get()
                    && let Some(primary) = root.primary.get()
                {
                    let _ = primary.open(app);
                }
            },
        ))
        .register_uri_scheme_protocol("dsh-app", move |_context, request| {
            let asset = scheme
                .get()
                .map(|root| root.asset(request.method().as_str(), &request.uri().to_string()))
                .unwrap_or(Asset {
                    status: 503,
                    content_type: "application/octet-stream",
                    content_length: 0,
                    body: vec![],
                });
            tauri::http::Response::builder()
                .status(asset.status)
                .header("Content-Type", asset.content_type)
                .header("Content-Length", asset.content_length.to_string())
                .header(
                    "Content-Security-Policy",
                    crate::protocol::content_security_policy(request.uri().host()),
                )
                .body(asset.body)
                .expect("Static response unavailable.")
        })
        .setup(move |app| {
            let root = Arc::new(Desktop {
                bridge: Arc::new(Bridge::new()),
                primary: OnceLock::new(),
                core: Mutex::new(None),
                failure: Mutex::new(None),
                root: data_root(app)?,
                disposed: AtomicBool::new(false),
                next: AtomicU64::new(1),
                init: tokio::sync::Mutex::new(()),
            });
            let weak = Arc::downgrade(&root);
            let detached = Arc::new(move |id| {
                PAGE.with(|page| {
                    let mut slot = page.borrow_mut();
                    if id == 0 || slot.as_ref().is_some_and(|p| p.id == id) {
                        slot.take();
                        REPLIES.with(|map| map.borrow_mut().clear());
                    }
                });
            });
            let created = Arc::new(move |window, id| {
                weak.upgrade()
                    .ok_or(Failure::Unavailable)?
                    .created(window, id)
            });
            let weak = Arc::downgrade(&root);
            let persist = Arc::new(move |bounds| {
                if let Some(root) = weak.upgrade()
                    && let Some(core) = root.core()
                {
                    let _ = core.application.save_bounds(bounds);
                }
            });
            let primary = Primary::new(root.root.clone(), created, detached, persist);
            let _ = root.primary.set(primary.clone());
            let _ = setup.set(root.clone());
            app.set_menu(menu::native::build(app.handle())?)?;
            let commands = root.clone();
            app.on_menu_event(move |app, event| {
                if let Some(core) = commands.core() {
                    menu::native::dispatch(
                        app,
                        event.id().as_ref(),
                        commands.primary.get().unwrap(),
                        &core.quit,
                        &core.manifest,
                    );
                } else {
                    match menu::command(event.id().as_ref()) {
                        Some(menu::Command::Open) => {
                            let _ = commands.primary.get().unwrap().open(app);
                        }
                        Some(menu::Command::Close) => {
                            let _ = commands.primary.get().unwrap().close(app);
                        }
                        Some(menu::Command::Quit) if !commands.request_quit() => {
                            app.exit(0);
                        }
                        Some(action @ (menu::Command::About | menu::Command::Safety)) => {
                            menu::native::informational(app, action, None)
                        }
                        _ => {}
                    }
                }
            });
            primary.open(app.handle())?;
            let handle = app.handle().clone();
            let initialized = root.clone();
            tauri::async_runtime::spawn(async move {
                let result = initialized.clone().initialize(handle).await;
                match result {
                    Ok(()) => fixture_event("authenticated-prepared-ready"),
                    Err(f) => fixture_event(&format!("startup-{f:?}")),
                };
                if let Err(f) = result
                    && let Ok(mut failure) = initialized.failure.lock()
                {
                    *failure = Some(f);
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("The desktop application could not start.");
    app.run(move |app, event| {
        if let Some(root) = root_slot.get() {
            if let Some(primary) = root.primary.get() {
                primary.event(app, &event);
            }
            match event {
                RunEvent::ExitRequested { api, .. } => {
                    if root.request_quit() {
                        api.prevent_exit();
                    }
                }
                RunEvent::Exit => {
                    root.disposed.store(true, Ordering::SeqCst);
                    if let Some(core) = root.core() {
                        core.quit.dispose();
                        core.application.dispose();
                    }
                    PAGE.with(|page| page.borrow_mut().take());
                    REPLIES.with(|map| map.borrow_mut().clear());
                }
                _ => {}
            }
        }
    });
}

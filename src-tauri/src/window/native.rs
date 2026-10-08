use super::{Navigation, WindowAction, WindowState, navigation};
use crate::{
    errors::Failure,
    preferences::{self, Bounds},
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tauri::{
    AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent, Wry,
};
// Hooks run on the native UI thread; T031 owns the non-Send document guard.
pub type Created = Arc<dyn Fn(WebviewWindow<Wry>, u64) -> Result<(), Failure> + Send + Sync>;
pub type Detached = Arc<dyn Fn(u64) + Send + Sync>;
pub struct Primary {
    state: Mutex<WindowState>,
    next: AtomicU64,
    root: PathBuf,
    created: Created,
    detached: Detached,
    persist: Arc<dyn Fn(Bounds) + Send + Sync>,
}
impl Primary {
    pub fn new(
        root: PathBuf,
        created: Created,
        detached: Detached,
        persist: Arc<dyn Fn(Bounds) + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(WindowState::default()),
            next: AtomicU64::new(1),
            root,
            created,
            detached,
            persist,
        })
    }
    pub fn failed(&self, id: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.failed(id);
        }
    }
    pub fn open(self: &Arc<Self>, app: &AppHandle) -> tauri::Result<()> {
        let this = self.clone();
        let handle = app.clone();
        app.run_on_main_thread(move || {
            let _ = this.open_main(&handle);
        })
    }
    fn open_main(self: &Arc<Self>, app: &AppHandle) -> tauri::Result<()> {
        let action = self
            .state
            .lock()
            .map_err(|_| tauri::Error::Io(std::io::Error::other("Native window unavailable.")))?
            .open();
        match action {
            WindowAction::Show(id) => {
                if let Some(w) = app.get_webview_window("primary") {
                    w.show()?;
                    w.unminimize()?;
                    w.set_focus()?;
                } else {
                    self.state.lock().unwrap().destroyed(id);
                    return self.open_main(app);
                }
            }
            WindowAction::Create => {
                let result = (|| -> tauri::Result<()> {
                    // Only an absent/failed document reaches Create. Detach before
                    // native destruction, never when the close control merely hides.
                    if let Some(old) = app.get_webview_window("primary") {
                        (self.detached)(0);
                        old.destroy()?;
                    }
                    let id = self.next.fetch_add(1, Ordering::Relaxed);
                    let url = "dsh-app://shell/"
                        .parse()
                        .map_err(tauri::Error::InvalidUrl)?;
                    let mut builder =
                        WebviewWindowBuilder::new(app, "primary", WebviewUrl::CustomProtocol(url))
                            .title("DeepSeek Harness Desktop")
                            .inner_size(1100., 760.)
                            .min_inner_size(640., 480.)
                            .on_navigation(|url| navigation(url, false) == Navigation::Owned);
                    let screens: Vec<_> = app
                        .available_monitors()?
                        .iter()
                        .map(|m| {
                            let r = m.work_area();
                            let s = m.scale_factor();
                            Bounds {
                                x: r.position.x as f64 / s,
                                y: r.position.y as f64 / s,
                                width: r.size.width as f64 / s,
                                height: r.size.height as f64 / s,
                            }
                        })
                        .collect();
                    let saved = preferences::load(&self.root)
                        .ok()
                        .and_then(|p| preferences::usable_bounds(p.window_bounds, &screens));
                    builder = if let Some(b) = saved {
                        builder.position(b.x, b.y).inner_size(b.width, b.height)
                    } else {
                        builder.center()
                    };
                    let window = builder.build()?;
                    self.state.lock().unwrap().opened(id);
                    let this = self.clone();
                    let w = window.clone();
                    window.on_window_event(move |event| match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            this.persist_window(&w);
                            let _ = w.hide();
                        }
                        WindowEvent::Destroyed => {
                            (this.detached)(id);
                            this.state.lock().unwrap().destroyed(id);
                        }
                        _ => {}
                    });
                    if (self.created)(window.clone(), id).is_err() {
                        self.failed(id);
                    }
                    window.show()?;
                    window.set_focus()?;
                    Ok(())
                })();
                if result.is_err() {
                    self.state.lock().unwrap().creation_failed();
                }
                result?;
            }
            _ => {}
        }
        Ok(())
    }
    pub fn close(&self, app: &AppHandle) -> tauri::Result<()> {
        if matches!(self.state.lock().unwrap().close(), WindowAction::Hide(_))
            && let Some(window) = app.get_webview_window("primary")
        {
            self.persist_window(&window);
            window.hide()?;
        }
        Ok(())
    }
    fn persist_window(&self, w: &WebviewWindow) {
        if let (Ok(position), Ok(size), Ok(scale)) =
            (w.outer_position(), w.inner_size(), w.scale_factor())
        {
            (self.persist)(Bounds {
                x: position.x as f64 / scale,
                y: position.y as f64 / scale,
                width: size.width as f64 / scale,
                height: size.height as f64 / scale,
            });
        }
    }
    pub fn save_bounds(&self, app: &AppHandle) {
        if let Some(w) = app.get_webview_window("primary") {
            self.persist_window(&w);
        }
    }
    pub fn event(self: &Arc<Self>, app: &AppHandle, event: &RunEvent) {
        if matches!(event, RunEvent::Reopen { .. }) {
            let _ = self.open(app);
        }
    }
}
pub fn single_instance(primary: Arc<Primary>) -> tauri::plugin::TauriPlugin<Wry> {
    tauri_plugin_single_instance::init(move |app, _args, _cwd| {
        let _ = primary.open(app);
    })
}

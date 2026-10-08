//! Main-thread document guard. Hiding never touches this object or registration.
use super::Application;
use super::failure::FailureGuard;
use crate::{
    bridge::macos::{Dispatch, Registration},
    errors::Failure,
};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_foundation::{NSString, NSURL, NSURLRequest};
use objc2_web_kit::WKWebView;
use std::{cell::Cell, rc::Rc, sync::Arc};

pub struct Document {
    view: Retained<WKWebView>,
    app: Arc<Application>,
    registration: Option<Registration>,
    _main: MainThreadMarker,
    navigation: Rc<Cell<usize>>,
    failed: Rc<Cell<bool>>,
    _failure: FailureGuard,
}
impl Document {
    pub fn new(
        view: Retained<WKWebView>,
        app: Arc<Application>,
        main: MainThreadMarker,
        on_failure: Rc<dyn Fn()>,
    ) -> Self {
        let navigation = Rc::new(Cell::new(0));
        let failed = Rc::new(Cell::new(false));
        let flag = failed.clone();
        let bridge = app.bridge();
        let pointer = Retained::as_ptr(&view) as usize;
        let failure = FailureGuard::install(
            view.clone(),
            navigation.clone(),
            Rc::new(move || {
                flag.set(true);
                bridge.detach_view(pointer);
                on_failure();
            }),
            main,
        );
        Self {
            view,
            app,
            registration: None,
            _main: main,
            navigation,
            failed,
            _failure: failure,
        }
    }
    pub fn enter(&mut self, dispatch: Dispatch) -> Result<(), Failure> {
        if self.failed.get() {
            return Err(Failure::Connection);
        }
        let runtime = self.app.enter()?;
        // Revoke old transfer/reply authority before even beginning navigation.
        self.detach();
        let registration =
            Registration::install(&self.view, self.app.bridge(), dispatch, self._main)
                .map_err(|_| Failure::Unavailable)?;
        let fragment = format!(
            "desktop-window={}&desktop-runtime={runtime}",
            registration.window_generation()
        );
        registration
            .expect_document(fragment.clone())
            .map_err(|_| Failure::Unavailable)?;
        let request = request(&format!("dsh-app://app/#{fragment}"))?;
        self.registration = Some(registration);
        // SAFETY: owned WKWebView, fixed native URL and main-thread guard.
        let Some(navigation) = (unsafe { self.view.loadRequest(&request) }) else {
            self.detach();
            return Err(Failure::Connection);
        };
        self.navigation.set(Retained::as_ptr(&navigation) as usize);
        Ok(())
    }
    pub fn local(&mut self) -> Result<(), Failure> {
        if self.failed.get() {
            return Err(Failure::Connection);
        }
        self.detach();
        let request = request("dsh-app://shell/")?;
        let Some(navigation) = (unsafe { self.view.loadRequest(&request) }) else {
            return Err(Failure::Connection);
        };
        self.navigation.set(Retained::as_ptr(&navigation) as usize);
        Ok(())
    }
    // Destruction/process failure/native replacement hooks call this, not a
    // renderer unload notification. It cancels transfers, not dsh tasks.
    pub fn detach(&mut self) {
        self.registration.take();
    }
    pub fn window_generation(&self) -> Option<u64> {
        self.registration
            .as_ref()
            .map(Registration::window_generation)
    }
}
fn request(url: &str) -> Result<Retained<NSURLRequest>, Failure> {
    let url = NSURL::URLWithString(&NSString::from_str(url)).ok_or(Failure::Unavailable)?;
    Ok(NSURLRequest::requestWithURL(&url))
}
impl Drop for Document {
    fn drop(&mut self) {
        self.detach();
    }
}

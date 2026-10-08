//! Preserve Wry's navigation delegate while observing native page/process loss.
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, NSObject, NSObjectProtocol, ProtocolObject, Sel},
};
use objc2_foundation::NSError;
use objc2_web_kit::{WKNavigation, WKNavigationDelegate, WKWebView};
use std::{cell::Cell, rc::Rc};
struct State {
    previous: Option<Retained<ProtocolObject<dyn WKNavigationDelegate>>>,
    navigation: Rc<Cell<usize>>,
    failed: Rc<dyn Fn()>,
}
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = State]
    #[name = "DSHDesktopNavigationObserver"]
    struct Observer;
    unsafe impl NSObjectProtocol for Observer {
        #[unsafe(method(respondsToSelector:))]
        fn responds_to_selector(&self, selector: Sel) -> bool {
            (unsafe {msg_send![super(self), respondsToSelector: selector]}) || self.ivars().previous.as_ref().is_some_and(|p|p.respondsToSelector(selector))
        }
    }
    impl Observer {
        #[unsafe(method(forwardingTargetForSelector:))]
        fn forwarding_target_for_selector(&self, _selector: Sel) -> *mut AnyObject {
            self.ivars().previous.as_ref().map_or(std::ptr::null_mut(), |p|Retained::as_ptr(p) as *mut AnyObject)
        }
    }
    unsafe impl WKNavigationDelegate for Observer {
        #[unsafe(method(webView:didFailProvisionalNavigation:withError:))]
        fn provisional(&self, view:&WKWebView, navigation:Option<&WKNavigation>, error:&NSError) {
            self.failure(navigation,error);
            if let Some(p)=&self.ivars().previous && p.respondsToSelector(objc2::sel!(webView:didFailProvisionalNavigation:withError:)) {
                unsafe {p.webView_didFailProvisionalNavigation_withError(view,navigation,error);}
            }
        }
        #[unsafe(method(webView:didFailNavigation:withError:))]
        fn navigation_failed(&self,view:&WKWebView,navigation:Option<&WKNavigation>,error:&NSError){
            self.failure(navigation,error);
            if let Some(p)=&self.ivars().previous && p.respondsToSelector(objc2::sel!(webView:didFailNavigation:withError:)) {
                unsafe {p.webView_didFailNavigation_withError(view,navigation,error);}
            }
        }
        #[unsafe(method(webViewWebContentProcessDidTerminate:))]
        fn terminated(&self,view:&WKWebView){
            self.notify_failure();
            if let Some(p)=&self.ivars().previous && p.respondsToSelector(objc2::sel!(webViewWebContentProcessDidTerminate:)) {
                unsafe {p.webViewWebContentProcessDidTerminate(view);}
            }
        }
    }
);
impl Observer {
    fn notify_failure(&self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (self.ivars().failed)()));
    }
    fn failure(&self, navigation: Option<&WKNavigation>, error: &NSError) {
        // Intentional replacement cancellation is not a crashed document.
        if error.code() == -999 && error.domain().to_string() == "NSURLErrorDomain" {
            return;
        }
        if navigation
            .is_some_and(|n| n as *const WKNavigation as usize == self.ivars().navigation.get())
        {
            self.notify_failure();
        }
    }
}
pub(super) struct FailureGuard {
    view: Retained<WKWebView>,
    observer: Retained<Observer>,
    previous: Option<Retained<ProtocolObject<dyn WKNavigationDelegate>>>,
}
impl FailureGuard {
    pub(super) fn install(
        view: Retained<WKWebView>,
        navigation: Rc<Cell<usize>>,
        failed: Rc<dyn Fn()>,
        main: MainThreadMarker,
    ) -> Self {
        // Main-thread owned view and delegate; no renderer-provided object.
        let previous = unsafe { view.navigationDelegate() };
        let observer = main.alloc::<Observer>().set_ivars(State {
            previous: previous.clone(),
            navigation,
            failed,
        });
        let observer: Retained<Observer> = unsafe { msg_send![super(observer), init] };
        unsafe {
            view.setNavigationDelegate(Some(ProtocolObject::from_ref(&*observer)));
        }
        Self {
            view,
            observer,
            previous,
        }
    }
}
impl Drop for FailureGuard {
    fn drop(&mut self) {
        let current = unsafe { self.view.navigationDelegate() };
        if current.as_ref().is_some_and(|p| {
            Retained::as_ptr(p) as *const () == Retained::as_ptr(&self.observer) as *const ()
        }) {
            unsafe {
                self.view.setNavigationDelegate(self.previous.as_deref());
            }
        }
    }
}

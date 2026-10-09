//! Preserve Wry's navigation delegate while observing native page/process loss.
use crate::window::{Navigation, navigation};
use block2::DynBlock;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, NSObject, NSObjectProtocol, ProtocolObject, Sel},
};
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSError;
use objc2_web_kit::{
    WKNavigation, WKNavigationAction, WKNavigationActionPolicy, WKNavigationDelegate,
    WKNavigationType, WKWebView,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
struct State {
    previous: Option<Retained<ProtocolObject<dyn WKNavigationDelegate>>>,
    navigation: Rc<Cell<usize>>,
    failed: Rc<dyn Fn()>,
    approved: Rc<RefCell<Option<String>>>,
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
        #[unsafe(method(webView:decidePolicyForNavigationAction:decisionHandler:))]
        fn policy(&self,view:&WKWebView,action:&WKNavigationAction,decision:&DynBlock<dyn Fn(WKNavigationActionPolicy)>){
            // Actual WK link activation, never a renderer-supplied gesture flag.
            let request=unsafe {action.request()};
            let native=request.URL();
            let disposition=native.as_ref().and_then(|u|u.absoluteString()).and_then(|s|url::Url::parse(&s.to_string()).ok()).map_or(Navigation::Deny,|url|navigation(&url,unsafe {action.navigationType()}==WKNavigationType::LinkActivated));
            if disposition == Navigation::Owned {
                let target=unsafe {action.targetFrame()};
                let allowed=target.as_ref().is_some_and(|frame| {
                    if !unsafe {frame.isMainFrame()}{return true;}
                    let actual=native.as_ref().and_then(|u|u.absoluteString()).map(|s|s.to_string());
                    let mut approved=self.ivars().approved.borrow_mut();
                    if actual.is_some() && *approved==actual {approved.take();true}else{false}
                });
                if !allowed {decision.call((WKNavigationActionPolicy::Cancel,));return;}
            }
            if disposition != Navigation::Owned {
                decision.call((WKNavigationActionPolicy::Cancel,));
                if disposition==Navigation::External && let Some(url)=native {NSWorkspace::sharedWorkspace().openURL(&url);}
            } else if let Some(p)=&self.ivars().previous && p.respondsToSelector(objc2::sel!(webView:decidePolicyForNavigationAction:decisionHandler:)) {
                unsafe {p.webView_decidePolicyForNavigationAction_decisionHandler(view,action,decision);}
            } else {decision.call((WKNavigationActionPolicy::Allow,));}
        }
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
        approved: Rc<RefCell<Option<String>>>,
        main: MainThreadMarker,
    ) -> Self {
        // Main-thread owned view and delegate; no renderer-provided object.
        let previous = unsafe { view.navigationDelegate() };
        let observer = main.alloc::<Observer>().set_ivars(State {
            previous: previous.clone(),
            navigation,
            failed,
            approved,
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

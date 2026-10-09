use super::*;
use crate::bridge::macos::{Reply, local_reply, native_body, native_sender, reject};
use block2::DynBlock;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, NSObject, NSObjectProtocol, ProtocolObject},
};
use objc2_foundation::NSString;
use objc2_web_kit::{
    WKContentWorld, WKScriptMessage, WKScriptMessageHandlerWithReply, WKUserContentController,
    WKWebView,
};
use std::rc::Rc;
pub type Dispatch = Rc<dyn Fn(Permit, Request, Reply)>;
struct Handler {
    bridge: Arc<Bridge>,
    binding: Binding,
    dispatch: Dispatch,
}
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind=MainThreadOnly]
    #[ivars=Handler]
    #[name = "DSHDesktopShellHandler"]
    struct ShellHandler;
    unsafe impl NSObjectProtocol for ShellHandler {}
    unsafe impl WKScriptMessageHandlerWithReply for ShellHandler {
        #[unsafe(method(userContentController:didReceiveScriptMessage:replyHandler:))]
        fn receive(
            &self,
            _controller: &WKUserContentController,
            message: &WKScriptMessage,
            reply: &DynBlock<dyn Fn(*mut AnyObject, *mut NSString)>,
        ) {
            let result = (|| {
                let sender = native_sender(message);
                let scope = self
                    .ivars()
                    .bridge
                    .admit_local(self.ivars().binding, &sender)?;
                let text = native_body(message)?;
                let request = serde_json::from_str::<Request>(&text).map_err(|_| Denied)?;
                Ok::<_, Denied>((scope, request))
            })();
            match result {
                Ok((scope, request)) => {
                    let response = local_reply(reply, scope.clone(), self.mtm());
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (self.ivars().dispatch)(scope, request, response)
                    }));
                }
                Err(_) => reject(reply),
            }
        }
    }
);
pub struct Registration {
    controller: Retained<WKUserContentController>,
    _handler: Retained<ShellHandler>,
    bridge: Arc<Bridge>,
    binding: Binding,
    name: Retained<NSString>,
}
impl Registration {
    pub fn install(
        view: &WKWebView,
        bridge: Arc<Bridge>,
        dispatch: Dispatch,
        main: MainThreadMarker,
    ) -> Result<Self, Denied> {
        let binding = bridge.bind_owned(view as *const WKWebView as usize, Domain::Shell)?;
        let name = NSString::from_str(&format!("dshShell{}", binding.generation));
        let controller = unsafe { view.configuration().userContentController() };
        let handler = main.alloc::<ShellHandler>().set_ivars(Handler {
            bridge: bridge.clone(),
            binding,
            dispatch,
        });
        let handler: Retained<ShellHandler> = unsafe { msg_send![super(handler), init] };
        unsafe {
            controller.addScriptMessageHandlerWithReply_contentWorld_name(
                ProtocolObject::from_ref(&*handler),
                &WKContentWorld::pageWorld(main),
                &name,
            );
        }
        Ok(Self {
            controller,
            _handler: handler,
            bridge,
            binding,
            name,
        })
    }
    pub fn window_generation(&self) -> u64 {
        self.binding.generation
    }
    pub fn expect_document(&self, fragment: String) -> Result<(), Denied> {
        self.bridge.expect_document(self.binding, fragment)
    }
}
impl Drop for Registration {
    fn drop(&mut self) {
        self.bridge.detach(self.binding);
        unsafe {
            self.controller
                .removeScriptMessageHandlerForName(&self.name);
        }
    }
}

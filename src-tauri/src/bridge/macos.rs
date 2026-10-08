//! WK-only adapter. Install/remove and replies stay on the main thread. Handler
//! registration alone is not admission: WK exposes it to subframes as well.
use super::*;
use block2::{DynBlock, RcBlock};
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, ProtocolObject},
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};
use objc2_web_kit::{
    WKContentWorld, WKScriptMessage, WKScriptMessageHandlerWithReply, WKUserContentController,
    WKWebView,
};
use std::{ptr, rc::Rc};

type ReplyBlock = RcBlock<dyn Fn(*mut AnyObject, *mut NSString)>;
pub struct Reply {
    block: Option<ReplyBlock>,
    permit: Permit,
    _main: MainThreadMarker,
}
impl Reply {
    pub fn complete(mut self, value: &Value) -> Result<(), Denied> {
        if !self.permit.is_current() {
            return Err(Denied);
        }
        let body = serde_json::to_string(value).map_err(|_| Denied)?;
        // Payload chunks are smaller than this. Bootstrap uses T020's 2 MiB cap.
        if body.len() > 4 * 1024 * 1024 {
            return Err(Denied);
        }
        let body = NSString::from_str(&body);
        self.permit.with_current(|| {
            if let Some(block) = self.block.take() {
                block.call((Retained::as_ptr(&body).cast_mut().cast(), ptr::null_mut()));
            }
        })
    }
}
impl Drop for Reply {
    fn drop(&mut self) {
        if let Some(block) = self.block.take() {
            reject(&block);
        }
    }
}
fn reject(block: &DynBlock<dyn Fn(*mut AnyObject, *mut NSString)>) {
    let error = NSString::from_str("Desktop request denied or no longer current.");
    block.call((ptr::null_mut(), Retained::as_ptr(&error).cast_mut()));
}
// A local dispatcher may retain Reply for asynchronous completion on the main
// thread. It cannot move WK reply blocks onto worker threads or evaluate a
// result in whichever document happens to occupy the WebView later.
pub type Dispatch = Rc<dyn Fn(Permit, Request, Reply)>;
struct HandlerState {
    bridge: Arc<Bridge>,
    binding: Binding,
    dispatch: Dispatch,
}
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = HandlerState]
    struct FrameHandler;
    unsafe impl NSObjectProtocol for FrameHandler {}
    unsafe impl WKScriptMessageHandlerWithReply for FrameHandler {
        #[unsafe(method(userContentController:didReceiveScriptMessage:replyHandler:))]
        fn receive(
            &self,
            _controller: &WKUserContentController,
            message: &WKScriptMessage,
            reply: &DynBlock<dyn Fn(*mut AnyObject, *mut NSString)>,
        ) {
            // SAFETY: WebKit calls the declared protocol on its main thread;
            // object types/lifetimes come from that callback, not renderer data.
            let result = unsafe {
                let frame = message.frameInfo();
                let origin = frame.securityOrigin();
                let message_view = message.webView();
                let frame_view = frame.webView();
                let sender = Sender {
                    message_view: message_view
                        .as_ref()
                        .map_or(0, |v| Retained::as_ptr(v) as usize),
                    frame_view: frame_view
                        .as_ref()
                        .map_or(0, |v| Retained::as_ptr(v) as usize),
                    main_frame: frame.isMainFrame(),
                    protocol: origin.protocol().to_string(),
                    host: origin.host().to_string(),
                    port: origin.port() as i64,
                    frame_url: frame
                        .request()
                        .URL()
                        .and_then(|u| u.absoluteString())
                        .map(|s| s.to_string()),
                    view_url: message_view
                        .and_then(|v| v.URL())
                        .and_then(|u| u.absoluteString())
                        .map(|s| s.to_string()),
                };
                // Reject native sender/gates before touching an unauthorized body.
                let allowed = self
                    .ivars()
                    .bridge
                    .state
                    .lock()
                    .map_err(|_| Denied)
                    .and_then(|state| validate_sender(&state, self.ivars().binding, &sender));
                // NSString-only bounded envelope. Body fields never form Sender.
                allowed
                    .and_then(|()| message.body().downcast::<NSString>().map_err(|_| Denied))
                    .and_then(|body| {
                        if body.length() > MAX_MESSAGE_BYTES {
                            return Err(Denied);
                        }
                        let request = parse_request(&body.to_string())?;
                        let permit =
                            self.ivars()
                                .bridge
                                .admit(self.ivars().binding, &sender, &request)?;
                        Ok((permit, request))
                    })
            };
            match result {
                Ok((permit, request)) => {
                    let response = Reply {
                        block: Some(reply.copy()),
                        permit: permit.clone(),
                        _main: self.mtm(),
                    };
                    // Never unwind through Objective-C's callback ABI or log bodies.
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (self.ivars().dispatch)(permit, request, response)
                    }));
                }
                Err(_) => reject(reply),
            }
        }
    }
);

// Keep this guard for the owned document. Drop/reinstall on native navigation,
// document failure or destruction, before admitting a replacement document.
// Names are unique per native generation: old cleanup cannot remove a new handler.
pub struct Registration {
    controller: Retained<WKUserContentController>,
    _handler: Retained<FrameHandler>,
    bridge: Arc<Bridge>,
    binding: Binding,
    name: Retained<NSString>,
}
impl Registration {
    pub fn window_generation(&self) -> u64 {
        self.binding.generation
    }
    pub fn handler_name(&self) -> String {
        self.name.to_string()
    }
    pub fn install(
        view: &WKWebView,
        bridge: Arc<Bridge>,
        dispatch: Dispatch,
        mtm: MainThreadMarker,
    ) -> Result<Self, Denied> {
        let binding = bridge.bind(view as *const WKWebView as usize)?;
        let name = NSString::from_str(&format!("dshBridge{}", binding.generation));
        // SAFETY: borrowed WKWebView and main-thread marker are native handles.
        let controller = unsafe { view.configuration().userContentController() };
        let handler = mtm.alloc::<FrameHandler>().set_ivars(HandlerState {
            bridge: bridge.clone(),
            binding,
            dispatch,
        });
        let handler: Retained<FrameHandler> = unsafe { msg_send![super(handler), init] };
        unsafe {
            controller.addScriptMessageHandlerWithReply_contentWorld_name(
                ProtocolObject::from_ref(&*handler),
                &WKContentWorld::pageWorld(mtm),
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
}
impl Drop for Registration {
    fn drop(&mut self) {
        self.bridge.detach(self.binding);
        // SAFETY: retained controller and unique name are main-thread owned.
        unsafe {
            self.controller
                .removeScriptMessageHandlerForName(&self.name);
        }
    }
}

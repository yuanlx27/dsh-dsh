# T024: frame-aware native WK bridge admission

## Implementation

`bridge.rs` is a small native capability policy. Sender metadata and bindings
are private; no renderer/Tauri command can construct them. The macOS adapter
installs a dedicated `WKScriptMessageHandlerWithReply` on the actual owned
WKWebView; the handler name and window generation are native-minted. It reads:

- `WKScriptMessage.webView` and `WKFrameInfo.webView` identities;
- `WKFrameInfo.isMainFrame`, security-origin protocol/host/port;
- the actual frame request URL and current WebView URL.

Every value must agree with the owned primary app main frame. Shell, foreign,
missing/opaque, child and other-window sources fail closed. Sender/gates are
checked before parsing any body. The bounded NSString JSON envelope has only
window/runtime freshness claims, operation and payload; origin/main-frame label
fields are rejected, not accepted as evidence. Bootstrap requires null payload
and exactly the same native ready/saved-notice gates as HTTP/stream admission.

Permits are non-serializable and capture the native owner/runtime/notice epoch.
Window replacement (even same pointer), gate changes, runtime invalidation and
disposal revoke handles/callbacks and wake cancellation waiters. Runtime
activation is monotonic; a retired authenticated generation cannot be resurrected.
Cross-owner permits cannot address each other's scoped handles. T025/T026 must
retain/check this permit and the native ServiceSession for every owned job.

Replies use WebKit's original frame-bound one-shot promise block, not later
`evaluateJavaScript` on a reused view. The final reply holds a fresh epoch check
through block delivery; stale/dropped replies reject with a fixed non-secret
message. Reply/registration types remain main-thread-only; no unsafe Send/Sync
wrapper is added. Async transport integration must retain these locally and
schedule final completion on the main thread. Unique handler names ensure an old
registration's disposal cannot remove its replacement's handler.

T028/T031 must register the actual primary WKWebView, supply only native runtime
and persisted-notice state, and drop/reinstall the registration before native
navigation/document replacement/failure. Ordinary hide/show retains registration.
This task adds the adapter, not an installed app or a renderer-authorized API.
Local shell command admission remains separate T031 work.

## Executed evidence and limits

- Tests written first: missing bridge module, exit 101 before assertions.
- Six admission/cancellation tests pass: native sender identity/origin/main frame;
  all three operation gates; old window/runtime/notice/reply revocation; monotonic
  activation; closed body claims and bounds; distinct native owners/handle scopes.
- Controlled main-frame guard bypass failed the child-frame assertion, exit 101.
  Restored the guard and reran passing regressions.
- `native-bridge` feature compiles against exact objc2/WebKit versions and links
  actual macOS Foundation/AppKit/WebKit in debug/release test binaries. No new
  versions: block2 0.6.2 is promoted from the existing lock graph. Needed locked
  native crates were downloaded; subsequent checks are locked/offline.
- Selected native-feature debug Rust: **40 pass** (19 library, 8 foundation,
  10 authentication, 3 protocol) plus proxy subprocess. Release library/protocol:
  **22 pass**. Native/default selected Clippy and formatting pass.
- Source policy/tool checks remain green; no TS/JS-layer code was added. Generated
  Tauri ACL schemas are ignored, not committed application source.

**Not executed:** receiving a real WKScriptMessage, actual custom-scheme security
origin/port behavior, queued messages during navigation, original-frame promise
completion, Tauri app registration, installed main/child frame attacks, app/DMG
signing or minimum-OS acceptance. The unit sender records are policy fixtures,
not real WK admission proof; T032 scenarios remain the acceptance gate.

A full `--features desktop --lib` check downloaded exact locked desktop crates
but failed in tauri-build resource copying with `Not a directory`. Its code
copies the `dsh` sidecar into the dev target directory before copying the `dsh/`
resource tree, producing a file/directory collision. Native-bridge feature checks
are separate and green; full desktop build is **not** claimed passing. Rename
one mapping consistently and test staged/installed paths before T031 packaging.

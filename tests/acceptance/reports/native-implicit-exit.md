# Native last-window destruction — retain service without quit prompt

## Observed defect and minimal fix

The actual installed `74dbeca` lifetime test exposed an automatic quit dialog
when the last primary window was destroyed. This is not a user quit request.
Locked `tauri-runtime-wry 2.12.1` explains the distinction in `src/lib.rs`:
lines 4256–4271 handle last-window Destroyed with `ExitRequested { code: None }`;
lines 4299–4314 handle RequestExit with `code: Some(code)`.

`desktop.rs` now prevents the implicit None event directly. Some/code exits retain
existing conservative confirmed-quit handling. No custom event flag, new shutdown
path, renderer authority or upstream modification is needed. The original
failure/Destroyed handling still revokes the document and cancels renderer jobs.
Normal red-close remains hide/retain. This also avoids a spurious quit request
when recovering/replacing a failed primary.

## Installed actual-WK regression

Fresh ad-hoc-signed/verified installed debug candidate:
`/private/tmp/dsh exit install.JPm3sB/DeepSeek Harness Desktop.app`.
Finalizer verified 17 signed nested Mach-O files and 26,761 hashes; native installed
closure verification and deep/strict codesign passed. Host `trochili.local`, arm64,
macOS 27.0.1 build 26A434. Debug fixture consent/entry/Inspector/destruction controls
were enabled. Exact hashes and closed results: `native-implicit-exit-data.json`.
Not production/minimum-macOS/Gatekeeper/DMG qualification.

App/launcher/Node **62244/62303/62304**. Created a real buffered HTTP handle in
entered generation **2**, runtime **1**. Host SIGUSR1 destroyed the primary.
After three seconds actual AX window count was **0**, not a hidden primary and
not a native quit dialog. All captured PIDs remained unchanged. Native File/Open
created the shell; explicit Open Harness entered generation **4**, runtime still
**1**, without selecting Stay or dismissing an automatic quit confirmation.
The fixture transfers only scoped opaque metadata between documents, then removes
its key; closed output never includes handle values or bootstrap/authentication.

Fresh bootstrap and buffered HTTP begin/cancel succeeded. Actual old generation
and old HTTP finish/write were explicitly denied; fresh bootstrap afterward also
succeeded. No timeout was substituted for rejection and no model/task/tool/provider
operation was sent. An early shell AX query lacked rendered Open Harness content;
refocusing the owned app and waiting exposed it. This is not startup-performance
or idle-host timing evidence.

Explicit Command-Q still displayed the native conservative confirmation. **Stay**
kept all three captured PIDs. A second explicit Command-Q followed by **Stop and
Quit** cleaned the complete trio. Thus the change removes implicit destruction
prompts without removing the exercised explicit confirmation/cleanup path.
Display assertion stopped and generated clipboard cleared conditionally.

## Validation and limits

Source regression RED before the one-condition fix, GREEN after. 74 packaging
tests, source policy, strict Node/browser tooling, default Rust tests, desktop
lib/bin Clippy, debug build and release desktop check pass. Existing all-targets
transport-fixture lint issue is unchanged and not waived.

This resolves the observed automatic quit-dialog defect for the exercised actual
native destruction/recreation path. It does not qualify pending mux/late replies,
all OS quit sources, running task/approval/selection/scroll retention or saved-valid
idle-host timing. **T032/T033 remain pending.** Historical pre-fix behavior and
HTTP scope evidence remain in `native-document-lifetime-retry.md`.

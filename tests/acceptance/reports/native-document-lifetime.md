# T032 — host-only document lifetime probe (matrix execution pending)

Adds a debug-only, opt-in **host SIGUSR1** trigger for destruction of the owned
`primary` native WebviewWindow. Both `DSH_DESKTOP_FIXTURE_ROOT` and
`DSH_DESKTOP_FIXTURE_DESTROY=1` are required. Registration is one-shot and the
public Tauri destroy operation runs on the main thread. No renderer command,
general IPC permission, production trigger, process-name termination or private
WK API was added. Existing native Destroyed/detach handling owns revocation.
This control is not a new normal close behavior: normal close still hides.

## Prepared F09 program — not yet qualified

`tests/acceptance/browser/wk-stale.ts` compiles in the separate strict DOM project.
It is intended to be pasted into the owned Inspector twice, before and after
native destruction plus File/Open recreation:

1. Begin a buffered HTTP GET for a nonexistent benign fixture API path and open
   an extra physical mux connection without submitting a logical/business request.
   Retain scoped opaque handles and generation numbers temporarily in a distinct
   localStorage fixture key. Start one pending mux read; do not print handles,
   replies, bootstrap values, credentials or settings.
2. Send host SIGUSR1 only after retaining the closed preparation rows. Verify
   actual native destruction/recreation, not a modified renderer generation or
   a reload rejected by navigation policy. Check the captured owned service PIDs
   remain unchanged; then use native File/Open.
3. Paste again in the fresh entered document. Require a changed window generation;
   separately record runtime-generation continuity. Test actual old-generation
   bootstrap and old HTTP finish/write and mux read/write handles using the fresh
   sender. Bound observation separately from rejection. Test fresh bootstrap and
   HTTP begin/cancel positives around refusals. Remove the temporary metadata.
4. Record whether the old read's resolution marker is absent. Absence alone is
   **not proof of every late-reply ordering scenario**: the old JS context may have
   disappeared. A controlled peer/late reply remains necessary for full F09.

HTTP begin does not dispatch the request. No model adapter/provider/task/tool
operation is encoded. Any accidental old-handle admission would at most issue the
benign unknown GET or an empty mux frame, never a task action. These source/type
checks are fixture preparation, **not executed native stale-handle evidence**.

## Installed control pilot and permission block

Host `trochili.local`, macOS 27.0.1 build 26A434, arm64. Copied installed debug
candidate: `/private/tmp/dsh stale install.OAYll0/DeepSeek Harness Desktop.app`.
Acceptance-only finalizer verified 17 signed nested Mach-O files and 26,761 final
hashes. Native installed closure verification and deep/strict codesign passed;
this is not production/DMG/Gatekeeper qualification.

Observed app/launcher/Node **54646/54695/54696**. Native fixture arming was observed.
System Events could not access window 1 (`-1719`) and then explicitly returned
**osascript is not allowed assistive access** (`-25211`). No keystrokes were
injected, no permission bypass was attempted and the browser preparation program
was never run. Closing OmniWM in the earlier retry does not imply Accessibility
permission remains available in this execution context.

The host-only signal produced exactly one
`fixture-native-primary-destroy-requested` marker. All three captured PIDs were
still present two seconds later. That marker acknowledges a successful public
API destruction request, **not independently observed Destroyed-event completion,
window recreation, retained task state or successful stale-handle rejection**.
With Inspector unavailable, the pilot stopped there.

Owner-only TERM of app 54646 then cleaned the complete captured trio within eight
seconds via the owned launcher lifecycle. The display assertion was stopped.
No claim of a native confirmed-menu quit is made for this pilot.

## Checks and remaining gate

Native-control source regression was RED before implementation and GREEN after.
Packaging/source/strict tooling, default Rust tests, desktop lib/bin Clippy,
debug build and release desktop check pass. The helper and registration are both
compile-time debug-only. The existing all-targets duplicate transport-fixture
Clippy issue is not fixed or waived here.

**T032/T033 remain unchecked.** Restore authorized Accessibility for the launching
terminal context before more Inspector/AX keyboard automation, then execute and
retain the complete two-document F09 sequence. Current fixture/control evidence
must not be substituted for it or for production, other-view/origin, stream-byte,
credential, platform, timing or work-state qualification.

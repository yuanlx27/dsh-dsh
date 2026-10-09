# T032 — installed native acceptance (IN PROGRESS, NOT PASSED)

Host: Apple Silicon, macOS 27.0.1 build 26A434. Implementation checkpoint:
`6fe83ff` (T031); debug-only temporary path diagnostics were removed after use.
All data was isolated fixture data. No provider operation/task was submitted.

## Candidate preparation observations

The ordinary Tauri app build succeeds with local ad-hoc signing, but is **not a
valid frozen-runtime delivery by itself**:

1. Signing changes Node/launcher hashes after the staged inventory was generated.
2. Tauri's resource copier flattens frozen `node_modules/.bin` symlinks.
3. The auto-discovered Rust launcher also appears in bundler signing output.

A temporary acceptance-only TypeScript finalizer restored the frozen symlink
kinds, signed 17 nested Mach-O files, recalculated all 26,761 installed hashes,
then signed the outer app and verified both deep/strict signing and post-sign
hash equality. It did not change the staged closure or download identities.
Source: `/private/tmp/dsh-finalize-candidate.ts`; build output under
`src-tauri/target/aarch64-apple-darwin/{debug,release}/bundle/macos/`.
This procedure is **not** completed T052 packaging automation or DMG qualification.

Installed debug candidate was copied to a path containing spaces:
`/private/tmp/dsh new install.o7F9JE/DeepSeek Harness Desktop.app`.
Fixture data: `/private/tmp/dsh-installed-private.PTpUEZ`.
`src-tauri/examples/verify-installed.rs` reuses the actual native installed
closure verifier; canonical absolute installed paths verify successfully.

Initial launches through `/tmp` failed closed: macOS `/tmp` is a symlink and
Tauri's cached starting-binary resolver intentionally rejects launch paths with
symlinks. Launching the canonical `/private/tmp` path succeeds. No dangerous
symlink feature was enabled and no installed-to-staging fallback was added.

## Executed installed debug checks

- Actual owned WK shell admission/original-frame replies and native verified,
  authenticated, prepared Ready.
- Actual approved Harness entry/bootstrap, packaged R queue/original frontend
  module load, admitted native HTTP and authenticated `/api/remote.mux` round trip
  to harmless nonexistent `_desktop_fixture/unknown`, producing the remote
  terminal failure marker rather than a carrier error.
- Default upstream credential file mode: `600`. After full owned smoke shutdown,
  changing only the fixture file to `644` caused startup to exit and no Ready/
  Harness authority; mode remained `644` (not silently repaired). Restoring `600`
  and explicitly relaunching restored Ready/HTTP/mux. Native failure category was
  `Exited`; specific corrective UX still requires review. Contents were never
  recorded and no real key was used.
- Separate unauthenticated client, same-service Origin/Host: index `401`, API
  `401`, mux upgrade `401`; public pinned frontend JS `200`. Invalid Origin API
  `403`. `lsof` showed the owned listener on literal `127.0.0.1` only. Port was
  discovered from the owned Node PID, never from a token-bearing URL.
- Smoke parents/launchers/Node were cleaned; fixture file restored to `600`.
  This was owned unexpected-exit cleanup, **not** native confirmed Quit acceptance.

## CSP defect fixed and visible UI/quit checks executed

The actual WK Inspector identified the white-window cause: pinned R's Cordis
expression evaluator uses `new Function` during initial module evaluation, and
the original desktop CSP rejected it. No upstream bytes/loader were replaced.
`protocol::content_security_policy` now adds **`'unsafe-eval'` only to host `app`**;
shell, absent and other hosts retain eval denial. This is a real CSP relaxation:
it permits string evaluation in the Harness, not solely a particular R function.
Frozen source admission and actual-WK authority checks do not make arbitrary
evaluated code harmless. External script/connection schemes remain denied,
script inline permission was not added, and object/base/form restrictions remain.
Future XSS/string-evaluation hardening must preserve R behavior or be an explicit
upstream compatibility change, not silently assert eval is sandboxed.

Regression: new pure Rust policy test RED (exit 101 before implementation), GREEN;
74 default Rust cases plus auth proxy subprocess, 67 packaging cases, desktop
Clippy, source/tools checks and production release binary build passed.

Actual unbundled debug WK with the fix visibly rendered the original R Preview
Notice, Continue, empty-key setup (Save disabled), Configure later and composer.
No key was entered and no task/provider operation was submitted. Native close
and File/Open retained an unsent fixture draft and the same launcher PID.
Cmd+Q displayed the native all-session warning with Stay as the default; Return
selected Stay and kept the owned processes. A second Cmd+Q plus explicit Stop
and Quit removed app/launcher/Node (`27431/27468/27469`).

The rebuilt debug executable was inserted into the owned candidate, nested/final
hashes and outer ad-hoc signature regenerated, and actual installed verifier and
`codesign --verify --deep --strict` passed. A canonical copy containing spaces,
`/private/tmp/dsh csp install.q0CVEU/DeepSeek Harness Desktop.app`, PID `29232`,
visibly rendered the R credential setup/composer **without Inspector enabled**.
Actual native red-close/File/Open was repeated ten times: hidden AX window count
was zero after each close; the unsent `installed-retained-fixture-NOT-SUBMITTED`
draft was identical each time and owned launcher/Node IDs remained unchanged.
Default Return/Stay preserved the process trio; explicit Stop and Quit then
removed all three, with zero owned-survivor rows after seven seconds. The installed
warning uses the bundled pinned-R icon (unbundled executable uses a generic icon).

These ten draft-only cycles do **not** qualify full SC-007: selection/scroll,
pending approvals, active transfer identity and ten confirmed quits are still
pending. Installed saved-setup production release trials have not yet executed.
No evidence above upgrades debug consent seeding into native notice UX acceptance.

## Earlier environment blockers and remaining qualification

A later screen capture returned success but contained only a black image.
System Events inspection of the owned app initially timed out. After user granted
permissions, screenshot and non-assistive process enumeration worked. Assistive
operation still failed explicitly: `osascript is not allowed assistive access`
(-1728); the macOS permission prompt identified the responsible app as **kitty**.
`open -a` activated the candidate without bypassing permissions. Its captured
window was blank white; entry-load/mux markers therefore do not prove a usable
rendered frontend. That render defect was subsequently diagnosed/fixed above.
After the additional kitty Accessibility grant, assistive window inspection and
key posting succeeded. A debug-only fixture `DSH_DESKTOP_FIXTURE_INSPECT=1` uses
Tauri's existing native devtools API; the shell and Inspector were visibly
captured, confirming this permission gate resolved. During subsequent checks the
display became black again and application window lists became empty; an owned
Cmd+Q/alert query timed out. Main-thread sampling showed the normal AppKit event
loop, not an observed Rust mutex deadlock. Quit outcomes were not established;
owned probes were terminated and cleaned. An active unlocked desktop is still
required. No new permission bypass or production renderer authority was added.
No focused key outcome, user acknowledgement, close/reopen state retention, native Stay/Stop
and Quit interaction, or accessibility outcome is inferred from debug markers.
An unlocked interactive desktop and working UI automation/manual operation are
needed for those checks.

Debug fixture consent seeding/auto-entry is not human notice/setup acceptance.
Subsequently a production release candidate was locally signed/verified and copied
to `/private/tmp/dsh release install.JMNtfi/DeepSeek Harness Desktop.app`.
Standard HOME isolation used `/private/tmp/dsh-release-home.j6dmz2`, not debug
fixture overrides; PATH was `/usr/bin:/bin` without host Node/dsh. Tauri's existing
`dirs::data_dir` resolved service/preferences under that private HOME; this is not
a clean macOS account or an uninstalled-host-runtime claim. Actual shell notice
Dismiss kept Open Harness disabled; explicit Save acknowledgement enabled it,
then UI Open rendered R Preview Notice/setup/composer. No real key was entered.
Preference/credential files were `600`. Production quit and saved-root relaunches
were executed; the measured empty-key baseline is separately reported in
`lifecycle-performance.md`, not upgraded to valid task-ready setup.

No clean-account, DMG, minimum macOS 14,
credential save/replace/remove/per-operation flow, complete
actual child/external/other-window/stale-frame/general-IPC rejection matrix,
upload/download/Stop, remote-machine admission, all-session quit, or T033 timing
trials has been qualified. No Developer ID/notarization/public Gatekeeper claim.

**T032 remains unchecked; US1/release is not qualified.** These observations do
not authorize moving test procedures or adapter fixtures into the passed column.

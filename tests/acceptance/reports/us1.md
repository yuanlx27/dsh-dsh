# T032 — installed native acceptance (IN PROGRESS, NOT PASSED)

Host: Apple Silicon, macOS 27.0.1 build 26A434. Implementation checkpoint:
`6fe83ff` (T031), CSP correction `d1bda7b`, lifecycle baseline `e6570d6`.
Debug-only temporary path diagnostics were removed after use. All data was
isolated fixture data. Earlier entry probes submitted no task/provider operation;
the later credential section below uses a controlled loopback model only.

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
pending for the complete work-state matrix. Production release baseline trials
subsequently executed as described in `lifecycle-performance.md`.
No evidence above upgrades debug consent seeding into native notice UX acceptance.

## Executed production credential flow (controlled model, partial)

The same installed production release ran with standard isolated HOME/PATH and
no debug overrides. Actual R Models settings UI, actual native HTTP/mux carrier,
and the real bundled upstream credential/settings/LLM implementations were used.
A temporary Node TypeScript **loopback model resource**, not a replacement engine,
accepted two public synthetic test identities. No public provider was contacted,
no real credential was entered, and it never emitted a tool/action request.

- Native focused input saved a synthetic key and loopback Base URL in R's
  DeepSeek editor. Its status confirmed Saved/API key configured; the credential
  file remained `600`. R's Messages adapter successfully streamed the fixture
  answer into the conversation; R also made its own title-generation request.
- Without restart, the same app/launcher/Node (`35872/35885/35886`) saved a
  replacement through R's editor. The next Messages operation used replacement
  authentication and completed. The fixture recorded only credential categories,
  never Authorization/x-api-key values or request bodies. This proves per-operation
  resolution for this exercised adapter, not every adapter or secret source.
- R's Custom model API form created `desktop-fixture` using OpenAI Chat
  Completions. Fetch available models returned its one fixture model; Add selected
  and Create provider stored the derived reference. Explicit Delete confirmation
  removed that provider and its derived reference; unrelated official reference
  remained configured and file mode remained `600`. Presence/removal assertions
  emitted booleans only, never the credential file.
- After native quit, explicit installed relaunch (app `37958`) reached the actual
  composer **without** the missing-key setup dialog. A further Messages request
  succeeded with replacement authentication. Explicit native Stop and Quit was
  observed; the current owned trio disappeared. No receipt-only Ready marker was
  substituted for rendered output.

Sanitized fixture events are in `credential-flow-data.json`: two initial Messages
operations, a replacement Messages operation, one model discovery, then a saved
replacement Messages operation after relaunch. The native experiment used
`/private/tmp/dsh-controlled-model.ts`; the equivalent loopback protocol resource
is retained as `tests/fixtures/controlled-model.ts` with typed tests (missing
module RED exit 1, then GREEN). Unit resource tests do not substitute for the
native observations above or qualify a real model/provider.

**Upstream removal limitation / release gap:** deletion cleanup is intentionally
limited to a user-layer profile whose reference is exactly the derived target.
The composition-owned official DeepSeek row has no independent Remove key action
in this R UI; its non-derived/shared reference was deliberately not edited on
disk or cleared by a new shell credential UI. This exercised custom-reference
removal is not a claim that every configured credential can be removed through
that page. Remaining required removal paths need an upstream-supported operation
and actual native verification; preserve upstream's ownership policy.

## Cleanup audit and app-only crash probe

A broader final process-path scan found **two earlier isolated pilot services**
(`32486/33676`, started 13:42:16/13:54:39), parent PID 1, still listening. They are
not among the final twenty-row dataset's captured Node IDs. Cause is unresolved;
combined test-runner process-group termination is a possibility, not a finding.
The successful per-trial captured-trio checks did not prove global absence of
older pilot survivors. Ownership was verified through exact installed Node/CLI/
web flags and the exclusive isolated working directory; only those two known
experiment PIDs received TERM and disappeared. No process-name scan was killed.

A separate production **app-only SIGKILL** probe then observed the complete trio
`39595/39612/39613`, killed only the app, and verified app/launcher/Node all gone
within the eight-second observation. This supports owner-pipe EOF cleanup in that
case; it does not resolve the older pilots or qualify simultaneous launcher death.
The provider/display assertions were stopped; final candidate-path process scan
had no owned app/launcher/Node survivors. Keep full lifecycle qualification pending.

## Subsequent actual WK negative probe

`native-wk-negatives.md` records a separate unbundled debug/native Inspector probe:
actual inherited-child sender denial, two positive bootstrap admissions across
negative-envelope attempts, and CSP-blocked/incompletely captured general IPC.
This is not installed production or a complete per-case reply/ACL matrix.
Shared-desktop concurrent editing stopped keyboard automation; the observed owned
trio was then cleaned via owner termination.

After the user reported closing OmniWM, the corrected typed observer retained
all ten per-case outcomes on actual WK and then a re-signed/verified installed
debug candidate. Five Harness negatives were rejected, three known Tauri commands
returned strictly classified permission denial, and two main bootstrap positives
resolved. Native counters independently showed one child-sender denial and only
two bootstrap admissions. See `native-wk-installed-retry.md` and its sanitized
JSON for exact scope, hashes, observed identities and pilot corrections. Explicit
native Stop and Quit cleaned the final captured trio. No model task/key or provider
operation was submitted; debug consent/Inspector are not production notice evidence.
T032 remains pending for the remaining matrix and production/platform gates.

`native-document-lifetime.md` describes the subsequent debug-only host destruction
control and prepared two-document stale-handle program. A verified installed
pilot observed the native destroy request, but Accessibility refusal (`-25211`)
prevented browser preparation/recreation; no stale-handle outcome is qualified.
Owner-only termination cleaned its captured trio. Further interactive automation
requires restored authorized Accessibility in the launching terminal context.

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
complete credential removal/adapter coverage, actual child/external/other-window/
stale-frame/general-IPC rejection matrix, upload/download/Stop, remote-machine
admission, ongoing-work/pending-approval all-session quit or valid/idle-host SC-002
qualification has been completed. No Developer ID/notarization/public Gatekeeper claim.

**T032 remains unchecked; US1/release is not qualified.** These observations do
not authorize moving test procedures or adapter fixtures into the passed column.

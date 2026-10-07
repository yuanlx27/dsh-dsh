# Quickstart and End-to-End Validation

## Status and prerequisites

This repository currently contains design artifacts only. The following npm
scripts are implementation deliverables, not existing runnable commands yet:
`runtime:prepare`, `runtime:verify`, `test:packaging`, `tauri:dev`,
`tauri:build`. Do not treat missing scripts as successful validation.

Implementation prerequisites:
- macOS Apple Silicon, Xcode Command Line Tools, pinned Rust toolchain;
- build-host Node 24.21.0 and npm; these are not installed-user prerequisites;
- registry access while preparing the locked runtime, valid DeepSeek test key,
  disposable Git workspace and controlled task/approval fixtures;
- Developer ID signing/notarization credentials for distribution validation;
- named release-test host from [plan](plan.md), plus macOS 14 Apple Silicon
  minimum-OS runner. Use native WKWebView acceptance, not only a browser.

Use fixture credentials and never store real keys or raw service URLs in Git.
Tests that intentionally edit files/run commands must use disposable directories.

## Planned developer commands (from repository root)

```sh
npm ci
npm run runtime:prepare
npm run runtime:verify
cargo test --manifest-path src-tauri/Cargo.toml
npm run test:packaging
npm run tauri:dev
npm run tauri:build -- --target aarch64-apple-darwin --bundles app,dmg
```

Expected: locked build-time downloads only; matching full desktop/dsh version;
bundled Node/dsh and complete frontend/native runtime; tests passing; generated
app/DMG. `runtime:verify` must fail on a deliberately mismatched manifest/package.
Development success is not a substitute for installed signed-build acceptance.
Transport tests must cover actual WKWebView frame identity and both native HTTP
and WebSocket paths; Electron-only tests do not prove the Tauri adaptation.

## Installed-build scenarios

Install the signed/notarized DMG on a clean supported account without separately
installed Node, npm or dsh. For offline checks, disconnect external networking
after installation. Keep model-backed tests online.

| Scenario | Steps | Expected evidence |
|----------|-------|-------------------|
| First launch | Open app, dismiss then acknowledge notice, configure key in upstream Settings, select fixture workspace | No task-capable view before acknowledgement; no terminal/browser required; visible workspace and usable composer |
| Runtime closure | Launch offline with no host runtime, from install path with spaces | Local setup/history UI loads; no runtime fetch; model networking failures are explicit |
| Credential lifecycle | Save/replace/remove fixture key, relaunch; try invalid key; inspect file mode; test overly permissive file while fully quit | Upstream plaintext store retained with `0600`; overly permissive file rejected; upstream settings concealment, retention/removal and corrective errors work; shell does not inject keys into conversation content or expose them in shell diagnostics/startup views; no blanket redaction of arbitrary upstream output; service tokens/cookies remain native-only |
| Controlled task | Summarize fixture project; separately exercise rapid repeated input, identical text, upstream queued/steered input and reconnect/reopen | User-input admission follows pinned dsh behavior; no shell text deduplication or startup lock; forwarding/reconnect/reopen introduce no repeated submission or automatic replay; streamed response/actions and accurate outcome |
| Approval | Force approval-required file/command action; wait, deny; repeat and allow | Nothing executes before allow; denial never executes; policy/action/target visible |
| Stop | Stop during streaming and an already-started controlled action | Acknowledgement <=1 second; no new actions; actual in-flight outcome reported, no claim of rollback |
| Workspace change | Switch visible workspace while multiple sessions have work underway; exercise any upstream prompts | Switching and task continuation/stopping match the pinned dsh release; no shell session inspection, extra confirmation or bulk-stop policy |
| Background | Set draft/selection/scroll context; close with Command+W/close control during task or pending approval; reopen via Dock/repeated launch | Same retained window/document, process and session; no closure-induced draft/selection/scroll reset, transport detach, Stop, approval decision or duplicate task |
| Full quit | Run work in multiple sessions/workspaces; quit from menu/Command+Q with window present and absent; choose Stay then Stop and Quit | Native warning covers the entire owned dsh service, not only the visible session; Stay leaves all work untouched by the shell; confirmed quit waits for owned service exit without claiming undo |
| Recovery | Force shell exit, separately force service exit; reopen/retry | Persisted records readable, unresolved task interrupted; pending approval not granted; no replay, old owned runtime cleaned up |
| Missing workspace | Move fixture directory, reopen the original session, inspect upstream options; select a new workspace separately, then restore the original directory | History, errors and continuation follow the pinned dsh release; no shell directory replacement, session migration or reassociation; a new workspace does not promise transfer of the original conversation |
| Bundle mismatch | Use test build with different dsh/desktop prerelease or damaged resource | Packaging/startup fails before task-capable navigation; supported version and next action shown |
| Port and repeat launch | Occupy 3080; repeatedly launch/reopen/Retry | Random free loopback port; one runtime generation only |
| Shell-mediated access | Inspect app-origin UI, private token exchange and native HTTP/live-stream forwarding; exercise uploads and binary responses | Packaged `dsh-app://app` UI; native cookie ownership; normal upstream flows work without token/cookie in renderer state or WebView cookie jar; byte fidelity and cancellation preserved |
| Native boundary | Attempt general shell/filesystem/network calls; invoke transport from external page, other window and child frame; follow external link | General authority denied; frame-aware transport checks reject unauthorized callers; external link receives no auth |
| Service admission | Use a separate unauthenticated browser/client for index, API and WebSocket routes; try direct connection from another computer | Session/task/approval APIs rejected; remote direct connection unavailable; public static resources permitted; no claim of denial for stolen/forged valid authentication |
| Transport lifetime | Hide/show during transfer/live stream; separately destroy/replace/fail document or Retry and send callbacks from invalidated generations | Normal hide/show retains authorized transport/document authority; destruction/page replacement/failure invalidates affected handles without stopping dsh work; runtime failure/retry revokes old authentication/handles; stale callbacks denied |
| Network scope | Capture idle/setup/task traffic with fixture destinations; inspect disabled surfaces | No product telemetry, feedback export or custom plugin installation; project/task data only reaches permitted destinations |
| Minimum OS | Install/run signed build on macOS 14 Apple Silicon | Runtime/native addons/window/lifecycle work at advertised minimum |

Use [lifecycle contract](contracts/desktop-lifecycle.md) and
[bundle contract](contracts/runtime-bundle.md) for exact behaviors and
[data model](data-model.md) for ownership/state assertions. Compare controlled
task, Stop, approvals and workspace switching against the same packaged upstream
version outside the shell, using a separate fixture data root. A mismatch or
upstream requirement gap blocks release; do not silently implement a second engine.

## Credential and shell-mediated access checks

Use only disposable fixture data. Record statuses and permissions, not credential
contents, launch-token URLs, Cookie/Set-Cookie values or raw network payloads.
Set `DSH_TEST_HOME` to the installed app's disposable dsh data root and
`DSH_TEST_PORT` to its observed numeric listener port; do not obtain them by
copying a token-bearing URL into shell history.

```sh
stat -f '%Lp' "$DSH_TEST_HOME/.credentials.yaml"
curl --noproxy '*' --silent --output /dev/null --write-out '%{http_code}\n' \
  "http://127.0.0.1:$DSH_TEST_PORT/"
curl --noproxy '*' --silent --output /dev/null --write-out '%{http_code}\n' \
  --request POST --header 'Content-Type: application/json' --data '{}' \
  "http://127.0.0.1:$DSH_TEST_PORT/api"
curl --noproxy '*' --silent --output /dev/null --write-out '%{http_code}\n' \
  --http1.1 --max-time 5 --header 'Connection: Upgrade' \
  --header 'Upgrade: websocket' --header 'Sec-WebSocket-Version: 13' \
  --header 'Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==' \
  "http://127.0.0.1:$DSH_TEST_PORT/api/remote.mux"
```

Expected: `600` and unauthenticated rejection (`401`) for index, API and stream
upgrade. Supplying a same-service Origin/Host must not bypass authentication;
cross-site/invalid Host tests yield `403` at protected API admission. From another
computer, connecting to the Mac's LAN address at the service port must fail.
Public non-sensitive assets may return successfully; that is not a failed gate.

Fully quit the app before the permission-negative test. In this disposable
fixture only, run `chmod 644 "$DSH_TEST_HOME/.credentials.yaml"`, relaunch and
verify startup rejects the file with a sanitized corrective action. Fully quit,
restore `chmod 600 "$DSH_TEST_HOME/.credentials.yaml"`, and retry explicitly.
No script or shell credential migration should repair/rewrite the file silently.

Native fixtures must attempt bridge use from another window, external origin,
and same-origin child frame; all are rejected before any authenticated dispatch.
Also test malformed/absolute service targets, another port, injected auth/trust
headers, redirects, oversized bodies, cancellation, binary/multipart responses,
stream ordering/uplink/downlink and stale generation/window handles. Verify tokens
and cookies are absent from renderer boot/state, cookie jar, transport replies
and shell diagnostics. Authorized main-frame work must still pass SC-003.

Same-user credential theft or cookie forgery is an explicitly accepted limitation,
not an expected-denial fixture or isolation claim. The shell grants a narrow
transport capability to authorized app content; compromised authorized code is
not sandboxed away from the dsh operations that content can request.

## Official desktop comparison protocol

This protocol validates A1–A6 in the [plan](plan.md#official-desktop-alignment-fr-022fr-024)
and FR-022–FR-024. Current status for all six runtime comparisons: **pending**.
The present update inspected source only; it did not build or execute either app.

### Reference setup and prerequisites

Use two isolated disposable data/workspace roots: one for a verified official
application at **O** (`5badb15009ae1756c3afe0ae0cef1faafc290ccc`, desktop
`0.2.1-alpha.1`), one for the product with bundled **R** (`0.2.0-rc.2`, source
`639ed015397290b3745d163aafe02ffee4aa3f84`). Do not run both against the same
live session store. Record the official artifact's provenance/build metadata;
a similarly named release or current `master` is not proof it was built from O.

For reproducible unit-level reference evidence, use an isolated upstream source
checkout and O's documented build/test prerequisites, including its declared
pnpm 11.7.0 and native build tooling. These are reference-test host dependencies,
not dependencies added to the product. Example preparation from a scratch directory:

```sh
git clone https://github.com/deepseek-ai/deepseek-harness.git official-reference
git -C official-reference checkout --detach 5badb15009ae1756c3afe0ae0cef1faafc290ccc
git -C official-reference rev-parse HEAD
cd official-reference
pnpm install --frozen-lockfile
pnpm run build:native-system
pnpm exec vitest run apps/desktop/tests/backend-controller.spec.ts \
  apps/desktop/tests/quit-confirmation.spec.ts \
  apps/desktop/tests/keyboard.spec.ts \
  apps/desktop/tests/fatal-recovery.spec.ts \
  apps/desktop/tests/desktop-build-version.spec.ts
```

Expected: exact checkout identity and recorded test results for the relevant
source behavior. These commands have **not** been run in this repository/session.
If the upstream prerequisites or build are unavailable, record that evidence as
pending rather than silently changing O. A passing upstream unit test is useful
reference evidence but does not prove product WKWebView, process lifetime or
installed-app behavior. A source-only read is not a passing comparison.

Use the product's planned developer commands above after implementation; they
are still deliverables, not currently existing scripts. Installed-app comparisons
require the actual product build and verified official artifact at O. For
unknown-state/late-response tests that cannot be induced safely in the official
UI, use executed deterministic official tests and matching product lifecycle
unit fixtures as reproducible behavior evidence, recording their limited scope.
Native zero-window/dialog/menu behavior still requires installed-app tests.

### Comparative scenarios

For each row, run its fixture on both sides, record observed outcomes separately,
then judge the product against the planned adaptation, not against identical UI.

| Decision | Controlled actions | Official reference expectation | Product expectation / allowed difference |
|----------|--------------------|--------------------------------|-------------------------------------------|
| A1 Launch/readiness | Launch with saved setup; reopen during startup; induce auth/startup failure; inspect HTTP/stream forwarding and renderer-visible state | One Host; native cookie exchange; packaged app-origin UI; authenticated HTTP forwarding and main-window WebSocket header injection; cleanup before retry | One Web CLI runtime; Rust-held cookie; packaged app-origin UI and frame-aware native HTTP/stream adapter using upstream hooks; no Electron header API assumption; 15-second failure ceiling; no replay or renderer cookie |
| A2 Window lifecycle | Run a fixture task, then test a pending approval; close and reopen in 10 cycles | Main window hidden, same document and Host continue | Normal close retains and hides the window/document; reopen shows/focuses it with no closure-induced draft/selection/scroll reset or transport detach. Same service/session/task/approval continue; no task duplication or permission decision. Recreation after actual destruction/page failure has no unsaved-context guarantee |
| A3 Quit confirmation | Run the 10-trial matrix below; also test idle quit and late responses/dialog failure using controlled unit fixtures | Active/scheduled or unknown work warns; idle inspected work may quit without a prompt; repeated requests join one decision | Always prompt while service is alive; Stay preserves work, confirmed quit waits for exit; no late or failed dialog grants quit. Extra idle prompt is intentional |
| A4 Menus/shortcuts | Invoke matching menu/keyboard actions with local content then Harness content focused, including contextual page state | Official Close Page can route through contextual shortcut handling; Quit shares its native decision | Explicit native Close Window/Command+W and Quit/Command+Q have matching outcomes; upstream task/page shortcuts remain upstream-owned; no new shortcut customization/native shortcut bridge |
| A5 Error recovery | Induce spawn/readiness/connection failure and service exit with a known test credential; select explicit recovery | Explicit error/recovery decision and cleanup; report text/path or plugin repair may be offered | Categorized non-secret local explanation/Retry, cleanup before new generation, readable persisted history, no replay. Report export/plugin repair omitted; test key absent from shell diagnostics |
| A6 Release compatibility | Inspect full versions/About; attempt mismatched prerelease and damaged/missing packaged resource | Shared immutable release/package identity validated by official descriptor/verification tests | Tauri manifest/build/startup checks reject mismatch before task input, full desktop/dsh equality at R; O and R versions intentionally differ. No reference-only runtime upgrade |

**SC-009 quit matrix**: two trials for each of five conditions (10 total):
(1) active task with window open, (2) active task with zero windows,
(3) pending approval with zero windows, (4) unknown work status while the owned
service is alive, and (5) repeated menu/Command+Q requests before the decision
resolves. In each product trial choose Stay first and verify continued service,
work and approval state; request again and confirm quit, verifying exit completes
only after owned service cleanup. Include repeated requests in condition 5 and
assert at most one unresolved confirmation. The conservative product predicate
requires no task inspector; unknown-state behavior uses its existing service state.
If a reference condition requires a deterministic official test instead of the
installed UI, record that limitation, and do not count it as native UI evidence.

### Evidence records and review gate

Keep a sanitized record per A1–A6: O/R identities, product and official build
provenance, platform/fixture, actions, expected and separately observed outcomes,
intentional differences, evidence location, reviewer and `pending | passing |
failing` status. Reuse the existing acceptance-report destination; no evidence
service or runtime storage is added.

A comparison passes only after relevant product and official behavior evidence
exists, required outcomes pass, and all differences match the recorded decision.
Untested/missing evidence remains pending, unexplained divergence fails, and
upstream tests alone cannot establish product compliance. Confirm all six rows
are complete before affected implementation (SC-008 planning gate); confirm all
six comparisons pass before an alignment-complete release claim. Changed O/R
or changed behavior invalidates affected prior results until reviewed/retested.

## Quantitative release checks

- **SC-001 / SC-006**: >=10 target developers; >=90% submit within 5 minutes
  without help, >=90% complete/recover/identify task state, >=80% rate visibility
  and control >=4/5. Apply the SC-001/SC-006 rubric: first submission ends with
  dsh receipt acknowledgement, not a model answer; summary workflow success
  means locating dsh's reported outcome, not assessing answer quality. Retrieval
  requires the original conversation and correct workspace identification;
  awaiting-approval/completed answers use dsh's reported state as ground truth.
  Give uniform task briefs, credentials and project locations without procedural
  hints. Assisted participants stay in the denominator and do not count as
  independently successful; the SC-006 90% outcome requires all three activities.
- **SC-002**: 20 saved-setup launches, >=19 usable within 10 seconds; record
  process-start and usable timestamps. End when the workspace is visible,
  service connection established and FR-002 task readiness satisfied. Use saved
  configuration, current notice acknowledgement, accessible workspace and valid
  configured credentials; exclude installation, first-use setup, user input time
  and model generation. Use the named otherwise-idle release-test environment.
- **SC-003 / SC-004**: 20 controlled task runs, each with observable response
  and action-status updates; >=19 runs must have every recorded relevant update
  visible within 1 second of arrival from dsh at the desktop-owned receiving
  boundary. Any recorded late update fails that run; do not score individual
  updates as the denominator. Exclude model generation and earlier dsh processing.
  Every Stop acknowledgement must be visible within 1 second of application
  receipt of the user's Stop request, with no 95% exemption. Every approval-
  required/denied action obeys the permission gate. No shell task engine is added.
- **SC-005**: 20 normal relaunches and 10 forced exits retain all previously
  persisted records without replay; >=19/20 openings of a 1,000-message fixture
  within 2 seconds. With application/dsh already connected, measure from receipt
  of the session-open request until the target session is identified, its first
  history screen is readable and browsing is available; do not require every
  message to appear simultaneously or include application startup. Use the named
  otherwise-idle environment; no shell-owned history cache/reader is required.
- **SC-007**: 10 close/reopen cycles retain service identity and work; 10
  confirmed quits stop it; every declined quit preserves it.
- **SC-008**: all six alignment decisions complete before affected implementation;
  100% of adaptations/exclusions explain reason and user impact; all adopted/
  adapted comparisons pass at release review with no unexplained differences.
- **SC-009**: run the 10-trial quit matrix above; at most one unresolved dialog,
  every declined quit preserves work, every confirmed quit awaits owned exit.
- **SC-010**: every unauthorized window/origin/frame fails bridge admission; all
  unauthenticated session/task/approval request and live-stream routes reject;
  remote direct connection fails; no auth token/cookie appears in renderer-visible
  state or shell diagnostics. Run the access checks above on the installed build.

## Shell accessibility qualification

For NFR-001, exercise shell-owned startup, failure/retry, safety notice and quit
confirmation without a mouse. Record confirmation/cancellation/retry reachability,
visible focus on custom controls, dialog focus entry/restoration, meaningful names
and non-color-only status/error information. On supported macOS, record VoiceOver
identification of controls, notices and key state changes. Check that embedding
preserves dsh's existing keyboard and assistive-technology capabilities; do not
claim product-wide WCAG conformance or require a replacement upstream UI.

Record app/dsh/Node versions, manifest/build hash, OS/hardware/power conditions,
fixture IDs, timestamps, owned PID/process-group/listener evidence, sanitized
network findings and pass/fail results. Never include credentials or authenticated
URLs in reports. Inspect signatures/notarization and generated plist version
mapping. Do not claim release compliance from this design-only phase.

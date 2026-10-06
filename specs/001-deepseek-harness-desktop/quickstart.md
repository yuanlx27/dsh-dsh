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

## Installed-build scenarios

Install the signed/notarized DMG on a clean supported account without separately
installed Node, npm or dsh. For offline checks, disconnect external networking
after installation. Keep model-backed tests online.

| Scenario | Steps | Expected evidence |
|----------|-------|-------------------|
| First launch | Open app, dismiss then acknowledge notice, configure key in upstream Settings, select fixture workspace | No task-capable view before acknowledgement; no terminal/browser required; visible workspace and usable composer |
| Runtime closure | Launch offline with no host runtime, from install path with spaces | Local setup/history UI loads; no runtime fetch; model networking failures are explicit |
| Credential lifecycle | Save/replace/remove known test key, relaunch; try invalid key | Masked settings; retention/removal work; invalid key yields correction; key absent from history and shell-authored diagnostics |
| Controlled task | Summarize fixture project; submit twice rapidly | One submitted request/active task, streamed response/actions and accurate outcome |
| Approval | Force approval-required file/command action; wait, deny; repeat and allow | Nothing executes before allow; denial never executes; policy/action/target visible |
| Stop | Stop during streaming and an already-started controlled action | Acknowledgement <=1 second; no new actions; actual in-flight outcome reported, no claim of rollback |
| Workspace change | Attempt change while running; cancel, then explicitly stop/leave | Confirmation required; cancel preserves current workspace/work; no silent reassociation |
| Background | Close with Command+W/close control during task or pending approval; reopen via Dock | Same process and session; no Stop, approval decision or duplicate task |
| Full quit | Quit from menu/Command+Q with window present and absent; choose Stay then Stop and Quit | Native confirmation works in both cases; Stay preserves service; confirmed quit waits for owned service exit |
| Recovery | Force shell exit, separately force service exit; reopen/retry | Persisted records readable, unresolved task interrupted; pending approval not granted; no replay, old owned runtime cleaned up |
| Missing workspace | Move fixture directory and reopen history; confirm replacement | History readable, execution blocked until explicit valid replacement |
| Bundle mismatch | Use test build with different dsh/desktop prerelease or damaged resource | Packaging/startup fails before task-capable navigation; supported version and next action shown |
| Port and repeat launch | Occupy 3080; repeatedly launch/reopen/Retry | Random free loopback port; one runtime generation only |
| Native boundary | Attempt native shell/filesystem/invoke from Harness or external content; follow external link | Native authority denied; external page not loaded in privileged main view; auth token not forwarded |
| Network scope | Capture idle/setup/task traffic with fixture destinations; inspect disabled surfaces | No product telemetry, feedback export or custom plugin installation; project/task data only reaches permitted destinations |
| Minimum OS | Install/run signed build on macOS 14 Apple Silicon | Runtime/native addons/window/lifecycle work at advertised minimum |

Use [lifecycle contract](contracts/desktop-lifecycle.md) and
[bundle contract](contracts/runtime-bundle.md) for exact behaviors and
[data model](data-model.md) for ownership/state assertions. Compare controlled
task, Stop, approvals and workspace switching against the same packaged upstream
version outside the shell, using a separate fixture data root. A mismatch or
upstream requirement gap blocks release; do not silently implement a second engine.

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
| A1 Launch/readiness | Launch with saved setup; request reopen during startup; separately induce startup failure and select recovery | One Host startup/readiness result, authenticated connection, failed child cleanup before retry | One Web CLI runtime and validated authenticated handoff; local status/Retry and 15-second failure ceiling; no browser or replay. Host IPC/welcome flow is not copied |
| A2 Window lifecycle | Run a fixture task, then test a pending approval; close and reopen in 10 cycles | Main window hidden, same document and Host continue | Window may be recreated; same owned service and dsh session/task/approval continue. Record transient page-state reset separately; no task duplication or permission decision |
| A3 Quit confirmation | Run the 10-trial matrix below; also test idle quit and late responses/dialog failure using controlled unit fixtures | Active/scheduled or unknown work warns; idle inspected work may quit without a prompt; repeated requests join one decision | Always prompt while service is alive; Stay preserves work, confirmed quit waits for exit; no late or failed dialog grants quit. Extra idle prompt is intentional |
| A4 Menus/shortcuts | Invoke matching menu/keyboard actions with local content then Harness content focused, including contextual page state | Official Close Page can route through contextual shortcut handling; Quit shares its native decision | Explicit native Close Window/Command+W and Quit/Command+Q have matching outcomes; upstream task/page shortcuts remain upstream-owned; no new customization/native bridge |
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
  and control >=4/5.
- **SC-002**: 20 saved-setup launches, >=19 usable within 10 seconds; record
  launch and usable timestamps, excluding model generation.
- **SC-003 / SC-004**: 20 controlled tasks, >=19 streamed updates visible within
  1 second of availability, all Stop acknowledgements within 1 second; every
  approval-required/denied action obeys the permission gate.
- **SC-005**: 20 normal relaunches and 10 forced exits retain all previously
  persisted records without replay; >=19/20 openings of a 1,000-message fixture
  within 2 seconds.
- **SC-007**: 10 close/reopen cycles retain service identity and work; 10
  confirmed quits stop it; every declined quit preserves it.
- **SC-008**: all six alignment decisions complete before affected implementation;
  100% of adaptations/exclusions explain reason and user impact; all adopted/
  adapted comparisons pass at release review with no unexplained differences.
- **SC-009**: run the 10-trial quit matrix above; at most one unresolved dialog,
  every declined quit preserves work, every confirmed quit awaits owned exit.

Record app/dsh/Node versions, manifest/build hash, OS/hardware/power conditions,
fixture IDs, timestamps, owned PID/process-group/listener evidence, sanitized
network findings and pass/fail results. Never include credentials or authenticated
URLs in reports. Inspect signatures/notarization and generated plist version
mapping. Do not claim release compliance from this design-only phase.

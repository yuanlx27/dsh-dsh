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

Record app/dsh/Node versions, manifest/build hash, OS/hardware/power conditions,
fixture IDs, timestamps, owned PID/process-group/listener evidence, sanitized
network findings and pass/fail results. Never include credentials or authenticated
URLs in reports. Inspect signatures/notarization and generated plist version
mapping. Do not claim release compliance from this design-only phase.

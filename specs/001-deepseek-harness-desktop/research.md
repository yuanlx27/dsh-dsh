# Phase 0 Research: DeepSeek Harness Desktop

## Research scope and evidence

Research tracks: supported release/runtime closure; Tauri/macOS distribution;
readiness/authentication; lifecycle/quit; persistence/secrets; acceptance tooling.
Independent network lookups were parallelized. No delegated-agent tool was
available; findings below are direct source/documentation inspection.

Release baseline:
- npm registry selected published `@deepseek-ai/dsh@0.2.0-rc.2`, bin
  `lib/bin.js`, with version-matched dsh dependencies.
- Source tag `dsh-v0.2.0-rc.2` points to
  `639ed015397290b3745d163aafe02ffee4aa3f84`.
- At the original inspection, master/tag `0.2.1-alpha.1` was newer than the
  selected npm baseline `0.2.0-rc.2`; this is historical context, not a live
  latest-version query. Do not mix the reference sources and runtime artifacts.
- Published tarball integrity:
  `sha512-EAJ3gPNcVt/uv8X19PMm9NkVhWgT7xXNMk0UKCVm+IQ5rpSQOcsMUa0HWlnYYVybKMsccjcRB21vVVsaXQ6IdA==`.
- Node release index provides `node-v24.21.0-darwin-arm64.tar.gz`, SHA-256
  `bed7eea5325e1108f32ce5228ddd6a5f0f08a499ee42aa7442aea583702f6057`.

Source inspection is not an installed-app conformance test. All end-to-end
requirements remain release acceptance gates after implementation.

## 1. Desktop stack and supported platform

**Decision**: Tauri 2 with Rust lifecycle ownership and WKWebView; first release
macOS 14+ Apple Silicon, signed/notarized DMG. Use small local HTML/TypeScript
startup/failure views and upstream's Web UI for the workspace.

**Rationale**: Tauri is the user's explicit choice. Native menus, dialogs,
application reopen and close/exit interception cover desktop duties. Restricting
the first release to one CPU reduces packaging/native-module test combinations.
No frontend framework is needed for a loading/error view.

**Alternatives considered**: Electron and upstream Desktop conflict with the
chosen shell; native Swift replaces the chosen stack; universal/Intel builds add
unrequested release scope. Revisit platform breadth separately.

References: [Tauri sidecars](https://v2.tauri.app/develop/sidecar/),
[macOS bundle](https://v2.tauri.app/distribute/macos-application-bundle/),
[macOS signing](https://v2.tauri.app/distribute/sign/macos/).

## 2. Runtime and release equality

**Decision**: Canonical desktop version = dsh version = `0.2.0-rc.2`.
Pin Node `24.21.0`, the dsh tarball, the complete dependency lock and artifact
hashes. Build two Tauri external binaries: `node` and a minimal native `dsh`
launcher. Ship upstream JavaScript, frontend assets and native modules as
resources, including all transitive runtime files.

**Rationale**: dsh's npm bin is JavaScript; copying it into `externalBin` alone
does not make it executable on a clean machine. The launcher resolves Node and
the CLI from bundle paths. Installing the locked closure at build time avoids
first-launch npx/network dependencies. Exact equality includes prereleases.

**Alternatives considered**: Global Node/dsh, runtime npx and loose `latest`
violate installation and reproducibility goals. Single-file JS bundling risks
Cordis package resolution, dynamic plugins and native modules. Independent
desktop semver violates the user's decision.

Native Apple version fields may require numeric encodings. They are platform
metadata only: derive them from the shared version, preserve the full semantic
version in the release manifest/About/package/Tauri configuration, and test
Tauri's actual generated plist rather than silently discarding the prerelease.
Shell-only rebuilds use a monotonic native build number, not a new product semver.

References: [npm baseline](https://registry.npmjs.org/@deepseek-ai/dsh/0.2.0-rc.2),
[release CLI package](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/apps/cli/package.json),
[Node checksums](https://nodejs.org/dist/v24.21.0/SHASUMS256.txt).

## 3. Launch, readiness and authentication

**Decision**: Invoke the packaged CLI with
`web --host 127.0.0.1 --port 0 --no-open` in a stable application-data working
directory and explicit isolated `DSH_HOME`. Parse the owned child's
`dsh web: <authenticated URL>` readiness announcement; validate HTTP,
127.0.0.1 and a nonzero port, retain authentication material only in memory.
Use a 15-second hard startup deadline with a visible failure and explicit Retry.

**Rationale**: Release `web-startup` supports OS-assigned ports and no browser
handoff. Release `web-app` announces after Loader settlement and required-entry
audit, using `connection.authenticatedUrl`; bare port-open checks and guessed
3080 URLs miss both readiness and authentication. A 15-second failure ceiling is
not a relaxation of the 10-second usable-launch success target.

**Alternatives considered**: Fixed 3080 can collide with another dsh; port
reservation before spawn races; a custom health endpoint changes upstream;
ignoring URL authentication breaks the supported connection flow.

References: release
[Web flags](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/bundle/web-app/src/startup.ts),
[Web announcement](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/bundle/web-app/src/index.ts).

## 4. Lifetime, shutdown and crash recovery

**Decision**: One app-level runtime owner; native single-instance handling; close
destroys/closes the window without authorizing app exit. Dock/reopen recreates the
window at the same authenticated service URL. Always show a native Stay / Stop
and Quit confirmation while a service is alive, even with no windows.

Confirmed quit closes task-capable windows, terminates the owned dsh process
group gracefully, waits up to seven seconds (upstream grace is five seconds),
then force-terminates owned survivors and waits for exit. Do not complete quit
while the service is still alive. A private owner-liveness pipe lets the launcher
clean up its child if the shell crashes. No automatic service restart or task replay.

**Rationale**: Upstream's active-work inspection is installed by the Electron
Desktop Host, not the chosen Web profile. Always confirming conservatively meets
FR-010 without a new task-inspection protocol or duplicated task state. A native
dialog works with no window. Process groups and parent liveness avoid ordinary
orphaned owned service processes; they are not a security sandbox and cannot
promise control over arbitrarily detached approved commands.

**Alternatives considered**: Using the Electron Host changes the selected
kernel surface; window-owned subprocesses cancel background work; renderer-only
quit checks fail without a window; arbitrary PID scanning could kill another
user's dsh; a reusable generic supervisor is unnecessary.

References: release
[quit inspection](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/apps/desktop-host/src/quit-inspection.ts),
[CLI shutdown](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/apps/cli/src/process-shutdown.ts).

## 5. Data ownership, safety and excluded surfaces

**Decision**: Upstream owns model credentials, workspaces, tasks, approvals and
session persistence. Shell owns only notice acknowledgement and window
preferences. Do not show the task-capable Web window until first-run notice is
acknowledged. Use an application-owned Web-profile overlay that disables product
telemetry/analytics, feedback reporting and custom plugin-installation UI; no
new cloud destination is added. Preserve upstream task and permission semantics.

**Rationale**: FR-018 explicitly forbids duplicate Harness management. Release
Web profile already disables desktop product telemetry by profile name; an
explicit overlay plus network tests make the no-telemetry constraint enforceable.
Feedback and plugin-installation surfaces otherwise expose out-of-scope actions.

Shell diagnostics must not copy raw dsh logs: published CLI startup reports
explicitly warn they can contain configuration/credentials. Show fixed,
categorized explanations plus exit status and Retry; never auto-export logs.
Use private filesystem permissions and distinguish dsh-owned raw reports from
shell-authored diagnostics. Verify credential masking and retention end to end.

**Alternatives considered**: Shell keychain migration, a new session database,
or rebuilding task controls duplicates upstream. Merely assuming no telemetry
because this is a desktop wrapper is insufficient.

References: release
[Web profile](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/bundle/web-app/cordis.patch.yml),
[home paths](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/util/home-paths/src/index.ts),
[user guide](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/docs/user/guide/index.md),
[Tauri capabilities](https://v2.tauri.app/security/capabilities/).

## 6. Native authority boundary

**Decision**: No Tauri filesystem, shell-execution or general invoke privileges
for the localhost Harness page. Run runtime operations and menu actions in Rust;
privileged local startup views have only narrow retry/status/notice commands.
Disallow navigation to a different origin in the Harness window and hand
explicit external HTTP(S) links to the system browser without forwarding the
service token. Keep upstream authentication and browser-trust checks.

**Rationale**: Local Web content executes agent-facing plugins and must not gain
a second native execution channel. Dynamic localhost ports do not justify broad
remote-origin capabilities. No renderer bridge is necessary for native menus.

**Alternatives considered**: Allowing `http://127.0.0.1:*` arbitrary native
shell access, rendering external links in a privileged window, or proxying the
full upstream RPC through Rust all enlarge authority unnecessarily.

## 7. Test strategy and unresolved-unknown disposition

**Decision**: Native integration on the named M5/macOS 27 host, minimum-OS
acceptance separately on macOS 14 Apple Silicon; deterministic fixtures for
approval, cancellation, failure, version mismatch, and 1,000-message history.
Use Rust and Node standard test runners for shell and packaging unit tests.

**Rationale**: Tauri/WKWebView menus, close/quit and signed sidecars require actual
macOS tests. Browser-only tests cannot prove installed-app lifetime or closure.

**Alternatives considered**: Passing upstream unit tests alone or testing only
development mode does not establish the desktop requirements.

All planning unknowns have a chosen design. Exact dependency toolchain locks,
signing identity, minimum-OS runner provisioning, dependency closure checks and
behavioral measurements are implementation/release work, not unresolved choices.
If upstream acceptance fails, block release and revisit the baseline/plan;
do not recreate missing Harness responsibilities in the shell.

## 8. Official desktop reference supplement

### Scope, baselines and research tasks

This incremental pass addresses FR-022–FR-024 and SC-008/SC-009 only. Sections
1–7 remain the design baseline: Tauri 2, macOS 14+ Apple Silicon, Node 24.21.0,
dsh 0.2.0-rc.2, two sidecars, one app-owned Web service, recreated windows,
conservative quit confirmation, and no duplicate Harness state.

- **O (official behavioral reference)**: repository commit
  `5badb15009ae1756c3afe0ae0cef1faafc290ccc`, inspected 2026-10-06 through the
  GitHub tree and immutable raw-source URLs. Its `apps/desktop/package.json`
  declares `0.2.1-alpha.1` and an Electron shell.
- **R (shipped runtime source)**: commit
  `639ed015397290b3745d163aafe02ffee4aa3f84`, published dsh `0.2.0-rc.2`.
  O is not substituted for R, and no runtime upgrade or new dependency is selected.
- Research tasks: identify O's launch/readiness and close/reopen behavior;
  identify O's quit and menu/shortcut decision rules; identify O's recovery and
  immutable release validation. Downloads for these tracks were parallelized.
  No delegated research-agent facility was available; findings are direct
  source inspection, not delegated-agent output or executed upstream tests.

Unknowns resolved by this pass: reference identity, hide-versus-recreate window
semantics, task-aware-versus-conservative quit policy, contextual-versus-native
close routing, recovery diagnostics, and reference/runtime compatibility boundary.
There are no unresolved design choices; installed-app evidence is pending work.

### Source findings and decisions

All source links in this subsection are pinned to O; filenames and symbols
allow review without relying on a moving `master` URL. The normative six-row
adoption record is in [plan](plan.md#official-desktop-alignment-fr-022fr-024).

**A1 — Launch/readiness**

- Evidence: [host-process.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/host-process.ts),
  `DesktopHostProcess.start`, spawns one bundled child, joins its readiness
  promise on repeated calls and accepts a validated `ready` event.
  [main.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/main.ts)
  authenticates the ready URL before loading workspace services.
  [backend-controller.spec.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/tests/backend-controller.spec.ts)
  covers shared concurrent startup, cleanup-before-retry and late readiness failures.
- **Decision**: Adapt single-owner readiness and retry sequencing to the existing
  Web CLI stdout handoff at R. Retain the selected timeout and authentication boundary.
- **Rationale**: The user selected the Web kernel and Tauri sidecars, not the
  official Electron Desktop Host. A ready port alone is not a usable/authenticated workspace.
- **Alternatives considered**: Copying Host IPC or replacing the Web profile
  would reopen FR-018/FR-020; neither is authorized. No native proxy is added.

**A2 — Window lifecycle**

- Evidence: `main.ts`, `createMainWindow` prevents close and hides the main window;
  `focusPrimaryWindow` shows it or recreates an absent window; macOS
  `window-all-closed` does not quit. The ordinary close preserves the document and Host.
- **Decision**: Adapt background lifetime and reopen semantics, retaining our
  previously selected window destruction/recreation against the same dsh instance.
- **Rationale**: FR-019 concerns uninterrupted work and approval/session state,
  not document identity. Our design remains minimal and keeps service lifetime
  independent of window lifetime. Transient page state may reset on reopen;
  record that difference and verify users return to the expected dsh session.
- **Alternatives considered**: Switching to hide-only could preserve more page
  state but is a redesign not requested here; coupling the child to a window
  violates FR-019. If acceptance fails, report the failure rather than claiming parity.

**A3 — Quit confirmation**

- Evidence: [quit-confirmation.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/quit-confirmation.ts),
  `resolveDesktopQuitPrompt` warns on unknown inspection and distinguishes active
  and scheduled work; `confirm` reuses one pending decision and focuses it.
  `main.ts` supplies an ownerless native dialog and waits for backend shutdown.
  [quit-confirmation.spec.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/tests/quit-confirmation.spec.ts)
  specifies idle/no-Host silent quit, inspection failure warning, joined requests,
  and ignoring late responses after disposal. These tests were read, not run.
- **Decision**: Adopt joined confirmation, cancellation, ownerless presentation,
  unknown-state warning and disposal semantics using native Tauri state. Adapt
  the predicate: always confirm while the owned service is alive, as already planned.
- **Rationale**: O can inspect Host task/schedule state; the chosen R Web kernel
  does not supply that desktop contract. An extra idle confirmation is an intentional
  user-visible difference, avoiding an inspector or duplicated task records.
- **Alternatives considered**: Copying Host task inspection or adding a new task
  protocol would violate simplicity and reopen the kernel decision. Unknown status
  is not an authorization to quit. Dialog failure also must not authorize shutdown;
  preserve the service and present a sanitized retryable failure.

**A4 — Menus/shortcuts**

- Evidence: [keyboard.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/keyboard.ts),
  `sendMenuClose` routes `page.close` to trusted renderer content, while the
  `shortcutsCloseWindow` handler validates current binding revision/focus before
  `window.close()`. [keybindings.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/keybindings.ts)
  owns atomic configurable shortcut preferences. `main.ts` builds native menus.
- **Decision**: Adapt to shared native Close Window / Command+W and Quit /
  Command+Q handlers. Leave task/page shortcuts inside upstream Web content.
- **Rationale**: FR-023 requires equal menu/keyboard outcomes; the current scope
  does not require contextual Close Page parity or a second shortcut preference
  store. The Web page must not gain privileged native invocation to imitate O.
- **Alternatives considered**: Porting the configurable Electron shortcut bridge
  adds state and authority outside the existing design. Compare outcomes with
  focused Web content to detect interception instead of assuming WKWebView parity.

**A5 — Error recovery**

- Evidence: [fatal-recovery.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/fatal-recovery.ts),
  `DesktopFatalRecovery.report` shows one fatal decision, waits boundedly for a
  report, and provides explicit exit/restart/plugin-disable actions; recovery
  awaits stop. [startup-error.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/startup-error.ts)
  serializes nested errors as message text.
- **Decision**: Adapt explicit recovery and cleanup; retain our fixed non-secret
  local error categories and Retry, without raw report export or plugin repair.
- **Rationale**: Copying nested diagnostic text can violate FR-004; custom plugin
  management is excluded. Existing failure views meet FR-014 without adding
  report persistence or the official recovery UI.
- **Alternatives considered**: Verbatim errors, automatic restart or plugin
  disable would enlarge scope or undermine safety/no-replay. Comparison checks
  explain these intentional omissions rather than report identical UX.

**A6 — Release compatibility**

- Evidence: [release.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/release.ts),
  `DesktopRelease` declares one exact shell/dsh version; `parseDesktopRelease`
  validates immutable release facts. [runtime-tree.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/runtime-tree.ts),
  `readDesktopRuntime` rejects mismatched shared package versions and
  `verifyDesktopRuntime` checks expected version and package metadata.
  [desktop/package.json](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/package.json)
  establishes the reference package version.
- **Decision**: Adapt matching immutable version/closure verification to our
  existing Runtime Manifest and two Tauri sidecars. Adopt exact identity including
  prerelease; keep R and Node unchanged and preserve numeric Apple metadata mapping.
- **Rationale**: O's Electron/Host/pnpm metadata is not a replacement for our
  Tauri manifest; product/dsh equality must hold independently of the reference version.
- **Alternatives considered**: Upgrading R to match O, copying the whole packaging
  toolchain or adding auto-updates contradicts this incremental update's scope.

### Evidence status and baseline changes

Source findings above are confirmed by inspection. Compatibility of adopted
concepts with R and product/official runtime comparisons are **pending**, not
passing. The six decisions have acceptance mappings in the plan and runnable-at-
implementation-time comparison instructions in [quickstart](quickstart.md#official-desktop-comparison-protocol).

Do not add a seventh runtime subsystem to store these decisions: they are
review-owned documentation. Before changing O or R, list affected A1–A6 rows,
inspect replacement immutable sources, check Web release compatibility, update
contracts/scenarios only where needed, and invalidate affected previous comparison
results. Unavailable official runtime/source evidence remains pending and blocks
an alignment-complete release claim; it is not an unresolved architecture choice.

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
requirements remain release acceptance gates after implementation. Section 9
records the user's official-style authentication decision and reconciled Tauri
transport design; all design questions are resolved, while runtime proof remains
pending.

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
Exchange the validated root token privately in Rust for the upstream signed
cookie (303 response; redirects disabled), then discard the launch URL. Keep the
cookie native and in memory. Serve packaged content at `dsh-app://app`, not the
authenticated localhost URL; inject only non-secret boot data and the narrow
transport adapter. Use a 15-second hard startup deadline including exchange/boot,
with visible failure and explicit Retry. See section 9 for forwarding.

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
hides and retains the primary window/document without authorizing app exit.
Dock/reopen shows and focuses that document at the same application origin with
its existing native service session, without restarting dsh or resetting draft,
selected conversation or scroll position solely because of close/reopen.
Actual destruction/page failure can require recreation; unsaved context after
crash/full quit/page failure is not guaranteed. CHK038 reviewer approval
supersedes the former normal-close destruction/recreation decision. Always show a native Stay / Stop
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
The later CHK032 reviewer clarification bounds this guarantee to upstream settings
protection and non-disclosure by shell-generated content. Arbitrary dsh/tool/model
output is not subject to a new shell redaction or history-rewriting guarantee;
service-token/cookie confidentiality and no-extra-destination constraints remain.

The clarified FR-004 explicitly accepts upstream's existing protection.
`LocalCredentialProvider` stores plaintext YAML under
`$DSH_HOME/.credentials.yaml`, creates/replaces the file with `0600` and parent
directories with `0700`, and checks group/other permission bits before load,
reload, and writes on macOS. Keep credential operations in upstream Settings
and the provider; the shell only establishes the private data root. Do not add
Keychain storage, encryption, or a shell credential API. Same-user processes,
including agent tools, can read the file; this is an accepted limitation, not
an isolation guarantee. Test save/replace/remove/relaunch, `0600` permissions,
rejection of an overly permissive file, and absence of the fixture key from
history and shell-authored diagnostics.

**Alternatives considered**: Keychain-backed storage or not saving credentials
were not selected by the user. A shell-owned store would duplicate upstream
responsibilities. Merely assuming no telemetry because this is a desktop wrapper
is insufficient.

References: release
[Web profile](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/bundle/web-app/cordis.patch.yml),
[home paths](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/util/home-paths/src/index.ts),
[credential provider](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/credentials/credentials-local/src/index.ts),
[credential protection and limitations](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/credentials/credentials-local/README.md),
[user guide](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/docs/user/guide/index.md),
[Tauri capabilities](https://v2.tauri.app/security/capabilities/).

## 6. Native authority boundary

**Decision**: No Tauri filesystem, shell-execution or general network privileges
for Harness content. Run runtime operations and menus in Rust. Local startup
views have narrow retry/status/notice commands; authorized main-frame app content
gets only the fixed-owned-service transport in section 9. Validate actual native
WebView identity, frame identity and application origin, not renderer assertions.
Reject child frames and external origins. Keep navigation at the application
origin; explicit external HTTP(S) links open in the system browser without any
service authentication. Preserve upstream authentication and request-trust checks.

**Rationale**: Local Web content executes agent-facing plugins and must not gain
a second native execution channel. Dynamic localhost ports do not justify broad
remote-origin capabilities. No renderer bridge is necessary for native menus.

**Alternatives considered**: Allowing `http://127.0.0.1:*` arbitrary native
shell access, rendering external links in a privileged window, or proxying the
arbitrary upstream destinations through Rust all enlarge authority unnecessarily.
The user-approved fixed-service adapter is transport-only, not a business RPC API.

## 7. Test strategy and unresolved-unknown disposition

**Decision**: Native integration on the named M5/macOS 27 host, minimum-OS
acceptance separately on macOS 14 Apple Silicon; deterministic fixtures for
approval, cancellation, failure, version mismatch, and 1,000-message history.
Use Rust and Node standard test runners for shell and packaging unit tests.

**Rationale**: Tauri/WKWebView menus, close/quit and signed sidecars require actual
macOS tests. Browser-only tests cannot prove installed-app lifetime or closure.

**Alternatives considered**: Passing upstream unit tests alone or testing only
development mode does not establish the desktop requirements.

The original planning unknowns and the FR-025 policy decision now have chosen
designs. Section 9 records the authorized Tauri transport adaptation. Exact
dependency locks, signing identity, native adapter conformance, minimum-OS runner
provisioning, closure checks and behavioral measurements remain implementation/
release work. No assumption of Electron APIs existing in WKWebView is made.
If upstream acceptance fails, block release and revisit the baseline/plan;
do not recreate missing Harness responsibilities in the shell.

## 8. Official desktop reference supplement

### Scope, baselines and research tasks

This incremental pass addresses FR-022–FR-024 and SC-008/SC-009 only. Sections
1–7 remain the design baseline: Tauri 2, macOS 14+ Apple Silicon, Node 24.21.0,
dsh 0.2.0-rc.2, two sidecars, one app-owned Web service, retained hidden windows
under the later CHK038 reviewer decision,
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
  Web CLI stdout handoff at R. Adopt O's private token exchange, native cookie,
  application-origin documents and authenticated forwarding under revised FR-025;
  retain the deadline. Section 9 records the Tauri-specific stream adaptation.
- **Rationale**: The user selected the Web kernel and Tauri sidecars, not the
  official Electron Desktop Host. A ready port alone is not a usable/authenticated workspace.
- **Alternatives considered**: Copying Host IPC or replacing the Web profile
  would reopen FR-018/FR-020; neither is authorized. Direct authenticated localhost
  navigation was superseded by the user's official-style mediation decision; the
  narrow adapter is permitted without copying Host business/control IPC.

**A2 — Window lifecycle**

- Evidence: `main.ts`, `createMainWindow` prevents close and hides the main window;
  `focusPrimaryWindow` shows it or recreates an absent window; macOS
  `window-all-closed` does not quit. The ordinary close preserves the document and Host.
- **Decision**: Adopt official normal-close hide/retain and reopen show/focus
  semantics after explicit CHK038 reviewer approval. Retain document and native
  transport authority while hidden; dsh instance/work/approvals remain unchanged.
- **Rationale**: Preserves draft, selection and scroll context without a separate
  persistence mechanism. Native Tauri hide/show replaces the former normal-close
  destruction/recreation design; service lifetime remains application-owned.
- **Alternatives considered**: The former destroy/recreate policy could lose
  transient state and is superseded. Recreation remains permitted for genuinely
  absent/failed documents, without guaranteeing unsaved context after crash,
  full quit or page failure. Runtime parity still requires comparison evidence.

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
  store. The Web page must not gain general native or shortcut invocation to imitate O.
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
an alignment-complete release claim; it is not an unresolved architecture choice
for the historical official-reference pass. The resolved FR-025 reconciliation follows.

## 9. FR-025 and credential protection reconciliation

### Research tasks and evidence

Inspected immutable R and O sources on 2026-10-06: token/cookie exchange,
HTTP/WebSocket admission, static serving, credential protection, and official
shell forwarding. Also inspected Tauri protocol/channel APIs and native
WKWebView frame metadata. Requests were parallelized; no delegated-agent facility
was available. No product or upstream runtime tests were executed.

- At R, [BrowserAuth](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/client/connection/src/browser-auth.ts)
  exchanges a process token on `GET /` for an authority-bound signed cookie.
  Its signing secret is the dsh-owned `client-connection/browser-session` grant
  in `.credentials.yaml`. Credential-provider protection is recorded in section 5.
- R's [admission](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/client/connection/src/rpc-host.ts)
  checks Host/Origin then the cookie; [Gateway WebSocket admission](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/api/gateway/src/index.ts)
  uses the same gate. Headers are not client identity. The HTTP
  `connection/request` hook does not cover WebSocket upgrades.
- R's [static serving](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/host/frontend-static/src/index.ts)
  authenticates root/index but leaves non-sensitive static assets public.
- O's [desktop Host](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop-host/src/index.ts)
  privately sends the authenticated URL over parent-child IPC.
  [web-document.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/web-document.ts)
  implements `authenticateWebHost` and `forwardWebRequest`: native token
  exchange, cookie attachment, removal of renderer-supplied auth/trust headers,
  and withholding `Set-Cookie` from the renderer.
- O's [main.ts](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/main.ts)
  serves packaged UI at `dsh-app://app` and injects WebSocket cookie/trust
  headers only for the main window, exact Host destination and app origin.
  This authenticates an authorized renderer through the shell, not client process
  identity at the Host.
- R already exposes [ClientTransportHooks](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/client/connection/src/client/index.ts):
  `__DSH_TRANSPORT__.fetch` and `openStream`. The
  [Gateway client](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/api/gateway/src/client/index.ts)
  uses `connection.rpc.open` when supplied rather than creating a browser
  WebSocket. Reuse its [stream protocol](https://github.com/deepseek-ai/deepseek-harness/blob/639ed015397290b3745d163aafe02ffee4aa3f84/packages/api/gateway/src/stream-protocol.ts)
  and Connection recovery semantics; do not create business RPC schemas.
- Tauri provides [asynchronous URI schemes](https://docs.rs/tauri/latest/tauri/struct.Builder.html#method.register_asynchronous_uri_scheme_protocol)
  and [ordered channels](https://v2.tauri.app/develop/calling-frontend/#channels).
  URI-scheme responses have byte bodies, not an Electron streaming-response API.
  Do not assume custom-scheme handling intercepts browser WebSocket handshakes.
  Native [WKScriptMessage](https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKScriptMessage.html)
  exposes the actual WebView and frame; [WKFrameInfo](https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKFrameInfo.html)
  exposes `isMainFrame`, request and security origin.

### Decision: official-style shell mediation adapted to Tauri

**Decision**: The user selected "We do as this official approach." Revised FR-025
requires loopback bearer authentication with shell-controlled delivery and
forwarding; it no longer claims to reject another local process possessing valid
or forged authentication. FR-004 remains upstream plaintext storage with owner-only
permissions. The previous Phase 0 blocker is resolved by that explicit decision.

1. Keep R, Web profile, sidecars and lifecycle unchanged. Rust consumes the owned
   child's readiness URL, exchanges the root token with redirects disabled, checks
   the upstream 303/cookie result, retains cookie and service origin in memory,
   and discards the launch token URL. Do not use a renderer cookie jar or persist
   shell copies of upstream credentials.
2. Serve packaged UI and local views from separate application-origin authorities
   (`dsh-app://app` and `dsh-app://shell`). Retrieve non-secret boot injections
   through the authenticated upstream index and preserve the release's boot/module
   contract; never inject the service URL with token or authentication headers.
3. Use a small bundled transport shim rather than rebuild the task interface.
   Its fixed-app-origin Fetch adapter uses the native HTTP bridge for upstream
   requests, including existing raw upload/download routes. Install that adapter
   in `__DSH_TRANSPORT__.fetch`; route same-app service fetches through it while
   leaving external requests outside the authenticated bridge. Packaged assets
   use the URI-scheme handler. Upstream `openStream` uses the narrow native
   stream bridge; it must not start a browser WebSocket with credentials.
4. Native bridge admission validates WKWebView identity, actual main-frame
   metadata and app origin, notice acknowledgement and current launch/window
   generation. Reject subframes and non-primary windows before attaching auth.
   Use a frame-aware WKWebView message handler and main-frame-only bootstrap,
   not a renderer-supplied label or Tauri window URL alone as proof of frame identity.
   Return ordered chunks/frames through a private window-scoped channel; no
   globally broadcast transport events. Main-frame replies contain no auth headers.
5. Rust HTTP forwarding uses `reqwest`; live streams use a
   `tokio-tungstenite` connection to R's fixed `/api/remote.mux`. Reuse
   upstream stream message types/parsers in the shim, preserving multiplexing,
   uplink/downlink, cancellation and Connection-owned recovery. Forward opaque
   payloads; do not parse task/workspace business data or invent new methods.
   Credentials and trust headers are native-controlled. See the runtime contract
   for destination, redirect, body-size and lifecycle rules.
6. Native chunked transfer is used for service HTTP bodies/live streams, because
   the URI-scheme responder alone cannot preserve streaming. Use bounded queues,
   cancellation and upstream body limits; do not buffer unbounded responses or
   broadcast data between windows. Custom-scheme static serving does not become
   a listening proxy or general network-fetch API.

**Rationale**: This adopts O's security policy and user-visible behavior while
using Tauri/native features and R's existing transport hooks. A narrow adapter
is now required by FR-025; it does not duplicate dsh-owned tasks, configuration,
approvals or persistence. A native transport avoids relying on Electron-only
WebSocket header interception or exposing the cookie to JavaScript.

**Alternatives considered**:
- Load the authenticated localhost URL directly: superseded by the user's choice;
  it puts service authentication in the renderer's browser session.
- Copy Electron `protocol.handle`/`webRequest` APIs or switch to Electron:
  incompatible with the approved Tauri shell.
- Grant the renderer generic HTTP/WebSocket plugin access or a secret header:
  unnecessarily exposes destination choice or credentials.
- Create a new Host API, second agent engine, generic proxy or credential store:
  outside FR-018 and unnecessary; reuse the Web carrier and transport hooks.
- Strict process-identity isolation: not selected; official bearer authentication
  does not provide it, particularly with same-user-readable signing secrets.

### Threat boundary and validation status

Ordinary unauthenticated local clients cannot use session/task/approval APIs;
direct connections from other computers cannot reach the loopback service.
Unauthorized WebViews, origins and child frames must not use the native bridge.
Non-sensitive static resources may remain public locally. Same-user credential
theft, cookie forgery using the upstream signing secret, privileged attackers,
and compromise of authorized app code are outside the isolation guarantee.
Do not test these excluded attacks as expected denials or claim a sandbox.

Design gates pass; implementation/conformance evidence is pending. Before release,
prove actual native frame validation, authenticated HTTP and WebSocket paths,
stream ordering/cancellation/backpressure, binary/upload/download fidelity,
no secret in renderer state, and close/reopen generation handling. A missing
platform capability or failed test blocks release and requires design review,
not a silent fallback to direct authenticated localhost navigation. Phase 1
contracts and quickstart carry these tests and the accepted credential limitation.

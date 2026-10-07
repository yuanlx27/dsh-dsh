---
description: "Executable implementation tasks for DeepSeek Harness Desktop"
---

# Tasks: DeepSeek Harness Desktop

**Input**: Design documents from `/specs/001-deepseek-harness-desktop/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/desktop-lifecycle.md`, `contracts/runtime-bundle.md`, `quickstart.md`, and `.specify/memory/constitution.md`.
**Tests**: Included because the specification explicitly requires controlled tests, native acceptance and SC-001–SC-010 evidence. Write automated tests before the corresponding implementation and verify meaningful failures; execute installed-app acceptance after implementation. Documentation-only review is not a passing runtime test.
**Organization**: One phase per user story, in specification order (US1 P1, US2 P1, US3 P2, US4 P2).

## Format: `[ID] [P?] [Story] Description`

- `[P]` means different files with no dependency on incomplete tasks in the same execution wave; prerequisite waves still apply.
- `[US1]`–`[US4]` map to the specification's stories. Setup, foundation and polish have no story label.
- All source/test paths are repository-relative implementation deliverables unless explicitly identified as generated resources. None currently exist.

## Path Conventions and Binding Constraints

- Single Tauri application: `src/` for semantic HTML/TypeScript local views and the transport shim; `src-tauri/src/` for native ownership; `scripts/` for build-time preparation; `tests/packaging/` and `tests/acceptance/` for tests/evidence.
- Runtime R: `@deepseek-ai/dsh@0.2.0-rc.2`, source `639ed015397290b3745d163aafe02ffee4aa3f84`, Node `24.21.0`. Official reference O: `5badb15009ae1756c3afe0ae0cef1faafc290ccc` / desktop `0.2.1-alpha.1`. O is reference-only, not a runtime upgrade.
- Delivery: current local testing uses ad-hoc-signed installed .app/DMG without an Apple developer account. Developer ID signing, notarization and public-distribution qualification are deferred; local results never imply public-release readiness or Gatekeeper acceptance elsewhere. Do not disable system security controls for validation.
- Native target: macOS 14+ Apple Silicon, `aarch64-apple-darwin`. Named measurement host: MacBook Air Mac17,3, M5, 16 GB, macOS 27.0 build 26A428; additionally require a macOS 14 Apple Silicon runner.
- Reuse dsh's settings, workspaces, tasks, approvals, history, stream protocol and recovery. No new business schemas, credential store, task inspector, workspace preflight, text deduplication, submission lock, history cache, page-state store or shortcut store.
- Transport is fixed-owned-service forwarding, not a general network proxy. Never expose/log/persist service token URLs or cookies, copy raw dsh diagnostics into shell surfaces, or silently fall back to authenticated localhost navigation.
- Acceptance fixtures use isolated disposable data/workspace roots and fixture credentials. Reports record sanitized evidence, not keys, Cookie/Set-Cookie values or raw payloads. Upstream requirement gaps block release, not authorize a second engine.
- Before adding code, follow the constitution's simplicity ladder; use native APIs, standard library and already selected dependencies before custom helpers. Keep tracked prose English.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the smallest locked Tauri project and test commands; do not install a frontend framework.

- [X] T001 Review the existing six A1–A6 decisions and FR/NFR/SC coverage before affected implementation; create `tests/acceptance/README.md` with scenario-to-story mapping, fixture safety rules, O/R identities, required environments and pending release gates, referring to `specs/001-deepseek-harness-desktop/plan.md` without creating a runtime reference registry.
- [ ] T002 Initialize exact compatible Tauri 2, TypeScript 5.x and Rust dependencies in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `rust-toolchain.toml`; pin Rust stable >=1.85/edition 2024, reqwest, tokio-tungstenite and compatible macOS WKWebView bindings, use version `0.2.0-rc.2`, and add minimal `src-tauri/build.rs` and `src-tauri/src/main.rs` entry points.
- [ ] T003 Configure the minimal TypeScript/static build and Node built-in test runner in `tsconfig.json`, `scripts/build-frontend.mjs`, and `package.json`; expose `runtime:prepare`, `runtime:verify`, `test:packaging`, `tauri:dev`, and `tauri:build` commands matching `specs/001-deepseek-harness-desktop/quickstart.md` (commands must fail when required deliverables are missing).
- [ ] T004 [P] Configure macOS 14/arm64-only bundling, immutable resource paths, `binaries/node` and `binaries/dsh` externalBin entries, canonical prerelease version and derived numeric Apple build metadata in `src-tauri/tauri.conf.json`; keep update/telemetry integrations absent.
- [ ] T005 [P] Define disposable workspace, credential, approval/action and service-failure fixture preparation instructions in `tests/acceptance/fixtures.md`; specify separate dsh roots and observable controlled commands/file changes, prohibit real keys in tracked files, and reserve a 1,000-message fixture using upstream-supported records rather than invented schemas.

**Checkpoint**: Locked project/build/test entry points and fixture rules exist. No command is reported passing merely because its implementation is pending.

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared immutable runtime, shell-owned persistence and process ownership. Complete this phase before story implementation.

- [ ] T006 [P] Write runtime-bundle contract tests in `tests/packaging/runtime-bundle.test.mjs` covering exact prerelease equality, lock hashes, artifact inventory, Node version/arm64, CLI version, frontend/native dependency closure, missing/corrupt resources and installation paths containing spaces.
- [ ] T007 [P] Write native foundational tests in `src-tauri/tests/foundation.rs` for preferences schema rejection/atomic write failure/onscreen bounds, private-directory modes, bounded readiness parsing (64 KiB line limit), invalid/LAN URLs, timeout/early exit, owner-pipe cleanup and owned-process-only shutdown; use helper test processes rather than unrelated running dsh instances.
- [ ] T008 Record the pinned Node SHA-256 and dsh tarball integrity from `specs/001-deepseek-harness-desktop/research.md`, R identity, exact production dependency resolution and native build number in `runtime.lock.json` and `runtime/package.json`/`runtime/package-lock.json`; resolve the complete closure once at build time and freeze subsequent installs.
- [ ] T009 Implement locked build-time staging in `scripts/prepare-runtime.mjs`: verify downloads, use the frozen production closure, stage target-suffixed Node and launcher resources, include dynamic packages/Web assets/native addons in `src-tauri/resources/dsh/`, and generate `src-tauri/resources/runtime-manifest.json` with all data-model fields and artifact hashes; support final inventory/hash regeneration after nested signing in T052, keep download-integrity checks separate, and never fetch dependencies at installed-app launch.
- [ ] T010 Implement `scripts/verify-runtime.mjs` and `src-tauri/src/bundle.rs` to enforce the bundle contract at build and startup: compare complete root/Tauri/manifest/package/CLI versions, probe bundled Node, validate target/inventory/hash closure and derive Apple metadata without losing canonical prerelease identity; reject unsafe bundles before task-capable navigation.
- [ ] T011 [P] Implement only shell Application Preferences in `src-tauri/src/preferences.rs`: app-data root at `0700`, atomic user-only JSON with schemaVersion 1, current-notice revision and usable window bounds; reject unsupported formats and surface write failures without storing workspace/task/history/model/authentication state.
- [ ] T012 [P] Implement the minimal native sidecar in `src-tauri/src/bin/dsh-launcher.rs`: launch absolute bundled Node/CLI paths in an owned process group, supervise the owner-liveness pipe and signals, gracefully terminate then force owned survivors after the shell's seven-second bound, wait for exit, and never scan/kill unrelated services or disappear before child cleanup.
- [ ] T013 Implement the one-generation runtime owner and readiness parser in `src-tauri/src/runtime.rs`: use `web --host 127.0.0.1 --port 0 --no-open` plus private DSH_HOME/stable cwd/validated overlay, remove inherited Node injection/model-key/telemetry variables without stripping ordinary approved-command environment, coalesce startup/retry, accept only the current child's bounded announcement, and enforce a 15-second total startup deadline including later auth/boot preparation.
- [ ] T014 Implement fixed non-secret failure categories and cleanup-before-retry transitions in `src-tauri/src/errors.rs` and `src-tauri/src/runtime.rs`; unexpected exits invalidate the generation without automatic restart, failed cleanup prevents competing launch/successful quit, and raw stdout/stderr/token-bearing URLs are neither persisted nor copied into diagnostics.
- [ ] T015 Execute the foundation tests and CLI/resource probes from T006–T014 and record actual results, closure inventory and any unresolved native feasibility issues in `tests/acceptance/reports/foundation.md`; do not advance with a version/ownership failure.

**Checkpoint**: Bundle verification, preferences and owned process cleanup pass; runtime announcement alone is not ready/authenticated. Foundation does not implement any dsh-owned entity.

## Phase 3: User Story 1 — Open a ready-to-use local desktop workspace (Priority: P1) — MVP

**Goal**: Launch without host Node/npm/dsh, acknowledge safety, privately connect to the bundled service and use upstream settings/workspaces inside a retained native window.

**Independent Test**: On a supported macOS account with fixture key and accessible project, launch without terminal/browser setup, acknowledge the notice, configure/select through dsh, reach task-ready input, relaunch with retained credentials, hide/reopen with identical document context, and confirm quit leaves no owned service. Missing/rejected prerequisites must show setup/corrective state, not false readiness.

### Tests for User Story 1

- [ ] T016 [P] [US1] Write authenticated-transport contract tests in `src-tauri/tests/transport.rs` for private root-token exchange (required 303/cookie, redirects disabled), destination/path/header rejection, auth stripping, 300 MiB input bound, binary/multipart fidelity, cancellation, ordered replies, generation invalidation and no authentication in returned boot data.
- [ ] T017 [P] [US1] Write transport-shim tests in `tests/packaging/transport-adapter.test.mjs` for R's fetch/openStream hook installation, same-app raw upload/download routing, external-fetch exclusion, opaque stream frames/uplink/downlink/multiplex ordering, cancellation, reconnect without replay and stale-handle rejection; do not replace global WebSocket or add task logic.
- [ ] T018 [P] [US1] Write lifecycle/notice/quit tests in `src-tauri/tests/lifecycle.rs` for acknowledgement revision/write failure, repeated launch, retained close/show identity, one ownerless quit decision, Stay/dismissal/dialog failure, late responses, unknown work status and waiting for actual owned exit.
- [ ] T019 [P] [US1] Define the native WKWebView acceptance procedure and report fields in `tests/acceptance/startup-security.md` for first launch, denied app-data write, clean/offline runtime, occupied 3080, credential lifecycle/read-only-source removal refusal, read-only/missing workspace, unauthorized frames/windows/origins, redirects, streams, renderer secret absence and unauthenticated local/remote access; include expected results from FR-002–005/FR-025 and SC-010.

### Implementation for User Story 1

- [ ] T020 [P] [US1] Implement Rust-only token exchange and authenticated non-secret boot extraction in `src-tauri/src/authentication.rs`, using the current runtime's validated token URL with reqwest redirects disabled, requiring R's 303 and valid service cookie, discarding the token URL after exchange and keeping cookie/origin in native transient state only.
- [ ] T021 [P] [US1] Implement traversal-safe immutable scheme serving in `src-tauri/src/protocol.rs` for separate `dsh-app://app` and `dsh-app://shell` authorities; serve R's packaged assets/boot-module protocol with only non-secret injections, never treat the scheme byte responder as a streaming service proxy.
- [ ] T022 [P] [US1] Add and validate the narrow upstream Web-profile overlay in `src-tauri/resources/desktop-web.patch.yml` disabling desktop-product-telemetry, product-analytics, feedback reporting and custom plugin installation entry points; verify R's CLI patch syntax/dependency relationships, preserve task/permission behavior and do not activate the Desktop profile.
- [ ] T023 [P] [US1] Implement semantic startup/safety/failure HTML, TypeScript and minimal focus styles in `src/shell/index.html`, `src/shell/main.ts`, and `src/shell/styles.css`; show categorized status/next action, require explicit current-revision acknowledgement successfully saved before entry, and provide keyboard/VoiceOver-visible controls with no raw diagnostics or rebuilt Harness setup UI.
- [ ] T024 [US1] Implement frame-aware native bridge admission in `src-tauri/src/bridge.rs` using actual WKScriptMessage WebView/frame/security-origin metadata: authorize only the current primary main frame at app origin after notice/runtime readiness, bootstrap main-frame-only, scope handles/replies to window/runtime generations, reject shell views/other windows/child frames/external origins and never trust renderer-supplied origin labels.
- [ ] T025 [US1] Implement fixed-service chunked HTTP forwarding in `src-tauri/src/http_transport.rs` through the admitted bridge: allow relative supported service requests only, reject authority-changing/malformed/token targets, strip caller auth/trust/hop-by-hop headers, attach native same-service cookie/trust headers, disable redirects, strip response Set-Cookie/connection headers, and preserve bytes/cancellation using bounded ordered queues and R's 300 MiB buffered-body default.
- [ ] T026 [US1] Implement native live-stream forwarding in `src-tauri/src/stream_transport.rs` to only the current owned `/api/remote.mux` via tokio-tungstenite on Tauri's Tokio runtime; attach native cookie/trust headers, preserve opaque ordered stream payloads/uplink/downlink/cancellation with bounded queues and private replies, and leave reconnect/replay decisions to upstream Connection.
- [ ] T027 [US1] Implement the narrow upstream shim in `src/transport.ts` using R's existing ClientTransportHooks/stream protocol to connect fetch/openStream and same-app raw Fetch consumers to native chunked handles; keep external fetch outside authenticated transport, with no renderer cookie, general destination control, business API, automatic replay, text deduplication or submission lock.
- [ ] T028 [US1] Wire ready/notice-gated Harness navigation, actual destruction/page-failure detachment and runtime-generation invalidation in `src-tauri/src/app.rs` and `src-tauri/src/bridge.rs`; startup is ready only after native auth/boot preparation within the deadline, detachment cancels renderer transfers not dsh tasks/approvals, and stale handles/callbacks are rejected before a recreated document receives fresh authority.
- [ ] T029 [US1] Implement primary-window retained hide/show, native single-instance/Dock activation and allowed navigation/external-link handling in `src-tauri/src/window.rs`; close control retains document/draft/selection/scroll/transport, reopen joins the same service, recreate only absent/failed documents, deny file/data/unapproved origins and open explicit external HTTP(S) links without authentication.
- [ ] T030 [US1] Implement native Close Window/Command+W, Quit/Command+Q, Open Workspace Window, About and Safety Notice commands in `src-tauri/src/menu.rs` and `src-tauri/src/quit.rs`; share native handlers, display canonical manifest versions, use one ownerless Stay / Stop and Quit dialog whenever service is alive (all sessions/workspaces, no undo), preserve work on cancellation/failure and complete confirmed quit only after generation revocation and owned shutdown.
- [ ] T031 [US1] Scope local status/retry/acknowledgement authority separately from the frame-aware Harness transport in `src-tauri/capabilities/shell.json` and `src-tauri/src/app.rs`; deny Harness general filesystem/shell/network plugin authority, validate shell command callers, wire the native application entry point, and ensure focused Web content cannot override native close/quit outcomes.
- [ ] T032 [US1] Run T016–T019 against actual native WKWebView plus an installed candidate build; record first/saved launch, credential save/replace/remove/per-operation behavior and 0600/permissive-file rejection, no shell credential injection, setup/readiness independence, byte/stream/frame rejection and local/remote admission evidence in `tests/acceptance/reports/us1.md`; fix only shell/adapter defects, mark upstream gaps release-blocking, and retain pending status for unavailable minimum-OS checks and distinguish local ad-hoc evidence from deferred Developer ID/notarization checks.
- [ ] T033 [US1] Execute the SC-007 ten close/reopen cycles and ten confirmed quits (including pending approvals, hidden windows, Stay, draft/selection/scroll and transfer identity), plus 20 saved-setup launches for SC-002 (>=19 usable within 10 seconds), and record process/document identities and sanitized timestamps in `tests/acceptance/reports/lifecycle-performance.md` on the named idle host.

**Checkpoint**: US1 provides the usable shell and secure transport. Task-capable setup stays upstream-owned; end-to-end task correctness is qualified in US2. Native feasibility failures block continuation, never authorize cookie exposure.

## Phase 4: User Story 2 — Complete a task with visible progress and control (Priority: P1)

**Goal**: Preserve upstream submission/progress/approval/Stop/workspace-switch semantics through the shell and prove explicit recovery/quit does not duplicate work.

**Independent Test**: With a prepared upstream workspace/model connection, summarize the fixture project, request a controlled change requiring approval, deny then allow separate actions, Stop during streaming/in-flight action, switch workspaces with multiple sessions active, and force service failure. Verify actual outcomes, visible states, no shell-induced repeat/replay and explicit recovery.

### Tests for User Story 2

- [ ] T034 [P] [US2] Specify and prepare controlled task/approval/Stop fixtures in `tests/acceptance/tasks-control.md` using R's existing permission policy; cover all FR-007 states, policy/action/target display, denial/dismissal never approving, in-flight Stop without rollback claims, identical/rapid/queued/steered submissions following dsh, reconnect/hide-show without replay, and multi-session workspace switching without shell confirmation/inspection.
- [ ] T035 [P] [US2] Add adapter failure/cancellation/reconnect regression tests in `tests/packaging/task-forwarding.test.mjs` using opaque recorded fixture envelopes from R (without keys); assert one dispatch per explicit upstream request, preserve legitimate identical-text submissions, never resubmit after reconnect/recreation, and do not introduce task admission/business state in the shim.

### Implementation and Verification for User Story 2

- [ ] T036 [US2] Exercise the US2 fixtures against standalone pinned R and the native app with isolated roots; identify transport-only incompatibilities and fix `src/transport.ts`, `src-tauri/src/http_transport.rs`, or `src-tauri/src/stream_transport.rs` only where needed to preserve task/action states, approval decisions, raw streams and Stop; record upstream requirement gaps rather than create a new engine in `tests/acceptance/reports/us2.md`.
- [ ] T037 [US2] Integrate unexpected-service-exit presentation and explicit Retry with native ownership in `src-tauri/src/app.rs`, `src-tauri/src/runtime.rs`, and `src/shell/main.ts`; invalidate old auth/handles, report interruption using non-secret next actions, preserve dsh's persisted history, clean predecessor before restart, and never replay submissions/commands/edits or resolve pending approvals.
- [ ] T038 [US2] Run service-exit, pending-approval, missing/rejected/rate-limited model, credential-change-during-work and multi-session full-quit fixtures; record that Stay leaves all work untouched, confirmed quit waits for owned exit, Stop does not stop dsh, and workspace switching follows R in `tests/acceptance/reports/us2-recovery.md`.
- [ ] T039 [US2] Measure SC-003/SC-004 across 20 native controlled runs with response and action updates; record native receiving-boundary-to-visible timestamps (>=19 runs with every update <=1 second), every Stop acknowledgement <=1 second, every gated/denied action unperformed and each in-flight action's actual result in `tests/acceptance/reports/task-performance.md`; any upstream gap blocks release rather than adding shell task logic.

**Checkpoint**: Tasks, approvals, Stop and recovery satisfy the comparative R fixtures. US2 adds integration qualification, not independent agent services or models.

## Phase 5: User Story 3 — Return to previous project conversations (Priority: P2)

**Goal**: Expose the existing durable upstream history and workspace associations through normal relaunch, failure recovery and unavailable-directory conditions.

**Independent Test**: Create completed/interrupted sessions in two fixture workspaces; relaunch, identify each original session/workspace and continue an available one without changing the other. Missing/moved directories retain R's history/error/continuation behavior with no reassociation, and unresolved approvals never resume automatically.

### Tests for User Story 3

- [ ] T040 [P] [US3] Define upstream-owned persistence/recovery cases in `tests/acceptance/history.md`: duplicate/blank titles with distinct identities, two-workspace follow-ups, completed/interrupted sessions, pending approval, recoverable incomplete tail versus corrupt/unsupported/missing records, discovery omission not implying no history, and moved/restored workspace without directory replacement or migration.
- [ ] T041 [P] [US3] Create a fixture-generation helper in `tests/acceptance/create-history-fixture.mjs` using supported pinned upstream session facilities to generate 1,000-message and two-workspace histories in disposable roots; validate committed records using upstream rules and fail rather than fabricate a replacement storage schema.

### Integration and Verification for User Story 3

- [ ] T042 [US3] Verify upstream session listing/history/follow-up routes through the installed adapter; fix only resource/boot/transport compatibility in `src-tauri/src/protocol.rs` and `src/transport.ts` if required, retaining R's title fallbacks, identities, associations, unavailable-directory choices and refusal behavior; record observed outcomes against standalone R in `tests/acceptance/reports/us3.md` without a shell history store/cache/repair API.
- [ ] T043 [US3] Execute 20 normal relaunches and 10 forced shell/service-exit recoveries with committed-history inventories, incomplete work and pending approvals; record readable persisted records, interrupted status, no auto-resume/approval/replay and actionable unreadable-history failures in `tests/acceptance/reports/history-recovery.md` (SC-005).
- [ ] T044 [US3] Execute 20 openings of the 1,000-message fixture with app/dsh already connected on the named idle host; record session-open-request to identified/readable/browsable first screen latency and require >=19 within two seconds in `tests/acceptance/reports/history-performance.md`, excluding startup and without adding shell caching.

**Checkpoint**: US3 independently qualifies upstream durability/history behavior using prepared sessions; it does not depend on completing US2's performance study.

## Phase 6: User Story 4 — Rely on official desktop behavior (Priority: P2)

**Goal**: Prove the six already-recorded official-reference adaptations, native command equivalence and conservative joined quit behavior; keep unavailable evidence pending.

**Independent Test**: Compare all A1–A6 fixtures against a verified official build/source execution at O, recording separate observed product/reference results and intentional differences. Exercise native close/quit via both menu and keyboard with local/Harness focus, repeated requests, unknown work state and hidden window.

### Tests for User Story 4

- [ ] T045 [P] [US4] Prepare comparison records and reproducible O checkout/build/test instructions in `tests/acceptance/official-comparison.md` using the quickstart protocol, isolated roots and verified provenance; cover A1–A6, define pending/passing/failing evidence rules, and distinguish executed official unit evidence from native installed-app evidence without introducing a reference runtime subsystem.
- [ ] T046 [P] [US4] Add native command/quit regression tests in `src-tauri/tests/official_alignment.rs` for menu/keyboard handler equivalence, one joined pending decision, unknown state warning, failed/dismissed dialogs, late response after disposal and service-exit-before-quit; reference approved A3/A4 differences, not official contextual Close Page or idle-inspection parity.

### Implementation and Comparative Verification for User Story 4

- [ ] T047 [US4] Run the official tests/provenance checks prescribed by `specs/001-deepseek-harness-desktop/quickstart.md` at immutable O in an isolated checkout; record exact artifact/source identity, executed results, native limitations and unavailable evidence as pending in `tests/acceptance/reports/official-reference.md`, without changing R or claiming source inspection proves parity.
- [ ] T048 [US4] Execute A1/A2/A5/A6 product/reference comparisons (startup/auth/transport, retained window, explicit sanitized recovery, canonical version/damaged bundle); record separately observed outcomes, intentional differences/reasons/user impact and evidence links in `tests/acceptance/reports/alignment.md`; require evidence for every adopted/adapted area before passing.
- [ ] T049 [US4] Execute A3/A4 native comparison and the exact SC-009 ten-trial matrix (two each: active/open, active/hidden, approval/hidden, unknown work, repeated quit); choose Stay then confirm in each product trial, assert <=1 unresolved dialog and cleanup-before-exit, test both local/Harness focus and menu/keyboard routes, and record outcomes/allowed differences in `tests/acceptance/reports/quit-alignment.md`.
- [ ] T050 [US4] Correct any shell-owned native command/dialog/lifecycle deviations uncovered by T046–T049 in `src-tauri/src/menu.rs`, `src-tauri/src/quit.rs`, and `src-tauri/src/window.rs`, rerun affected tests, and update `tests/acceptance/reports/alignment.md`; retain approved native Close Window and always-confirm predicates without a task inspector, shortcut store or official report/plugin UI.
- [ ] T051 [US4] Document the O/R change-review checklist in `tests/acceptance/official-comparison.md` and review `specs/001-deepseek-harness-desktop/plan.md`/`research.md` for continued A1–A6 traceability: require immutable replacement evidence, compatibility/affected-scenario review and invalidation/retesting of changed comparisons before adoption; leave any missing evidence pending and block alignment-complete claims.

**Checkpoint**: All six comparative rows pass only with actual relevant evidence and explained differences. US4 planning prerequisites were established in T001, before affected implementation; this phase does not defer that required early review.

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Qualify the locally installed ad-hoc build and remaining cross-story criteria; document deferred public-distribution gates separately. No automatic updater or extra platforms.

- [ ] T052 [P] Add local ad-hoc signing/build procedure in `scripts/package-macos.mjs` and `src-tauri/entitlements.plist`: stage the runtime, ad-hoc sign nested Node/launcher/native addons with only required entitlements, regenerate the final runtime inventory/hashes using T009, sign the outer arm64 .app, and package .app/DMG; verify signatures and generated Apple numeric metadata against the canonical manifest version. Do not modify inventoried resources or the manifest after outer signing. Document Developer ID signing, hardened-runtime/JIT qualification and notarization/stapling as deferred public-release work, with future credentials supplied only through untracked inputs; do not require those credentials or claim public-distribution validation for local completion.
- [ ] T053 [P] Add installed-distribution cases in `tests/packaging/distribution.test.mjs` for target-suffixed sidecars, closure/native addon loading, no startup download, paths with spaces, complete prerelease/About identity, corrupt/missing artifacts, wrong architecture, generated plist mapping and manifest equality with final signed nested resources; ensure `npm run test:packaging` discovers all packaging tests.
- [ ] T054 [P] Execute keyboard-only and VoiceOver qualification for startup, safety notice, failure/Retry and ownerless quit focus/name/status transitions in `tests/acceptance/accessibility.md`; verify the embedding preserves upstream keyboard/assistive capability, record findings in `tests/acceptance/reports/accessibility.md`, and fix shell-only semantics/styles in `src/shell/index.html`/`src/shell/styles.css` without rebuilding upstream UI.
- [ ] T055 Run the locally installed ad-hoc-signed .app/DMG on a clean account with no Node/npm/dsh and, when available, separately on macOS 14 Apple Silicon; execute clean/offline/path-space/occupied-port/native-addon/bundle-mismatch/app-data-denial/crash/repeated-launch cases from `specs/001-deepseek-harness-desktop/quickstart.md`, verify installed resource hashes against the final manifest, and record actual OS, signing mode, versions and remaining failures in `tests/acceptance/reports/distribution.md`. Missing minimum-OS evidence remains pending and blocks a minimum-OS-qualified claim. Developer ID/notarization/Gatekeeper distribution checks remain deferred, never passing by substitution or by disabling system security controls.
- [ ] T056 Execute final SC-010 native installed-build access tests and idle/setup/task traffic capture in `tests/acceptance/reports/security-network.md`: reject unauthorized window/origin/child-frame/stale-handle and unauthenticated index/API/WebSocket access, deny remote direct access/general native authority, verify authentication absence in renderer/cookie jar/diagnostics, preserve permitted public non-sensitive assets, and show only configured model/explicitly approved action destinations with excluded telemetry/feedback/plugin surfaces disabled.
- [ ] T057 Conduct the SC-001/SC-006 installed-app study with >=10 target developers and uniform briefs/fixture keys/project locations; record receipt-acknowledged first submission within five minutes, summary outcome discovery, original-session/workspace retrieval, awaiting-approval versus completed identification and visibility/control rating in `tests/acceptance/reports/usability.md`; assisted participants remain in the denominator, require >=90% independent outcomes and >=80% ratings >=4/5.
- [ ] T058 Re-run quickstart commands and cross-story acceptance on the final locally installed ad-hoc build, reconcile SC-001–SC-010/NFR-001 and all FR scenarios with evidence/build provenance, and record local qualification results, pending/failing functional or environment gates and deferred public-release gates separately in `tests/acceptance/reports/release-readiness.md`. Developer ID signing/notarization absence does not block local-build testing, but public-release readiness remains deferred; never substitute development/browser-only/source-inspection or ad-hoc success for installed native/official/minimum-OS or public-distribution evidence.
- [ ] T059 Update `README.md` and `specs/001-deepseek-harness-desktop/quickstart.md` with actual implemented commands, install/development prerequisites, shared versions, background-close versus full-quit behavior, explicit recovery and accepted plaintext/bearer limitations; remove obsolete design-only status only when supported by results, keep secrets out and review constitutional English/simplicity/commit-format constraints.

**Checkpoint**: Report local qualification only against actually executed installed/native/security/performance/usability and six official-comparison gates; unavailable minimum-OS or other required evidence remains pending. Public release additionally requires the deferred Developer ID/notarization/distribution gates and minimum-OS qualification. Planning completion alone does not imply application completion.

## Dependencies & Execution Order

### Phase Dependencies

```text
Setup T001–T005
       |
Foundation T006–T015
       |
US1 T016–T033 (native shell + secure transport; MVP)
       |----------------------|-----------------------|
US2 T034–T039             US3 T040–T044            US4 T045–T051
(task/control)           (prepared history)       (official alignment)
       |----------------------|-----------------------|
                 Polish T052–T059 -> release gate
```

- US1 depends on foundation, not another story. US2, US3 and US4 use US1's shell/transport, not each other's implementation. US3 fixtures provide history without requiring US2 completion; US4 uses independent controlled fixtures.
- Recommended priority delivery is US1 -> US2 -> US3 -> US4 -> polish. These are genuine integration dependencies, not a claim that all stories can run from foundation independently.
- US2/US3/US4 can prepare their independent test documents in parallel after US1; implementation fixes touching shared native/transport files must be serialized, and acceptance runs need isolated roots/accounts or scheduling to avoid process/test interference.
- T001 establishes the official decision review before any affected implementation. T045–T051 supply later comparative proof; missing O evidence stays pending.
- Automated tests precede implementation; installed acceptance follows implemented candidates. Candidate testing is repeated against the final locally installed ad-hoc build in T055/T056/T058; Developer ID/notarization evidence is deferred separately.

### Within-Phase Waves and Parallel Opportunities

- Setup: T001 -> T002 -> T003, then T004 + T005 (different files).
- Foundation: T006 + T007; T008 -> T009 -> T010 implement the lock and preparation/verification scripts. T011 + T012 may run after their tests with T008–T010. Execute runtime preparation only once T012's launcher is built; T015 verifies the generated inventory and actual CLI probes. T013 waits for bundle/preferences/launcher implementation, then T014 -> T015. Foundation probes CLI/version and helper-process ownership; actual Web-profile startup waits for T022's validated overlay and is qualified in T032.
- US1: T016 + T017 + T018 + T019; T020 + T021 + T022 + T023 can run after their relevant tests/foundation. T024 waits for auth/serving/notice contracts, then T025 -> T026 -> T027 -> T028 -> T029 -> T030 -> T031 -> T032 -> T033. HTTP and stream owners can be split only after bridge contracts stabilize; no parallel marker promises incomplete shared integration is safe.
- US2: T034 + T035, then T036 -> T037 -> T038 -> T039.
- US3: T040 + T041, then T042 -> T043 -> T044.
- US4: T045 + T046, then T047 -> T048 -> T049 -> T050 -> T051.
- Polish: T052 + T053 + T054 use separate files after desired stories. Local ad-hoc packaging and shared-shell accessibility fixes must finish before T055 -> T056 -> T057 -> T058 -> T059.
- Shared-file changes (especially `src/transport.ts`, `src-tauri/src/app.rs`, runtime/menu/quit/window modules) never run concurrently merely because story labels differ.

## Parallel Example: User Story 1

```text
Wave 1: T016 src-tauri/tests/transport.rs
        T017 tests/packaging/transport-adapter.test.mjs
        T018 src-tauri/tests/lifecycle.rs
        T019 tests/acceptance/startup-security.md
Wave 2 (foundation/relevant tests ready):
        T020 src-tauri/src/authentication.rs
        T021 src-tauri/src/protocol.rs
        T022 src-tauri/resources/desktop-web.patch.yml
        T023 src/shell/index.html + main.ts + styles.css
Join before T024; serialize subsequent integration.
```

## Parallel Example: User Story 2

```text
T034 tests/acceptance/tasks-control.md
T035 tests/packaging/task-forwarding.test.mjs
Join before T036; use separate roots for standalone R and the product.
```

## Parallel Example: User Story 3

```text
T040 tests/acceptance/history.md
T041 tests/acceptance/create-history-fixture.mjs
Join before T042; do not generate fixtures in live user DSH_HOME.
```

## Parallel Example: User Story 4

```text
T045 tests/acceptance/official-comparison.md
T046 src-tauri/tests/official_alignment.rs
Join before executed comparisons T047–T049.
```

## Implementation Strategy

### MVP First (US1)

1. Complete setup/foundation and prove bundle/process ownership.
2. Complete US1 tests, native auth/frame boundary, fixed HTTP/stream transport, safety and retained-window/quit handling.
3. Stop and validate US1 independently using fixture settings/workspace and native candidate acceptance; do not market a release based only on source review or development mode.
4. US1 is the suggested implementation MVP. The specification's minimum useful release additionally requires US2 task/control and US3 persistence qualification; shipping also requires US4 comparisons and final release gates.

### Incremental Delivery

- Add US2 using upstream task/permission semantics; block on upstream gaps rather than fill them with custom agent behavior.
- Add US3 using prepared upstream sessions, with no duplicated data management.
- Add US4 comparative proof against O, while maintaining R and explicitly allowed differences.
- Ad-hoc sign/package and repeat all cross-story gates on the final locally installed build; qualify the minimum OS when a runner is available, otherwise retain pending status. Defer Developer ID signing, notarization and public-distribution checks without claiming public-release readiness.

### Parallel Team Strategy

One developer owns US1 shared native/transport integration. Once it passes, separate developers can prepare and execute US2, US3 and US4 fixtures in isolated environments. Coordinate any fixes to shared files sequentially. Distribute distinct packaging/accessibility/documentation tasks only when prerequisite story behavior is stable.

## Notes and Completeness Validation

- Every task has an unchecked checkbox, unique sequential ID, concrete file path and (only in story phases) a US label. `[P]` entries identify explicit independent waves, not a license to bypass prerequisites.
- US1 covers setup/readiness/credential/workspace, safety, bundle/version, retained lifecycle, native authentication/access and explicit quit; US2 covers tasks/states/approvals/Stop/recovery/workspace switching; US3 covers history/identity/durability/unavailable directories; US4 covers A1–A6 provenance/comparisons/command equivalence/joined confirmation/baseline review.
- Shared shell Application Preferences and Runtime Manifest are foundational; upstream Workspace/Model Connection/Session/Task/Permission Decision records remain upstream-owned, with their acceptance mapped to the earliest serving story instead of new model tasks.
- Bundle contract maps to foundation/US1 and distribution; lifecycle contract maps to US1/US2 and US4 comparisons. Data-model transient runtime/transport/window/quit invariants map to foundation and US1. Review-owned alignment records remain test/review documentation.
- Cross-cutting qualification covers NFR-001, SC-001–SC-010, network exclusions, clean/minimum-OS distribution and constitutional constraints. Each story has its own independent acceptance criteria and report destination.
- Commit only if requested; any eventual commit uses Conventional Commits, subject <=50 columns and body <=72 columns. No application code or passing acceptance result is claimed by generating this file.

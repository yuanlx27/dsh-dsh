# Implementation Plan: DeepSeek Harness Desktop

**Branch**: `main` | **Date**: 2026-10-06 | **Spec**: [spec.md](spec.md)

**Feature identifier**: `001-deepseek-harness-desktop`. Setup reports this logical
identifier as `BRANCH`; the actual Git branch is `main`. No branch switch is needed.

**Input**: Feature specification from `specs/001-deepseek-harness-desktop/spec.md`, including the user's Tauri, bundled Node/dsh sidecar, version-equality decisions, FR-022–FR-024 official desktop reference supplement, revised FR-025 official-style shell-mediated access, and clarified FR-004 credential protection. This incremental reconciliation preserves the stack, runtime, platform, ownership boundary, and source layout.

**Reconciliation status**: Phase 0 and Phase 1 reconciled with the user's official-style authentication decision. FR-025 now requires shell-mediated loopback bearer authentication, not isolation from same-user clients with stolen/forged credentials. The authenticated localhost URL is no longer loaded into the WebView. See [research section 9](research.md#9-fr-025-and-credential-protection-reconciliation).

## Summary

Build a macOS Tauri shell around the supported upstream dsh Web application.
Bundle Node.js and the complete dsh runtime; do not rebuild the agent, settings,
workspace, approval, or history interfaces. The application owns one service
instance independently of window lifetime. Normal window close hides and retains the document, preserving drafts and page context as well as work;
confirmed full quit stops the owned service. Desktop and dsh share the exact
canonical semantic version. Serve the packaged interface from `dsh-app://app`;
Rust privately exchanges the launch token for the upstream cookie and forwards
HTTP and live-stream traffic for authorized application content only. dsh retains
all task/data behavior; the shell adds only a transport adapter.

Use published `@deepseek-ai/dsh@0.2.0-rc.2` and Node.js `24.21.0`.
Research inspected published CLI artifacts and source at release tag
`dsh-v0.2.0-rc.2`; behavioral acceptance remains an implementation release gate,
not a claim that source inspection proves the feature works.

## Technical Context

**Language/Version**: Rust stable, minimum 1.85 (edition 2024), exact toolchain pinned at implementation; TypeScript 5.x/HTML for local views and the narrow upstream transport shim; Node.js 24.21.0 for dsh.

**Primary Dependencies**: Tauri 2.x (exact mutually compatible crates/CLI pinned in lockfiles), native macOS WKWebView, Rust-side Tauri shell and dialog plugins, upstream @deepseek-ai/dsh 0.2.0-rc.2 and its complete locked production dependency closure. Use Rust `reqwest` for authenticated HTTP and `tokio-tungstenite` for the fixed upstream WebSocket, sharing Tauri's Tokio runtime. Use WKWebView's native frame-aware script-message API through Rust macOS bindings for the narrow transport bridge; pin compatible crate versions during implementation. No independent frontend framework or agent SDK.

**Storage**: Shell preferences in Tauri app-data JSON with atomic replacement; separate dsh-owned `DSH_HOME` under that app-data root. dsh stores model credentials as plaintext in `.credentials.yaml`, creates/replaces it with `0600` permissions, and rejects group/other permission bits on macOS. Same-user processes, including agent tools, can read it. This protection is explicitly accepted by FR-004; no encryption, Keychain migration, duplicate credential store, or new conversation schema is required. The shell creates its private data directory with `0700` permissions; credential reads/writes remain upstream-owned.

**Testing**: `cargo test` for lifecycle/version/parser, authentication exchange, destination/header validation and stale-generation rejection; Node built-in test runner for packaging/transport-adapter checks; native macOS installed-app acceptance for frame-aware HTTP/WebSocket forwarding and credential permissions. Include SC-010 negative access tests. Do not rely on Linux browser automation to validate WKWebView behavior.

**Target Platform**: macOS 14+ Apple Silicon, `aarch64-apple-darwin`. Current delivery is a locally installed ad-hoc-signed .app/DMG for testing, with no Apple developer account required. Public distribution with Developer ID signing and notarization is deferred until credentials are available. Intel, Windows, Linux, App Store packaging, and universal binaries remain deferred. Minimum-OS installation remains a qualification gate; unavailable runner evidence stays pending.

**Project Type**: Desktop application with bundled local service.

**Performance Goals**: SC-002: >=19/20 saved-setup launches usable within 10 seconds; SC-003: >=19/20 task runs have every recorded relevant update visible within 1 second of arrival at the desktop-owned receiving boundary; every Stop acknowledgement within 1 second of application receipt of the request; SC-005: >=19/20 openings of 1,000-message history within 2 seconds. No extra throughput/RAM target.

**Constraints**: Full local runtime without host Node/npm/dsh; official-style shell-mediated loopback authentication with cookie/token withheld from renderer code; no general network proxy or renderer-controlled destinations; no telemetry or auto-replay; window close must not stop dsh; complete quit must stop only owned processes; exact desktop/dsh version equality. Upstream plaintext credentials and same-user-readable cookie-signing secret are accepted limitations, not process-identity isolation. Standard tools used by approved project commands are separate from the bundled runtime.

**Scale/Scope**: One user, one app-owned dsh process tree, one primary workspace window hidden/shown on normal close/reopen and recreated only if absent or failed; multiple workspaces/sessions managed by upstream. Four user stories, including reference alignment; no cloud sync, automatic updates, custom plugin installation, or independently redesigned Harness UI.

**Release-test environment**: MacBook Air Mac17,3, Apple M5, 16 GB RAM,
macOS 27.0 build 26A428, local APFS sample workspaces, idle host. Record build
hash, power mode, configuration and test fixtures with measurements.
Additionally validate installation and behavior on macOS 14 Apple Silicon before
claiming the advertised minimum OS. The present host does not prove minimum-OS compatibility.

## Delivery and Signing Scope

The user has no Apple developer account usable for Developer ID signing. Local
qualification therefore uses ad-hoc signing and installed native builds; it does
not establish Apple notarization or Gatekeeper acceptance on other computers.
Do not disable Gatekeeper or other system security controls as a validation step.
Functional, security, ownership, performance and comparative gates remain in
force. Record unavailable environment evidence as pending, never passing.

Developer ID signing, hardened-runtime/JIT qualification, notarization/stapling
and public-distribution installation checks are deferred public-release gates,
not prerequisites for producing or testing the current local build. A local
qualification report must distinguish its result from public-release readiness.

For either signing mode: stage the runtime, sign nested executables/native addons,
generate the final artifact inventory and hashes from those signed bytes, then
sign the outer application and package it. For public distribution, notarize and
staple afterward. Verify the installed inventory against the final manifest;
never modify the manifest or inventoried resources after outer signing. Keep
original download-integrity checks separate from final installed-artifact hashes.

## Constitution Check

*Gate before research and rechecked after Phase 1.*

| Gate | Before research | After design |
|------|-----------------|--------------|
| I: English in version control | PASS: specification/constitution are English; all planned artifacts will be English | PASS: all generated repository prose is English |
| II: Simplicity first | PASS: reuse dsh, Tauri, native WKWebView and existing upstream transport hooks | PASS: FR-025 justifies a narrow authenticated HTTP/stream adapter; no agent engine, duplicate data store, second task API, UI framework, task inspector, or generic proxy/supervisor framework |
| Specification first | PASS: specification and user-confirmed technology/security decisions provide intent | PASS: FR-004 and FR-025 are preserved without silently weakening them; no application implementation performed |
| Proportional testing | PASS: identify packaging, lifecycle, safety, persistence, transport and usability gates | PASS: contracts/quickstart cover comparative evidence, credential protection, frame/destination/header rejection and SC-001–SC-010 |
| Commit policy | PASS: no commit requested | PASS: future commits must use Conventional Commits, subject <=50 columns, body <=72 columns |

No unjustified constitutional violations.

**Feature feasibility gate: Requirements-quality review complete; runtime feasibility remains pending validation.** The user selected the official
shell-mediated approach; revised FR-025 explicitly accepts bearer authentication
and its same-user credential-compromise limitation. Research identifies the
upstream hooks and native APIs for a Tauri adaptation. Implementation must prove
frame-aware sender validation and authenticated HTTP/WebSocket forwarding on
WKWebView before release; failure blocks release, never authorizes leaking the
cookie or falling back to authenticated localhost navigation.

The 2026-10-06 [requirements-quality review](desktop-requirements-review.md)
records all 40 criteria satisfied after reviewer decisions, including adoption
of official hide/retain behavior for normal window close under CHK038. NFR-001
bounds accessibility to shell-owned surfaces and preservation of upstream
capability. FR-013 adopts dsh-owned unavailable-directory history/continuation;
no replacement-directory promise, shell migration or reassociation store remains.
Requirements-quality approval does not establish runtime conformance or passing
comparative/installed-app evidence.

Release validation work remains pending and must not be represented as passing
tests.

## Project Structure

### Documentation (this feature)

```text
specs/001-deepseek-harness-desktop/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    ├── desktop-lifecycle.md
    └── runtime-bundle.md
```

`tasks.md` is a later Phase 2 artifact and is not generated here.

### Source Code (repository root, proposed; not yet implemented)

```text
package.json
package-lock.json
runtime.lock.json
rust-toolchain.toml
src/                         # local views and narrow upstream transport adapter
src-tauri/
├── Cargo.toml
├── Cargo.lock
├── tauri.conf.json
├── capabilities/
├── src/                     # lifecycle, preferences, runtime and native transport owner
├── src/bin/dsh-launcher.rs  # minimal native dsh sidecar launcher
├── binaries/                # staged node and dsh target-specific executables
└── resources/dsh/           # immutable installed package closure and metadata
scripts/
└── prepare-runtime.ts
tests/
├── packaging/
└── acceptance/
```

**Structure Decision**: A single Tauri application, not a monorepo. The repository
currently contains planning/tooling documents only. Prefer Rust standard library
and Tauri/native lifecycle, menu, dialog and resource APIs. Use a small native
launcher because the upstream dsh bin is JavaScript, not a binary suitable for
Tauri `externalBin`; it invokes the bundled Node and upstream CLI, not a second
Harness implementation. Keep artifact-generation scripts out of runtime logic.

## Official Desktop Alignment (FR-022–FR-024)

Reference-only baseline **O**: official `apps` source at
`5badb15009ae1756c3afe0ae0cef1faafc290ccc` (desktop package `0.2.1-alpha.1`),
inspected on 2026-10-06. Runtime baseline **R** remains
`639ed015397290b3745d163aafe02ffee4aa3f84` / dsh `0.2.0-rc.2` with Node
`24.21.0`. O is behavioral evidence, not a runtime dependency or an instruction
to mix newer packages into R. Immutable source links and symbol-level findings
are recorded in [research section 8](research.md#8-official-desktop-reference-supplement).

| ID / area | Official source at O | Decision and expected product behavior | Reason and user impact | Acceptance mapping |
|-----------|----------------------|----------------------------------------|------------------------|--------------------|
| A1 Launch/readiness | [Host startup](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/host-process.ts), `DesktopHostProcess.start`; [Web forwarding](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/web-document.ts), `authenticateWebHost`, `forwardWebRequest`; `main.ts` protocol/WebSocket routing | **Adapted**: start once, exchange authentication privately, serve packaged UI at an application origin and mediate HTTP/live streams; retain R's Web CLI announcement and 15-second deadline | FR-018/FR-020 retain Web kernel/Tauri sidecars rather than Electron Host IPC. Adopt shell-owned authentication; Tauri uses a frame-aware native transport bridge instead of Electron request-header interception. No task reimplementation or process-identity guarantee | US1.1–5, US4.1; FR-025/SC-010; quickstart A1 and shell-mediated access checks |
| A2 Window lifecycle | [Window lifecycle](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/main.ts), `createMainWindow`, `focusPrimaryWindow`, `activate`, `window-all-closed` | **Adopted**: normal close hides/retains the primary window and document; reopen shows/focuses it against the same service; drafts, selection and scroll state have no closure-induced reset, and work/approvals continue | Reviewer CHK038 supersedes the former destroy/recreate decision. Use native Tauri hide/show without separate page-state persistence. Actual destruction/page failure may require recreation; unsaved state is not guaranteed after crash/full quit/page failure. Comparative evidence remains pending | US1.6–7, US4.1–2; SC-007; quickstart A2 |
| A3 Quit confirmation | [Quit decision](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/quit-confirmation.ts), `confirm`, `resolveDesktopQuitPrompt`; `main.ts` quit wiring | **Adapted**: retain an ownerless native Stay / Stop and Quit confirmation whenever a service is alive; adopt one pending decision, repeated-request joining, cancellation, and conservative unknown-state handling | Official Host can report active/scheduled work and allow idle quit without prompting; FR-018 and simplicity exclude a new inspector. Product users may see an extra idle prompt, including with zero windows; no unknown state authorizes silent interruption | US1.8, US2.6, US4.4; SC-007/SC-009; quickstart A3 |
| A4 Menus/shortcuts | [Keyboard routing](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/keyboard.ts), `sendMenuClose`, `shortcutsCloseWindow`; `main.ts` application menu | **Adapted**: native Close Window / Command+W and Quit / Command+Q share their action handlers; other task shortcuts stay upstream-owned | Official Close Page uses contextual renderer routing and configurable bindings. Retain the chosen native window command without a native shortcut bridge; users get explicit close/quit behavior, not full official shortcut customization. Verify Web content cannot consume a native command and change its outcome | US1.6/8, US4.3; FR-023; quickstart A4 |
| A5 Error recovery | [Fatal recovery](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/fatal-recovery.ts), `DesktopFatalRecovery.report`; `startup-error.ts` | **Adapted**: retain explicit recovery after owned-child cleanup, categorized non-secret explanations, and no replay | Official dialogs can include nested error text/report paths and disable third-party plugins. FR-004/FR-014 and excluded plugin management require sanitized local views and no copied report/export/plugin-repair surface; users retain safe recovery without secret leakage | US1.5, US2.5, US3.3, US4.2/5; quickstart A5 |
| A6 Release compatibility | [Release identity](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/release.ts), `parseDesktopRelease`; [Runtime validation](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/runtime-tree.ts), `verifyDesktopRuntime` | **Adapted**: adopt immutable matching shell/dsh identity and validated bundle metadata, mapped to the existing Tauri Runtime Manifest, exact prerelease equality and checks | FR-020/FR-021 retain Tauri resources and the pinned R release; Electron Host protocol/pnpm descriptors are not copied. Users keep a self-contained compatible installation and full version identity, not an upgrade to O | FR-020/FR-021, US4.1/5; quickstart A6 and bundle mismatch |

All six decisions are complete at planning level. Source inspection is confirmed;
product/official runtime comparisons are **pending**, not passing. See
[quickstart comparison protocol](quickstart.md#official-desktop-comparison-protocol)
for fixtures, observed differences and SC-008 release evidence.

Exclusions apply across the matrix: official account flows, automatic/mandatory
updates, reporting/telemetry, custom plugin management, Windows/Linux behavior,
and general Electron renderer/native APIs are not added. The narrowly scoped
authenticated transport adapter is now authorized by FR-025; it adds no native
filesystem/shell authority. Exclusions remain due to scope and FR-018/FR-020.

Use upstream concepts and test scenarios, not a copied Electron subsystem.
Prefer Tauri/native commands, standard library state, and existing dsh behavior.
Do not add a reference registry, task inspector, shortcut store or new runtime
protocol. A proposed change to O or R must review all affected A1–A6 decisions,
source evidence, compatibility with R, and comparisons before adoption; unavailable
evidence stays pending and blocks an alignment-complete release claim.

## Phase 0: Research

[research.md](research.md) records release selection, sidecar layout, readiness,
security, window lifetime, quit behavior, storage and testing decisions.
Independent research tracks were executed in parallel using source/registry/docs
requests. No separate research-agent facility was available in this session.
The current pass inspected R's HTTP/WebSocket admission and credential provider,
O's private authentication/custom-origin forwarding and window-scoped WebSocket
headers, and native Tauri/WKWebView transport APIs. Section 9 records the user's
resolution and the Tauri adaptation. Source requests were parallelized; no
delegated-agent facility was available. All design questions are resolved;
installed-app conformance and comparative evidence remain pending.

## Phase 1: Design and Contracts

The existing artifacts are reconciled incrementally with FR-004 and revised
FR-025. They define shell-owned transient authentication, frame-aware fixed-host
HTTP/stream forwarding, the accepted threat boundary, and validation scenarios.

- [data-model.md](data-model.md): ownership boundaries, shell state, upstream
  conceptual entities, lifecycle transitions and validation invariants.
- [contracts/desktop-lifecycle.md](contracts/desktop-lifecycle.md): windows,
  commands, notice, errors, background operation and quit semantics.
- [contracts/runtime-bundle.md](contracts/runtime-bundle.md): reproducible
  packaging, exact version checks, launcher protocol and loopback handoff.
- [quickstart.md](quickstart.md): implementation-time validation guide and
  release evidence requirements.

### Implementation sequence (not a task list)

1. Lock and stage the runtime, verify closure/version/architecture, then launch
   the packaged Web profile without a browser or host tools.
2. Add the minimal shell, safety acknowledgement, readiness/error handling,
   native menus, close-to-background and confirmed process-tree shutdown.
   Carry A1–A6 source decisions into the existing implementation; share native
   menu/shortcut handlers and one pending quit decision without adding an inspector.
3. Add official-style native authentication, packaged app-origin serving and
   fixed-destination HTTP/live-stream forwarding. Use upstream transport hooks,
   frame-aware caller validation, and no renderer-visible cookie. Exercise tasks,
   approvals, settings/history and credential-permission/access-denial fixtures;
   constrain excluded surfaces through the narrow profile overlay.
4. Ad-hoc sign/package and execute locally installed-app acceptance and performance
   gates, including A1–A6 comparative evidence and SC-008–SC-010. Record minimum-OS
   or other unavailable evidence as pending; defer Developer ID signing,
   notarization and public-distribution qualification. No `tasks.md` or
   application code is generated by this incremental planning update.

## Complexity Tracking

No constitutional exceptions requested. The native sidecar launcher is required
by the explicit two-sidecar distribution constraint and upstream's JavaScript
entry point; it must remain limited to startup and process ownership. Revised
FR-025 separately justifies the shell transport adapter. It forwards existing
upstream protocols without adding task schemas, business logic, a listening proxy,
arbitrary network access, or a second credential store.

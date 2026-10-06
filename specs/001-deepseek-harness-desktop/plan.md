# Implementation Plan: DeepSeek Harness Desktop

**Branch**: `main` | **Date**: 2026-10-06 | **Spec**: [spec.md](spec.md)

**Feature identifier**: `001-deepseek-harness-desktop`. Setup reports this logical
identifier as `BRANCH`; the actual Git branch is `main`. No branch switch is needed.

**Input**: Feature specification from `specs/001-deepseek-harness-desktop/spec.md`, including the user's Tauri, bundled Node/dsh sidecar, version-equality decisions, and FR-022–FR-024 official desktop reference supplement. This update reconciles existing design artifacts incrementally; it does not reselect the stack, runtime, platform, ownership boundary, or source layout.

## Summary

Build a macOS Tauri shell around the supported upstream dsh Web application.
Bundle Node.js and the complete dsh runtime; do not rebuild the agent, settings,
workspace, approval, or history interfaces. The application owns one service
instance independently of window lifetime. Closing windows preserves work;
confirmed full quit stops the owned service. Desktop and dsh share the exact
canonical semantic version.

Use published `@deepseek-ai/dsh@0.2.0-rc.2` and Node.js `24.21.0`.
Research inspected published CLI artifacts and source at release tag
`dsh-v0.2.0-rc.2`; behavioral acceptance remains an implementation release gate,
not a claim that source inspection proves the feature works.

## Technical Context

**Language/Version**: Rust stable, minimum 1.85 (edition 2024), exact toolchain pinned at implementation; minimal TypeScript 5.x/HTML for local startup and error views; Node.js 24.21.0 for dsh.

**Primary Dependencies**: Tauri 2.x (exact mutually compatible crates/CLI pinned in lockfiles), native macOS WKWebView, Rust-side Tauri shell and dialog plugins, upstream @deepseek-ai/dsh 0.2.0-rc.2 and its complete locked production dependency closure. No independent frontend framework or agent SDK required.

**Storage**: Shell preferences in Tauri app-data JSON with atomic replacement; separate dsh-owned `DSH_HOME` under that app-data root. No new database, credential store, or conversation schema. dsh controls its existing credentials and session persistence.

**Testing**: `cargo test` for shell lifecycle/version/parser logic; Node built-in test runner for packaging checks; native macOS installed-app integration and manual Web UI acceptance. Do not rely on Linux browser automation to validate WKWebView behavior.

**Target Platform**: Initial distribution: macOS 14+ Apple Silicon, `aarch64-apple-darwin`, signed/notarized .app in a DMG. Intel, Windows, Linux, App Store packaging, and universal binaries deferred. Minimum-OS installation is a release gate.

**Project Type**: Desktop application with bundled local service.

**Performance Goals**: SC-002: >=19/20 saved-setup launches usable within 10 seconds; SC-003: >=19/20 updates visible within 1 second and every Stop acknowledgement within 1 second; SC-005: >=19/20 openings of 1,000-message history within 2 seconds. No extra throughput/RAM target.

**Constraints**: Full local runtime without host Node/npm/dsh; localhost-only authenticated service; no telemetry or auto-replay; window close must not stop dsh; complete quit must stop only owned processes; exact desktop/dsh version equality. Standard filesystem and shell tools used by approved tasks are separate from the bundled Harness runtime.

**Scale/Scope**: One user, one app-owned dsh process tree, one primary workspace window recreated as needed; multiple workspaces/sessions managed by upstream. Four user stories, including reference alignment; no cloud sync, automatic updates, custom plugin installation, or independently redesigned Harness UI.

**Release-test environment**: MacBook Air Mac17,3, Apple M5, 16 GB RAM,
macOS 27.0 build 26A428, local APFS sample workspaces, idle host. Record build
hash, power mode, configuration and test fixtures with measurements.
Additionally validate installation and behavior on macOS 14 Apple Silicon before
claiming the advertised minimum OS. The present host does not prove minimum-OS compatibility.

## Constitution Check

*Gate before research and rechecked after Phase 1.*

| Gate | Before research | After design |
|------|-----------------|--------------|
| I: English in version control | PASS: specification/constitution are English; all planned artifacts will be English | PASS: all generated repository prose is English |
| II: Simplicity first | PASS: repository has specifications but no reusable application implementation; reuse dsh and Tauri/native platform APIs | PASS: no agent engine, duplicate data store, application RPC proxy, UI framework, task inspector, or generic supervisor framework |
| Specification first | PASS: validated specification and user technology decisions provide intent | PASS: FR-020–FR-024 record approved constraints and the reference supplement; no application implementation performed |
| Proportional testing | PASS: identify packaging, lifecycle, safety, persistence and usability gates | PASS: contract and quickstart cover these blast radii, comparative evidence, and SC-001–SC-009 |
| Commit policy | PASS: no commit requested | PASS: future commits must use Conventional Commits, subject <=50 columns, body <=72 columns |

No unjustified violations. No unresolved design clarifications.
Release validation work is explicitly pending and must not be represented as
already passing tests.

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
src/                         # local startup, notice and failure views only
src-tauri/
├── Cargo.toml
├── Cargo.lock
├── tauri.conf.json
├── capabilities/
├── src/                     # lifecycle, window, preferences and runtime owner
├── src/bin/dsh-launcher.rs  # minimal native dsh sidecar launcher
├── binaries/                # staged node and dsh target-specific executables
└── resources/dsh/           # immutable installed package closure and metadata
scripts/
└── prepare-runtime.mjs
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
| A1 Launch/readiness | [Host startup](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/host-process.ts), `DesktopHostProcess.start`; `backend-controller.spec.ts` | **Adapted**: start once, await readiness, authenticate the connection, clean up before Retry; retain R's Web CLI announcement and 15-second deadline | FR-018/FR-020 select the Web kernel and bundled Tauri sidecars, not Electron Host IPC; users still reach a usable workspace without terminal/browser startup | US1.1–5, US4.1; quickstart A1, runtime closure/port/recovery checks |
| A2 Window lifecycle | [Window lifecycle](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/main.ts), `createMainWindow`, `focusPrimaryWindow`, `activate`, `window-all-closed` | **Adapted**: close destroys the window, reopen recreates it against the same service; work and approvals remain unchanged | Official close hides and retains the document; preserve the existing window-recreation design and FR-019. Transient page state may reset, but dsh session/work must not; this difference needs comparative verification | US1.6–7, US4.1–2; SC-007; quickstart A2 |
| A3 Quit confirmation | [Quit decision](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/quit-confirmation.ts), `confirm`, `resolveDesktopQuitPrompt`; `main.ts` quit wiring | **Adapted**: retain an ownerless native Stay / Stop and Quit confirmation whenever a service is alive; adopt one pending decision, repeated-request joining, cancellation, and conservative unknown-state handling | Official Host can report active/scheduled work and allow idle quit without prompting; FR-018 and simplicity exclude a new inspector. Product users may see an extra idle prompt, including with zero windows; no unknown state authorizes silent interruption | US1.8, US2.6, US4.4; SC-007/SC-009; quickstart A3 |
| A4 Menus/shortcuts | [Keyboard routing](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/keyboard.ts), `sendMenuClose`, `shortcutsCloseWindow`; `main.ts` application menu | **Adapted**: native Close Window / Command+W and Quit / Command+Q share their action handlers; other task shortcuts stay upstream-owned | Official Close Page uses contextual renderer routing and configurable bindings. Retain the chosen native window command and no privileged Web bridge; users get explicit close/quit behavior, not full official shortcut customization. Verify Web content cannot consume a native command and change its outcome | US1.6/8, US4.3; FR-023; quickstart A4 |
| A5 Error recovery | [Fatal recovery](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/fatal-recovery.ts), `DesktopFatalRecovery.report`; `startup-error.ts` | **Adapted**: retain explicit recovery after owned-child cleanup, categorized non-secret explanations, and no replay | Official dialogs can include nested error text/report paths and disable third-party plugins. FR-004/FR-014 and excluded plugin management require sanitized local views and no copied report/export/plugin-repair surface; users retain safe recovery without secret leakage | US1.5, US2.5, US3.3, US4.2/5; quickstart A5 |
| A6 Release compatibility | [Release identity](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/release.ts), `parseDesktopRelease`; [Runtime validation](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/runtime-tree.ts), `verifyDesktopRuntime` | **Adapted**: adopt immutable matching shell/dsh identity and validated bundle metadata, mapped to the existing Tauri Runtime Manifest, exact prerelease equality and checks | FR-020/FR-021 retain Tauri resources and the pinned R release; Electron Host protocol/pnpm descriptors are not copied. Users keep a self-contained compatible installation and full version identity, not an upgrade to O | FR-020/FR-021, US4.1/5; quickstart A6 and bundle mismatch |

All six decisions are complete at planning level. Source inspection is confirmed;
product/official runtime comparisons are **pending**, not passing. See
[quickstart comparison protocol](quickstart.md#official-desktop-comparison-protocol)
for fixtures, observed differences and SC-008 release evidence.

Exclusions apply across the matrix: official account flows, automatic/mandatory
updates, reporting/telemetry, custom plugin management, Windows/Linux behavior,
and Electron renderer/native bridges are not added. These are exclusions due
to existing scope, safety and FR-018/FR-020, not missing adoption decisions.

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
The incremental pass inspected the pinned official startup/lifecycle, quit/keyboard,
and recovery/release tracks in parallel; section 8 records the findings without
reopening decisions from sections 1–7. No design clarification remains.

## Phase 1: Design and Contracts

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
3. Exercise upstream tasks, approvals, settings and history in WKWebView;
   constrain excluded upstream surfaces through a narrow profile overlay.
4. Sign/notarize and execute the installed-app acceptance and performance gates,
   including A1–A6 comparative evidence and SC-008/SC-009. No `tasks.md` or
   application code is generated by this incremental planning update.

## Complexity Tracking

No constitutional exceptions requested. The native sidecar launcher is required
by the explicit two-sidecar distribution constraint and upstream's JavaScript
entry point; it must remain limited to startup and process ownership.

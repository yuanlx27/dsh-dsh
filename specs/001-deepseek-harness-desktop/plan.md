# Implementation Plan: DeepSeek Harness Desktop

**Branch**: `001-deepseek-harness-desktop` | **Date**: 2026-10-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-deepseek-harness-desktop/spec.md`, including the user's Tauri, bundled Node/dsh sidecar, and version-equality decisions.

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

**Scale/Scope**: One user, one app-owned dsh process tree, one primary workspace window recreated as needed; multiple workspaces/sessions managed by upstream. Three user stories; no cloud sync, automatic updates, custom plugin installation, or independently redesigned Harness UI.

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
| Specification first | PASS: validated specification and user technology decisions provide intent | PASS: FR-020/FR-021 record the new constraints; no application implementation performed |
| Proportional testing | PASS: identify packaging, lifecycle, safety, persistence and usability gates | PASS: contract and quickstart cover these blast radii and SC-001–SC-007 |
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

## Phase 0: Research

[research.md](research.md) records release selection, sidecar layout, readiness,
security, window lifetime, quit behavior, storage and testing decisions.
Independent research tracks were executed in parallel using source/registry/docs
requests. No separate research-agent facility was available in this session.

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
3. Exercise upstream tasks, approvals, settings and history in WKWebView;
   constrain excluded upstream surfaces through a narrow profile overlay.
4. Sign/notarize and execute the installed-app acceptance and performance gates.

## Complexity Tracking

No constitutional exceptions requested. The native sidecar launcher is required
by the explicit two-sidecar distribution constraint and upstream's JavaScript
entry point; it must remain limited to startup and process ownership.

# Desktop acceptance

## Planning review (T001)

The [plan](../../specs/001-deepseek-harness-desktop/plan.md#official-desktop-alignment-fr-022fr-024)
and [research](../../specs/001-deepseek-harness-desktop/research.md) contain all six
A1–A6 decisions, immutable source locations, expected outcomes, reasons and user
impact. Reviewed before implementation: A1 adapts private authentication and
single-owner startup to the Web CLI/Tauri bridge; A2 adopts retained hide/show;
A3 intentionally always confirms a live service without inspecting tasks;
A4 uses shared native Close Window/Quit handlers, not contextual Close Page;
A5 provides sanitized explicit recovery without replay/report export/plugin
repair; A6 validates the exact canonical release and immutable runtime closure.
No decision requires changing the agreed runtime or adding an agent subsystem.
This review satisfies the planning portion of SC-008 only. All runtime,
comparative and installed-app evidence remains pending.

## Baselines

- R (bundled runtime): `@deepseek-ai/dsh@0.2.0-rc.2`, source
  `639ed015397290b3745d163aafe02ffee4aa3f84`, Node `24.21.0`.
- O (behavioral reference only):
  `5badb15009ae1756c3afe0ae0cef1faafc290ccc`, official desktop `0.2.1-alpha.1`.
- Product canonical version: `0.2.0-rc.2`. Never mix O's packages into R.

These are test provenance, not a new runtime reference registry. Baseline or
behavior changes require affected A1–A6 review and invalidation/retesting of
prior evidence under FR-024.

## Scenario coverage

Use the [specification](../../specs/001-deepseek-harness-desktop/spec.md),
[contracts](../../specs/001-deepseek-harness-desktop/contracts/) and
[quickstart](../../specs/001-deepseek-harness-desktop/quickstart.md) for exact
steps and thresholds. Shell readiness is not upstream task readiness.

| Story / area | Requirements | Scenarios and evidence destination |
|---|---|---|
| US1: setup and safety | FR-001–005, FR-014–015, FR-018 | First/saved launch; failed acknowledgement writes; absent/rejected/replaced/removed credentials; read-only credential source; private file modes; missing/read-only workspace; `startup-security.md`, `reports/us1.md` |
| US1: runtime and access | FR-016–018, FR-020–021, FR-025 | Offline/clean/path-space launch; version, architecture and closure failures; private exchange; binary/multipart, stream and cancellation fidelity; denied frames/origins/windows/stale handles and unauthenticated index/API/stream; `reports/foundation.md`, `reports/us1.md` |
| US1: lifecycle | FR-010, FR-019, FR-023 | Retained draft/selection/scroll/transfer identity; one service; native close/reopen and ownerless quit; Stay/dismissal/failure; cleanup before exit; `reports/lifecycle-performance.md` |
| US2: tasks and control | FR-006–010, FR-014, FR-016–018 | Every task state, queued/steered/identical inputs; explicit allow/deny; in-flight Stop without undo; multiple workspaces/sessions; failure/retry without replay; `tasks-control.md`, `reports/us2.md`, `reports/us2-recovery.md`, `reports/task-performance.md` |
| US3: persistence | FR-011–014, FR-018 | Distinct identities with duplicate/blank titles; committed history, interrupted work, pending approval; incomplete tail vs corrupt/unsupported/missing records; discovery omission; moved/restored workspace; `history.md`, `reports/us3.md`, `reports/history-recovery.md`, `reports/history-performance.md` |
| US4: official alignment | FR-022–024 | Separately observed A1–A6 O/product results, explained differences, joined native quit/command routes, baseline review; `official-comparison.md`, `reports/official-reference.md`, `reports/alignment.md`, `reports/quit-alignment.md` |
| Cross-story: accessibility | NFR-001 | Keyboard and VoiceOver on shell surfaces; focus/name/status transitions and preservation of upstream capability; `accessibility.md`, `reports/accessibility.md` |
| Cross-story: delivery/network | FR-001, FR-004, FR-017, FR-020–021, FR-025 | Final signed-resource/plist checks; installed distribution, offline startup, no extra destinations or native authority; `reports/distribution.md`, `reports/security-network.md`, `reports/release-readiness.md` |

## Quantitative gates

| Criterion | Required trials / outcome | Evidence |
|---|---|---|
| SC-001 | >=10 developers; >=90% independent receipt-acknowledged first submission within 5 minutes | `reports/usability.md` |
| SC-002 | >=19/20 saved-setup launches usable within 10 seconds | `reports/lifecycle-performance.md` |
| SC-003 | >=19/20 runs with every response/action update visible <=1 second from native receiving boundary; every Stop acknowledgement <=1 second | `reports/task-performance.md` |
| SC-004 | Every gated/denied action unperformed; actual in-flight Stop outcomes | `reports/task-performance.md` |
| SC-005 | 20 relaunches + 10 forced recoveries retain committed records without replay; >=19/20 1,000-message openings <=2 seconds, already connected | `reports/history-recovery.md`, `reports/history-performance.md` |
| SC-006 | >=90% independent summary-outcome discovery, original-session/workspace retrieval and correct approval/completion identification; >=80% ratings >=4/5 | `reports/usability.md` |
| SC-007 | 10 retained close/reopen cycles + 10 confirmed quits; every Stay preserves work | `reports/lifecycle-performance.md` |
| SC-008 | Six complete planning decisions; all runtime comparisons pass with explained differences | Planning review above; `reports/alignment.md` |
| SC-009 | 10 trials: two each active/open, active/hidden, approval/hidden, unknown work, repeated quit; Stay then confirm; <=1 unresolved dialog and actual cleanup before exit | `reports/quit-alignment.md` |
| SC-010 | All unauthorized frame/window/origin/stale-handle and unauthenticated protected requests denied; remote direct access unavailable; no renderer/diagnostic service auth | `reports/security-network.md` |

## Fixture safety

- Use separate disposable dsh data and workspace roots for standalone R, O and
  product runs. Never point fixtures at live user data or real projects.
- Supply fixture credentials through untracked inputs. Never commit keys,
  Cookie/Set-Cookie values, launch-token URLs or raw diagnostic/network payloads.
- Commands and edits must have observable bounded effects in disposable roots.
  Permission policy remains upstream-owned; no dismissal grants approval.
- Generate histories using supported pinned upstream facilities, not invented
  record schemas. Inventory committed records without storing their contents.
- Only terminate processes owned by the fixture/app; do not scan/kill arbitrary
  dsh instances. Ownership is not a sandbox for detached approved commands.
- Same-user credential theft/forged valid cookies are accepted limitations, not
  expected-denial fixtures. Public non-index static assets may remain accessible.
- Do not disable system security controls for installation or testing.

## Environments and evidence rules

Native target: macOS 14+ Apple Silicon (`aarch64-apple-darwin`). Timing host:
MacBook Air Mac17,3, M5, 16 GB, macOS 27.0 build 26A428, idle, local APFS fixtures.
Record actual hardware/OS, power mode, build/revision/manifest hashes, canonical
versions, signing mode, fixture IDs, sanitized timestamps and owned process/
document/listener identities. Minimum-OS qualification needs a separate macOS 14
Apple Silicon runner; the timing host cannot establish it.

Reports distinguish `pending`, `passing`, and `failing`, with separately observed
product and O/R outcomes. Missing tests, source reads, documentation review,
development/browser-only runs or upstream unit tests do not pass installed
WKWebView/security/lifecycle/official-comparison gates. Upstream requirement gaps
block release instead of authorizing shell-owned business logic.

Pending local gates: foundation, native bridge feasibility, installed app,
security, performance, accessibility, O comparisons, minimum OS and >=10-person
usability study. Current delivery uses a local ad-hoc-signed .app/DMG without an
Apple developer account. Sign nested resources, regenerate final inventory,
then sign the outer app; verify installed bytes without modifying them afterward.
Developer ID signing, hardened-runtime/JIT qualification, notarization/stapling
and public-distribution/Gatekeeper qualification are deferred separate gates.
Local ad-hoc success never implies public-release readiness.

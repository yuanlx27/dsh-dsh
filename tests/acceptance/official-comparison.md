# Official comparison protocol and change review (T045/T051)

## Immutable identities and separated roots

Official reference **O** = `5badb15009ae1756c3afe0ae0cef1faafc290ccc`, desktop
0.2.1-alpha.1. Shipped engine **R** = `639ed015397290b3745d163aafe02ffee4aa3f84`,
published dsh 0.2.0-rc.2 / Node 24.21.0. O is reference evidence only: do not mix
its packages, Host IPC, task inspector or Electron APIs into R/Tauri.

Use independent disposable HOME/data/workspace roots for O and product; never
share a live session store, use actual credentials/projects, or infer source from
an app's display name/version alone. Record immutable checkout and clean tracked
tree, lockfile hash, Node/pnpm/native toolchain, artifact hash/architecture, build
options and actual OS. For an official installed artifact, establish that it was
built from O independently of a similarly named public release.

## Reproducible source/unit and native build preparation

Follow [quickstart](../../specs/001-deepseek-harness-desktop/quickstart.md#official-desktop-comparison-protocol).
A full Node installation must include development headers; an isolated copy of
just the Node executable is insufficient for O's native-system build. Use O's
**pnpm 11.7.0**, not whatever global pnpm happens to be installed. Supply no model
key and run with a minimal environment and disposable HOME. Keep downloads,
reference checkout, dependencies and generated programs outside product sources.

```sh
git clone https://github.com/deepseek-ai/deepseek-harness.git official-reference
git -C official-reference checkout --detach 5badb15009ae1756c3afe0ae0cef1faafc290ccc
git -C official-reference rev-parse HEAD
cd official-reference
pnpm --version                    # must be 11.7.0
node --version                    # record; full installation with headers
pnpm install --frozen-lockfile
pnpm run build:native-system
pnpm exec vitest run apps/desktop/tests/backend-controller.spec.ts \
  apps/desktop/tests/quit-confirmation.spec.ts \
  apps/desktop/tests/keyboard.spec.ts \
  apps/desktop/tests/fatal-recovery.spec.ts \
  apps/desktop/tests/desktop-build-version.spec.ts
git status --short                # no tracked source changes
```

For native artifact preparation, O declares root `pnpm run build`,
`pnpm run package:desktop:mac:arm64:dir` and `pnpm run package:desktop:mac:arm64`.
Read O's packaging prerequisites before execution, keep its credential inputs
untracked, and do not use upload/release/publish scripts. These artifact commands
are **not claimed executed** by the unit-reference run. Dependency/native build
success alone is not an installed official app; unavailable prerequisites leave
native comparisons pending. Do not patch O to make tests pass.

The current actual run is recorded in [official reference](reports/official-reference.md):
exact O, frozen install, native addon build and 101 tests pass. No official GUI
was launched. Product build and installed evidence remain independently pinned.

## A1–A6 comparison records

Each row requires separately observed O and product outcomes, scope/identities,
reason, user impact and an evidence link. Source findings in
[plan](../../specs/001-deepseek-harness-desktop/plan.md#official-desktop-alignment-fr-022fr-024)
and [research](../../specs/001-deepseek-harness-desktop/research.md#8-official-desktop-reference-supplement)
are normative decisions, not passing native comparison records.

| ID | Procedure / approved adaptation | Product evidence available / missing |
|---|---|---|
| A1 | Joined startup, private auth, app-origin HTTP/stream, explicit failure; Tauri Web CLI and actual WK frame authority replace Electron Host/header interception | Authentication/native transport reports are subsets; actual O installed transport/renderer-state comparison remains pending |
| A2 | Close retains/hides same document/service, reopen focuses; actual failure recreation need not preserve unsaved context | Retained-window/native pending hide/show subsets; full draft/selection/scroll/transfer 10-cycle and O installed counterparts remain pending |
| A3 | Joined ownerless Stay/Stop and Quit; product always warns for live service, unlike O inspected-idle silent quit | Quit policy/tests and native idle/Stay cases exist; SC-009 ten-trial matrix and actual O remain pending. Extra idle prompt is intentional, not a bug or inspector requirement |
| A4 | Native Close Window/Cmd-W and Quit/Cmd-Q share outcomes; O contextual Close Page/configurable shortcut store intentionally omitted | Native command subsets; both content-focus/menu/keyboard routes and full installed reference comparison remain pending |
| A5 | Explicit cleanup before Retry, non-secret explanation, no replay; no raw report export/plugin repair | Installed backend exit/Retry report is a product pass for its sampled case; O installed counterpart and pending-work/recovery matrix remain pending |
| A6 | Exact shell/dsh release plus installed closure/numeric Apple metadata; O 0.2.1-alpha.1 need not equal R 0.2.0-rc.2 | Product inventory/version/signature and official unit version checks exist; installed O identity/damage comparisons and final local distribution gate remain pending |

Link product reports through [US1](reports/us1.md), [US2](reports/us2.md),
[native recovery](reports/native-service-retry.md), [native commands](reports/native-commands.md)
and [lifecycle performance](reports/lifecycle-performance.md). T048/T049 must
create actual comparison reports; this protocol does not fill them with passes.

SC-009: two trials each active/open, active/hidden, approval/hidden, unknown work,
repeated quit. In every product trial choose Stay first, then confirm; check <=1
unresolved dialog, untouched work on Stay, owned cleanup before exit, local and
Harness focus, both menu and keyboard. Source/unit evidence can reproducibly
cover unknown/late/disposed states but cannot replace installed zero-window,
focus, dialog and owned-process observations.

## Evidence rules

- **Pending**: procedure/precondition/environment not executed or available;
  source-only inspection, unit mocks or another platform do not upgrade it.
- **Passing scoped evidence**: exact identified setup, executed assertions and
  observed actual results; state whether unit, backend, debug WK or installed.
- **Failing**: actual requirement mismatch; preserve it and block the affected
  release gate. Approved A3/A4 differences are explicitly judged against the
  chosen adaptation, not against identical UI.
- Every adopted/adapted row needs passing **comparative** evidence before
  alignment-complete/SC-008. 101 official tests do not certify six native rows.
- Keep secrets, raw reports/bodies, token URLs and actual project data out of
  evidence. Public synthetic fixtures only. No reference runtime subsystem.

## O/R change-review checklist and traceability review

Before replacing either baseline:

1. Record immutable replacement commit/package/lock/artifact identity; never adopt
   moving master/latest by name. Keep original provenance/download checksums.
2. Diff startup/auth/HTTP/stream, window lifetime, quit/keyboard, recovery, release
   metadata, persistence and credentials; list each affected A1–A6/FR/SC row.
3. Check replacement compatibility with Web profile, two Tauri sidecars, actual
   WK authority and upstream transport hooks; do not infer Electron API support.
4. Explain adopted/adapted/excluded behavior, rationale and user impact. Preserve
   accepted A3 conservative confirmation/A4 native Close Window unless separately
   approved; no inspector, shortcut store or new task/history/credential API.
5. Invalidate affected earlier test/comparison/native artifact claims, rebuild,
   rerun both sides' affected scenarios and security/lifecycle/persistence gates.
6. Record unavailable evidence pending and actual failures release-blocking;
   require all affected evidence before adoption/alignment-complete claims.

Review of current plan's six rows and research section 8 confirms unchanged
A1–A6 identities, symbols, adaptations, FR mappings and unavailable-evidence rule.
FR-025 reconciliation in research section 9 continues to require actual native
frame authority; no source-only findings are promoted to runtime proof. Research
is historical inspection; current executions belong in reports. Missing native
O/platform/quantitative evidence blocks alignment-complete, not independent work.
T051 is this documented review/checklist, not an engine/reference upgrade.

# T023: semantic startup, safety and failure surface

Added semantic HTML, strictly typed TS controller/DOM binding and minimal CSS.
`DESIGN.md` records conservative shell-only style recommendations; no upstream
Harness redesign, framework, remote font or extra UI dependency was introduced.

The shell displays owned service starting/available/failed status with fixed
messages/next actions. Available is explicitly not task-ready: credential,
workspace and task readiness stay inside Harness. It does not display raw
exceptions, process logs or arbitrary native diagnostics. Snapshot parsing is
closed to known fields/status/category values; there is no token/cookie field.

Notice revision is `experimental-2026-10-06`, tied to this notice content, not the
app version. T028 native acknowledgement must use the same revision (and verify
it in cross-layer tests). The UI never writes preferences or flips an accepted
flag on a click/dismissal. Entry requires available service, exact current
revision and a successful native result with persisted acknowledgement. Busy
operations disable entry; stale polling cannot overwrite a later saved decision.
Native T024/T028 admission/storage remains authoritative, never this UI check.

The modal `<dialog>` has an explicit accessible heading/summary, non-accepting
dismiss action, keyboard Escape behavior and focus return. Native semantic
buttons have labels; status is politely live-announced; custom focus is visible;
controls wrap and content/dialogs scroll. No innerHTML or raw-error interpolation.
The separate shell uses a small typed native bridge contract. Its callbacks are
not wired or renderer-authorized here: T024/T028/T031 perform native integration.
Until supplied, it fails closed with a fixed unavailable message.

## Executed evidence

- Tests first: missing `src/shell/state.ts` produced exit 1 before assertions.
- Six Node tests pass: asynchronous saved-notice entry gate, failed write/old
  revision/no raw-error leak, explicit retry/not task readiness, stale callback
  protection, closed snapshot parsing and structural semantic/focus markup.
- Controlled notice-gate bypass caused three behavioral assertions to fail,
  including native entry count 1 vs expected 0, exit 1. Restored and reran green.
- `npm run build:frontend` now succeeds, including source policy, strict tool
  typing, strict browser compilation and actual dist HTML/CSS/JS generation.
  Generated JS stays ignored and no committed JS sources were added.
- Full Node suite: **48 passing, 1 failing** only on absent T027 `transport.ts`.

These are controller/markup/build tests, not an executed browser DOM, real native
acknowledgement, keyboard/VoiceOver or installed-app acceptance. Dialog rendering,
focus/announcements, notice revision persistence, failed native writes and frame
security remain actual macOS qualification gates. No product-wide WCAG claim.

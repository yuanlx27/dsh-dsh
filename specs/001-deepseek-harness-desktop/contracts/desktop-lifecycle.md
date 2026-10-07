# Desktop Lifecycle and UI Contract

## Boundary

The product exposes a macOS window, native application commands and upstream
dsh Web UI at `dsh-app://app`. It does not expose a new task API. Rust mediates
authenticated HTTP/live-stream transport; dsh retains protocol, state, storage
and permission semantics. The local startup/notice/error authority is separate
at `dsh-app://shell`.

## User commands

| Input | Required behavior |
|-------|-------------------|
| Launch / Dock reopen / Open Workspace Window | Show/focus the retained primary window; create only if genuinely absent/failed; join existing startup or same ready dsh instance |
| Window close / Command+W | Hide/retain primary window and document, draft, selection, scroll and authorized transport; no quit, service restart, task Stop or permission decision |
| Application Quit / Command+Q, including no windows | Show native Stay / Stop and Quit confirmation when service is alive; one dialog at a time |
| Stay / dialog dismissal | Preserve the service, active work and current workspace; never authorize shutdown |
| Stop and Quit | Disable/close task-capable windows, stop owned process tree and wait for exit before completing application quit |
| Stop in Harness | Use upstream task cancellation; keep dsh and the application alive |
| Retry in local failure view | Clean up failed generation, verify bundle, start once; no task replay |
| About | Display canonical desktop version, identical dsh version and bundled Node version; no secret URL |
| Safety Notice menu item | Reopen experimental-operation notice without changing tasks |

Always confirming full quit is deliberate: no Electron-only task-inspection
service is introduced. A confirmation must mention that running work will stop
across all sessions/workspaces served by the owned instance, not just the visible
session, and completed file changes are not undone. Stay leaves all work untouched
by the shell. It must work without an owner window.
Menu and keyboard routes share the same close/quit handlers; focused Harness
content must not consume a native command and change its outcome.
Repeated quit requests focus/join the existing pending decision, not stack dialogs.
Unknown task status receives the same conservative warning. Dialog failure or
dismissal never authorizes shutdown: preserve work and show a sanitized next action.
Once confirmation is accepted, disposal prevents new actions and ignores late
responses; do not claim to undo already executed actions.

## Startup and failure views

Before readiness, render a local starting view. Before first task-capable
navigation, require acknowledgement of the experimental safety notice covering
command execution, file changes, absence of isolation guarantees and no undo.
A declined/dismissed notice never counts as acknowledgement.

Service readiness is not task readiness. After service connection and current
notice acknowledgement, missing credentials or an invalid/unselected workspace
leave setup incomplete while settings and available history remain accessible.
Task readiness additionally requires a valid workspace and configured credentials
not currently reported unavailable/rejected. Follow FR-002: no separate shell-owned
online credential check or promise of continuous model availability. Rejected
credentials require correction; application readiness remains independent of
upstream task state and does not itself cancel work.

Failure states distinguish invalid/incompatible bundle, launch failure,
startup timeout, connection failure and unexpected service exit. Each includes
a fixed user-readable explanation and next action; raw stdout/stderr and tokens
are not UI error messages. A service that fails while all windows are closed is
reported when reopened. Persisted history remains dsh-owned and recoverable.

## Harness user surface

The supported release must supply FR-003–FR-009 and FR-011–FR-013 end to end:
masked model credentials with upstream plaintext, owner-only storage under
FR-004; workspace selection; session creation and history;
single active task per session; streamed responses/actions; allow/deny approvals;
Stop acknowledgement; explicit service recovery and upstream-owned history/
continuation behavior when a workspace directory is unavailable. No directory
replacement, session migration or reassociation mechanism is added by the shell.
Workspace switching, any upstream prompts and task continuation/stopping follow
dsh even when multiple sessions have work underway. The shell adds no task
inspection, workspace-switch confirmation or bulk-stop policy. Any upstream gap
against the remaining agreed requirements blocks release instead of authorizing a
shell reimplementation.

Workspace validity and per-operation permissions are dsh-owned under FR-005
and FR-018. Present upstream availability and permission failures; add no shell
permission scans or separate task-admission policy. Lack of write permission
alone is not a reason to prohibit read-only analysis. History availability is
independent of the current directory, and no blanket file-access guarantee is
made.

Model credential storage/settings concealment and arbitrary tool/model output
remain dsh-owned under FR-004. The shell never injects saved credentials into
conversations or shell diagnostics, and does not copy potentially secret-bearing
raw output into its startup/error views. No blanket upstream-output redaction or
history rewriting is added. Service tokens/cookies remain native-only under
FR-025; FR-017 still excludes additional data destinations, telemetry and sync.

Submission acceptance, rejection, queueing, steering and repeat handling remain
dsh-owned under FR-006. The shell adds no text-based deduplication, task-start
lock or separate admission policy; forwarding, reconnect and window reopening
must not duplicate or automatically replay an already-submitted request.

No custom plugin-installation, automatic-update, telemetry or feedback-reporting
surface is added. A narrow launch overlay disables excluded upstream entry
points without modifying task or permission behavior.

## Shell accessibility

NFR-001 applies to shell-owned startup, failure/retry, safety-notice and native
quit-confirmation surfaces. Support keyboard-only operation, visible focus for
custom controls, dialog focus entry and appropriate restoration, meaningful
accessible names and non-color-only status/error information. Native or standard
semantic controls must expose controls, notices and key state changes to VoiceOver.
Upstream setup/settings, conversations and approvals retain dsh accessibility;
embedding must not disable upstream keyboard/assistive-technology capability.
No replacement accessibility layer or unassessed product-wide WCAG claim is added.

## Security boundary

Harness content receives no native shell/filesystem/general network authority.
Only the current authorized application main frame may use the narrow HTTP/live-
stream adapter in the [runtime contract](runtime-bundle.md#authenticated-transport).
Check actual native WebView/frame identity, application origin, notice and launch/
window generations before forwarding. Reject external pages, other windows,
child frames and stale handles. Do not trust a caller-supplied label or main
window URL as proof of the calling frame. Use a frame-aware WKWebView handler;
local status/retry/notice commands remain separately scoped. Native menus do not
depend on a renderer.

Main-frame navigation stays at the application origin; deny file/data and
unapproved origins. Explicit external HTTP(S) links open in the system browser
without service token/cookie. Rust owns the token exchange and cookie; no auth
value appears in page URLs, boot data, renderer cookie jars or IPC results.

The service remains loopback bearer-authenticated, not process-identity-bound.
Unauthenticated local session/task/approval HTTP and WebSocket requests are
rejected. Public non-sensitive static assets may remain accessible locally.
Same-user credential theft/forged cookies, privileged attackers and compromised
authorized application code are outside the isolation guarantee. No sandbox or
isolation from agent tool processes is claimed.

## Official reference and intentional differences

The [plan alignment matrix](../plan.md#official-desktop-alignment-fr-022fr-024)
records A1–A6 against immutable O, separately from shipped R. This contract
preserves the chosen architecture; it adapts authentication/forwarding to Tauri
without exposing general Electron IPC.

- **A1 / FR-025**: Adopt native cookie ownership, packaged app-origin UI and
  authenticated forwarding. Tauri's frame-aware HTTP/stream bridge replaces
  Electron protocol forwarding/WebSocket header interception; task APIs and
  permission behavior remain upstream-owned.

- **A2**: Adopt official hide/retain and show/focus on normal close/reopen.
  Preserve document identity and prevent closure-induced draft/selection/scroll
  reset without a separate shell persistence store. Actual destruction/page
  failure may require recreation; unsaved state is not guaranteed after crash,
  full quit or page failure. Persisted recovery remains dsh-owned.
- **A3**: Official Host inspection can suppress an idle prompt. Product always
  confirms while its service is alive, including zero windows and unknown work
  status. The extra idle prompt is intentional; no task inspector is added.
- **A4**: Official Close Page is contextual/configurable. Product Close Window
  is the explicit native command/Command+W. Upstream page/task shortcuts remain
  upstream-owned; no native shortcut bridge or shortcut store is added.
- **A5**: Official fatal recovery can include reports/plugin repair. Product
  provides categorized error views and explicit Retry after cleanup, with no raw
  secret-bearing diagnostics, report export, plugin repair or automatic task replay.

Review acceptance against these stated differences, not full feature/UI parity.
Unexplained differences or missing comparative evidence prevent a passing
alignment result; use the [quickstart protocol](../quickstart.md#official-desktop-comparison-protocol).

## Required lifecycle acceptance

- Ten close/reopen cycles retain the same launcher/service PID and pending
  approval/current session with no cancellation or duplicate submission.
- Ten confirmed quits leave no owned dsh listener/process; cancel preserves it.
- Quit and cancellation work with zero windows, active work and pending approval.
- Repeated launch/reopen/Retry/Quit operations coalesce rather than duplicate.
- Menu and keyboard close/quit routes agree with both local and Harness content
  focused; Command+W does not acquire the meaning of an unrelated page action.
- Ten SC-009 quit trials cover repeated requests, zero windows, pending approval
  and unknown work status; each has at most one unresolved confirmation, declined
  quit preserves work, and confirmed quit waits for owned service exit.
- A failed or dismissed quit dialog and a late response after disposal never
  authorize a new shutdown decision.
- Forced shell/service exits retain persisted history, show interruption and
  never approve or replay pending work.
- Authorized app flows work through native HTTP/live-stream forwarding; denied
  windows/origins/frames cannot obtain authentication or use the bridge. Direct
  unauthenticated local API/stream clients and another computer fail as specified
  by SC-010; public assets and stolen/forged bearer credentials are not expected
  to be denied solely by process identity.
- Normal close/reopen retains document identity and authorized transport without
  closure-induced draft/selection/scroll reset or interruption of dsh work.
  Actual destruction/page replacement/failure invalidates affected handles;
  recreation requires fresh validated handles and denies stale generations.
- Credential save/replace/remove/relaunch retain upstream behavior; the file is
  `0600`, overly permissive files are rejected, and fixture keys are absent from
  shell-authored diagnostics/startup views and shell-created conversation content.
  Arbitrary dsh/tool/model output is not subject to a shell redaction guarantee;
  service token/cookie confidentiality remains independently required.

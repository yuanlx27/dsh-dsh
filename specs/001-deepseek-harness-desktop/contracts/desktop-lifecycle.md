# Desktop Lifecycle and UI Contract

## Boundary

The product exposes a macOS window, native application commands and upstream
dsh Web UI. It does not expose a new task API. dsh retains its authenticated
fetch/SSE/RPC transport, state, storage and permission semantics.

## User commands

| Input | Required behavior |
|-------|-------------------|
| Launch / Dock reopen / Open Workspace Window | Create/focus the primary window; join existing startup or reconnect to the same ready dsh instance |
| Window close / Command+W | Close window only; no quit, service restart, task Stop or permission decision |
| Application Quit / Command+Q, including no windows | Show native Stay / Stop and Quit confirmation when service is alive; one dialog at a time |
| Stay / dialog dismissal | Preserve the service, active work and current workspace; never authorize shutdown |
| Stop and Quit | Disable/close task-capable windows, stop owned process tree and wait for exit before completing application quit |
| Stop in Harness | Use upstream task cancellation; keep dsh and the application alive |
| Retry in local failure view | Clean up failed generation, verify bundle, start once; no task replay |
| About | Display canonical desktop version, identical dsh version and bundled Node version; no secret URL |
| Safety Notice menu item | Reopen experimental-operation notice without changing tasks |

Always confirming full quit is deliberate: no Electron-only task-inspection
service is introduced. A confirmation must mention that running work will stop
and completed file changes are not undone. It must work without an owner window.
Once confirmation is accepted, disposal prevents new actions; do not claim to
undo already executed actions.

## Startup and failure views

Before readiness, render a local starting view. Before first task-capable
navigation, require acknowledgement of the experimental safety notice covering
command execution, file changes, absence of isolation guarantees and no undo.
A declined/dismissed notice never counts as acknowledgement.

Failure states distinguish invalid/incompatible bundle, launch failure,
startup timeout, connection failure and unexpected service exit. Each includes
a fixed user-readable explanation and next action; raw stdout/stderr and tokens
are not UI error messages. A service that fails while all windows are closed is
reported when reopened. Persisted history remains dsh-owned and recoverable.

## Harness user surface

The supported release must supply FR-003–FR-009 and FR-011–FR-013 end to end:
masked model credentials; workspace selection; session creation and history;
single active task per session; streamed responses/actions; allow/deny approvals;
Stop acknowledgement; explicit recovery and confirmed unavailable-directory
replacement. Workspace changes during active work must use upstream's safe
confirmation path. Any upstream gap blocks release instead of authorizing a
shell reimplementation.

No custom plugin-installation, automatic-update, telemetry or feedback-reporting
surface is added. A narrow launch overlay disables excluded upstream entry
points without modifying task or permission behavior.

## Security boundary

Harness Web content receives no native shell/filesystem/general invoke
capabilities. Local startup/notice/failure content can call only narrowly scoped
shell status/retry/acknowledgement actions; validate the caller window and
current local origin and define application-command ACLs explicitly.
Native menu actions do not depend on a renderer.

Restrict main-frame navigation to the exact runtime origin; deny file/data and
unapproved origins. Explicit external HTTP(S) links open in the system browser,
never with the service authentication token. Test attempted native invocation
from Harness content, external content and child frames. Retain upstream
authentication and trust checks.

## Required lifecycle acceptance

- Ten close/reopen cycles retain the same launcher/service PID and pending
  approval/current session with no cancellation or duplicate submission.
- Ten confirmed quits leave no owned dsh listener/process; cancel preserves it.
- Quit and cancellation work with zero windows, active work and pending approval.
- Repeated launch/reopen/Retry/Quit operations coalesce rather than duplicate.
- Forced shell/service exits retain persisted history, show interruption and
  never approve or replay pending work.

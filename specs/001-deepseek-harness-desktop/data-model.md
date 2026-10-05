# Data Model and Ownership

This document specifies conceptual fields and invariants, not new dsh database
tables or a replacement wire format. Map upstream fields using the pinned
release; do not migrate or mirror upstream records in the shell.

## Upstream-owned entities

| Entity | Conceptual fields | Relationships and validation |
|--------|-------------------|------------------------------|
| Workspace | id, directory, displayName, availability | Has sessions; execution requires an accessible selected directory; replacement requires explicit confirmation |
| Model Connection | id, model/provider, destination, credential reference/value, readiness | Used by tasks; credentials are private, concealed and excluded from history/shell diagnostics; removal blocks dependent new work |
| Session | id, workspaceId, title, createdAt, lastActivityAt, messages, tasks | Belongs to one workspace; at most one active task per session; unavailable directory still permits history reading |
| Task | id, sessionId, request, executionState, actions, result/interruption | Request submitted once; no automatic replay after failure; completed changes are not implicitly undone by Stop |
| Permission Decision | id, taskId, action, target, status, explicitDecision | Pending/allowed/denied; no implicit allow from dismissal, close, relaunch or recovery |
| Message / Action record | upstream id/order, task association, content, outcome | Ordered durable session history; secrets excluded from ordinary presentation |

### Task presentation states

`idle -> running -> completed | failed | interrupted`;
`running -> awaiting-approval -> running` after upstream resolves an explicit
decision; `running | awaiting-approval -> stopping -> completed | failed |
interrupted` according to the actual upstream outcome. A denial does not
necessarily end the task: render what dsh reports. These are UI meanings from
FR-007, not proposed changes to upstream state enums.

An unexpected exit turns unresolved work into interrupted on recovery; a
pending approval never becomes allowed. Window close changes none of these
entities. Stop applies to a task, not the dsh process.

## Shell-owned persistent entity: Application Preferences

| Field | Type | Rule |
|-------|------|------|
| schemaVersion | positive integer | Start at 1; reject unsupported format rather than corrupting it |
| safetyNoticeRevision | optional string | Current notice revision must be acknowledged before showing the task-capable window |
| windowBounds | optional numeric rectangle | Restore only usable onscreen bounds; native defaults on invalid data |

Store atomically in app-data with user-only permissions. No credential,
authenticated URL, selected workspace, active task, session history or duplicate
model configuration belongs here. Reopened Harness selection is dsh-owned.

## Build-owned immutable entity: Runtime Manifest

| Field | Type | Rule |
|-------|------|------|
| desktopVersion | semantic version string | Exact equality with dshVersion, including prerelease |
| dshVersion | semantic version string | 0.2.0-rc.2 baseline; equality with shipped package and CLI version |
| upstreamRevision | commit hash | 639ed015397290b3745d163aafe02ffee4aa3f84 |
| nodeVersion | version string | 24.21.0; runtime probe must match |
| target | Rust target string | aarch64-apple-darwin for initial release |
| dependencyLockHash | SHA-256 | Identifies complete dependency resolution |
| artifacts | path/hash inventory | Includes executables, packages, native modules and frontend resources |
| nativeBuildNumber | numeric build identifier | Monotonic packaging metadata, not an independent product version |

The manifest is packaged with immutable resources, not editable user settings.
Canonical versions are distinct from any numeric Apple plist encoding.

## Shell-owned transient entities

### Runtime Instance

Fields: launch generation, owned launcher PID, process-group ID, owner pipe,
state, optional in-memory authenticated URL, optional sanitized failure category.
One instance per application; never adopt a service merely because a port responds.

States:
- `stopped -> starting -> ready`.
- `starting -> failed` on invalid bundle, spawn failure, bad announcement,
  timeout or early exit; clean up owned children before enabling Retry.
- `ready -> failed` on unexpected exit; report interruption, no auto-restart.
- `failed -> starting` only after explicit Retry and cleanup.
- `starting | ready -> stopping -> stopped` on confirmed quit.
- `stopping` remains visible until owned processes have exited; escalation is
  allowed but a successful quit cannot be reported with live owned dsh.

The URL contains authentication material; it is never persisted or logged.

### Window State

`absent | local-startup | safety-notice | harness | failure`.
Opening/reopening while ready does not spawn a runtime. Closing a window makes
it absent without changing runtime state. The app remains available via Dock/menu.

### Quit Decision

`none -> confirming -> cancelled | confirmed -> shutting-down -> exited`.
One native confirmation at a time. Repeated quit requests focus/join it.
Cancel returns to the previous runtime/window state. Unknown work state is
handled conservatively by the same confirmation, without duplicating dsh tasks.

## Cross-entity invariants

1. The shell never writes dsh's workspace, model, task or history records.
2. Window creation/destruction and task creation/cancellation are independent.
3. Notice acknowledgement controls entry into the task-capable Web experience.
4. Version mismatch disables task-capable navigation before startup.
5. Only explicit quit confirmation authorizes stopping a healthy service.
6. Recovery launches a new service only after owned predecessor cleanup and does
   not submit messages or resolve approvals.

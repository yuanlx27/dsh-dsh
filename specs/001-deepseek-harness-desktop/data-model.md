# Data Model and Ownership

This document specifies conceptual fields and invariants, not new dsh database
tables or a replacement wire format. Map upstream fields using the pinned
release; do not migrate or mirror upstream records in the shell.

## Upstream-owned entities

| Entity | Conceptual fields | Relationships and validation |
|--------|-------------------|------------------------------|
| Workspace | id, directory, displayName, availability | Has sessions; availability and execution access are dsh-owned; canonical path remains upstream-owned and no shell directory replacement or session migration is provided |
| Model Connection | id, model/provider, destination, credential reference/value, readiness | Upstream plaintext credential file with owner-only macOS permissions is accepted; no shell credential store; settings concealment is upstream-owned; no shell credential injection into history or disclosure in shell diagnostics; arbitrary upstream output is not shell-redacted; removal blocks dependent new work |
| Session | id, workspaceId, title, createdAt, lastActivityAt, messages, tasks | Workspace association is dsh-owned; at most one active task per session; history access and continuation with an unavailable directory follow dsh; no shell reassociation |
| Task | id, sessionId, request, executionState, actions, result/interruption | Submission identity/admission/queueing are dsh-owned; identical text is not shell-deduplicated; no shell-induced duplication or automatic replay; completed changes are not implicitly undone by Stop |
| Permission Decision | id, taskId, action, target, status, explicitDecision | Pending/allowed/denied; no implicit allow from dismissal, close, relaunch or recovery |
| Message / Action record | upstream id/order, task association, content, outcome | Ordered durable session history owned by dsh; no shell credential injection, blanket output redaction or history rewriting; service authentication remains native-only |

### Upstream credential records

`DSH_HOME/.credentials.yaml` is dsh-owned plaintext YAML. Upstream creates/replaces
it at `0600` and rejects group/other permission bits on macOS; the shell creates
its private data root at `0700`. Credential reference values and the distinct
`client-connection/browser-session` signing-secret grant remain upstream-managed.
The shell never edits or mirrors these records. Model calls resolve credentials
per operation, including later calls in an existing task; changing credentials
does not cancel already-started work. Missing credentials prevent subsequent
dependent calls, and settings follow upstream credential-availability events.
Removal shadowed by a read-only source is refused, not reported as success.
Same-user processes, including
agent tools, can read them; there is no encryption or process-isolation claim.

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
Artifact hashes describe final signed nested resources, not their pre-signing
bytes or original download checksums. Generate the final inventory after nested
signing and before outer application signing; do not change inventoried resources
or the manifest afterward. This applies to local ad-hoc and future Developer ID
builds alike; validate the actual installed bytes.
Canonical versions are distinct from any numeric Apple plist encoding.

## Shell-owned transient entities

### Runtime Instance

Fields: launch generation, owned launcher PID, process-group ID, owner pipe,
state, optional validated service origin, optional transient launch-token URL,
optional in-memory service cookie, optional sanitized failure category.
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

`starting` includes private token exchange and non-secret boot preparation; a
readiness announcement alone does not make this instance `ready`. Discard the
launch-token URL after exchange. Neither it nor the cookie may be persisted,
logged, returned to the renderer, or placed in a WebView cookie store. Upstream's
signing-secret record is separate from these shell-owned transient values.
Invalidate the cookie and all transport handles on service-generation change.

### Authenticated Transport Session

Fields: runtime generation, current primary WebView identity, native main-frame
origin, acknowledged notice revision, window generation, opaque request/stream
handles, bounded pending transfer state. The Rust owner holds authentication;
handles authorize no other window or destination and reveal no cookie.

`inactive -> attached -> detached`: attach only to the authorized main frame at
`dsh-app://app` after notice/readiness. Normal close hides the retained document
without detaching its authorized transport or invalidating its handles; show
reuses the same document/window generation. Detach on actual destruction,
document replacement/navigation or page failure. Detaching aborts renderer
transfers/streams, not dsh tasks/approvals. A recreated document receives new
window-scoped handles using the ready service cookie when available.
Late chunks/callbacks and stale handles from invalidated generations are rejected. Runtime failure/quit
invalidates every attached transport and discards native authentication.

### Window State

Document state: `absent | local-startup | safety-notice | harness | failure`;
retained-window visibility: `visible | hidden`.
Opening/reopening while ready does not spawn a runtime. Normal close changes
visibility to hidden, preserving document identity, draft, selected conversation,
scroll context and authorized transport; reopen shows/focuses the same window.
Actual destruction/page failure may produce absence/recreation with no unsaved-
context guarantee. Crash/full quit recovery uses dsh persistence, not a shell
page-state store. The app remains available via Dock/menu.

### Quit Decision

`none -> confirming -> cancelled | confirmed -> shutting-down -> exited`.
One native confirmation at a time. Repeated quit requests focus/join it.
Cancel returns to the previous runtime/window state. Unknown work state is
handled conservatively by the same confirmation, without duplicating dsh tasks.
Menu Quit and Command+Q share this decision. Late dialog responses after shutdown
begins are ignored; a dialog failure preserves the service rather than granting quit.
This decision concerns the entire owned service and all of its sessions/workspaces,
not only the visible conversation. Workspace switching and its task consequences
are dsh-owned; no separate switch decision, session inspector or bulk-stop entity
is introduced.

## Review-owned alignment records (documentation only)

These records live in the existing plan/research and acceptance reports, not
application preferences, Runtime Manifest fields or a new runtime database.

| Record | Fields and relationships | Validation |
|--------|--------------------------|------------|
| Official Reference Baseline | immutable revision O, source locations, inspection date, declared reference version; linked to shipped runtime baseline R | O and R are distinct identities; a moving URL is not sufficient, and changing O does not upgrade R |
| Alignment Decision | stable ID A1–A6, behavior area, baseline, source symbols, adopted/adapted/excluded classification, expected outcome, reason/user impact, acceptance IDs | All six FR-022 areas covered; differences explained; constraints take precedence; see plan matrix |
| Comparison Result | decision ID, O/R and product build, fixture/platform, expected/observed outcomes on each side, difference explanation, evidence links, status | `pending -> passing | failing` only after comparison; source inspection alone is not passing; changed baseline/behavior returns affected results to pending |

No entity for shell-owned task inspection, scheduled-task duplication or
configurable native shortcuts is introduced. The existing runtime owner and
Quit Decision suffice for FR-023.

## Cross-entity invariants

1. The shell never writes dsh's workspace, model, task or history records.
2. Window creation/destruction and task creation/cancellation are independent.
3. Notice acknowledgement controls entry into the task-capable Web experience.
4. Version mismatch disables task-capable navigation before startup.
5. Only explicit quit confirmation authorizes stopping a healthy service.
6. Recovery launches a new service only after owned predecessor cleanup and does
   not submit messages or resolve approvals.
7. The renderer never owns the service launch token/cookie. HTTP and live streams
   use frame-aware, fixed-owned-service forwarding; no general network authority.
8. Unauthenticated local clients cannot use session/task/approval APIs, but public
   static resources and same-user credential-compromise limitations are accepted.

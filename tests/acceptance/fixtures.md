# Disposable acceptance fixtures

Follow the [acceptance safety rules](README.md#fixture-safety). This document
prepares fixtures; it does not claim that any installed-app scenario has passed.
Use the exact R/O identities in [README.md](README.md#baselines).

## Isolated roots

Create a fresh temporary parent outside the repository for each run:

```sh
umask 077
FIXTURE_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/dsh-desktop-fixture.XXXXXX")
mkdir -m 700 "$FIXTURE_ROOT/product" "$FIXTURE_ROOT/standalone-r" \
  "$FIXTURE_ROOT/official-o" "$FIXTURE_ROOT/workspace-a" \
  "$FIXTURE_ROOT/workspace-b"
printf 'Fixture A: read-only summary input.\n' > "$FIXTURE_ROOT/workspace-a/README.md"
printf 'Fixture B: independent session input.\n' > "$FIXTURE_ROOT/workspace-b/README.md"
printf 'unchanged\n' > "$FIXTURE_ROOT/workspace-a/controlled.txt"
git -C "$FIXTURE_ROOT/workspace-a" init
git -C "$FIXTURE_ROOT/workspace-a" add README.md controlled.txt
git -C "$FIXTURE_ROOT/workspace-a" -c user.name='Fixture User' \
  -c user.email='fixture@example.invalid' commit -m 'test: initialize fixture'
```

Store fixture locations only in untracked local notes. Set standalone R's
`DSH_HOME` to `standalone-r`; give O a distinct supported data root. For installed
product tests use a disposable macOS account, whose private app-data/dsh root
cannot contain real user records. The production app must not accept a
renderer-controlled data-root override. Record the actual resolved root before
running destructive fixtures. Never share one live history store between apps.

Prepare equivalent workspace copies for each side's comparison; do not run
concurrent changes against the same files. Repeat generation for fresh trials.

## Credentials and setup

Supply a dedicated revocable test-model credential privately through upstream
Settings. Do not place it in commands, shell history, tracked fixtures/reports,
model prompts, screenshots or raw network captures. No real keys belong in Git.
The shell has no credential API or parallel store. Prepare fresh/no-key,
saved-valid, rejected, replaced and removed configurations through upstream.

After save, inspect only the `.credentials.yaml` permission mode, not its
contents; require `0600`, with app-data/dsh directories `0700`. Fully quit before
changing this disposable file to `0644` for rejection tests, then fully quit,
restore `0600` and Retry explicitly. Include an upstream-supported read-only
credential source to test refused removal; confirm the supported R mechanism
before use rather than inventing a shell credential provider. Track that case
as pending if its preparation cannot be reproduced.

Model credentials are resolved per upstream operation. Observe replacement or
removal between operations within an active task; do not assume it cancels an
already-started model request/command/edit or revokes a remote key.

## Workspaces and observable actions

Workspace A is the task/edit target; B demonstrates independent grouping and
follow-up history. Record original canonical directories and session identities
without storing conversation payloads. Prepare additional disposable copies:

- Read-only directory/files: remove write permission from the fixture only;
  expect upstream read-only analysis to remain possible where supported, not a
  new shell task-admission check. Restore permissions before disposal.
- Missing directory: move A aside after a session is committed, inspect its
  original history and upstream continuation/refusal, then restore A's original
  path. Selecting B does not migrate or reassociate A's session.
- Unreadable directory: deny access on a separate disposable copy and inspect
  upstream errors; do not revoke access to a real project or system directory.

Use the active pinned R permission policy to request explicit approval for
separate controlled operations. Record the displayed policy, action and target.
Suggested bounded effects (only after reviewing the actual proposed action):

1. Replace `controlled.txt` with a known short fixture string.
2. Create `approved-marker.txt` inside A with a known short string.
3. Execute a bounded command in A that creates `command-marker.txt`.
4. For in-flight Stop, approve a bounded delayed marker command (for example,
   sleep for three seconds before writing a marker), with no child detachment,
   network traffic, external filesystem targets or infinite execution.

Inspect Git diff and marker existence before pending, after denial/dismissal,
and after a separately allowed trial. No pending/denied action may occur; Stop
must report the actual already-started outcome, not promise rollback. Do not
forge permission records, disable policy or auto-approve to force a passing run.
If R cannot produce the required policy/action fixture, record an upstream gap.

For duplicate/queued/steered submissions, use explicit upstream input events,
including legitimate identical text, and compare dispatch/receipt identities
against standalone R. Reconnect or hide/show must not submit on the user's behalf.
Switch workspaces with sessions active in both; the shell must not inspect tasks,
confirm workspace changes or impose a bulk-stop rule.

## Service and ownership failures

Foundation ownership tests use spawned disposable helper processes and a
separately spawned sentinel, never existing user services. Record handles/group
identity while spawning; avoid persisted bare PID identity or process-name scans.

Installed-app trials record the app-owned launcher/service identity before
inducing a failure. Kill only that owned service for unexpected-exit/Retry tests;
separately terminate only the fixture app for owner-pipe cleanup. Verify owned
process/listener exit while the sentinel/unrelated fixture service stays alive.
Never publish token-bearing process output or raw startup reports.

Occupy port 3080 with a disposable loopback fixture listener; the app must use
an OS-assigned port, not adopt the fixture. Test denied app-data writes in the
disposable account and damaged/mismatched resources in a copy of the candidate,
not the original signed distribution. Do not alter final installed resources
and still describe their signatures/hashes as passing.

Clean/offline trials must still permit local setup/history without downloading
runtime dependencies. Online model-operation failures are distinct from native
service startup readiness. Startup timeout/auth failures and failed cleanup must
never permit a competing generation or silent authenticated-localhost fallback.

## History fixtures (reserved)

Reserve one fixture containing 1,000 committed upstream messages and two
workspace-associated sessions, plus completed/interrupted/pending-approval cases.
T041 will generate them using facilities verified against pinned R and validate
records using upstream rules. Do not create a speculative JSON/YAML/session
schema now. Large-fixture generation and its performance results remain pending.

Prepare duplicate and blank titles with distinct upstream session identities.
Use disposable copies for supported incomplete-tail, corrupt committed record,
unsupported header and missing record cases; retain a clean baseline and record
upstream discovery omission/refusal without silently repairing data or claiming
that an omitted session never existed.

## Evidence and disposal

Reports include fixture ID, OS/build/signing provenance, timestamps, modes,
owned identities, sanitized outcomes and separately observed R/O/product results.
No keys, launch URLs, cookie headers, raw payloads or potentially secret-bearing
upstream reports. Never grant expected-denial status to stolen/forged valid auth;
public non-index static assets are allowed. Required missing evidence is pending.

Fully quit each app, verify its owned processes have exited, revoke test keys,
restore fixture-only permissions and remove only the exact temporary root that
was created above. Do not use broad wildcard deletion or a real user data root.

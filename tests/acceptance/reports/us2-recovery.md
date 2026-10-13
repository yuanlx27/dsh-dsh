# US2 — service recovery (T037/T038 in progress)

## Existing native recovery path, now cross-layer regression-tested

Reused existing ownership/authentication/protocol/bridge/dispatcher components;
no second process manager, task engine or upstream modification was introduced.
`Application` watches native Runtime state and revokes its ServiceSession and
bridge authority on failure. The desktop watcher returns the owned window to the
local shell. Explicit shell Retry invokes joined startup, authenticates/prepares
fresh state and permits entry only when ready. Runtime prevents replacement when
predecessor cleanup is unproven. No shell task submission/approval API is added.

The new `service_exit_revokes_auth_and_handles_before_explicit_retry_without_replaying_upload`
test in `src-tauri/src/bridge/dispatch_tests.rs` composes these existing core
components with a real owned **synthetic shell child** and synthetic loopback
HTTP/WebSocket authentication fixture. It is not upstream dsh or installed WK.

Observed/asserted:

- Authenticate generation one and admit a policy-fixture sender; obtain its
  ServiceSession, buffer an unfinished HTTP upload and open an opaque stream.
- Private first-generation exit marker makes the owned child exit status 2.
  Runtime publishes Exited; Application denies entry, retires protocol/auth and
  the live-runtime witness invalidates the old native permit.
- Old upload finish and old stream read fail. After 100 ms bounded observation,
  launch count remains one; no buffered upload request reached the supplier.
- Explicit `Application.start()` (standing for Retry) alone starts generation two,
  after predecessor cleanup. Existing notice remains acknowledged. Old-generation
  admission fails; fresh admitted state obtains a different valid auth instance.
  Old auth remains invalid even for the new generation.
- Even the fresh permit cannot address either old handle. No upload was replayed.
  A separately created/written/finished fresh upload delivers exactly one matching
  public synthetic body. Owned shutdown completes normally.

The auth fixture reuses synthetic cookie bytes, so this proves session authority
retirement, **not real cookie rotation**. No task/RPC/approval semantics are parsed;
an opaque buffered upload is not a model submission or pending-approval test.
The sender is synthetic policy metadata, not actual WK sender observation.
Existing unproven-cleanup and late-failure-generation regressions still pass.

## Corrected recovery presentation — test first

`Failure::Exited` also occurs after readiness, but shell `earlyExit` claimed
**stopped during startup**. New `shell.test.ts` regression first failed on that
message. Minimal change to the existing fixed, non-secret message table in
`src/shell/state.ts` now reports the local dsh service stopped and any active work
was interrupted. Its next action explains explicit Retry without resubmitting
inputs and asks the user to review task history in Harness.

The same regression confirms repeated status refresh performs neither Retry nor
entry, failure blocks entry, and one explicit Retry does not itself invoke entry.
No task state or raw backend diagnostic is introduced into the shell.

## Checks and qualification boundary

- Targeted composed Rust regression: pass. An initial private-field test compile
  error was corrected to use the public serialized Snapshot, not new visibility.
- Shell presentation regression: RED old text → GREEN; seven shell checks pass.
- Full packaging suite: **94 passed**; source policy and strict tooling/browser
  checks pass.
- Default-feature Rust suite passes: **35 lib tests** plus integration suites;
  desktop-feature **lib/bin** Clippy passes with warnings denied. This is not an
  all-target desktop Clippy qualification; the previously recorded shared-fixture
  duplicate-module issue remains open.

At the core-regression commit b8169e0 the installed candidate was still source
2bdf282 and lacked the changed message. A subsequent separately verified
[b8169e0 native candidate](native-service-retry.md) now executes actual backend
exit, explicit Retry, persisted interrupted history and bounded no-replay, then
fresh explicit task completion. Old backend exit precedes new backend startup.
Direct installed old-cookie/handle negatives and persisted pending-work/approval
restart remain unqualified; core fixtures must not substitute for them.
T037/T038 remain unchecked; prior native task reports remain separate evidence.

The native held-task procedure above is now executed once with captured owned
identities, updated presentation, exact receipts/counts and confirmed cleanup.
Next: expand pending/queued work and interrupted recovery negatives without
speculative PID signals, real profiles or automatic submission replay.
Remaining T038 matrix includes pending approval, missing/rejected/rate-limited
model, credential change during work and multi-session Stay/confirmed quit.
T039 requires actual receiving-boundary timestamps across 20 native runs; no test
elapsed times here qualify those metrics.

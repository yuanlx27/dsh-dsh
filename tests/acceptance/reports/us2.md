# US2 — task/control parity (T036 in progress)

## Executed standalone backend subset

`tests/fixtures/standalone-task-driver.ts` boots downloaded pinned R's own
exported `runProfile` with its stock **web** profile, literal-loopback/ephemeral
port and `--no-open`. R remains the actual agent, model adapter, credential,
policy, session log and cancellation owner. The driver calls the existing Host
`sessionController` service directly: it does **not** exercise HTTP RPC admission,
Remote mux, browser frontend, desktop transport or WKWebView.

Producer pin: dsh `0.2.0-rc.2`, R
`639ed015397290b3745d163aafe02ffee4aa3f84`, Node `24.21.0`. This is a repository
runtime/web-profile execution, not an installed bundle closure qualification.
`standalone-tasks.ts` launches one separately owned Node per case with an explicit
minimal environment, private disposable HOME/DSH_HOME and workspace. No real
credential or user's manual candidate root is read, copied or inherited; no
window, keyboard or Inspector automation is used.

The existing controlled loopback supplier holds its Messages stream after a
text delta. Supported R settings/credentials APIs configure the exact
`llm-deepseek` namespace and public synthetic credential. A fixture-only fetch
guard refuses non-literal-loopback destinations and redirects; final cases assert
zero refused external attempts. Upstream stdout is discarded except a closed
boolean/count result; stderr is discarded. No bootstrap URL, raw error, header,
prompt, response or session record is retained in evidence. Disposable trees
are removed after child closure; escalation targets only the captured child.

| Case | Actually asserted |
|---|---|
| Completion | Supported prompt admitted; live Session running while supplier stream held; release ends exactly one turn with R `completed` reason and one non-interrupted assistant message |
| Stop | Same admission/running observation; R `cancel` settles exactly one turn with `aborted` reason and preserves one interrupted assistant message containing the delivered prefix |

Both cases assert one **matching RPC request identity** in the durable human
message source, authenticated fixture requests, unchanged README-only workspace,
whole R context disposal and child exit zero. R may inject other user-role context
messages; these are not counted as duplicate human submissions. Stop intentionally
does not erase already delivered text. No tool ran, so these cases establish
neither action rollback nor permission/action-effect correctness.

Missing-fixture RED preceded implementation. Targeted execution and strict tooling
checks pass; full packaging suite now has **88 passing tests**, source policy and
both tooling/browser typechecks pass. Test elapsed time is not SC-003 latency.

## Executed approval/action backend subset

`standalone-approvals.test.ts` extends the same isolated stock web-profile runner.
It selects the supported per-Session `read-only` preset and verifies effective
sandbox `read-only` plus approval `ask` before submission. The existing controlled
supplier emits exactly one **actually advertised** `bash` call requesting a
workspace-write escalation. No fixture executes the tool or constructs a fake
approval request: upstream dsh performs its real model/tool/approval pipeline.

A **test-only same-process answerer** is prepended on the documented
`approval/request` waterfall for this Session's bash call. It holds the actual
request until the driver decides. This substitutes for interactive presentation,
not for policy, gating or execution; it does not prove the stock browser/Remote
answerer, decision controls, UI dismissal, or native forwarding.

| Case | Actual audit and filesystem observations |
|---|---|
| Allow | Pending marker absent; one matched ask/`allowed-once` decision; successful tool result; exactly `approved` in approval-marker.txt |
| Deny | Pending marker absent; one matched ask/`rejected` decision; errored tool result; no marker afterward |
| Withdraw pending approval | Stop while the real ask is held; one matched `cancelled` decision; errored tool result and aborted turn; a late `allowed-once` answer is discarded, with no marker after the bounded observation/disposal |
| Stop foreground action | Grant once; observe started marker and a running Session-owned bash job; Stop settles an aborted turn/errored tool result; upstream registry has no running/stopping Session jobs at observation; stop-marker.txt remains exactly `started`, without `finished`, before and after context disposal |

The action command is bounded: write `started`, sleep 20 seconds, append `finished`;
foreground mode is explicit and its timeout is 30 seconds. No rollback is expected.
Every case asserts one tool call/result, one paired approval audit, unchanged
standing read-only/ask policy, exact allowed workspace entries/README retention,
zero refused external fetch attempts, authenticated loopback requests, whole
context disposal and child exit zero. Allow/deny proceed to upstream `completed`
turns; withdrawal/action Stop to `aborted`. No model narrative substitutes for the
parent process's filesystem checks. No process-name cleanup or user-window action
is performed.

New approval and action-Stop assertions were RED before driver support, then GREEN.
Five prior/new cases plus the action-Stop case pass strict tooling; all **88**
packaging checks and source policy pass. Test wall time is not Stop acknowledgement
or receiving-boundary latency. The bounded late-grant check is not every race;
registry observation is not an independently captured OS process-tree trace.

## Failed preparation, not passing evidence

An early selector matched the first `apiKeyEnv` field, mistakenly selecting the
web-search namespace rather than the chat adapter. Pre-guard preparations saw no
controlled supplier request and ended with AUTH failures; they **cannot be
certified loopback-only** and may have contacted the default public endpoint with
the public synthetic credential. No successful supplier response was observed;
no genuine credential or manual-user provider connection was used. Those pilots
are not parity evidence. The final exact namespace selector, non-loopback fetch
guard and zero-external-attempt assertion prevent this preparation error from
silently passing. An initial Stop assertion also incorrectly expected the partial
message to disappear; the corrected expectation follows R's documented
`assistant/message.interrupted` semantics, without changing R or product code.

The [native task handoff](native-task-handoff.md) prepares a separate private
profile and live controlled supplier for user interaction; no native case is
pre-filled as passing. The [first native attempt](native-task-first-attempt.md)
created the marker without approval: persisted permission sequences show the tool
ran under workspace-write before the read-only switch. This is an ungated
preparation attempt, not read-only bypass evidence or an approval pass.
The [installed native decision subset](native-task-decisions.md) now correlates
actual pending/Reject UI observations with a rejected audit, errored tool result
and independent marker absence before/after the decision. Other native cases
and timing/identity gates remain pending.
The [computer-use native subset](native-task-computer-use.md) subsequently
executes real Reject, pending close/reopen/Allow once, streamed Stop and started
foreground action Stop, with selected PID/window retention and confirmed cleanup.
It is an actual installed release run, but not the full task/latency/lifecycle matrix.

The subsequent [native queue/steer continuation](native-queue-steer.md) and
live standalone backend regression preserve two identical queued inputs with
distinct receipts through explicit steering and delivery. The native pending
approval panel exposes no established Stop/withdrawal route; cleanup Reject is
not withdrawal. A later unsubmitted Escape-trial preparation lost controllable
window observation, so automation stopped and owned resources were cleaned by
TERM/private supplier close, not counted as confirmed-quit acceptance.

The [backend queue Stop and session-scope cases](backend-queue-stop-and-session-scope.md)
now verify that scoped Stop leaves two identical pending inputs intact without
resuming them during a bounded observation, and that stopping A leaves B running
in a second disposable workspace. These are actual service tests, not native
workspace/queue/quit qualification. An explicitly authorized computer-use retry
found the isolated App but semantic observation timed out; no task control was
executed and the owned supplier was closed.

A later user-monitored [native queued Stop retry](native-queued-stop.md) restores
normal computer-use controls: original Stop aborts the active streamed task,
retains both distinct identical queued receipts with no deliveries, and original
Remove buttons explicitly clean them. Confirmed native quit cleans the captured
trio, with an exact-PID Accessibility fallback for the still-unobservable modal.
This is actual installed UI evidence, not proof that caffeinate caused recovery
or full task/latency/quit qualification.

The subsequent [native two-workspace/session scope case](native-workspace-session-scope.md)
correlates two original Running entries, Stop A with an interrupted/aborted
record, and B still running before controlled response release and independent
completion. It records the failed long preparation and checks exact directory
registration before submission, rather than accepting ambiguous B labels. Native
chooser/modal controls require bounded exact-owned-PID AX fallback. All captured
processes are now cleaned by confirmed native quit/private supplier close.

The [service recovery core/presentation continuation](us2-recovery.md) composes
actual owned synthetic-child exit with auth/permit/opaque-handle revocation and
explicit generation-two startup, rejecting old buffered work. It corrects the
startup-only exit wording with a RED-to-GREEN shell regression. These core tests
do not replace installed dsh/WK service-exit, task-history or Retry acceptance;
the prior installed candidate still lacks the updated shell text. A separately
built/verified [b8169e0 installed recovery candidate](native-service-retry.md)
subsequently exercises actual updated failure UI, old backend exit before explicit
Retry/new backend startup, original interrupted history with no sampled replay,
and a separate fresh submission/completion. Pending-work, direct installed old
cookie/handle negatives and the full recovery/quit/latency matrix remain open.

## Still pending

T036 remains **unchecked**. See [task/control procedures](../tasks-control.md),
[T034 resources](task-fixture-preparation.md), and
[T035 adapter-only regressions](task-forwarding.md).

- Complete native permission/gate/action matrix beyond the correlated real
  Reject/Allow/Stop subset, especially supported withdrawal and separate UI
  dismissal. Preserve the observed upstream presentation gap.
- Queue-only natural draining, pending-queue Stop/order races beyond the executed
  queue/steer/identical-receipt and now native queued-Stop subsets. Backend
  two-workspace/two-session scoped Stop now has backend and native subsets;
  full workspace/session/quit matrix and failed/interrupted tasks remain.
- Installed native R frontend comparison, raw stream/transport fidelity,
  hide/show/reconnect and no replay, receiving-boundary/Stop latency measurements.
- Controllable authorized interactive session for further native automation.
  The user explicitly authorized isolated computer-use testing; this does not
  authorize genuine-profile access or bypassing unavailable window observation.
  The intermittent composer defect remains open.

User-reported M01–M06 are separately recorded in [manual results](manual-results.md).
They do not complete this controlled matrix, T032/T033, or SC-003/SC-004.

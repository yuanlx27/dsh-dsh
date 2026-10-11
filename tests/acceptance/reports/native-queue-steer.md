# Native queue/steer continuation and pending-control gap

Continuation after `5708112`, authorized by the user's “continue”. Installed
release source remains `2bdf282`; same private task HOME, workspace A, custom
OpenAI Completions fixture model and loopback port 58426. No genuine profile,
credential settings or real project was inspected. No product/upstream code was
changed. Supplier PID **31653**, owned app/launcher/Node **31663/31677/31678**;
extension root @r11, native window ID **5035**.

## Pending withdrawal attempt: blocked, not passed

The previous **known regular public** approved marker was read, validated as
9 bytes/non-symlink, then removed from disposable A before the new request.
The original UI showed current Read Only. One typed withdrawal-trial request
produced call **115**, asked **116**, real Waiting for approval and no marker.

At this pending state the composer is replaced by the original approval panel;
there is no Stop generating control. Its two decision buttons are Reject and
Allow once. The examined header More actions offered Download session log,
not cancellation. Escape closed **that header menu only**, leaving approval
pending. The request was then explicitly **Rejected for cleanup**: decision
**117** rejected, result **118** errored, completed turn **123**, marker absent.
This is not a successful withdrawal, an aborted turn or a distinct dismiss case.

Pinned upstream source corroborates the examined UI:
- `dsh-client-ui-approval/lib/client.js`, ApprovalFlow: composer takeover,
  Reject/Allow once; scoped Escape answers rejected, not cancelled.
- `dsh-client-ui-conversation/lib/client.js`, installStopShortcut: explicitly
  excludes an active pending interaction and targets inside data-approval-key.

The real backend supports cancellation of a pending ask, already tested through
its service seam. A supported native UI withdrawal route is **not established**
by this run. Record this as an upstream presentation requirement gap, not a
transport corruption or justification for a second shell approval engine.
A fresh proposed Escape-on-approval trial below was not submitted; separate
native dismissal/withdrawal qualification remains open.

## Two identical queued requests, explicit bulk steering: passed subset

After actual idle settlement the private supplier was armed with a held stream.
One original Send started anchor turn **125**, step **127**, user message **128**.
While it remained running, two separately typed identical public follow-ups
were each submitted once through **Queue message**. Original UI showed
**2 queued messages**, and persisted inbox insertions **129/130** each contain
one identical input with **two distinct RPC receipts**. Nothing was deduplicated.

The empty editor offered **Cmd/Ctrl+Enter steers all queued messages**. Referenced
extension Cmd+Enter used that actual gesture. Original UI removed the queued
count and displayed the inputs; actual inbox events removed each next-turn
occurrence (**131/133**) and inserted it into next-step (**132/134**).

The supplier's private finish control then released the held model response;
it does not submit prompts, edit inboxes or execute tools. The current step
finished (**135/136**), next-step drained both inputs (**137**), and next step
started (**138**) with two matching user messages (**139/140**). Both delivered
RPC receipts match the original two queue receipts, once each. Assistant **141**,
step end **142**, completed turn **143**; original UI returned Send message and
showed both identical inputs. One active turn, two steps; steering here did not
mean a second user turn or a forced interruption of the held model response.
Denied marker stayed absent and the same owned trio remained alive.

## Live standalone backend comparison

`standalone-queue.test.ts` was first RED (scenario unsupported), then GREEN after
minimally extending the existing real-web-profile driver and closed-result
schema. The actual pinned upstream backend accepts two identical mode-queue
prompts with fresh request IDs while the loopback response is held. Inspection
proves both pending, then actual SessionController.updateQueue(kind steer) moves
each occurrence. Response release yields exactly one matching delivered message
per receipt, two identical payloads, one turn start/end, completed result, no tool
or approval calls, unchanged workspace, authenticated loopback-only traffic and
context disposal/child exit 0. This uses supported service calls, not browser RPC
or a fake queue implementation. It uses the official Messages adapter whereas
the installed native run uses custom OpenAI Completions; exact adapter parity,
queue-only natural draining, cancellation with pending queue and every ordering
race are not inferred. The full packaging suite passes **91 tests**.

## Later preparation failure and resource cleanup

After completed turn 143 a fresh supplier approval scenario was armed. An
editor/typeText action batch for a separate Escape trial was delivered but its
successor capture failed with **audio/video capture failure**. A refreshed exact
app root then had low pairing and yielded an abnormal recursively repeated
AXApplication tree rather than the expected window controls. Bounded System
Events observation of **only PID 31663** reported zero windows despite all three
captured processes remaining alive. The same-profile second invocation did not
restore a controllable window. Cause is **unresolved**: no attribution to window
manager, display lock, caret, app crash or successful lifetime retention.

No Send was issued for that proposed trial, no submitted-trial result is claimed,
and the confirmed queue log still ends at 143. A native Quit menu AX click was
attempted on the owned process, but no confirmation window could be observed.
It is not confirmed-quit evidence. Automation stopped rather than guessing at
controls. **Only owned app PID 31663 received TERM for fixture cleanup**; selected
PID checks then proved 31663/31677/31678 all absent. Afterwards private supplier
close proved 31653 absent. No name-based kill, privilege change, permission/lock
bypass or second app restart was used. Public fixture history and started-only
marker are retained; approved marker remains absent. An unsent draft after the
failed batch is not independently established and is not replayed.

[Closed native data](native-queue-steer-data.json) contains only sequence/category,
receipt-count and boolean outcomes, not content, keys, URLs with tokens, raw
records, Inspector dumps or screenshots. T036 remains unchecked: the pending
presentation gap and additional task/workspace/failure/metric matrix remain.
Neither this subset nor fixture TERM cleanup qualifies T037/T038/SC-007.

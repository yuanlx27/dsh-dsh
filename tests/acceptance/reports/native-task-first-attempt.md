# Native task first attempt — ungated, not an approval pass

The user reports selecting custom `desktop-fixture-model` with
`openai-completions`, rather than the standalone subset's official DeepSeek
Messages route. They report the proposed command/directory otherwise matched,
but no approval appeared. Record this adapter/composition difference; do not
claim exact adapter parity.

In the newly prepared private workspace A, the agent independently observed a
regular `approval-marker.txt` of 9 bytes. Thus this attempt performed the write
without an observed approval gate and is **not** a pending/deny/allow test pass.

Read-only observation of permission metadata from the **two A-only fixture
session files** used the installed `zstd` decoder and JSON field filtering.
Only event type, sequence, preset/mode/policy/outcome were emitted; no messages,
arguments, keys, record bodies or general logs were retained. No session or
credential store was modified and no second upstream service was launched.
The unrelated default-workspace session was not read.

For fixture Session `b83d4d12-63ea-4109-9850-36bddb97ad82`:

| Sequence | Event / value |
|---|---|
| 0 | permission/preset = workspace-write |
| 1 | sandbox/mode = workspace-write |
| 2 | approval/policy = ask |
| 4 | turn/start |
| 15 | tool/call |
| 17 | tool/result |
| 22 | turn/end |
| 24 | permission/preset = read-only |
| 25 | sandbox/mode = read-only |

No approval audit was observed for that tool. The read-only switch occurred
**after** execution. Under the tool's documented same-mode escalation behavior,
workspace-write requested while standing mode is workspace-write does not require
an approval ask. This is an invalid gated-test precondition, not evidence that the
native shim bypassed a read-only policy. The separate A Session had read-only
selected before its turn but no tool call; it cannot qualify the intended action.

Retry must stay in the intended current A Session, select read-only **before**
submission, explicitly rearm a fresh one-shot supplier response and remove only
the known disposable marker after confirming its public expected content. Do not
alter policy records or genuine profiles. Stop at the actual pending approval,
then independently verify the marker is absent before any decision. All native
approval, dismissal, action Stop, receiving-boundary and replay gates remain open.

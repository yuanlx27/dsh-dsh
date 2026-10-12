# Installed release queued Stop — observed computer-use retry

After the user reported running caffeinate and offered live screen monitoring,
only the existing isolated task HOME/workspace A and loopback fixture were
restarted. Candidate source remains `2bdf282`; no native, transport or upstream
code changed. No genuine profile, credential settings or actual project was
used. This successful observation does **not** establish why prior capture failed
or that caffeinate fixes every extension/native-dialog issue.

Owned supplier **16058**, app/launcher/Node **16083/16097/16098**. Extension root
@r13, native window **5924** was exactly paired, main and onscreen. Semantic
observation and checked AXPress worked on Open Harness and original upstream
controls. The editor was inspected before input; each public typed value was
observed before its one explicit Send/Queue click. Original current access mode
was **Read Only**, using the existing custom OpenAI fixture model.

## Actual native case

In existing prepared A session `d68aad58-7f53-4413-9efa-f6269d171664`:

- Private supplier armed **stream**, not a tool action, before launch. One typed
  anchor and Send started turn **146**, step **148**, matching user **150**.
  Original UI showed Stop generating and actual streamed public response.
- Two separately typed identical follow-ups were each submitted once through
  **Queue message**. Original UI showed **2 queued messages**; real durable
  next-turn insertions **152/153** each contain one matching input and **two
  distinct RPC receipts**. There was no input deduplication.
- Checked AXPress on original **Stop generating** returned **Send message** and
  **Stopped**. Assistant **154** preserves a nonempty prefix with interrupted
  true; step **155**, aborted end **156**. All three captured owned PIDs stayed
  alive, and no action marker was created.
- After Stop, original UI still showed **2 queued messages**. Persisted records
  had no removal/next-step move for them, **zero matching deliveries**, no second
  turn start, tool or approval in this interval. The observation continued until
  explicit cleanup, not an infinite delayed-race or reconnect proof.
- Clicking the original queued-count control exposed both exact queued entries
  and their original Edit/Remove/Steer buttons. Two scoped **Remove queued
  message** actions cleaned only these test inputs, with durable one-item
  next-turn removals **157/158**, no deliveries or extra turn. The queue controls
  then disappeared. No direct log editing, service-side queue injection or
  browser/Inspector program was used.

This correlates with the [real backend queue-Stop baseline](backend-queue-stop-and-session-scope.md):
current task aborts while unstarted queue survives until explicit operation.
Native uses custom OpenAI Completions; backend baseline uses official Messages
and different public literal text. Exact adapter/payload parity, queue-only
natural draining, restart, all order races and native two-workspace/session
isolation are not claimed. No receiving-boundary/Stop latency qualification.

## Exit and cleanup

Referenced extension Cmd-Q opened the real native confirmation. Explicit modal
@r14 observe still failed **Root is not owned by a running app** even though
normal controls now worked. This remains an extension/native-dialog limitation.

Bounded System Events Accessibility targeted **unix ID 16083 only**, read the
actual Stay/Stop and Quit button names and clicked **Stop and Quit**. Selected
PID checks proved **16083/16097/16098** all absent. Only then was positively owned
supplier **16058** closed through private control and its exit verified. No TERM,
process-name kill, privilege/lock bypass or user-caffeinate manipulation occurred.
This is one idle confirmed quit after explicit queue removal, not the multi-session
active/queued/approval quit matrix. App and supplier are now stopped; private
public history and started-only marker are retained for review.

[Closed data](native-queued-stop-data.json) retains only identities, sequences,
counts, flags and selected UI state IDs. No bodies, actual keys, cookies, raw
records, token URLs, Inspector dumps or screenshots were retained as evidence.
Source policy and strict tooling/browser checks pass. The existing **93** tests
remain the backend/resource regression baseline, not 93 native acceptance cases.
T036/full metrics/native pending-withdrawal route/platform/caret gates remain open.

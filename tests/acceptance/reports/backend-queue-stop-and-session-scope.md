# Actual upstream backend: queued Stop and session scope

Continuation of [queue/steer](native-queue-steer.md), using the existing pinned
web-profile child driver, private HOME and separate disposable standalone A/B
workspaces. No graphical App, real provider, inherited credentials, actual
project or fake queue implementation is involved in these backend cases.

Both new tests were first RED because their scenarios were unsupported, then
GREEN after minimal extensions of the existing driver and closed result schema.
Published upstream Inbox/SessionController declarations are used directly;
strict types are checked, not silenced. All upstream stdout/ephemeral bootstrap
URLs remain discarded; only booleans/counts and fixed failure-stage categories
leave the child. Existing lifecycle/action regressions still pass.

## Queue Stop

`standalone-queue-stop.test.ts`:

1. Real official Messages request admitted and running against held loopback SSE.
2. Two separately accepted, identical public inputs use fresh request IDs and
   real mode-queue admission. Both are inspected as durable pending occurrences.
3. SessionController.cancel aborts the active turn, not the whole web service.
4. After idle settlement and a **200 ms bounded late observation**, both exact
   payload/receipt occurrences remain in the live Agent inbox's nextTurn, with
   zero nextStep items and zero deliveries of those queued requests.
5. Only one turn start/end and one original matching user message; end aborted,
   nonempty interrupted assistant prefix preserved, still idle at observation.
6. No tool/approval or filesystem effects; authenticated loopback-only traffic,
   context disposal, child exit 0 and parent-side unchanged A/B README checks.

This proves the sampled backend Stop semantics: pending work survives for a
later explicit operation, rather than being discarded or immediately resumed.
It does not prove every delayed race, restart/reconnect behavior, receiving-
boundary latency or installed-native queue Stop. The existing queue/steer case
separately tests explicit delivery; no delayed future execution is inferred here.

## Two sessions / two workspaces

`standalone-multi-session.test.ts`:

1. Two real sessions are created for the prepared A/B workspaces in one actual
   upstream web profile. Each selects the official model and submits one fresh
   identified public request; both actual provider operations and running states
   are observed before cancellation.
2. Cancel targets A only. A reaches one aborted turn with preserved nonempty
   interrupted prefix and no tool calls.
3. After **200 ms**, B remains running. Only then does host fixture response
   release let B finish one completed turn with exactly one matching user receipt
   and no tools/approvals. The fixture release is not a task submission.
4. The same service subsequently disposes normally; child exits 0. Independent
   parent filesystem checks confirm both workspaces contain unchanged README
   only. All provider calls use authenticated synthetic loopback credentials;
   rejected non-loopback destinations/redirects remain guarded.

This is real backend isolation between sessions/workspaces, not native workspace
switching, a multi-session Stay/quit matrix or app history/reconnect qualification.
The baseline uses official Messages; native custom OpenAI adapter parity remains
separate. No rollback, new shell task state or global process-cleanup claim.

## Computer-use retry: not usable / not executed task comparison

The user explicitly requested using computer use if available. Existing desktop
roots were absent. Only the known released loopback port/private supplier and
**same isolated HOME/A launch command** were reused; no genuine profile settings
were accessed. Supplier **14111** was armed stream before launch. find_roots found
candidate App **14130**, window **5865**, root @r12 with low pairing; semantic
observe_ui then timed out after **33000 ms**. No Open Harness, Send, approval
or task-control action was delivered by the agent in this attempt.

On the next cleanup observation App 14130 was already absent; no process with
the exact candidate Contents/MacOS executable prefix remained. The cause and
launcher/Node identities were **not captured**, so this is neither a native
startup pass/failure diagnosis nor complete captured-trio cleanup evidence.
No stale PID was signalled or process-name kill attempted. Only the positively
owned supplier 14111 was closed by private control and its exit verified.
Its closed category reports **zero provider operations**, consistent with no
native task comparison having run; this is not proof of startup readiness.

Do not bypass locks/permissions or repeat blind input after this timeout. Native
queue Stop and remaining graphical matrix remain pending until controllable
observation returns. The prepared public workspace/history are retained, and
future fixture startup is explicit; no submissions are replayed.

Validation: **93 packaging tests**, source policy, strict tooling and browser
projects pass. No product, native transport or downloaded upstream code changed.
T036 remains unchecked; T037–T039, native pending-withdrawal route, full metrics,
platform/distribution and intermittent composer gates are not waived.

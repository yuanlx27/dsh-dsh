# T034 — controlled task, approval and Stop fixtures

Preparation only. Use this protocol for later T036/T038/T039 standalone-R versus
installed-native execution. US1's implemented shell/transport is available; its
pending full acceptance is not waived. No native keyboard automation runs during
user-led manual testing, and no user credential/provider is used by these resources.

## Reuse and provenance

- R: dsh `0.2.0-rc.2`, revision `639ed015397290b3745d163aafe02ffee4aa3f84`.
- `tests/fixtures/task-workspaces.ts`: owner-only, disposable standalone/product
  roots, each with separate A/B directories and seeded README; no marker exists
  initially. Never point it at live projects or migrate records between roots.
- Existing `controlledModel()` now optionally emits one advertised tool call on
  its first **streaming** request or holds a text stream after a visible delta.
  Later replies are ordinary text. This is a loopback supplier resource, not a
  second engine: real R owns task state, policy, tools, queues and persistence.
- Public credentials `desktop-fixture-first` / `desktop-fixture-replaced` identify
  fixtures only. Provider events retain operation/credential categories, never
  headers, request bodies, system prompts, arguments or real keys. Discovery is
  available through the existing original provider setup flow.
- Configure the original R connection through supported settings/configuration.
  Do not inject credentials into conversations or add shell online validation.
  Record the actually selected adapter/model, Base URL category, workspace and
  effective policy. Discovery/title requests must not be mistaken for task calls.

## Controlled supplier resources

`controlledModel({ streamGate })` takes a host-owned Promise: first text delta is
sent, terminal markers wait for release or peer cancellation. Use a fresh gate per
trial. Closing the fixture destroys only its own HTTP connections and releases
pending responses; it does not stop dsh. Do not use uncontrolled delay timers as
proof of visible UI acknowledgement.

`controlledModel({ tool: { name: "bash", input: ... } })` advertises one tool_use /
OpenAI tool_calls response. The tool must appear in R's outgoing request catalog;
otherwise the resource returns 422 rather than silently fabricating success.
Nonstreaming requests receive text, preserving normal title-generation behavior.
A new resource per independent case prevents previous one-shot state contaminating
later cases. Absence of the expected streaming adapter/catalog is Blocked, not Pass.

Canonical upstream schema is in `dsh-tool-bash/lib/index.js` around lines 488–525:
`command` and `description` required; `timeoutMs`, `workdir`, optional background
and composed escalation fields. Use the case's actual absolute A directory as
workdir, not an invented tool-wire format. These bounded task data are not shell
startup scripts or native transport business logic.

| Resource | bash input / observable effect |
| --- | --- |
| Read-only summary | `cat README.md`; description `Read the disposable workspace overview`; no file changes |
| Approval | `printf '%s\n' approved > approval-marker.txt`; description `Write the disposable approval marker`; request `sandbox_permissions: workspace-write` with a one-sentence justification; only this target may change |
| Dispatch counter | `printf '%s\n' explicit >> actions.log`; description `Append one disposable dispatch marker`; count actual lines, not model claims |
| In-flight Stop | `printf '%s\n' started > stop-marker.txt; sleep 20; printf '%s\n' finished >> stop-marker.txt`; description `Observe a bounded stoppable fixture action`; `timeoutMs: 30000`, `run_in_background: false`; record promotion/background status if the executor cap differs |

Review the complete actual proposed action and target. A workspace is context,
not a promise of a security sandbox. No network, credentials, process-name kill,
real-file edits, permanent detached jobs or rollback expectation is part of a case.

## Upstream-owned permission configuration

Inspect effective policy rather than assuming labels. R permission preset types
pair sandbox (`read-only`, `workspace-write`, `danger-full-access`) with approval
(`ask`, `never`). **never means reject approval asks without prompting, not approve
every action**; an action allowed by sandbox policy may never ask at all.
`custom` is a derived display value, not a selectable preset. Defaults may vary by
composition. Do not modify the product overlay or add a shell permission policy.

For the gated-write case use an available supported `read-only` + `ask` setting
and an explicitly composed workspace-write escalation. Verify actual R approval
request before testing; if unavailable, record the composition gap. A workspace-
write policy might perform the ordinary write without prompting and cannot serve
as a pending/denied test. Escalation grants are one-shot, not standing policy edits.

## State and behavior matrix

| Case | Procedure | Required evidence |
| --- | --- | --- |
| C01 idle/completed | Open fresh A; submit read-only request and allow supplier completion | Request receipt, progressive response, actions/outcomes and explicit distinguishable idle/running/completed states; no claim about model answer quality |
| C02 running | Hold text gate after first delta | Visible ongoing state and text update while supplier remains held, no premature completion |
| C03 awaiting approval | Advertised gated write; leave decision unresolved | Effective policy/action/target and allow/deny controls; marker absent before decision |
| C04 denied/dismissed | Separate fresh requests: deny one, dismiss another | Actual marker absent, never automatically allowed; record follow-up requests/counters, not just assistant narrative |
| C05 allowed | Separate fresh request; explicitly allow reviewed write | Exactly one intended marker effect, accurate result; standing policy unchanged |
| C06 stopping | Stop while C02 gate held | Visible stopping acknowledgement and final outcome; supplier cancellation category; no later new actions, dsh still alive |
| C07 in-flight Stop | Permit bounded action; Stop after started marker | Record started/finished lines and actual job state/exit; already-started action may finish, no claimed rollback or false success |
| C08 failed | Public rejected key; separately close controlled supplier mid-stream | Corrective upstream outcome, settings/history accessible, failed state independent of retained service readiness |
| C09 interrupted | Schedule owner-proven service exit while C02/C03 active | Interrupted state, old authority invalidation, explicit recovery only; no submission/command replay or approval resolution |
| C10 duplicates/rapid | Explicitly submit identical text twice, then rapid distinct requests | Distinct upstream request identities, two explicit counter effects where R admits both; no text deduplication or desktop submission lock |
| C11 queued/steered | Submit follow-up while busy using R's actual offered gestures | Record actual queue/steer semantics and receipt order, including unavailable mode; do not invent a queue policy |
| C12 hide/show | Hide/reopen C02 and C03 with draft, selection and scroll | Same native document/service/transfer, session state and draft; no Stop/decision/resubmit |
| C13 reconnect/recreate | Controlled carrier disconnect, then document recreation separately | R chooses recovery; one dispatch per explicit upstream request, no adapter autonomous replay; persisted recovery belongs to R |
| C14 workspace switch | Separate sessions A/B; retain C02/C03 in one while switching | Same associations and active work; no shell workspace confirmation, permission scan, task/approval cancellation or replay |
| C15 full quit | Open/hidden active and approval cases; repeated quit, Stay then confirm | <=1 unresolved ownerless dialog; Stay preserves all work, confirmation cleans owned service across sessions; Stop is not full quit |

Run each case against standalone pinned R and the installed native candidate with
separate resources. Differences in underlying policy/tool availability are
upstream/composition gaps unless transport-only evidence proves a shim defect.
Every FR-007 state must be observed or explicitly pending; a spinner/model narrative
alone is not a state or filesystem assertion.

## Sanitized recording and qualification

Record candidate/manifest identities, R composition/policy, fixture IDs, captured
owned PIDs/document/stream identity, state transitions, file existence/digests/line
counts, explicit dispatch counters and categorical supplier events. Do not retain
raw credentials, prompts/replies, model bodies, tokens, general Inspector dumps or
real project data. Any required opaque R envelope recording uses public fixture
data with provenance; synthetic envelopes are not mislabeled captured R requests.

T035 adds forwarding regression; T036 executes parity; T038 executes recovery;
T039 measures 20 native runs. For SC-003 timestamp native receive-to-visible updates
and Stop receipt-to-visible acknowledgement; provider generation and pre-boundary
dsh processing are excluded. Every update in >=19/20 runs <=1s and every Stop <=1s;
all gated/denied writes must remain unperformed for SC-004. Unit fixture tests or
manual aggregate passes do not supply those metrics.

Prepared resources are unit-tested, not executed comparative/native task evidence.
Reports go to `reports/us2.md`, `us2-recovery.md`, `task-performance.md`. Keep
unavailable cases pending/release-blocking rather than building a desktop engine.

# Installed native task decisions — partial execution

Candidate/profile/supplier provenance: [native handoff](native-task-handoff.md).
The user operates the original R frontend in the installed release candidate;
there is no fixture approval answerer, Inspector injection or keyboard/window
automation. The user selected custom `desktop-fixture-model` with
`openai-completions`; this differs from the standalone official Messages adapter
and is not exact adapter parity. The earlier [ungated attempt](native-task-first-attempt.md)
is excluded from approval results.

## Pending then explicit Reject — observed

The user confirmed current read-only permission before retry. The agent verified
A-only persisted permission metadata, checked the prior disposable marker's
public `approved` content and regular-file status, removed only that marker, and
explicitly rearmed the controlled supplier after the completed preceding work.
The user submitted once and reported actual approval with the expected command.
While the user left that approval undecided, independent directory observation
showed README only: no `approval-marker.txt`.

The user then clicked **Reject**, reported the tool Error card and fixed supplier
follow-up text, and reported no new approval. Independent directory observation
again showed README only. Fixed model prose is not treated as outcome evidence.

The current A fixture Session `6eb27260-082d-4fae-a9bf-d6eeeddc6a53` contains:

| Sequence | Observed metadata |
|---|---|
| 4 | permission/preset = read-only |
| 5 | sandbox/mode = read-only; seeded approval policy remains ask |
| 23 | turn/start |
| 28 | tool/call |
| 29 | approval/asked; initially no decided event while UI was pending |
| 30 | approval/decided, outcome rejected, after the reported Reject |
| 31 | tool/result, isError true |
| 36 | turn/end, reason completed |

No additional approval audit or policy switch was observed in this trial.
The turn completes after the denied tool and mock follow-up; this does not mean
the denied action succeeded. User UI observations correlate with permission,
decision/error metadata and independent before/after filesystem absence.
This is a concrete pending/denial result, not merely a model narrative.

The observer decodes only the **new A fixture sessions**, using installed zstd
and JSON field selection; it emits event type/sequence and preset/mode/policy/
outcome/isError/end-reason only. No messages, tool arguments, credentials or raw
records are retained. It neither edits upstream records nor starts another
upstream service. The unrelated default-workspace Session is not inspected.

## Later no-tool report — fixture limitations, not a policy-bypass finding

The user reports that Read Only appears ineffective because subsequent requests
produce no operation or approval. A-only metadata still records read-only/ask,
with no switch back to workspace-write. The workspace remains README-only.
Session 6eb's next tool sequence is call 43, ask 44, rejected decision 45, errored
result 46, completed end 51; later turns and two other A fixture Sessions have
completed turns without tool calls. Whether that second approval disappeared
during hide/show, was explicitly rejected, or followed other gestures is **not
confirmed**. Do not qualify pending hide/show from these rows.

The supplier is intentionally one-shot per explicit host arming: later streaming
operations receive fixed text until the host rearms. Absence of a tool request
cannot establish that read-only enforcement failed, nor that a natural model
understood the user's prompt. Human retries/new Sessions do not rearm it.

Source review also found a real **fixture** defect: rearming reused constant
provider tool/response IDs. It is an identity-collision risk for repeated trials
in one Session, not a proven cause of the user's UI observation. The fixture now
uses standard-library UUIDs, one tool ID per actual tool reply and one response
ID shared by that response's chunks. Cross-arming Messages/OpenAI regressions
failed before the fix and pass afterward; all 89 packaging/source/strict checks
pass. No product transport, permission or downloaded upstream source is changed.

The live supplier PID 9171 still runs the previously loaded source. It has **not**
been restarted or patched by this source commit; native retry must wait for a
scheduled fixture restart/reconfiguration. Preserve the concrete first Reject
observations, but do not treat repeated-ID trials as full provider identity
qualification. User clarification of the hide/show/decision sequence is pending.

## Remaining qualification

Allow, pending hide/show, UI dismissal distinct from Reject, withdrawal, actual
foreground Stop, queue/steer/duplicates and multiple sessions remain pending.
No native latency, process/document identity or complete race matrix is inferred.
T036, T032/T033 and quantitative/native/platform gates remain unchecked.

# Desktop Requirements Quality Review

**Date**: 2026-10-06
**Checklist**: [checklists/desktop.md](checklists/desktop.md)
**Authority**: The reviewer approved CHK001 and the official credential behavior for CHK002, then authorized the agent to approve adequately specified criteria and adopt compatible official behavior for remaining gaps. Unresolved product choices or insufficient evidence remain unchecked.
**Result**: 40 satisfied criteria; no pending reviewer decisions.

This is a requirements-quality review, not implementation verification. No product runtime, installed-app comparison, usability study, or performance qualification was performed. Checkbox approval does not establish upstream runtime conformance or official parity.

## Evidence baseline

Official desktop reference O is `5badb15009ae1756c3afe0ae0cef1faafc290ccc`.
Bundled runtime reference R remains `639ed015397290b3745d163aafe02ffee4aa3f84`.
Source was inspected in a temporary upstream checkout; neither reference was changed.
The official desktop uses the shared upstream model/workspace/session services, so shared-service evidence is relevant to desktop behavior without implying that this shell owns those capabilities.

| Evidence | Immutable official source | Findings relevant to this review |
|----------|---------------------------|----------------------------------|
| E1 Credentials | [service contract](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/credentials/credentials/src/index.ts), [API-key resolver](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/llm/llm-deepseek-api-key/src/index.ts), [model adapter](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/llm/llm-deepseek/src/adapter.ts), [settings subscriptions](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/client/ui-settings-models/src/client/index.ts) | Credentials resolve per operation; missing API keys produce an error; read-only shadowing refuses writes/removal; settings refresh from owner events. Core resolver/adapter and settings-subscription files are unchanged between O and R. The credential contract's changed listener-error handling does not alter its per-operation resolution rule. |
| E2 Session identity | [rows](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/client/ui-workspace/src/client/rows/Rows.tsx), [workspace browser](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/client/ui-workspace/src/client/rows/WorkspaceBrowser.tsx) | Session identities remain distinct within workspace groups; blank/untitled rows use fallback labels. Both files are unchanged between O and R. No new shell list or unique-title constraint is adopted. |
| E3 Versioned notice | [welcome store](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/client/ui-settings-models/src/client/welcome-store.ts) | Durable acknowledgement is compared with the notice-copy version; unavailable/malformed acknowledgement is not success. The file is unchanged between O and R. Adapt persistence ownership to the existing shell-owned safetyNoticeRevision; do not introduce a second upstream acknowledgement store. |
| E4 Public assets | [frontend static service](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/host/frontend-static/src/index.ts) | Index responses require authentication; non-index static assets remain public. The file is unchanged between O and R. The product's public exception excludes sensitive/user-generated content. |
| E5 Persistence | [durability and reading contract](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/session/session-persistence-jsonl/README.md#durability-and-crash-semantics) | Recoverable incomplete tails are distinguished from corruption of committed records; unsupported/unreadable headers may be omitted from discovery and direct access can fail. Persistence source is unchanged between O and R; README differences do not change the inspected durability/recovery rules. |
| E6 Lifecycle and recovery | [desktop README](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/README.md), [quit decision](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop/src/quit-confirmation.ts), [quit inspection](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/apps/desktop-host/src/quit-inspection.ts) | Official close retains the document, including drafts/scroll state; quit inspection covers application-wide work and joins repeated requests. Official fatal reporting may expose raw diagnostic details, and official startup has no timeout heuristic. Existing product adaptations and safety constraints take precedence. |
| E7 Workspace availability | [workspace type](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/workspace/workspace/src/types.ts), [status](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/workspace/workspace/src/entity.ts) | The canonical path is never rewritten. Status distinguishes a directory detectable by stat from missing/failed-stat cases; this does not establish all read/write requirements or the requested replacement workflow. |
| E8 Submissions | [input machine](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/client/ui-conversation/src/client/input/machine.ts), [composer facade](https://github.com/deepseek-ai/deepseek-harness/blob/5badb15009ae1756c3afe0ae0cef1faafc290ccc/packages/client/ui-conversation/src/client/input/facade.ts) | Command submissions can freeze admission, while ordinary sends use detached attempts and commit their drafts; busy input can be queued or steered. This is not a blanket prohibition on repeated identical text or all input during task startup. Do not substitute it for a defined product duplicate-submission policy. |

## Adopted requirement clarifications

- **CHK002 / FR-003**: Credential changes apply to subsequent model calls, including calls within the current task. They do not cancel work already started, undo changes, or revoke a remote key. Missing credentials block dependent future calls/new work; refused removal is not success. Availability presentation follows upstream, independently of the running task state. See E1 and the updated data-model credential section.
- **CHK004 / FR-011**: Preserve upstream workspace grouping, titles/fallback labels, and separate session identities even when titles coincide. See E2.
- **CHK009 / FR-015**: Acknowledge the current notice revision in local application data. An unchanged notice does not require renewed acknowledgement merely because credentials, workspace, or application version changes. Changed notice copy or missing/invalid acknowledgement does require it; failed persistence is not success. See E3.
- **CHK010 / FR-025**: Define authorized content using the already-planned native main-frame/origin/window/runtime boundary. Public assets mean non-sensitive non-index UI assets, not authenticated index/boot content or user/configuration data. See E4 and the existing runtime transport contract.
- **CHK030 / FR-012**: Adopt upstream recoverable-tail/corruption/refusal semantics. Preservation does not promise reconstruction of unsaved, externally deleted, or corrupt committed data; unavailable records must not be presented as restored. Discovery omissions remain an upstream limitation. See E5. This also makes CHK039's durable-history boundary explicit.

## Criterion decisions

| Item | Decision | Requirements-quality basis |
|------|----------|----------------------------|
| CHK001 | Satisfied | Reviewer explicitly approved; FR-001–FR-005, FR-015 and US1 cover first/return use. |
| CHK002 | Satisfied after clarification | FR-003 and data model now document E1; reviewer explicitly approved adopting this behavior. |
| CHK003 | Satisfied | FR-006–FR-009 define the end-to-end task capability; FR-018 assigns ownership to dsh. |
| CHK004 | Satisfied after clarification | FR-011 now documents E2's grouping/title/identity boundary. |
| CHK005 | Satisfied | FR-010/FR-019, edge cases and lifecycle contract cover background tasks, pending approvals and no-window quit. |
| CHK006 | Satisfied after reviewer decision | FR-002 now distinguishes service availability, safety acknowledgement, incomplete setup, task readiness and independent task state; configured credentials are not proof of remote validity. |
| CHK007 | Satisfied after reviewer decision | FR-005/FR-018 now assign validity and action permissions to dsh, permit read-only analysis without a blanket write requirement, and keep history access independent; no shell preflight or parallel admission policy is authorized. |
| CHK008 | Satisfied | FR-009 separates acknowledgement and in-flight actions; SC-003 sets a one-second acknowledgement threshold. |
| CHK009 | Satisfied after clarification | FR-015 now defines the E3-inspired notice revision and persistence lifecycle. |
| CHK010 | Satisfied after clarification | FR-025 plus runtime contract define authorized senders and E4-inspired public-resource scope. |
| CHK011 | Satisfied | FR-009/FR-019 consistently separate task Stop, window close and full quit. |
| CHK012 | Satisfied | Plan A3 and lifecycle contract explicitly justify the conservative additional idle prompt. |
| CHK013 | Satisfied | FR-018, plan Summary and data-model invariants prohibit duplicated task/configuration/history ownership. |
| CHK014 | Satisfied | FR-004 and Assumptions distinguish concealment from encryption, sandboxing and same-user isolation. |
| CHK015 | Satisfied | Platform narrowing is an authorized planning decision; plan fixes minimum OS/architecture and preserves FR-020/FR-021. |
| CHK016 | Satisfied after reviewer decision | SC-001/SC-006 define receipt, outcome visibility, original-session/workspace retrieval and dsh-grounded state identification; uniform instructions and assistance scoring exclude model-answer quality from the desktop guarantee. |
| CHK017 | Satisfied after reviewer decision | SC-002/SC-005 and quickstart define process-start/task-ready and session-request/first-readable-history boundaries, fixture prerequisites and named idle environment without adding shell history logic. |
| CHK018 | Satisfied after reviewer decision | SC-003, quickstart and plan now score task runs: >=19/20 runs with every recorded relevant update timely from the desktop receiving boundary; all Stop acknowledgements remain subject to one second. |
| CHK019 | Satisfied | SC-004–SC-005, SC-007, SC-009–SC-010 and quickstart define controlled fixtures/matrices; 100% refers to those scenarios, not an unlimited proof. |
| CHK020 | Satisfied | FR-022–FR-024, SC-008 and plan separate complete decisions, source inspection and pending comparisons. |
| CHK021 | Satisfied | US1.4 and FR-011 cover returning; amended FR-010 and US2.6–7 distinguish dsh-owned workspace switching from native Stay/quit behavior. |
| CHK022 | Satisfied | FR-003/FR-014 and Edge Cases specify corrective explanations and explicit next actions. |
| CHK023 | Satisfied | FR-012/FR-014 and contracts define interruption, preserved committed history and explicit no-replay recovery. |
| CHK024 | Satisfied after reviewer scope change | FR-013/US3.4 and related documents remove the replacement-directory promise, adopt dsh history/continuation behavior, and explicitly exclude shell migration or reassociation of any sessions. |
| CHK025 | Satisfied | FR-019/FR-023 and lifecycle contract define single conservative quit decision without an owner window. |
| CHK026 | Satisfied after reviewer scope change | FR-006 and Edge Cases delegate repeated-input admission to dsh, exclude shell text deduplication/startup locks, and require no shell-induced duplication or replay through forwarding/reconnect/reopen. |
| CHK027 | Satisfied | FR-008/FR-012/FR-019 and Edge Cases consistently forbid implicit approval through dismissal/close/recovery. |
| CHK028 | Satisfied after reviewer scope change | FR-010/US2.6–7 separate dsh-owned workspace switching from native whole-service quit; warning scope covers all sessions/workspaces and no shell switch-inspection or bulk-stop policy is added. |
| CHK029 | Satisfied | Runtime contract specifies 15-second startup and seven-second shell shutdown escalation boundaries, actual-exit requirement and visible cleanup failure; FR-009 disclaims undo. E6's differing official timeout behavior is already an intentional adaptation. |
| CHK030 | Satisfied after clarification | FR-012 now bounds damaged/missing record guarantees and adopts E5 without shell repair. |
| CHK031 | Satisfied | FR-025/SC-010 and runtime contract cover sender/client classes, confidentiality and explicit attack exclusions. |
| CHK032 | Satisfied after reviewer scope change | FR-004 separates upstream credential/output handling from shell-generated non-disclosure, excludes blanket redaction/history rewriting, and preserves FR-025 native service authentication and FR-017 destination limits. |
| CHK033 | Satisfied after reviewer scope decision | NFR-001 defines shell keyboard/focus/naming/status/VoiceOver requirements, preserves upstream capability without UI remediation, and excludes unassessed product-wide WCAG claims. |
| CHK034 | Satisfied | SC-002–SC-005 and Assumptions explicitly bound desktop timings and exclude model-generation latency. |
| CHK035 | Satisfied | FR-016/FR-018, Assumptions and plan identify upstream capabilities as dependencies requiring qualification. |
| CHK036 | Satisfied | FR-020, Assumptions and plan distinguish bundled runtime, network/model dependencies and project-command tools. |
| CHK037 | Satisfied | FR-022–FR-024 and plan distinguish immutable O/R and require affected-decision/evidence review on change. |
| CHK038 | Satisfied after reviewer design change | FR-019/US1.7 and plan A2 adopt normal hide/retain/show/focus, preserve document context and transport, and exclude unsaved-context guarantees after actual destruction/page failure/crash/full quit. |
| CHK039 | Satisfied | FR-012's clarification explicitly distinguishes committed records from unsaved progress and unrepairable data; SC-005 uses previously persisted records. |
| CHK040 | Satisfied | FR-022, US4.2 and Assumptions give approved constraints precedence and require justified differences without expanding scope. |

## Resolved reviewer decisions

### CHK006 — Application readiness

The reviewer approved distinguishing service availability from task readiness.
FR-002 now defines starting/failure, notice acknowledgement, incomplete setup
with settings/history access, and task readiness requiring service/connection,
notice, workspace and configured non-rejected credentials. Remote rejection
requires correction; no separate desktop-owned online validation is added.
Configured credentials are not proof of remote validity or continuous model
availability. Task states remain independent and loss of prerequisites does
not itself cancel active work. This adopts E6's configuration-first approach
without importing its weaker task-readiness interpretation.

### CHK007 — Valid workspace

The reviewer approved read-only analysis and clarified that dsh-owned decisions
are not this shell's responsibility. FR-005/FR-018 now leave directory validity,
operation permissions and associated admission decisions to dsh. Directory access
needed by the requested work is distinguished from blanket write permission;
individual failures use upstream handling and history access is independent.
No shell permission scan, duplicate validation or custom admission policy is
introduced. This records a product scope/ownership decision, not proof that the
bundled release satisfies every scenario. Apply the same ownership boundary in
subsequent reviews without automatically approving conflicting requirements.

### CHK016 — Usability outcomes

The reviewer approved the desktop-usability rubric in SC-001/SC-006. First-task
success requires dsh receipt rather than answer completion. Summary-flow success
requires finding dsh's reported outcome, not grading model answer quality.
Participants must reopen the original session, identify its workspace and
identify awaiting-approval/completed states against dsh's reported state.
Uniform briefs, credentials and project locations are allowed; procedural hints
or intervention disqualify independent success without removing the participant
from the denominator. SC-006's 90% criterion requires all three activities.
Quickstart uses the same rubric. No usability study was performed in this review.

### CHK017 — Timing boundaries

The reviewer approved startup timing from installed-process start to visible
workspace with service connection and FR-002 task readiness. Saved configuration,
current notice acknowledgement, an accessible workspace and a valid configured
credential are fixture prerequisites; installation, first-use setup, user input
and model response generation are excluded. History timing begins with receipt
of the target session-open request in an already-connected application and ends
with target identification, readable first-screen history and browsing available,
not simultaneous display of all 1,000 messages. The named idle test environment
applies to both measurements. SC-002/SC-005 and quickstart agree; no shell cache or
reader is required and no performance run was performed in this review.

### CHK018 — Update-latency denominator

The reviewer approved per-task-run scoring: at least 19 of 20 controlled runs,
each with observable response and action-status updates, must have every recorded
relevant update visible within one second. Any recorded late update fails that
run. Timing begins when the update from dsh reaches the desktop-owned receiving
boundary; earlier dsh processing and model generation are excluded. Every Stop
request must receive visible acknowledgement within one second of application
receipt, without a 95% exemption. Spec, quickstart and plan agree; no additional
shell task logic is authorized and no latency run was performed in this review.

### CHK024 — Replacement directory and other sessions

The reviewer chose to adopt dsh's existing behavior rather than require a new
replacement-directory workflow. FR-013, US3.4, Edge Cases, workspace/session
entities, lifecycle contract and quickstart now remove that promise. Unavailable-
directory history, errors and continuation remain upstream-owned. New-directory
selection uses dsh's existing workspace flow without promising transfer of the
old conversation; restoration of the original directory also follows dsh. The
shell neither changes the selected session's association nor other sessions'
associations, and adds no migration or recovery store. This scope amendment
resolves the conflict with E7's immutable path; it is not runtime qualification.

### CHK026 — Duplicate submission identity

The reviewer approved retaining dsh acceptance, rejection, queueing, steering
and repeated-input semantics. The former guarantee that two rapid user attempts
produce one request is removed. Identical text is not shell-deduplicated and no
shell task-start lock or additional admission policy is introduced. The shell
must not itself repeat/replay a submission through forwarding, reconnect or
window reopening. FR-006, Edge Cases, data model, lifecycle contract and quickstart
now distinguish intentional user attempts from shell-induced duplication. E8
informs this ownership boundary; no runtime duplication test was performed.

### CHK028 — Concurrent sessions and workspace changes

The reviewer approved separating workspace switching from full quit. Switching,
any prompts and task continuation/stopping follow dsh; no shell session inspection,
extra switch confirmation or bulk-stop policy is added. The native quit warning
covers all sessions/workspaces served by the owned dsh instance. Stay leaves all
work untouched by the shell; Stop and Quit stops the owned service before quit
completes without implying rollback. FR-010, US2.6–7, lifecycle contract, data model
and quickstart reflect this distinction. Whole-service native confirmation remains
the existing intentional E6 adaptation; no concurrent-session runtime test was run.

### CHK032 — Secret-bearing upstream/tool output

The reviewer approved narrowing the former absolute conversation-history promise.
Model credential entry/storage/settings concealment and arbitrary upstream output
remain dsh-owned. The shell adds no credential store, injects no saved credential
into conversations or shell diagnostics, and uses non-secret startup/error content
instead of raw potentially secret-bearing output. No universal tool/model/dsh
output redaction or history rewriting is promised. FR-025 native service token/
cookie confidentiality and FR-017 destination/telemetry/sync restrictions remain
binding. Spec, data model, lifecycle contract, research and quickstart reflect the
boundary; this explicit scope amendment is not evidence of a working output filter.

### CHK033 — Accessibility scope

The reviewer approved NFR-001 for shell-owned startup, failure/retry, safety-notice
and quit-confirmation surfaces. Requirements cover keyboard-only operation, visible
custom-control focus, dialog entry/restoration, meaningful accessible names,
non-color-only status/errors and VoiceOver identification of controls, notices
and key state changes using native/standard semantics. dsh surfaces retain upstream
accessibility; embedding must not disable it. No separate upstream UI remediation
layer or unassessed product-wide WCAG level is promised. Spec, lifecycle contract
and quickstart agree; no keyboard or VoiceOver runtime qualification was performed.

### CHK038 — Transient document state

The reviewer explicitly approved changing the former destroy/recreate design to
E6's normal-close hide/retain and reopen show/focus. FR-019/US1.7 and SC-007 now
preserve the same primary document with no closure-induced draft, conversation-
selection or scroll reset. Hiding does not detach that document's authorized
transport or invalidate its generation. Actual destruction, document replacement/
navigation or page failure invalidates affected handles; runtime failure/quit
revokes all authentication/handles. Recreated content must satisfy fresh sender
validation. No new draft/page-state persistence store is added and unsaved context
is not guaranteed after page failure, crash or full quit. Spec, plan A2, research,
data model, lifecycle/runtime contracts and quickstart agree; runtime comparison
and lifecycle qualification remain pending.

## Pending reviewer decisions

None. All 40 requirements-quality criteria are approved; this does not mark any
implementation or runtime-evidence task complete.

## Next action

Keep the linked requirements and this review basis synchronized when decisions change; reassess affected criteria rather than treating approval as permanent. Runtime conformance and comparative evidence remain pending implementation/release work, regardless of these review markers.

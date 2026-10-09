# Manual desktop acceptance — short first pass

Use this alongside [startup-security.md](startup-security.md), not instead of the
native/security contracts. No outcome is pre-filled as passing. Start with M01–M03
and send those results before spending time on the rest.

## Before starting

- Use the **current verified release candidate and launch command supplied by the
  maintainer**, not an older app or the debug carrier-only page. That command uses
  an isolated disposable HOME. Relaunch using the same command/profile; Finder
  launch or copying to Applications does not preserve that environment.
- Quit other desktop test candidates first. Keep the launching terminal open.
  OmniWM may remain enabled; record whether it was enabled if behavior differs.
  Never disable Gatekeeper or grant broad permissions to make a test pass.
- Use the supplied disposable workspaces A and B only, not real projects. Review
  actual proposed tools/commands/targets: workspace context is **not a sandbox**.
  Reject anything outside these roots, credential access or unrelated commands.
- Model-backed tests require a connection configured through original R settings.
  Real provider calls may incur charges: proceed only with your own explicit
  consent. The maintainer will not submit those requests for you. A controlled
  model fixture is preferable for repeatable approval/Stop tests.
- Never send keys, cookies, token URLs, credential files or unredacted settings /
  network / Inspector dumps. Report categories and redact screenshots.

## First three checks (one pass)

| ID | Do | Expected / record |
| --- | --- | --- |
| M01 | Launch with the supplied command. Dismiss the initial safety notice without accepting; then reopen it and explicitly accept. | Harness entry remains disabled before acceptance and becomes available after saved acceptance. Record any startup failure/category, never raw diagnostics. |
| M02 | Open Harness. Inspect missing-key setup before configuring a model. Select A through upstream UI. If you consent to provider use, configure the model and ask: “Read README.md and summarize it. Do not edit files or execute commands.” | Missing setup is truthful; settings remain usable. Record request receipt, streaming/progress and final reported outcome, or corrective model error. Verify files are unchanged; missing credentials are not a shell-ready success. |
| M03 | Leave a recognizable **unsent** draft. Select text and scroll in the conversation. Red-close the window; reopen using Dock or File/Open Workspace Window. Repeat once with Command-W. | Same conversation, draft, selection and scroll remain. No extra submission or setup restart. Do not expect unsent context to survive full quit/crash. |

M02 is not completed if you skip model setup. “Skipped” is acceptable; it is not a
passing task result. If any first-three check fails, stop and report it first.

## Remaining interactive checks

| ID | Do | Expected / record |
| --- | --- | --- |
| M04 | Start a bounded task. Close/reopen while it is running. | Work continues; no closure-induced cancellation, duplicate submission or reset. Record state before/after; a too-fast task cannot qualify this running-state case. |
| M05 | In original R permission UI select/record a policy requiring approval for the intended write. Request creation of `approval-marker.txt` containing only `approved`. Do not allow yet; inspect the disposable directory. | Policy, actual action, target and allow/deny controls visible. File remains absent while pending. If policy or a pending request cannot be obtained, report **Blocked**, not passing. Prompt wording alone does not establish a permission gate. |
| M06 | Deny that request. In a **separate new request**, obtain another pending approval and dismiss/cancel its UI without allowing. | Neither denial nor dismissal executes the gated action. Record actual pending/denied state and filesystem result, including later follow-up attempts. Never count an ungated action as a denied approval test. |
| M07 | Obtain another fresh approval; hide/reopen while pending, then explicitly allow only the reviewed safe write. | Pending remains undecided during hide/show. The allowed action creates only the intended marker; final state matches actual outcome. |
| M08 | Stop during a streaming response; separately Stop during a reviewed bounded in-flight action (for example a 20-second sleep). | Visible stopping acknowledgement and truthful final state. No later new actions for that task; already-started actions may finish. Stop does not quit dsh or promise rollback. Use a marker/counter to record actual action effects. If too fast to Stop, rerun or mark untested. |
| M09 | Submit identical text deliberately twice; separately submit a follow-up while work is active. | Both explicit submissions remain distinct. Queue/steer behavior follows R; no shell deduplication or submission lock. Record observed upstream behavior rather than prescribing a queue policy. |
| M10 | Create a separate session in B. Switch A/B while one task or approval is active. | Original identities/workspace associations remain; switching neither decides approval nor replays/stops work. No second shell workspace-confirmation policy. |
| M11 | With active work, invoke Command-Q; select Stay. Repeat while hidden using the native menu. Request quit repeatedly before resolving it. Finally explicitly Stop and Quit. | At most one unresolved confirmation. Stay preserves work. Confirmation warns about all sessions/workspaces; full quit interrupts work and cleans the owned service. Maintainer verifies captured process cleanup; do not kill every process named node/dsh. |
| M12 | Relaunch through the same isolated command; reopen the original A/B sessions. | Saved history and associations remain. Interrupted work/approvals do not execute automatically. No guarantee for an unsent draft after full quit. |
| M13 | Replace a model credential through supported upstream UI; test the next dependent operation. Attempt supported removal. | Current availability and corrective errors are truthful. Do not disclose key values. If the official provider has no independent Remove action, record the known upstream gap as **Blocked**; do not edit storage or invent a shell removal UI. |
| M14 | Navigate shell safety/status and native quit controls with keyboard; optionally enable VoiceOver. | Meaningful labels, visible focus, actionable state and cancel/confirm behavior. Report preservation of R accessibility separately from shell controls. |

Record observed idle/running/awaiting approval/stopping/completed/failed/interrupted
states. An absent state, policy label or corrective outcome is a gap, not a state
inferred from a spinner. Service-failure/forced-exit testing needs a separately
scheduled owned-PID fixture; do not simulate it by killing unrelated processes.

## Results to send

```text
Candidate/revision:
macOS / hardware:
OmniWM enabled: yes/no
M01: Pass / Fail / Blocked / Not run — observation
M02: Pass / Fail / Blocked / Not run — observation
M03: Pass / Fail / Blocked / Not run — observation
Other IDs tested:
Failure: exact steps, expected result, actual result, redacted screenshot if useful
```

Do not submit a single “all passed” without case results. User reports remain
user-observed evidence until correlated with the candidate and relevant records;
not every qualitative pass closes a quantitative/native gate.

## What this does and does not complete

- This first pass finds actionable UI/task/lifecycle defects. It does **not**
  automatically check T032/T033 or certify US2/US3 implementation complete.
- SC-007 needs ten documented hide/show cycles, covering active work and pending
  approval, plus ten confirmed quits with Stay checks. M03 alone is insufficient.
  Pair the later repeated run with maintainer process/document/transfer identity
  capture, not just screenshots. Suggested ten-cycle mix: 3 idle, 4 running,
  3 awaiting approval; include draft, selection, conversation and scroll checks.
- SC-002 needs 20 saved-valid-setup launches on the named otherwise-idle host,
  process-start-to-task-ready instrumentation and >=19 within ten seconds. A
  stopwatch or missing-key setup is exploratory, not qualifying evidence.
- Native bytes/headers/origins/other views/stale replies, dispatch/no-replay counts,
  per-operation credential resolution and receiving-boundary latency stay automated
  or instrumented. No keyboard automation will run during your manual session.
- Minimum macOS 14/clean account/remote access, O comparison and developer study
  require separate environments/evidence. Known upstream capability gaps remain
  release-blocking; passing this list does not waive them.
- Local ad-hoc signing/closure verification is not Developer ID, notarization,
  public Gatekeeper or DMG qualification. This temporary candidate is disposable,
  not a promise of production installation or history migration.

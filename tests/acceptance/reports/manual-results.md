# User-led manual results

The user reported M01–M03 passed after the candidate handoff, then reported the
next batch M04–M06 passed and supplied a screenshot with a new composer defect. This is **user-reported manual evidence**, not an independently captured
native trace or a full release qualification. No keys or raw output were requested.

| Case | User-reported result | Scope |
| --- | --- | --- |
| M01 | Pass | Initial safety-notice gating and explicit acceptance |
| M02 | Pass | Upstream model setup and read-only summary workflow |
| M03 | Pass | Unsent draft/conversation context retained across close/reopen |
| M04 | Pass | Running-task hide/show, reported by user |
| M05 | Pass | Policy-gated pending write, reported by user |
| M06 | Pass | Denial/separate dismissal, reported by user |

The report contains no per-step screenshots, timestamps, process/document/transfer
identities, explicit hardware/OS confirmation or repeat counts. Candidate identity
is the supplied handoff (`manual-candidate.md`/data.json, source `2bdf282`), not
separately reconfirmed by the user. Do not infer measured SC-002/SC-007, every
selection/scroll subcase, zero replay counts or task/provider details from this
aggregate statement. Preparation metadata remains historical, not a native test
record captured during the user's run.

M07–M14 remain **not reported**. The screenshot's model narrative about an approval
and resulting file is not independent evidence of actual filesystem effects or
the denied/dismissed cases; no tool trace was supplied. The aggregate passes remain
user-reported, not an automatic SC-004 qualification.

**Open defect:** after a response completes, typed composer characters appear in
reverse order; focusing another surface and refocusing the composer restores
normal input. See `composer-caret.md`. Pause the next batch to narrow this defect.
Do not treat the workflow passes as evidence that post-response typing is sound.

No desktop automation is run alongside the user's manual session. T032/T033 and
other quantitative/security/platform/release gates remain pending as documented
in `manual.md`; these three reports do not advance their checkmarks.

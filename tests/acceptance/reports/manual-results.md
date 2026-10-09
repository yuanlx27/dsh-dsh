# User-led manual results

The user reported that all three first-pass checks passed after the candidate
handoff. This is **user-reported manual evidence**, not an independently captured
native trace or a full release qualification. No keys or raw output were requested.

| Case | User-reported result | Scope |
| --- | --- | --- |
| M01 | Pass | Initial safety-notice gating and explicit acceptance |
| M02 | Pass | Upstream model setup and read-only summary workflow |
| M03 | Pass | Unsent draft/conversation context retained across close/reopen |

The report contains no per-step screenshots, timestamps, process/document/transfer
identities, explicit hardware/OS confirmation or repeat counts. Candidate identity
is the supplied handoff (`manual-candidate.md`/data.json, source `2bdf282`), not
separately reconfirmed by the user. Do not infer measured SC-002/SC-007, every
selection/scroll subcase, zero replay counts or task/provider details from this
aggregate statement. Preparation metadata remains historical, not a native test
record captured during the user's run.

M04–M14 remain **not reported**. Next batch: M04 running hide/show, M05 genuine
policy-gated pending write, M06 denial and separate dismissal. Pending and denied
writes must remain absent; inability to obtain the gate is Blocked, not Pass.
Use only the disposable workspace and review actual proposed action/target.

No desktop automation is run alongside the user's manual session. T032/T033 and
other quantitative/security/platform/release gates remain pending as documented
in `manual.md`; these three reports do not advance their checkmarks.

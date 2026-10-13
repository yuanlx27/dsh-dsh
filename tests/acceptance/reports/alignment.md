# Official alignment — core regressions and open native comparisons

## T046 regression coverage

`src-tauri/tests/official_alignment.rs` adds four passing product regressions:

- Source contract: native Close/Quit menu items carry Cmd-W/Cmd-Q accelerators
  and route the same closed command identities; renderer/contextual `page.close`
  is not a native command. This is **not native event-loop/focus execution**.
- Late dialog/shutdown success after disposal cannot authorize exit.
- Proven service exit before quit differs from unconfirmed cleanup, which warns.
- Service loss during an already pending decision does not implicitly grant it.

Reuse existing `tests/lifecycle.rs` rather than copy its ten cases: repeated
requests join one decision; unknown/hidden/active/approval states conservatively
confirm without a task inspector; Stay/dismiss/failure do not stop; failed owned
cleanup blocks exit and requires a new decision. Four new plus ten reused tests
pass. Approved A3 always-confirm/A4 native Close Window adaptations remain intact;
no official idle-inspection/contextual Close Page parity is claimed.

## T050 discovered ownership/quit deviation — RED to GREEN

Review found `Runtime::fail`, invalid-announcement handling and startup deadline
published final non-cleanup failure **before awaiting owned child cleanup**.
`service_alive(Failed(Connection/Timeout/...))` is false, so native Quit could
interpret a still-cleaning owned process as absent and choose immediate Exit.

New `failure_cleanup_must_remain_live_for_quit_until_owned_exit_is_confirmed`
uses a real owned synthetic child blocked on a private cleanup-release marker.
It captures the failure-phase state, releases/joins the child even if the later
assertion fails, then asserts conservative quit liveness during pending cleanup.
It failed on the old behavior and passes after the minimal state-publication fix.

Reuse existing `Failure::Cleanup` while ownership/exit is unconfirmed; auth and
permits still revoke immediately and Retry remains blocked. After successful
owned exit publish the original category, otherwise retain Cleanup. No process
manager, task inspection API, new failure schema or new runtime subsystem added.
Deadline regression now waits for final Timeout after pending cleanup, rather
than confusing the interim unconfirmed phase with the final outcome. The initial
failure still publishes at the deadline; no startup ceiling relaxation.

The three repeated transport-fixture modules in lib tests also caused prior
all-target desktop Clippy failure. One cfg(test) module is now reused via imports;
no warning suppression or third-party/fixture behavior modification.

Checks: default-feature Rust suite **36 lib tests** plus integrations passes;
four alignment/ten lifecycle tests pass; desktop **all-target** Clippy passes
with warnings denied. 94 packaging, source policy and strict tooling/browser
checks pass. The existing installed b8169e0 acceptance candidate does **not**
contain this subsequent Rust cleanup-state fix; no new native race result claimed.

## Actual comparison boundary

[Official O reference](official-reference.md): immutable checkout, frozen install,
native addon and five unit suites/101 tests pass. These are not installed O GUI
results. Product installed subsets linked in [comparison protocol](../official-comparison.md)
remain separate. A1/A2/A5/A6 installed product/reference comparisons (T048), exact
SC-009 ten-trial native matrix (T049), any further resulting corrections (T050),
and full SC-008 remain pending. No alignment-complete release claim.

# T032 — actual WK negative probe (partial, unbundled debug)

Host `trochili.local`, arm64, macOS 27.0.1 build 26A434. Base checkpoint
`712ed44`, plus the debug observation changes accompanying this report.
This is **actual WK**, not a browser mock, but **not an installed production
candidate run**. The already-qualified installed credential observations remain
separate in `us1.md`.

## Instrumentation and execution

The unbundled debug app used the verified staged closure, fixture-only consent,
auto-entry and native Inspector. Private root was
`/private/tmp/dsh-installed-private.PTpUEZ`; app/launcher/Node were
`43657/43709/43710`. No model credential, task or provider operation was submitted.

The new observer is compiled only with `debug_assertions` and `desktop`:
`bridge/macos.rs` records a fixed category from actual `native_sender(message)`
and `validate_sender`, before reading any body. Categories distinguish WK's
main-frame flag and metadata admission, not renderer-provided flags. No URL,
origin tuple, body, cookie/token or reply value is logged. It reuses the existing
fixture-root-only native marker sink. It introduces no handler, command, permit,
renderer authority or production logging.

`tests/acceptance/browser/wk-negative.ts` was compiled to ignored JavaScript
outside the repository, then the generated file was pasted into the owned WK
Inspector. The script discards successful bootstrap contents; reports only fixed
scenario/outcome labels. Browser code has its own strict DOM project, separately
checked by `npm run typecheck:tools`, so adding DOM globals does not silently
change Node's fetch/stream typing.

## Retained native observations

For the bounded second probe, native counters captured before/after execution:

| Native marker | Delta |
| --- | ---: |
| `wk-harness-child-sender-denied` | 1 |
| `harness-bootstrap-admitted` | 2 |
| `wk-harness-main-sender-denied` | 0 |

The program calls bootstrap positively before and after its negatives. The
inherited initial about:blank iframe receives the **same** envelope as the main
frame. Calling its handler produced WK's actual non-main-frame category and was
rejected before body parsing/dispatch. This proves this exercised child-frame
rejection, not every child origin/window configuration. A caller's extra
`mainFrame` field is not used to build a native Sender.

Wrong window/runtime generations, a forged extra main-frame label and an oversized
envelope were also exercised between the positive calls. Native bootstrap
admissions increased only by two; no extra bootstrap dispatch was observed.
However, the final per-scenario browser rows were not retained: the shared desktop
became occupied by another editor window. **Do not promote this aggregate counter
observation to a fully passing per-case promise/reply matrix.**

The first, unbounded pilot stalled while testing general Tauri invoke. Inspector
showed CSP refusal of the `ipc://localhost/...` connection. The driver was corrected
to bound each observation to 1.5 seconds, retaining timeout as distinct from
rejection. This is not a product CSP fix: **do not add IPC connect permission to
make the probe pass**. General IPC commands, their native ACL refusal, valid command
arguments and both IPC carriers remain pending; a blocked connection or an
unobserved reply is not equivalent to comprehensive native ACL qualification.

No actual late-document reply, other owned/unowned WK view, external/sandboxed
origin or stale opaque transport handle was qualified by this probe. Wrong
renderer generation numbers are not a substitute for an actual destroyed/stale
native document test.

## Cleanup and checks

Keyboard automation stopped when concurrent use of the shared desktop was
observed. The probe app received owner-only TERM, not a claimed confirmed native
Quit; eight seconds later its complete captured trio was absent. The owned display
assertion was stopped. Clipboard cleanup is conditional on still containing the
exact generated fixture file, preserving any subsequent user clipboard contents.

Observer source regression: RED exit 1 before instrumentation, then GREEN.
70 packaging cases and default Rust tests pass. Strict Node/browser tooling and
source-policy checks pass. Desktop lib/application Clippy passes. An additional
`--all-targets -D warnings` Clippy attempt failed on the existing duplicate
`tests/transport/fixture.rs` test-module inclusion; it was not silently waived or
reported as passing. Cargo's unrelated lockfile ordering rewrite was restored.

**T032 remains unchecked.** Repeat on a finalized installed candidate with complete
closed browser outcome capture, native sender/view categories and dispatch counts;
then complete the remaining byte/stream/document/origin/admission and platform
matrix. Native keyboard-driven tests need a temporarily exclusive interactive
desktop; do not inject keys into concurrently used editors.

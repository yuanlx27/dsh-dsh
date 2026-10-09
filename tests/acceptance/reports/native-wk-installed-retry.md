# T032 — installed actual-WK retry (partial qualification)

Host: `trochili.local`, arm64, macOS 27.0.1 build 26A434. Implementation
checkpoint `2dc8c57`; this change improves the typed probe/result capture only,
not the native authority implementation. The user reported closing OmniWM;
process enumeration showed no OmniWM process before the retry.

## Candidate and instrumentation

Copied, locally ad-hoc-signed debug candidate with spaces:
`/private/tmp/dsh wk install.xEG2iU/DeepSeek Harness Desktop.app`.
The existing acceptance-only finalizer signed 17 nested Mach-O files and verified
26,761 final artifact hashes. Both the actual native installed closure verifier
(`verify-installed`) and `codesign --verify --deep --strict` passed. Final manifest
and app-binary SHA-256 identities are in `native-wk-installed-retry-data.json`.
This is not T052 packaging automation, a DMG, minimum-macOS or public Gatekeeper
qualification.

Debug differences: existing native fixture consent/auto-entry/Inspector switches,
fixed native sender observer and private root
`/private/tmp/dsh-installed-private.PTpUEZ`. Sender metadata still comes from
WKScriptMessage/frameInfo/securityOrigin and native view identity; no renderer
field grants authority. No authority-check relaxation or IPC connect permission
was added. **This is an installed debug instrumentation run, not an installed
production release negative-matrix pass.** No model key, provider call or task was
submitted. Bootstrap values were discarded, never printed or archived.

## Closed per-case result

The strictly type-checked TypeScript probe was compiled outside the repository and
its generated file pasted into the owned WK Inspector. It records only fixed
scenario/outcome/failure categories in an accessible status node. Dismissing R's
missing-key dialog via Configure later exposes that node to AX; this is not setup
completion or task-readiness qualification. Only the closed result JSON is copied
into the tracked dataset, not an AX/console/document dump.

Final observed app/launcher/Node: **51351/51393/51394**.

| Scenario | Actual outcome | Failure category |
| --- | --- | --- |
| Main bootstrap positive | Resolved | — |
| Wrong window generation | Rejected | desktop-denied |
| Wrong runtime generation | Rejected | desktop-denied |
| Forged extra main-frame field | Rejected | desktop-denied |
| Oversized string envelope | Rejected | desktop-denied |
| Inherited about:blank child, identical envelope | Rejected | desktop-denied |
| Tauri window enumeration | Rejected | acl-denied |
| Tauri webview enumeration | Rejected | acl-denied |
| Tauri event listener registration | Rejected | acl-denied |
| Main bootstrap positive after negatives | Resolved | — |

Native observer delta independently corroborated **one actual WK child sender
denial**, **two bootstrap dispatch admissions**, and zero main-sender metadata
denials. Wrong envelope/generation/body rows pass sender metadata but are rejected
at later parsing/admission gates; do not call them native origin mismatches.
No negative request generated an additional bootstrap dispatch.

The general Tauri invoke path first encountered the unchanged CSP denial for the
custom `ipc:` protocol and then returned command permission refusals. Failure
classification requires both `not allowed` and Tauri's exact **Permissions
associated with this command** diagnostic marker. Command/plugin-not-found and
invalid-argument diagnoses have precedence, so missing commands do not count as
permission rejection. The raw diagnostic, URLs and headers are never recorded.
All three final IPC rows met this strict category, not merely a timeout or generic
network failure. This qualifies these three calls under these conditions, not all
commands, both carriers independently, or every window/origin.

Cross-realm child errors are read through a guarded string message property,
not `instanceof Error` in the parent realm. An earlier provisional classifier
labelled that child unclassified and used a broader ACL match. Those provisional
labels are not the final dataset: the stricter program was compiled and the full
installed run was repeated, retaining all ten rows above.

## Window/observer corrections and pilots

Closing OmniWM did not by itself remove every observation failure: an initial
System Events coordinate click returned `-25208`. A temporary native Swift
CoreGraphics pointer probe used existing Accessibility permission but did not
produce a valid test result. No permission was bypassed. A later full-window AX
traversal exposed Inspector's actual **Console prompt**; explicitly focusing it
worked. Clear Filters and Configure later made the closed rows observable.

A flat AX-button traversal timed out because the Inspector tree/history was large;
subsequent actions targeted named controls directly. A later prompt-focus attempt
used an outdated group index (6 instead of 14) and failed with `-10006`: its native
counters remained zero. Refreshing the AX path fixed the observer; only the
subsequent complete, classified run is retained. No failed pilot is called passing
and no stable-window conclusion is attributed solely to OmniWM.

The unbundled retry also captured two resolved/eight rejected rows before the
installed repeat. Its earlier/provisional diagnostics are not substituted for the
final installed dataset. The first installed trio `50301/50333/50334` and its
shell driver `50259` were cleaned by explicit native Stop and Quit before the final
strict run.

## Cleanup and validation

Final explicit native **Stop and Quit** cleaned **51351/51393/51394**. Owned display
assertions were stopped. Generated clipboard cleanup checked exact file equality
before clearing, preserving any later user clipboard replacement. No owned
candidate service remains. Native app/launcher/source code was unchanged by this
retry; 71 packaging tests, source policy and strict Node/browser tooling pass.
Result-retention and strict failure-classification source regressions were RED
before implementation and GREEN afterward.

**T032/T033 remain unchecked.** Other windows/views/origins, actual old/destroyed
native documents and late replies, opaque byte/stream handle rejection, complete
credential removal, remote-machine admission and remaining lifecycle/platform
cases are still pending. Wrong renderer generation numbers are not an actual
stale-document test. Nothing here qualifies saved-valid/idle-host SC-002 timing or
running-task/pending-approval SC-007 retention.

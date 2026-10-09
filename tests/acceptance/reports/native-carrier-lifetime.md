# T032 — actual WK carrier scope across native destruction (partial F09)

## Fixture isolation, not a second R frontend carrier

The previous standalone program could not open a second mux because R already
owns the document's sole physical carrier. This run preserves that constraint.
A minimal host-selected **debug bootstrap** switch
`DSH_DESKTOP_FIXTURE_CARRIER_ONLY=1`, together with the existing fixture root gate,
adds a non-authority fixture boolean. `harness.ts` withholds the original R frontend
entry/transport initialization only when both debug fixture booleans are present,
showing **Carrier acceptance fixture** instead. Original R assets and the real
bundled R service/mux are unchanged. No replacement engine or peer was introduced.

The native flag emission is compile-time debug-only. Release bootstrap cannot
select it; compiled frontend code contains the conditional but does not grant
native authority. Sender/view/origin/notice/readiness checks, private native
Cookie/Origin, handle scope and the physical singleton are unchanged. Fixture
consent/Inspector/destruction instrumentation are not production UX evidence.

`tests/acceptance/browser/wk-carrier.ts` is strictly typed, compiled outside the
repository and pasted into the owned WK Inspector. It obtains a real native mux
handle, sends only the previously exercised harmless unknown endpoint and reads
its real R reply through native ordered chunks. It checks sequence zero, text
kind, end flag and nonzero decoded byte length, discarding reply bytes without
printing or recording them. This is not a large/binary/byte-equality qualification.
Scoped opaque metadata is held temporarily in a distinct localStorage key; closed
output records only generations and fixed scenario/outcome categories.

## Installed run

Host `trochili.local`, arm64, macOS 27.0.1 build 26A434. Installed debug candidate
`/private/tmp/dsh carrier install.cTx6B6/DeepSeek Harness Desktop.app`, native base
`501745d` plus the debug fixture. Existing acceptance-only finalizer signed 17
nested Mach-O files and verified 26,761 final hashes. Native installed closure
verifier and deep/strict codesign passed. Hashes and closed output are in
`native-carrier-lifetime-data.json`. This is not T052/distribution or installed
production qualification.

Captured app/launcher/Node **70277/70336/70337**. Original generation **2**, runtime
**1**:

| Preparation | Observed outcome |
| --- | --- |
| Real R unknown-endpoint carrier round trip | Resolved |
| Second physical carrier in same document | Desktop denied |
| Next read, observed after 500 ms | Pending |

Host SIGUSR1 actually destroyed the primary. AX reported **zero** native windows;
no automatic quit dialog appeared. All three captured PIDs remained unchanged.
File/Open recreated the shell and explicit Open Harness entered a fresh fixture
at generation **4**, runtime still **1**. An early shell button lookup raced its
rendering (`-1719`); refreshing the AX tree and retrying the owned button succeeded.
It was not a permission bypass, successful intermediate probe or timing test.

| Replacement | Observed outcome |
| --- | --- |
| Actual window generation changed | Confirmed |
| Runtime generation retained | Confirmed |
| Fresh main bootstrap | Resolved |
| Actual previous generation | Desktop denied |
| Actual old stream handle read | Desktop denied |
| Actual old stream handle write | Desktop denied |
| Fresh native carrier round trip then cancel | Resolved |
| Old read resolution marker | Absent |
| Fresh main bootstrap after negatives | Resolved |

All refusals are explicit native errors, not the 1.5-second observation timeout.
The replayed old handle is the actual successful prior carrier handle, not an
invented identifier. Fresh carrier success also verifies old document revocation
no longer monopolizes the native physical singleton. The old read was genuinely
pending before destruction; its completion marker was not seen by the replacement.
**Marker absence alone is not proof of all late-reply races**: the old JS context
may have disappeared. No deliberately delayed peer reply or every completion
interleaving is qualified here. The program removes its fixture keys afterward.

## Normal frontend control and cleanup

Relaunched the same signed candidate **without** the carrier-only flag. Original
R missing-key setup was actually observed and the fixture banner was absent:
app/launcher/Node **71266/71322/71323**. Thus the conditional does not always
suppress R. This control did not submit a credential, model or task operation.

Both runs used explicit native Stop and Quit and cleaned their complete captured
trios. Owned display assertions stopped; generated clipboard cleared conditionally.
Private AX trees and raw console/document dumps were not committed.

75 packaging tests, source policy, strict Node/browser tooling, frontend build,
default Rust tests, desktop lib/bin Clippy, debug build and release desktop check
pass. The debug fixture gate source regression was RED before implementation and
GREEN afterward. Existing all-targets transport-fixture lint issue remains pending.

**T032/T033 remain unchecked.** This closes the exercised actual old-stream-handle
scope cases in an installed debug carrier-only page, not the production full-R
carrier/reconnect matrix, arbitrary delayed replies, other native views/origins,
complete bytes/credentials/remote/minimum-OS, idle-host saved-valid timing or
substantive running-task/approval lifecycle qualification.

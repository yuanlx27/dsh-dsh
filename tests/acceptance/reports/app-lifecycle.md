# T028: native authentication/notice/document orchestration

Application serializes startup joins through one Runtime, consumes a token once,
authenticates natively, prepares Protocol on a blocking worker under the original
15-second deadline, and only then completes Ready. Preparation failure invalidates
credentials and cleans the owned service. Notice revision matches TS exactly;
entry needs the current durable save and authenticated prepared native generation.
Local projections contain only phase/revision/acknowledgement/fixed categories.
No session is returned without this application's current native permit.

Runtime's actual watch value is now a Bridge witness: admission rejects failure
before the asynchronous app observer catches up; final original-frame delivery
holds the watch read guard through completion. Reconciliation serializes bridge
updates with native local state, avoiding out-of-order observer retirement.
Failure/shutdown/disposal invalidates sessions and all native transfer permits.

Main-thread `app::macos::Document` revokes Registration before navigation or
replacement, installs fresh generation authority and loads a fixed native URL.
The non-secret generation fragment is checked against actual WK frame/view URL,
not body claims; old-frame metadata cannot gain authority by claiming fresh IDs.
Drop detaches transfers only, not dsh task/approval state. A native WK navigation
observer retains/forwards Wry's delegate, tracks the current navigation identity,
ignores intentional NSURLErrorCancelled replacement, and revokes on current page
failure or WK content-process termination. Hiding does not touch this guard.
Native primary-window/entry event integration remains T029/T031.

Executed fixture tests: joined real loopback auth/preparation (one launcher),
saved notice entry, owned shutdown, preparation failure/no retry, notice revision
and bounds/failing save; live watch failure before observer and stale frame/view
fragment denial. Controlled live-witness bypass fails the stale permit assertion;
restored. Selected debug lib/foundation/auth/protocol/HTTP/stream: 56 passing plus
proxy subprocess; release native-bridge library: 26 passing. Default/native Clippy,
format and desktop-feature library check pass. Test protocol is explicitly an
empty mock, not R's prepared graph; actual graph contracts remain T021's tests.

Not executed: actual WK navigation/delegate forwarding/fragment retention,
process termination, main-thread original-frame completion or installed R boot.
If WK request metadata strips the fragment, entry deliberately fails closed and
requires compatibility correction before T032 acceptance. Neither the compiled
observer nor synthetic frame fixtures establish executed WK qualification.
App binary still requires the icon deliverable and T031 entry wiring; full
lifecycle suite awaits T029/T030 window/quit modules. No app-ready claim.

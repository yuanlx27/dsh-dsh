# T035 — opaque pinned-R forwarding regressions

Producer: locked R `639ed015397290b3745d163aafe02ffee4aa3f84`, dsh `0.2.0-rc.2`.
`tests/fixtures/r-request-recording.ts` actually executes downloaded R's
`dsh-api-gateway/lib/types/client/stream-client.js` RemoteStreamMuxClient and its
stock protocol parser. A test-process fake socket captures R's serialized open
frames for two explicit identical public payloads. These are executable-SDK
fixture recordings, **not live backend/task/UI captures**. The fixture endpoint
`fixtures/request` is not claimed to be an actual task API. No key/network/user
window is used; fresh stream IDs come from R itself, not handwritten envelope IDs.

The fake socket/document descriptors exist only while the producer is captured
and are restored in finally. Product installTransport does not replace WebSocket;
regression asserts the actual global remains unchanged after forwarding.

`task-forwarding.test.ts` consumes those recorded opaque endpoint/payload values:

- Each of two legitimate identical explicit calls dispatches once, not deduplicated.
- Initial failure dispatches once and does not cause adapter retry.
- Supplier recovery alone dispatches nothing.
- Detach/reinstall cannot replay; stale hooks reject. Only explicit new invocation
  creates another dispatch after recreation.
- In-flight signal cancellation and carrier loss terminate without reopening.
- A pre-aborted call dispatches nothing; signal/uplink and duplicate uplink items
  pass through without task-state interpretation or rewriting.

Existing native transport tests separately cover mux demultiplexing/cancellation;
no new product adapter/task admission logic is introduced. Missing recording-module
RED preceded implementation, targeted regressions GREEN. Full packaging/source/
strict tooling checks cover the new TypeScript files.

This completes T035's adapter regression preparation, not native WK admission,
receiving-boundary latency or end-to-end task semantics. T036 must compare real
standalone/native task operations; T038/T039 cover recovery and measurements.
The pending intermittent composer issue is not fixed by these transport tests.
T032/T033 and all other release-blocking acceptance gaps remain pending.

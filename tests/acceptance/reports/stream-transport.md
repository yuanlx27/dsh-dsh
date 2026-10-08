# T026: owned native WebSocket carrier

Native tokio-tungstenite connects only the current session's literal loopback
`/api/remote.mux` with native cookie/Origin. No destination argument, proxy,
redirect/reconnect loop, business decoder or task state. Text/binary messages
remain opaque and ordered. Pinned ws's 100 MiB payload default bounds each frame;
uplink/downlink queues hold one frame each and the socket write buffer is bounded.
A clonable ordered sender is independent of the reader: uplink is not blocked by
a pending downlink read. Drop cancels both tasks and releases the physical socket.
Cancellation, session revocation and admitted native permit invalidation deny
old queues/senders; `connect_admitted` is the renderer integration entry point.
The generation-only connect is native fixture IO, not an IPC authority.

Two real loopback WebSocket tests execute: native handshake cookie/Origin,
opaque text/binary echo/order, concurrent send during a pending read, sequence
poison/cancellation/session revocation. An admitted native fixture test rejects
old queued/read/uplink/reconnect after window detachment. Shared disposable HTTP
fixture accepts WS on the same authority, not a separately selected port. Its
upstream callback fixes a large ErrorResponse type; only that callback statement
has a documented Clippy result_large_err exemption.

Debug selected library/HTTP/stream: 30 passing (21+7+2); release library/streams:
23 passing. Selected Clippy/format/diff checks pass. Rust/Tokio executes all
carrier work; JS WebSocket globals are not changed. T027 adapts R's logical mux
open/item/end/cancel protocol, not business messages; upstream Connection still
owns requested retries. Main-thread WK chunk/handle dispatch is T031 integration,
not completed here. Real R stream admission, large-frame pressure, native frame
scope and installed stream/task/Stop qualification remain pending T032/US2 gates.

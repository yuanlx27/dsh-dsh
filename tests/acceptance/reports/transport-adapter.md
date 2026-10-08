# T027: TypeScript R hook and byte adapter

`src/transport.ts` installs pinned R's ownsHost/fetch/openStream hooks. Explicit
service paths cannot select authority or token auth; same-app `/api/` raw fetch
uses these hooks (including multipart/downloads), while immutable assets and
external fetch retain the original fetch. No cookie/origin/port is exposed and
WebSocket is untouched. Stale hooks reject; disposal restores previous hooks
without overwriting a later owner. Business values/uplink are passed unchanged.

`src/native-transport.ts` supplies the native-command byte client: ordered 64 KiB
base64 chunks, 300 MiB HTTP input, pull-based Response bodies, Abort/reader cancel,
scoped handles and disposal. It adapts exactly R's public physical mux protocol
(open/item/end/cancel and item/error/end), with one physical carrier, distinct
logical IDs, opaque values, no task/event interpretation or deduplication and no
automatic retry. Pinned R Connection owns retry requests; only a new explicit
open may start another carrier. Frames are bounded at R's ws 100 MiB default and
logical inboxes at 32 items; pressure fails the carrier, never silently drops an
item. Remote errors retain code/details/marker. Stop sends a logical cancel and
releases uplink; dispose cancels native handles. The native command action schema
is consumed by T031's pending WK dispatcher, not a generic public IPC command.

11 previously red T017 adapter contracts now pass. Two added byte/mux tests
exercise a real 70,000-byte multipart body (including boundary), binary response,
concurrent read/send, distinct stream IDs, duplicate values and Stop cancel.
Controlled HTTP chunk byte-drop fails the multipart byte equality assertion;
restoring it returns green. Full packaging suite: 62/62 pass. Both strict tool
and browser/source build pass. Removed the missing-T027 temporary typecheck
exclusion; Node tooling retains its own standard-library stream types, browser
build retains DOM checking. No tracked own JavaScript or untyped suppression.

These are Node fake-native byte/adapter contracts, not live WK/real R transport
acceptance. Main-thread native handle/action wiring, live framework integration,
WK disposal ordering, physical pressure and installed upload/download/Stop remain
T031/T032 and US2 acceptance work. Do not substitute this report for those gates.

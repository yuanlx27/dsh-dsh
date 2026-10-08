# T025: fixed owned-service HTTP transport

Implemented reqwest forwarding with relative raw-target validation before URL
normalization, exact owned origin/cookie, no proxy/redirect/cookie jar or retry.
Caller auth/trust/Host/Fetch-Metadata/forwarding and hop-by-hop headers are removed;
Connection-nominated headers and response Set-Cookie are removed too. Only
standard supported methods. Private index/token targets are not a public HTTP
bridge endpoint. Binary and multipart bytes retain identity.

Input is ordered 64 KiB chunks with the exact 300 MiB R buffered default; any
invalid order/oversize poisons the body before dispatch. Output is pull-based
64 KiB ordered chunks, with at most one reqwest chunk pending (1 MiB cap), not a
background unbounded/global event queue. Caller cancellation, native frame permit
revocation and session invalidation cancel held headers/body reads. Cancelled
reply handles remain terminal even if a synthetic cancellation flag is reset.
Native authentication now has a watch wakeup on revocation, without exposing or
serializing credentials.

`fetch_admitted` requires the nonserializable T024 permit and checks it throughout
IO. Lower-level Rust-only `fetch(generation, ...)` remains for independent IO
fixtures; it must never be exposed via IPC. T031 must wire admitted handles and
main-thread one-shot replies; this task does not install a command handler.

## Actual evidence

Existing T016 seven HTTP tests now execute/pass (previously missing module before
assertions): malformed/foreign/token paths, method/header rejection, exact body
bound and poisoned input, binary/multipart/order, redirect/response stripping,
held-header cancellation and queued-reply session revocation. New admitted IO
fixture test proves native window revocation before dispatch and while headers
are held; it checks real fixture request counts. Library plus HTTP/auth selected
debug: 37 pass (+proxy subprocess); release lib/HTTP: 27 pass. Selected Clippy,
format/diff checks pass. Existing test MutexGuard was lexically scoped before
await to satisfy Clippy, not suppressed. `bytes` is promoted from the existing
locked dependency graph; no package versions changed.

All cookies/listeners here are disposable loopback fixtures, not upstream HMAC
admission or installed WK qualification. Actual raw uploads/downloads and native
request/reply wiring still await T027/T031/T032. Full lifecycle suite remains red
on pending app/quit/window modules; no installed-app pass is claimed.

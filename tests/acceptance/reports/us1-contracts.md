# US1 contract-first checkpoint

This report records test development, not installed-app acceptance. R remains
`@deepseek-ai/dsh@0.2.0-rc.2`; no O packages or business engine were introduced.

## T016: native authentication and HTTP contracts

- Added `src-tauri/tests/transport.rs` with a disposable loopback HTTP fixture.
  Synthetic cookie bytes reproduce R's name and opaque shape; they are not a
  real upstream signed cookie or a substitute for T032 admission checks.
- Covers private root exchange, required 303/clean redirect/service cookie,
  authenticated boot extraction and secret-echo rejection, startup deadline,
  destination/path/method rejection before dispatch, native auth/trust headers,
  Connection-nominated hop headers, response stripping, redirect rejection,
  exact 300 MiB limit, poisoned oversized-body dispatch rejection, byte-exact
  multipart/binary forwarding, sequential replies, cancellation and revocation.
- Command: `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
  --test transport`.
- Actual initial result: **red**, exit 101, E0432/E0433: authentication and HTTP
  modules do not exist yet (T020/T025). No transport assertion has passed and
  no WKWebView/frame authorization has been exercised.
- Fixture encoding reuses already-locked `base64@0.22.1` as a test dependency.
  Offline full-lock regeneration encountered stale cached `objc2` registry
  metadata; no versions were changed. Added only the root's existing-base64
  dependency edge, and the locked/offline test command accepted that lock.

Regression checks at this checkpoint: 18 existing Rust foundation/library tests
and 37 existing Node packaging tests passed; `cargo fmt` and `git diff --check`
passed. The full Rust suite intentionally remains red until T020/T025 exist.

## T017: upstream hook adapter contracts

- Added `tests/packaging/transport-adapter.test.mjs`, using Node 24's native
  TypeScript loader rather than requiring the unfinished shell build.
- Covers R's fetch/openStream hook shape, byte-exact raw upload/download routing,
  external/immutable-asset exclusion, custom-scheme authority (not null-origin)
  comparisons, destination/token rejection, opaque concurrent streams, duplicate
  item preservation, uplink disposal, cancellation, no adapter retry/replay and
  detached hook rejection. It never replaces global WebSocket.
- Command: pinned Node 24.21.0 `node --test
  tests/packaging/transport-adapter.test.mjs`.
- Actual initial result: **red**, exit 1, `ERR_MODULE_NOT_FOUND` for
  `src/transport.ts` (T027). This also intentionally makes the aggregate Node
  packaging command red. Hook fakes do not prove actual multiplex wire handling,
  native transfer handles or frame authorization; T026/T032 must qualify those.

T018 automated contracts and T019 native procedure are still pending.
Installed candidate, actual WKScriptMessage metadata, real R cookie admission,
macOS 14 qualification, performance and public signing gates remain pending.

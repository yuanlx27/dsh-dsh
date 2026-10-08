# US1 contract-first checkpoint

Historical `.mjs` paths below refer to original executions; tooling/tests now
use `.ts`. See [source migration](typescript-sources.md) for current commands.

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

## T018: notice, retained-window and quit contracts

- Added `src-tauri/tests/lifecycle.rs`: current saved revision only, failed
  writes deny entry, bounds preservation/no Harness preference fields, retained
  document identity, absent/failed-only recreation and stale native callbacks.
- Quit policy takes only service liveness, not task/session/window inspection:
  repeated requests join one decision; Stay/dismissal/failure cannot stop or
  exit; late responses are denied; confirmation waits for actual owned shutdown;
  failed cleanup cannot exit; an absent service needs no confirmation.
- Command: `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
  --test lifecycle`.
- Actual initial result: **red**, exit 101, E0432 for app/window/quit modules
  (T028/T029/T030). Fixture IDs test only native-side policy transitions, not
  renderer-provided identities. Existing runtime tests already cover coalesced
  service launch/owned exit; actual retained DOM and ownerless native dialog
  qualification remain T032/T033 and the T019 procedure.

## T019: installed WKWebView qualification procedure

- Added `tests/acceptance/startup-security.md` with explicit fresh/saved launch,
  denied writes, clean/offline/path-space/occupied-port cases; upstream credential
  lifecycle/per-operation/read-only-source/modes and workspace access cases;
  actual native frame/window/origin/generation rejection, redirects, byte/stream
  fidelity/cancellation, secret absence and unauthenticated local/remote access.
- Includes safe fixture preparation, actual-WK metadata requirements, validity of
  protected-route probes, sanitized per-scenario report fields, accepted bearer
  limitations and separate pending minimum-OS/public-signing gates.
- Procedure review/static ID checks pass. **No native scenario has been run.**

The US1 test-definition wave T016–T019 is complete. T020 is now implemented;
see [authentication checkpoint](authentication.md) for 10 executed auth tests,
28 passing Rust tests per debug/release run and a detected 303 regression.
Auth tests are independently runnable in `src-tauri/tests/authentication.rs`;
HTTP assertions remain in `transport.rs`, without placeholder implementations.
The initial red results above are historical, not the current auth result.
Next is T021–T023, followed
by native admission/integration and T025–T031. Contract APIs may be reconciled
with those implementations without weakening their behavioral assertions.
Full Rust/Node suites remain intentionally red for missing implementation;
18 existing Rust and 37 existing Node regressions passed before the new suites.
No installed native, auth, lifecycle, performance or release qualification is
claimed by this checkpoint.
Installed candidate, actual WKScriptMessage metadata, real R cookie admission,
macOS 14 qualification, performance and public signing gates remain pending.

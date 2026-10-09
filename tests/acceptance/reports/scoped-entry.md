# T031 — scoped native entry and dispatch

## Implemented

- Separate actual-WK owned shell main-frame permits and closed Status/Retry/
  Acknowledge/Enter messages. Harness bootstrap/HTTP/mux retain their distinct
  ready/notice/runtime-witness permits. No general invoke handler; `shell.json`
  grants zero Tauri filesystem/shell/network/plugin permissions.
- Main-thread original-frame reply retention (64 pending), bounded random native
  job handles (32), aggregate 300 MiB HTTP input reservations, ordered 64 KiB
  chunks, one physical mux per scope, independent reads/writes, cancellation,
  terminal removal and scope/session revocation.
- Tauri entry, frozen protocols/compiled TypeScript assets, coalesced verified
  startup, retained window/events/menu/quit and native failure recovery wiring.
  Native navigation uses one-use approved targets. Enter replies before changing
  the native document. Informational menus work even before runtime verification.
- R queue/registry/entry are reused, not replaced. Native transport uses Response's
  standard BodyInit encoder (no streaming Request-upload platform requirement),
  validates native stream frame order/type, and reuses R's existing
  `dshRemoteStreamFailure` worker-boundary markers. Opaque item values are intact.
- macOS icons are conversions of the pinned R favicon; provenance is in
  `src-tauri/icons/README.md`.

## Executed evidence

- Full locked/offline default Rust: **73 tests pass**, plus the isolated auth proxy
  subprocess. Two real fixture dispatch tests cover foreign scope, poisoned
  ordering, binary fidelity, chunked WS echo, independent pending read/write,
  duplicate physical rejection and revocation. Local closed-command/metadata
  assertions pass. Desktop Clippy `-D warnings` and full app binary build pass.
- Node packaging: **67 pass**. Strict tools/browser checks and source policy pass.
  Additional tests cover WK string envelopes, carrier marker compatibility,
  multipart encoding without Request construction, and rejected native frame
  sequence/binary metadata.
- Negative mutation: making local Status a serde unit variant instead of an empty
  struct variant makes the unknown-authority-field assertion fail (exit 101,
  assertion panic, not a compile error). Restoring it passes.
- Actual unbundled debug WK execution on arm64 **macOS 27.0.1, build 26A434**:
  owned shell main-frame admission and original-frame native replies; native
  authenticated/prepared Ready; approved native Harness navigation; actual
  Harness bootstrap admission; packaged queue and original R entry load;
  authenticated admitted HTTP; and native mux round trip to R's deliberately
  nonexistent `_desktop_fixture/unknown` endpoint yields R's remote terminal
  failure, not a local carrier failure. No task/provider operation was submitted.
  Evidence is fixed non-secret marker output from the isolated private fixture
  root; no token/cookie/body/raw exception is logged.
- The explicitly requested debug fixture seeded notice consent natively and
  auto-entered. This is **not** proof of human acknowledgement or setup UX.
  Owned smoke app termination was followed by launcher/Node cleanup. It is **not**
  proof of confirmed native Quit or retained-window performance.

## Development correction and limits

Cargo tests can overwrite `target/debug/dsh-launcher` with the source build,
invalidating the staged manifest hash. Unbundled debug startup now reuses
`BundlePaths::staged`; installed debug/release builds continue to verify their
installed closure without fallback. Native startup no longer hashes the closure
 twice: `LaunchConfig::verified_manifest` reuses the verified manifest.

Debug-only fixture environment controls (`DSH_DESKTOP_FIXTURE_ROOT`, optional
`DSH_DESKTOP_FIXTURE_ENTER=1`) and fixed markers exist solely for isolated smoke
observability. Renderer checkpoint reports are not authority or release gates.
Production bootstrap has no fixture flag. The debug-only GET markers target
nonexistent fixed paths; they create no desktop business API.

**T032/T033 remain pending:** installed app/DMG, complete WK rejection matrix,
credential lifecycle/permissive-file rejection, uploads/downloads/Stop, actual
focused close/quit outcomes, retained identity/performance, minimum macOS 14 and
accessibility/user qualification have not been executed. The screen-capture
attempt returned `could not create image from display`; no screenshot is claimed.
No local result establishes Developer ID/notarization or public Gatekeeper status.

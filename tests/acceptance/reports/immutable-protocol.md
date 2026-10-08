# T021: immutable app/shell resource protocol

Implemented `protocol.rs` as a native, memory-snapshotted byte responder with
separate `dsh-app://app` and `dsh-app://shell` tables. Only GET/HEAD; fixed assets,
exact current-generation module URLs, explicit MIME/length. No service client,
stream, token query, generic filesystem endpoint or bootstrap JSON endpoint.

## Pinned R composition, not a second module engine

`prepare-web-assets.ts` uses pinned R's `ClientModuleRegistry` prototype and
`bootInjections` at build time. Its narrow internal adapter supplies immutable
package metadata/bytes and cache tables; it does not construct a Host, activate
plugins, subscribe a loader, read user data or start a service. R prepares wrapped
primary registrations, lazy chunks, queue prelude and identity-map sections.
The published R closure has no client maps; a changed map closure fails preparation
rather than silently losing debug semantics. The baseline version is required.

Native code only concatenates these frozen pieces in the authenticated R graph's
entry/batch order, adds R's relative map trailers and indexed section offsets, and
requires exact entry/chunk/batch revision URLs. It does not invent module order,
dependency resolution, service wire methods or plugin activation. Cold resources
are cached and hash checked under the canonical packaged dsh root. Traversal,
encoded separators, double encoding, invalid UTF-8/escapes, control characters,
foreign authority/userinfo and arbitrary asset queries are rejected. Internal
symlinks use the target's inventory digest; external or unlisted targets fail.
Reads and combo responses have a 64 MiB ceiling; request payload transport remains
T025's separate 300 MiB contract.

The packaged frontend gets only a static external R queue-script element. The
non-secret native `Bootstrap` DTO supplies the six authenticated boot globals and
R bootstrap/preload URLs separately: **T024 must admit its native main-frame
caller before handoff**. It is not a renderer-label-authorized command or publicly
servable JSON. Later T028/T031 attach the WK document-start bootstrap and Tauri
scheme registration; this library completion is not an installed WK result.

`runtime:prepare` freezes web pieces before inventory/signing; `--inventory-only`
does not rewrite signed bytes. Existing staged resources were prepared without
redownloading dependencies, then re-inventoried: 26,760 artifacts (previously
26,610). Vendor/generated JS remains ignored. No production versions changed;
`percent-encoding = 2.3.2` is promoted from the existing locked dependency graph.

## Actual tests

- Tests written first: Node exited 1 on missing `prepare-web-assets.ts`; Rust
  exited 101 on missing `protocol`. These were pre-assertion failures.
- Two Node tests now pass: genuine pinned R primary/queue/lazy-terminal bytes and
  map sections, plus fail-closed missing closure. Native combo tests check actual
  prepared parts in order, exact trailers, indexed offsets, current/stale URLs,
  app/shell separation, no API/boot endpoint and traversal rejection.
- Three library tests cover private IO corruption/unlisted/external symlinks,
  internal target digests, immutable cached responses, HEAD lengths, Unicode
  decoding and chunk grammar. Three native integration tests pass.
- Controlled hash-check bypass: the corruption test failed its real assertion,
  exit 101. Restored the guard; debug and release library/protocol tests pass.
- Selected debug Rust regression: **34 passing** (13 library, 8 foundation,
  10 authentication, 3 protocol), plus proxy subprocess. Real bundle/addon
  foundation still passes under ordinary and space-containing native layout.
- Selected Clippy, format/diff, strict TS/source-policy and runtime verification
  pass. Node packaging: **41 passing, 1 expected missing-T027-module failure**.

Real upstream boot graph/service admission, WK custom-scheme subresources,
worker/blob URLs and installed frame/principal qualification are still T032 gates.
This report does not certify those, sign an application or qualify public delivery.

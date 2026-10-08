# T020 native authentication checkpoint

## Scope and provenance

Implemented `src-tauri/src/authentication.rs` for R `0.2.0-rc.2`. This is a
native-library/loopback-fixture checkpoint, not installed WKWebView acceptance.
No O packages, model-credential store or task logic were introduced.

- Reuses the runtime's token-URL validator; only literal HTTP loopback/root,
  valid port and one non-empty token query are accepted.
- Reqwest has redirects/proxies disabled and no cookie jar. Required exchange:
  303, exact `./` location and one authority-named, bounded R-shaped cookie with
  canonical opaque v1 fields and expected private session attributes.
- A clean authenticated index must return HTML/200. The complete exchange and
  streamed index extraction share the original startup deadline. Index data is
  capped at 2 MiB, including chunked responses without Content-Length.
- Extracts only the six pinned public JSON globals. Duplicate/malformed rows,
  missing boot/recovery, auth echoes and native-origin echoes fail closed.
  Raw HTML, queue scripts and unknown injections are not returned/executed.
- Token-bearing URL is discarded after exchange. Native session retains only
  generation and transient origin/cookie; neither session nor outer result has
  Debug/Serialize. Cookie headers are marked sensitive. Explicit invalidation
  drops credentials and rejects the generation; lifecycle wiring is still T028.
- Does not reproduce HMAC/signing storage: R's clean-index admission is the
  production proof of the minted cookie. Fixtures use synthetic opaque bytes;
  actual R cookie admission/installed native qualification remains pending.

## Executed tests

T016's five auth tests moved unchanged in behavior to `tests/authentication.rs`,
sharing `tests/transport/fixture.rs` with the remaining HTTP tests. Before
implementation, the independent auth executable failed with missing module
E0432; no HTTP stub or feature-gated test skip was introduced.

Five additional tests cover invalid destinations/expired deadlines, duplicate
cookie attributes/headers and bad opaque values, boot size/redirect/duplicate
and encoded-secret rejection, all public globals, idempotent revocation and a
separate-process inherited-proxy test (loopback-only bad proxy).

Both debug and release commands passed **28 tests each** (10 library, 8
foundation, 5 original auth and 5 additional auth), plus the proxy subprocess:

```sh
cargo test --locked --offline --manifest-path src-tauri/Cargo.toml \
  --lib --test foundation --test authentication --test authentication-extra
cargo test --locked --offline --release --manifest-path src-tauri/Cargo.toml \
  --lib --test foundation --test authentication --test authentication-extra
cargo clippy --locked --offline --manifest-path src-tauri/Cargo.toml \
  --lib --bin dsh-launcher --test authentication --test authentication-extra \
  -- -D warnings
```

Clippy initially found the old test's explicit `drop(MutexGuard)` scope before
await; changed it to a lexical scope. Final Clippy, formatting and diff checks
pass. Base64 moved from test-only to production dependency; its already-locked
version remains `0.22.1`, with no dependency resolution/version changes.

## Behavioral negative control

Temporarily bypassed only the required-303 condition and ran
`unsuccessful_exchange_cannot_follow_any_redirect_or_prepare_boot`.
It **failed an actual assertion**, exit 101: unexpected status was accepted
(`None` instead of `Some(Connection)`). Restored the condition immediately and
re-ran the passing debug/release suites. No injected defect remains in source.
This verifies that assertion detects that regression, not every possible defect.

## Remaining failures and release gates

Re-executed full `cargo test`: still exits 101 solely on missing HTTP and
app/window/quit modules (T025/T028–T030). Authentication no longer fails imports.
Re-executed full `npm run test:packaging` under Node 24.21.0: 37 passing tests,
1 missing `src/transport.ts` file failure (T027).

Native frame admission, immutable app/module serving, real Web-profile overlay,
lifecycle integration, actual upstream-cookie/byte/stream handling, installed
candidate, macOS 14, performance and usability qualification remain pending.
Developer ID/notarization/public-distribution gates remain deferred separately.

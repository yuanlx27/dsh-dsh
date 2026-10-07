# Foundation evidence (partial; blocked)

**Status: failing build-time staging; T015 remains incomplete.** T001–T012
implementation/test-authoring tasks are checked off. T013–T015 and all story/
installed-distribution tasks remain unchecked. Source completion is not runtime
qualification. No candidate .app/DMG or native transport has been qualified.

## Provenance

- Product/R: `0.2.0-rc.2`, source
  `639ed015397290b3745d163aafe02ffee4aa3f84`; O remains reference-only at
  `5badb15009ae1756c3afe0ae0cef1faafc290ccc`.
- Host observed: macOS 27.0 build 26A428, arm64; Xcode Command Line Tools present,
  full Xcode absent. Hardware/power conditions were not measured; no performance
  qualification is claimed.
- Rust: pinned `1.96.0`, Cargo `1.96.0`, edition 2024; Tauri crate/CLI `2.12.1`,
  tauri-build `2.7.1`; TypeScript `5.9.3`.
- Initial build-host Node was `26.10.0`. Subsequent packaging checks/preparation
  used verified Node `24.21.0` and its npm `11.19.0`, extracted under a temporary
  build-host directory. This is not an installed-user runtime dependency.
- Frozen runtime lock SHA-256:
  `66c429d2a9d33c25fe341721898c7b2195216fba216c19483907c4788794efc8`.
- Last completed task commit: `4a4bb40` (T012). Preceding T011: `9f6c33a`;
  T010: `c50701a`; native-install approval correction: `729300e`.

## Executed evidence

| Check | Observed result | Scope |
|---|---|---|
| Node download | Official archive matched pinned SHA-256; executable reports `v24.21.0` | Build-host artifact, not installed app |
| dsh download | Registry integrity matched research; runtime preparation verified the pinned downloaded tarball | Build-time download integrity, not final signed hashes |
| Production resolution | 608 lock records with exact versions/integrities; `npm ci --omit=dev` installed 538 platform-selected packages | Frozen npm closure; optional other-platform records are not installed |
| npm native scripts | Default npm 11 skipped scripts; project-scoped `--allow-scripts` was rejected; corrected with version-pinned `allowScripts` entries in runtime/package.json, then clean `npm ci` succeeded | Reviewed spawn-helper/node-pty/koffi/protobufjs scripts permitted; unrelated no-op denied |
| CLI identity | Verified Node running installed R's `lib/bin.js --version` reports `0.2.0-rc.2` | Actual R version only; no Web/task acceptance |
| Node packaging tests | 36/36 passed under Node 24.21.0 | Static build/config, inventory and disposable bundle contract fixtures; fake probes are not real installed binary/native-addon evidence |
| Rust library tests | 3/3 passed | Bundle path/version mapping and preferences privacy/atomic failure/schema/bounds |
| Clippy | Library and launcher passed with `-D warnings` | Static native checks |
| Debug launcher build | Succeeded with pinned toolchain/arm64 target | Helper-process execution, not bundled dsh Web |
| Owned helper EOF | Launcher/group exited; separate sentinel remained alive, about 1.04 seconds | Actual spawned disposable shell helper |
| Explicit Stop | Launcher/group exited; sentinel alive, about 1.03 seconds | Actual helper, private stdin pipe |
| SIGTERM | Launcher/group exited; sentinel alive, about 1.04 seconds | Actual helper, signal listener |
| Ignored graceful signal | Forced group cleanup after seven-second bound; sentinel alive, about 7.09 seconds | Actual helper; no user processes targeted |
| T006/T007 red baseline | Missing verifier/library/launcher produced expected failures before implementation | TDD authoring evidence, not passing foundation suite |

Tracked test roots are disposable. No model key or service token/cookie was used
in these checks. The helper timings are observations, not SC release measurements.

## Blocking failure

Running `npm run runtime:prepare` under Node 24.21.0 completed verified downloads
and frozen production install, then failed in:

```sh
cargo build --locked --release --manifest-path src-tauri/Cargo.toml --bin dsh-launcher
```

Rust reported `E0463` (cannot find/load procedural-macro crates):
`zerofrom_derive`, `tokio_macros`, and `futures_macro`. Preparation returned
nonzero and did not generate a successful staged manifest; the subsequent
`runtime:verify` command was not executed. Do not describe staging as passing.

Read-only diagnostics found the three release macro dylibs on disk, identified
as arm64 Mach-O libraries. `otool -L` on the tokio macro showed only libSystem;
`codesign --verify` reported that dylib valid on disk. These observations do
not identify or fix the loader failure. Debug compilation previously succeeded.
No system security controls were disabled and no baseline/toolchain was changed
as a workaround.

Execution stopped at this non-parallel build failure. Next work: reproduce the
release command directly and inspect the Rust/Cargo procedural-macro loading
environment/cache issue; fix and rerun locked release staging/verification before
advancing. Preserve the exact toolchain and runtime locks unless a reviewed
change is necessary. Do not copy a debug binary and claim a successful release.

## Still pending

- Successful release launcher, complete staged/final-signed inventory and actual
  Rust startup bundle verification; actual bundled native-addon loading and
  path-with-spaces runtime execution.
- T013 bounded readiness/one-generation owner and T014 failure/retry transitions;
  T007 integration suite cannot pass before their implementation.
- T022 validated Web overlay and real Web launch/auth/boot/transport/native-frame
  feasibility; no readiness alone is represented as authenticated readiness.
- Installed ad-hoc app/DMG, clean/offline/minimum-OS checks, owner crash and full
  application lifecycle, security/access/network tests, O comparisons,
  accessibility/performance and the >=10-developer study.
- Developer ID/notarization/public-distribution qualification remains deferred,
  separate from the current blocking local build failure.

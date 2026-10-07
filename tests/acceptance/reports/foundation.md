# Foundation evidence (partial; staging restored)

**Status: build-time staging and verification pass; T015 remains incomplete.** T001–T012
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
| Release launcher build | Succeeded after disabling stripping only for host build dependencies, including in a fresh target directory | Same Rust 1.96.0 and locked dependencies; target release optimization unchanged |
| Complete runtime staging | `runtime:prepare` succeeded and generated 26,610 artifact entries | Build-time staging, not final nested/outer signed distribution |
| Staged runtime verification | `runtime:verify` succeeded: dsh `0.2.0-rc.2`, Node `24.21.0`, arm64; complete inventory and dependency resolution checks | Actual staged Node/CLI probes, not installed native-addon/Web acceptance |
| Owned helper EOF | Launcher/group exited; separate sentinel remained alive, about 1.04 seconds | Actual spawned disposable shell helper |
| Explicit Stop | Launcher/group exited; sentinel alive, about 1.03 seconds | Actual helper, private stdin pipe |
| SIGTERM | Launcher/group exited; sentinel alive, about 1.04 seconds | Actual helper, signal listener |
| Ignored graceful signal | Forced group cleanup after seven-second bound; sentinel alive, about 7.09 seconds | Actual helper; no user processes targeted |
| T006/T007 red baseline | Missing verifier/library/launcher produced expected failures before implementation | TDD authoring evidence, not passing foundation suite |

Tracked test roots are disposable. No model key or service token/cookie was used
in these checks. The helper timings are observations, not SC release measurements.

## Resolved release-build failure

The first `runtime:prepare` failed while compiling the release launcher, with
`E0463` for `zerofrom_derive`, `tokio_macros` and `futures_macro`. Execution
initially stopped and did not claim successful staging. Follow-up investigation
reproduced the failure directly, outside npm:

```sh
cargo build --locked --release --manifest-path src-tauri/Cargo.toml --bin dsh-launcher -vv
```

Verbose output exposed the underlying macOS `dlopen` rejection:
`mis-aligned LINKEDIT string pool`, also observed for `serde_derive`. Direct
loading of the four original release macro dylibs failed; their Mach-O
`LC_SYMTAB.stroff` values had remainder 4 modulo 8. Corresponding debug macro
libraries were aligned modulo 8 and loaded successfully. Signatures and CPU
architecture were valid, so signature validity alone did not establish dyld
loadability. The tokio library depended only on libSystem.

A fresh, isolated Cargo target directory reproduced the default release failure,
excluding the existing project cache as the cause. Its host macro compiler
invocation used `-C strip=debuginfo`. Changing only the host build-dependency
strip setting made both the project and fresh-directory release builds succeed:

```sh
cargo --config 'profile.release.build-override.strip="none"' build \
  --locked --release --manifest-path src-tauri/Cargo.toml --bin dsh-launcher
```

The resulting tokio macro library was aligned and loaded successfully. The
workaround is now persisted in `src-tauri/Cargo.toml`:

```toml
[profile.release.build-override]
strip = "none"
```

This changes only host build dependencies (including procedural macros), not
target release optimization or canonical runtime/toolchain identities. The
interaction is localized to link-time stripping of the affected macro libraries
on this macOS 27 host (Apple linker 27037.1, rustc LLVM 22.1.2). A trivial standalone
macro and a temporary copy stripped afterward with `strip -S` both loaded, so
this is not evidence that every stripped library is broken, nor a determination
of which upstream tool component needs a permanent fix.

After persisting the narrow configuration, full `runtime:prepare` and
`runtime:verify` passed under pinned Node/npm, with 26,610 staged artifacts.
The 36 packaging tests, three library tests and library/launcher Clippy checks
were rerun and passed. No security controls were disabled; no debug binary was
substituted, no dependency version changed and no shared cache was deleted.
Other feature tasks were not advanced during this investigation.

## Still pending

- Final-signed inventory and actual Rust startup bundle verification; actual
  bundled native-addon loading and path-with-spaces runtime execution. Successful
  release compilation/staging alone does not qualify these gates.
- T013 bounded readiness/one-generation owner and T014 failure/retry transitions;
  T007 integration suite cannot pass before their implementation.
- T022 validated Web overlay and real Web launch/auth/boot/transport/native-frame
  feasibility; no readiness alone is represented as authenticated readiness.
- Installed ad-hoc app/DMG, clean/offline/minimum-OS checks, owner crash and full
  application lifecycle, security/access/network tests, O comparisons,
  accessibility/performance and the >=10-developer study.
- Developer ID/notarization/public-distribution qualification remains deferred,
  separate from the now-resolved local release-build failure.

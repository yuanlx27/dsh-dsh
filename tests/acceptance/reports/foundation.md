# Foundation evidence

**Status: T015 foundation checkpoint passes on the current arm64 build host.**
T001–T015 are complete. This qualifies bundle/version checks, private preferences,
bounded announcement parsing, native runtime ownership and helper cleanup only.
Actual dsh Web-profile/authentication, native transport and all installed-app/
comparative acceptance remain pending. No candidate .app/DMG is qualified.

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
- Implementation baseline: T013 `13a5319`, T014 `fca6725`, T012 `4a4bb40`;
  release-macro correction `6270c19`. This report and additional actual resource
  probes are committed with T015.
- Current staged manifest SHA-256:
  `129dd05bc0803e2db01a66dfa9370393262ac925a7eda446508d292e0af2d065`.
  Inventory: 26,610 artifacts, including 13 `.node` files and 12 internal symlinks;
  538 installed production packages, approximately 530 MiB staged dsh tree.
  This identifies build-time bytes, not final signed distribution.

## Executed evidence

| Check | Observed result | Scope |
|---|---|---|
| Node download | Official archive matched pinned SHA-256; executable reports `v24.21.0` | Build-host artifact, not installed app |
| dsh download | Registry integrity matched research; runtime preparation verified the pinned downloaded tarball | Build-time download integrity, not final signed hashes |
| Production resolution | 608 lock records with exact versions/integrities; `npm ci --omit=dev` installed 538 platform-selected packages | Frozen npm closure; optional other-platform records are not installed |
| npm native scripts | Default npm 11 skipped scripts; project-scoped `--allow-scripts` was rejected; corrected with version-pinned `allowScripts` entries in runtime/package.json, then clean `npm ci` succeeded | Reviewed spawn-helper/node-pty/koffi/protobufjs scripts permitted; unrelated no-op denied |
| CLI identity | Verified Node running installed R's `lib/bin.js --version` reports `0.2.0-rc.2` | Actual R version only; no Web/task acceptance |
| Node packaging tests | 37/37 passed under Node 24.21.0 | Static build/config/inventory, disposable bundle fixtures and actual bundled native-entry/flock probe; not installed WKWebView evidence |
| Rust library tests | 10/10 passed in both debug and release profiles | Preferences, errors, environment, coalesced startup, auth-phase deadline, token revocation, generation-safe failure and conservative cleanup/retry |
| Foundation integration tests | 8/8 passed in both debug and release profiles | T007 schema/atomic write/bounds/parser/timeout/EOF/owned helper cases plus actual native bundle verification |
| Native bundle gate | Verified actual staged inventory, frozen production versions, bundled Node/CLI and target; copied binaries in a path containing spaces passed native installed-layout mapping; corrupted launcher rejected | Resources shared through a fixture directory alias; not a signed installed .app |
| Native addons | Bundled Node loaded node-pty, sharp and koffi native entries; upstream system/flock acquired a disposable file lock | Actual build-time production resources; not every optional native module or minimum OS |
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

## Runtime owner checkpoint

Concurrent starts join one service generation, with announcement distinct from
successful native auth/boot completion. A 15-second deadline remains active after
the announcement; deterministic clock advancement verifies failure, token
revocation, rejected late completion and explicit cleanup-before-retry. Model-key,
Node injection and known telemetry environment variables are excluded without
removing ordinary PATH/HOME/SHELL/SSH/Git command environment.

Unexpected normal service exit is reported only after the launcher confirms group
cleanup (private exit code 2). Launcher signal death or unproven cleanup retains a
failed ownership guard and blocks both competing startup and successful quit.
The signal-death regression first failed against T013, then passed after T014.
A native generation-scoped failure entry point covers later auth/boot errors;
cleanup precedes retry and a late failure cannot invalidate a newer generation.
Raw child stdout/stderr are discarded without copying them into shell diagnostics;
Startup token data has no Debug/Serialize implementation and failure categories
contain fixed explanations/actions only. Owner tests use disposable helper output,
not a claim that dsh Web authentication has already passed.

The full integration resource test took about 41 seconds in the unoptimized test
profile (two full valid native gates plus corruption rejection). This is not an
SC-002 measurement. The complete eight-test integration suite passed in about
4.18 seconds in the optimized release profile against the final staged manifest.
This still excludes Web startup/auth/UI; installed SC-002 performance must be
measured separately.
Initial ad-hoc addon probe attempts had a JavaScript require-shadowing error and
used an unexported system package root. They were corrected to the documented
system/flock entry and an actual native lock operation; the successful form is
now a tracked packaging regression test, not an upstream defect claim.

No unresolved version/ownership failure remains at this foundation checkpoint.
Missing future Web/native-frame evidence remains explicitly pending, not passing.

## Still pending

- Final-signed inventory, complete installed-app native-addon loading and actual
  installed path-with-spaces execution. Build-time/native-layout probes alone do
  not qualify these distribution gates.
- T022 validated Web overlay and real Web launch/auth/boot/transport/native-frame
  feasibility; no readiness alone is represented as authenticated readiness.
- Installed ad-hoc app/DMG, clean/offline/minimum-OS checks, owner crash and full
  application lifecycle, security/access/network tests, O comparisons,
  accessibility/performance and the >=10-developer study.
- Developer ID/notarization/public-distribution qualification remains deferred,
  separate from the now-resolved local release-build failure.

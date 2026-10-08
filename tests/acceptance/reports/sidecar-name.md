# Desktop build prerequisite: sidecar/resource collision

T024's full desktop-feature library check revealed a concrete tauri-build
collision: it copies sidecars and resources into the same development target
directory. `binaries/dsh` became the file `target/<triple>/debug/dsh`, conflicting
with resource mapping `dsh/`; copy_resources failed ENOTDIR.

Renamed only the owned launcher sidecar to `dsh-launcher`, consistently across
staging, inventory/verifier allowlists, native staged/installed paths, config and
fixtures/contracts. Node, R package name/CLI/runtime resource directory and all
product versions stay unchanged. Existing Cargo launcher is already named
`dsh-launcher`, so no second binary or wrapper is added.

A new basename/resource collision assertion failed before the fix. After
renaming the actual staged launcher and regenerating its inventory, checks pass:

- Actual runtime verification and all 8 native foundation tests, including real
  installed/space-containing path fixture and owned-helper cleanup.
- All 34 selected packaging/config/inventory tests.
- Full Node suite: 49 pass / 1 missing-T027-module failure.
- Actual `cargo check --locked --offline --features desktop --lib`.

The known old generated dev `debug/dsh` file was removed (only that stale file,
not a resource tree); existing checkouts with the old dev artifact need the same
cleanup or `cargo clean` before rebuilding. Other user data is untouched.

The application-bin check now reaches generate_context, then fails on the pending
`src-tauri/icons/icon.png` deliverable. No full executable, installed app, signing,
DMG or native message qualification is claimed. Tauri's dev resource copy also
is not an installed symlink-preservation acceptance result; verify actual bundle
inventory/layout during packaging/native integration.

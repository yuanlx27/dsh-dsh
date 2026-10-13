# T052 local ad-hoc distribution / T053 partial cases

## Maintained procedure, executed

`npm run package:macos -- --output ABSOLUTE_NEW_DIRECTORY` now stages the frozen
runtime, verifies T009, builds current source, copies the owned build into a NEW
output, restores frozen internal links, signs **17 nested Mach-O files**, uses
T009's **same inventory implementation** to record **26,761 signed artifact
hashes**, signs outer .app and verifies deep/strict plus every nested signature.
It checks canonical header/dsh identity, arm64 Node/launcher, generated numeric
Apple short version **0.2.0**, build **1**, bundle identifier/minimum **14.0**.
Final resource/manifest equality is read-checked; neither is edited after outer
signing. Only distribution metadata outside .app is written afterwards.

`src-tauri/entitlements.plist` requests **no additional entitlements** for local
ad-hoc/non-hardened mode. This is not public hardened-runtime/Node JIT
qualification; Developer ID/signing credentials, notarization/stapling and
Gatekeeper are deferred and must use untracked inputs when later authorized.
The local command never asks for those credentials.

Tauri itself changes the two copied sidecar signatures. The first strict
pre-sign execution correctly refused that byte mismatch. The maintained tool
allows only its **own known build producer's two verified sidecar signatures**
to reconcile before final inventory; default standalone `finalizeApp` rejects
unapproved sidecar changes. All other closure hashes/membership remain strict.
No arbitrary app-input CLI, disabled integrity check or broad Mach-O hash exemption.
Ad-hoc signatures demonstrate consistency, not source authenticity against a
malicious same-user build/dependency replacement.

## DMG and native installed proof

Creates .app plus compressed DMG/Applications link; verifies image checksum,
mounts **read-only/nobrowse** at an owned temporary mount, verifies deep/strict
signatures and uses the production Rust `BundlePaths::installed` verifier on both
output .app and mounted .app. On both, installed Node executes version checks and
loads existing native addon fixture: node-pty, sharp, Koffi and actual disposable
system flock. Probe env is minimal, no host Node/npm/keys/config; HOME/DSH_HOME
and the lock file are private temporary resources. Reinventory after probes
confirms no sealed-resource mutation. No GUI/backend/Host/supplier was started.

The successfully tested output has spaces in parent/output/app/image paths.
Current retained candidate:
`/private/tmp/dsh local delivery.7YiYXl/distribution with spaces/DeepSeek Harness Desktop.app`

DMG SHA-256:
`d6982e1b0af598fe38d2b9f27524cea0069bad5d20aaf29a5c63edb0b34319ff`

Private path pointer: `/private/tmp/dsh-package-sweep-output.txt`.
`distribution.json` pins paths/runtime/build/artifact count/image hash outside
sealed app. Executed from 2117c8c plus working packaging code; compiled Rust
includes **8694388** pending-cleanup/quit fix (old b8169e0 acceptance app does not).
It is not falsely attributed to a subsequent clean commit. No installed GUI
race/behavior retest yet. Initial failed output was retained, not called a release.
Successful image mounts were detached/owned staging removed; filtered hdiutil
inspection finds no owned verification mount. On macOS 27, hdiutil warns of
new diskutil alternatives; retain the existing minimum-macOS-14-compatible tool,
not a macOS-27-only packaging dependency.

If an operation fails, output is **incomplete/unqualified** and not implicitly
reused/overwritten. If native detach fails, preserve owned output/mount rather
than recursively delete unconfirmed mounted resources; investigate exact owned
mount, never generic volumes. Do not install incomplete outputs.

## Regression scope

Ten `tests/packaging/distribution.test.ts` portable pipeline/path cases run with
an explicit **fake native signer/metadata adapter**, not forged native results:

- Target-suffixed canonical inventory maps unsuffixed installed filenames;
  unsafe/wrong-target paths refuse.
- Nested sign → final hashes → outer sign ordering; source link identity.
- Complete prerelease mismatch, wrong architecture/numeric build refuse.
- Missing, corrupt or unlisted closure refuse before signing.
- Outer signature cannot mutate inventoried resources undetected.
- Tauri-flattened link restoration stays frozen/internal.
- Unapproved sidecar mutation refuses; own build signatures are verified before
  narrow reconciliation and again after final signing.
- Read-only help/unknown option cannot initiate build.

All **107 packaging tests**, source policy/strict tooling+browser and desktop
all-target Clippy warnings-denied pass. Existing T009 symlink/inventory tests
still pass with the optional native layout mapper; no second inventory schema.

T052 procedure is complete for **local ad-hoc** scope. T053 remains **partial**:
portable negative cases and actual installed closure/native loads are available,
but no new installed GUI/About/complete startup download observation claimed.
Those native/traffic/minimum-OS/clean-account/accessibility/final release gates
remain open T032/T033/T055/T056/T058/T059. Public distribution is not qualified.

# Official immutable reference — executed unit evidence (T047)

Reference-only checkout **O**:
`5badb15009ae1756c3afe0ae0cef1faafc290ccc`, desktop/root **0.2.1-alpha.1**.
Created a detached shallow fetch from the public official repository, verified
HEAD before/after, and observed clean tracked `git status --short` after install,
native build and tests. No O source or lockfile was modified; product's shipped
runtime remains pinned R 0.2.0-rc.2.

Disposable checkout: `/private/tmp/dsh official reference.FuLQr3/source`, with
separate private HOME, pnpm tools/store. Minimal environment excludes real model
keys and user's normal configuration. No official GUI or model task was started.
Host arm64 macOS **27.0.1 / 26A434**, Node **24.21.0**, exact private **pnpm 11.7.0**,
Vitest **4.1.8**. Global pnpm 12.11.2 was not substituted.

Executed `pnpm install --frozen-lockfile`: pass. Lock SHA256:
`2893d71a9ef07d2d72d47b2199e581b9e8d90638a15ee151908eebf69dfc637c`.

Initial native build with the standalone copied Node executable failed because
its computed installation prefix lacked Node-API headers. Kept this prerequisite
failure, then reused the existing complete verified Node 24.21.0 installation
under ignored `.cache/runtime`, with headers, without patching O. Executed
`pnpm run build:native-system`: pass, built **darwin-arm64/bin/system.node**.
Addon SHA256:
`d14698a8dda5a8eb6cbc97b309d6c389f28c695f79192cb825c2197dce310220`.
This addon is a reference-build artifact, not a shipped product runtime upgrade.

Re-ran exactly the five quickstart-prescribed suites **after** successful native
build, using O's unmodified Vitest configuration:

| File under apps/desktop/tests | Passed |
|---|---:|
| backend-controller.spec.ts | 9 |
| quit-confirmation.spec.ts | 12 |
| keyboard.spec.ts | 38 |
| fatal-recovery.spec.ts | 22 |
| desktop-build-version.spec.ts | 20 |
| **Total** | **101 / 101; five files** |

Exit zero, no skipped tests in this selection. Vite emitted a non-failing
vite-tsconfig-paths deprecation warning. The earlier pre-native-build selection
also passed but is not the final prerequisite-qualified run. Test durations are
not native performance metrics. No entire-upstream-suite claim is made.

These execute reference controller/quit/keyboard/recovery/version tests and
support the [A1–A6 protocol](../official-comparison.md). They use deterministic
unit seams; they do **not** prove native installed Electron windows, model tasks,
real cookie/stream boundaries, zero-window dialogs, menu interception, Gatekeeper
or product WK behavior. No official arm64 .app/DMG was built/launched or associated
with O during this run. Native official artifact provenance and T048/T049
comparisons remain pending, rather than passing by source inspection or test count.

T047's prescribed immutable checkout, dependency/native preparation, selected
tests and limitation recording are executed. Missing native reference evidence
still blocks alignment-complete/SC-008; T045/T051 preparation/review cannot waive it.
No secrets, raw user/model/session reports or authenticated URLs were retained.

## Subsequent native artifact preflight (T048/T049 prerequisite)

Exact-O mac-arm64 packaging `--check` was attempted in its separate HOME/tool
environment and fails **missing `.env.macos`** before build/signing. Source
validation requires file-owned release settings, signing identity/team/P12/password
and a notarization strategy. `--unsigned --check` actually refuses:
**`--unsigned requires win-x64`**. No fake credentials, source patch, preflight
bypass, keychain modification or release operation. This prescribed reference
packaging route needs unavailable external inputs; the product's independent
local ad-hoc T052 remains complete.

Private pnpm first attempted a dependency-status reinstall (missing pnpm PATH,
then refused non-TTY purge); no reinstall/source mutation accepted. Direct
existing **root tsx** safely executes the same immutable preflight without that
wrapper. Tracked O tree remains clean; no official app/backend launched.
Public GitHub releases queried include `dsh-v0.2.1-alpha.1` with **zero assets**;
this bounded acquisition route supplies no verified exact-O native artifact.
Other acquisition is not claimed impossible, but cannot be invented or replaced
silently with another commit/unpackaged development shell.

T048/T049 affected installed portions stay skipped/open pending a trusted exact-O
artifact or supported prerequisites. Continue independent product work rather
than request real secrets or infer unit/native parity.

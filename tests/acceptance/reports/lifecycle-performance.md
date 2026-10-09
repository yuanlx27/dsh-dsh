# T033 — native lifecycle/performance observations (PENDING)

Host: **trochili.local**, arm64, macOS 27.0.1 build 26A434. Checkpoint `d1bda7b`.
Production release binary in a locally ad-hoc-signed, deep/strict verified
installed copy with spaces: `/private/tmp/dsh release install.JMNtfi/DeepSeek Harness Desktop.app`.
Private standard HOME: `/private/tmp/dsh-release-home.j6dmz2`; PATH `/usr/bin:/bin`.
No debug fixture consent, logging, root override or auto-entry enabled. Current
native notice was explicitly acknowledged through UI before these trials.
This is not clean-account, macOS 14, DMG or public Gatekeeper qualification.

## Executed baseline

`lifecycle-performance-data.json` retains 20 curated rows: public owned PID
identities, monotonic elapsed milliseconds and owned cleanup outcomes. No
service port/token/cookie, request/response body, credential or task log.

- **20/20 empty-key baseline launches: 8,235–8,689 ms** to an actually rendered
  composer with a verified unsent fixture draft entered via focused native AX
  keyboard input. Each launch used the same saved isolated HOME.
- Every trial explicitly entered Harness through the UI and chose Configure
  later where R requested a key. No prompt was submitted to a provider.
- **20/20 explicit native Stop and Quit actions**: owned app, launcher and Node
  disappeared. No unknown child PID was killed; cleanup came from native quit.
- Installed debug candidate separately completed **10 actual red-close/File/Open
  cycles**: hidden native AX windows zero, exact unsent draft preserved, same
  launcher/Node IDs; default Return/Stay preserved the process trio, confirmed
  quit cleaned it. See `us1.md`. No task/approval was running in those cycles.

**These are not SC-002 passes.** SC-002 explicitly requires a saved *valid*
configured credential, accessible task-ready workspace, excludes setup/user input
and requires the named otherwise-idle release environment. This baseline has no
valid key, includes automated Open/Configure later/draft actions, and has no idle
host qualification. It establishes local release rendering/connection/cleanup,
not receipt acknowledgement, provider availability or task-readiness.

**These are not full SC-007 passes.** Running tasks/pending approvals, selected
conversation/scroll, live transfer/document identities and Stay in every work
state are still missing. Counting twenty no-task quits does not satisfy those
work-state requirements.

## Observation corrections, not product fixes

Temporary TypeScript observer `/private/tmp/dsh-native-lifecycle.ts` used standard
Node subprocess/timer APIs and `/usr/bin/osascript` only. It never injected a JS
program into the WebView or used general Tauri invoke. It did not inspect secret
values. Earlier unqualified pilots are retained outside Git under
`/private/tmp/dsh-t033-{ax,inactive-display,renewed-display}-pilot.*`.

An initial structural AX selector/early input observer failed to verify drafts.
Actual screenshot showed Cmd+A selecting the whole page: AX click alone had not
focused the existing nonempty composer. Explicit native AXFocused plus a brief
input-settling interval fixed the observer; the two corrected pilot rows were
8,396/8,358 ms, then a complete new twenty-row baseline was run. Null pilot rows
were never relabelled usable and are not evidence of a confirmed app render
failure. Some diagnostic trials also encountered an inactive display; bounded
native caffeinate assertions were renewed per trial without bypassing lock or
permission checks. The final complete dataset uses the corrected observer.

## Pending

Valid controlled model credential/setup, idle-host verification, process-start
and visible/task-ready receiving-boundary instrumentation without setup input,
all running/approval/hidden/transfer/selection/scroll cases, native Stay coverage,
minimum-OS and other release gates. **T033 remains unchecked.**

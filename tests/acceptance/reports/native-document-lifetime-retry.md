# T032 — actual destroyed-document HTTP retry (partial F09)

**Subsequent fix:** `native-implicit-exit.md`/data.json records removal of the
automatic last-window quit prompt and a fresh installed actual-WK regression.
The pre-fix observations below remain historical evidence, not current behavior.

After the user restored Accessibility and authorized continued work, System
Events reported UI access enabled and read AXWindow successfully. This retry used
the existing signed/verified installed debug candidate
`/private/tmp/dsh stale install.OAYll0/DeepSeek Harness Desktop.app`, native
implementation checkpoint `b79ced6`. Deep/strict codesign passed again. Exact
binary/manifest identities and closed rows are in
`native-document-lifetime-data.json`. Host: `trochili.local`, arm64,
macOS 27.0.1 build 26A434. This is not installed production/minimum-OS qualification.

## Corrected fixture, unchanged authority

The earlier prepared program had two fixture defects, not product defects:

- WK promise replies are JSON strings (`src/wk.ts` already documents this); the
  handle checker initially received the string instead of a decoded object.
  Added standard JSON parsing, with a source regression RED then GREEN.
- Dispatcher intentionally admits only one physical mux per document. R already
  owns it; the attempted extra begin returned Desktop request denied. Do not add
  another carrier, weaken the singleton or call that expected refusal a stale
  handle pass. The corrected standalone program now tests **HTTP only**.

The failed preparations buffered harmless HTTP jobs but did not finish/send them,
retain a usable output or submit any model/task/tool/provider operation. They are
not passing rows. Native destruction subsequently revoked the old document scope;
this report does not independently count every pilot job's release.

The typed program is compiled outside the repository and pasted into the owned
Inspector, with its actual Console prompt explicitly focused. Only opaque HTTP
handle/generation metadata is temporarily held in a distinct localStorage fixture
key; bootstrap values and authentication are discarded. Closed output contains
only numeric generations and fixed scenario/outcome strings, never handles,
headers, settings, raw exceptions or request/reply bodies. The fixture storage
key is removed after the replacement phase.

## Actual destruction and recreation

Original entered window generation **2**, runtime generation **1**. HTTP begin
created a real native buffered handle for GET `/api/_desktop_fixture_lifetime`;
no HTTP request is dispatched by begin. Preparation result was retained.

Host SIGUSR1 requested destruction of the owned native primary. The original
primary/Inspector disappeared; AX then exposed only the native **Stop the service
and quit?** confirmation. This is not a renderer reload/generation mutation.
**Unexpected automatic exit confirmation after last-window destruction was
observed.** The user-visible path was explicitly Stay, File/Open Workspace Window,
then the recreated shell's Open Harness. It was not prompt-free automatic recovery
and remains a lifecycle behavior to investigate, not silently waived as ideal UX.
A probe pasted in the intermediate shell correctly refused to run; that pilot is
not replacement-Harness evidence.

After entering the recreated document, its window generation was **4**, runtime
still **1**. Captured app/launcher/Node **60485/60551/60552** and their installed
executable identities stayed unchanged throughout destruction and recreation.
The real old HTTP handle was retrieved from fixture metadata and submitted through
the **fresh current sender**, not forged as a random unknown handle.

| Replacement scenario | Actual outcome |
| --- | --- |
| Actual window generation changed | Confirmed |
| Runtime generation retained | Confirmed |
| Fresh main bootstrap | Resolved |
| Actual previous generation 2 | Desktop denied |
| Old HTTP handle finish | Desktop denied |
| Old HTTP handle write | Desktop denied |
| Fresh HTTP begin/cancel | Resolved |
| Fresh main bootstrap after negatives | Resolved |

The three refusals are explicit native errors, not 1.5-second observation timeouts.
This qualifies actual old-generation/HTTP-handle refusal across this destruction
and manual recreation path. It does not demonstrate an HTTP download, old reply
arrival at a controlled peer, pending mux callback ordering, carrier/task identity
retention or every late-completion race. No stale finish successfully dispatched.

## Cleanup and checks

Explicit native Stop and Quit cleaned the complete captured trio. Owned display
assertion stopped; generated clipboard cleared only after exact equality check.
AX dumps remain untracked/private. 73 packaging tests, source policy and strict
Node/browser tooling pass. Native authority code did not change in this retry.

**T032/T033 remain unchecked.** Stream/late-reply F09, other native views/origins,
remaining credential/admission/platform gates, saved-valid idle-host timing and
substantive work-state lifecycle remain pending. `native-document-lifetime.md`
preserves the earlier blocked pilot; its original extra-mux recipe is superseded
by the corrected HTTP-only program and this actual partial result.

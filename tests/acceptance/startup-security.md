# Native startup and security acceptance (T019)

**Status: procedure defined; not executed.** Execute with T032's implemented,
locally installed candidate. Rust/Node fixtures, browser-only runs, source
inspection and successful development builds do not pass this procedure.

Read [fixtures](fixtures.md), [acceptance mapping](README.md), the
[runtime contract](../../specs/001-deepseek-harness-desktop/contracts/runtime-bundle.md)
and [lifecycle contract](../../specs/001-deepseek-harness-desktop/contracts/desktop-lifecycle.md)
first. R is `@deepseek-ai/dsh@0.2.0-rc.2`, revision
`639ed015397290b3745d163aafe02ffee4aa3f84`; O is reference-only.

## Preconditions and safety

1. Use an installed `.app` from the candidate DMG, local ad-hoc signing, a
   dedicated macOS test account and disposable dsh/workspace roots. Never change
   application-support permissions, credentials or projects in the daily-use
   account. Save/restore fixture permission modes; do not bypass OS security.
2. Record actual machine/OS, build provenance and final installed manifest hash.
   Test the named arm64 host and a separate macOS 14 Apple Silicon runner. An
   unavailable runner stays pending. Local results do not qualify Developer ID,
   notarization, hardened-runtime/JIT or public Gatekeeper distribution.
3. Use fixture credentials supplied through untracked inputs and an approved
   bounded model-call budget. Never record keys, launch-token URLs, Cookie or
   Set-Cookie values, signing secrets, raw network bodies or dsh output. Do not
   put credentials in command-line arguments or the shell's inherited environment.
4. Prepare an accessible project, read-only project, missing-directory case,
   existing history and a controlled approval action. Use upstream-supported
   configuration/records; do not fabricate task or credential-source schemas.
5. Native qualification instrumentation must observe actual WKScriptMessage
   WebView identity, frame/main-frame flag and security origin. Renderer-supplied
   labels are adversarial inputs, never the observation source. Allow only
   native-side test controls; no production renderer command may create an
   alternate authorized view or select a service destination.
6. Observe native dispatch counts, opaque request/stream handle lifecycle,
   window/document/runtime generations and owned process exit. Record hashes,
   counts, lengths and sanitized outcomes, not payloads. Do not broadcast replies
   or expose native auth for testing. No authenticated-localhost fallback.
7. Any necessary negative-case instrumentation is identified by its exact build
   difference. Re-run normal/security flows on the final installed distribution;
   instrumentation-only success is not final-candidate evidence.

## Execution and expected results

Run each row with a fresh fixture or document exactly what persisted. A failing
upstream requirement is release-blocking, not permission to add a shell-owned
credential store, workspace preflight, task gate or second engine.

### Startup, safety and independence of readiness

| ID | Procedure | Required observation |
|---|---|---|
| N01 | Launch the fresh installed app without host Node/npm/dsh available on PATH. Leave the current safety notice unacknowledged. | One owned bundled service at most; setup/starting state is truthful. No task-capable main frame or authenticated bridge admission before a successfully saved current notice. Notice text describes local command/file access and confirmation responsibilities. |
| N02 | Acknowledge using keyboard; confirm focus/status with VoiceOver. Relaunch normally, then repeat with an obsolete saved notice revision. | Current revision is atomically saved in shell preferences; normal relaunch does not ask again. Obsolete revision requires acknowledgement again. No Harness/model state is copied into preferences. |
| N03 | In the dedicated account, deny creation of the app-data root by removing parent write access while the root is absent; separately make preference replacement fail. Try launch/acknowledgement. Restore original modes afterward. | Fixed corrective error, no false successful acknowledgement or entry; no competing service or damaged old preferences. A denied write is not bypassed by continuing without saving. |
| N04 | Start with no model credential and no valid selected workspace after native connection/notice are available. | Upstream settings and available saved history remain usable; dependent task submission is unavailable. A service announcement/native authentication alone does not assert task readiness. |
| N05 | Provide the fixture credential and accessible workspace through upstream settings. Select the workspace; repeat with a deliberately rejected credential. | Valid prerequisites permit input without a separate shell online-validation step. Actual credential rejection gives a corrective upstream state. Credential presence is not a guarantee of future validity. Runtime/readiness and task state remain independent. |
| N06 | Exercise an invalid announcement, slow private exchange/boot, early service exit and native connection failure in controlled candidate fixtures. | Total startup deadline includes auth/boot and is 15 seconds; fixed category/action only. Generation is revoked and owned cleanup precedes explicit Retry. No automatic restart, task replay or raw dsh diagnostics. |
| N07 | Activate the installed application repeatedly during starting and ready states. | Startup joins one owned service; ready activation shows the retained primary window. No second service or competing generation. |

### Clean, offline and occupied-port launch

| ID | Procedure | Required observation |
|---|---|---|
| B01 | Install/run under an application path containing spaces with no system runtime tools available. Verify the installed immutable resources. | Exact desktop/dsh prerelease equality, bundled Node/arm64 and native addons; no launch-time dependency fetch. A missing/corrupt/version-mismatched resource blocks unsafe entry with a fixed explanation. |
| B02 | Launch with network unavailable, without asking for model inference. Repeat with saved current notice, credential and workspace. | Bundled service/auth/UI/settings/history initialize locally without downloads, analytics, feedback or updater traffic. Offline model-service failure is not confused with local runtime/auth failure; no remote response is required to prove local UI startup. |
| B03 | Start a fixture-owned sentinel listener on loopback port 3080, then launch the app. Close the fixture sentinel afterward. | App uses its own randomly assigned port, does not adopt/contact/kill the sentinel, and works normally. Record listener/owned-process identity only, never a token URL. |
| B04 | Observe startup/idle traffic with no model request or approved network command. | No shell telemetry/updater/reporting/plugin-install destinations. Additional traffic is explained against FR-017; unexplained destinations fail qualification. |

### Credentials and workspace behavior (upstream-owned)

| ID | Procedure | Required observation |
|---|---|---|
| C01 | Enter a fixture DeepSeek credential via upstream settings. Leave the edit view and relaunch. Inspect routine settings concealment and dsh-owned storage modes. | Credential persists upstream, is concealed in routine views and stored at `0600`; app-data root is `0700`. No second shell store, saved-key insertion into conversations or shell-authored secret diagnostics. |
| C02 | Make the disposable credentials file group/other readable and relaunch; restore `0600` and retry. | Upstream rejects overly permissive storage. The shell presents fixed corrective failure/setup rather than rewriting or silently accepting it. |
| C03 | Use a controlled model fixture to pause a request after its credential was resolved. Replace the credential; allow the first request to finish, then issue another dependent call, including one later in the same task. | Already-started work is not cancelled or claimed revoked. Subsequent calls resolve the new upstream credential per operation; readiness reflects upstream availability. Record credential *fixture IDs* seen by the controlled endpoint, not values. |
| C04 | Repeat C03 with removal. Attempt new dependent work and a later model call in an existing task; retain access to settings/history. | Removal blocks subsequent dependent calls/new work but does not cancel already-started model requests, commands or file changes or relabel a running task as idle. Missing-credential outcomes are visible. |
| C05 | Configure a read-only credential source using supported R configuration. Attempt removal of its supplied credential in settings. | Refusal is presented as refusal, not successful removal; availability remains truthful. If R cannot satisfy this requirement, record a release-blocking upstream gap with provenance. |
| W01 | Add/select an accessible disposable workspace via upstream UI. Switch to another prepared workspace. | Active workspace is visible; settings/history/task policies stay upstream-owned, with no new shell confirmation or preflight. |
| W02 | Select a readable but non-writable project and request bounded read-only analysis. Separately request a controlled write under upstream approval policy. | Lack of write permission alone does not prohibit read-only analysis. Required write failures follow upstream/OS handling; no write success is falsely reported and no shell-wide permission scan is added. |
| W03 | Remove/make inaccessible a selected fixture directory; inspect saved history and attempt dependent new work. Restore it and retry through upstream UI. | Available history stays independent of directory access. Missing/inaccessible prerequisites prevent dependent work according to upstream; no reassociation, history migration or shell admission policy. |

Routine model-key concealment is distinct from FR-025's absolute prohibition on
renderer-visible **service** auth. A key intentionally entered in upstream
settings is not proof of shell injection. Arbitrary upstream/model/tool output
is not promised blanket redaction and must not be rewritten for this test.

### Native frame, destination and generation boundary

Use a harmless, already verified service operation. Observe the actual native
sender metadata and count backend dispatches. Denial means no authenticated
service request, reply bytes or usable handle, not merely a UI error.

| ID | Attempt | Required observation |
|---|---|---|
| F01 | Current owned primary main frame at `dsh-app://app`, after current notice and runtime/boot readiness. | Supported authenticated requests work with private, frame/window-scoped ordered replies. No general filesystem/shell/network plugin authority is granted. |
| F02 | Primary main frame before readiness or saved current acknowledgement; local startup/failure/safety frame at `dsh-app://shell`. | No Harness transport/bootstrap authority. Shell status/retry/acknowledgement commands are separately caller-validated. |
| F03 | Child frames: same-app URL, same-origin `about:blank`/srcdoc, sandboxed preview, blob/data and external origins where permitted by CSP. Send a direct native bridge message and counterfeit main-frame labels. | Child-frame native metadata is denied regardless of labels. Main-frame-only boot never supplies child frames usable authority. A CSP-blocked frame is recorded as blocked, not substituted for testing permitted child types. |
| F04 | Another native window/WebView at the exact app URL and an external top-level page. Attempt bootstrap, HTTP and live stream. | Only the current primary WebView/main frame is accepted; matching origin/labels in another window are insufficient. |
| F05 | Counterfeit origin/window/runtime fields in every renderer request; try `dsh-app://shell`, other authorities and user-info/port variants. | Native WK metadata and current generations decide. Custom-scheme URL `.origin == "null"` equality cannot grant authority. |
| F06 | Absolute/authority-changing paths, other loopback port, external URL, traversal/encoded traversal, backslash/control/fragment/malformed targets and encoded token query. | Rejected before dispatch. Caller cannot choose the native socket target or reintroduce launch-token exchange. |
| F07 | Inject Cookie/Authorization/Host/Origin/Fetch-Metadata, proxy/forwarded headers and Connection-nominated hop headers. Return Set-Cookie/hop headers from a controlled service response. | Native fixed-service auth/trust only; caller trust/auth/hop fields removed and response auth/connection fields absent. No auth values appear in replies/errors. |
| F08 | Controlled redirects during private exchange, authenticated boot, HTTP and stream handshake, including another owned-port path and an external endpoint. | Only R's expected private 303-to-clean-root exchange is processed explicitly. Other redirects fail closed; no redirected endpoint receives credentials or replay. |
| F09 | Retain opaque handles/callbacks, then destroy/replace/fail the document. Recreate the primary frame and submit old chunks/replies/handles. | A fresh window generation is required; old state is denied before the replacement receives authority. Renderer transfers cancel, not upstream tasks/approvals. |
| F10 | Invalidate/stop/fail the runtime with transfers outstanding; retry explicitly after confirmed cleanup and deliver old responses. | Native cookie/handles are discarded. Old runtime generations cannot dispatch or deliver bytes, even if an old numeric port or window identity is reused. |
| F11 | Hide normally via close/Command+W, then show via Dock/menu; repeat while uploading/downloading/streaming. | Same native document/window/runtime/transport identities remain; normal hide does not revoke handles or Stop/replay work. Actual destruction is tested separately. |
| F12 | Invoke Quit from menu/Command+Q while visible/hidden, repeat it, Stay/dismiss/fail the dialog, then confirm. | One ownerless native decision, usable without a visible primary window. Cancellation preserves service/document; confirmation revokes generations and waits for actual owned exit. DOM retention and quantitative cycles are additionally T033. |

Record blocked and rejected attempts separately. If WKWebView cannot establish
the required native sender/admission facts or cannot securely mediate a required
transport, stop qualification. Never expose cookies, accept labels or navigate
to an authenticated localhost page as a workaround.

### HTTP, streaming and cancellation fidelity

1. Compare deterministic binary downloads and multipart uploads against
   standalone pinned R using byte counts and cryptographic digests. Include
   NUL/non-UTF-8 bytes, multi-chunk payloads and original multipart boundaries.
   Responses remain streamed through the bridge, not the scheme byte responder.
2. Exercise the exact 300 MiB buffered input limit and an oversized attempt.
   The latter is rejected before backend dispatch; a failed/partial body is not
   silently forwarded as a truncated request. Observe bounded queues with a
   slow consumer and record high-water counts/bytes, not contents.
3. Exercise R's `/api/remote.mux` with multiple logical stream IDs, opaque
   downlink/uplink frames, equal consecutive items, half-close, failure and
   cancellation. Verify ordering and multiplex isolation using fixture counters.
   Compare against the upstream protocol, not an invented task-wire schema.
4. Abort before dispatch, during request headers, upload, download and live
   stream. Include an uplink blocked in `next()` and queued late replies. Confirm
   transfer resources release, stale bytes are not delivered and unrelated
   streams remain usable. Cancellation is not rollback or automatic task Stop.
5. Force a carrier disconnect and let upstream Connection choose its recovery.
   The adapter does not independently retry/replay a submission/approval. Only
   an explicit subsequent upstream request causes a new operation. Record
   dispatch counts and opaque fixture IDs.
6. Verify same-app raw Fetch upload/download consumers use mediation and
   external Fetch remains outside it. Global WebSocket is not replaced and
   renderer networking never receives native credentials.

### Renderer secret absence and unauthenticated clients (SC-010)

1. Inspect the actual main frame, permitted child frames and denied windows:
   document/navigation/history URLs, DOM and boot globals, hook/handle metadata,
   local/session storage, IndexedDB, cookies, console/error surfaces and
   renderer-visible bridge request/reply headers. Inspect shell-native diagnostics
   separately. No service token, cookie or authenticated localhost navigation may
   exist there; no service cookie may be placed in WKWebView's cookie store.
2. Compare against the current ephemeral native auth values **only in a
   controlled native test process**, returning boolean/count evidence. Never
   export those values to a renderer probe or tracked report. Renderer requests
   cannot learn the owned native origin through a `streamBaseUrl` bootstrap field.
3. From a separate unauthenticated local client, GET the real service root/index
   and valid protected session-read, task-submit and approval-resolve requests.
   Derive each exact method/route and non-secret fixture payload from a successful
   upstream flow; do not count an invented route's 404/invalid-body error as an
   auth denial. No Cookie/token/auth header is supplied.
4. All protected operations are denied by authentication/admission, disclose no
   session/configuration/workspace/approval data and have no side effects. Index
   and per-launch boot are authenticated. Public non-sensitive static asset
   success is permitted and is recorded separately, never mistaken for API access.
5. Attempt an unauthenticated WebSocket handshake at the actual
   `/api/remote.mux`; opening a usable stream or receiving protected data fails.
   Record sanitized handshake rejection and dispatch counts, not raw bodies.
6. From another computer on the test network, connect to the host's LAN address
   and current service port. Direct TCP/HTTP/WebSocket access must be unavailable;
   corroborate the listener is loopback-only. A missing second machine is a
   pending remote-access gate, not passing evidence from localhost alone.
7. Same-user theft of readable upstream auth/signing storage and forged valid
   cookies are accepted bearer-auth limitations. Do not claim process-identity
   isolation or expect denial when a valid stolen/forged credential is supplied.

## Report schema and release disposition

Write actual results to `reports/us1.md`; cross-link `reports/security-network.md`
and lifecycle/performance evidence without duplicating secrets. For each row:

- Scenario ID and requirement mapping (FR-002–005/FR-025; SC-010 as applicable).
- Status: **pending**, **passing**, **failing** or **release-blocking upstream
  gap**. Missing instrumentation/environment or unimplemented behavior is pending.
- UTC times, machine/OS/architecture, installed app/DMG/build revision, canonical
  versions, final manifest/dependency-lock hashes and signing mode.
- Disposable fixture IDs and prerequisites, current notice revision and
  acknowledgement/write result; upstream setup/task states recorded separately.
- Native observed sender WebView/main-frame/security-origin category, primary
  document/window/runtime generation and opaque handle identity. Do not record
  renderer assertions as native facts or include the private service auth URL.
- Owned launcher/service/listener identity, sanitized dispatch/side-effect counts,
  denied/revoked outcomes and actual cleanup completion. No unrelated PID scans
  or kills; no implication of a sandbox for arbitrarily detached commands.
- Byte digests/counts, frame sequence/multiplex counters, queue high-water marks,
  cancellation stage and no-replay observations; no bodies or credential values.
- Native-only secret-absence boolean/count results, protected-request admission
  outcome and remote-client environment. No raw stdout/stderr or cookie dumps.
- Separately observed standalone R/product result, instrumentation differences,
  exact defect owner, correction/retest reference and remaining minimum-OS/final
  installed/public-signing gates.

All unauthorized native frames/windows/origins/stale handles and unauthenticated
protected operations must be denied for SC-010. A partial table, ad-hoc install,
source review or successful unit suite cannot turn unavailable/failing gates into
passing qualification. Keep Developer ID/notarization/public-distribution status
separate from locally observed functional qualification.

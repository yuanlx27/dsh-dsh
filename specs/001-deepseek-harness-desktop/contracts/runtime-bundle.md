# Runtime Bundle and Launcher Contract

## Immutable distribution

Initial target: `aarch64-apple-darwin`, macOS 14+, signed/notarized .app and DMG.
Tauri `bundle.externalBin` contains `binaries/node` and `binaries/dsh`;
staging filenames include the target suffix required by Tauri. The dsh binary is
a minimal native launcher because upstream's actual CLI bin is JavaScript.

Ship the locked production install of `@deepseek-ai/dsh@0.2.0-rc.2` as bundle
resources, including dynamically resolved packages, Web assets and macOS/Node
compatible native addons. Do not assume the CLI tarball alone contains the Web
application. Resolve paths through Tauri/bundle resources, never installation
cwd, a shebang's PATH lookup, global npm or npx.

The launcher starts bundled Node with the packaged CLI; it does not add agent
logic, a task API, a duplicate Web service, or an alternative upstream Desktop
Host. Node and dsh are both shipped, while only one upstream service is running.

## Official reference boundary

A1/A6 in the [plan alignment matrix](../plan.md#official-desktop-alignment-fr-022fr-024)
adapt official Host readiness and immutable release verification to this existing
Web CLI/Tauri contract. The behavioral reference O at
`5badb15009ae1756c3afe0ae0cef1faafc290ccc` is not the bundled source R at
`639ed015397290b3745d163aafe02ffee4aa3f84` and is not a new manifest dependency.
Do not copy O's Electron Host IPC, pnpm/runtime layout or package versions into R.

Comparative acceptance must record the two baselines separately. Reference-only
changes do not alter the bundle/version gate; a proposed R upgrade needs the
FR-024 affected-decision review and fresh packaging/behavior evidence.

## Manifest and version gate

`runtime.lock.json` records the baseline/version/checksums; implementation
generates the packaged Runtime Manifest described in [data-model](../data-model.md).
Check all of:
- root package and Tauri canonical version;
- manifest desktopVersion and dshVersion;
- installed dsh package version and bundled CLI `--version`;
- bundled Node `--version`, target architecture and artifact integrity;
- frozen full dependency closure and frontend/native resource availability.

Desktop/dsh equality compares complete strings, including `-rc.2`. Any
mismatch fails packaging and startup before Web navigation or task input.
About uses the manifest's full canonical version. Numeric Apple version/build
fields are derived packaging representations, not separate release versions.

No runtime dependency fetch. Native binaries/addons must be signed in the final
bundle; inspect entitlements required for Node JIT under hardened runtime, apply
the least privileges necessary, and test the actual notarized installed build.
Do not distribute an unsigned development runtime as a passing production test.

## Launch inputs

Private, Rust-controlled launcher inputs:
- absolute bundled Node path and absolute packaged CLI path;
- fixed args: `web --host 127.0.0.1 --port 0 --no-open` plus the validated
  application-owned overlay path using upstream CLI patch syntax;
- explicit app-private `DSH_HOME`, stable writable working directory;
- owner-liveness pipe, owned process group and captured stdout/stderr.

No untrusted arbitrary executable/CLI argument is accepted from Web content.
Set the app-private data root to `0700`. Leave dsh's plaintext credential store
upstream-owned: its `.credentials.yaml` writes use `0600` and overly permissive
files are rejected on macOS. Do not add encryption, Keychain migration or a
shell model-credential API. Remove inherited Node injection
variables such as `NODE_OPTIONS` and `NODE_PATH`; do not inherit model keys or
product telemetry endpoints accidentally. Preserve the ordinary environment
needed by user-approved project commands rather than pretending bundled Node
replaces a developer's entire toolchain.

Use an app-owned Web-profile overlay to disable the release's
`desktop-product-telemetry`, `product-analytics`, feedback-reporting rows/UI and
custom plugin-installation UI. Do not enable the upstream Desktop profile.
Validate dependency relationships and resulting UI at implementation time.

## Readiness output and handoff

Read bounded, line-framed UTF-8 stdout from the owned launcher only.
Release baseline readiness record:

```text
dsh web: <authenticated URL>
```

A potential LAN annotation is not another navigation target. Reject a LAN
announcement from the localhost-only configured launch. Validate the first URL
with a URL parser: HTTP, literal 127.0.0.1, port 1–65535, no username/password,
and release-compatible path/authentication representation. Preserve legitimate
upstream query/fragment authentication; do not reconstruct a bare URL.

Only accept readiness from the current launch generation. Cap line size at
64 KiB; ignore bounded unrelated log lines without persisting them. Never log
or persist the URL. A malformed announcement, child exit or 15-second deadline
produces a sanitized failure and explicit Retry after cleanup.

Do not load the authenticated URL in the WebView. Rust exchanges its root token
privately with redirects disabled; require the upstream 303 response and valid
service cookie, then discard the token URL. Keep the cookie native and transient;
never put it in renderer state or a WebView cookie jar. Retrieve the authenticated
index's non-secret boot injections for the packaged app-origin interface, retaining
R's boot/module protocol. Complete this within the startup deadline. A server
announcement does not prove connection/model/task readiness; failed exchange,
boot or connection produces an actionable sanitized error.

## Authenticated transport

The main document and packaged assets use `dsh-app://app`; local shell views use
`dsh-app://shell`. URI-scheme serving resolves immutable packaged assets only,
rejecting path traversal. Service bodies use the native streaming bridge rather
than assuming Tauri's byte-body URI responder is a streaming HTTP server.

- Admit the actual current primary WKWebView main frame at the app origin only
  after notice acknowledgement and runtime readiness. Validate native frame
  metadata, not a renderer-supplied owner/origin. No child-frame, other-window,
  external-origin or shell-view transport access. Bootstrap only the main frame.
- Adapt upstream `__DSH_TRANSPORT__.fetch`/`openStream` with the narrow bundled
  shim. Existing service Fetch consumers, including raw uploads/downloads, use
  the same fixed-origin HTTP adapter. Leave external fetching outside this
  authenticated path. No task-specific API, alternate schema or agent logic.
- HTTP inputs are relative service paths, supported methods, non-auth headers
  and body chunks. Resolve only against the current owned service origin; reject
  absolute/authority-changing targets, malformed paths and token query inputs.
  The native socket target is never renderer-selected. Do not forward requests
  to another generation, another listening port or any external destination.
- Rust `reqwest` removes renderer-supplied Host, Origin, Cookie, Authorization,
  Fetch-Metadata and hop-by-hop headers before adding its native cookie and
  appropriate same-service trust headers. Automatic redirects are disabled;
  unexpected service redirects fail closed rather than forwarding credentials.
  Strip response `Set-Cookie` and connection-level headers; no auth value is
  returned to renderer code or diagnostics.
- Rust `tokio-tungstenite` opens only the owned `/api/remote.mux` WebSocket with
  native cookie/trust headers. The shim uses R's stream protocol/types and
  `openStream` hook; preserve opaque frames, multiplexing, uplink/downlink,
  ordering and cancellation. No renderer WebSocket with credentials and no
  global WebSocket replacement. Connection remains the recovery/generation owner;
  the adapter must not independently replay operations or restart dsh.
- Use private, window-scoped ordered replies with bounded queues and byte limits
  consistent with R, including its 300 MiB buffered API-body default. Preserve
  binary/multipart data and streaming cancellation; reject oversized input before
  dispatch. Do not route arbitrary responses through global broadcast events.
- Closing the window invalidates its handles and cancels its transfers/streams,
  not agent tasks or approvals. Reopening reattaches to the same native cookie
  and service generation. Failure/quit invalidates all handles and discards native
  authentication; stale chunks, callbacks and closed-window operations are denied.

The underlying server still authenticates bearer cookies. Unauthenticated local
session/task/approval APIs are denied; public non-sensitive assets may be reachable
locally. A client with stolen/forged auth is not denied by process identity.
Same-user-readable upstream signing-secret storage is an accepted limitation.
Direct connections from other computers must fail through loopback binding.

## Shutdown/ownership

Launcher owns the child process group; the shell owns the launcher. Normal
confirmed quit sends a graceful termination request once. Upstream's disposer
has a five-second bound; allow seven seconds at the shell boundary before
force-terminating owned survivors, then wait for actual process exit.
Unexpected shell death closes the owner pipe; launcher stops its group.
Ensure the launcher's own signal handler cannot disappear before child cleanup.
This small protocol is private and never available to Web content.

Do not scan/kill unrelated dsh instances or use bare persisted PIDs as identity.
Do not adopt another service on a responding port. If cleanup fails, retain a
visible failure and do not report successful shutdown or launch a competing
generation. Arbitrarily detached commands are not sandboxed by this ownership
mechanism; do not make isolation guarantees.

## Packaging acceptance

Clean-machine launch without Node/npm/dsh; offline startup; bundle path containing
spaces; architecture mismatch; corrupt/missing resources; different desktop/dsh
prereleases; occupied conventional port; native addon loading; denied app-data
write; repeated launch; shell crash; signed/notarized execution. Every negative
case must block unsafe startup and provide a non-secret explanation.

Transport acceptance additionally covers private token exchange, auth-header and
redirect stripping, exact-owned-destination validation, native frame rejection,
HTTP and WebSocket denial for unauthenticated local clients, no authentication in
renderer state, streaming/binary fidelity, bounded queues and stale-handle rejection.
Run credential lifecycle and `0600`/overly-permissive-file tests separately. These
are implementation release gates, not evidence of an already tested build.

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
Set private directory/file permissions. Remove inherited Node injection
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

Use the authenticated URL for the Web view and retain the release's own
connection/trust protocol. A ready announcement proves server initialization,
not successful model credentials or task readiness. A blank page, failed
navigation or unusable connection must become an actionable error rather than
an indefinitely blank window.

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

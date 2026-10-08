# T022: pinned Web profile exclusion overlay

The shipped `resources/desktop-web.patch.yml` disables exactly seven R rows:
`desktop-product-telemetry`, `product-analytics`, `session-telemetry-otel`,
`command-feedback`, `message-feedback`, `ui-message-feedback`,
`ui-plugin-manager`. These are ordinary `id`/`disabled: true` patches, not
removal, `!!js`, config replacement or Desktop profile activation.

Both product analytics/telemetry are already profile-gated off in R's Web
profile; explicit patches prevent user/home overlays re-enabling them. The
feedback-only network exporter is also disabled, while local log/event services
and core OTel plumbing remain. All other fields/rows, including preset configs,
task controllers, permission/approval, trust/connection, credentials, workspaces,
read-only inventory and built-in settings, match the original composed tree.

R has no install-only UI setting. `ui-plugin-manager` owns the sidebar management
entry and automatic registry probe, so its whole surface is disabled. This does
not remove the upstream host `plugin-manager` provider used by preset tools, nor
pretend that users/tools can no longer install arbitrary software. There is no
new management UI/RPC or native replacement schema; this is the approved narrow
UI exclusion, not a sandbox or a universal plugin-installation prohibition.

## Executed evidence

- Test written before overlay: failed with ENOENT, before behavioral assertions.
- Test uses the actual bundled Node and pinned R CLI, disposable HOME/DSH_HOME,
  filtered environment, 15-second bound and bounded output. No real user profile
  or credentials are touched.
- Real `--profile web --patch <absolute path> --dump-config` accepts/composes the
  overlay; parsed through R's `entryListSchema`. Every row is compared to R's
  baseline, asserting only the seven disabled fields change. Core task,
  permission and trust-related rows remain enabled.
- Real patched `--help` mounts the Web dependency tree and succeeds without
  binding a service or activating the reserved Desktop profile.
- Controlled `ui-plugin-manager` disabled=false mutation fails an actual
  assertion, exit 1. Restored the patch and reran passing test.
- Runtime inventory regenerated (26,761 artifacts), now includes the overlay's
  final hash; `runtime:verify`, source policy, strict tool typing and diff checks
  pass. Production package/version closure unchanged.

This is CLI syntax/composition/dependency evidence, not a live-task execution or
installed browser/network audit. Actual module admission, absence of excluded
UI/network traffic, preserved permission behavior and native launch remain T032
qualification scenarios. Task definition/implementation is not their acceptance.

# US2 — task/control parity (T036 in progress)

## Executed standalone backend subset

`tests/fixtures/standalone-task-driver.ts` boots downloaded pinned R's own
exported `runProfile` with its stock **web** profile, literal-loopback/ephemeral
port and `--no-open`. R remains the actual agent, model adapter, credential,
policy, session log and cancellation owner. The driver calls the existing Host
`sessionController` service directly: it does **not** exercise HTTP RPC admission,
Remote mux, browser frontend, desktop transport or WKWebView.

Producer pin: dsh `0.2.0-rc.2`, R
`639ed015397290b3745d163aafe02ffee4aa3f84`, Node `24.21.0`. This is a repository
runtime/web-profile execution, not an installed bundle closure qualification.
`standalone-tasks.ts` launches one separately owned Node per case with an explicit
minimal environment, private disposable HOME/DSH_HOME and workspace. No real
credential or user's manual candidate root is read, copied or inherited; no
window, keyboard or Inspector automation is used.

The existing controlled loopback supplier holds its Messages stream after a
text delta. Supported R settings/credentials APIs configure the exact
`llm-deepseek` namespace and public synthetic credential. A fixture-only fetch
guard refuses non-literal-loopback destinations and redirects; final cases assert
zero refused external attempts. Upstream stdout is discarded except a closed
boolean/count result; stderr is discarded. No bootstrap URL, raw error, header,
prompt, response or session record is retained in evidence. Disposable trees
are removed after child closure; escalation targets only the captured child.

| Case | Actually asserted |
|---|---|
| Completion | Supported prompt admitted; live Session running while supplier stream held; release ends exactly one turn with R `completed` reason and one non-interrupted assistant message |
| Stop | Same admission/running observation; R `cancel` settles exactly one turn with `aborted` reason and preserves one interrupted assistant message containing the delivered prefix |

Both cases assert one **matching RPC request identity** in the durable human
message source, authenticated fixture requests, unchanged README-only workspace,
whole R context disposal and child exit zero. R may inject other user-role context
messages; these are not counted as duplicate human submissions. Stop intentionally
does not erase already delivered text. No tool ran, so these cases establish
neither action rollback nor permission/action-effect correctness.

Missing-fixture RED preceded implementation. Targeted execution and strict tooling
checks pass; full packaging suite now has **84 passing tests**, source policy and
both tooling/browser typechecks pass. Test elapsed time is not SC-003 latency.

## Failed preparation, not passing evidence

An early selector matched the first `apiKeyEnv` field, mistakenly selecting the
web-search namespace rather than the chat adapter. Pre-guard preparations saw no
controlled supplier request and ended with AUTH failures; they **cannot be
certified loopback-only** and may have contacted the default public endpoint with
the public synthetic credential. No successful supplier response was observed;
no genuine credential or manual-user provider connection was used. Those pilots
are not parity evidence. The final exact namespace selector, non-loopback fetch
guard and zero-external-attempt assertion prevent this preparation error from
silently passing. An initial Stop assertion also incorrectly expected the partial
message to disappear; the corrected expectation follows R's documented
`assistant/message.interrupted` semantics, without changing R or product code.

## Still pending

T036 remains **unchecked**. See [task/control procedures](../tasks-control.md),
[T034 resources](task-fixture-preparation.md), and
[T035 adapter-only regressions](task-forwarding.md).

- Actual permission gate/preset, tool advertisement, allow/deny/dismiss, bounded
  tool effects and in-flight action Stop in separate standalone/product roots.
- Queued/steered/identical inputs, multiple workspaces/sessions, failed/interrupted
  tasks and upstream requirement gaps.
- Installed native R frontend comparison, raw stream/transport fidelity,
  hide/show/reconnect and no replay, receiving-boundary/Stop latency measurements.
- Exclusive interactive session before any native UI automation; the user's
  ongoing manual session and intermittent composer defect remain undisturbed.

User-reported M01–M06 are separately recorded in [manual results](manual-results.md).
They do not complete this controlled matrix, T032/T033, or SC-003/SC-004.

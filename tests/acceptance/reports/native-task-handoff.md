# Native task/control handoff — prepared, not passed

The user reports the previous manual application has exited and is available for
interactive testing. No window/keyboard automation or application launch has been
performed by the agent in this handoff.

The unchanged release candidate from source `2bdf282` is reused:
`/private/tmp/dsh manual install.bZA0oI/DeepSeek Harness Desktop.app`.
Deep/strict signature and the native installed-closure verifier were re-run and
passed. App binary SHA-256 remains
`eb996e69267d34451f5067ac3f681fde8bd31103b3ce60a539a43fa6dc44d810`.
New fixture sources do not change the candidate or downloaded upstream assets.

A **new** private disposable tree is prepared:

- HOME: `/private/tmp/dsh-task-fixture-xHoqW0/native-home`
- Workspace A: `/private/tmp/dsh-task-fixture-xHoqW0/product/A`
- Workspace B: `/private/tmp/dsh-task-fixture-xHoqW0/product/B`
- User launch: `source /private/tmp/dsh-native-task-start.txt`
- Public fixture Messages Base URL: `http://127.0.0.1:58426/v1`
- Public synthetic credential: `desktop-fixture-first` (not a real provider key)
- Supplier process: `9171`, Node `24.21.0`, owned setup spawn; initial mode approval.

The launch command sets the working directory to A and uses env-i/private HOME,
no debug flags. It does not read/copy the prior manual HOME or any genuine key.
The supplier listens only on literal loopback. Original upstream Models UI must
set **DeepSeek**, not DeepSeek Account, to this Base URL and synthetic credential
before any task submission. Skip credential onboarding with Configure later;
never enter a real key or use a published provider endpoint for this fixture.
Select the original Session's read-only preset; pending approval must be obtained
from the actual browser/Remote answerer, with no same-process answerer hook.

`scripts/manual-task-supplier.ts` reuses `controlledModel`. Host-only private
file control selects approval, bounded action-stop, held stream, finish or close;
there is no HTTP administration route and no tool execution by the supplier.
Rearming refuses active provider requests. **The host must also wait for the
native task to finish or Stop before rearming**: supplier HTTP idleness is not
Session idleness. Each arming emits one actually advertised bash call on its
first streaming operation; nonstreaming title requests do not consume it. Fixed
follow-up model text is not evidence of action success; inspect the actual tool
card and filesystem. An inactive/closed supplier cannot be rearmed.

Public-only ready/control/category files are under the new tree; startup command
and handoff JSON under `/private/tmp`. No service token, cookie, body, general
Inspector dump or actual project data is recorded. Initial files are README-only,
owner-only directories; no action marker exists. A file-control smoke held and
released one direct synthetic HTTP stream, then rearmed approval. This is a supplier
self-test only: its one request must not be counted as native task traffic.
The supplier remains running for
the user session and must be closed after native confirmed quit. Do not remove
its tree or signal any previous manual app/user-owned service.

Rearming regression was RED before support, then GREEN. Full packaging suite:
89 passing tests, source policy and strict tooling/browser checks pass. Help and
unknown flag exit behavior were exercised. Signature/closure verification and a
live loopback listener are preparation only, not actual native task evidence.
T036 and T032/T033 remain unchecked. Pending/allow/deny/withdraw/action Stop and
native no-replay/identity/latency observations must be recorded separately in
[US2](us2.md) after execution, without inferring quantitative gates from user reports.

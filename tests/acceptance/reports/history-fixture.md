# T041 — supported upstream history fixture generation

`tests/acceptance/create-history-fixture.ts` creates NEW owner-only disposable
roots, reusing `taskWorkspaces()` and the existing pinned-export binding (now
shared, not copied). Requires Node 24.21.0/dsh 0.2.0-rc.2. No Host/server/model,
credentials, network API or product/official-O data root is used.

```sh
node tests/acceptance/create-history-fixture.ts --help
node tests/acceptance/create-history-fixture.ts
```

stdout gives owned root/inventory location only. Keep the generated root for
history acceptance; later remove only that reported owned root. The inventory
contains public fixture metadata/hashes, not raw session messages. Standalone
and product are **both pinned R**, independent homes/stores/workspaces/IDs, not
an official O artifact or a real standalone UI run.

## Exact generated case

Four current-format compressed histories: A=**1,000 user messages**, B=10 for
each owner, independently minted stable message/session IDs, explicit nonempty
public titles and exact workspace cwd. This is a **defined user-only generated
fixture**, not 1,000 actual model/tool completions or representative mixed media.
Generated turn/step boundaries are fixture data, not live task effects/status
acceptance. Mixed-role/approval/corrupt derivatives remain their separate cases.

The product home anticipates the implemented native app's private
`Library/Application Support/io.github.yuanlx27.dsh-dsh/dsh/sessions` root;
standalone has its own `.dsh/sessions`. It does not register workspaces through a
shell schema; choose/register exact A/B through original upstream UI for later
native trials. Notice/model/permission setup remains explicit when appropriate.
Do not reuse old live test homes or copy official O sessions into this corpus.

## Authority and durability

Create events/header through published `SessionStore.create`/`Session.append`,
not a replacement encoder. Use published JSONL `create`/handle `append`, **flush**
and `close`. Every artifact is then opened by a **fresh upstream backend**;
`Session.fromRestore`, `deriveMessages` and `foldSessionTitle` verify identity,
canonical lifecycle/surface rules, full unique count/title/cwd. Original hashes
pin the validated artifact. No deprecated synchronous session-log reads.

Initial zero-based turns/steps were rejected by upstream released-v4 relationship
rules. Corrected to its one-based turn/step convention; no validator relaxation,
upstream patch or fake read fallback. Wrong Context disposal API and private
constructor type binding were caught/fixed without `any` or suppressed checks.
A fresh Context per backend honors one service registration per scope.

## Tests/evidence

Three tests: generation/cold restoration/private modes/exact counts; corrupted
physical artifact **propagates upstream refusal**; missing artifact **refuses and
is not recreated**. First RED missing module, then actual upstream refusal during
implementation, finally all three GREEN. Complete packaging suite **97/97** and
source policy/strict tooling+browser checks pass. Existing standalone real-backend
tests pass after shared binding extraction.

One retained CLI fixture was generated/validated in this sweep (path in private
`/private/tmp/dsh-history-fixture-ready.txt`); four histories, no live app/supplier.
This completes T041 preparation, not T042/T043/T044: no native rendered-history,
20 normal/10 forced recovery, UI title correction, directory move or 2-second
1,000-message latency result claimed. The measured unit/SDK generation duration
is not SC-006 native open latency.

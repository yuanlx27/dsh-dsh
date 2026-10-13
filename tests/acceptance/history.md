# Upstream-owned history acceptance (T040)

## Ownership, provenance and fixtures

Runtime baseline is published dsh 0.2.0-rc.2 at
`639ed015397290b3745d163aafe02ffee4aa3f84`, Node 24.21.0. Use separate disposable
standalone and installed-product data roots and accessible local A/B directories.
Never share a live store, use a genuine profile, or export raw records/keys.
Record exact candidate/runtime identity, session identities, canonical workspace
associations and committed inventory counts/hashes before each destructive case.
Use only public synthetic messages, loopback model supply and controlled actions.

Generate ordinary histories through supported upstream Session/SessionPersistence
facilities or original UI; flush through upstream durability barriers. Validate
records using upstream current-format/storage rules and reopen them through the
upstream reader. Do not fabricate a shell storage schema, reader, title fallback,
cache, repair API, directory migration, or model/task engine.
[T041 helper](create-history-fixture.ts) now cold-validates four independent
public user-only SDK histories (A=1,000/B=10 per owner); see
[fixture evidence](reports/history-fixture.md). This is not native history/open
latency acceptance. Count **messages**, not events, frames or turns; preserve
exact corpus identity/roles rather than claim 1,000 real model completions.

Corruption/torn-tail specimens are explicitly labelled negative derivatives of
flushed disposable fixtures. Preserve their clean originals. Mutate only selected
fixture artifacts while every owning writer is closed. Record the mutation kind
and expected committed prefix, not its raw body. Do not treat intentionally
changed artifacts as normal committed-history inventory equality.

## Case matrix

| ID | Preparation/action | Required comparison and evidence |
|---|---|---|
| H01 Distinct identity | Two sessions in A with identical title, plus blank-title sessions where supported by upstream title operations | Listing preserves upstream title/fallback behavior and distinct identities; opening/follow-up targets the selected identity, never merges by text. If blank title is normalized/refused, record the actual upstream result, do not invent a stored title |
| H02 Workspace grouping | At least one independently identified A session and B session; submit different public follow-ups through original UI | Listing groups by original canonical workspace; each follow-up has one matching receipt in the intended original session; no other session/workspace receives it. Renaming a display title does not change association |
| H03 Completed/interrupted | Flush completed work; separately terminate only proven owned shell/service during controlled incomplete work | Prior committed messages remain readable; orphaned work follows upstream interrupted closer rules. Unsaved RAM output is outside preservation guarantee. Restore/list/open performs no automatic submission, command or edit |
| H04 Pending approval | Hold an actual upstream approval-required action with marker absent; preserve inventory, then exit/recover | No implicit allow or replay; marker remains absent, late stale answer cannot execute. Distinguish actual pending, rejected/cancelled and interrupted states; do not relabel Reject as withdrawal. Standing policy remains upstream-owned |
| H05 Recoverable tail | Derive a torn final physical record/frame after a known flushed prefix using the actual backend format | Upstream read returns only its valid prefix, and write recovery follows its documented truncation rules. No invented message or rewritten committed prefix; native matches standalone. Unsupported damage must fail, not be classified recoverable by a shell heuristic |
| H06 Corrupt/unsupported | Separately damage a committed record/header or declare an unsupported format in disposable derivatives | Preserve upstream corruption/version refusal and actionable unavailable-history behavior; never silently replace, migrate, repair, drop into an empty success state or certify restoration |
| H07 Missing/discovery | Remove a selected fixture artifact, deny its access, and separately exercise discovery omission of invalid headers | Direct open failure remains explicit. A list omission is not proof that no history existed; record known prior identity/inventory alongside discovery result, with an explicit upstream corrective action. No shell filesystem preflight/index |
| H08 Moved/restored workspace | While owners are stopped, rename original A to a disposable backup, reopen its original session, then select B separately; restore A to exactly its former path | History and dependent work follow upstream availability/continuation/refusal choices. No directory replacement or session reassociation; choosing B does not transfer A's conversation. Restoration does not migrate history; follow-up after restoration remains linked to original A |
| H09 Normal relaunch | 20 installed normal relaunches with complete A/B inventory | Every previously committed message and association remains available; capture owned exit/new startup and readable original identities. Ordinary close/hide is not a relaunch |
| H10 Forced recovery | 10 installed forced shell/service-exit recoveries, including incomplete work and approval specimens | Proven ownership-only exit injection; predecessor cleanup before replacement; no auto-resume/approval/replay. One successful installed retry is not ten cases |
| H11 Large opening | 20 openings of validated 1,000-message session after app/service already connected | Named otherwise-idle host, request-received to identified/readable/browsable first screen, >=19 within 2 seconds. Exclude startup, preparation, all-message rendering and model response; do not use test elapsed time or a shell cache |

## Execution and reporting

For each H01–H08, report standalone baseline and installed native outcome
separately, including title normalization, missing-directory options and any
upstream gap. Source inspection, persistence-only fixtures and browser/SDK tests
cannot substitute for installed listing/history/follow-up routes (T042).

T043 writes `reports/history-recovery.md` with 20 normal/10 forced runs and
sanitized inventories. T044 writes `reports/history-performance.md` with 20
receiving-boundary measurements. No latency/performance pass is pre-filled.
Reports retain only identities, counts, flags, hashes, states and selected timing;
never secrets, token URLs, general Inspector dumps or raw user/model/tool logs.

Block history/release qualification on a failing requirement or unavailable
runner. Record actionable upstream presentation gaps rather than implementing
history business logic in the desktop. T040 defines cases only; it does not
complete T041–T044, SC-005 or minimum-OS acceptance.

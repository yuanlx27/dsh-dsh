import assert from "node:assert/strict";
import test from "node:test";
import { readFile, stat, writeFile, unlink } from "node:fs/promises";
import { createHistoryFixture } from "../acceptance/create-history-fixture.ts";

test("pinned upstream creates and reopens 1,000-message/two-workspace histories without a shell codec", async t => {
  const fixture = await createHistoryFixture(); t.after(fixture.close);
  assert.equal((await stat(fixture.root)).mode & 0o777, 0o700);
  assert.equal(fixture.sessions.length, 4);
  assert.equal(new Set(fixture.sessions.map(s => s.id)).size, 4);
  for (const owner of ["standalone", "product"] as const) {
    const sessions = fixture.sessions.filter(s => s.owner === owner);
    assert.deepEqual(sessions.map(s => s.messageCount), [1000, 10]);
    assert.notEqual(sessions[0]!.cwd, sessions[1]!.cwd);
    for (const s of sessions) {
      assert.equal((await stat(s.path)).mode & 0o777, 0o600);
      assert.equal(s.generatedRoles, "user-only");
      assert.match(s.title, /^Public generated history/);
    }
  }
  await fixture.validate(); // A fresh upstream backend, not a warm generation cache.
  const metadata = await readFile(fixture.inventory, "utf8");
  assert.doesNotMatch(metadata, /token=|api_key|cookie|fixture record/);
});

test("history helper propagates upstream corruption refusal instead of accepting invented records", async t => {
  const fixture = await createHistoryFixture(); t.after(fixture.close);
  await writeFile(fixture.sessions[0]!.path, Buffer.from([0, 1, 2]));
  await assert.rejects(fixture.validate());
});

test("history helper propagates a missing authoritative artifact rather than recreating it", async t => {
  const fixture = await createHistoryFixture(); t.after(fixture.close);
  await unlink(fixture.sessions[0]!.path);
  await assert.rejects(fixture.validate());
  await assert.rejects(stat(fixture.sessions[0]!.path), { code: "ENOENT" });
});

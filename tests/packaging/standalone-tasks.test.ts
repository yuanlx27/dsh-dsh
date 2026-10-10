import assert from "node:assert/strict";
import test from "node:test";
import { standaloneTaskCase } from "../fixtures/standalone-tasks.ts";

for (const kind of ["complete", "stop"] as const) {
  test(`pinned R web profile ${kind}: real admission, running and settlement without a window`, { timeout: 45000 }, async () => {
    const result = await standaloneTaskCase(kind);
    assert.equal(result.accepted, true);
    assert.equal(result.runningObserved, true);
    assert.equal(result.settled, true);
    assert.equal(result.loopbackOnly, true);
    assert.equal(result.endExpected, true);
    assert.equal(result.providerAuthenticated, true);
    assert.equal(result.promptCount, 1);
    assert.equal(result.turnStarts, 1);
    assert.equal(result.turnEnds, 1);
    assert.equal(result.assistantMessages, 1);
    assert.equal(result.interrupted, kind === "stop");
    assert.equal(result.prefixPreserved, true);
    assert.equal(result.workspaceUnchanged, true);
    assert.equal(result.disposed, true);
    assert.equal(result.exitCode, 0);
  });
}

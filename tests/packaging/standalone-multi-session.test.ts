import assert from "node:assert/strict";
import test from "node:test";
import { standaloneTaskCase } from "../fixtures/standalone-tasks.ts";

test("pinned backend Stop is session-scoped across two disposable workspaces", { timeout: 45000 }, async () => {
  const result = await standaloneTaskCase("multi-session-stop");
  assert.equal(result.accepted, true);
  assert.equal(result.multiRunningObserved, true);
  assert.equal(result.otherRunningAfterStop, true);
  assert.equal(result.otherPromptCount, 1);
  assert.equal(result.otherCompleted, true);
  assert.equal(result.endExpected, true);
  assert.equal(result.promptCount, 1);
  assert.equal(result.turnStarts, 1);
  assert.equal(result.turnEnds, 1);
  assert.equal(result.interrupted, true);
  assert.equal(result.prefixPreserved, true);
  assert.equal(result.toolCalls, 0);
  assert.equal(result.loopbackOnly, true);
  assert.equal(result.providerAuthenticated, true);
  assert.equal(result.workspaceUnchanged, true);
  assert.equal(result.otherWorkspaceUnchanged, true);
  assert.equal(result.disposed, true);
  assert.equal(result.exitCode, 0);
});

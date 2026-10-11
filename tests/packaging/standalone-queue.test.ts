import assert from "node:assert/strict";
import test from "node:test";
import { standaloneTaskCase } from "../fixtures/standalone-tasks.ts";

test("pinned backend preserves two identical queued receipts through explicit steering", { timeout: 45000 }, async () => {
  const result = await standaloneTaskCase("queue-steer");
  assert.equal(result.accepted, true);
  assert.equal(result.runningObserved, true);
  assert.equal(result.queuePendingObserved, true);
  assert.equal(result.queueAdmissions, 2);
  assert.equal(result.queueSteers, 2);
  assert.equal(result.queueDeliveries, 2);
  assert.equal(result.queueReceiptsDistinct, true);
  assert.equal(result.queueReceiptsPreserved, true);
  assert.equal(result.promptCount, 1);
  assert.equal(result.turnStarts, 1);
  assert.equal(result.turnEnds, 1);
  assert.equal(result.endExpected, true);
  assert.equal(result.toolCalls, 0);
  assert.equal(result.approvalAsked, 0);
  assert.equal(result.loopbackOnly, true);
  assert.equal(result.providerAuthenticated, true);
  assert.equal(result.workspaceUnchanged, true);
  assert.equal(result.disposed, true);
  assert.equal(result.exitCode, 0);
});

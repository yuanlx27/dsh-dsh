import assert from "node:assert/strict";
import test from "node:test";
import { standaloneTaskCase } from "../fixtures/standalone-tasks.ts";

for (const kind of ["allow", "deny", "withdraw"] as const) {
  test(`upstream dsh actual bash gate: ${kind} has the expected disposable file effect`, { timeout: 45000 }, async () => {
    const result = await standaloneTaskCase(kind);
    assert.equal(result.accepted, true);
    assert.equal(result.runningObserved, true);
    assert.equal(result.gateObserved, true);
    assert.equal(result.pendingNoEffect, true);
    assert.equal(result.decisionExpected, true);
    assert.equal(result.policyUnchanged, true);
    assert.equal(result.approvalAsked, 1);
    assert.equal(result.approvalDecided, 1);
    assert.equal(result.toolCalls, 1);
    assert.equal(result.toolResults, 1);
    assert.equal(result.toolSucceeded, kind === "allow");
    assert.equal(result.markerApproved, kind === "allow");
    assert.equal(result.workspaceExpected, true);
    assert.equal(result.endExpected, true);
    assert.equal(result.promptCount, 1);
    assert.equal(result.turnStarts, 1);
    assert.equal(result.turnEnds, 1);
    assert.equal(result.settled, true);
    assert.equal(result.loopbackOnly, true);
    assert.equal(result.providerAuthenticated, true);
    assert.equal(result.disposed, true);
    assert.equal(result.exitCode, 0);
  });
}

test("upstream dsh Stop interrupts a started foreground bash job without undo", { timeout: 45000 }, async () => {
  const result = await standaloneTaskCase("action-stop");
  assert.equal(result.gateObserved, true);
  assert.equal(result.pendingNoEffect, true);
  assert.equal(result.decisionExpected, true);
  assert.equal(result.policyUnchanged, true);
  assert.equal(result.actionRunningObserved, true);
  assert.equal(result.actionPrefixPreserved, true);
  assert.equal(result.jobsQuiescent, true);
  assert.equal(result.workspaceExpected, true);
  assert.equal(result.toolSucceeded, false);
  assert.equal(result.toolCalls, 1);
  assert.equal(result.toolResults, 1);
  assert.equal(result.approvalAsked, 1);
  assert.equal(result.approvalDecided, 1);
  assert.equal(result.endExpected, true);
  assert.equal(result.settled, true);
  assert.equal(result.loopbackOnly, true);
  assert.equal(result.providerAuthenticated, true);
  assert.equal(result.disposed, true);
  assert.equal(result.exitCode, 0);
});

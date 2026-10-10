import assert from "node:assert/strict";
import test from "node:test";
import { readFile, stat } from "node:fs/promises";
import { controlledModel } from "../fixtures/controlled-model.ts";
import { taskWorkspaces } from "../fixtures/task-workspaces.ts";

const headers = { "x-api-key": "desktop-fixture-first", "content-type": "application/json" };
const tool = { name: "bash", input: { command: "printf approved > approval-marker.txt", description: "Write the disposable acceptance marker", sandbox_permissions: "workspace-write", justification: "Write the disposable acceptance marker only." } };

test("task roots separate standalone/product and prepare no action effects", async t => {
  const fixture = await taskWorkspaces(); t.after(fixture.close);
  assert.notEqual(fixture.standalone.A, fixture.product.A);
  for (const pair of [fixture.standalone, fixture.product]) for (const path of [pair.A, pair.B]) {
    assert.equal((await stat(path)).mode & 0o777, 0o700);
    assert.match(await readFile(`${path}/README.md`, "utf8"), /Disposable/);
    await assert.rejects(stat(`${path}/approval-marker.txt`), { code: "ENOENT" });
  }
});

test("controlled tool reply is advertised, one-shot, and leaves execution to R", async t => {
  const fixture = await controlledModel({ tool }); t.after(fixture.close);
  const body = JSON.stringify({ model: "desktop-fixture-model", stream: true, tools: [{ name: "bash", input_schema: { type: "object" } }] });
  const first = await fetch(`${fixture.origin}/v1/messages`, { method: "POST", headers, body });
  const bytes = await first.text();
  assert.match(bytes, /tool_use/); assert.match(bytes, /input_json_delta/); assert.match(bytes, /workspace-write/);
  const second = await fetch(`${fixture.origin}/v1/messages`, { method: "POST", headers, body });
  assert.doesNotMatch(await second.text(), /tool_use/);
  assert.deepEqual(fixture.events, [{ operation: "messages", credential: "first" }, { operation: "messages", credential: "first" }]);
});

test("tool fixture rejects absent advertisement and supports OpenAI tool chunks", async t => {
  const fixture = await controlledModel({ tool }); t.after(fixture.close);
  const absent = await fetch(`${fixture.origin}/v1/messages`, { method: "POST", headers, body: JSON.stringify({ stream: true }) });
  assert.equal(absent.status, 422);
  const response = await fetch(`${fixture.origin}/v1/chat/completions`, { method: "POST", headers, body: JSON.stringify({ stream: true, tools: [{ type: "function", function: { name: "bash", parameters: { type: "object" } } }] }) });
  const body = await response.text(); assert.match(body, /tool_calls/); assert.match(body, /\[DONE\]/);
  assert.doesNotMatch(JSON.stringify(fixture.events), /command|approval-marker|justification/);
});

test("controlled stream holds after visible text and fixture close releases a pending request", async () => {
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  const fixture = await controlledModel({ streamGate: gate });
  try {
    const response = await fetch(`${fixture.origin}/v1/messages`, { method: "POST", headers, body: JSON.stringify({ stream: true }) });
    const reader = response.body!.getReader();
    const first = await reader.read(); assert.equal(first.done, false);
    const text = new TextDecoder().decode(first.value);
    assert.match(text, /text_delta/); assert.doesNotMatch(text, /message_stop/);
    await fixture.close();
    await assert.rejects(reader.read());
  } finally { release(); await fixture.close(); }
});

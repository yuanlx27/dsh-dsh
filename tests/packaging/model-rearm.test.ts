import assert from "node:assert/strict";
import test from "node:test";
import { controlledModel } from "../fixtures/controlled-model.ts";

test("controlled supplier rearms one-shot replies only between completed requests", async () => {
  const supplier = await controlledModel();
  const call = (endpoint = "messages") => fetch(`${supplier.origin}/v1/${endpoint}`, { method: "POST", headers: { "x-api-key": "desktop-fixture-first", "content-type": "application/json" }, body: JSON.stringify({ stream: true, tools: [{ name: "bash" }] }) });
  try {
    const toolIds = new Set<string>();
    const responseIds = new Set<string>();
    for (const endpoint of ["messages", "chat/completions"]) {
      for (let i = 0; i < 2; i++) {
        supplier.arm({ tool: { name: "bash", input: { command: "public-fixture-data" } } });
        const text = await (await call(endpoint)).text();
        const toolId = text.match(/"id":"(fixture-tool-[^"]+)"/)?.[1];
        const responseId = text.match(/"id":"(fixture-(?:message|response)[^"]*)"/)?.[1];
        assert.ok(toolId);
        assert.ok(responseId);
        assert.equal(toolIds.has(toolId), false, "rearming must not reuse a tool identity");
        assert.equal(responseIds.has(responseId), false, "each reply needs a fresh response identity");
        toolIds.add(toolId);
        responseIds.add(responseId);
        assert.doesNotMatch(await (await call(endpoint)).text(), /tool_use|tool_calls/);
      }
    }
    let release = () => {};
    supplier.arm({ streamGate: new Promise<void>(resolve => { release = resolve; }) });
    const held = await call();
    assert.throws(() => supplier.arm({}), /active/);
    release();
    await held.text();
    supplier.arm({});
  } finally { await supplier.close(); }
  assert.throws(() => supplier.arm({}), /closed/);
});

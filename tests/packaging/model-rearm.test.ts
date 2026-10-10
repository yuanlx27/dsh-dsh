import assert from "node:assert/strict";
import test from "node:test";
import { controlledModel } from "../fixtures/controlled-model.ts";

test("controlled supplier rearms one-shot replies only between completed requests", async () => {
  const supplier = await controlledModel();
  const call = () => fetch(`${supplier.origin}/v1/messages`, { method: "POST", headers: { "x-api-key": "desktop-fixture-first", "content-type": "application/json" }, body: JSON.stringify({ stream: true, tools: [{ name: "bash" }] }) });
  try {
    for (let i = 0; i < 2; i++) {
      supplier.arm({ tool: { name: "bash", input: { command: "public-fixture-data" } } });
      assert.match(await (await call()).text(), /tool_use/);
      assert.doesNotMatch(await (await call()).text(), /tool_use/);
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

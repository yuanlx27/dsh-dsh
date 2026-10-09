import test from "node:test";
import assert from "node:assert/strict";
import { controlledModel } from "../fixtures/controlled-model.ts";

test("controlled provider rejects unknown credentials and records categories only", async (t) => {
  const fixture = await controlledModel();
  t.after(fixture.close);
  const response = await fetch(`${fixture.origin}/v1/models`, { headers: { authorization: "Bearer never-record-this" } });
  assert.equal(response.status, 401);
  assert.deepEqual(fixture.events, [{ operation: "models", credential: "rejected" }]);
  assert.ok(!JSON.stringify(fixture.events).includes("never-record-this"));
});

test("controlled provider accepts both replacement identities, discovery and streaming families", async (t) => {
  const fixture = await controlledModel();
  t.after(fixture.close);
  const discovery = await fetch(`${fixture.origin}/v1/models`, { headers: { authorization: "Bearer desktop-fixture-first" } });
  assert.equal(discovery.status, 200);
  const catalog: unknown = await discovery.json();
  assert.deepEqual(catalog, { object: "list", data: [{ id: "desktop-fixture-model", object: "model", owned_by: "controlled-fixture", created: 0 }] });
  for (const [path, headers, end] of [
    ["/v1/messages", { "x-api-key": "desktop-fixture-replaced" }, "event: message_stop"],
    ["/v1/chat/completions", { authorization: "Bearer desktop-fixture-replaced" }, "data: [DONE]"]
  ] as const) {
    const response = await fetch(`${fixture.origin}${path}`, { method: "POST", headers: { ...headers, "content-type": "application/json" }, body: JSON.stringify({ model: "desktop-fixture-model", stream: true }) });
    assert.equal(response.status, 200);
    assert.equal(response.headers.get("content-type"), "text/event-stream");
    const body = await response.text();
    assert.match(body, /Controlled fixture response/);
    assert.ok(body.includes(end));
    assert.ok(!body.includes("tool_use"));
  }
  assert.deepEqual(fixture.events, [
    { operation: "models", credential: "first" },
    { operation: "messages", credential: "replacement" },
    { operation: "completion", credential: "replacement" }
  ]);
});

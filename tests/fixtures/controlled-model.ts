import { createServer } from "node:http";

// Public test literals, NOT actual provider credentials. Never record header/body values.
const keys = new Map([["desktop-fixture-first", "first"], ["desktop-fixture-replaced", "replacement"]]);
export interface ProviderEvent { operation: string; credential: string }

/** Loopback-only model resource; upstream dsh remains the real engine/credential owner. */
export async function controlledModel() {
  const events: ProviderEvent[] = [];
  const server = createServer(async (request, response) => {
    const supplied = request.headers.authorization?.replace(/^Bearer /, "") ?? request.headers["x-api-key"];
    const credential = typeof supplied === "string" ? (keys.get(supplied) ?? "rejected") : "rejected";
    const operation = request.url?.endsWith("/models") ? "models" : request.url?.endsWith("/chat/completions") ? "completion" : request.url?.endsWith("/messages") ? "messages" : "unknown";
    events.push({ operation, credential });
    response.setHeader("content-type", "application/json");
    if (credential === "rejected") { response.writeHead(401); response.end(JSON.stringify({ error: { message: "Controlled fixture credential rejected", type: "authentication_error" } })); return; }
    if (operation === "models") { response.end(JSON.stringify({ object: "list", data: [{ id: "desktop-fixture-model", object: "model", owned_by: "controlled-fixture", created: 0 }] })); return; }
    if (operation === "unknown") { response.writeHead(404); response.end(); return; }
    try {
      const parts: Buffer[] = [];
      let size = 0;
      for await (const part of request) {
        if (!Buffer.isBuffer(part) && typeof part !== "string") throw Error("Unexpected chunk");
        const chunk = Buffer.from(part); size += chunk.length;
        if (size > 1024 * 1024) throw Error("Fixture input limit");
        parts.push(chunk);
      }
      const body: unknown = JSON.parse(Buffer.concat(parts).toString("utf8"));
      const model = typeof body === "object" && body !== null && "model" in body && typeof body.model === "string" ? body.model : "desktop-fixture-model";
      const streaming = typeof body === "object" && body !== null && "stream" in body && body.stream === true;
      const text = "Controlled fixture response. No action requested.";
      if (operation === "messages") {
        const message = { id: "fixture-message", type: "message", role: "assistant", model, content: [], stop_reason: null, stop_sequence: null, usage: { input_tokens: 1, output_tokens: 0 } };
        if (streaming) {
          response.setHeader("content-type", "text/event-stream");
          for (const [event, data] of [
            ["message_start", { type: "message_start", message }],
            ["content_block_start", { type: "content_block_start", index: 0, content_block: { type: "text", text: "" } }],
            ["content_block_delta", { type: "content_block_delta", index: 0, delta: { type: "text_delta", text } }],
            ["content_block_stop", { type: "content_block_stop", index: 0 }],
            ["message_delta", { type: "message_delta", delta: { stop_reason: "end_turn", stop_sequence: null }, usage: { output_tokens: 1 } }],
            ["message_stop", { type: "message_stop" }]
          ] as const) response.write(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
          response.end();
        } else response.end(JSON.stringify({ ...message, content: [{ type: "text", text }], stop_reason: "end_turn", usage: { input_tokens: 1, output_tokens: 1 } }));
      } else {
        const envelope = { id: "fixture-response", created: 0, model };
        if (streaming) {
          response.setHeader("content-type", "text/event-stream");
          response.write(`data: ${JSON.stringify({ ...envelope, object: "chat.completion.chunk", choices: [{ index: 0, delta: { role: "assistant", content: text }, finish_reason: null }] })}\n\n`);
          response.end(`data: ${JSON.stringify({ ...envelope, object: "chat.completion.chunk", choices: [{ index: 0, delta: {}, finish_reason: "stop" }], usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } })}\n\ndata: [DONE]\n\n`);
        } else response.end(JSON.stringify({ ...envelope, object: "chat.completion", choices: [{ index: 0, message: { role: "assistant", content: text }, finish_reason: "stop" }], usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } }));
      }
    } catch { response.writeHead(400); response.end(JSON.stringify({ error: "Invalid controlled fixture input" })); }
  });
  await new Promise<void>((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  const address = server.address();
  if (address === null || typeof address === "string") throw Error("Fixture listener unavailable");
  return {
    origin: `http://127.0.0.1:${address.port}`,
    events: events as readonly ProviderEvent[],
    close: () => new Promise<void>((resolve, reject) => { server.close((error) => error ? reject(error) : resolve()); })
  };
}

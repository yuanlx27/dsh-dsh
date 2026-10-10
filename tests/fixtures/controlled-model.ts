import { createServer } from "node:http";

// Public test literals, NOT actual provider credentials. Never record header/body values.
const keys = new Map([["desktop-fixture-first", "first"], ["desktop-fixture-replaced", "replacement"]]);
export interface ProviderEvent { operation: string; credential: string }
export interface ControlledModelOptions {
  /** One advertised tool reply on the first streaming operation; R executes/approves it. */
  tool?: { name: string; input: Record<string, unknown> };
  /** Hold a text stream after its first visible delta; no timers or model calls needed. */
  streamGate?: Promise<void>;
}

/** Loopback-only model resource; upstream dsh remains the real engine/credential owner. */
export async function controlledModel(options: ControlledModelOptions = {}) {
  const events: ProviderEvent[] = [];
  let toolIssued = false;
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
      const tool = streaming && !toolIssued ? options.tool : undefined;
      if (tool) {
        const advertised = typeof body === "object" && body !== null && "tools" in body && Array.isArray(body.tools) && body.tools.some((value: unknown) => {
          if (!value || typeof value !== "object") return false;
          if ("name" in value && value.name === tool.name) return true;
          return "function" in value && typeof value.function === "object" && value.function !== null && "name" in value.function && value.function.name === tool.name;
        });
        if (!advertised) { response.writeHead(422); response.end(JSON.stringify({ error: "Fixture tool not advertised by upstream" })); return; }
        toolIssued = true;
      }
      const text = "Controlled fixture response. No action requested.";
      const wait = async () => {
        if (!options.streamGate || tool) return;
        let closed: (() => void) | undefined;
        try {
          await Promise.race([options.streamGate, new Promise<void>(resolve => { closed = resolve; response.once("close", resolve); })]);
        } finally { if (closed) response.off("close", closed); }
      };
      if (operation === "messages") {
        const message = { id: "fixture-message", type: "message", role: "assistant", model, content: [], stop_reason: null, stop_sequence: null, usage: { input_tokens: 1, output_tokens: 0 } };
        if (streaming) {
          response.setHeader("content-type", "text/event-stream");
          const emit = (event: string, data: unknown) => response.write(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
          emit("message_start", { type: "message_start", message });
          emit("content_block_start", { type: "content_block_start", index: 0, content_block: tool ? { type: "tool_use", id: "fixture-tool-call", name: tool.name, input: {} } : { type: "text", text: "" } });
          emit("content_block_delta", { type: "content_block_delta", index: 0, delta: tool ? { type: "input_json_delta", partial_json: JSON.stringify(tool.input) } : { type: "text_delta", text } });
          await wait();
          if (response.destroyed) return;
          emit("content_block_stop", { type: "content_block_stop", index: 0 });
          emit("message_delta", { type: "message_delta", delta: { stop_reason: tool ? "tool_use" : "end_turn", stop_sequence: null }, usage: { output_tokens: 1 } });
          emit("message_stop", { type: "message_stop" });
          response.end();
        } else response.end(JSON.stringify({ ...message, content: [{ type: "text", text }], stop_reason: "end_turn", usage: { input_tokens: 1, output_tokens: 1 } }));
      } else {
        const envelope = { id: "fixture-response", created: 0, model };
        if (streaming) {
          response.setHeader("content-type", "text/event-stream");
          const delta = tool ? { role: "assistant", tool_calls: [{ index: 0, id: "fixture-tool-call", type: "function", function: { name: tool.name, arguments: JSON.stringify(tool.input) } }] } : { role: "assistant", content: text };
          response.write(`data: ${JSON.stringify({ ...envelope, object: "chat.completion.chunk", choices: [{ index: 0, delta, finish_reason: null }] })}\n\n`);
          await wait();
          if (response.destroyed) return;
          response.end(`data: ${JSON.stringify({ ...envelope, object: "chat.completion.chunk", choices: [{ index: 0, delta: {}, finish_reason: tool ? "tool_calls" : "stop" }], usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } })}\n\ndata: [DONE]\n\n`);
        } else response.end(JSON.stringify({ ...envelope, object: "chat.completion", choices: [{ index: 0, message: { role: "assistant", content: text }, finish_reason: "stop" }], usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } }));
      }
    } catch {
      if (response.headersSent) response.destroy();
      else { response.writeHead(400); response.end(JSON.stringify({ error: "Invalid controlled fixture input" })); }
    }
  });
  await new Promise<void>((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  const address = server.address();
  if (address === null || typeof address === "string") throw Error("Fixture listener unavailable");
  return {
    origin: `http://127.0.0.1:${address.port}`,
    events: events as readonly ProviderEvent[],
    close: () => new Promise<void>((resolve, reject) => {
      if (!server.listening) { resolve(); return; }
      server.close((error) => error ? reject(error) : resolve());
      server.closeAllConnections();
    })
  };
}

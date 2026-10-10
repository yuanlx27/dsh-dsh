import { RemoteStreamMuxClient } from "../../runtime/node_modules/@deepseek-ai/dsh-api-gateway/lib/types/client/stream-client.js";
import { parseRemoteStreamClientMessage, type RemoteStreamClientMessage } from "../../runtime/node_modules/@deepseek-ai/dsh-api-gateway/lib/types/stream-protocol.js";

/** Capture actual pinned-R SDK serialization, with a fake socket and public data.
 * This is NOT a live R backend/task trace. No network or credential is used. */
export async function recordRRequests() {
  const records: Array<Extract<RemoteStreamClientMessage, { type: "open" }>> = [];
  class CaptureSocket extends EventTarget {
    static OPEN = 1;
    readyState = 0;
    constructor(_url: string) {
      super();
      queueMicrotask(() => { this.readyState = 1; this.dispatchEvent(new Event("open")); });
    }
    send(text: string) {
      const frame = parseRemoteStreamClientMessage(text);
      if (frame.type !== "open") return;
      records.push(frame);
      queueMicrotask(() => {
        this.dispatchEvent(new MessageEvent("message", { data: JSON.stringify({ type: "item", streamId: frame.streamId, value: { receipt: true } }) }));
        this.dispatchEvent(new MessageEvent("message", { data: JSON.stringify({ type: "end", streamId: frame.streamId }) }));
      });
    }
    close() { this.readyState = 3; this.dispatchEvent(new Event("close")); }
  }
  const names = ["WebSocket", "document", "__DSH_TRANSPORT__"] as const;
  const originals = names.map(name => Object.getOwnPropertyDescriptor(globalThis, name));
  const client = new RemoteStreamMuxClient();
  try {
    // Isolated test-process producer only; product adapter does not replace WebSocket.
    Object.defineProperty(globalThis, "WebSocket", { value: CaptureSocket, configurable: true });
    Object.defineProperty(globalThis, "document", { value: { baseURI: "https://fixture.invalid/" }, configurable: true });
    Object.defineProperty(globalThis, "__DSH_TRANSPORT__", { value: undefined, configurable: true });
    client.start();
    for (let i = 0; i < 2; i++) {
      for await (const value of client.open("fixtures/request", { fixtureText: "identical public submission" }, new AbortController().signal)) void value;
    }
    return records;
  } finally {
    await client.close();
    names.forEach((name, i) => {
      const original = originals[i];
      if (original) Object.defineProperty(globalThis, name, original);
      else Reflect.deleteProperty(globalThis, name);
    });
  }
}

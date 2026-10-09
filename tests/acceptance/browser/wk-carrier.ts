// Generated standalone probe for host-selected debug carrier-only page.
// Real R mux, unknown endpoint only. No model/tool/task request or raw reply logging.
void (async () => {
  type Handler = { postMessage(body: string): Promise<unknown> };
  const page = window as Window & { webkit?: { messageHandlers: Record<string, Handler> } };
  if (document.querySelector("h1")?.textContent !== "Carrier acceptance fixture") throw Error("Carrier-only fixture required.");
  const params = new URLSearchParams(location.hash.slice(1));
  const windowGeneration = Number(params.get("desktop-window"));
  const runtimeGeneration = Number(params.get("desktop-runtime"));
  const handler = page.webkit?.messageHandlers[`dshBridge${windowGeneration}`];
  if (!handler || !windowGeneration || !runtimeGeneration) throw Error("Native fixture unavailable.");
  const post = handler.postMessage.bind(handler);
  async function call(operation: string, payload: unknown, generation = windowGeneration): Promise<unknown> {
    const reply = await post(JSON.stringify({ windowGeneration: generation, runtimeGeneration, operation, payload }));
    if (typeof reply !== "string") throw Error("Invalid native reply.");
    return JSON.parse(reply) as unknown;
  }
  const key = "desktop-fixture-carrier-handle";
  const late = "desktop-fixture-carrier-old-read-resolved";
  const rows: Array<{ scenario: string; outcome: string }> = [];
  async function observe(scenario: string, work: () => Promise<unknown>) {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const outcome = await Promise.race([
        work().then(() => "resolved", (error: unknown) => {
          const message = typeof error === "object" && error !== null && "message" in error ? error.message : error;
          return typeof message === "string" && message.includes("Desktop request denied") ? "desktop-denied" : "other-rejection";
        }),
        new Promise<string>(resolve => { timer = setTimeout(() => resolve("timeout"), 1500); }),
      ]);
      rows.push({ scenario, outcome });
    } finally { clearTimeout(timer); }
  }
  async function begin(): Promise<string> {
    const value = await call("stream", { action: "begin" });
    if (!value || typeof value !== "object" || !("handle" in value) || typeof value.handle !== "string") throw Error("Native carrier unavailable.");
    return value.handle;
  }
  async function roundTrip(handle: string): Promise<void> {
    const frame = JSON.stringify({ type: "open", streamId: "fixture-lifetime", endpoint: "_desktop_fixture/unknown", payload: {} });
    await call("stream", { action: "write", handle, sequence: 0, chunk: 0, data: btoa(frame), end: true });
    const packet = await call("stream", { action: "read", handle });
    if (!packet || typeof packet !== "object" || !("sequence" in packet) || !("kind" in packet) || !("end" in packet) || !("data" in packet) || packet.sequence !== 0 || packet.kind !== "text" || packet.end !== true || typeof packet.data !== "string" || atob(packet.data).length === 0) throw Error("Invalid native carrier chunk.");
  }
  const raw = localStorage.getItem(key);
  if (raw === null) {
    localStorage.removeItem(late);
    const stream = await begin();
    await roundTrip(stream);
    localStorage.setItem(key, JSON.stringify({ windowGeneration, runtimeGeneration, stream }));
    rows.push({ scenario: "old-carrier-unknown-endpoint-roundtrip", outcome: "resolved" });
    await observe("same-document-second-carrier", () => begin());
    void call("stream", { action: "read", handle: stream }).then(() => localStorage.setItem(late, "resolved"), () => {});
    await new Promise(resolve => setTimeout(resolve, 500));
    rows.push({ scenario: "old-read-before-destruction", outcome: localStorage.getItem(late) === null ? "pending" : "resolved" });
  } else {
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== "object" || !("windowGeneration" in value) || !("runtimeGeneration" in value) || !("stream" in value) || typeof value.windowGeneration !== "number" || typeof value.runtimeGeneration !== "number" || typeof value.stream !== "string") throw Error("Invalid fixture metadata.");
    const oldGeneration = value.windowGeneration;
    if (oldGeneration === windowGeneration) throw Error("Native document was not replaced.");
    rows.push({ scenario: "actual-window-generation-changed", outcome: "confirmed" });
    rows.push({ scenario: "runtime-generation-retained", outcome: value.runtimeGeneration === runtimeGeneration ? "confirmed" : "changed" });
    await observe("fresh-main-positive", () => call("bootstrap", null));
    await observe("actual-old-generation", () => call("bootstrap", null, oldGeneration));
    await observe("old-stream-read", () => call("stream", { action: "read", handle: value.stream }));
    await observe("old-stream-write", () => call("stream", { action: "write", handle: value.stream, sequence: 1, chunk: 0, data: "", end: true }));
    await observe("fresh-carrier-positive", async () => {
      const stream = await begin();
      try { await roundTrip(stream); } finally { await call("stream", { action: "cancel", handle: stream }); }
    });
    rows.push({ scenario: "old-read-resolution-marker", outcome: localStorage.getItem(late) === null ? "absent" : "present" });
    await observe("fresh-main-positive-after-negatives", () => call("bootstrap", null));
    localStorage.removeItem(key);
    localStorage.removeItem(late);
  }
  const node = document.createElement("pre");
  node.id = "desktop-wk-carrier-results";
  node.setAttribute("role", "status");
  node.textContent = JSON.stringify({ windowGeneration, runtimeGeneration, rows });
  document.body.append(node);
})();

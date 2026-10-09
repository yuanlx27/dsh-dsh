// Compile outside the repository. Run before and after HOST-ONLY native destruction.
// HTTP begin buffers only; the extra mux connection sends no business operation.
void (async () => {
  type Handler = { postMessage(body: string): Promise<unknown> };
  const page = window as Window & { webkit?: { messageHandlers: Record<string, Handler> } };
  const params = new URLSearchParams(location.hash.slice(1));
  const windowGeneration = Number(params.get("desktop-window"));
  const runtimeGeneration = Number(params.get("desktop-runtime"));
  const handler = page.webkit?.messageHandlers[`dshBridge${windowGeneration}`];
  if (!handler || !windowGeneration || !runtimeGeneration) throw Error("Entered fixture Harness required.");
  const call = (operation: string, payload: unknown, generation = windowGeneration) => handler.postMessage(JSON.stringify({ windowGeneration: generation, runtimeGeneration, operation, payload }));
  const key = "desktop-fixture-stale-handles";
  const late = "desktop-fixture-old-read-resolved";
  interface Saved { windowGeneration: number; runtimeGeneration: number; http: string; stream: string }
  interface Row { scenario: string; outcome: string }
  const rows: Row[] = [];
  const render = () => {
    document.getElementById("desktop-wk-stale-results")?.remove();
    const node = document.createElement("pre");
    node.id = "desktop-wk-stale-results";
    node.setAttribute("role", "status");
    node.textContent = JSON.stringify({ windowGeneration, runtimeGeneration, rows });
    document.body.append(node);
  };
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
  function handle(value: unknown): string {
    if (!value || typeof value !== "object" || !("handle" in value) || typeof value.handle !== "string" || !value.handle) throw Error("Native handle unavailable.");
    return value.handle;
  }
  const input = { action: "begin", path: "/api/_desktop_fixture_lifetime", method: "GET", headers: [] };
  const raw = localStorage.getItem(key);
  if (raw === null) {
    localStorage.removeItem(late);
    const http = handle(await call("http", input));
    const stream = handle(await call("stream", { action: "begin" }));
    const saved: Saved = { windowGeneration, runtimeGeneration, http, stream };
    localStorage.setItem(key, JSON.stringify(saved)); // Only scoped opaque handles, never authentication/bootstrap values.
    void call("stream", { action: "read", handle: stream }).then(() => localStorage.setItem(late, "resolved"), () => {});
    await new Promise(resolve => setTimeout(resolve, 500));
    rows.push({ scenario: "old-http-and-stream-created", outcome: "resolved" });
    rows.push({ scenario: "old-read-before-destruction", outcome: localStorage.getItem(late) === null ? "pending" : "resolved" });
  } else {
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== "object" || !("windowGeneration" in value) || !("runtimeGeneration" in value) || !("http" in value) || !("stream" in value) || typeof value.windowGeneration !== "number" || typeof value.runtimeGeneration !== "number" || typeof value.http !== "string" || typeof value.stream !== "string") throw Error("Invalid fixture metadata.");
    const oldWindowGeneration = value.windowGeneration;
    if (oldWindowGeneration === windowGeneration) throw Error("Native document was not replaced.");
    rows.push({ scenario: "actual-window-generation-changed", outcome: "confirmed" });
    rows.push({ scenario: "runtime-generation-retained", outcome: value.runtimeGeneration === runtimeGeneration ? "confirmed" : "changed" });
    await observe("fresh-main-positive", () => call("bootstrap", null));
    await observe("actual-old-generation", () => call("bootstrap", null, oldWindowGeneration));
    await observe("old-http-finish", () => call("http", { action: "finish", handle: value.http }));
    await observe("old-http-write", () => call("http", { action: "write", handle: value.http, sequence: 0, data: "AA==" }));
    await observe("old-stream-read", () => call("stream", { action: "read", handle: value.stream }));
    await observe("old-stream-write", () => call("stream", { action: "write", handle: value.stream, sequence: 0, chunk: 0, data: "", end: true }));
    rows.push({ scenario: "old-read-resolution-marker", outcome: localStorage.getItem(late) === null ? "absent" : "present" });
    await observe("fresh-http-positive", async () => {
      const fresh = handle(await call("http", input));
      await call("http", { action: "cancel", handle: fresh });
    });
    await observe("fresh-main-positive-after-negatives", () => call("bootstrap", null));
    localStorage.removeItem(key);
    localStorage.removeItem(late);
  }
  render();
})();

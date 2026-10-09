// Compile outside the repository. Run before and after HOST-ONLY native destruction.
// Begin buffers a harmless unknown GET only; no model/task/provider operation.
void (async () => {
  type Handler = { postMessage(body: string): Promise<unknown> };
  const page = window as Window & { webkit?: { messageHandlers: Record<string, Handler> } };
  const params = new URLSearchParams(location.hash.slice(1));
  const windowGeneration = Number(params.get("desktop-window"));
  const runtimeGeneration = Number(params.get("desktop-runtime"));
  const handler = page.webkit?.messageHandlers[`dshBridge${windowGeneration}`];
  if (!handler || !windowGeneration || !runtimeGeneration) throw Error("Entered fixture Harness required.");
  const call = async (operation: string, payload: unknown, generation = windowGeneration): Promise<unknown> => {
    const reply = await handler.postMessage(JSON.stringify({ windowGeneration: generation, runtimeGeneration, operation, payload }));
    if (typeof reply !== "string") throw Error("Invalid native reply.");
    return JSON.parse(reply) as unknown;
  };
  const key = "desktop-fixture-stale-handles";
  interface Row { scenario: string; outcome: string }
  const rows: Row[] = [];
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
    const http = handle(await call("http", input));
    localStorage.setItem(key, JSON.stringify({ windowGeneration, runtimeGeneration, http })); // Scoped opaque handle only, never authentication/bootstrap values.
    rows.push({ scenario: "old-http-created", outcome: "resolved" });
  } else {
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== "object" || !("windowGeneration" in value) || !("runtimeGeneration" in value) || !("http" in value) || typeof value.windowGeneration !== "number" || typeof value.runtimeGeneration !== "number" || typeof value.http !== "string") throw Error("Invalid fixture metadata.");
    const oldWindowGeneration = value.windowGeneration;
    if (oldWindowGeneration === windowGeneration) throw Error("Native document was not replaced.");
    rows.push({ scenario: "actual-window-generation-changed", outcome: "confirmed" });
    rows.push({ scenario: "runtime-generation-retained", outcome: value.runtimeGeneration === runtimeGeneration ? "confirmed" : "changed" });
    await observe("fresh-main-positive", () => call("bootstrap", null));
    await observe("actual-old-generation", () => call("bootstrap", null, oldWindowGeneration));
    await observe("old-http-finish", () => call("http", { action: "finish", handle: value.http }));
    await observe("old-http-write", () => call("http", { action: "write", handle: value.http, sequence: 0, data: "AA==" }));
    await observe("fresh-http-positive", async () => {
      const fresh = handle(await call("http", input));
      await call("http", { action: "cancel", handle: fresh });
    });
    await observe("fresh-main-positive-after-negatives", () => call("bootstrap", null));
    localStorage.removeItem(key);
  }
  document.getElementById("desktop-wk-stale-results")?.remove();
  const node = document.createElement("pre");
  node.id = "desktop-wk-stale-results";
  node.setAttribute("role", "status");
  node.textContent = JSON.stringify({ windowGeneration, runtimeGeneration, rows });
  document.body.append(node);
})();

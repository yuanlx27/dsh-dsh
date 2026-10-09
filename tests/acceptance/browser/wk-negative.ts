// Compile to ignored JavaScript; paste the GENERATED file in the owned WK Inspector.
// No service/API/provider operation is submitted; successful bootstrap is discarded.
void (async () => {
  type Page = Window & { webkit?: { messageHandlers: Record<string, { postMessage(body: string): Promise<unknown> }> }; __TAURI_INTERNALS__?: { invoke(command: string, args?: Record<string, unknown>): Promise<unknown> } };
  const page = window as Page;
  const fragment = new URLSearchParams(location.hash.slice(1));
  const windowGeneration = Number(fragment.get("desktop-window"));
  const runtimeGeneration = Number(fragment.get("desktop-runtime"));
  if (!windowGeneration || !runtimeGeneration) throw Error("Run only in entered fixture Harness.");
  const name = `dshBridge${windowGeneration}`;
  const envelope = { windowGeneration, runtimeGeneration, operation: "bootstrap", payload: null };
  interface Outcome { outcome: string; failure?: string }
  const rows: Array<Outcome & { scenario: string }> = [];
  function failure(error: unknown): string {
    const message = typeof error === "object" && error !== null && "message" in error ? error.message : undefined;
    const text = typeof error === "string" ? error : typeof message === "string" ? message : undefined;
    if (text === undefined) return "unclassified";
    if (text.includes("Desktop request denied")) return "desktop-denied";
    if (/command.*not found|unknown command|plugin not found/i.test(text)) return "command-missing";
    if (/invalid args|invalid argument|missing required/i.test(text)) return "invalid-arguments";
    if (text.includes("Permissions associated with this command") && text.includes("not allowed")) return "acl-denied";
    return "unclassified";
  }
  async function observe(scenario: string, work: () => Promise<unknown>) {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const outcome = await Promise.race([
        work().then<Outcome, Outcome>(() => ({ outcome: "resolved" }), (error: unknown) => ({ outcome: "rejected", failure: failure(error) })),
        new Promise<Outcome>((resolve) => { timer = setTimeout(() => resolve({ outcome: "timeout" }), 1500); })
      ]);
      rows.push({ scenario, ...outcome });
    } finally { clearTimeout(timer); }
  }
  const handler = page.webkit?.messageHandlers[name];
  if (!handler) throw Error("Harness handler absent.");
  await observe("main-positive", () => handler.postMessage(JSON.stringify(envelope)));
  await observe("wrong-window-generation", () => handler.postMessage(JSON.stringify({ ...envelope, windowGeneration: windowGeneration + 1 })));
  await observe("wrong-runtime-generation", () => handler.postMessage(JSON.stringify({ ...envelope, runtimeGeneration: runtimeGeneration + 1 })));
  await observe("forged-main-frame-label", () => handler.postMessage(JSON.stringify({ ...envelope, mainFrame: true })));
  await observe("oversized-envelope", () => handler.postMessage("x".repeat(512 * 1024 + 1)));
  const frame = document.createElement("iframe");
  document.body.append(frame); // Initial inherited about:blank, no remote navigation/script.
  const child = frame.contentWindow as Page | null;
  const childHandler = child?.webkit?.messageHandlers[name];
  if (childHandler) await observe("inherited-child-same-envelope", () => childHandler.postMessage(JSON.stringify(envelope)));
  else rows.push({ scenario: "inherited-child-same-envelope", outcome: "handler-absent" });
  frame.remove();
  const invoke = page.__TAURI_INTERNALS__?.invoke;
  if (invoke) {
    for (const command of ["plugin:window|get_all_windows", "plugin:webview|get_all_webviews", "plugin:event|listen"]) {
      await observe(command, () => invoke(command, { event: "fixture-negative", target: { kind: "Any" }, handler: 0 }));
    }
  } else rows.push({ scenario: "general-ipc", outcome: "handler-absent" });
  await observe("main-positive-after-negatives", () => handler.postMessage(JSON.stringify(envelope)));
  document.getElementById("desktop-wk-negative-results")?.remove();
  const result = document.createElement("pre");
  result.id = "desktop-wk-negative-results";
  result.setAttribute("role", "status");
  result.textContent = JSON.stringify(rows);
  document.body.append(result);
  console.log("DESKTOP-WK-NEGATIVE", JSON.stringify(rows));
})();

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
  const rows: Array<{ scenario: string; outcome: string }> = [];
  async function observe(scenario: string, work: () => Promise<unknown>) {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const outcome = await Promise.race([
        work().then(() => "resolved", () => "rejected"),
        new Promise<string>((resolve) => { timer = setTimeout(() => resolve("timeout"), 1500); })
      ]);
      rows.push({ scenario, outcome });
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
  console.log("DESKTOP-WK-NEGATIVE", JSON.stringify(rows));
})();

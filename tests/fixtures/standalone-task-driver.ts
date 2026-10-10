import { randomUUID } from "node:crypto";
import { setTimeout as delay } from "node:timers/promises";
import { controlledModel } from "./controlled-model.ts";
import type { runProfile as RunProfile } from "../../runtime/node_modules/@deepseek-ai/dsh/lib/types/profile-boot.d.ts";
import type { createLaunchEnvironmentSnapshot as Snapshot } from "../../runtime/node_modules/@deepseek-ai/dsh-launch-environment/lib/types/index.d.ts";
import type { brandString as BrandString } from "../../runtime/node_modules/@deepseek-ai/dsh-brand/lib/types/index.d.ts";
import { credentialRef } from "../../runtime/node_modules/@deepseek-ai/dsh-credentials/lib/types/index.js";
import type { SessionRequestId } from "../../runtime/node_modules/@deepseek-ai/dsh-api-session-controller/lib/types/index.d.ts";
import type {} from "../../runtime/node_modules/@deepseek-ai/dsh-settings/lib/types/index.d.ts";

// Bind pinned upstream exports to their published declarations, checking the runtime boundary.
async function functionExport<T extends (...args: never[]) => unknown>(path: string, name: string): Promise<T> {
  const module: unknown = await import(new URL(path, import.meta.url).href);
  if (!module || typeof module !== "object" || !(name in module) || typeof Reflect.get(module, name) !== "function") throw Error("Missing pinned export");
  return Reflect.get(module, name) as T;
}
const runProfile = await functionExport<typeof RunProfile>("../../runtime/node_modules/@deepseek-ai/dsh/lib/profile-boot.js", "runProfile");
const snapshot = await functionExport<typeof Snapshot>("../../runtime/node_modules/@deepseek-ai/dsh-launch-environment/lib/index.js", "createLaunchEnvironmentSnapshot");
const brandString = await functionExport<typeof BrandString>("../../runtime/node_modules/@deepseek-ai/dsh-brand/lib/index.js", "brandString");
const kind = process.argv[2];
if (kind !== "complete" && kind !== "stop") throw Error("Unknown standalone case");
if (process.version !== "v24.21.0") throw Error("Standalone requires pinned Node");
const fetch = globalThis.fetch;
let blockedExternal = 0;
globalThis.fetch = (input, init) => {
  const url = new URL(input instanceof Request ? input.url : String(input));
  if (url.hostname !== "127.0.0.1") { blockedExternal++; throw Error("Fixture refuses non-loopback provider traffic"); }
  return fetch(input, { ...init, redirect: "error" });
};
let release = () => {};
const streamGate = new Promise<void>(resolve => { release = resolve; });
const provider = await controlledModel({ streamGate });
let ctx: Awaited<ReturnType<typeof RunProfile>>["ctx"] | undefined;
let stage = "boot";
try {
  const boot = await runProfile({
    profile: "web", patchFiles: [],
    args: ["--host", "127.0.0.1", "--port", "0", "--no-open"],
    environment: snapshot([{ source: "process", values: Object.fromEntries(Object.entries(process.env).filter((entry): entry is [string, string] => entry[1] !== undefined)) }])
  });
  ctx = boot.ctx;
  stage = "configure";
  const descriptor = ctx.settings.describe({ redactSecrets: true }).find(entry => entry.ns === "llm-deepseek");
  if (!descriptor) throw Error("Official provider unavailable");
  await ctx.settings.update(descriptor.ns, { baseURL: `${provider.origin}/v1` }, descriptor.revision);
  const configured = ctx.settings.describe({ redactSecrets: true }).find(entry => entry.ns === descriptor.ns)?.value;
  if (!configured || typeof configured !== "object" || !("baseURL" in configured) || configured.baseURL !== `${provider.origin}/v1`) throw Error("Endpoint was not configured");
  await ctx.credentials.set(credentialRef("DEEPSEEK_API_KEY"), "desktop-fixture-first");
  if ((await ctx.credentials.resolve(credentialRef("DEEPSEEK_API_KEY")))?.value !== "desktop-fixture-first") throw Error("Fixture credential did not resolve");
  const catalog = await ctx.sessionController.modelCatalog();
  const model = catalog.groups.find(group => group.id === "deepseek-official")?.models[0];
  if (!model) throw Error("Official model unavailable");
  stage = "start";
  const { sessionId } = await ctx.sessionController.create({ cwd: process.cwd() });
  await ctx.sessionController.selectModel({ sessionId, provider: "deepseek-official", model: model.id });
  const requestId = brandString<SessionRequestId>(randomUUID());
  const admission = await ctx.sessionController.prompt({ sessionId, requestId, mode: "queue", content: [{ type: "text", text: "Public controlled fixture request. No actions requested." }] }, new AbortController().signal);
  const owner = ctx;
  async function waitRunning(expected: boolean) {
    const deadline = Date.now() + 10000;
    while (Date.now() < deadline) {
      const rows = await owner.sessionController.list({}, new AbortController().signal);
      if (rows.items.some(row => row.sessionId === sessionId && row.running === expected) && (!expected || provider.events.some(event => event.operation === "messages"))) return;
      await delay(20);
    }
    throw Error("Task state deadline");
  }
  stage = "running";
  await waitRunning(true);
  stage = kind === "stop" ? "cancel" : "release";
  if (kind === "stop") ctx.sessionController.cancel({ sessionId });
  else release();
  stage = "settle";
  await waitRunning(false);
  const inspection = await ctx.sessionController.inspect(sessionId);
  const count = (type: string) => inspection.events.filter(event => event.type === type).length;
  const result = {
    accepted: admission.accepted, runningObserved: true, settled: true, loopbackOnly: blockedExternal === 0,
    endExpected: inspection.events.some(event => event.type === "turn/end" && event.data.reason.kind === (kind === "complete" ? "completed" : "aborted")),
    providerAuthenticated: provider.events.length > 0 && provider.events.every(event => event.credential === "first"),
    promptCount: inspection.events.filter(event => event.type === "user/message" && "rpcId" in event.data.source && event.data.source.rpcId === requestId).length, turnStarts: count("turn/start"), turnEnds: count("turn/end"),
    assistantMessages: count("assistant/message"),
    prefixPreserved: inspection.events.some(event => event.type === "assistant/message" && event.data.message.content.some(part => part.type === "text" && part.text.length > 0)),
    interrupted: inspection.events.some(event => event.type === "assistant/message" && event.data.interrupted === true), disposed: true
  };
  stage = "dispose";
  release();
  await provider.close();
  await ctx.fiber.dispose();
  ctx = undefined;
  process.stdout.write(`DSH_TASK_RESULT ${JSON.stringify(result)}\n`);
} catch {
  process.stdout.write(`DSH_TASK_FAILURE ${stage}\n`);
  process.exitCode = 1;
} finally {
  release();
  await provider.close();
  await ctx?.fiber.dispose();
  globalThis.fetch = fetch;
}

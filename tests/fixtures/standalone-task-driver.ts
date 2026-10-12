import { randomUUID } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { controlledModel } from "./controlled-model.ts";
import type { runProfile as RunProfile } from "../../runtime/node_modules/@deepseek-ai/dsh/lib/types/profile-boot.d.ts";
import type { createLaunchEnvironmentSnapshot as Snapshot } from "../../runtime/node_modules/@deepseek-ai/dsh-launch-environment/lib/types/index.d.ts";
import type { brandString as BrandString } from "../../runtime/node_modules/@deepseek-ai/dsh-brand/lib/types/index.d.ts";
import { credentialRef } from "../../runtime/node_modules/@deepseek-ai/dsh-credentials/lib/types/index.js";
import type { SessionRequestId } from "../../runtime/node_modules/@deepseek-ai/dsh-api-session-controller/lib/types/index.d.ts";
import type { SessionId } from "../../runtime/node_modules/@deepseek-ai/dsh-session/lib/types/index.d.ts";
import type {} from "../../runtime/node_modules/@deepseek-ai/dsh-settings/lib/types/index.d.ts";
import type {} from "../../runtime/node_modules/@deepseek-ai/dsh-permission-presets/lib/types/index.d.ts";
import type {} from "../../runtime/node_modules/@deepseek-ai/dsh-sandbox-policy/lib/types/index.d.ts";
import type { ApprovalOutcome } from "../../runtime/node_modules/@deepseek-ai/dsh-user-approval/lib/types/index.d.ts";
import type {} from "../../runtime/node_modules/@deepseek-ai/dsh-jobs/lib/types/index.d.ts";
import type { ToolCallId } from "../../runtime/node_modules/@deepseek-ai/dsh-llm/lib/types/index.d.ts";

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
if (kind !== "complete" && kind !== "stop" && kind !== "allow" && kind !== "deny" && kind !== "withdraw" && kind !== "action-stop" && kind !== "queue-steer" && kind !== "queue-stop" && kind !== "multi-session-stop") throw Error("Unknown standalone case");
const toolCase = kind === "allow" || kind === "deny" || kind === "withdraw" || kind === "action-stop";
const actionStop = kind === "action-stop";
const cancelsTurn = kind === "stop" || kind === "withdraw" || actionStop || kind === "queue-stop" || kind === "multi-session-stop";
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
let answer = (_outcome: ApprovalOutcome) => {};
const approvalGate = new Promise<ApprovalOutcome>(resolve => { answer = resolve; });
const provider = await controlledModel(toolCase ? { tool: { name: "bash", input: {
  command: actionStop ? "printf '%s\\n' started > stop-marker.txt; sleep 20; printf '%s\\n' finished >> stop-marker.txt" : "printf '%s\\n' approved > approval-marker.txt", description: "Write only the disposable fixture marker",
  workdir: process.cwd(), timeoutMs: actionStop ? 30000 : 10000, run_in_background: false,
  sandbox_permissions: "workspace-write", justification: "Write only the reviewed marker in the disposable workspace."
} } } : { streamGate });
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
  let gateObserved = false;
  let pendingNoEffect = false;
  let gateCallId: ToolCallId | undefined;
  const resolved = await ctx.sessionController.resolveAgent(sessionId);
  if (!("agent" in resolved)) throw Error("Fixture agent unavailable");
  const agent = resolved.agent;
  if (toolCase) {
    ctx.permissionPresets.set(agent.session, "read-only");
    if (ctx.sandboxPolicy.resolve({ session: agent.session }).mode !== "read-only" || ctx.approval.overrideOf(agent.session) !== "ask") throw Error("Fixture policy unavailable");
    // Test-only answerer on R's documented seam, NOT a browser/Remote decision.
    ctx.on("approval/request", (request, next) => {
      if (request.agent.session.id !== sessionId || request.toolName !== "bash") return next();
      if (gateObserved || request.callId === undefined) throw Error("Unexpected fixture gate");
      gateObserved = true;
      gateCallId = request.callId;
      return approvalGate;
    }, true);
  }
  await ctx.sessionController.selectModel({ sessionId, provider: "deepseek-official", model: model.id });
  const requestId = brandString<SessionRequestId>(randomUUID());
  const admission = await ctx.sessionController.prompt({ sessionId, requestId, mode: "queue", content: [{ type: "text", text: toolCase ? "Public controlled fixture: perform the reviewed disposable marker action." : "Public controlled fixture request. No actions requested." }] }, new AbortController().signal);
  const owner = ctx;
  async function waitRunning(expected: boolean, targetSessionId = sessionId, minimumOperations = 1) {
    const deadline = Date.now() + 10000;
    while (Date.now() < deadline) {
      const rows = await owner.sessionController.list({}, new AbortController().signal);
      if (rows.items.some(row => row.sessionId === targetSessionId && row.running === expected) && (!expected || provider.events.filter(event => event.operation === "messages").length >= minimumOperations) && (!toolCase || !expected || gateObserved)) return;
      await delay(20);
    }
    throw Error("Task state deadline");
  }
  stage = "running";
  await waitRunning(true);
  let other: { sessionId: SessionId; requestId: SessionRequestId } | undefined;
  let multiRunningObserved = false;
  let otherRunningAfterStop = false;
  let otherCompleted = false;
  let otherPromptCount = 0;
  if (kind === "multi-session-stop") {
    stage = "other-start";
    const cwd = process.argv[3];
    if (!cwd) throw Error("Missing prepared second workspace");
    const created = await ctx.sessionController.create({ cwd });
    other = { sessionId: created.sessionId, requestId: brandString<SessionRequestId>(randomUUID()) };
    await ctx.sessionController.selectModel({ sessionId: other.sessionId, provider: "deepseek-official", model: model.id });
    const admitted = await ctx.sessionController.prompt({ sessionId: other.sessionId, requestId: other.requestId, mode: "queue", content: [{ type: "text", text: "Public controlled second-workspace request. No actions requested." }] }, new AbortController().signal);
    await waitRunning(true, other.sessionId, 2);
    const rows = await ctx.sessionController.list({}, new AbortController().signal);
    multiRunningObserved = admitted.accepted && [sessionId, other.sessionId].every(id => rows.items.some(row => row.sessionId === id && row.running));
    if (!multiRunningObserved) throw Error("Both prepared sessions must actually run");
  }
  const queueIds: SessionRequestId[] = [];
  let queueAdmissions = 0;
  let queueSteers = 0;
  let queuePendingObserved = false;
  if (kind === "queue-steer" || kind === "queue-stop") {
    stage = "queue";
    for (let index = 0; index < 2; index++) {
      const id = brandString<SessionRequestId>(randomUUID());
      queueIds.push(id);
      const receipt = await ctx.sessionController.prompt({ sessionId, requestId: id, mode: "queue", content: [{ type: "text", text: "Controlled CU identical queued input." }] }, new AbortController().signal);
      if (receipt.accepted) queueAdmissions++;
    }
    const pending = await ctx.sessionController.inspect(sessionId);
    const queued = pending.events.flatMap(event => event.type === "agent/inbox/spliced" && event.data.target === "next-turn" ? event.data.inserted.filter(message => { const source = message.source; return "rpcId" in source && queueIds.some(id => id === source.rpcId); }) : []);
    queuePendingObserved = queued.length === 2 && queued.every(message => message.content.some(part => part.type === "text" && part.text === "Controlled CU identical queued input."));
    if (!queuePendingObserved) throw Error("Queue did not retain both public inputs");
    stage = "steer";
    for (const message of kind === "queue-steer" ? queued : []) {
      const receipt = await ctx.sessionController.updateQueue({ sessionId, itemId: message.id, action: { kind: "steer" } });
      if (receipt.accepted) queueSteers++;
    }
  }
  stage = cancelsTurn ? "cancel" : "release";
  if (toolCase) pendingNoEffect = !existsSync(join(process.cwd(), actionStop ? "stop-marker.txt" : "approval-marker.txt"));
  let actionRunningObserved = false;
  if (actionStop) {
    answer("allowed-once");
    stage = "action-start";
    const deadline = Date.now() + 10000;
    while (!existsSync(join(process.cwd(), "stop-marker.txt")) && Date.now() < deadline) await delay(20);
    if (!existsSync(join(process.cwd(), "stop-marker.txt"))) throw Error("Action did not start");
    actionRunningObserved = ctx.jobs.list(sessionId).some(job => job.kind === "bash" && job.status === "running");
    stage = "action-cancel";
    ctx.sessionController.cancel({ sessionId });
  } else if (cancelsTurn) ctx.sessionController.cancel({ sessionId });
  else if (toolCase) answer(kind === "allow" ? "allowed-once" : "rejected");
  else release();
  stage = "settle";
  await waitRunning(false);
  // A bounded late observation, not proof of every scheduling/reconnect race.
  if (kind === "queue-stop") await delay(200);
  if (other) {
    stage = "other-still-running";
    await delay(200);
    const rows = await ctx.sessionController.list({}, new AbortController().signal);
    otherRunningAfterStop = rows.items.some(row => row.sessionId === other?.sessionId && row.running);
    if (!otherRunningAfterStop) throw Error("Stop affected the other session");
    release();
    await waitRunning(false, other.sessionId);
    const otherInspection = await ctx.sessionController.inspect(other.sessionId);
    const otherRequestId = other.requestId;
    otherPromptCount = otherInspection.events.filter(event => event.type === "user/message" && "rpcId" in event.data.source && event.data.source.rpcId === otherRequestId).length;
    otherCompleted = otherInspection.events.filter(event => event.type === "turn/start").length === 1 && otherInspection.events.filter(event => event.type === "turn/end").length === 1 && otherInspection.events.some(event => event.type === "turn/end" && event.data.reason.kind === "completed") && !otherInspection.events.some(event => event.type === "tool/call" || event.type === "approval/asked");
  }
  const inspection = await ctx.sessionController.inspect(sessionId);
  const count = (type: string) => inspection.events.filter(event => event.type === type).length;
  // A cancelled ask must discard a late grant, never execute the held command.
  if (kind === "withdraw") { answer("allowed-once"); await delay(100); }
  const asked = inspection.events.find(event => event.type === "approval/asked" && event.data.callId === gateCallId);
  const decisionExpected = toolCase && asked?.type === "approval/asked" && inspection.events.some(event => event.type === "approval/decided" && event.data.id === asked.data.id && event.data.outcome === (kind === "allow" || actionStop ? "allowed-once" : kind === "deny" ? "rejected" : "cancelled"));
  const queueDelivered = inspection.events.filter(event => {
    if (event.type !== "user/message") return false;
    const source = event.data.source;
    return "rpcId" in source && queueIds.some(id => id === source.rpcId) && event.data.content.some(part => part.type === "text" && part.text === "Controlled CU identical queued input.");
  });
  const queueReceiptsPreserved = queueIds.length === 2 && queueIds.every(id => inspection.events.filter(event => event.type === "user/message" && "rpcId" in event.data.source && event.data.source.rpcId === id).length === 1);
  const queuePendingAfterStop = kind === "queue-stop" ? agent.inbox.nextTurn.length : 0;
  const queueRetainedAfterStop = kind === "queue-stop" && queuePendingAfterStop === 2 && agent.inbox.nextStep.length === 0 && queueIds.every(id => agent.inbox.nextTurn.filter(message => "rpcId" in message.source && message.source.rpcId === id && message.content.some(part => part.type === "text" && part.text === "Controlled CU identical queued input.")).length === 1);
  const queueNoAutoResume = kind === "queue-stop" && queueDelivered.length === 0 && count("turn/start") === 1 && count("turn/end") === 1 && (await ctx.sessionController.list({}, new AbortController().signal)).items.some(row => row.sessionId === sessionId && !row.running);
  const result = {
    multiRunningObserved, otherRunningAfterStop, otherCompleted, otherPromptCount,
    queuePendingAfterStop, queueRetainedAfterStop, queueNoAutoResume,
    queueAdmissions, queueSteers, queueDeliveries: queueDelivered.length,
    queuePendingObserved, queueReceiptsDistinct: queueIds.length === 2 && new Set(queueIds).size === 2, queueReceiptsPreserved,
    accepted: admission.accepted, runningObserved: true, settled: true, loopbackOnly: blockedExternal === 0,
    endExpected: inspection.events.some(event => event.type === "turn/end" && event.data.reason.kind === (cancelsTurn ? "aborted" : "completed")),
    providerAuthenticated: provider.events.length > 0 && provider.events.every(event => event.credential === "first"),
    promptCount: inspection.events.filter(event => event.type === "user/message" && "rpcId" in event.data.source && event.data.source.rpcId === requestId).length, turnStarts: count("turn/start"), turnEnds: count("turn/end"),
    assistantMessages: count("assistant/message"),
    prefixPreserved: inspection.events.some(event => event.type === "assistant/message" && event.data.message.content.some(part => part.type === "text" && part.text.length > 0)),
    interrupted: inspection.events.some(event => event.type === "assistant/message" && event.data.interrupted === true), disposed: true,
    gateObserved, pendingNoEffect, decisionExpected, actionRunningObserved,
    actionPrefixPreserved: actionStop && readFileSync(join(process.cwd(), "stop-marker.txt"), "utf8") === "started\n",
    jobsQuiescent: ctx.jobs.list(sessionId).every(job => job.status !== "running" && job.status !== "stopping"),
    policyUnchanged: toolCase && ctx.permissionPresets.current(agent.session) === "read-only" && ctx.sandboxPolicy.resolve({ session: agent.session }).mode === "read-only" && ctx.approval.overrideOf(agent.session) === "ask",
    approvalAsked: count("approval/asked"), approvalDecided: count("approval/decided"),
    toolCalls: count("tool/call"), toolResults: count("tool/result"),
    toolSucceeded: inspection.events.some(event => event.type === "tool/result" && event.data.message.toolCallId === gateCallId && event.data.message.isError !== true)
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
  answer("rejected");
  await provider.close();
  await ctx?.fiber.dispose();
  globalThis.fetch = fetch;
}

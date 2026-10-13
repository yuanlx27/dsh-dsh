// Fixture generation only: no Host, credentials, model, task engine or shell codec.
import { randomUUID, createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { parseArgs } from "node:util";
import { taskWorkspaces } from "../fixtures/task-workspaces.ts";
import { functionExport } from "../fixtures/upstream-export.ts";
import type { Context as ContextClass } from "../../runtime/node_modules/@deepseek-ai/cordis/lib/types/index.d.ts";
import type { Session as SessionClass, SessionStore as StoreClass, SessionEvent, SessionId } from "../../runtime/node_modules/@deepseek-ai/dsh-session/lib/types/index.d.ts";
import type JsonlClass from "../../runtime/node_modules/@deepseek-ai/dsh-session-persistence-jsonl/lib/types/index.d.ts";
import type { brandString as BrandString } from "../../runtime/node_modules/@deepseek-ai/dsh-brand/lib/types/index.d.ts";
import type { MessageId } from "../../runtime/node_modules/@deepseek-ai/dsh-llm/lib/types/index.d.ts";
import type { foldSessionTitle as FoldTitle } from "../../runtime/node_modules/@deepseek-ai/dsh-session-title/lib/types/index.d.ts";

interface HistorySession {
  owner: "standalone" | "product"; workspace: "A" | "B";
  home: string; store: string; cwd: string; id: SessionId; title: string;
  messageCount: number; generatedRoles: "user-only"; path: string; sha256: string;
}

async function upstream() {
  if (process.version !== "v24.21.0") throw Error("History fixtures require pinned Node 24.21.0");
  const metadata: unknown = JSON.parse(await readFile(new URL("../../runtime/node_modules/@deepseek-ai/dsh/package.json", import.meta.url), "utf8"));
  if (!metadata || typeof metadata !== "object" || Reflect.get(metadata, "version") !== "0.2.0-rc.2") throw Error("History fixtures require pinned dsh 0.2.0-rc.2");
  return {
    Context: await functionExport<typeof ContextClass>("../../runtime/node_modules/@deepseek-ai/cordis/lib/index.js", "Context"),
    Session: await functionExport<typeof SessionClass>("../../runtime/node_modules/@deepseek-ai/dsh-session/lib/index.js", "Session"),
    Store: await functionExport<typeof StoreClass>("../../runtime/node_modules/@deepseek-ai/dsh-session/lib/index.js", "SessionStore"),
    Jsonl: await functionExport<typeof JsonlClass>("../../runtime/node_modules/@deepseek-ai/dsh-session-persistence-jsonl/lib/index.js", "default"),
    brand: await functionExport<typeof BrandString>("../../runtime/node_modules/@deepseek-ai/dsh-brand/lib/index.js", "brandString"),
    foldTitle: await functionExport<typeof FoldTitle>("../../runtime/node_modules/@deepseek-ai/dsh-session-title/lib/index.js", "foldSessionTitle"),
  };
}

/** Fresh upstream readers validate durable records; never repair missing/corrupt fixtures. */
async function validate(sessions: readonly HistorySession[]) {
  const sdk = await upstream();
  for (const item of sessions) {
    const ctx = new sdk.Context();
    try {
      const storage = new sdk.Jsonl(ctx, { root: item.store });
      const handle = await storage.open(item.id, "read");
      try {
        const stored = await handle.read(); // Upstream current-format validation.
        const restored = sdk.Session.fromRestore(item.id, stored.events, handle.header, handle.inheritedEventCount, stored.eventState);
        const messages = restored.deriveMessages();
        if (handle.header.cwd !== item.cwd || messages.length !== item.messageCount || new Set(messages.map(m => m.id)).size !== item.messageCount || messages.some(m => m.role !== "user") || sdk.foldTitle(stored.events)?.title !== item.title) throw Error("History fixture upstream identity/projection mismatch");
        if (createHash("sha256").update(await readFile(item.path)).digest("hex") !== item.sha256) throw Error("History fixture artifact changed");
      } finally { await handle.close(); }
    } finally { await ctx.fiber.dispose(); }
  }
}

/** Four separate pinned-R histories: A=1,000 user messages, B=10, per owner. */
export async function createHistoryFixture() {
  const sdk = await upstream();
  const resources = await taskWorkspaces();
  const sessions: HistorySession[] = [];
  try {
    for (const owner of ["standalone", "product"] as const) {
      const home = join(resources.root, owner, "history-home");
      const dshHome = owner === "product" ? join(home, "Library", "Application Support", "io.github.yuanlx27.dsh-dsh", "dsh") : join(home, ".dsh");
      const root = join(dshHome, "sessions");
      await mkdir(home, { recursive: true, mode: 0o700 });
      const ctx = new sdk.Context();
      try {
        const store = new sdk.Store(ctx);
        const storage = new sdk.Jsonl(ctx, { root });
        for (const workspace of ["A", "B"] as const) {
          const messageCount = workspace === "A" ? 1000 : 10;
          const title = `Public generated history ${workspace} ${messageCount}`;
          const session = store.create(sdk.brand<SessionId>(randomUUID()), { meta: { cwd: resources[owner][workspace] } });
          const events: SessionEvent[] = [session.append("session/title", { title, source: { kind: "user" }, messageSeqs: [] })];
          // Synthetic user-only histories, not claimed as executed model/task turns.
          for (let i = 0; i < messageCount; i++) {
            events.push(session.append("turn/start", { turn: i + 1 }));
            events.push(session.append("step/start", { turn: i + 1, step: 1 }));
            events.push(session.append("user/message", { id: sdk.brand<MessageId>(randomUUID()), role: "user", source: { kind: "user" }, content: [{ type: "text", text: `Public fixture record ${workspace} ${i + 1}. 中文 / plain generated history; no real project data.` }] }, { surfaceOp: "append" }));
            events.push(session.append("step/end", { turn: i + 1, step: 1 }));
            events.push(session.append("turn/end", { turn: i + 1, reason: { kind: "completed" } }));
          }
          const handle = await storage.create(session.header);
          try { await handle.append(events); await handle.flush(); } finally { await handle.close(); }
          const path = await storage.resolveCurrentLog(session.id);
          if (!path) throw Error("Upstream did not materialize a fixture artifact");
          sessions.push({ owner, workspace, home, store: root, cwd: resources[owner][workspace], id: session.id, title, messageCount, generatedRoles: "user-only", path, sha256: createHash("sha256").update(await readFile(path)).digest("hex") });
        }
      } finally { await ctx.fiber.dispose(); }
    }
    await validate(sessions);
    const inventory = join(resources.root, "history-fixture.json");
    await writeFile(inventory, JSON.stringify({ runtime: "0.2.0-rc.2", source: "public generated user-only SDK fixture; not actual task execution", sessions }, null, 2) + "\n", { mode: 0o600 });
    return { root: resources.root, sessions, inventory, validate: () => validate(sessions), close: resources.close };
  } catch (error) { await resources.close(); throw error; }
}

if (import.meta.main) {
  const { values } = parseArgs({ options: { help: { type: "boolean", short: "h" } } });
  if (values.help) process.stdout.write("Generate public pinned-R SDK histories in NEW disposable roots only. No keys/network/Host/model engine.\nUsage: node tests/acceptance/create-history-fixture.ts\nA=1,000 user-only messages, B=10, separate standalone/product homes. stdout contains inventory path, not raw records. Retain for acceptance; remove only the reported owned root when finished.\n");
  else { const fixture = await createHistoryFixture(); process.stdout.write(JSON.stringify({ root: fixture.root, inventory: fixture.inventory, sessions: fixture.sessions.length }) + "\n"); }
}

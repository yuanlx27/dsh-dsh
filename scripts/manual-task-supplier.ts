import { parseArgs } from "node:util";
import { readFile, writeFile, unlink, stat } from "node:fs/promises";
import { isAbsolute, dirname, join } from "node:path";
import { controlledModel, type ControlledModelOptions } from "../tests/fixtures/controlled-model.ts";

async function main() {
  const { values } = parseArgs({ options: { workspace: { type: "string" }, control: { type: "string" }, ready: { type: "string" }, port: { type: "string" }, help: { type: "boolean", short: "h" } } });
  if (values.help) {
    process.stdout.write("Loopback-only desktop task supplier; no real keys or provider calls.\nUsage: node scripts/manual-task-supplier.ts --workspace PRIVATE_A --control PRIVATE_COMMAND_FILE --ready PRIVATE_STATUS_FILE [--port OWNED_PORT]\nPort defaults to an ephemeral loopback port; use an owned released port only for replacement.\nHost control: {\"scenario\":\"approval\"|\"action-stop\"|\"stream\"|\"finish\"|\"close\"}.\nRearming refuses active requests. Finish releases a held text stream; close stops only this supplier.\n");
    return;
  }
  const { workspace, control, ready } = values;
  const port = values.port === undefined ? 0 : Number(values.port);
  if (!Number.isInteger(port) || port < 0 || port > 65535) throw Error("Use an integer listener port from 0 to 65535. See --help.");
  if (!workspace || !control || !ready || ![workspace, control, ready].every(isAbsolute) || dirname(control) !== dirname(ready)) throw Error("Provide absolute workspace/control/ready paths; control and ready must share a private directory. See --help.");
  for (const path of [workspace, dirname(control)]) {
    const metadata = await stat(path);
    if (!metadata.isDirectory() || metadata.uid !== process.getuid?.() || (metadata.mode & 0o077) !== 0) throw Error("Use owner-only disposable directories, not a real project.");
  }
  if (!(await readFile(join(workspace, "README.md"), "utf8")).startsWith("# Disposable product workspace\n")) throw Error("Use the prepared product fixture workspace.");
  let release = () => {};
  const supplier = await controlledModel({}, port);
  const options = (scenario: string): ControlledModelOptions => {
    if (scenario === "approval" || scenario === "action-stop") return { tool: { name: "bash", input: {
      command: scenario === "approval" ? "printf '%s\\n' approved > approval-marker.txt" : "printf '%s\\n' started > stop-marker.txt; sleep 20; printf '%s\\n' finished >> stop-marker.txt",
      description: "Write only the disposable fixture marker", workdir: workspace,
      timeoutMs: scenario === "approval" ? 10000 : 30000, run_in_background: false,
      sandbox_permissions: "workspace-write", justification: "Write only the reviewed marker in the disposable workspace."
    } } };
    if (scenario === "stream") return { streamGate: new Promise<void>(resolve => { release = resolve; }) };
    throw Error("Unknown scenario. Use approval, action-stop, stream, finish or close.");
  };
  try {
    supplier.arm(options("approval"));
    await writeFile(ready, JSON.stringify({ origin: supplier.origin, scenario: "approval", workspace }) + "\n", { mode: 0o600 });
    process.stdout.write(JSON.stringify({ event: "ready", origin: supplier.origin, scenario: "approval" }) + "\n");
  } catch (error) { await supplier.close(); throw error; }
  let stop = () => {};
  const stopped = new Promise<void>(resolve => { stop = resolve; });
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);
  let busy = false;
  const timer = setInterval(() => {
    if (busy) return;
    busy = true;
    void (async () => {
      let content: string;
      try { content = await readFile(control, "utf8"); }
      catch (error) { if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") return; throw error; }
      await unlink(control);
      if (content.length > 1024) throw Error("Control file too large; no change made.");
      const command: unknown = JSON.parse(content);
      if (!command || typeof command !== "object" || Object.keys(command).length !== 1 || !("scenario" in command) || typeof command.scenario !== "string") throw Error("Invalid control file; no change made.");
      if (command.scenario === "close") { stop(); return; }
      if (command.scenario === "finish") { release(); return; }
      const previousRelease = release;
      const next = options(command.scenario);
      try { supplier.arm(next); }
      catch (error) { release = previousRelease; throw error; }
      previousRelease();
      await writeFile(ready, JSON.stringify({ origin: supplier.origin, scenario: command.scenario, workspace }) + "\n", { mode: 0o600 });
      process.stdout.write(JSON.stringify({ event: "armed", scenario: command.scenario, operations: supplier.events.length }) + "\n");
    })().catch(() => { process.stderr.write("Scenario change refused. Finish/Stop the current operation, then submit a valid control file again.\n"); }).finally(() => { busy = false; });
  }, 250);
  try { await stopped; }
  finally {
    clearInterval(timer);
    release();
    await supplier.close();
    process.removeListener("SIGINT", stop);
    process.removeListener("SIGTERM", stop);
    process.stdout.write(JSON.stringify({ event: "closed", operations: supplier.events.length, rejected: supplier.events.filter(event => event.credential === "rejected").length }) + "\n");
  }
}
await main().catch(() => { process.stderr.write("Supplier could not start. Check --help and the private fixture paths.\n"); process.exitCode = 1; });

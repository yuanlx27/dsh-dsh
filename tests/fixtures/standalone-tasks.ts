import { spawn } from "node:child_process";
import { mkdir, readFile, readdir } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { taskWorkspaces } from "./task-workspaces.ts";

export type StandaloneTaskKind = "complete" | "stop";
export interface StandaloneTaskResult {
  accepted: boolean;
  runningObserved: boolean;
  settled: boolean;
  loopbackOnly: boolean;
  endExpected: boolean;
  providerAuthenticated: boolean;
  promptCount: number;
  turnStarts: number;
  turnEnds: number;
  assistantMessages: number;
  interrupted: boolean;
  prefixPreserved: boolean;
  disposed: boolean;
  workspaceUnchanged: boolean;
  exitCode: number | null;
}
const prefix = "DSH_TASK_RESULT ";

/** A child owns the real R web profile; no inherited credentials or GUI interaction. */
export async function standaloneTaskCase(kind: StandaloneTaskKind): Promise<StandaloneTaskResult> {
  const spaces = await taskWorkspaces();
  try {
    const home = join(spaces.root, "home");
    await mkdir(home, { mode: 0o700 });
    const workspace = spaces.standalone.A;
    const before = await readFile(join(workspace, "README.md"), "utf8");
    return await new Promise<StandaloneTaskResult>((resolve, reject) => {
      const child = spawn(process.execPath, [fileURLToPath(new URL("./standalone-task-driver.ts", import.meta.url)), kind], {
        cwd: workspace,
        env: { HOME: home, DSH_HOME: home, PATH: "/usr/bin:/bin", TMPDIR: tmpdir(), DSH_TELEMETRY_DISABLED: "1" },
        stdio: ["ignore", "pipe", "ignore"]
      });
      // Discard all upstream logs, including ephemeral bootstrap URLs. Retain only our closed result.
      let result: Omit<StandaloneTaskResult, "workspaceUnchanged" | "exitCode"> | undefined;
      let failure: string | undefined;
      const lines = createInterface({ input: child.stdout });
      lines.on("line", line => {
        if (/^DSH_TASK_FAILURE [a-z-]+$/.test(line)) { failure = line; return; }
        if (!line.startsWith(prefix)) return;
        try {
          const value: unknown = JSON.parse(line.slice(prefix.length));
          const bools = ["accepted", "runningObserved", "settled", "loopbackOnly", "endExpected", "providerAuthenticated", "interrupted", "prefixPreserved", "disposed"];
          const counts = ["promptCount", "turnStarts", "turnEnds", "assistantMessages"];
          if (!value || typeof value !== "object" || Object.keys(value).length !== bools.length + counts.length ||
              !bools.every(key => key in value && typeof Reflect.get(value, key) === "boolean") ||
              !counts.every(key => key in value && Number.isSafeInteger(Reflect.get(value, key)) && Reflect.get(value, key) >= 0)) {
            failure = "Invalid standalone result";
          } else if (result) failure = "Duplicate standalone result";
          else result = value as Omit<StandaloneTaskResult, "workspaceUnchanged" | "exitCode">;
        } catch { failure = "Invalid standalone result"; }
      });
      const timeout = setTimeout(() => { failure = "Standalone case exceeded its deadline"; child.kill("SIGTERM"); }, 30000);
      const force = setTimeout(() => child.kill("SIGKILL"), 38000);
      child.once("error", error => { clearTimeout(timeout); clearTimeout(force); lines.close(); reject(error); });
      child.once("close", (exitCode) => {
        clearTimeout(timeout); clearTimeout(force); lines.close();
        if (failure || !result || exitCode !== 0) { reject(Error(failure ?? `Standalone fixture failed (exit ${exitCode})`)); return; }
        const captured = result;
        void Promise.all([readdir(workspace), readFile(join(workspace, "README.md"), "utf8")]).then(([entries, after]) => {
          resolve({ ...captured, exitCode, workspaceUnchanged: entries.length === 1 && entries[0] === "README.md" && before === after });
        }, reject);
      });
    });
  } finally { await spaces.close(); }
}

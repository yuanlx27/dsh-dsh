import { spawn } from "node:child_process";
import { mkdir, readFile, readdir } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { taskWorkspaces } from "./task-workspaces.ts";

export type StandaloneTaskKind = "complete" | "stop" | "allow" | "deny" | "withdraw" | "action-stop" | "queue-steer";
export interface StandaloneTaskResult {
  accepted: boolean;
  queuePendingObserved: boolean;
  queueReceiptsDistinct: boolean;
  queueReceiptsPreserved: boolean;
  queueAdmissions: number;
  queueSteers: number;
  queueDeliveries: number;
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
  gateObserved: boolean;
  pendingNoEffect: boolean;
  decisionExpected: boolean;
  policyUnchanged: boolean;
  toolSucceeded: boolean;
  actionRunningObserved: boolean;
  actionPrefixPreserved: boolean;
  jobsQuiescent: boolean;
  approvalAsked: number;
  approvalDecided: number;
  toolCalls: number;
  toolResults: number;
  markerApproved: boolean;
  workspaceExpected: boolean;
  workspaceUnchanged: boolean;
  exitCode: number | null;
}
type ChildTaskResult = Omit<StandaloneTaskResult, "workspaceUnchanged" | "workspaceExpected" | "markerApproved" | "exitCode">;
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
      let result: ChildTaskResult | undefined;
      let failure: string | undefined;
      const lines = createInterface({ input: child.stdout });
      lines.on("line", line => {
        if (/^DSH_TASK_FAILURE [a-z-]+$/.test(line)) { failure = line; return; }
        if (!line.startsWith(prefix)) return;
        try {
          const value: unknown = JSON.parse(line.slice(prefix.length));
          const bools = ["queuePendingObserved", "queueReceiptsDistinct", "queueReceiptsPreserved", "accepted", "runningObserved", "settled", "loopbackOnly", "endExpected", "providerAuthenticated", "interrupted", "prefixPreserved", "disposed", "gateObserved", "pendingNoEffect", "decisionExpected", "policyUnchanged", "toolSucceeded", "actionRunningObserved", "actionPrefixPreserved", "jobsQuiescent"];
          const counts = ["queueAdmissions", "queueSteers", "queueDeliveries", "promptCount", "turnStarts", "turnEnds", "assistantMessages", "approvalAsked", "approvalDecided", "toolCalls", "toolResults"];
          if (!value || typeof value !== "object" || Object.keys(value).length !== bools.length + counts.length ||
              !bools.every(key => key in value && typeof Reflect.get(value, key) === "boolean") ||
              !counts.every(key => key in value && Number.isSafeInteger(Reflect.get(value, key)) && Reflect.get(value, key) >= 0)) {
            failure = "Invalid standalone result";
          } else if (result) failure = "Duplicate standalone result";
          else result = value as ChildTaskResult;
        } catch { failure = "Invalid standalone result"; }
      });
      const timeout = setTimeout(() => { failure = "Standalone case exceeded its deadline"; child.kill("SIGTERM"); }, 30000);
      const force = setTimeout(() => child.kill("SIGKILL"), 38000);
      child.once("error", error => { clearTimeout(timeout); clearTimeout(force); lines.close(); reject(error); });
      child.once("close", (exitCode) => {
        clearTimeout(timeout); clearTimeout(force); lines.close();
        if (failure || !result || exitCode !== 0) { reject(Error(failure ?? `Standalone fixture failed (exit ${exitCode})`)); return; }
        const captured = result;
        void Promise.all([readdir(workspace), readFile(join(workspace, "README.md"), "utf8")]).then(async ([entries, after]) => {
          const markerApproved = entries.includes("approval-marker.txt") && await readFile(join(workspace, "approval-marker.txt"), "utf8") === "approved\n";
          const workspaceUnchanged = entries.length === 1 && entries[0] === "README.md" && before === after;
          const actionStopped = kind === "action-stop" && entries.includes("stop-marker.txt") && await readFile(join(workspace, "stop-marker.txt"), "utf8") === "started\n";
          const workspaceExpected = before === after && (kind === "allow" ? markerApproved && entries.sort().join(",") === "README.md,approval-marker.txt" : kind === "action-stop" ? actionStopped && entries.sort().join(",") === "README.md,stop-marker.txt" : workspaceUnchanged);
          resolve({ ...captured, exitCode, markerApproved, workspaceExpected, workspaceUnchanged });
        }).catch(reject);
      });
    });
  } finally { await spaces.close(); }
}

// UI projection only. Native admission and atomic notice storage are not here.
export const NOTICE_REVISION = "experimental-2026-10-06";
const failures = {
  invalidBundle: ["The installed runtime is missing, damaged or incompatible.", "Reinstall a matching build."],
  launch: ["The local service could not start.", "Retry or reinstall a matching build."],
  timeout: ["The local service did not become available in time.", "Retry startup."],
  earlyExit: ["The local service stopped during startup.", "Retry startup."],
  connection: ["The authenticated local connection is unavailable.", "Retry the connection."],
  cleanup: ["Service cleanup could not be confirmed.", "Do not start another service; quit and check the local installation."],
  preferences: ["Application preferences could not be saved.", "Check local application-data access and try again."],
} as const;
export interface ShellSnapshot {
  phase: "starting" | "available" | "failed";
  noticeRevision: string;
  acknowledged: boolean;
  failure?: keyof typeof failures;
}
export interface ShellBridge {
  status(): Promise<unknown>;
  acknowledge(revision: string): Promise<unknown>;
  retry(): Promise<unknown>;
  enter(): Promise<void>;
}
export function parseSnapshot(input: unknown): ShellSnapshot {
  if (typeof input !== "object" || input === null || Array.isArray(input)) throw new Error("Invalid shell status.");
  const row = input as Record<string, unknown>;
  if (Object.keys(row).some(k => !["phase", "noticeRevision", "acknowledged", "failure"].includes(k)) ||
      !(typeof row.phase === "string" && ["starting", "available", "failed"].includes(row.phase)) ||
      typeof row.noticeRevision !== "string" || !/^[a-zA-Z0-9.-]{1,80}$/.test(row.noticeRevision) ||
      typeof row.acknowledged !== "boolean" ||
      row.failure !== undefined && !(typeof row.failure === "string" && Object.hasOwn(failures, row.failure))) throw new Error("Invalid shell status.");
  return row as unknown as ShellSnapshot;
}
export class ShellController {
  private snapshot: ShellSnapshot = { phase: "starting", noticeRevision: NOTICE_REVISION, acknowledged: false };
  private error = "";
  private busy = false;
  private revision = 0;
  private readonly bridge: ShellBridge;
  constructor(bridge: ShellBridge) { this.bridge = bridge; }
  get view() {
    const current = this.snapshot.noticeRevision === NOTICE_REVISION;
    const accepted = current && this.snapshot.acknowledged;
    const failure = this.snapshot.failure ? failures[this.snapshot.failure] : undefined;
    const message = this.error || (!current ? "The safety notice has changed. Reopen a matching application build." :
      this.snapshot.phase === "starting" ? "Starting the owned local service and authenticated connection…" :
      this.snapshot.phase === "available" ? "Local service available. Setup and task readiness are shown in Harness." :
      failure?.[0] ?? "The local service is unavailable.");
    return { phase: this.snapshot.phase, message, nextAction: failure?.[1] ?? "", busy: this.busy,
      canEnter: !this.busy && !this.error && accepted && this.snapshot.phase === "available",
      canRetry: !this.busy && this.snapshot.phase === "failed" && this.snapshot.failure !== "cleanup",
      canAcknowledge: current && !this.busy, needsNotice: !accepted };
  }
  async refresh() {
    if (this.busy) return;
    const revision = ++this.revision;
    try {
      const snapshot = parseSnapshot(await this.bridge.status());
      if (revision === this.revision) this.snapshot = snapshot;
    } catch {
      if (revision === this.revision) { this.error = "Desktop status is unavailable. Reopen the application."; this.snapshot.phase = "failed"; }
    }
  }
  private async update(operation: () => Promise<unknown>, failure: string) {
    if (this.busy) return;
    this.busy = true;
    ++this.revision;
    this.error = "";
    try { this.snapshot = parseSnapshot(await operation()); }
    catch { this.error = failure; }
    finally { this.busy = false; }
  }
  async acknowledge() {
    if (!this.view.canAcknowledge) return;
    await this.update(() => this.bridge.acknowledge(NOTICE_REVISION), "Could not save acknowledgement. Check local application-data access and try again.");
  }
  async retry() {
    if (!this.view.canRetry) return;
    await this.update(() => this.bridge.retry(), "Could not retry startup. Reopen the application.");
  }
  async enter() {
    if (!this.view.canEnter) return;
    this.busy = true;
    ++this.revision;
    try { await this.bridge.enter(); }
    catch { this.error = "Harness could not open. Reopen the application."; }
    finally { this.busy = false; }
  }
}

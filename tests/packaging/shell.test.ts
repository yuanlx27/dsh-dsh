import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { ShellController, NOTICE_REVISION, parseSnapshot } from "../../src/shell/state.ts";
import type { ShellSnapshot } from "../../src/shell/state.ts";
const status: ShellSnapshot = { phase: "available", noticeRevision: NOTICE_REVISION, acknowledged: false };

test("entry requires current notice and completed native save, never a local click", async () => {
  let release: (v: ShellSnapshot) => void = () => {};
  let entries = 0;
  const controller = new ShellController({ status: async () => status, acknowledge: () => new Promise(r => { release = r; }), retry: async () => status, enter: async () => { entries++; } });
  await controller.refresh();
  await controller.enter();
  assert.equal(entries, 0);
  const saving = controller.acknowledge();
  assert.equal(controller.view.canEnter, false);
  release({ ...status, acknowledged: true });
  await saving;
  assert.equal(controller.view.canEnter, true);
  await controller.enter();
  assert.equal(entries, 1);
});

test("failed writes, old revisions and raw errors cannot grant entry or leak diagnostics", async () => {
  let current = { ...status, acknowledged: true, noticeRevision: "obsolete" };
  const controller = new ShellController({ status: async () => current, acknowledge: async () => { throw new Error("cookie=private-fixture"); }, retry: async () => status, enter: async () => {} });
  await controller.refresh();
  assert.equal(controller.view.canEnter, false);
  current = { ...status };
  await controller.refresh();
  await controller.acknowledge();
  assert.equal(controller.view.canEnter, false);
  assert.ok(!JSON.stringify(controller.view).includes("private-fixture"));
  assert.ok(controller.view.message.includes("save"));
});

test("explicit retry and available are service states, not duplicated task readiness", async () => {
  let retry = 0;
  const controller = new ShellController({ status: async () => ({ ...status, phase: "failed", failure: "connection" }), acknowledge: async () => status, retry: async () => { retry++; return status; }, enter: async () => {} });
  await controller.refresh();
  assert.equal(controller.view.canRetry, true);
  assert.ok(controller.view.message.includes("connection"));
  await controller.retry();
  assert.equal(retry, 1);
  assert.equal(controller.view.phase, "available");
  assert.equal(controller.view.canEnter, false);
  assert.ok(controller.view.message.includes("Harness"));
});

test("late status cannot overwrite a newer saved notice decision", async () => {
  let resolveStatus: (v: ShellSnapshot) => void = () => {};
  const controller = new ShellController({ status: () => new Promise(r => { resolveStatus = r; }), acknowledge: async () => ({ ...status, acknowledged: true }), retry: async () => status, enter: async () => {} });
  const pending = controller.refresh();
  await controller.acknowledge();
  assert.equal(controller.view.canEnter, true);
  resolveStatus(status);
  await pending;
  assert.equal(controller.view.canEnter, true);
});

test("snapshot parser rejects unknown diagnostics and native authority claims", () => {
  assert.throws(() => parseSnapshot({ ...status, token: "fixture" }));
  assert.throws(() => parseSnapshot({ ...status, phase: "ready" }));
  assert.throws(() => parseSnapshot({ ...status, failure: "raw log fixture" }));
  assert.throws(() => parseSnapshot({ ...status, acknowledged: "true" }));
});

test("shell uses semantic modal, named controls, status announcements and visible focus", async () => {
  const root = fileURLToPath(new URL("../../src/shell/", import.meta.url));
  const html = await readFile(root + "index.html", "utf8");
  const css = await readFile(root + "styles.css", "utf8");
  const main = await readFile(root + "main.ts", "utf8");
  assert.ok(html.includes('<dialog id="notice" aria-labelledby="notice-title"'));
  assert.ok(html.includes('role="status" aria-live="polite"'));
  assert.ok(html.includes("modify files"));
  assert.ok(html.includes("isolation or undo"));
  for (const id of ["enter", "retry", "open-notice", "accept-notice", "dismiss-notice"]) assert.ok(html.includes(`id="${id}"`));
  assert.ok(css.includes(":focus-visible"));
  assert.ok(main.includes("showModal()"));
  assert.ok(main.includes(".focus()"));
  assert.ok(!main.includes("innerHTML"));
  assert.ok(!html.includes("http://") && !html.includes("https://"));
});

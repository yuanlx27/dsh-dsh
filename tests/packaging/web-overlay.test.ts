import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, rm, readFile } from "node:fs/promises";
import { join, dirname } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

const root = fileURLToPath(new URL("../../", import.meta.url));
const node = join(root, "src-tauri/binaries/node-aarch64-apple-darwin");
const cli = join(root, "src-tauri/resources/dsh/node_modules/@deepseek-ai/dsh/lib/bin.js");
const overlay = join(root, "src-tauri/resources/desktop-web.patch.yml");
const disabled = ["desktop-product-telemetry", "product-analytics", "session-telemetry-otel", "command-feedback", "message-feedback", "ui-message-feedback", "ui-plugin-manager"];
interface Row { id: string; name?: string; disabled?: unknown; group?: boolean; config?: unknown }
interface Yaml { load(source: string, options: { schema: unknown }): Row[] }
async function parser() {
  const modules = join(root, "src-tauri/resources/dsh/node_modules");
  const yaml = await import(pathToFileURL(join(modules, "js-yaml/dist/js-yaml.mjs")).href) as Yaml;
  const include = await import(pathToFileURL(join(modules, "@deepseek-ai/cordis-plugin-include/lib/index.js")).href) as { entryListSchema: unknown };
  return (source: string) => yaml.load(source, { schema: include.entryListSchema });
}

test("real R CLI overlay composes exactly excluded Web rows and preserves all other task/permission config", async (t) => {
  const home = await mkdtemp(join(tmpdir(), "web overlay home "));
  t.after(() => rm(home, { recursive: true, force: true }));
  const env: NodeJS.ProcessEnv = { HOME: home, DSH_HOME: join(home, "harness"), PATH: dirname(node), TMPDIR: tmpdir(), DSH_TELEMETRY_DISABLED: "1" };
  const run = (args: string[]) => spawnSync(node, [cli, "--profile", "web", ...args], { env, cwd: home, encoding: "utf8", timeout: 15_000, maxBuffer: 4 * 1024 * 1024 });
  const parse = await parser();
  const patch = parse(await readFile(overlay, "utf8"));
  assert.deepEqual(patch.map(row => row.id).sort(), [...disabled].sort());
  assert.ok(patch.every(row => row.disabled === true && Object.keys(row).length === 2));
  const baseline = run(["--dump-config"]);
  assert.equal(baseline.status, 0, "Pinned R baseline dump must execute.");
  const changed = run(["--patch", overlay, "--dump-config"]);
  assert.equal(changed.status, 0, "R must parse/compose the actual shipped overlay.");
  const before = parse(baseline.stdout);
  const after = parse(changed.stdout);
  assert.equal(after.length, before.length);
  for (const row of before) {
    const actual = after.find(r => r.id === row.id);
    assert.deepEqual(actual, disabled.includes(row.id) ? { ...row, disabled: true } : row, row.id);
  }
  for (const id of ["permission", "ui-approval", "session-controller", "workspace-controller", "agent-preset-registry", "modules", "connection"]) {
    assert.ok(after.some(row => row.id === id && row.disabled !== true), id);
  }
  const help = run(["--patch", overlay, "--help"]);
  assert.equal(help.status, 0, "Mount R's real patched Web help to audit plugin dependencies without binding a service.");
  assert.ok(help.stdout.includes("--host"));
  assert.ok(!help.stdout.includes("Electron"));
  assert.ok(!help.stderr.includes("StartupError"));
});

import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("bundled Node loads production native entries and performs a disposable flock", async (t) => {
  const home = await mkdtemp(join(tmpdir(), "bundled native addon probe "));
  t.after(() => rm(home, { recursive: true, force: true }));
  const script = fileURLToPath(new URL("../fixtures/native-addons.ts", import.meta.url));
  const env: NodeJS.ProcessEnv = { ...process.env, DSH_HOME: home };
  delete env.NODE_OPTIONS;
  delete env.NODE_PATH;
  const result = spawnSync(join(root, "src-tauri/binaries/node-aarch64-apple-darwin"),
    [script, join(root, "src-tauri/resources/dsh/package.json")],
    { cwd: home, env, encoding: "utf8", timeout: 10_000, maxBuffer: 65536 });
  assert.equal(result.status, 0, "Prepare the pinned runtime; bundled native addon loading/flock must succeed.");
});

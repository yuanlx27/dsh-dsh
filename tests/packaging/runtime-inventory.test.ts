import test, { type TestContext } from "node:test";
import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { inventory } from "../../scripts/runtime-files.ts";

async function fixture(t: TestContext) {
  const root = await mkdtemp(join(tmpdir(), "runtime inventory "));
  t.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(join(root, "src-tauri/resources/dsh"), { recursive: true });
  await mkdir(join(root, "src-tauri/binaries"));
  for (const name of ["node", "dsh"]) await writeFile(join(root, `src-tauri/binaries/${name}-aarch64-apple-darwin`), name);
  await writeFile(join(root, "src-tauri/resources/dsh/asset.js"), "before-signing");
  return root;
}

test("inventory records final bytes and preserves internal symlink identity", async (t) => {
  const root = await fixture(t);
  await symlink("asset.js", join(root, "src-tauri/resources/dsh/link.js"));
  const first = await inventory(root, "aarch64-apple-darwin");
  assert.equal(first.length, 4);
  assert.equal(first.find((a) => a.path.endsWith("link.js"))?.symlink, "asset.js");
  await writeFile(join(root, "src-tauri/resources/dsh/asset.js"), "after-nested-signing");
  const second = await inventory(root, "aarch64-apple-darwin");
  assert.notEqual(first.find((a) => a.path.endsWith("asset.js"))?.sha256,
    second.find((a) => a.path.endsWith("asset.js"))?.sha256);
});

test("inventory rejects symlinks escaping immutable runtime closure", async (t) => {
  const root = await fixture(t);
  await writeFile(join(root, "outside"), "outside");
  await symlink("../../../outside", join(root, "src-tauri/resources/dsh/link"));
  await assert.rejects(inventory(root, "aarch64-apple-darwin"));
});

test("prepare help is read-only and unknown flags fail promptly", () => {
  const script = fileURLToPath(new URL("../../scripts/prepare-runtime.ts", import.meta.url));
  const help = spawnSync(process.execPath, [script, "--help"], { encoding: "utf8" });
  assert.equal(help.status, 0);
  assert.match(help.stdout, /--inventory-only/);
  assert.equal(help.stderr, "");
  const invalid = spawnSync(process.execPath, [script, "--inventroy-only"], { encoding: "utf8" });
  assert.equal(invalid.status, 1);
  assert.match(invalid.stderr, /Unknown option/);
});

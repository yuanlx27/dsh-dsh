import test, { type TestContext } from "node:test";
import assert from "node:assert/strict";
import { cp, mkdir, mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));

async function fixture(t: TestContext) {
  const dir = await mkdtemp(join(tmpdir(), "desktop frontend with spaces "));
  t.after(() => rm(dir, { recursive: true, force: true }));
  await mkdir(join(dir, "scripts"));
  await cp(join(root, "scripts/build-frontend.ts"), join(dir, "scripts/build-frontend.ts"));
  await cp(join(root, "tsconfig.json"), join(dir, "tsconfig.json"));
  await symlink(join(root, "node_modules"), join(dir, "node_modules"), "dir");
  return dir;
}

function build(dir: string) {
  return spawnSync(process.execPath, [join(dir, "scripts/build-frontend.ts")], {
    cwd: tmpdir(), encoding: "utf8",
  });
}

async function source(dir: string, ts: string) {
  await mkdir(join(dir, "src/shell"), { recursive: true });
  await writeFile(join(dir, "src/shell/index.html"), "<!doctype html><script type=module src=main.js></script>");
  await writeFile(join(dir, "src/shell/styles.css"), ":focus-visible { outline: solid; }");
  await writeFile(join(dir, "src/shell/main.ts"), ts);
}

test("missing shell deliverables fail rather than create an empty successful build", async (t) => {
  const result = build(await fixture(t));
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Frontend build failed/);
});

test("static build compiles TypeScript and preserves assets independently of cwd", async (t) => {
  const dir = await fixture(t);
  await source(dir, "const value: string = \"ready\"; export { value };");
  const result = build(dir);
  assert.equal(result.status, 0, result.stderr + result.stdout);
  assert.match(await readFile(join(dir, "dist/shell/main.js"), "utf8"), /const value =/);
  assert.match(await readFile(join(dir, "dist/shell/index.html"), "utf8"), /main.js/);
  assert.match(await readFile(join(dir, "dist/shell/styles.css"), "utf8"), /focus-visible/);
  await assert.rejects(readFile(join(dir, "dist/shell/main.ts")), { code: "ENOENT" });
});

test("type errors fail the build", async (t) => {
  const dir = await fixture(t);
  await source(dir, "const value: string = 42; export { value };");
  const result = build(dir);
  assert.equal(result.status, 1);
  assert.match(result.stdout, /TS2322/);
});

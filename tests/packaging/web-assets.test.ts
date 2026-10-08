import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, symlink, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { prepareWebAssets, createRegistry } from "../../scripts/prepare-web-assets.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));

test("frozen web assets reuse R's queue, primary registrations and lazy chunks", async (t) => {
  const resources = await mkdtemp(join(tmpdir(), "frozen web assets "));
  t.after(() => rm(resources, { recursive: true, force: true }));
  await mkdir(join(resources, "dsh"));
  await symlink(join(root, "runtime/node_modules"), join(resources, "dsh/node_modules"), "dir");
  const index = await prepareWebAssets(resources);
  const registry = await createRegistry(resources);
  assert.equal(index.version, 1);
  assert.ok(Object.keys(index.packages).length > 20);
  const queue = await readFile(join(resources, "dsh/desktop-web/queue.js"), "utf8");
  assert.ok(queue.includes("window.__ModuleLoader__"));
  assert.ok(!queue.includes("127.0.0.1"));
  for (const id of ["@deepseek-ai/dsh-client-modules", "@deepseek-ai/dsh-client-connection"]) {
    const piece = index.packages[id];
    assert.ok(piece);
    const actual = await registry.bundleResource("GET", `/plugins/??${id}/client.js&rev=000000000000`);
    const prepared = await readFile(join(resources, "dsh/desktop-web", piece.main.source), "utf8");
    assert.equal(prepared + `//# sourceMappingURL=??${id}/client.js.map&rev=000000000000\n`, actual.body?.toString("utf8"));
    const map = JSON.parse(await readFile(join(resources, "dsh/desktop-web", piece.main.map), "utf8"));
    assert.equal(map.version, 3);
  }
  const terminal = index.packages["@deepseek-ai/dsh-client-ui-sidebar-terminal"];
  assert.ok(terminal.chunks["client.terminal.js"]);
  const chunk = terminal.chunks["client.terminal.js"];
  const actual = await registry.bundleResource("GET", "/plugins/@deepseek-ai/dsh-client-ui-sidebar-terminal/client.terminal.js?rev=000000000000");
  const prepared = await readFile(join(resources, "dsh/desktop-web", chunk.source), "utf8");
  assert.equal(prepared + "//# sourceMappingURL=client.terminal.js.map?rev=000000000000\n", actual.body?.toString("utf8"));
});

test("web preparation fails without the pinned local package closure", async (t) => {
  const resources = await mkdtemp(join(tmpdir(), "missing web assets "));
  t.after(() => rm(resources, { recursive: true, force: true }));
  await assert.rejects(prepareWebAssets(resources));
});

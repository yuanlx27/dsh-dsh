// Build-time only: reuse pinned R's registry composition without a Host/service.
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
// Structural views of pinned R's build-time wire metadata; no dependency on
// ignored runtime declarations is needed for CI type checking.
interface WebBootEntry { id: string; rev: string; url: string; external?: string[]; inject?: string[]; immediately?: boolean }
interface WebBootGraph { rev: string; entries: WebBootEntry[]; batches: { phase: string; url: string; rev: string; entries: string[] }[] }
interface PinnedModule {
  ClientModuleRegistry: { prototype: object };
  bootInjections(graph: WebBootGraph): { kind: string; text?: string }[];
}
interface PackageMetadata {
  name: string;
  version: string;
  exports?: Record<string, string | { default?: string }>;
  dsh?: { client?: { platform?: string; external?: string[]; inject?: string[]; immediately?: boolean } };
}
interface RecordSnapshot { entry: WebBootEntry; meta: { clientPath: string }; bundle: Buffer }
// This small adapter uses pinned private registry fields only at build time.
// No loader, settings, credentials, network, task API or second engine is started.
export interface RegistrySnapshot {
  table: Map<string, RecordSnapshot>;
  responses: Map<string, unknown>;
  previousBatchResponses: Map<string, unknown>;
  batchResponses: Map<string, unknown>;
  readSourceMap(path: string): undefined;
  compose(): WebBootGraph;
  bundleResource(method: string, url: string): Promise<{ status: number; body?: Buffer }>;
  queue: string;
}
export interface WebPiece { source: string; map: string }
export interface WebIndex {
  version: 1;
  packages: Record<string, { main: WebPiece; chunks: Record<string, WebPiece> }>;
}
const revision = "000000000000";
const chunkName = /^client\.[A-Za-z0-9][A-Za-z0-9._-]*\.js$/;

export async function createRegistry(resources: string): Promise<RegistrySnapshot> {
  const modules = join(resources, "dsh/node_modules");
  const registryPackage = join(modules, "@deepseek-ai/dsh-client-modules");
  const pkg: PackageMetadata = JSON.parse(await readFile(join(registryPackage, "package.json"), "utf8"));
  if (pkg.version !== "0.2.0-rc.2") throw new Error("Unsupported web module baseline.");
  const upstream = await import(pathToFileURL(join(registryPackage, "lib/index.js")).href) as PinnedModule;
  const registry = Object.create(upstream.ClientModuleRegistry.prototype) as RegistrySnapshot;
  registry.table = new Map();
  registry.responses = new Map();
  registry.previousBatchResponses = new Map();
  registry.batchResponses = new Map();
  // R's published client artifacts have no maps; reuse its identity-map path.
  // Fail on a changed closure rather than silently discard a new source map.
  registry.readSourceMap = () => undefined;
  for (const scope of (await readdir(modules)).sort()) {
    if (!scope.startsWith("@")) continue;
    for (const name of (await readdir(join(modules, scope))).sort()) {
      const directory = join(modules, scope, name);
      const metadata: PackageMetadata = JSON.parse(await readFile(join(directory, "package.json"), "utf8"));
      if (metadata.dsh?.client?.platform !== "web") continue;
      const exported = metadata.exports?.["./client"];
      const client = typeof exported === "string" ? exported : exported?.default;
      if (!client?.startsWith("./") || client.split("/").slice(1).some(p => p === "..")) throw new Error("Missing immutable client export.");
      const clientPath = join(directory, client);
      const files = await readdir(dirname(clientPath));
      if (files.some(f => /^client.*\.js\.map$/.test(f))) throw new Error("Pinned source-map closure changed; review composition.");
      registry.table.set(metadata.name, {
        entry: { id: metadata.name, rev: revision, url: `plugins/??${metadata.name}/client.js&rev=${revision}`, ...metadata.dsh.client },
        meta: { clientPath }, bundle: await readFile(clientPath),
      });
    }
  }
  const graph = registry.compose();
  const rows = upstream.bootInjections(graph);
  const queue = rows.find(row => row.kind === "script");
  if (!queue || queue.kind !== "script" || typeof queue.text !== "string") throw new Error("Missing pinned module queue.");
  registry.queue = queue.text;
  return registry;
}

export async function prepareWebAssets(resources: string): Promise<WebIndex> {
  const registry = await createRegistry(resources);
  const output = join(resources, "dsh/desktop-web");
  await rm(output, { recursive: true, force: true });
  await mkdir(output, { recursive: true });
  await writeFile(join(output, "queue.js"), registry.queue);
  const index: WebIndex = { version: 1, packages: {} };
  for (const [id, record] of registry.table) {
    const key = createHash("sha256").update(id).digest("hex");
    await mkdir(join(output, key));
    async function freeze(file: string, url: string): Promise<WebPiece> {
      const script = await registry.bundleResource("GET", url);
      const mapUrl = url.replace(`${file}&`, `${file}.map&`).replace(`${file}?`, `${file}.map?`);
      const map = await registry.bundleResource("GET", mapUrl);
      if (script.status !== 200 || !script.body || map.status !== 200 || !map.body) throw new Error("Pinned module composition failed.");
      const text = script.body.toString("utf8");
      const trailer = text.lastIndexOf("//# sourceMappingURL=");
      if (trailer < 0 || !text.endsWith("\n")) throw new Error("Pinned debug trailer changed.");
      const source = `${key}/${file}.part`;
      const mapPath = `${key}/${file}.section.json`;
      await writeFile(join(output, source), text.slice(0, trailer));
      const parsed: { sections: { map: unknown }[] } = JSON.parse(map.body.toString("utf8"));
      if (parsed.sections.length !== 1) throw new Error("Unexpected singleton source map.");
      await writeFile(join(output, mapPath), JSON.stringify(parsed.sections[0].map));
      return { source, map: mapPath };
    }
    const main = await freeze("client.js", `/plugins/??${id}/client.js&rev=${revision}`);
    const chunks: Record<string, WebPiece> = {};
    for (const file of (await readdir(dirname(record.meta.clientPath))).sort()) {
      if (chunkName.test(file)) chunks[file] = await freeze(file, `/plugins/${id}/${file}?rev=${revision}`);
    }
    index.packages[id] = { main, chunks };
  }
  await writeFile(join(output, "index.json"), JSON.stringify(index, null, 2) + "\n");
  return index;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { await prepareWebAssets(fileURLToPath(new URL("../src-tauri/resources/", import.meta.url))); }
  catch { console.error("Web asset preparation failed; verify the pinned package closure."); process.exitCode = 1; }
}

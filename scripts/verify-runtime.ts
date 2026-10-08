import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { inventory, sha256, type RuntimeArtifact } from "./runtime-files.ts";
import type runtimeLock from "../runtime.lock.json";
import type desktopConfig from "../src-tauri/tauri.conf.json";

export type RuntimeManifest = Pick<typeof runtimeLock,
  "desktopVersion" | "dshVersion" | "upstreamRevision" | "nodeVersion" | "target" |
  "dependencyLockHash" | "nativeBuildNumber"> & { artifacts: RuntimeArtifact[] };
interface DependencyResolution {
  packages: Record<string, { version?: string; dev?: boolean; optional?: boolean; os?: string[]; cpu?: string[] }>;
}

const root = fileURLToPath(new URL("../", import.meta.url));
const json = async <T>(root: string, path: string): Promise<T> => JSON.parse(await readFile(join(root, path), "utf8"));

export async function verifyRuntime(projectRoot: string): Promise<RuntimeManifest> {
  const lock = await json<typeof runtimeLock>(projectRoot, "runtime.lock.json");
  const manifest = await json<RuntimeManifest>(projectRoot, "src-tauri/resources/runtime-manifest.json");
  const pkg = await json<{ version: string }>(projectRoot, "package.json");
  const config = await json<typeof desktopConfig>(projectRoot, "src-tauri/tauri.conf.json");
  const dsh = await json<{ version: string }>(projectRoot, "src-tauri/resources/dsh/node_modules/@deepseek-ai/dsh/package.json");
  assert.equal(lock.desktopVersion, "0.2.0-rc.2", "Unsupported canonical runtime baseline.");
  assert.equal(lock.dshVersion, lock.desktopVersion, "Runtime lock release mismatch.");
  assert.equal(lock.target, "aarch64-apple-darwin", "Unsupported runtime target.");
  assert.equal(lock.nodeVersion, "24.21.0", "Unsupported Node baseline.");
  assert.equal(lock.upstreamRevision, "639ed015397290b3745d163aafe02ffee4aa3f84", "Unsupported source baseline.");
  assert.equal(lock.node.sha256, "bed7eea5325e1108f32ce5228ddd6a5f0f08a499ee42aa7442aea583702f6057", "Node download identity changed.");
  assert.equal(lock.dsh.integrity, "sha512-EAJ3gPNcVt/uv8X19PMm9NkVhWgT7xXNMk0UKCVm+IQ5rpSQOcsMUa0HWlnYYVybKMsccjcRB21vVVsaXQ6IdA==", "dsh download identity changed.");
  for (const version of [pkg.version, config.version, dsh.version, manifest.desktopVersion, manifest.dshVersion]) {
    assert.equal(version, lock.desktopVersion, "Canonical desktop/dsh prerelease mismatch.");
  }
  for (const key of ["upstreamRevision", "nodeVersion", "target", "dependencyLockHash", "nativeBuildNumber"] as const) {
    assert.equal(manifest[key], lock[key], `Runtime manifest ${key} mismatch.`);
  }
  assert.ok(Number.isSafeInteger(lock.nativeBuildNumber) && lock.nativeBuildNumber > 0, "Invalid native build number.");
  assert.equal(config.bundle.macOS.bundleVersion, String(lock.nativeBuildNumber), "Apple build metadata mismatch.");
  assert.equal(config.bundle.macOS.minimumSystemVersion, "14.0", "Minimum macOS mismatch.");
  assert.equal(await sha256(join(projectRoot, "runtime/package-lock.json")), lock.dependencyLockHash, "Frozen dependency resolution changed.");
  assert.ok(Array.isArray(manifest.artifacts) && manifest.artifacts.length > 0, "Missing artifact inventory.");
  const seen = new Set();
  for (const artifact of manifest.artifacts) {
    assert.ok(typeof artifact.path === "string" &&
      !artifact.path.includes("\\") && !artifact.path.split("/").some((p) => ["", ".", ".."].includes(p)) &&
      (artifact.path.startsWith("src-tauri/resources/") || artifact.path === `src-tauri/binaries/node-${lock.target}` ||
       artifact.path === `src-tauri/binaries/dsh-${lock.target}`), "Unsafe runtime inventory path.");
    assert.ok(!seen.has(artifact.path), "Duplicate runtime inventory path.");
    seen.add(artifact.path);
    assert.match(artifact.sha256, /^[a-f0-9]{64}$/, "Invalid artifact hash.");
  }
  const actual = await inventory(projectRoot, lock.target);
  assert.deepEqual([...manifest.artifacts].sort((a, b) => a.path.localeCompare(b.path, "en")), actual,
    "Runtime artifact inventory is missing, corrupt or incomplete.");
  assert.equal(await sha256(join(projectRoot, "src-tauri/resources/dsh/package-lock.json")), lock.dependencyLockHash,
    "Packaged dependency resolution changed.");
  const dependencies = await json<DependencyResolution>(projectRoot, "runtime/package-lock.json");
  const supports = (list: string[] | undefined, target: string) => !list || (!list.includes(`!${target}`) &&
    (list.every((item) => item.startsWith("!")) || list.includes(target)));
  for (const [path, expected] of Object.entries(dependencies.packages)) {
    if (!path || expected.dev || !supports(expected.os, "darwin") || !supports(expected.cpu, "arm64")) continue;
    try {
      const installed = await json<{ version: string }>(projectRoot, `src-tauri/resources/dsh/${path}/package.json`);
      assert.equal(installed.version, expected.version, "Installed dependency version mismatch.");
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "ENOENT" && expected.optional) continue;
      throw error;
    }
  }
  assert.ok(actual.some((a) => /\.(html|js)$/.test(a.path)), "Missing frontend dependency closure.");
  assert.ok(actual.some((a) => a.path.endsWith(".node")), "Missing native dependency closure.");
  const home = await mkdtemp(join(tmpdir(), "dsh-version-probe-"));
  try {
    const node = join(projectRoot, `src-tauri/binaries/node-${lock.target}`);
    const env: NodeJS.ProcessEnv = { ...process.env, DSH_HOME: home };
    delete env.NODE_OPTIONS;
    delete env.NODE_PATH;
    function probe(args: string[], expected: string) {
      const result = spawnSync(node, args, { cwd: home, env, encoding: "utf8", timeout: 5000, maxBuffer: 65536 });
      assert.equal(result.status, 0, "Bundled runtime probe failed.");
      assert.equal(result.stdout.trim(), expected, "Bundled runtime probe identity mismatch.");
    }
    probe(["--version"], `v${lock.nodeVersion}`);
    probe(["-p", "process.arch"], "arm64");
    probe([join(projectRoot, "src-tauri/resources/dsh/node_modules/@deepseek-ai/dsh/lib/bin.js"), "--version"], lock.dshVersion);
  } finally { await rm(home, { recursive: true, force: true }); }
  return manifest;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const { values } = parseArgs({ options: { help: { type: "boolean", short: "h" } } });
    if (values.help) {
      process.stdout.write("Verify the staged immutable runtime without downloads.\nUsage: npm run runtime:verify -- [--help]\nExample: npm run runtime:prepare && npm run runtime:verify\nSee specs/001-deepseek-harness-desktop/quickstart.md.\n");
    } else {
      const manifest = await verifyRuntime(root);
      process.stdout.write(`Runtime verified: ${manifest.dshVersion}, Node ${manifest.nodeVersion}, ${manifest.target}.\n`);
    }
  } catch {
    console.error("Runtime verification failed: missing, corrupt or incompatible resources. Run npm run runtime:prepare; review baseline changes before rebuilding.");
    process.exitCode = 1;
  }
}

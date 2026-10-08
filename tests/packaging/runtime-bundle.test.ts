import test, { type TestContext } from "node:test";
import assert from "node:assert/strict";
import { createHash, type BinaryLike } from "node:crypto";
import { chmod, copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { verifyRuntime, type RuntimeManifest } from "../../scripts/verify-runtime.ts";

const hash = (bytes: BinaryLike) => createHash("sha256").update(bytes).digest("hex");
const version = "0.2.0-rc.2";
const target = "aarch64-apple-darwin";
const revision = "639ed015397290b3745d163aafe02ffee4aa3f84";

async function json(path: string, value: unknown) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, JSON.stringify(value));
}

async function fixture(t: TestContext) {
  const root = await mkdtemp(join(tmpdir(), "desktop runtime path with spaces "));
  t.after(() => rm(root, { recursive: true, force: true }));
  const nodePath = `src-tauri/binaries/node-${target}`;
  const launcherPath = `src-tauri/binaries/dsh-${target}`;
  const packagePath = "src-tauri/resources/dsh/node_modules/@deepseek-ai/dsh/package.json";
  const cliPath = "src-tauri/resources/dsh/node_modules/@deepseek-ai/dsh/lib/bin.js";
  const assetPath = "src-tauri/resources/dsh/node_modules/@fixture/frontend/dist/index.html";
  const addonPath = "src-tauri/resources/dsh/node_modules/@fixture/native/addon.node";
  const lockPath = "runtime/package-lock.json";
  await json(join(root, "package.json"), { version });
  await json(join(root, "src-tauri/tauri.conf.json"), {
    version, bundle: { macOS: { bundleVersion: "1", minimumSystemVersion: "14.0" } },
  });
  await json(join(root, lockPath), {
    lockfileVersion: 3,
    packages: {
      "": { dependencies: { "@deepseek-ai/dsh": version } },
      "node_modules/@deepseek-ai/dsh": { version },
      "node_modules/@fixture/frontend": { version: "1.0.0" },
      "node_modules/@fixture/native": { version: "1.0.0" },
    },
  });
  await json(join(root, "runtime.lock.json"), {
    desktopVersion: version, dshVersion: version, upstreamRevision: revision,
    nodeVersion: "24.21.0", target, nativeBuildNumber: 1,
    dependencyLockHash: hash(await readFile(join(root, lockPath))),
    node: {
      url: "https://nodejs.org/dist/v24.21.0/node-v24.21.0-darwin-arm64.tar.gz",
      sha256: "bed7eea5325e1108f32ce5228ddd6a5f0f08a499ee42aa7442aea583702f6057",
    },
    dsh: {
      integrity: "sha512-EAJ3gPNcVt/uv8X19PMm9NkVhWgT7xXNMk0UKCVm+IQ5rpSQOcsMUa0HWlnYYVybKMsccjcRB21vVVsaXQ6IdA==",
    },
  });
  await json(join(root, packagePath), { version, bin: { dsh: "lib/bin.js" } });
  await copyFile(join(root, lockPath), join(root, "src-tauri/resources/dsh/package-lock.json"));
  for (const name of ["frontend", "native"]) {
    await json(join(root, `src-tauri/resources/dsh/node_modules/@fixture/${name}/package.json`), { version: "1.0.0" });
  }
  // These disposable probes simulate version/architecture results, not real Node/native execution.
  for (const [path, bytes] of [
    [nodePath, '#!/bin/sh\ncase "$1" in\n--version) echo v24.21.0;;\n-p) echo arm64;;\n*) echo 0.2.0-rc.2;;\nesac\n'],
    [launcherPath, "#!/bin/sh\nexit 0\n"],
    [cliPath, "// Disposable probe fixture.\n"],
    [assetPath, "<!doctype html><title>Fixture</title>"],
    [addonPath, "fixture-native-bytes"],
  ]) {
    await mkdir(dirname(join(root, path)), { recursive: true });
    await writeFile(join(root, path), bytes);
  }
  await chmod(join(root, nodePath), 0o700);
  await chmod(join(root, launcherPath), 0o700);
  const paths = [nodePath, launcherPath, packagePath, cliPath, assetPath, addonPath,
    "src-tauri/resources/dsh/package-lock.json",
    ...["frontend", "native"].map((name) => `src-tauri/resources/dsh/node_modules/@fixture/${name}/package.json`)];
  const artifacts = await Promise.all(paths.map(async (path) => ({
    path, sha256: hash(await readFile(join(root, path))),
  })));
  await json(join(root, "src-tauri/resources/runtime-manifest.json"), {
    desktopVersion: version, dshVersion: version, upstreamRevision: revision,
    nodeVersion: "24.21.0", target, nativeBuildNumber: 1,
    dependencyLockHash: hash(await readFile(join(root, lockPath))), artifacts,
  });
  return { root, nodePath, packagePath, assetPath, addonPath, lockPath };
}

async function mutate(root: string, file: string, change: (value: RuntimeManifest) => void) {
  const path = join(root, file);
  const value: RuntimeManifest = JSON.parse(await readFile(path, "utf8"));
  change(value);
  await json(path, value);
}

const manifestPath = "src-tauri/resources/runtime-manifest.json";

test("complete canonical bundle verifies even when its path contains spaces", async (t) => {
  const { root } = await fixture(t);
  const manifest = await verifyRuntime(root);
  assert.equal(manifest.desktopVersion, version);
});

for (const file of ["package.json", "src-tauri/tauri.conf.json", manifestPath,
  "src-tauri/resources/dsh/node_modules/@deepseek-ai/dsh/package.json", "runtime.lock.json"]) {
  test(`rejects a different prerelease in ${file}`, async (t) => {
    const { root } = await fixture(t);
    await mutate(root, file, (value) => {
      if ("version" in value) value.version = "0.2.0-rc.1";
      else value.dshVersion = "0.2.0-rc.1";
    });
    await assert.rejects(verifyRuntime(root));
  });
}

for (const field of ["target", "upstreamRevision", "nodeVersion", "dependencyLockHash", "nativeBuildNumber"] as const) {
  test(`rejects inconsistent manifest ${field}`, async (t) => {
    const { root } = await fixture(t);
    await mutate(root, manifestPath, (value) => { if (field === "nativeBuildNumber") value[field] = 2; else value[field] = "invalid"; });
    await assert.rejects(verifyRuntime(root));
  });
}

test("rejects changed frozen dependency resolution", async (t) => {
  const { root, lockPath } = await fixture(t);
  await writeFile(join(root, lockPath), "{}");
  await assert.rejects(verifyRuntime(root));
});

for (const artifact of ["nodePath", "packagePath", "assetPath", "addonPath"] as const) {
  for (const mode of ["missing", "corrupt", "uninventoried"]) {
    test(`rejects ${mode} ${artifact}`, async (t) => {
      const f = await fixture(t);
      if (mode === "missing") await rm(join(f.root, f[artifact]));
      else if (mode === "corrupt") await writeFile(join(f.root, f[artifact]), "corrupt");
      else await mutate(f.root, manifestPath, (m) => { m.artifacts = m.artifacts.filter((a) => a.path !== f[artifact]); });
      await assert.rejects(verifyRuntime(f.root));
    });
  }
}

for (const [label, from, to] of [
  ["Node version", "echo v24.21.0", "echo v24.20.0"],
  ["Node architecture", "echo arm64", "echo x64"],
  ["CLI version", "echo 0.2.0-rc.2", "echo 0.2.0-rc.1"],
]) {
  test(`rejects ${label} even if artifact hashes are regenerated`, async (t) => {
    const { root, nodePath } = await fixture(t);
    const bytes = (await readFile(join(root, nodePath), "utf8")).replace(from, to);
    await writeFile(join(root, nodePath), bytes);
    await mutate(root, manifestPath, (m) => { const artifact = m.artifacts.find((a) => a.path === nodePath); assert.ok(artifact); artifact.sha256 = hash(bytes); });
    await assert.rejects(verifyRuntime(root));
  });
}

test("rejects inventory traversal and duplicate entries", async (t) => {
  const { root } = await fixture(t);
  await mutate(root, manifestPath, (m) => { m.artifacts.push({ path: "../outside", sha256: "0".repeat(64) }); });
  await assert.rejects(verifyRuntime(root));
  await mutate(root, manifestPath, (m) => { m.artifacts.pop(); m.artifacts.push(m.artifacts[0]); });
  await assert.rejects(verifyRuntime(root));
});

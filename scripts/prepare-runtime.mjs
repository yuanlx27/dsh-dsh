import { createHash } from "node:crypto";
import { createWriteStream } from "node:fs";
import { chmod, copyFile, cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { inventory, sha256 } from "./runtime-files.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const help = `Prepare the locked macOS arm64 runtime at build time.
Usage: npm run runtime:prepare -- [--inventory-only] [--help]
  --inventory-only  Regenerate final hashes after nested signing; no downloads.
  -h, --help        Show this help.
Examples:
  npm run runtime:prepare
  npm run runtime:prepare -- --inventory-only
See specs/001-deepseek-harness-desktop/quickstart.md for signing and acceptance.
`;

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit", ...options });
  if (result.error || result.status !== 0) throw new Error(`Build-time command failed: ${command}`);
}

async function download(url, path, algorithm, expected, encoding) {
  const response = await fetch(url, { redirect: "error", signal: AbortSignal.timeout(120_000) });
  if (!response.ok || !response.body) throw new Error("Pinned runtime download failed; retry preparation with registry access.");
  await pipeline(response.body, createWriteStream(path, { mode: 0o600 }));
  const digest = createHash(algorithm).update(await readFile(path)).digest(encoding);
  if (digest !== expected) {
    await rm(path, { force: true });
    throw new Error("Pinned runtime download integrity failed; do not use this artifact.");
  }
}

try {
  const { values } = parseArgs({ options: {
    help: { type: "boolean", short: "h" },
    "inventory-only": { type: "boolean" },
  } });
  if (values.help) {
    process.stdout.write(help);
  } else {
    const lock = JSON.parse(await readFile(join(root, "runtime.lock.json"), "utf8"));
    if (lock.target !== "aarch64-apple-darwin" || process.platform !== "darwin" || process.arch !== "arm64") {
      throw new Error("Runtime preparation requires macOS Apple Silicon.");
    }
    if (await sha256(join(root, "runtime/package-lock.json")) !== lock.dependencyLockHash) {
      throw new Error("Frozen runtime lock changed; review the baseline before preparing.");
    }
    if (!values["inventory-only"]) {
      console.error("Preparing verified Node and frozen production packages (build-time networking only).");
      const cache = join(root, ".cache/runtime");
      await mkdir(cache, { recursive: true });
      const nodeArchive = join(cache, "node.tar.gz");
      const dshArchive = join(cache, "dsh.tgz");
      await download(lock.node.url, nodeArchive, "sha256", lock.node.sha256, "hex");
      await download(lock.dsh.url, dshArchive, "sha512", lock.dsh.integrity.replace(/^sha512-/, ""), "base64");
      const nodeRoot = join(cache, `node-v${lock.nodeVersion}-darwin-arm64`);
      await rm(nodeRoot, { recursive: true, force: true });
      run("/usr/bin/tar", ["-xzf", nodeArchive, "-C", cache]);
      const node = join(nodeRoot, "bin/node");
      const env = { ...process.env, PATH: `${join(nodeRoot, "bin")}:${process.env.PATH || ""}` };
      delete env.NODE_OPTIONS;
      delete env.NODE_PATH;
      const probe = spawnSync(node, ["--version"], { encoding: "utf8", env });
      if (probe.status !== 0 || probe.stdout.trim() !== `v${lock.nodeVersion}`) throw new Error("Downloaded Node version is incompatible.");
      run(node, [join(nodeRoot, "lib/node_modules/npm/bin/npm-cli.js"), "ci", "--prefix", join(root, "runtime"),
        "--omit=dev", "--no-audit", "--no-fund"], { env });
      if (await sha256(join(root, "runtime/package-lock.json")) !== lock.dependencyLockHash) throw new Error("Production install changed the frozen lock.");
      // The launcher is private native process ownership, not another Harness engine.
      run("cargo", ["build", "--locked", "--release", "--manifest-path", "src-tauri/Cargo.toml", "--bin", "dsh-launcher"]);
      const binaries = join(root, "src-tauri/binaries");
      const resources = join(root, "src-tauri/resources");
      await mkdir(binaries, { recursive: true });
      await mkdir(resources, { recursive: true });
      await copyFile(node, join(binaries, `node-${lock.target}`));
      await copyFile(join(root, `src-tauri/target/${lock.target}/release/dsh-launcher`), join(binaries, `dsh-${lock.target}`));
      for (const name of ["node", "dsh"]) await chmod(join(binaries, `${name}-${lock.target}`), 0o755);
      await rm(join(resources, "dsh"), { recursive: true, force: true });
      await mkdir(join(resources, "dsh"));
      // Preserve dynamic packages, web assets and native addons without JS rebundling.
      await cp(join(root, "runtime/node_modules"), join(resources, "dsh/node_modules"), { recursive: true, verbatimSymlinks: true });
      for (const name of ["package.json", "package-lock.json"]) await copyFile(join(root, "runtime", name), join(resources, "dsh", name));
      await copyFile(join(root, "runtime.lock.json"), join(resources, "runtime.lock.json"));
    }
    const { desktopVersion, dshVersion, upstreamRevision, nodeVersion, target, dependencyLockHash, nativeBuildNumber } = lock;
    const manifest = { desktopVersion, dshVersion, upstreamRevision, nodeVersion, target, dependencyLockHash, nativeBuildNumber,
      artifacts: await inventory(root, target) };
    await writeFile(join(root, "src-tauri/resources/runtime-manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
    console.error(`Runtime inventory generated: ${manifest.artifacts.length} artifacts. Run npm run runtime:verify next.`);
  }
} catch (error) {
  console.error(`Runtime preparation failed: ${error.message}`);
  process.exitCode = 1;
}

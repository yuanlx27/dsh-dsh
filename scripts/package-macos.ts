import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { cp, lstat, mkdir, mkdtemp, open, readFile, realpath, rm, symlink, unlink, writeFile } from "node:fs/promises";
import { dirname, isAbsolute, join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import lock from "../runtime.lock.json" with { type: "json" };
import config from "../src-tauri/tauri.conf.json" with { type: "json" };
import { inventory, sha256 } from "./runtime-files.ts";
import { verifyRuntime, type RuntimeManifest } from "./verify-runtime.ts";

export type NativeRun = (command: string, args: readonly string[]) => string;
const project = fileURLToPath(new URL("../", import.meta.url));
const entitlements = join(project, "src-tauri/entitlements.plist");
const native: NativeRun = (command, args) => {
  const env: NodeJS.ProcessEnv = { ...process.env, PATH: `${dirname(process.execPath)}:${process.env.PATH ?? "/usr/bin:/bin"}` };
  delete env.NODE_OPTIONS; delete env.NODE_PATH;
  return execFileSync(command, args, { cwd: project, env, encoding: "utf8", timeout: 600000, maxBuffer: 1024 * 1024 });
};

/** Native .app layout adapter only; T009 remains the single inventory algorithm. */
export function installedPath(app: string, target: string, path: string): string {
  if (path.includes("\\") || path.split("/").some(part => ["", ".", ".."].includes(part))) throw Error("Unsafe inventory path");
  if (path.startsWith("src-tauri/resources/")) return join(app, "Contents/Resources", path.slice("src-tauri/resources/".length));
  for (const name of ["node", "dsh-launcher"]) if (path === `src-tauri/binaries/${name}-${target}`) return join(app, "Contents/MacOS", name);
  throw Error("Invalid installed inventory path");
}

function header(manifest: RuntimeManifest) {
  for (const key of ["desktopVersion", "dshVersion", "upstreamRevision", "nodeVersion", "target", "dependencyLockHash", "nativeBuildNumber"] as const) assert.equal(manifest[key], lock[key], `Installed canonical ${key} mismatch`);
  assert.ok(Array.isArray(manifest.artifacts) && manifest.artifacts.length > 0);
  assert.equal(new Set(manifest.artifacts.map(a => a.path)).size, manifest.artifacts.length);
  for (const a of manifest.artifacts) assert.match(a.sha256, /^[a-f0-9]{64}$/);
}

function appleMetadata(app: string, manifest: RuntimeManifest, run: NativeRun) {
  const plist = join(app, "Contents/Info.plist");
  const read = (key: string) => run("/usr/libexec/PlistBuddy", ["-c", `Print :${key}`, plist]).trim();
  assert.equal(read("CFBundleIdentifier"), config.identifier);
  assert.equal(read("CFBundleShortVersionString"), manifest.dshVersion.split("-")[0]);
  assert.equal(read("CFBundleVersion"), String(manifest.nativeBuildNumber));
  assert.equal(read("LSMinimumSystemVersion"), config.bundle.macOS.minimumSystemVersion);
  assert.equal(read("CFBundleExecutable"), "deepseek-harness-desktop");
}

async function macho(path: string) {
  const file = await open(path, "r"); const magic = Buffer.alloc(4);
  try { await file.read(magic, 0, 4, 0); } finally { await file.close(); }
  return ["cffaedfe", "cefaedfe", "feedfacf", "feedface", "cafebabe", "bebafeca", "cafebabf", "bfbafeca"].includes(magic.toString("hex"));
}

/** Finalize a copied owned build. No writes after outer signing, even to manifest. */
export async function finalizeApp(input: string, run: NativeRun = native, tauriResignedSidecars = false) {
  const app = await realpath(input);
  const manifestPath = join(app, "Contents/Resources/runtime-manifest.json");
  const manifest: RuntimeManifest = JSON.parse(await readFile(manifestPath, "utf8"));
  header(manifest); appleMetadata(app, manifest, run);
  const pathOf = (path: string) => installedPath(app, manifest.target, path);
  for (const name of ["node", "dsh-launcher"]) {
    const path = join(app, "Contents/MacOS", name);
    assert.ok((await lstat(path)).isFile(), "Sidecar must be a regular file");
    assert.equal(run("lipo", ["-archs", path]).trim(), "arm64", "Wrong sidecar architecture");
  }
  // Restore Tauri-flattened links from the frozen canonical link identities.
  for (const artifact of manifest.artifacts) {
    const path = pathOf(artifact.path);
    assert.ok((await realpath(path)).startsWith(app + "/"), "Artifact escaped app");
    if (artifact.symlink !== undefined && !(await lstat(path)).isSymbolicLink()) {
      await unlink(path); await symlink(artifact.symlink, path);
    }
  }
  const copied = await inventory(app, manifest.target, pathOf);
  const expected = manifest.artifacts.map(a => ({ ...a })).sort((a, b) => a.path.localeCompare(b.path, "en"));
  // Only the known Tauri build producer may replace the two sidecar signatures.
  // Data/native-addon closure hashes and complete inventory membership stay strict.
  if (tauriResignedSidecars) for (const name of ["node", "dsh-launcher"]) {
    const path = `src-tauri/binaries/${name}-${manifest.target}`;
    const old = expected.find(a => a.path === path), signed = copied.find(a => a.path === path);
    assert.ok(old && signed && old.symlink === undefined && signed.symlink === undefined);
    run("codesign", ["--verify", "--strict", pathOf(path)]);
    old.sha256 = signed.sha256;
  }
  assert.deepEqual(copied, expected, "Pre-sign copied closure mismatch");
  const nested: string[] = [];
  for (const artifact of manifest.artifacts) if (artifact.symlink === undefined) {
    const path = pathOf(artifact.path);
    if (await macho(path)) { run("codesign", ["--force", "--sign", "-", path]); nested.push(path); }
  }
  // Inventory reflects signed bytes; download/lock identities are unchanged.
  manifest.artifacts = await inventory(app, manifest.target, pathOf);
  await writeFile(manifestPath, JSON.stringify(manifest, null, 2) + "\n");
  run("codesign", ["--force", "--sign", "-", "--entitlements", entitlements, app]);
  run("codesign", ["--verify", "--deep", "--strict", app]);
  for (const path of nested) run("codesign", ["--verify", "--strict", path]);
  appleMetadata(app, manifest, run);
  assert.deepEqual(await inventory(app, manifest.target, pathOf), manifest.artifacts, "Post-sign resource mutation");
  assert.deepEqual(JSON.parse(await readFile(manifestPath, "utf8")), manifest, "Post-sign manifest mutation");
  return { manifest, signedNested: nested.length };
}

/** Build-time proof only: execute installed Node/SDK, never the GUI or a Host. */
export async function probeInstalledRuntime(app: string) {
  const home = await mkdtemp(join(tmpdir(), "dsh installed addon probe "));
  const node = join(app, "Contents/MacOS/node");
  const resources = join(app, "Contents/Resources");
  const env: NodeJS.ProcessEnv = { HOME: home, DSH_HOME: home, TMPDIR: tmpdir(), PATH: "/usr/bin:/bin", DO_NOT_TRACK: "1" };
  const run = (args: string[]) => execFileSync(node, args, { cwd: home, env, encoding: "utf8", timeout: 15000, maxBuffer: 65536 }).trim();
  try {
    assert.equal(run(["--version"]), `v${lock.nodeVersion}`);
    assert.equal(run([join(resources, "dsh/node_modules/@deepseek-ai/dsh/lib/bin.js"), "--version"]), lock.dshVersion);
    run([join(project, "tests/fixtures/native-addons.ts"), join(resources, "dsh/package.json")]);
    const manifest: RuntimeManifest = JSON.parse(await readFile(join(resources, "runtime-manifest.json"), "utf8"));
    header(manifest);
    assert.deepEqual(await inventory(app, manifest.target, p => installedPath(app, manifest.target, p)), manifest.artifacts, "Installed probes mutated sealed resources");
  } finally { await rm(home, { recursive: true, force: true }); }
}

/** Fresh output only. Always stage/verify/build current source; no arbitrary app input. */
export async function packageMacos(output: string) {
  if (process.platform !== "darwin" || process.arch !== "arm64" || process.version !== "v24.21.0") throw Error("Use macOS arm64 with Node 24.21.0");
  if (!isAbsolute(output)) throw Error("Provide an absolute NEW output directory");
  // Refuse existing paths; preserve any incomplete output/mount on native
  // cleanup failure rather than deleting artifacts with unconfirmed ownership.
  await mkdir(output, { mode: 0o700 });
    native("npm", ["run", "runtime:prepare"]);
    await verifyRuntime(project);
    native("npm", ["run", "tauri:build", "--", "--bundles", "app"]);
    const source = join(project, "src-tauri/target", lock.target, "release/bundle/macos", `${config.productName}.app`);
    const app = join(output, `${config.productName}.app`);
    await cp(source, app, { recursive: true, force: false, errorOnExist: true, verbatimSymlinks: true });
    const result = await finalizeApp(app, native, true);
    // Native verifier uses the production installed BundlePaths/manifest checks.
    native("cargo", ["run", "--manifest-path", "src-tauri/Cargo.toml", "--example", "verify-installed", "--", app]);
    await probeInstalledRuntime(app);
    const imageRoot = await mkdtemp(join(tmpdir(), "dsh image staging "));
    const mount = await mkdtemp(join(tmpdir(), "dsh image verify "));
    const dmg = join(output, `${config.productName}-${lock.desktopVersion}-arm64.dmg`);
    let attached = false;
    try {
      await cp(app, join(imageRoot, `${config.productName}.app`), { recursive: true, verbatimSymlinks: true });
      await symlink("/Applications", join(imageRoot, "Applications"));
      native("hdiutil", ["create", "-volname", config.productName, "-srcfolder", imageRoot, "-format", "UDZO", "-ov", dmg]);
      native("hdiutil", ["verify", dmg]);
      native("hdiutil", ["attach", "-readonly", "-nobrowse", "-mountpoint", mount, dmg]); attached = true;
      const installed = join(mount, `${config.productName}.app`);
      native("codesign", ["--verify", "--deep", "--strict", installed]);
      native("cargo", ["run", "--manifest-path", "src-tauri/Cargo.toml", "--example", "verify-installed", "--", installed]);
      await probeInstalledRuntime(installed);
    } finally {
      if (attached) native("hdiutil", ["detach", mount]);
      await rm(imageRoot, { recursive: true, force: true }); await rm(mount, { recursive: true, force: true });
    }
    const metadata = { kind: "local-ad-hoc-not-public-qualified", version: lock.desktopVersion, target: lock.target, nativeBuildNumber: lock.nativeBuildNumber, signedNested: result.signedNested, artifacts: result.manifest.artifacts.length, app, dmg, dmgSha256: await sha256(dmg) };
    await writeFile(join(output, "distribution.json"), JSON.stringify(metadata, null, 2) + "\n", { mode: 0o600 });
    return metadata;
}

if (import.meta.main) {
  try {
    const { values } = parseArgs({ options: { output: { type: "string" }, help: { type: "boolean", short: "h" } } });
    if (values.help) process.stdout.write("Build/finalize local ad-hoc arm64 app + verified read-only DMG. No Developer ID/notarization/Gatekeeper claim.\nUsage: npm run package:macos -- --output ABSOLUTE_NEW_DIRECTORY\nRequires Node 24.21.0, Rust/CLT, locked registry preparation. Never change resources after outer signing. Public hardened-runtime/JIT/signing/notarization needs separate qualification and untracked credentials.\n");
    else { if (!values.output) throw Error("Missing --output"); process.stdout.write(JSON.stringify(await packageMacos(values.output)) + "\n"); }
  } catch { process.stderr.write("Local packaging failed: review prerequisites/owned paths/runtime integrity. No release qualification claimed.\n"); process.exitCode = 1; }
}

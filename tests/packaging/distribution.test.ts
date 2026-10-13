import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, mkdir, writeFile, rm, readFile, readlink, symlink, realpath, unlink } from "node:fs/promises";
import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { join } from "node:path";
import lock from "../../runtime.lock.json" with { type: "json" };
import { inventory, sha256 } from "../../scripts/runtime-files.ts";
import { finalizeApp, installedPath, type NativeRun } from "../../scripts/package-macos.ts";
import type { RuntimeManifest } from "../../scripts/verify-runtime.ts";

async function fixture(t: test.TestContext) {
  const root = await realpath(await mkdtemp(join(tmpdir(), "dsh distribution fixture "))); t.after(() => rm(root, { recursive: true, force: true }));
  const app = join(root, "App with spaces.app");
  await mkdir(join(app, "Contents/MacOS"), { recursive: true });
  const dsh = join(app, "Contents/Resources/dsh");
  await mkdir(join(dsh, "node_modules/@deepseek-ai/dsh"), { recursive: true });
  await writeFile(join(app, "Contents/Info.plist"), "public metadata fixture");
  await writeFile(join(app, "Contents/MacOS/node"), Buffer.concat([Buffer.from("cffaedfe", "hex"), Buffer.from("unsigned node")]));
  await writeFile(join(app, "Contents/MacOS/dsh-launcher"), Buffer.concat([Buffer.from("cffaedfe", "hex"), Buffer.from("unsigned launcher")]));
  await writeFile(join(dsh, "node_modules/@deepseek-ai/dsh/package.json"), JSON.stringify({ version: lock.dshVersion }));
  await writeFile(join(dsh, "asset.txt"), "public immutable data");
  await symlink("asset.txt", join(dsh, "link.txt"));
  const artifacts = await inventory(app, lock.target, path => installedPath(app, lock.target, path));
  const manifest: RuntimeManifest = { desktopVersion: lock.desktopVersion, dshVersion: lock.dshVersion, upstreamRevision: lock.upstreamRevision, nodeVersion: lock.nodeVersion, target: lock.target, dependencyLockHash: lock.dependencyLockHash, nativeBuildNumber: lock.nativeBuildNumber, artifacts };
  const manifestPath = join(app, "Contents/Resources/runtime-manifest.json");
  await writeFile(manifestPath, JSON.stringify(manifest));
  const calls: string[] = [];
  const run: NativeRun = (command, args) => {
    if (command.endsWith("PlistBuddy")) {
      const key = args[1]!.replace("Print :", "");
      return ({ CFBundleIdentifier: "io.github.yuanlx27.dsh-dsh", CFBundleShortVersionString: "0.2.0", CFBundleVersion: "1", LSMinimumSystemVersion: "14.0", CFBundleExecutable: "deepseek-harness-desktop" } as Record<string, string>)[key] ?? "";
    }
    if (command === "lipo") return "arm64";
    if (command === "codesign" && args.includes("--force")) {
      const path = args.at(-1)!;
      if (path !== app) { writeFileSync(path, Buffer.concat([readFileSync(path), Buffer.from("signed")])); calls.push("nested"); }
      else { const current: RuntimeManifest = JSON.parse(readFileSync(manifestPath, "utf8")); assert.ok(current.artifacts.every(a => a.sha256 !== "")); calls.push("outer"); }
    }
    return "";
  };
  return { app, dsh, manifestPath, manifest, run, calls };
}

test("installed mapping retains target-suffixed inventory while native filenames are unsuffixed", () => {
  assert.equal(installedPath("/owned/App.app", lock.target, `src-tauri/binaries/node-${lock.target}`), "/owned/App.app/Contents/MacOS/node");
  assert.throws(() => installedPath("/owned/App.app", lock.target, "src-tauri/resources/dsh/../escape"));
  assert.throws(() => installedPath("/owned/App.app", lock.target, "src-tauri/binaries/node-wrong-target"));
});

test("nested signing precedes final manifest and outer signing; post-sign bytes/symlink identity match", async t => {
  const f = await fixture(t);
  await finalizeApp(f.app, f.run);
  assert.deepEqual(f.calls, ["nested", "nested", "outer"]);
  const final: RuntimeManifest = JSON.parse(await readFile(f.manifestPath, "utf8"));
  assert.deepEqual(final.artifacts, await inventory(f.app, lock.target, p => installedPath(f.app, lock.target, p)));
  assert.equal(await readlink(join(f.dsh, "link.txt")), "asset.txt");
  assert.equal(final.artifacts.find(a => a.path.endsWith(`node-${lock.target}`))!.sha256, await sha256(join(f.app, "Contents/MacOS/node")));
});

test("wrong complete prerelease cannot be silently finalized", async t => {
  const f = await fixture(t);
  await writeFile(f.manifestPath, JSON.stringify({ ...f.manifest, dshVersion: "0.2.0" }));
  await assert.rejects(finalizeApp(f.app, f.run), /Installed canonical dshVersion mismatch/);
  assert.equal(f.calls.length, 0);
});

test("wrong architecture or Apple numeric build metadata blocks signing", async t => {
  const f = await fixture(t);
  const wrongArch: NativeRun = (command, args) => command === "lipo" ? "x86_64" : f.run(command, args);
  await assert.rejects(finalizeApp(f.app, wrongArch), /Wrong sidecar architecture/);
  const wrongBuild: NativeRun = (command, args) => command.endsWith("PlistBuddy") && args[1] === "Print :CFBundleVersion" ? "999" : f.run(command, args);
  await assert.rejects(finalizeApp(f.app, wrongBuild));
  assert.equal(f.calls.length, 0);
});

test("outer signing cannot silently mutate inventoried resources", async t => {
  const f = await fixture(t);
  const mutation: NativeRun = (command, args) => {
    const result = f.run(command, args);
    if (command === "codesign" && args.includes("--force") && args.at(-1) === f.app) writeFileSync(join(f.dsh, "asset.txt"), "unexpected outer mutation");
    return result;
  };
  await assert.rejects(finalizeApp(f.app, mutation), /Post-sign resource mutation/);
});

test("missing/corrupt/unlisted closure files refuse before any nested signing", async t => {
  const missing = await fixture(t);
  await unlink(join(missing.dsh, "asset.txt"));
  await assert.rejects(finalizeApp(missing.app, missing.run), /ENOENT/);
  assert.equal(missing.calls.length, 0);
  const corrupt = await fixture(t);
  await writeFile(join(corrupt.dsh, "asset.txt"), "corrupt");
  await assert.rejects(finalizeApp(corrupt.app, corrupt.run), /Pre-sign copied closure mismatch/);
  assert.equal(corrupt.calls.length, 0);
  const extra = await fixture(t);
  await writeFile(join(extra.dsh, "unlisted.txt"), "unlisted");
  await assert.rejects(finalizeApp(extra.app, extra.run), /Pre-sign copied closure mismatch/);
  assert.equal(extra.calls.length, 0);
});

test("unapproved sidecar hash mutation is not covered by ordinary finalization", async t => {
  const f = await fixture(t);
  const path = join(f.app, "Contents/MacOS/node");
  await writeFile(path, Buffer.concat([await readFile(path), Buffer.from("unapproved") ]));
  await assert.rejects(finalizeApp(f.app, f.run), /Pre-sign copied closure mismatch/);
  assert.equal(f.calls.length, 0);
});

test("known build-producer Tauri sidecar signatures are verified before hash reconciliation", async t => {
  const f = await fixture(t);
  const path = join(f.app, "Contents/MacOS/node");
  await writeFile(path, Buffer.concat([await readFile(path), Buffer.from("tauri-signature") ]));
  let sidecarVerification = 0;
  const run: NativeRun = (command, args) => {
    if (command === "codesign" && args.includes("--verify") && args.includes("--strict") && args.at(-1) === path) sidecarVerification++;
    return f.run(command, args);
  };
  await finalizeApp(f.app, run, true);
  assert.equal(sidecarVerification, 2); // Pre-reconciliation and post-sign verification.
});

test("packaging CLI help is read-only; unknown flags cannot start a build", () => {
  const script = fileURLToPath(new URL("../../scripts/package-macos.ts", import.meta.url));
  const help = spawnSync(process.execPath, [script, "--help"], { encoding: "utf8" });
  assert.equal(help.status, 0); assert.match(help.stdout, /No Developer ID/);
  const invalid = spawnSync(process.execPath, [script, "--ouput", "/unused"], { encoding: "utf8" });
  assert.equal(invalid.status, 1);
});

test("Tauri-flattened links restore only frozen internal target identity", async t => {
  const f = await fixture(t);
  await unlink(join(f.dsh, "link.txt"));
  await writeFile(join(f.dsh, "link.txt"), "public immutable data");
  await finalizeApp(f.app, f.run);
  assert.equal(await readlink(join(f.dsh, "link.txt")), "asset.txt");
});

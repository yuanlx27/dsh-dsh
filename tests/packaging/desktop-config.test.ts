import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = new URL("../../", import.meta.url);
const config = JSON.parse(await readFile(new URL("src-tauri/tauri.conf.json", root), "utf8"));
const pkg = JSON.parse(await readFile(new URL("package.json", root), "utf8"));

test("sidecar basenames cannot collide with top-level dev resource directories", () => {
  const sidecars: string[] = config.bundle.externalBin;
  const resources: string[] = Object.values(config.bundle.resources);
  for (const path of sidecars) {
    const name = path.split("/").at(-1);
    assert.ok(!resources.some(target => target.split("/")[0] === name), `${name} collides with a dev resource directory`);
  }
});

test("canonical prerelease identity is separate from numeric Apple metadata", () => {
  assert.equal(config.version, pkg.version);
  assert.equal(config.version, "0.2.0-rc.2");
  assert.match(config.bundle.macOS.bundleVersion, /^[1-9]\d*$/);
  const result = spawnSync("plutil", ["-extract", "CFBundleShortVersionString", "raw",
    fileURLToPath(new URL("src-tauri/Info.plist", root))], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), pkg.version.split("-")[0]);
});

test("bundle declares fixed resources, both sidecars and local macOS-only delivery", async () => {
  assert.deepEqual(config.bundle.targets, ["app", "dmg"]);
  assert.deepEqual(config.bundle.externalBin, ["binaries/node", "binaries/dsh-launcher"]);
  assert.equal(config.bundle.macOS.minimumSystemVersion, "14.0");
  assert.equal(config.bundle.macOS.signingIdentity, "-");
  assert.deepEqual(config.bundle.resources, {
    "resources/dsh/": "dsh/",
    "resources/runtime-manifest.json": "runtime-manifest.json",
    "resources/runtime.lock.json": "runtime.lock.json",
    "resources/desktop-web.patch.yml": "desktop-web.patch.yml",
  });
  const cargo = await readFile(new URL(".cargo/config.toml", root), "utf8");
  assert.match(cargo, /target = "aarch64-apple-darwin"/);
  assert.match(cargo, /MACOSX_DEPLOYMENT_TARGET = "14.0"/);
  const manifest = await readFile(new URL("src-tauri/Cargo.toml", root), "utf8");
  assert.match(manifest, /\[profile\.release\.build-override\]\s+strip = "none"/);
  assert.deepEqual(config.app.windows, []);
  assert.deepEqual(config.app.security.capabilities, ["shell"]);
  const capability=JSON.parse(await readFile(new URL("src-tauri/capabilities/shell.json",root),"utf8"));
  assert.deepEqual(capability.permissions,[]);
  assert.equal(capability.local,false);
  assert.deepEqual(capability.remote.urls,["dsh-app://shell/*"]);
  assert.equal(config.plugins, undefined);
  assert.match(config.build.beforeBuildCommand, /runtime:verify/);
});

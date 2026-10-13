import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { lstat, readdir, readlink, realpath } from "node:fs/promises";
import { join, sep } from "node:path";

export interface RuntimeArtifact {
  path: string;
  sha256: string;
  symlink?: string;
}

export async function sha256(path: string) {
  const hash = createHash("sha256");
  for await (const bytes of createReadStream(path)) hash.update(bytes);
  return hash.digest("hex");
}

export async function inventory(root: string, target: string, resolvePath: (logicalPath: string) => string = path => join(root, path)): Promise<RuntimeArtifact[]> {
  const entries: RuntimeArtifact[] = [];
  const resources = await realpath(resolvePath("src-tauri/resources/dsh"));
  async function visit(logicalPath: string): Promise<void> {
    const path = resolvePath(logicalPath);
    const info = await lstat(path);
    if (info.isDirectory()) {
      for (const name of (await readdir(path)).sort()) await visit(`${logicalPath}/${name}`);
    } else {
      const item: RuntimeArtifact = { path: logicalPath, sha256: "" };
      if (info.isSymbolicLink()) {
        const resolved = await realpath(path);
        if (!resolved.startsWith(resources + sep) || !(await lstat(resolved)).isFile()) {
          throw new Error("Runtime symlink must resolve to an internal regular file.");
        }
        item.symlink = await readlink(path);
        item.sha256 = createHash("sha256").update(item.symlink).digest("hex");
      } else if (info.isFile()) {
        item.sha256 = await sha256(path);
      } else throw new Error("Unsupported runtime artifact type.");
      entries.push(item);
    }
  }
  for (const path of [
    `src-tauri/binaries/node-${target}`,
    `src-tauri/binaries/dsh-launcher-${target}`,
    "src-tauri/resources/dsh",
  ]) await visit(path);
  for (const path of ["src-tauri/resources/runtime.lock.json", "src-tauri/resources/desktop-web.patch.yml"]) {
    try { await lstat(resolvePath(path)); }
    catch (error) { if (error instanceof Error && "code" in error && error.code === "ENOENT") continue; throw error; }
    await visit(path);
  }
  return entries.sort((a, b) => a.path.localeCompare(b.path, "en"));
}

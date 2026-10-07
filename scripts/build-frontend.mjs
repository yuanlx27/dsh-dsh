import { access, cp, lstat, mkdir, rm } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const root = fileURLToPath(new URL("../", import.meta.url));
try {
  for (const file of ["src/shell/index.html", "src/shell/main.ts", "src/shell/styles.css"]) {
    await access(join(root, file));
  }
  await rm(join(root, "dist"), { recursive: true, force: true });
  const result = spawnSync(process.execPath, [
    join(root, "node_modules/typescript/bin/tsc"), "--project", join(root, "tsconfig.json"),
  ], { cwd: root, stdio: "inherit" });
  if (result.error || result.status !== 0) throw new Error("TypeScript compilation failed.");
  await mkdir(join(root, "dist"), { recursive: true });
  await cp(join(root, "src"), join(root, "dist"), {
    recursive: true,
    filter: async (path) => {
      const info = await lstat(path);
      return !info.isSymbolicLink() && (info.isDirectory() || /\.(html|css)$/.test(path));
    },
  });
} catch (error) {
  console.error(`Frontend build failed: ${error.message}`);
  process.exitCode = 1;
}

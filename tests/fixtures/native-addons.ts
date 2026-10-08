// Executed by the bundled Node, not by a host/global dependency installation.
import { createRequire } from "node:module";
import { openSync, closeSync } from "node:fs";

const packagePath = process.argv[2];
if (!packagePath) throw new Error("A bundled package path is required.");
const load = createRequire(packagePath);
for (const name of ["node-pty", "sharp", "koffi"]) load(name);
const flock = load("@deepseek-ai/node-addon-system/flock") as {
  tryLockExclusive(fd: number): Promise<void>;
};
const fd = openSync("fixture.lock", "w", 0o600);
try { await flock.tryLockExclusive(fd); }
finally { closeSync(fd); }

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const javascriptFile = /\.(?:[cm]?js|jsx)$/i;

test("tracked sources contain no JavaScript files", () => {
  const files = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard", "-z"], {
    cwd: fileURLToPath(new URL("../../", import.meta.url)), encoding: "utf8",
  }).split("\0").filter(Boolean);
  assert.deepEqual(files.filter((file) => javascriptFile.test(file)), [],
    "Use TypeScript for committed JS/TS-layer code; keep dependencies/output ignored.");
});

test("source policy rejects JS module variants without banning generated-path references", () => {
  for (const file of ["source.js", "source.mjs", "source.cjs", "source.jsx", "source.JS"]) {
    assert.equal(javascriptFile.test(file), true);
  }
  for (const file of ["source.ts", "source.mts", "source.cts", "native.rs", "package.json"]) {
    assert.equal(javascriptFile.test(file), false);
  }
});

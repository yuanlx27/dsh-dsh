import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("native Harness sender observation is debug-only and excludes URLs and bodies", () => {
  const source = readFileSync(new URL("../../src-tauri/src/bridge/macos.rs", import.meta.url), "utf8");
  const start = source.indexOf('#[cfg(all(debug_assertions, feature = "desktop"))]');
  assert.ok(start >= 0);
  const observation = source.slice(start, source.indexOf("// NSString-only", start));
  assert.match(observation, /sender\.main_frame/);
  assert.match(observation, /allowed\.is_err\(\)/);
  assert.match(observation, /wk-harness-child-sender-denied/);
  assert.doesNotMatch(observation, /frame_url|view_url|request\.payload|message\.body|format!/);
});

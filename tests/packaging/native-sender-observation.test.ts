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

test("WK probe retains only bounded scenario outcomes in an accessible result node", () => {
  const source = readFileSync(new URL("../acceptance/browser/wk-negative.ts", import.meta.url), "utf8");
  assert.match(source, /desktop-wk-negative-results/);
  assert.match(source, /textContent = JSON\.stringify\(rows\)/);
  assert.match(source, /setAttribute\("role", "status"\)/);
  assert.match(source, /1500/);
  assert.match(source, /acl-denied/);
  assert.match(source, /command-missing/);
  assert.match(source, /invalid-arguments/);
  assert.match(source, /Permissions associated with this command/);
  assert.ok(source.indexOf('return "command-missing"') < source.indexOf('return "acl-denied"'));
  assert.doesNotMatch(source, /rows\.push\([^\n]*error/);
});

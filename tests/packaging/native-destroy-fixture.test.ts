import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

test("native document destruction is debug-only, host-triggered and primary-scoped", () => {
  const source = readFileSync("src-tauri/src/desktop.rs", "utf8");
  assert.match(source, /#\[cfg\(debug_assertions\)\]\s*fn fixture_destroy/);
  const helper = source.slice(source.indexOf("fn fixture_destroy"), source.indexOf("pub fn run"));
  assert.match(helper, /DSH_DESKTOP_FIXTURE_ROOT/);
  assert.match(helper, /DSH_DESKTOP_FIXTURE_DESTROY/);
  assert.match(helper, /SignalKind::user_defined1/);
  assert.match(helper, /run_on_main_thread/);
  assert.match(helper, /get_webview_window\("primary"\)/);
  assert.match(helper, /window\.destroy\(\)/);
  assert.doesNotMatch(helper, /\.exit\(|\.shutdown\(|\.stop\(|eval\(/);
  assert.match(source, /#\[cfg\(debug_assertions\)\]\s*fixture_destroy\(app\.handle\(\)\)/);
});

test("stale probe gates replay on real replacement and records no opaque handles", () => {
  const source = readFileSync("tests/acceptance/browser/wk-stale.ts", "utf8");
  assert.match(source, /oldWindowGeneration === windowGeneration/);
  assert.match(source, /Native document was not replaced/);
  assert.match(source, /old-http-finish/);
  assert.match(source, /old-stream-write/);
  assert.match(source, /fresh-http-positive/);
  assert.match(source, /localStorage\.removeItem\(key\)/);
  assert.match(source, /JSON\.stringify\(\{ windowGeneration, runtimeGeneration, rows \}\)/);
  assert.doesNotMatch(source, /console\.(log|error)|__DSH_BOOT__|\.reload\(|location\s*=/);
});

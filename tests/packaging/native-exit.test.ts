import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

test("last-window destruction prevents implicit exit without requesting quit", () => {
  const source = readFileSync("src-tauri/src/desktop.rs", "utf8");
  assert.match(source, /RunEvent::ExitRequested \{ code, api, \.\. \}/);
  assert.match(source, /if code\.is_none\(\) \|\| root\.request_quit\(\) \{\s*api\.prevent_exit\(\)/);
});

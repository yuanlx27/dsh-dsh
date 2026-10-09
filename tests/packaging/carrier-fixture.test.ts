import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

test("carrier-only fixture is host-selected inside the existing debug bootstrap gate", () => {
  const native = readFileSync("src-tauri/src/desktop.rs", "utf8");
  const start = native.indexOf("#[cfg(debug_assertions)]\n                    let value");
  const end = native.indexOf("let _ = reply.complete(&value)", start);
  assert.ok(start >= 0 && end > start);
  const block = native.slice(start, end);
  assert.match(block, /DSH_DESKTOP_FIXTURE_ROOT/);
  assert.match(block, /DSH_DESKTOP_FIXTURE_CARRIER_ONLY/);
  assert.match(block, /fixtureCarrierOnly/);
  const browser = readFileSync("src/harness.ts", "utf8");
  const branch = browser.indexOf("bootstrap.fixture===true&&bootstrap.fixtureCarrierOnly===true");
  assert.ok(branch > 0 && branch < browser.indexOf("const native=createNativeTransport"));
  assert.match(browser.slice(branch, browser.indexOf("const native=createNativeTransport")), /return;/);
  assert.match(browser, /Carrier acceptance fixture/);
});

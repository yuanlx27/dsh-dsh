// T017: Node's built-in TypeScript loader tests the shim independently of UI builds.
// Fake native hooks establish adapter behavior, not WKWebView admission evidence.
import assert from "node:assert/strict";
import test from "node:test";
import { installTransport } from "../../src/transport.ts";

function fixture() {
  const http = [], streams = [], external = [];
  const failure = { active: false, attempts: 0 };
  const originalFetch = async (...args) => {
    external.push(args);
    return new Response("unmediated");
  };
  const WebSocket = class UnchangedWebSocket {};
  const page = {
    location: { href: "dsh-app://app/index.html" },
    fetch: originalFetch,
    WebSocket,
  };
  const native = {
    async fetch(path, init) {
      http.push({ path, init });
      return new Response(new Uint8Array([0, 255, 1]), {
        status: 200, headers: { "content-type": "application/octet-stream" },
      });
    },
    async *openStream(endpoint, payload, signal, uplink) {
      failure.attempts++;
      if (failure.active) throw new Error("fixture carrier lost");
      streams.push({ endpoint, payload, signal, uplink });
      yield { fixture: "opaque", payload };
      if (uplink) for await (const item of uplink) yield item;
    },
  };
  const dispose = installTransport(page, native);
  return { page, native, failure, http, streams, external, originalFetch, WebSocket, dispose };
}

async function collect(iterable) {
  const values = [];
  for await (const value of iterable) values.push(value);
  return values;
}

test("install R fetch/openStream hooks without a service origin, cookie, or WebSocket replacement", () => {
  const f = fixture();
  const hooks = f.page.__DSH_TRANSPORT__;
  assert.equal(typeof hooks.fetch, "function");
  assert.equal(typeof hooks.openStream, "function");
  assert.equal(hooks.ownsHost, true);
  assert.equal(hooks.streamBaseUrl, undefined);
  assert.equal(hooks.rpc, undefined);
  assert.equal(f.page.WebSocket, f.WebSocket);
  assert.doesNotMatch(JSON.stringify(hooks), /127\.0\.0\.1|cookie|token|authorization/i);
  f.dispose();
});

test("R's explicit fetch hook preserves method, body bytes, and cancellation", async () => {
  const f = fixture();
  const controller = new AbortController();
  const body = new Uint8Array([255, 0, 195, 169]);
  const response = await f.page.__DSH_TRANSPORT__.fetch("api/fixture", {
    method: "POST", body, signal: controller.signal,
    headers: { "content-type": "application/octet-stream" },
  });
  assert.deepEqual(new Uint8Array(await response.arrayBuffer()), new Uint8Array([0, 255, 1]));
  assert.equal(f.http.length, 1);
  assert.equal(f.http[0].path, "/api/fixture");
  assert.equal(f.http[0].init.method, "POST");
  assert.equal(f.http[0].init.signal.aborted, false);
  controller.abort();
  assert.equal(f.http[0].init.signal.aborted, true);
  assert.deepEqual(new Uint8Array(await new Response(f.http[0].init.body).arrayBuffer()), body);
  assert.equal(f.external.length, 0);
  f.dispose();
});

test("same-app raw upload/download fetches use the same native hooks", async () => {
  const f = fixture();
  const multipart = new Uint8Array([45, 45, 0, 255, 13, 10]);
  await f.page.fetch("dsh-app://app/api/fixture-upload", {
    method: "POST", body: multipart,
    headers: { "content-type": "multipart/form-data; boundary=fixture" },
  });
  const download = await f.page.fetch(new Request("dsh-app://app/api/fixture-download"));
  assert.deepEqual(new Uint8Array(await download.arrayBuffer()), new Uint8Array([0, 255, 1]));
  assert.deepEqual(f.http.map((call) => call.path), ["/api/fixture-upload", "/api/fixture-download"]);
  assert.equal(f.external.length, 0);
  assert.deepEqual(new Uint8Array(await new Response(f.http[0].init.body).arrayBuffer()), multipart);
  f.dispose();
});

test("external fetching and immutable app assets stay outside authenticated forwarding", async () => {
  const f = fixture();
  for (const input of [
    "https://example.invalid/model", "http://127.0.0.1:1/api/fixture",
    "dsh-app://shell/api/fixture", "dsh-app://other/api/fixture",
    "dsh-app://app/assets/index.js", "data:text/plain,fixture",
  ]) assert.equal(await (await f.page.fetch(input)).text(), "unmediated");
  assert.equal(f.http.length, 0);
  assert.equal(f.external.length, 6);
  // Custom URL origins all serialize to "null"; compare authority, not .origin.
  assert.equal(new URL("dsh-app://app/").origin, new URL("dsh-app://shell/").origin);
  f.dispose();
});

test("the explicit service hook cannot select another destination or a token path", async () => {
  const f = fixture();
  for (const path of [
    "https://example.invalid/api", "http://127.0.0.1:1/api", "//other/api",
    "dsh-app://shell/api/fixture", "/api/../secret", "/api?token=fixture",
  ]) await assert.rejects(() => f.page.__DSH_TRANSPORT__.fetch(path, {}));
  assert.equal(f.http.length, 0);
  assert.equal(f.external.length, 0);
  f.dispose();
});

test("pre-aborted fetches are not dispatched", async () => {
  const f = fixture();
  const signal = AbortSignal.abort();
  await assert.rejects(() => f.page.__DSH_TRANSPORT__.fetch("/api/fixture", { signal }), { name: "AbortError" });
  assert.equal(f.http.length, 0);
  f.dispose();
});

test("opaque stream payloads, uplink and downlink retain identity and order", async () => {
  const f = fixture();
  const signal = new AbortController().signal;
  const payload = { args: { fixture: [0, "same", "same"] } };
  const items = [{ unknown: 1 }, { unknown: 1 }, null, [255, 0]];
  const uplink = { async *[Symbol.asyncIterator]() { yield* items; } };
  const frames = await collect(f.page.__DSH_TRANSPORT__.openStream("fixture.remote", payload, signal, uplink));
  assert.deepEqual(frames, [{ fixture: "opaque", payload }, ...items]);
  assert.equal(f.streams.length, 1);
  assert.equal(f.streams[0].payload, payload);
  assert.equal(f.streams[0].signal, signal);
  assert.equal(f.streams[0].uplink, uplink);
  assert.equal(f.http.length, 0);
  assert.equal(f.page.WebSocket, f.WebSocket);
  f.dispose();
});

test("concurrent logical streams do not mix replies or deduplicate equal items", async () => {
  const f = fixture();
  const signal = new AbortController().signal;
  const hooks = f.page.__DSH_TRANSPORT__;
  const uplink = (marker) => ({ async *[Symbol.asyncIterator]() { yield marker; yield marker; } });
  const [left, right] = await Promise.all([
    collect(hooks.openStream("left", { id: 1 }, signal, uplink("left"))),
    collect(hooks.openStream("right", { id: 2 }, signal, uplink("right"))),
  ]);
  assert.deepEqual(left, [{ fixture: "opaque", payload: { id: 1 } }, "left", "left"]);
  assert.deepEqual(right, [{ fixture: "opaque", payload: { id: 2 } }, "right", "right"]);
  assert.equal(f.streams.length, 2);
  f.dispose();
});

test("stream cancellation reaches the native carrier and releases uplink", async () => {
  const f = fixture();
  const controller = new AbortController();
  let released = false;
  const uplink = {
    async *[Symbol.asyncIterator]() { try { yield "one"; yield "two"; } finally { released = true; } },
  };
  const iterator = f.page.__DSH_TRANSPORT__.openStream("fixture", {}, controller.signal, uplink)[Symbol.asyncIterator]();
  await iterator.next();
  await iterator.next();
  controller.abort();
  assert.equal(f.streams[0].signal.aborted, true);
  await iterator.return();
  assert.equal(released, true);
  f.dispose();
});

test("carrier failure is propagated; only an explicit upstream call opens again", async () => {
  const f = fixture();
  f.failure.active = true;
  await assert.rejects(() => collect(f.page.__DSH_TRANSPORT__.openStream("fixture", {}, new AbortController().signal)));
  await new Promise((resolve) => setTimeout(resolve, 30));
  assert.equal(f.failure.attempts, 1);
  await assert.rejects(() => collect(f.page.__DSH_TRANSPORT__.openStream("fixture", {}, new AbortController().signal)));
  assert.equal(f.failure.attempts, 2);
  assert.equal(f.http.length, 0);
  f.dispose();
});

test("detachment rejects stale hook references without replay and restores raw fetch", async () => {
  const f = fixture();
  const old = f.page.__DSH_TRANSPORT__;
  f.dispose();
  f.dispose();
  assert.equal(f.page.fetch, f.originalFetch);
  assert.equal(f.page.WebSocket, f.WebSocket);
  await assert.rejects(() => old.fetch("/api/fixture", {}));
  await assert.rejects(() => collect(old.openStream("fixture", {}, new AbortController().signal)));
  assert.equal(f.http.length, 0);
  assert.equal(f.streams.length, 0);
});

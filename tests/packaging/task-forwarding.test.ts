import assert from "node:assert/strict";
import test from "node:test";
import { recordRRequests } from "../fixtures/r-request-recording.ts";
import { installTransport, type TransportPage } from "../../src/transport.ts";

test("recorded pinned-R envelopes survive explicit duplicates, failures and recreation without replay", async () => {
  const records = await recordRRequests();
  assert.equal(records.length, 2);
  assert.notEqual(records[0].streamId, records[1].streamId);
  assert.deepEqual(records[0].payload, records[1].payload);
  const dispatches: unknown[] = [];
  let fail = false;
  const native = {
    async fetch() { return new Response(); },
    async *openStream(endpoint: string, payload: unknown, signal: AbortSignal) {
      signal.throwIfAborted(); dispatches.push({ endpoint, payload });
      if (fail) throw Error("Controlled carrier unavailable");
      yield { receipt: true };
    },
  };
  const page: TransportPage = { location: { href: "dsh-app://app/" }, fetch };
  const originalSocket = globalThis.WebSocket;
  let detach = installTransport(page, native);
  const hooks = page.__DSH_TRANSPORT__!;
  async function submit(index: number) {
    const row = records[index];
    const values = [];
    for await (const value of page.__DSH_TRANSPORT__!.openStream(row.endpoint, row.payload, new AbortController().signal)) values.push(value);
    return values;
  }
  assert.deepEqual(await submit(0), [{ receipt: true }]);
  assert.deepEqual(await submit(1), [{ receipt: true }]);
  assert.equal(dispatches.length, 2);
  fail = true;
  await assert.rejects(submit(0), /Controlled carrier unavailable/);
  assert.equal(dispatches.length, 3); // No adapter retry on a throw.
  fail = false;
  await Promise.resolve(); assert.equal(dispatches.length, 3); // Recovering supplier isn't a submission.
  detach();
  detach = installTransport(page, native);
  await Promise.resolve(); assert.equal(dispatches.length, 3); // New document hooks replay nothing.
  await assert.rejects(async () => {
    for await (const value of hooks.openStream(records[0].endpoint, records[0].payload, new AbortController().signal)) void value;
  }, /detached/);
  assert.equal(dispatches.length, 3);
  await submit(0); assert.equal(dispatches.length, 4); // Only explicit upstream call after recreation.
  assert.equal(globalThis.WebSocket, originalSocket);
  detach();
});

test("in-flight cancellation and carrier loss never reopen a recorded request", async () => {
  const [record] = await recordRRequests();
  for (const mode of ["abort", "loss"] as const) {
    const controller = new AbortController();
    let release!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; });
    let count = 0;
    const native = {
      async fetch() { return new Response(); },
      async *openStream(_endpoint: string, payload: unknown, signal: AbortSignal) {
        count++; assert.equal(payload, record.payload);
        yield { update: true };
        await gate;
        signal.throwIfAborted();
        throw Error("Controlled in-flight carrier loss");
      },
    };
    const page: TransportPage = { location: { href: "dsh-app://app/" }, fetch };
    const detach = installTransport(page, native);
    try {
      const iterator = page.__DSH_TRANSPORT__!.openStream(record.endpoint, record.payload, controller.signal)[Symbol.asyncIterator]();
      assert.deepEqual((await iterator.next()).value, { update: true });
      const pending = iterator.next();
      if (mode === "abort") controller.abort();
      release(); await assert.rejects(pending);
      await Promise.resolve(); assert.equal(count, 1);
    } finally { release(); detach(); }
  }
});

test("aborted recorded request dispatches nothing and opaque uplink identity is preserved", async () => {
  const [record] = await recordRRequests();
  const controller = new AbortController(); controller.abort();
  let count = 0;
  const uplink: AsyncIterable<unknown> = { async *[Symbol.asyncIterator]() { yield record.payload; yield record.payload; } };
  const native = {
    async fetch() { return new Response(); },
    async *openStream(endpoint: string, payload: unknown, signal: AbortSignal, input?: AsyncIterable<unknown>) {
      count++; assert.equal(endpoint, record.endpoint); assert.equal(payload, record.payload); assert.equal(input, uplink);
      signal.throwIfAborted();
      if (input) for await (const item of input) yield item;
    },
  };
  const page: TransportPage = { location: { href: "dsh-app://app/" }, fetch };
  const detach = installTransport(page, native);
  try {
    const cancelled = page.__DSH_TRANSPORT__!.openStream(record.endpoint, record.payload, controller.signal, uplink);
    await assert.rejects(async () => { for await (const item of cancelled) void item; });
    assert.equal(count, 0);
    const values = [];
    for await (const item of page.__DSH_TRANSPORT__!.openStream(record.endpoint, record.payload, new AbortController().signal, uplink)) values.push(item);
    assert.deepEqual(values, [record.payload, record.payload]); assert.equal(count, 1);
  } finally { detach(); }
});

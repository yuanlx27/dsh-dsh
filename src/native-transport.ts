import type { NativeTransport } from "./transport.js";
export type NativeCall = (operation: "http" | "stream", payload: Record<string, unknown>) => Promise<unknown>;
const CHUNK = 65536;
const MAX_BODY = 300 * 1024 * 1024;
const MAX_FRAME = 100 * 1024 * 1024;
// Reuse R's exported worker-boundary marker; Gateway rebuilds its own class.
function carrierFailure(cause:unknown):Error{return Object.assign(new Error("Desktop stream carrier unavailable.",{cause}),{dshRemoteStreamFailure:{kind:"carrier"}});}
function row(value: unknown): Record<string, unknown> { if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("Invalid desktop reply."); return value as Record<string, unknown>; }
function id(value: unknown): string { const handle = row(value).handle; if (typeof handle !== "string" || !handle) throw new Error("Invalid desktop handle."); return handle; }
function encode(bytes: Uint8Array): string { let text = ""; for (const b of bytes) text += String.fromCharCode(b); return btoa(text); }
function decode(value: unknown): Uint8Array { if (typeof value !== "string" || value.length > CHUNK * 2) throw new Error("Invalid desktop bytes."); const bytes = Uint8Array.from(atob(value), c => c.charCodeAt(0)); if (bytes.length > CHUNK) throw new Error("Invalid desktop bytes."); return bytes; }

export function createNativeTransport(call: NativeCall): NativeTransport {
  let disposed = false;
  const transfers = new Map<string, "http" | "stream">();
  const streams = new Map<string, { controller: ReadableStreamDefaultController<unknown>; terminal: boolean; remoteEnded: boolean }>();
  let carrier: Promise<string> | undefined;
  let sendChain = Promise.resolve();
  const sequences = new Map<string, number>();
  const incoming=new Map<string,number>();
  function current(signal?: AbortSignal | null) { if (disposed) throw new Error("Desktop transport detached."); signal?.throwIfAborted(); }
  async function close(kind: "http" | "stream", handle: string) { transfers.delete(handle); try { await call(kind, { action: "cancel", handle }); } catch { /* Already revoked; no retry. */ } }
  function failStreams(error: unknown) { error=carrierFailure(error); for (const stream of streams.values()) { if (!stream.terminal) {stream.terminal = true; stream.controller.error(error);} } streams.clear(); }
  async function readFrame(handle: string): Promise<string | undefined> {
    const pieces: Uint8Array[] = []; let size = 0;
    while (true) {
      const value = await call("stream", { action: "read", handle });
      if (value === null) return undefined;
      const packet = row(value); const bytes = decode(packet.data);
      if(packet.sequence!==incoming.get(handle)||packet.kind!=="text"||typeof packet.end!=="boolean"||(!packet.end&&bytes.length!==CHUNK))throw new Error("Invalid ordered desktop stream chunk.");
      size += bytes.length; if (size > MAX_FRAME) throw new Error("Desktop stream frame exceeded its bound.");
      pieces.push(bytes);
      if (packet.end === true) {incoming.set(handle,packet.sequence as number+1); const all = new Uint8Array(size); let offset = 0; for (const p of pieces) {all.set(p, offset); offset += p.length;} return new TextDecoder("utf-8", {fatal: true}).decode(all); }
    }
  }
  async function receive(handle: string) {
    try {
      while (!disposed) {
        const text = await readFrame(handle); if (text === undefined) throw new Error("Desktop stream carrier closed.");
        const frame = row(JSON.parse(text));
        if (typeof frame.streamId !== "string") throw new Error("Invalid stream frame.");
        const stream = streams.get(frame.streamId); if (!stream || stream.terminal) continue;
        if (frame.type === "item") { if ((stream.controller.desiredSize ?? 0) <= 0) throw new Error("Desktop stream queue exceeded its bound."); stream.controller.enqueue(frame.value); }
        else if (frame.type === "end") { stream.terminal = true; stream.remoteEnded = true; stream.controller.close(); }
        else if (frame.type === "error") {
          const remote = row(frame.error);
          const error = Object.assign(new Error(typeof remote.message === "string" ? remote.message : "Remote stream failed."), { name: "RemoteError", code: remote.code, details: remote.details, isDSHRemoteError: true, dshRemoteStreamFailure:{kind:"remote",code:remote.code,details:remote.details} });
          stream.terminal = true; stream.remoteEnded = true; stream.controller.error(error);
        } else throw new Error("Invalid stream frame.");
      }
    } catch (error) { failStreams(error); }
    finally { carrier = undefined; sequences.delete(handle);incoming.delete(handle); await close("stream", handle); }
  }
  async function physical(): Promise<string> {
    current();
    if (!carrier) {
      carrier = call("stream", { action: "begin" }).then(value => { const handle = id(value); transfers.set(handle, "stream"); if (disposed) {void close("stream",handle);current();} sequences.set(handle,0);incoming.set(handle,0); void receive(handle); return handle; }).catch(error => {carrier = undefined; throw carrierFailure(error);});
    }
    return carrier;
  }
  function send(handle: string, frame: Record<string, unknown>): Promise<void> {
    const work = sendChain.then(async () => {
      current(); const bytes = new TextEncoder().encode(JSON.stringify(frame));
      if (bytes.length > MAX_FRAME) throw new Error("Desktop stream frame exceeded its bound.");
      const seq = sequences.get(handle); if (seq === undefined || !transfers.has(handle)) throw new Error("Desktop stream carrier closed."); sequences.set(handle, seq + 1);
      for (let offset = 0; offset < bytes.length; offset += CHUNK) await call("stream", { action: "write", handle, sequence: seq, chunk: offset / CHUNK, data: encode(bytes.subarray(offset, offset + CHUNK)), end: offset + CHUNK >= bytes.length });
    });
    const delivered=work.catch(error=>{throw carrierFailure(error);});
    sendChain = delivered.catch(() => {}); return delivered;
  }
  return {
    async fetch(path, init) {
      current(init.signal);
      const method=(init.method??"GET").toUpperCase();
      const headers=new Headers(init.headers);
      if ((method==="GET"||method==="HEAD")&&init.body!=null) throw new TypeError("Body not allowed for this method.");
      // Response's standard BodyInit encoder preserves FormData boundaries and
      // streams without requiring WK's optional streaming request-upload support.
      const input=init.body==null?undefined:new Response(init.body);
      const contentType=input?.headers.get("content-type");
      if(contentType&&!headers.has("content-type"))headers.set("content-type",contentType);
      const handle = id(await call("http", { action: "begin", path, method, headers: [...headers] }));
      transfers.set(handle, "http");
      let reader: ReadableStreamDefaultReader<Uint8Array> | undefined;
      const abort = () => { void close("http", handle); void reader?.cancel(init.signal?.reason).catch(() => {}); };
      init.signal?.addEventListener("abort", abort, {once: true});
      try {
        current(init.signal);
        reader = input?.body?.getReader(); let total = 0; let inputSequence = 0;
        if (reader) try {
          while (true) {
            current(init.signal); const item = await reader.read(); if (item.done) break;
            total += item.value.byteLength; if (total > MAX_BODY) throw new Error("Desktop request body exceeded its bound.");
            for (let offset = 0; offset < item.value.length; offset += CHUNK) await call("http", { action: "write", handle, sequence: inputSequence++, data: encode(item.value.subarray(offset, offset + CHUNK)) });
          }
        } finally { reader.releaseLock(); }
        current(init.signal);
        const head = row(await call("http", {action: "finish", handle})); current(init.signal);
        if (typeof head.status !== "number" || !Array.isArray(head.headers)) throw new Error("Invalid desktop HTTP reply.");
        if ([101,204,205,304].includes(head.status) || method === "HEAD") { await close("http",handle); init.signal?.removeEventListener("abort",abort); return new Response(null, {status:head.status,headers:head.headers as [string,string][]}); }
        let sequence = 0;
        const body = new ReadableStream<Uint8Array>({
          async pull(controller) {
            try {
              current(init.signal); const value = await call("http", {action: "read", handle}); current(init.signal);
              if (value === null) { controller.close(); await close("http", handle); init.signal?.removeEventListener("abort", abort); return; }
              const packet = row(value); if (packet.sequence !== sequence++) throw new Error("Out-of-order desktop reply."); controller.enqueue(decode(packet.data));
            } catch (error) { controller.error(error); await close("http", handle); }
          },
          async cancel() { init.signal?.removeEventListener("abort", abort); await close("http", handle); },
        });
        return new Response(body, {status: head.status, headers: head.headers as [string,string][]});
      } catch (error) { init.signal?.removeEventListener("abort", abort); await close("http", handle); throw error; }
    },
    async *openStream(endpoint, payload, signal, uplink) {
      current(signal); const handle = await physical(); current(signal);
      if(streams.size>=32)throw carrierFailure(new Error("Desktop stream count exceeded its bound."));
      const streamId = Array.from(crypto.getRandomValues(new Uint8Array(16)),byte=>byte.toString(16).padStart(2,"0")).join("");
      let control!: ReadableStreamDefaultController<unknown>;
      const inbox = new ReadableStream<unknown>({start(c) {control=c;}}, {highWaterMark: 32});
      const stream = {controller: control, terminal: false, remoteEnded: false}; streams.set(streamId, stream);
      const reader = inbox.getReader(); const iterator = uplink?.[Symbol.asyncIterator]();
      let stopped = false;
      const abort = () => { if (!stream.terminal) {stream.terminal=true;control.error(signal.reason);} };
      signal.addEventListener("abort", abort, {once: true});
      try {
        await send(handle, {type: "open", streamId, endpoint, payload});
        if (iterator) void (async () => {
          try { while (!stopped) { const item = await iterator.next(); if (stopped) return; if (item.done) {await send(handle, {type:"end",streamId});return;} await send(handle, {type:"item",streamId,value:item.value}); } }
          catch (error) { if (!stream.terminal) {stream.terminal=true;control.error(error);} }
        })();
        while (true) { current(signal); const item = await reader.read(); if (item.done) break; yield item.value; }
      } finally {
        stopped=true; signal.removeEventListener("abort",abort); streams.delete(streamId);
        void iterator?.return?.().catch(() => {});
        reader.releaseLock();
        if (!stream.remoteEnded) await send(handle, {type:"cancel",streamId}).catch(() => {});
      }
    },
    dispose() { if (disposed) return; disposed=true;failStreams(new Error("Desktop transport detached.")); for (const [handle,kind] of transfers) void close(kind,handle); },
  };
}

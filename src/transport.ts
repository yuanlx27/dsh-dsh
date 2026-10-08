// Structural view of pinned R ClientTransportHooks; no business RPC/schema.
export interface NativeTransport {
  fetch(path: string, init: RequestInit): Promise<Response>;
  openStream(endpoint: string, payload: unknown, signal: AbortSignal, uplink?: AsyncIterable<unknown>): AsyncIterable<unknown>;
  dispose?(): void;
}
export interface TransportHooks extends NativeTransport { ownsHost: true; rpc?: never; streamBaseUrl?: never }
export interface TransportPage {
  location: { href: string };
  fetch: typeof fetch;
  __DSH_TRANSPORT__?: TransportHooks;
}
export function servicePath(input: string): string {
  if (!input || input.length > 16384 || input.includes(":") || input.startsWith("//") || /[\\#\x00-\x20]/.test(input)) throw new TypeError("Invalid service target.");
  const path = input.startsWith("/") ? input : `/${input}`;
  const [pathname, query = ""] = path.split("?", 2);
  let decoded: string;
  try { decoded = decodeURIComponent(pathname); decodeURIComponent(query); }
  catch { throw new TypeError("Invalid service target."); }
  if (/[\\%\x00-\x1f]/.test(decoded) || /%2f|%5c/i.test(pathname) || decoded.split("/").slice(1).some(p => !p || p === "." || p === "..") || new URLSearchParams(query).has("token")) throw new TypeError("Invalid service target.");
  return path;
}
export function installTransport(page: TransportPage, native: NativeTransport): () => void {
  const original = page.fetch;
  const previous = page.__DSH_TRANSPORT__;
  let attached = true;
  function current() { if (!attached) throw new Error("Desktop transport detached."); }
  const hooks: TransportHooks = {
    ownsHost: true,
    async fetch(path, init = {}) {
      current(); init.signal?.throwIfAborted();
      const result = await native.fetch(servicePath(path), init);
      current(); return result;
    },
    async *openStream(endpoint, payload, signal, uplink) {
      current(); signal.throwIfAborted();
      if (!endpoint || endpoint.length > 1024) throw new TypeError("Invalid stream endpoint.");
      // Delegate opaque values, including legitimate duplicates. Native scope
      // cancellation belongs to the carrier and native document registration.
      for await (const value of native.openStream(endpoint, payload, signal, uplink)) { current(); signal.throwIfAborted(); yield value; }
    },
  };
  page.__DSH_TRANSPORT__ = hooks;
  const raw: typeof fetch = async (input, init) => {
    current();
    const url = new URL(input instanceof Request ? input.url : String(input), page.location.href);
    if (url.protocol !== "dsh-app:" || url.hostname !== "app" || url.port || url.username || url.password || !url.pathname.startsWith("/api/")) return original.call(page, input, init);
    const request = new Request(input instanceof Request ? input : url.href, init);
    return hooks.fetch(url.pathname + url.search, { method: request.method, headers: request.headers, signal: request.signal, body: request.body });
  };
  page.fetch = raw;
  return () => {
    if (!attached) return;
    attached = false;
    native.dispose?.();
    if (page.fetch === raw) page.fetch = original;
    if (page.__DSH_TRANSPORT__ === hooks) {
      if (previous) page.__DSH_TRANSPORT__ = previous; else delete page.__DSH_TRANSPORT__;
    }
  };
}

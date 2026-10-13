/** Bind a pinned upstream function/class to its published declaration. */
export async function functionExport<T extends ((...args: never[]) => unknown) | { prototype: object }>(path: string, name: string): Promise<T> {
  const module: unknown = await import(new URL(path, import.meta.url).href);
  if (!module || typeof module !== "object" || !(name in module) || typeof Reflect.get(module, name) !== "function") throw Error("Missing pinned export");
  return Reflect.get(module, name) as T;
}

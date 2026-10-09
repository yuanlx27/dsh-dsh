// WebKit's promise replies are JSON strings. These names/labels are freshness,
// not authorization; only native frame metadata can admit a message.
export interface WkPage {location:{hash:string};webkit?:{messageHandlers:Record<string,{postMessage(body:string):Promise<unknown>}>}}
declare global {interface Window {webkit?:WkPage["webkit"]}}
export function generation(page:WkPage,name:string):number {
  const raw=new URLSearchParams(page.location.hash.slice(1)).get(name);
  if (!raw || !/^[1-9]\d{0,15}$/.test(raw)) throw new Error("Native document unavailable.");
  const value=Number(raw);if(!Number.isSafeInteger(value))throw new Error("Native document unavailable.");return value;
}
export async function send(page:WkPage,name:string,payload:unknown):Promise<unknown>{
  const handler=page.webkit?.messageHandlers[name];if(!handler)throw new Error("Native document unavailable.");
  const result=await handler.postMessage(JSON.stringify(payload));
  if(typeof result!=="string")throw new Error("Invalid native reply.");return JSON.parse(result) as unknown;
}

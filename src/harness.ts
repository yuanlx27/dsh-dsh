import {generation,send} from "./wk.js";
import {createNativeTransport} from "./native-transport.js";
import {installTransport} from "./transport.js";
interface Bootstrap {globals:Record<string,unknown>;scripts:string[];preloads:string[];fixture?:boolean;fixtureCarrierOnly?:boolean}
const globals=["__DSH_BOOT__","__DSH_CONNECTION_RECOVERY__","__DSH_MODELS_ONBOARDING__","__DSH_CONTACT_CONFIG__","__DSH_SHORTCUTS_CONFIG__","__DSH_DOCUMENT_PREVIEW_CONFIG__"];
function boot(value:unknown):Bootstrap {
  if(!value||typeof value!=="object")throw new Error("Invalid bootstrap.");
  const row=value as Record<string,unknown>;
  if(!row.globals||typeof row.globals!=="object"||Array.isArray(row.globals)||!Array.isArray(row.scripts)||!Array.isArray(row.preloads)||Object.keys(row.globals).some(k=>!globals.includes(k))||[...row.scripts,...row.preloads].some(u=>typeof u!=="string"||!u.startsWith("dsh-app://app/")))throw new Error("Invalid bootstrap.");
  return row as unknown as Bootstrap;
}
async function start(){
  const windowGeneration=generation(window,"desktop-window"),runtimeGeneration=generation(window,"desktop-runtime");
  const name=`dshBridge${windowGeneration}`;
  const call=(operation:"bootstrap"|"http"|"stream",payload:unknown)=>send(window,name,{windowGeneration,runtimeGeneration,operation,payload});
  const bootstrap=boot(await call("bootstrap",null));
  if(bootstrap.fixture===true&&bootstrap.fixtureCarrierOnly===true){
    const title=document.createElement("h1");title.textContent="Carrier acceptance fixture";
    document.body.replaceChildren(title);return; // Host-selected debug probe, no R frontend carrier.
  }
  for(const [key,value]of Object.entries(bootstrap.globals))Object.defineProperty(window,key,{value,writable:true,configurable:true});
  const native=createNativeTransport(call);installTransport(window,native);
  const checkpoint=(stage:"boot"|"queue"|"entry"|"error"|"mux"|"mux-error")=>{if(bootstrap.fixture===true)void native.fetch(`/api/_desktop_fixture_${stage}`,{}).then(response=>response.body?.cancel()).catch(()=>{});};
  checkpoint("boot");
  // Reuse R's queue/registrations/lazy loader, never another module engine.
  for(const src of bootstrap.preloads){const link=document.createElement("link");link.rel="preload";link.as="script";link.href=src;document.head.append(link);}
  for(const src of bootstrap.scripts){
    if(src==="dsh-app://app/_desktop/queue.js")continue; // already in the head
    await new Promise<void>((resolve,reject)=>{const script=document.createElement("script");script.src=src;script.onload=()=>resolve();script.onerror=()=>reject(new Error("Packaged module unavailable."));document.head.append(script);});
  }
  checkpoint("queue");
  const entry=document.querySelector<HTMLScriptElement>("script[data-upstream-src]")?.dataset.upstreamSrc;
  if(!entry?.startsWith("./assets/"))throw new Error("Packaged frontend unavailable.");
  const module=document.createElement("script");module.type="module";module.src=new URL(entry,location.href).href;module.onload=()=>{checkpoint("entry");if(bootstrap.fixture===true)void(async()=>{
    const abort=new AbortController();const timer=setTimeout(()=>abort.abort(),5000);
    const iterator=native.openStream("_desktop_fixture/unknown",{},abort.signal)[Symbol.asyncIterator]();
    try{await iterator.next();checkpoint("mux-error");}catch(error){const marker=error instanceof Error?(error as Error&{dshRemoteStreamFailure?:{kind?:string}}).dshRemoteStreamFailure:undefined;checkpoint(marker?.kind==="remote"?"mux":"mux-error");}
    finally{clearTimeout(timer);await iterator.return?.().catch(()=>{});}
  })();};module.onerror=()=>checkpoint("error");document.head.append(module);
}
void start().catch(()=>{
  // Fixed diagnostics, no raw exception, token, cookie, program or service URL.
  const title=document.createElement("h1");title.textContent="Harness could not open.";
  const action=document.createElement("p");action.textContent="Quit and reopen the application. If the problem persists, reinstall a matching build.";
  document.body.replaceChildren(title,action);
});

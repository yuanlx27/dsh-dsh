import {generation,send} from "../wk.js";
import type {ShellBridge} from "./state.js";
export function localBridge(page:Window):ShellBridge {
  const name=`dshShell${generation(page,"desktop-window")}`;
  return {status:()=>send(page,name,{action:"status"}),retry:()=>send(page,name,{action:"retry"}),acknowledge:revision=>send(page,name,{action:"acknowledge",revision}),enter:async()=>{await send(page,name,{action:"enter"});}};
}

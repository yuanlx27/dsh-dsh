import test from "node:test";
import assert from "node:assert/strict";
import {createNativeTransport} from "../../src/native-transport.ts";
for(const packet of [{sequence:9,kind:"text"},{sequence:0,kind:"binary"}]){
  test(`mux rejects invalid native frame metadata ${JSON.stringify(packet)}`,async()=>{
    let resolve:((value:unknown)=>void)|undefined;
    const next=new Promise<unknown>(ready=>{resolve=ready;});
    const native=createNativeTransport(async(_operation,p)=>{
      if(p.action==="begin")return {handle:"owned"};
      if(p.action==="read")return next;
      if(p.action==="write"){
        const sent=JSON.parse(Buffer.from(String(p.data),"base64").toString()) as Record<string,unknown>;
        if(sent.type==="open")resolve?.({...packet,end:true,data:Buffer.from(JSON.stringify({type:"item",streamId:sent.streamId,value:"must not arrive"})).toString("base64")});
      }
      return null;
    });
    try{await assert.rejects(()=>native.openStream("opaque",{},new AbortController().signal)[Symbol.asyncIterator]().next(),(error:unknown)=>{
      assert.ok(error instanceof Error);assert.deepEqual((error as Error&{dshRemoteStreamFailure:unknown}).dshRemoteStreamFailure,{kind:"carrier"});return true;
    });}finally{native.dispose!();}
  });
}

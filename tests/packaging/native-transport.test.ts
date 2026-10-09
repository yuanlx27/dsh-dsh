import test from "node:test";
import assert from "node:assert/strict";
import {createNativeTransport, type NativeCall} from "../../src/native-transport.ts";

test("native HTTP chunks preserve multipart bytes and ordered binary response", async () => {
  const form = new FormData(); form.append("file", new Blob([new Uint8Array(70000).fill(255)]), "spaced file.bin");
  const request = new Request("dsh-app://app/api/upload", {method:"POST",body:form});
  const expected = new Uint8Array(await request.clone().arrayBuffer());
  const writes: Uint8Array[]=[]; let cancelled=false; let pull=0;
  const native=createNativeTransport(async (operation,p) => {
    assert.equal(operation,"http");
    if (p.action==="begin") {assert.equal(p.path,"/api/upload");assert.equal(p.method,"POST");return {handle:"owned"};}
    if(p.action==="write") {assert.equal(p.sequence,writes.length);const bytes=Buffer.from(String(p.data),"base64");assert.ok(bytes.length<=65536);writes.push(bytes);return null;}
    if(p.action==="finish") return {status:200,headers:[["content-type","application/octet-stream"]]};
    if(p.action==="read") return pull++===0?{sequence:0,data:Buffer.from([0,255,1]).toString("base64")}:null;
    if(p.action==="cancel") {cancelled=true;return null;}
    throw new Error("Unexpected action");
  });
  const response=await native.fetch("/api/upload",{method:request.method,headers:request.headers,body:request.body});
  assert.deepEqual(new Uint8Array(Buffer.concat(writes)),expected);
  assert.deepEqual(new Uint8Array(await response.arrayBuffer()),new Uint8Array([0,255,1]));
  assert.equal(cancelled,true);native.dispose!();
});

test("native shim demultiplexes opaque values, does not deduplicate, cancels Stop",async()=>{
  const records: Record<string,unknown>[]=[];
  const queue: Record<string,unknown>[]=[];let waiting:((v:unknown)=>void)|undefined; let physical=0;let frameSequence=0;
  function push(frame:Record<string,unknown>){const packet={sequence:frameSequence++,kind:"text",data:Buffer.from(JSON.stringify(frame)).toString("base64"),end:true};if(waiting){const resolve=waiting;waiting=undefined;resolve(packet);}else queue.push(packet);}
  const call:NativeCall=async(operation,p)=>{
    assert.equal(operation,"stream");
    if(p.action==="begin"){physical++;return {handle:"carrier"};}
    if(p.action==="read"){if(queue.length)return queue.shift();return new Promise(resolve=>{waiting=resolve;});}
    if(p.action==="cancel"){waiting?.(null);waiting=undefined;return null;}
    if(p.action==="write"){
      const frame=JSON.parse(Buffer.from(String(p.data),"base64").toString()) as Record<string,unknown>;records.push(frame);
      if(frame.type==="open"){push({type:"item",streamId:frame.streamId,value:frame.payload});push({type:"item",streamId:frame.streamId,value:frame.payload});}
      return null;
    }
    throw new Error("Unexpected action");
  };
  const native=createNativeTransport(call);
  const abort=new AbortController();
  const a=native.openStream("opaque-a",{same:1},abort.signal)[Symbol.asyncIterator]();
  const b=native.openStream("opaque-b",{same:2},new AbortController().signal)[Symbol.asyncIterator]();
  assert.deepEqual((await a.next()).value,{same:1});assert.deepEqual((await b.next()).value,{same:2});
  assert.deepEqual((await a.next()).value,{same:1});assert.deepEqual((await b.next()).value,{same:2});
  assert.equal(physical,1);
  abort.abort();await assert.rejects(()=>a.next());
  assert.ok(records.some(r=>r.type==="cancel"&&r.streamId===records.find(r=>r.endpoint==="opaque-a")!.streamId));
  await b.return!(undefined);native.dispose!();
});

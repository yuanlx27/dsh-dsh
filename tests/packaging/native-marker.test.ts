import test from "node:test";
import assert from "node:assert/strict";
import {createNativeTransport} from "../../src/native-transport.ts";
test("R's existing worker-boundary carrier marker survives native open failure",async()=>{
  const native=createNativeTransport(async()=>{throw new Error("Native carrier unavailable.");});
  try{
    await assert.rejects(()=>native.openStream("opaque",{},new AbortController().signal)[Symbol.asyncIterator]().next(),(error:unknown)=>{
      assert.ok(error instanceof Error);
      assert.deepEqual((error as Error&{dshRemoteStreamFailure:unknown}).dshRemoteStreamFailure,{kind:"carrier"});return true;
    });
  }finally{native.dispose!();}
});
test("BodyInit encoding needs no streaming Request-upload platform feature",async()=>{
  const original=globalThis.Request;
  globalThis.Request=new Proxy(original,{construct(){throw new TypeError("Streaming Request uploads unavailable.");}});
  let body=Buffer.alloc(0);let read=0;let contentType="";
  const native=createNativeTransport(async(_operation,p)=>{
    if(p.action==="begin"){contentType=String((p.headers as string[][]).find(row=>row[0]==="content-type")?.[1]);return {handle:"owned"};}
    if(p.action==="write"){body=Buffer.concat([body,Buffer.from(String(p.data),"base64")]);return null;}
    if(p.action==="finish")return {status:200,headers:[]};
    if(p.action==="read")return read++===0?{sequence:0,data:Buffer.from("ok").toString("base64")}:null;
    return null;
  });
  try{
    const form=new FormData();form.append("name","opaque");
    assert.equal(await (await native.fetch("/api/upload",{method:"POST",body:form})).text(),"ok");
    assert.match(contentType,/^multipart\/form-data; boundary=/);
    assert.ok(body.includes(Buffer.from(contentType.split("boundary=")[1])));
    assert.ok(body.includes(Buffer.from("opaque")));
  }finally{globalThis.Request=original;native.dispose!();}
});

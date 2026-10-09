import test from "node:test";
import assert from "node:assert/strict";
import {generation,send,type WkPage} from "../../src/wk.ts";
test("WK adapter uses native document freshness and string envelopes, never caller origin/cookies",async()=>{
  const bodies:unknown[]=[];
  const page:WkPage={location:{hash:"#desktop-window=12&desktop-runtime=7"},webkit:{messageHandlers:{dshBridge12:{async postMessage(body){bodies.push(JSON.parse(body));return JSON.stringify({safe:true});}}}}};
  assert.equal(generation(page,"desktop-window"),12);
  assert.deepEqual(await send(page,"dshBridge12",{operation:"bootstrap",payload:null}),{safe:true});
  assert.deepEqual(bodies,[{operation:"bootstrap",payload:null}]);
  await assert.rejects(()=>send(page,"unowned",{}));
  assert.throws(()=>generation({location:{hash:"#desktop-window=0"}},"desktop-window"));
});

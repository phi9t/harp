// Model-free crash probe; no receipt cache: every execution crosses controller authority.
import { connect } from 'node:net';
import { readFileSync, writeFileSync, appendFileSync } from 'node:fs';
import { createHash, randomUUID } from 'node:crypto';
import { BACKGROUND_CONTEXT as context } from '@earendil-works/chord/context';
import { createModels, fauxProvider, fauxAssistantMessage, fauxToolCall, Type } from '@earendil-works/pi-ai';
import { Harness, createRegistry, defineExtension, defineTool, ToolResultEntry } from '@earendil-works/pi-durable';
import { openNodeSqliteStorage } from '@earendil-works/pi-durable/storage/sqlite/node';
const rootDir = process.env.PROBE_ROOT!;
const mode = process.argv[2];
const hash = (v: unknown) => createHash('sha256').update(JSON.stringify(v)).digest('hex');
const record = (v: unknown) => appendFileSync(`${rootDir}/worker.jsonl`, JSON.stringify(v)+'\n');
const faux = fauxProvider();
faux.setResponses(mode === 'start' ? [fauxAssistantMessage(fauxToolCall('progress', {}, {id:'original-invocation'}), {stopReason:'toolUse'}), fauxAssistantMessage('done')] : [fauxAssistantMessage('done')]);
const models = createModels(); models.setProvider(faux.provider);
const registry = createRegistry();
registry.install(defineExtension({name:'controller-probe', tools:[defineTool({
 name:'progress', description:'Report fixture progress through controller', parameters:Type.Object({}), replay:'safe',
 execute: async (_args, api, ctx) => {
  // A deliberately different proposal on restart proves the durable memo wins.
  const proposal = {kind:'report_progress', expected_revision:Number(process.env.PROBE_REVISION), idempotency_key:randomUUID(), args:{task:'work', text:'one observable progress event'}};
  const command = await api.memo('controller-request-v1', proposal, ctx);
  record({phase:'before-effect', mode, taskId:api.taskId, callId:api.callId, command, commandHash:hash(command)});
  const request = {auth:{principal:process.env.PROBE_PRINCIPAL, token:process.env.PROBE_TOKEN}, project:'probe', command};
  const reply: any = await new Promise((resolve,reject) => {
   const s=connect(process.env.PROBE_SOCKET!); let data='';
   s.on('connect',()=>s.write(JSON.stringify(request)+'\n'));
   s.on('data',chunk=>{data+=chunk; if(data.includes('\n')) {s.end();resolve(JSON.parse(data.split('\n')[0]));}});
   s.on('error',reject);
  });
  record({phase:'reply', mode, taskId:api.taskId, callId:api.callId, reply, replyHash:hash(reply)});
  return {content:[{type:'text',text:JSON.stringify(reply)}],isError:Boolean(reply.error)};
 }
})]}));
const harness=await Harness.open(await openNodeSqliteStorage(`${rootDir}/pi.sqlite`), {models,registry}, context);
const conversation=await harness.root(context,{agent:{model:{provider:'faux',modelId:'faux-1'}}});
harness.resume();
let id;
if(mode==='start') {id=(await conversation.submit({type:'input',content:'Report progress once'},context)).id; writeFileSync(`${rootDir}/submission.json`,JSON.stringify(id));}
else id=JSON.parse(readFileSync(`${rootDir}/submission.json`,'utf8'));
const done=await (await harness.submission(id,context))!.wait(context);
const entries=await conversation.entries({},1000,undefined,context);
record({phase:'settled',mode,submission:id,status:done.status,toolResults:entries.items.filter(x=>ToolResultEntry.is(x)).map(x=>x.model)});
await harness.close(context);

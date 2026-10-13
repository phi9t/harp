"""Actual native input with a public progress command between save and admission."""
import concurrent.futures,json,os,pathlib,signal,sqlite3,subprocess,sys
from wsutil import WS

def exercise(root,query,command,request,grants,lead,wait,topology,token):
 auth={k:lead[k] for k in ['principal','token']}
 binding=wait(lambda:next((b for b in query('codex_registry')['bindings'] if b['role']=='lead' and b['document'].get('ui_input_ready')),None))
 bid=binding['id'];thread=binding['document']['thread_id']
 def current():return next(b for b in query('codex_registry')['bindings'] if b['id']==bid)['document']
 wait(lambda:current().get('current_turn',{}).get('state')=='terminal',8)
 value={'auth':auth,'project':'p','native_ui':{'method':'turn/start','params':{'threadId':thread,'clientUserMessageId':'repair3-revision-race','input':[{'type':'text','text':'REPAIR3_EXACT_REVISION_RETRY','text_elements':[]}]}}}
 (root/'repair3-ui-request.json').write_text(json.dumps(value))
 barrier=root/'ui-admission-barrier';barrier.write_text('after saved intent before public command admission')
 with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
  future=pool.submit(request,root/'controller.sock',value)
  wait(lambda:(root/'ui-admission-barrier.observed').exists(),8)
  key=(root/'ui-admission-barrier.observed').read_text()
  before=query('snapshot')['revision'];progress=command('report_progress',{'task':'TASK_A','text':'actual unrelated public progress before UI admission'});after=query('snapshot')['revision'];assert after>before
  barrier.unlink();first=future.result(timeout=12)
 def counts():
  with sqlite3.connect((root/'state.db').as_uri()+'?mode=ro',uri=True) as db:
   return {'ui':db.execute('SELECT parameters,request,receipt FROM control_ui_commands WHERE identity=?',(key,)).fetchall(),'receipts':db.execute('SELECT COUNT(*) FROM receipts WHERE key=?',(key,)).fetchone()[0],'operations':db.execute("SELECT COUNT(*) FROM codex_operations WHERE binding=? AND json_extract(intent,'$.text')='REPAIR3_EXACT_REVISION_RETRY'",(bid,)).fetchone()[0]}
 initial=counts();assert len(initial['ui'])==1 and initial['operations']==0 and initial['receipts']==0 and 'revision conflict' in first.get('error',''),(first,initial)
 changed=json.loads(json.dumps(value));changed['native_ui']['params']['input'][0]['text']='changed retained input'
 assert 'error' in request(root/'controller.sock',changed)
 foreign=json.loads(json.dumps(value));foreign['native_ui']['params']['threadId']='foreign-thread'
 assert 'error' in request(root/'controller.sock',foreign)
 retry=request(root/'controller.sock',value);final=counts()
 record={'topology':topology,'public_progress':progress,'revisions':[before,after],'first':first,'initial':initial,'retry':retry,'final':final}
 (root/'repair3-ui-result.json').write_text(json.dumps(record,indent=2));print('actual_repair3_ui_unadmitted_retry',json.dumps(record),flush=True)
 assert 'ok' in retry and final['operations']==1 and final['receipts']==1,'exact proven-unadmitted UI intent must recover after revision conflict'
 assert request(root/'controller.sock',value)==retry and counts()==final
 wait(lambda:current().get('current_turn',{}).get('state')=='terminal',8)
 # The authenticated gateway uses this real pinned native binding. Keepalive adds no turn.
 env={'PATH':'/usr/bin:/bin','CORENIUS_CLIENT_PRINCIPAL':auth['principal'],'CORENIUS_CLIENT_TOKEN':auth['token'],'CORENIUS_UI_TOKEN':'repair3-native-ui-capability'}
 gateway=subprocess.Popen([sys.argv[1],'--native-ui',str(root/'controller.sock'),'p','127.0.0.1:0'],env=env,stdout=(root/'repair3-gateway.stdout').open('wb'),stderr=(root/'repair3-gateway.stderr').open('wb'))
 fd=os.pidfd_open(gateway.pid);stat=pathlib.Path(f'/proc/{gateway.pid}/stat').read_text()
 try:
  port=wait(lambda:int((root/'repair3-gateway.stdout').read_text().strip().split(':')[-1]) if (root/'repair3-gateway.stdout').read_text().strip() else False,5)
  ws=WS.connect('localhost',port,env['CORENIUS_UI_TOKEN']);ops=len(query('codex_registry')['operations'])
  try:
   ws.send(b'exact-native-keepalive',9);header=ws.f.read(2);assert header==bytes([0x8a,22]),header
   assert ws.f.read(22)==b'exact-native-keepalive'
   ws.send(b'unsolicited native pong',10);ws.send({'id':'after-ping','method':'thread/read','params':{'threadId':thread}})
   reply=ws.recv();assert reply['id']=='after-ping' and 'result' in reply,reply
   assert len(query('codex_registry')['operations'])==ops
   print('actual_native_gateway_ping_no_turn',topology,json.dumps({'reply':reply,'operations_before_after':ops}),flush=True)
  finally:ws.close()
 finally:
  if gateway.poll() is None:signal.pidfd_send_signal(fd,signal.SIGTERM)
  gateway.wait(timeout=5);os.close(fd)
  print('owned_repair3_gateway',json.dumps({'pid':gateway.pid,'start_stat':stat,'exit':gateway.returncode,'waited':True}),flush=True)

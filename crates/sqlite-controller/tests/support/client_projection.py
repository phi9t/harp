"""Actual socket/dashboard/MCP accumulated-load and scoped continuation contract."""
import hashlib,json,os,pathlib,signal,socket,subprocess,sys,tempfile,time,tarfile

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

binary=pathlib.Path(sys.argv[1]);token='projection-private-fixture-secret';auth={'principal':'operator','token':token}
env={'PATH':'/usr/bin:/bin','CORENIUS_OPERATOR_TOKEN':token,'CORENIUS_CLIENT_TOKEN':token}
with tempfile.TemporaryDirectory(prefix='cnr-projection-') as directory:
 root=pathlib.Path(directory);sock=root/'s';proc=subprocess.Popen([str(binary),str(root/'state.db'),str(sock)],env=env,stdout=(root/'stdout').open('wb'),stderr=(root/'stderr').open('wb'))
 fd=os.pidfd_open(proc.pid);start=pathlib.Path(f'/proc/{proc.pid}/stat').read_text();revision=0
 def request(value,actor=auth):
  with socket.socket(socket.AF_UNIX) as stream:
   stream.settimeout(15);stream.connect(str(sock));stream.sendall(json.dumps(dict(auth=actor,project='p',**value)).encode()+b'\n');raw=stream.makefile('rb').readline();return json.loads(raw),len(raw)
 def query(kind,args={},actor=auth):
  reply,size=request({'query':{'kind':kind,'args':args}},actor);assert 'ok' in reply,reply;assert size<1_048_576,(kind,size);return reply['ok']
 def command(kind,key,args):
  global revision
  saved={'kind':kind,'idempotency_key':key,'expected_revision':revision,'args':args};reply,_=request({'command':saved});assert 'ok' in reply,reply;revision=reply['ok']['revision'];return reply['ok'],saved
 def client(mode,payload=None,actor=auth):
  e=dict(env,CORENIUS_CLIENT_TOKEN=actor['token'],CORENIUS_CLIENT_PRINCIPAL=actor['principal'])
  return subprocess.run([str(binary),mode,str(sock),'p']+(['--once'] if mode=='--dashboard' else []),input=payload,text=True,env=e,capture_output=True,timeout=15)
 def mcp(kind,args={},actor=auth):
  result=client('--mcp',json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'corenius_query','arguments':{'kind':kind,'args':args}}})+'\n',actor)
  assert result.returncode==0,result.stderr;response=json.loads(result.stdout);assert 'error' not in response,response
  return json.loads(response['result']['content'][0]['text'])
 try:
  deadline=time.monotonic()+5
  while not sock.exists():assert proc.poll() is None and time.monotonic()<deadline;time.sleep(.01)
  command('create_project','project',{'name':'p','base':'0123456789abcdef0123456789abcdef01234567'})
  for task in ['a','b']:command('create_task',task,{'task':task,'brief':'immutable scoped source brief '+task,'work_ref':{'deployment':'fixture','project':'p','item':task},'source_revision':'original-r1'})
  assert client('--dashboard').returncode==0
  assert mcp('control_center')['revision']==revision
  content='é'*32768;assert len(content.encode())==65536
  expected={};first=None
  for i in range(70):
   receipt,saved=command('ask_question','question-'+str(i),{'task':'a' if i%2==0 else 'b','text':content});expected[receipt['result']['question']]='a' if i%2==0 else 'b'
   if first is None:first=(receipt,saved)
  center,center_bytes=request({'query':{'kind':'control_center','args':{}}});events,event_bytes=request({'query':{'kind':'events','args':{'after':0,'limit':1000}}})
  dashboard=client('--dashboard');transport=client('--mcp',json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'corenius_query','arguments':{'kind':'control_center'}}})+'\n')
  print('actual_repair2_accumulated_projection_transport',json.dumps({'valid_questions':70,'individual_utf8_bytes':65536,'control_center_wire_bytes':center_bytes,'event_wire_bytes':event_bytes,'dashboard_exit':dashboard.returncode,'dashboard_error':dashboard.stderr,'mcp_response':json.loads(transport.stdout) if len(transport.stdout)<1000 else {'response_bytes':len(transport.stdout.encode())}}),flush=True)
  assert dashboard.returncode==0 and 'error' not in json.loads(transport.stdout),'accumulated valid input must keep dashboard/MCP usable'
  assert center_bytes<1_048_576 and event_bytes<1_048_576
  assert center['ok']['pagination']['questions']['has_more'] and center['ok']['tasks'][0]['source_revision']=='original-r1'
  def pages(collection,actor=auth):
   cursor=0;items=[];seen=set();count=0
   while True:
    page=mcp('control_records',{'collection':collection,'after':cursor,'limit':20},actor);count+=1
    assert len(page['items'])<=20 and page['page']['collection']==collection
    for item in page['items']:assert item['cursor'] not in seen;seen.add(item['cursor']);items.append(item)
    if not page['page']['has_more']:break
    assert page['page']['after']>cursor;cursor=page['page']['after']
   return items,count
  questions,count=pages('questions');assert {q['id'] for q in questions}==set(expected)
  for question in questions:
   reference=question['text_reference'];assert reference['bytes']==65536 and reference['sha256']==hashlib.sha256(content.encode()).hexdigest()
   full=query('question_input',{'question':question['id']});assert full['text']==content and full['bytes']==65536 and full['sha256']==reference['sha256']
  historical,event_pages=pages('events');assert sum(e['kind']=='ask_question' for e in historical)==70
  large_event=next(e for e in historical if e['kind']=='ask_question');reference=large_event['body']['text_reference'];chunks=[];offset=0
  while True:
   part=mcp('control_input',dict(reference,offset=offset));chunks.append(part['text']);assert part['offset']==offset
   if part['next_offset'] is None:break
   assert part['next_offset']>offset;offset=part['next_offset']
  original=''.join(chunks);assert hashlib.sha256(original.encode()).hexdigest()==large_event['body']['sha256'] and json.loads(original)['args']['text']==content
  replay,_=request({'command':first[1]});assert replay['ok']==first[0]
  a,_=command('admit','worker-a',{'task':'a'});b,_=command('admit','worker-b',{'task':'b'});lead,_=command('replace_lead','lead',{});observer,_=command('create_monitor_observer','observer',{'name':'readonly bounded observer'})
  worker={k:a['result'][k] for k in ['principal','token']};observer_auth={k:observer['result'][k] for k in ['principal','token']};lead_auth={k:lead['result'][k] for k in ['principal','token']}
  own,own_pages=pages('questions',worker);assert len(own)==35 and all(q['task']=='a' for q in own)
  peer=next(q for q in questions if q['task']=='b');assert 'error' in request({'query':{'kind':'question_input','args':{'question':peer['id']}}},worker)[0]
  peer_event=next(e for e in historical if e['kind']=='ask_question' and e['task']=='b');peer_reference=peer_event['body']['text_reference']
  assert 'error' in request({'query':{'kind':'control_input','args':dict(peer_reference,offset=0)}},worker)[0]
  for changed in [dict(reference,sha256='0'*64),dict(reference,bytes=reference['bytes']+1)]:
   assert 'error' in request({'query':{'kind':'control_input','args':changed}})[0]
  utf8_offset=original.encode().index('é'.encode())+1
  assert 'error' in request({'query':{'kind':'control_input','args':dict(reference,offset=utf8_offset)}})[0]
  assert query('snapshot')['revision']==revision
  assert 'error' in request({'query':{'kind':'control_records','args':{'collection':'questions'}}},observer_auth)[0]
  assert mcp('control_center',actor=lead_auth)['occupied_slots']==2
  command('replace_lead','new-lead',{});assert 'error' in request({'query':{'kind':'control_records','args':{'collection':'questions'}}},lead_auth)[0]
  print('actual_repair2_bounded_pages_reconnect',json.dumps({'question_pages':count,'questions':len(questions),'event_pages':event_pages,'events':len(historical),'own_pages':own_pages,'own_questions':len(own),'full_utf8_digest':hashlib.sha256(content.encode()).hexdigest(),'duplicate_receipt_equal':True,'scope_negatives':True}),flush=True)
 finally:
  if proc.poll() is None:signal.pidfd_send_signal(fd,signal.SIGTERM)
  proc.wait(timeout=5);os.close(fd)
  print('owned_projection_resources',json.dumps([{'pid':proc.pid,'start_stat':start,'exit':proc.returncode,'executed_binary':{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()},'waited':True}]),flush=True)
  destination=pathlib.Path(__file__).resolve().parents[4]/'.scratch/sqlite-controller/05/projection-fixtures';destination.mkdir(parents=True,exist_ok=True)
  with tarfile.open(destination/(root.name+'.tar.gz'),'w:gz') as archive:archive.add(root,arcname=root.name)

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
with tempfile.TemporaryDirectory(prefix='monitor05-') as directory:
 root=pathlib.Path(directory);sock=root/'s';proc=subprocess.Popen([str(binary),str(root/'state.db'),str(sock)],env=env,stdout=(root/'stdout').open('wb'),stderr=(root/'stderr').open('wb'))
 fd=os.pidfd_open(proc.pid);start=pathlib.Path(f'/proc/{proc.pid}/stat').read_text();revision=0
 def request(value,actor=auth):
  with socket.socket(socket.AF_UNIX) as stream:
   stream.settimeout(15);stream.connect(str(sock));stream.sendall(json.dumps(dict(auth=actor,project='p',**value)).encode()+b'\n');raw=stream.makefile('rb').readline()
  with (root/'public-calls.jsonl').open('a') as log:log.write(json.dumps({'request':dict(auth=actor,project='p',**value),'response':json.loads(raw)})+'\n')
  return json.loads(raw),len(raw)
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
  grants=[]
  for task in ['a','b']:
   command('create_task','task-'+task,{'task':task,'brief':'immutable intent','work_ref':{'deployment':'fixture','project':'p','item':task},'source_revision':'r1'})
   grant,_=command('admit','admit-'+task,{'task':task});grants.append(grant['result'])
  stream,_=command('register_monitor_stream','stream',{'attempt':grants[0]['attempt'],'source':{'deployment':'fixture','store':'fixture','thread':'own','segment':'one','parser':'visible-v1','identity':'pinned-input'},'live_baseline':0});stream=stream['result']['stream']
  command('ingest_monitor_events','input',{'stream':stream,'expected_ordinal':0,'previous_digest':'','events':[{'item':'one','turn':'t','kind':'public_message','text':'actual visible fixture observation','ordinal':1,'byte_end':20,'digest':'d1'}]})
  batch,_=command('seal_monitor_batch','batch',{'stream':stream,'lane':'history','through':1,'boundary':'compaction'})
  command('publish_monitor_assessment','assessment',{'batch':batch['result']['batch'],'analyst':'observer','version':'1','steps':[{'item':'one','criterion':'original-intent','hypothesis':'inspect observed result','rationale':'public observed input','observation':'actual fixture observation','assessment':'valid_learning','next_test':'inspect exact input'}],'findings':[{'issue':'issue-'+str(i),'criterion':'original-intent','text':'requires authorized disposition','items':['one']} for i in range(20)],'evidence':[]})
  content='é'*32767+'ab';assert len(content.encode())==65536
  findings=query('monitor_context')['findings'];assert len(findings)==20
  for i,finding in enumerate(findings):
   command('adjudicate_monitor_finding','adjudicate-'+str(i),{'finding':finding['id'],'decision':'accepted','rationale':'operator decision'})
   command('authorize_correction','correct-'+str(i),dict(finding=finding['id'],expected_epoch=finding['epoch'],criterion='original-intent',**{key:content for key in ['action','prohibited_scope','next_checkpoint','rationale']}))
  observer,_=command('create_monitor_observer','observer',{'name':'bounded observer'});actor={k:observer['result'][k] for k in ['principal','token']}
  context,context_bytes=request({'query':{'kind':'monitor_context','args':{}}},actor)
  records,records_bytes=request({'query':{'kind':'monitor_records','args':{'collection':'corrections','limit':20}}},actor)
  print('actual_monitor_max_corrections',json.dumps({'count':20,'field_bytes':65536,'context_wire_bytes':context_bytes,'records_wire_bytes':records_bytes}),flush=True)
  assert context_bytes<=200000 and records_bytes<=32768,'monitor projections must have serialized byte budgets'
  assert 'ok' in context and 'ok' in records
  assert client('--dashboard').returncode==0 and mcp('monitor_context',actor=actor)['streams'][0]['reviewed_history']==1
  cursor=0;items=[]
  while True:
   page=mcp('monitor_records',{'collection':'corrections','after':cursor,'limit':20},actor)
   assert len(json.dumps(page).encode())<=32768 and page['page']['byte_limit']==32768
   items+=page['items']
   if not page['page']['has_more']:break
   assert page['page']['after']>cursor;cursor=page['page']['after']
  assert len(items)==20 and len({x['id'] for x in items})==20
  for item in items:
   reference=item['document']['text_reference'];chunks=[];offset=0
   while True:
    chunk=mcp('control_input',dict(reference,offset=offset),actor);chunks.append(chunk['text'])
    assert len(json.dumps(chunk).encode())<=400000 and chunk['offset']==offset
    if chunk['next_offset'] is None:break
    assert chunk['next_offset']>offset;offset=chunk['next_offset']
   full=''.join(chunks);assert len(full.encode())==reference['bytes'] and hashlib.sha256(full.encode()).hexdigest()==reference['sha256']
   document=json.loads(full)
   for field in ['action','prohibited_scope','next_checkpoint','rationale']:assert document[field]==content
  own={k:grants[0][k] for k in ['principal','token']};peer={k:grants[1][k] for k in ['principal','token']}
  assert mcp('control_input',reference,own)['text']==full[:len(mcp('control_input',reference,own)['text'])]
  for bad,who in [(reference,peer),(dict(reference,sha256='0'*64),actor),(dict(reference,bytes=1),actor),(dict(reference,offset=full.encode().index('é'.encode())+1),actor),(dict(reference,collection='events'),actor)]:
   assert 'error' in request({'query':{'kind':'control_input','args':bad}},who)[0]
  assert 'error' in request({'query':{'kind':'control_records','args':{'collection':'questions'}}},actor)[0]
  assert query('snapshot')['revision']==revision
  print('actual_monitor_bounded_roundtrip',json.dumps({'corrections':20,'complete_fields':80,'utf8_sha256':hashlib.sha256(content.encode()).hexdigest(),'observer_operator_worker_scope':True,'credit_unchanged':query('monitor_context')['streams'][0]['reviewed_history']==1}),flush=True)
 finally:
  if proc.poll() is None:signal.pidfd_send_signal(fd,signal.SIGTERM)
  proc.wait(timeout=5);os.close(fd)
  print('owned_projection_resources',json.dumps([{'pid':proc.pid,'start_stat':start,'exit':proc.returncode,'executed_binary':{'path':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest()},'waited':True}]),flush=True)
  destination=_temp_policy_file.parents[1]/'.scratch/sqlite-controller/05-repair3/fixtures';destination.mkdir(parents=True,exist_ok=True)
  with tarfile.open(destination/(root.name+'.tar.gz'),'w:gz') as archive:archive.add(root,arcname=root.name)

"""MODEL_FREE_ACTUAL_PROTOCOL: real controller/supervisor/Codex/companion/tools.
The harness only admits/cancels through the controller, observes owned resources,
and supplies deterministic local Responses SSE bytes. It never dispatches workers.
"""
import pathlib,tempfile,json,subprocess,socket,hashlib,time,sys,threading,http.server,socketserver,os,select,signal,sqlite3
from wsutil import WS
from native_registry_selection import binding_for_thread

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)
BIN=pathlib.Path('/data02/home/philip.yang/.codex/packages/standalone/releases/0.159.3-x86_64-unknown-linux-musl/bin')
HERE=pathlib.Path(__file__).resolve().parents[2]
ORIGIN=pathlib.Path('/data02/home/philip.yang/devx/tmp/corenius-sqlite-topology-sms522gg')
TOKEN='operator-registry-secret'; SUPERVISOR='native-executor-secret'; CONTROL='native-controlplane-secret'
SCHEMA='e587a3d5ffab12e528589e14d385b04d1192b9ae6914ce008cd3ea507e5aa455'
def pin(path):return {'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
def request(path,value):
 with socket.socket(socket.AF_UNIX) as s:
  s.settimeout(35);s.connect(str(path));s.sendall(json.dumps(value).encode()+b'\n');return json.loads(s.makefile('rb').readline())
def fixture_rpc(ws,method,params):
 identifier='owned-fixture-'+str(__import__('uuid').uuid4());ws.send({'id':identifier,'method':method,'params':params})
 while True:
  response=ws.recv()
  if os.environ.get('CORENIUS_TEST_RETIRED_EVENT_FIXTURE')=='1' and response.get('method')=='thread/name/updated':
   if not hasattr(ws,'name_frames'):ws.name_frames=[]
   ws.name_frames.append(response);assert len(ws.name_frames)<4096,'owned fixture notification bound'
  if response.get('id')==identifier:
   assert 'result' in response,response
   return response['result']
def fixture_batch(ws,calls):
 identifiers=[]
 for method,params in calls:
  identifier='owned-batch-'+str(__import__('uuid').uuid4());identifiers.append(identifier);ws.send({'id':identifier,'method':method,'params':params})
 pending=set(identifiers);results=[]
 while pending:
  response=ws.recv()
  if response.get('method')=='thread/name/updated':
   if not hasattr(ws,'name_frames'):ws.name_frames=[]
   ws.name_frames.append(response);assert len(ws.name_frames)<4096
  if response.get('id') in pending:
   assert 'result' in response,response
   pending.remove(response['id']);results.append(response)
 return results
def wait(predicate,seconds=20):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  result=predicate()
  if result:return result
  time.sleep(.04)
 raise AssertionError('behavior deadline')
class UnixHTTP(socketserver.ThreadingMixIn,socketserver.UnixStreamServer):daemon_threads=True
class Owned:
 def __init__(self,r):self.r=r;self.processes=[];self.records=[];self.namespace_fds={}
 def spawn(self,args,name,env):
  p=subprocess.Popen(args,env=env,stdout=open(self.r/(name+'.stdout'),'w'),stderr=open(self.r/(name+'.stderr'),'w'))
  executable=pathlib.Path('/proc/'+str(p.pid)+'/exe');actual_pin={'path':os.readlink(executable),'sha256':hashlib.sha256(executable.read_bytes()).hexdigest()}
  self.processes.append(p);self.records.append({'name':name,'pid':p.pid,'start_stat':pathlib.Path('/proc/'+str(p.pid)+'/stat').read_text(),'argv':args,'executed_binary':actual_pin});return p
 def capture(self,registry):
  executions=[b['document'].get('executor_execution',{}) for b in registry.get('bindings',[])]+[b['document'].get('server_execution',{}) for b in registry.get('bindings',[])]+[server['document'].get('execution',{}) for server in registry.get('servers',[])]
  for execution in executions:
   pid=execution.get('namespace_init');start=execution.get('init_start_ticks');identity=(pid,start)
   if not pid or identity in self.namespace_fds:continue
   try:
    fd=os.pidfd_open(pid);stat=pathlib.Path('/proc/'+str(pid)+'/stat').read_text().rsplit(') ',1)[1].split()
    namespace=int(os.readlink('/proc/'+str(pid)+'/ns/pid').split('[')[1][:-1])
    poll=select.poll();poll.register(fd,select.POLLIN)
    if stat[19]!=start or namespace!=execution['pid_namespace'] or poll.poll(0):os.close(fd);continue
    self.namespace_fds[identity]=(fd,dict(execution))
   except (FileNotFoundError,ProcessLookupError):pass
 def stop(self):
  for p in reversed(self.processes):
   if p.poll() is None:p.kill()
   p.wait(timeout=5)
  for record,p in zip(self.records,self.processes):record['exit']=p.returncode
  namespace_records=[]
  for (pid,start),(fd,execution) in self.namespace_fds.items():
   poll=select.poll();poll.register(fd,select.POLLIN);terminated=bool(poll.poll(0))
   record={'pid':pid,'start_ticks':start,'execution_handle':execution['handle'],'pid_namespace':execution['pid_namespace'],'terminated_after_service_cleanup':terminated,'fixture_cleanup_signalled':False}
   if not terminated:
    signal.pidfd_send_signal(fd,signal.SIGKILL);record['fixture_cleanup_signalled']=True
   record['whole_process_terminated']=bool(poll.poll(5000));os.close(fd);namespace_records.append(record)
  print('owned_fixture_namespaces',json.dumps(namespace_records),flush=True)
  assert all(record['whole_process_terminated'] for record in namespace_records),'owned namespace remains after identity-fenced fixture cleanup'
  print('owned_fixture_resources',json.dumps(self.records),flush=True)

def owned_tmp():
 p=pathlib.Path(os.environ.get('TMPDIR') or '/run/corenius-tmp');p.mkdir(parents=True,exist_ok=True);return str(p)
def run(topology):
 with tempfile.TemporaryDirectory(prefix='cnr-',dir=owned_tmp()) as directory:
  r=pathlib.Path(directory);owned=Owned(r);release=threading.Event();cancel_hold=threading.Event();event_b_release=threading.Event();counts={};counter_lock=threading.Lock();requests=[]
  for name in ['workers','controlplanes','t-exec','t-app','canonical']:(r/name).mkdir()
  (r/'canonical/protected').write_text('never mounted')
  git_native=os.environ.get('CORENIUS_TEST_GIT_NATIVE_FIXTURE')=='1';native_base='0123456789abcdef0123456789abcdef01234567';git_policy=None
  if git_native:
   def trusted_git(path,*args):
    result=subprocess.run(['/usr/bin/git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-C',str(path),*args],env={'PATH':'/usr/bin:/bin','GIT_CONFIG_NOSYSTEM':'1','GIT_CONFIG_GLOBAL':'/dev/null'},capture_output=True,text=True);assert result.returncode==0,result.stderr;return result.stdout.strip()
   (r/'authority').mkdir();(r/'seed').mkdir();trusted_git(r/'canonical','init','--bare','--template=');trusted_git(r/'seed','init','--template=')
   (r/'seed/README').write_text('native Git workspace base');trusted_git(r/'seed','add','README');trusted_git(r/'seed','-c','user.name=Fixture','-c','user.email=fixture@invalid','commit','-m','native base')
   native_base=trusted_git(r/'seed','rev-parse','HEAD');trusted_git(r/'seed','push',str(r/'canonical'),native_base+':refs/heads/protected')
   suite=r/'native-verification.py';suite.write_text("import pathlib,sys,os\nw=pathlib.Path('/workspace');assert (w/(sys.argv[1]+'-shell')).read_text()=='actual_native_shell';assert not pathlib.Path(sys.argv[2]).exists();assert 'CapEff:\\t0000000000000000' in pathlib.Path('/proc/self/status').read_text();print('NATIVE_GIT_FIXED_CONTRACT_OS_ISOLATED')\n")
   policy={'project':'p','root':str(r/'authority'),'attempts_root':str(r/'workers'),'canonical':str(r/'canonical'),'protected_ref':'refs/heads/protected','git':pin(pathlib.Path('/usr/bin/git')),'python':pin(pathlib.Path('/usr/bin/python3')),'bubblewrap':pin(pathlib.Path('/usr/bin/bwrap')),'suites':[{'id':task+'-native','task':task,'base':native_base,'script':pin(suite),'args':[task,str(r)]} for task in ['TASK_A','TASK_B']]}
   git_policy=r/'git.json';git_policy.write_text(json.dumps(policy))
  guard=socket.socket();guard.bind(('127.0.0.1',0));guard.listen();guard_port=guard.getsockname()[1]
  class Handler(http.server.BaseHTTPRequestHandler):
   def log_message(self,*args):pass
   def do_POST(self):
    assert self.path=='/v1/responses',self.path
    value=json.loads(self.rfile.read(int(self.headers['Content-Length'])));requests.append(value)
    text=json.dumps(value.get('input',[]));group='LEAD' if os.environ.get('CORENIUS_TEST_CONTROL_CENTER')=='1' and 'CORENIUS_QUESTIONS' in text else 'TASK_A' if 'TASK_A' in text else 'TASK_B' if 'TASK_B' in text else 'LEAD'
    selected=event_b_release if group=='TASK_B' and os.environ.get('CORENIUS_TEST_EVENT_FIXTURE')=='1' else release
    assert selected.wait(60),'fixture model response release deadline'
    if 'CONTROL_ACTIVE_HOLD' in text:
     deadline=time.monotonic()+15
     while (r/'control-active-hold').exists():
      assert time.monotonic()<deadline,'declared lead hold deadline';time.sleep(.01)
    if 'MODEL_FREE_ACTIVE_CANCEL' in text:cancel_hold.wait(60)
    with counter_lock:n=counts.get(group,0)+1;counts[group]=n
    item={'type':'message','id':f'msg-{group}-{n}','role':'assistant','content':[{'type':'output_text','text':'MODEL_FREE_FIXTURE_DONE'}]}
    if group!='LEAD' and n%3==1:
     probe=f"import pathlib,os,socket,json; w=pathlib.Path('/workspace'); forbidden={json.dumps([str(r),str(r/'controller.sock'),str(r/'state.db'),str(r/'canonical'),str(r/'controlplanes'),str(r/'workers/native-authority'),'/home/philip.yang','/root/.ssh'])}; assert all(not pathlib.Path(p).exists() for p in forbidden); s=socket.socket(); s.settimeout(.1); assert s.connect_ex(('127.0.0.1',{guard_port}))!=0; status=pathlib.Path('/proc/self/status').read_text(); assert 'CapEff:\\t0000000000000000' in status; (w/'{group}-shell').write_text('actual_native_shell'); (w/'{group}-shell-{n}').write_text('actual_native_shell'); (w/'{group}-probe').write_text(json.dumps({{'pid_namespace':os.readlink('/proc/self/ns/pid'),'net_namespace':os.readlink('/proc/self/ns/net'),'host_and_authority_absent':True,'host_tcp_blocked':True}}))"
     if git_native:
      probe += "; import subprocess; assert (w/'.git/objects').is_dir() and not (w/'.git/objects/info/alternates').exists(); g=lambda *a:subprocess.check_output(['/usr/bin/git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-C','/workspace']+list(a),text=True).strip(); assert g('rev-parse','HEAD')=="+repr(native_base)+"; g('add','--all'); g('-c','user.name=NativeFixture','-c','user.email=native@invalid','commit','-m','actual native shell candidate'); (w/'native-git-proof').write_text(g('rev-parse','HEAD'))"
     cmd='/usr/bin/python3 -c '+__import__('shlex').quote(probe)
     code='text(await tools.exec_command('+json.dumps({'cmd':cmd,'workdir':'/workspace','max_output_tokens':1000})+'));'
     item={'type':'custom_tool_call','id':f'ct-{group}-{n}','call_id':f'call-{group}-{n}','name':'exec','namespace':'functions','input':code}
    elif group!='LEAD' and n%3==2:
     patch='*** Begin Patch\n*** Add File: /workspace/'+group+'-patch-'+str(n)+'\n+actual_native_patch\n*** End Patch'
     code='text(await tools.apply_patch('+json.dumps(patch)+'));'
     item={'type':'custom_tool_call','id':f'ct-{group}-{n}','call_id':f'call-{group}-{n}','name':'exec','namespace':'functions','input':code}
    if os.environ.get('CORENIUS_TEST_CONTROL_CENTER')=='1':
     import control_center_native
     item=control_center_native.provider_item(value,r,request,TOKEN) or item
    response=f'response-{group}-{n}';events=[{'type':'response.created','response':{'id':response,'status':'in_progress'}},{'type':'response.output_item.done','output_index':0,'item':item},{'type':'response.completed','response':{'id':response,'status':'completed','output':[item],'usage':{'input_tokens':1,'output_tokens':1,'total_tokens':2}}}]
    body=''.join('data: '+json.dumps(event)+'\n\n' for event in events).encode();self.send_response(200);self.send_header('Content-Type','text/event-stream');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
  provider=UnixHTTP(str(r/'provider.sock'),Handler);threading.Thread(target=provider.serve_forever,daemon=True).start()
  cfg=r/'runtime.toml';cfg.write_text((ORIGIN/'codex/config.toml').read_text().replace(str(ORIGIN/'catalog.json'),'/run/codex/catalog.json').replace('35361','8777'))
  assets={'binary':pin(BIN/'codex'),'companion':pin(BIN/'codex-code-mode-host'),'catalog':pin(ORIGIN/'catalog.json'),'config':pin(cfg),'launcher':pin(HERE/'support/codex_insula.py')}
  supervisor=pathlib.Path(sys.argv[1]).parent/'corenius-attempt-supervisor'
  try:
   for root,transport,kind,secret in [('workers','t-exec','exec',SUPERVISOR),('controlplanes','t-app','app',CONTROL)]:
    profile=dict(assets,kind=kind,transport_root=str(r/transport))
    if kind=='app':profile.update(provider_socket=str(r/'provider.sock'),executor_root=str(r/'t-exec'),own_executor_only=topology=='separate')
    config=r/(root+'.json');config.write_text(json.dumps({'root':str(r/root),'profile':{'executable':'/usr/bin/python3','args':['/run/codex/launcher.py'],'codex':profile}}))
    supervisor_env={'CORENIUS_SUPERVISOR_TOKEN':secret,'PATH':'/usr/bin:/bin','CORENIUS_TEST_PAUSE_EXEC_READY':str(r/'pause-exec-ready')}
    if git_native and kind=='exec':supervisor_env['CORENIUS_GIT_CONFIG']=str(git_policy)
    process=owned.spawn([str(supervisor),str(config),str(r/(root+'.sock'))],root,supervisor_env)
    def ready():
     assert process.poll() is None,(r/(root+'.stderr')).read_text()
     return (r/(root+'.sock')).exists()
    wait(ready,30)
   runtime_config=r/'controller-native.json';runtime_config.write_text(json.dumps({'schema_migration':'1-to-2','topology':topology,'transport_root':str(r/'t-app'),'servers':[{'id':'native','supervisor_socket':str(r/'controlplanes.sock'),'auth_ref':'CONTROLPLANE_SECRET','transport_root':str(r/'t-app'),'worker_supervisor_socket':str(r/'workers.sock'),'worker_auth_ref':'CORENIUS_SUPERVISOR_TOKEN','worker_transport_root':str(r/'t-exec'),'schema_digest':SCHEMA}]}))
   print('exact_native_profile_configs',topology,json.dumps({'workers':json.loads((r/'workers.json').read_text()),'controlplanes':json.loads((r/'controlplanes.json').read_text()),'controller':json.loads(runtime_config.read_text()),'native_config_toml':cfg.read_text()}),flush=True)
   env={'CORENIUS_TEST_PRE_UNSUBSCRIBE':str(r/'pre-unsubscribe'),'PATH':'/usr/bin:/bin','CORENIUS_OPERATOR_TOKEN':TOKEN,'CORENIUS_CODEX_CONFIG':str(runtime_config),'CORENIUS_SUPERVISOR_SOCKET':str(r/'workers.sock'),'CORENIUS_SUPERVISOR_TOKEN':SUPERVISOR,'CONTROLPLANE_SECRET':CONTROL,'CORENIUS_TEST_LOSE_NATIVE_ACK':str(r/'lose-ack'),'CORENIUS_TEST_HOLD_CODEX':str(r/'hold-codex'),'CORENIUS_TEST_CRASH_AFTER_NATIVE_CONFIRM':str(r/'crash-confirm'),'CORENIUS_TEST_ACK_FENCE_FIXTURE':os.environ.get('CORENIUS_TEST_ACK_FENCE_FIXTURE','0'),'CORENIUS_TEST_CRASH_FILE':str(r/'crash-boundary'),'CORENIUS_TEST_CRASH_POINT':os.environ.get('CORENIUS_TEST_ADOPT_CRASH','initial_turn_confirmation' if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1' else 'empty_thread_binding' if os.environ.get('CORENIUS_TEST_EMPTY_CRASH')=='1' else ''),'CORENIUS_TEST_EVENT_BARRIER':str(r/'event-barrier'),'CORENIUS_TEST_PAGE_SETUP':str(r/'page-setup'),'CORENIUS_TEST_RETIRED_EVENTS':str(r/'retired-events'),'CORENIUS_TEST_LOSE_UNSUBSCRIBE_ACK':str(r/'lose-unsubscribe'),'CORENIUS_TEST_ADOPT_EVENTS':str(r/'adopt-events'),'CORENIUS_TEST_CLEANUP_HISTORY':str(r/'cleanup-history')}
   if os.environ.get('CORENIUS_TEST_CONFIRM_FIXTURE')=='1':(r/'crash-confirm').write_text('declared test-only post-confirmation crash')
   if os.environ.get('CORENIUS_TEST_ADOPT_EVENT_FIXTURE')=='1':
    (r/'adopt-events').write_text('declared actual native frame/projection observation');(r/'event-barrier').write_text('declared wait for actual recovered first-turn completion');(r/'hold-codex').write_text('fixture setup only: pin exact admitted actors before native RPC')
   if os.environ.get('CORENIUS_TEST_EMPTY_CRASH')=='1':(r/'crash-boundary').write_text('declared durable empty binding crash')
   if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1':(r/'crash-boundary').write_text('declared initial native turn receipt before projection adoption')
   if os.environ.get('CORENIUS_TEST_PAGE_FIXTURE')=='1':(r/'page-setup').write_text('declared native-only setup threads, no controller assignments')
   if git_native:env['CORENIUS_GIT_CONFIG']=str(git_policy)
   env['CORENIUS_TEST_UI_ADMISSION_BARRIER']=str(r/'ui-admission-barrier')
   env['CORENIUS_TEST_EARLY_QUESTION']=str(r/'early-question')
   env['CORENIUS_TEST_LOSE_TOOL_REPLY']=str(r/'lose-tool-reply')
   env['CORENIUS_TEST_QUESTION_RACE']=str(r/'question-race')
   env['CORENIUS_TEST_UI_DRAIN_BARRIER']=str(r/'ui-drain-barrier')
   env['CORENIUS_TEST_TOOL_ROUTING']=str(r/'tool-routing')
   controller=owned.spawn([sys.argv[1],str(r/'state.db'),str(r/'controller.sock')],'controller',env)
   def startup():
    assert controller.poll() is None,(r/'controller.stderr').read_text()
    return (r/'controller.sock').exists()
   wait(startup)
   auth={'principal':'operator','token':TOKEN}
   def query(kind,args=None):
    value=request(r/'controller.sock',{'auth':auth,'project':'p','query':{'kind':kind,'args':args or {}}});assert 'ok' in value,value
    if kind=='codex_registry':owned.capture(value['ok'])
    return value['ok']
   def command(kind,args):
    for retry in range(8):
     rev=0 if kind=='create_project' else query('snapshot')['revision']
     value=request(r/'controller.sock',{'auth':auth,'project':'p','command':{'kind':kind,'args':args,'expected_revision':rev,'idempotency_key':str(__import__('uuid').uuid4())}})
     if 'ok' in value:return value['ok']['result']
     assert 'revision' in value.get('error',''),value
    raise AssertionError('command revision contention')
   command('create_project',{'name':'p','base':native_base})
   grants=[]
   page_fixture=os.environ.get('CORENIUS_TEST_PAGE_FIXTURE')=='1'
   # The empty-binding crash applies to the first native thread: defer the peer
   # until A has established and checked the real pagination boundary.
   tasks=['TASK_B','TASK_A'] if page_fixture and os.environ.get('CORENIUS_TEST_PAGE_PEER_FIRST')=='1' else ['TASK_A','TASK_B']
   for task in tasks:
    command('create_task',{'task':task,'work_ref':{'deployment':'fixture','project':'p','item':task},'source_revision':'r1','brief':task+' MODEL_FREE_ACTUAL_PROTOCOL'})
    if not page_fixture or task=='TASK_A':grants.append(command('admit',{'task':task}))
   if page_fixture:assert len(grants)==1
   if os.environ.get('CORENIUS_TEST_ADOPT_EVENT_FIXTURE')=='1':
    (r/'adopt-events.actors').write_text(json.dumps({'a':grants[0]['attempt'],'b':grants[1]['attempt']}));(r/'hold-codex').unlink()
   confirm_restarted=False;page_ws=None;padding=[]
   def padding_setup(document,count):
    nonlocal page_ws
    if page_ws is None:
     app=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':document['server_execution']['binding']})['ok'];actual=app['runtime']
     page_ws=WS.connect('localhost',8788,app['capability'],unix=actual['endpoint'],path='/?corenius_start='+actual['runtime_start'])
     page_ws.s.settimeout(8)
     fixture_rpc(page_ws,'initialize',{'clientInfo':{'name':'owned-pagination-setup-only','version':'0.1'},'capabilities':{'experimentalApi':True}});page_ws.send({'method':'initialized','params':{}})
    deadline=time.monotonic()+100
    for n in range(count):
     assert time.monotonic()<deadline,'finite real native padding setup deadline'
     value=fixture_rpc(page_ws,'thread/start',{'model':'gpt-6-sol','modelProvider':'corenius-relay','cwd':'/workspace','approvalPolicy':'never','sandbox':'danger-full-access','ephemeral':True,'environments':[{'environmentId':document['environment_id'],'cwd':'/workspace','runtimeWorkspaceRoots':['/workspace']}]})
     padding.append(value['thread']['id'])
     if n%10==0:print('actual_native_padding_progress',topology,json.dumps({'created':len(padding),'seconds':round(time.monotonic()-(deadline-100),3)}),flush=True)
    print('actual_native_padding_setup_only',topology,json.dumps({'count':count,'all_ids':padding}),flush=True)
   page_errors=[]
   if os.environ.get('CORENIUS_TEST_PAGE_FIXTURE')=='1':
    def prepare_padding():
     try:
      wait(lambda:(r/'page-setup.observed').exists(),30)
      padding_setup(json.loads((r/'page-setup.observed').read_text()),101);(r/'page-setup').unlink()
     except Exception as error:page_errors.append(__import__('traceback').format_exc());print('actual_native_padding_failure',topology,json.dumps({'created':padding,'error':page_errors[-1]}),flush=True);(r/'page-setup').unlink(missing_ok=True)
    threading.Thread(target=prepare_padding,daemon=True).start()
   adopt_errors=[]
   if os.environ.get('CORENIUS_TEST_ADOPT_EVENT_FIXTURE')=='1':
    def complete_adopted_turn():
     try:
      wait(lambda:(r/'event-barrier.observed').exists(),30);release.set()
      workspace=r/'workers'/grants[0]['attempt']/'workspace';wait(lambda:(workspace/'TASK_A-shell').exists() and (workspace/'TASK_A-patch-2').exists(),10)
      with sqlite3.connect((r/'state.db').as_uri()+'?mode=ro',uri=True) as reader:doc=json.loads(reader.execute('SELECT document FROM codex_bindings WHERE attempt=?',(grants[0]['attempt'],)).fetchone()[0])
      app=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':doc['server_execution']['binding']})['ok'];actual=app['runtime'];observer=WS.connect('localhost',8788,app['capability'],unix=actual['endpoint'],path='/?corenius_start='+actual['runtime_start']);observer.s.settimeout(8)
      try:
       fixture_rpc(observer,'initialize',{'clientInfo':{'name':'owned-adoption-history-observer','version':'0.1'},'capabilities':{'experimentalApi':True}});observer.send({'method':'initialized','params':{}})
       def terminal_history():
        observed=fixture_rpc(observer,'thread/read',{'threadId':doc['thread_id'],'includeTurns':True});turns=observed['thread']['turns'];return observed if turns and turns[-1]['id']==doc['turn_id'] and turns[-1]['status']=='completed' else False
       observed=wait(terminal_history,10);print('actual_adopted_first_turn_completed',topology,json.dumps(observed),flush=True)
      finally:observer.close()
     except Exception:adopt_errors.append(__import__('traceback').format_exc())
     finally:(r/'event-barrier').unlink(missing_ok=True)
    threading.Thread(target=complete_adopted_turn,daemon=True).start()
   def bound():
    nonlocal controller,confirm_restarted
    if (os.environ.get('CORENIUS_TEST_CONFIRM_FIXTURE')=='1' or os.environ.get('CORENIUS_TEST_EMPTY_CRASH')=='1' or os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1') and controller.poll()==79 and not confirm_restarted:
     assert (r/'crash-confirm.observed' if os.environ.get('CORENIUS_TEST_CONFIRM_FIXTURE')=='1' else r/'crash-boundary.observed').exists()
     if os.environ.get('CORENIUS_TEST_PAGE_FIXTURE')=='1':
      assert not page_errors,page_errors
      observation=json.loads((r/'crash-boundary.observed').read_text())
      with sqlite3.connect((r/'state.db').as_uri()+'?mode=ro',uri=True) as reader:
       target=json.loads(reader.execute('SELECT document FROM codex_bindings WHERE attempt=?',(grants[0]['attempt'],)).fetchone()[0])
      assert len(grants)==1 and target['thread_id']==observation['thread_id'],'pagination crash must target the sole admitted A'
      padding_setup(json.loads((r/'page-setup.observed').read_text()),101)
      cursor=None;pages=[];deadline=time.monotonic()+8
      while True:
       assert time.monotonic()<deadline,'native pagination observation deadline'
       page=fixture_rpc(page_ws,'thread/loaded/list',{'limit':100,'cursor':cursor});pages.append(page);cursor=page.get('nextCursor')
       if cursor is None:break
       assert len(pages)<5,'unexpected native fixture page count'
      assert observation['thread_id'] not in pages[0]['data'] and any(observation['thread_id'] in page['data'] for page in pages[1:]),'actual empty target must be beyond first native page'
      print('actual_native_empty_target_after_first_page',topology,json.dumps({'target':observation,'pages':pages,'padding_count':len(padding)}),flush=True)
     if os.environ.get('CORENIUS_TEST_EMPTY_RESTART')=='1' or os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1':
      observation=json.loads((r/'crash-boundary.observed').read_text())
      with sqlite3.connect((r/'state.db').as_uri()+'?mode=ro',uri=True) as observer:
       rows=[{'id':bid,'document':json.loads(raw)} for bid,raw in observer.execute('SELECT id,document FROM codex_bindings')]
       servers=[{'id':sid,'document':json.loads(raw)} for sid,raw in observer.execute('SELECT id,document FROM codex_servers')]
      owned.capture({'bindings':rows,'servers':servers});empty=next(b for b in rows if b['document'].get('thread_id')==observation['thread_id']);server=next(server for server in servers if server['id']==empty['document']['server_id'])
      app=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':server['document']['effect_binding']})['ok'];actual=app['runtime']
      if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1':
       observation['server_incarnation']=empty['document']['server_incarnation'];assert empty['document'].get('turn_id') is None and empty['document']['thread_id']==observation['thread_id']
       observer=WS.connect('localhost',8788,app['capability'],unix=actual['endpoint'],path='/?corenius_start='+actual['runtime_start'])
       try:
        fixture_rpc(observer,'initialize',{'clientInfo':{'name':'owned-fixture-history-observer','version':'0.1'},'capabilities':{'experimentalApi':True}});observer.send({'method':'initialized','params':{}})
        if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM_PERSISTENCE')=='1':
         started=time.monotonic();deadline=started+10;samples=[]
         while True:
          native=fixture_rpc(observer,'thread/read',{'threadId':observation['thread_id'],'includeTurns':True})['thread']
          samples.append({'elapsed':time.monotonic()-started,'turns':[{'id':turn['id'],'client_ids':[item.get('clientId') for item in turn['items'] if item.get('clientId')]} for turn in native['turns']]})
          if any(turn['id']==observation['turn_id'] and any(item.get('clientId')==observation['client_user_message_id'] for item in turn['items']) for turn in native['turns']):break
          assert time.monotonic()<deadline,'original10s initial persistence predicate deadline';time.sleep(.03)
         (r/'initial-confirm-persistence-diagnostic.json').write_text(json.dumps({'receipt':observation,'samples':samples,'deadline_seconds':10,'driver_change':'poll original turn+clientId predicate instead of truthy thread; no runtime fence change'},indent=2))
         print('actual_initial_same10s_persistence_predicate_diagnostic',topology,json.dumps({'receipt':observation,'samples':samples,'deadline_seconds':10}),flush=True)
        else:
         def original_receipt_persisted():
          value=fixture_rpc(observer,'thread/read',{'threadId':observation['thread_id'],'includeTurns':True})['thread']
          return value if any(turn['id']==observation['turn_id'] and any(item.get('clientId')==observation['client_user_message_id'] for item in turn['items']) for turn in value['turns']) else False
         native=wait(original_receipt_persisted,10)
        if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM_ORDERING')=='1':
         def allowed_snapshot(value):
          return [{'id':turn['id'],'status':turn.get('status'),'items':[{'id':item.get('id'),'type':item.get('type'),'clientId':item.get('clientId')} for item in turn.get('items',[]) if item.get('type')=='userMessage']} for turn in value['turns']]
         started=time.monotonic();first=allowed_snapshot(native);later=[]
         matches=lambda value:any(turn['id']==observation['turn_id'] and any(item.get('clientId')==observation['client_user_message_id'] for item in turn['items']) for turn in value['turns'])
         if not matches(native):
          deadline=started+10
          while time.monotonic()<deadline:
           sample=fixture_rpc(observer,'thread/read',{'threadId':observation['thread_id'],'includeTurns':True})['thread'];later.append({'elapsed':time.monotonic()-started,'snapshot':allowed_snapshot(sample),'matches_original_predicate':matches(sample)})
           if matches(sample):break
           time.sleep(.03)
         with sqlite3.connect((r/'state.db').as_uri()+'?mode=ro',uri=True) as db:
          operations=[{'id':oid,'kind':kind,'state':state,'clientUserMessageId':json.loads(raw).get('clientUserMessageId')} for oid,kind,state,raw in db.execute('SELECT id,kind,state,intent FROM codex_operations WHERE binding=?',(empty['id'],))]
         measured={'receipt':observation,'first_snapshot':first,'first_matches_original_predicate':matches(native),'later':later,'operations':operations,'original_assertion_preserved':True,'diagnostic_only_seconds':10,'qualification':'first truthy snapshot remains acceptance input; later reads never replace it; metadata only'}
         (r/'initial-confirm-ordering.json').write_text(json.dumps(measured,indent=2));print('actual_initial_original_first_read_ordering',topology,json.dumps(measured),flush=True)
        assert any(turn['id']==observation['turn_id'] and any(item.get('clientId')==observation['client_user_message_id'] for item in turn['items']) for turn in native['turns'])
        print('actual_initial_confirmation_before_adoption',topology,json.dumps({'receipt':observation,'binding':empty,'native_thread':native}),flush=True)
       finally:observer.close()
      assert actual['incarnation']==observation['server_incarnation']
      descriptor=os.pidfd_open(actual['runtime_pid'])
      try:
       stat=pathlib.Path('/proc/'+str(actual['runtime_pid'])+'/stat').read_text().rsplit(') ',1)[1].split();namespace=int(os.readlink('/proc/'+str(actual['runtime_pid'])+'/ns/pid').split('[')[1][:-1]);poll=select.poll();poll.register(descriptor,select.POLLIN)
       assert stat[19]==actual['runtime_start'] and namespace==actual['pid_namespace'] and not poll.poll(0)
       signal.pidfd_send_signal(descriptor,signal.SIGKILL);assert poll.poll(5000)
      finally:os.close(descriptor)
      def replaced():
       value=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':server['document']['effect_binding']})
       return value['ok']['runtime'] if 'ok' in value and value['ok']['runtime']['incarnation']!=actual['incarnation'] else False
      replacement=wait(replaced,20)
      if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1':release.set()
      print('actual_empty_native_incarnation_replaced',topology,json.dumps({'old':actual,'new':replacement,'empty_thread':observation}),flush=True)
     controller=owned.spawn([sys.argv[1],str(r/'state.db'),str(r/'controller.sock')],'controller-confirm-restarted',env);confirm_restarted=True
    try:registry=query('codex_registry')
    except (ConnectionRefusedError,FileNotFoundError,ConnectionResetError,json.JSONDecodeError):return False
    if page_fixture and confirm_restarted and len(grants)==1:
     assert len(padding)==202 and not page_errors
     grants.append(command('admit',{'task':'TASK_B'}))
     print('actual_native_pagination_admission_order',topology,json.dumps({'task_creation_order':tasks,'admission_order':['TASK_A','TASK_B'],'A_attempt':grants[0]['attempt'],'B_attempt':grants[1]['attempt'],'padding_count_before_B':len(padding),'crash_boundary_before_B':True}),flush=True)
     return False
    if os.environ.get('CORENIUS_TEST_EMPTY_RESTART')=='1' and confirm_restarted:
     observation=json.loads((r/'crash-boundary.observed').read_text());empty=next((b for b in registry['bindings'] if b['document'].get('thread_id')==observation['thread_id']),None)
     return registry if empty and empty['state']=='unknown' and empty['document'].get('recovery_state')=='qualification_unknown' else False
    if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1' and confirm_restarted:
     observation=json.loads((r/'crash-boundary.observed').read_text());target=next((b['document'] for b in registry['bindings'] if b['document'].get('thread_id')==observation['thread_id']),None)
     return registry if len(registry['bindings'])==2 and all(b['document'].get('thread_id') and b['document'].get('turn_id') for b in registry['bindings']) and target and target.get('current_turn',{}).get('id')==observation['turn_id'] and any(turn.get('id')==observation['turn_id'] for turn in target.get('native_turns',[])) and target.get('recovery_state')=='remote_registered_input_blocked' and not target.get('ui_input_ready') else False
    return registry if len(registry['bindings'])==2 and all(b['document'].get('thread_id') and b['document'].get('turn_id') and b['document'].get('ui_input_ready') for b in registry['bindings']) else False
   try:registry=wait(bound,30)
   except AssertionError:
    if os.environ.get('CORENIUS_TEST_ADOPT_EVENT_FIXTURE')=='1':
     registry=query('codex_registry');a=next((b for b in registry['bindings'] if b['attempt']==grants[0]['attempt']),None);b=next((b for b in registry['bindings'] if b['attempt']==grants[1]['attempt']),None)
     projection=json.loads((r/'adopt-events.projection').read_text()) if (r/'adopt-events.projection').exists() else None
     frames=[json.loads(line) for line in (r/'adopt-events.received').read_text().splitlines()] if (r/'adopt-events.received').exists() else []
     delivered=[event for event in frames if event.get('method')=='thread/status/changed' and projection and event.get('params',{}).get('threadId')==projection['thread_id']]
     workspace=r/'workers'/grants[0]['attempt']/'workspace'
     prerequisites=bool(confirm_restarted and a and projection==a['document'] and projection.get('ui_input_ready') and projection.get('current_turn',{}).get('native_status')=='completed' and (workspace/'TASK_A-shell').exists() and (workspace/'TASK_A-patch-2').exists() and [event['params']['status']['type'] for event in delivered]==['active','idle'] and all(projection['native_activity'].count(event)==1 for event in delivered) and (r/'hold-codex').exists() and (r/'adopt-events.disarmed').exists() and (r/'adopt-events.deferrals').exists() and not (r/'adopt-events.retry-B').exists() and (not b or not b['document'].get('ui_input_ready')) and not adopt_errors)
     print('declared_exact_B_deferral_deadline',topology,json.dumps({'credited_scheduling_RED_prerequisites':prerequisites,'registry':registry,'projection':projection,'received_frames':frames,'delivered':delivered,'deferrals':(r/'adopt-events.deferrals').read_text() if (r/'adopt-events.deferrals').exists() else None,'disarmed':(r/'adopt-events.disarmed').exists(),'B_retry_after_disarm':(r/'adopt-events.retry-B').exists(),'hold':(r/'hold-codex').exists(),'adopt_errors':adopt_errors}),flush=True)
    if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1':print('actual_initial_recovery_failure',topology,json.dumps(query('codex_registry')),flush=True)
    raise
   assert query('snapshot')['occupied_slots']==2
   if os.environ.get('CORENIUS_TEST_CONFIRM_FIXTURE')=='1':
    assert confirm_restarted
    for binding in registry['bindings']:
     starts=[o for o in registry['operations'] if o['binding']==binding['id'] and o['kind']=='thread/start']
     assert len(starts)==1 and starts[0]['observation']['thread']['id']==binding['document']['thread_id'],'confirmed native creation must be adopted without replay after daemon crash'
    print('native_confirmed_creation_adopted',topology,flush=True)
   if os.environ.get('CORENIUS_TEST_PROJECTION_FIXTURE')=='1':
    for binding in registry['bindings']:
     document=binding['document'];assert document.get('current_turn',{}).get('id')==document['turn_id'] and document['current_turn']['state'] in ['active','terminal','unknown'],'typed current native turn projection required'
     assert all(event.get('params',{}).get('threadId',event.get('params',{}).get('thread',{}).get('id'))==document['thread_id'] for event in document.get('native_activity',[])), 'native activity must carry this exact bound thread identity'
    assert all(server['document'].get('capabilities',{}).get('explicit_remote_turn_environment') is True for server in registry['servers']), 'registered actual native capability contract required'
   print('actual_bound_registry',topology,json.dumps(registry),flush=True)
   if os.environ.get('CORENIUS_TEST_ADOPT_EVENT_FIXTURE')=='1':
    assert confirm_restarted and (r/'hold-codex').exists()
    a=next(b for b in registry['bindings'] if b['attempt']==grants[0]['attempt']);b=next(b for b in registry['bindings'] if b['attempt']==grants[1]['attempt']);projection=json.loads((r/'adopt-events.projection').read_text());assert projection==a['document']
    frames=[json.loads(line) for line in (r/'adopt-events.received').read_text().splitlines()]
    delivered=[event for event in frames if event.get('method')=='thread/status/changed' and event.get('params',{}).get('threadId')==projection['thread_id']]
    print('actual_adoption_received_frame_snapshot',topology,json.dumps({'all_frames':frames,'projection':a,'peer':b,'setup_errors':adopt_errors}),flush=True)
    assert not adopt_errors,adopt_errors
    assert [event['params']['status']['type'] for event in delivered]==['active','idle'],'actual recovered first-turn lifecycle notifications required before loss oracle'
    turns=projection['native_turns'];assert len(turns)==1 and turns[0]['id']==projection['turn_id'] and turns[0]['status']=='completed' and any(item.get('clientId')==next(op['intent']['clientUserMessageId'] for op in registry['operations'] if op['binding']==a['id'] and op['kind']=='turn/start') for item in turns[0]['items'])
    workspace=r/'workers'/grants[0]['attempt']/'workspace';assert (workspace/'TASK_A-shell').exists() and (workspace/'TASK_A-patch-2').exists()
    print('actual_recovered_thread_received_frames',topology,json.dumps({'delivered':delivered,'projection':a,'peer':b}),flush=True)
    peer=[event for event in b['document']['native_activity'] if event.get('method')=='thread/started'];assert peer and all(event['params']['thread']['id']==b['document']['thread_id'] for event in peer),'new-thread peer creation event positive control'
    assert all(projection['native_activity'].count(event)==1 for event in delivered),'validated adopted thread must retain each actual delivered first-turn notification once'
    assert all(event.get('params',{}).get('threadId',event.get('params',{}).get('thread',{}).get('id'))==projection['thread_id'] for event in projection['native_activity'])
    assert len([op for op in registry['operations'] if op['binding']==a['id'] and op['kind']=='thread/start'])==1 and len([op for op in registry['operations'] if op['binding']==a['id'] and op['kind']=='turn/start'])==1
    print('actual_recovered_thread_notifications_once',topology,json.dumps(delivered),flush=True)
    (r/'hold-codex').unlink();(r/'adopt-events').unlink();release.set()
    for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
    wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_PRE_UNSUBSCRIBE_FIXTURE')=='1':
    release.set();workspace=r/'workers'/grants[0]['attempt']/'workspace'
    wait(lambda:(workspace/'TASK_A-shell').exists() and (workspace/'TASK_A-patch-2').exists(),15)
    def completed():
     row=next(b for b in query('codex_registry')['bindings'] if b['attempt']==grants[0]['attempt']);turns=row['document'].get('native_turns',[])
     return row if turns and turns[-1]['status']=='completed' else False
    before=wait(completed,15)
    if os.environ.get('CORENIUS_TEST_RESTART_LOST_ACK_FIXTURE')=='1':
     command('cancel_attempt',{'attempt':grants[1]['attempt']});wait(lambda:query('snapshot')['occupied_slots']==1,15)
    if git_native:
     grant=grants[0];value=request(r/'controller.sock',{'auth':{'principal':grant['principal'],'token':grant['token']},'project':'p','command':{'kind':'capture_candidate','args':{'report':'actual pinned Codex native tool commit'},'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}});assert 'ok' in value,value
     candidate=value['ok']['result'];assert candidate['commit']==(workspace/'native-git-proof').read_text() and candidate['base']==native_base and candidate['commit']!=native_base
     verification=command('verify_candidate',{'candidate':candidate['candidate']});assert verification['verdict']=='pass',verification
     inventory=query('workspace_inventory');print('actual_native_private_git_candidate',topology,json.dumps({'candidate':candidate,'verification':verification,'inventory':inventory,'policy':policy}),flush=True)
     assert all((r/'workers'/g['attempt']/'workspace/.git').is_dir() for g in grants)
    (r/'pre-unsubscribe').write_text('owned first-unsubscribe boundary')
    command('cancel_attempt',{'attempt':grants[0]['attempt']})
    wait(lambda:(r/'pre-unsubscribe.observed').exists() and (r/'hold-codex').exists(),15)
    paused=query('codex_registry');row=next(b for b in paused['bindings'] if b['id']==before['id']);original=row['document']['turn_shutdown']
    assert original['terminal_receipt'] and not [o for o in paused['operations'] if o['binding']==row['id'] and o['kind']=='thread/unsubscribe']
    actual=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':row['document']['server_execution']['binding']})['ok']['runtime']
    fd=os.pidfd_open(actual['runtime_pid'])
    try:
     stat=pathlib.Path('/proc/'+str(actual['runtime_pid'])+'/stat').read_text().rsplit(') ',1)[1].split();poll=select.poll();poll.register(fd,select.POLLIN)
     assert stat[19]==actual['runtime_start'] and int(os.readlink('/proc/'+str(actual['runtime_pid'])+'/ns/pid').split('[')[1][:-1])==actual['pid_namespace'] and not poll.poll(0)
     signal.pidfd_send_signal(fd,signal.SIGKILL);assert poll.poll(5000)
    finally:os.close(fd)
    def restarted():
     value=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':row['document']['server_execution']['binding']})
     return value['ok']['runtime'] if 'ok' in value and value['ok']['runtime']['incarnation']!=actual['incarnation'] else False
    replacement=wait(restarted,20);assert replacement['store_id']==actual['store_id']
    print('actual_pre_unsubscribe_restart',topology,json.dumps({'original_stop':original,'before':row,'old_runtime':actual,'fresh_runtime':replacement}),flush=True)
    lost_after_restart=os.environ.get('CORENIUS_TEST_RESTART_LOST_ACK_FIXTURE')=='1';pending=None
    if lost_after_restart:
     (r/'lose-unsubscribe').write_text('actual fresh-incarnation unsubscribe lost acknowledgement');(r/'cleanup-history').write_text('actual subsequent history discard observation barrier')
    (r/'hold-codex').unlink()
    if lost_after_restart:
     wait(lambda:(r/'lose-unsubscribe.observed').exists() and (r/'cleanup-history.observed').exists() and (r/'hold-codex').exists(),15)
     paused=query('codex_registry');pending=next(o for o in paused['operations'] if o['binding']==row['id'] and o['kind']=='thread/unsubscribe');assert pending['state']=='unknown' and pending['intent']['server_incarnation']==replacement['incarnation']
     print('actual_restart_lost_unsubscribe_ack_pending',topology,json.dumps({'operation':pending,'original_stop':original,'replacement':replacement}),flush=True)
     (r/'hold-codex').unlink()
    def retired():
     projection=query('codex_registry');current=next(b for b in projection['bindings'] if b['id']==row['id'])
     if current['document'].get('cleanup_stop_observation',{}).get('server_incarnation')==replacement['incarnation']:
      (r/'latest-cleanup-observation.json').write_text(json.dumps(current))
     return (current,projection) if current['state']=='revoked' else False
    current,projection=wait(retired,15)
    ops=[o for o in projection['operations'] if o['binding']==row['id'] and o['kind']=='thread/unsubscribe']
    assert current['document']['turn_shutdown']==original and len(ops)==1 and ops[0]['state']=='confirmed' and ops[0]['intent']['server_incarnation']==replacement['incarnation']
    if pending:
     assert ops[0]['id']==pending['id'] and ops[0]['intent']==pending['intent'] and ops[0]['observation']['previous_state']=='unknown'
     assert ops[0]['intent']['stop_qualification']['server_incarnation']==replacement['incarnation'] and ops[0]['intent']['stop_qualification']!=original
    assert current['document']['cleanup_stop_observation']['terminal_receipt'] and not current['document']['ui_input_ready']
    wait(lambda:next(a for a in query('snapshot')['attempts'] if a['id']==grants[0]['attempt'])['state']=='exited',10)
    execution=row['document']['executor_execution'];fd,_=owned.namespace_fds[(execution['namespace_init'],execution['init_start_ticks'])];poll=select.poll();poll.register(fd,select.POLLIN);assert poll.poll(5000),'actual owned executor namespace must stop'
    print('actual_fresh_first_cleanup_terminal_immutable',topology,json.dumps({'binding':current,'journal':ops}),flush=True)
    if not lost_after_restart:command('cancel_attempt',{'attempt':grants[1]['attempt']})
    wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_CLEANUP_FIXTURE')=='1':
    release.set();workspace=r/'workers'/grants[0]['attempt']/'workspace';wait(lambda:(workspace/'TASK_A-shell').exists() and (workspace/'TASK_A-patch-2').exists(),10)
    def completed():
     rows=query('codex_registry')['bindings'];row=next(row for row in rows if row['attempt']==grants[0]['attempt']);turns=row['document'].get('native_turns',[]);return row if turns and turns[-1]['id']==row['document']['turn_id'] and turns[-1]['status']=='completed' else False
    before=wait(completed,10);(r/'lose-unsubscribe').write_text('declared discard after actual native unsubscribe reply');(r/'cleanup-history').write_text('declared discard of later actual history reply after retained terminal stop')
    command('cancel_attempt',{'attempt':grants[0]['attempt']});wait(lambda:(r/'hold-codex').exists() and (r/'cleanup-history.observed').exists(),15)
    paused=query('codex_registry');row=next(row for row in paused['bindings'] if row['id']==before['id']);history=json.loads((r/'cleanup-history.observed').read_text());lost=json.loads((r/'lose-unsubscribe.observed').read_text());cleanup=[op for op in paused['operations'] if op['binding']==row['id'] and op['kind']=='thread/unsubscribe'];snapshot=query('snapshot')
    print('actual_cleanup_retry_observation',topology,json.dumps({'prior_terminal':history['prior_terminal'],'discarded_real_history':history['actual_native_history'],'discarded_real_unsubscribe':lost,'binding':row,'cleanup':cleanup,'snapshot':snapshot}),flush=True)
    assert history['prior_terminal']['terminal_receipt'] and history['prior_terminal']['turn_id']==before['document']['turn_id'] and history['actual_native_history']['thread']['turns'][-1]['status']=='completed' and lost['status']=='unsubscribed'
    assert row['document']['turn_shutdown']==history['prior_terminal'],'a later failed observation must not overwrite a confirmed terminal receipt'
    assert row['state']=='revoking' and not row['document']['ui_input_ready'] and len(cleanup)==1 and cleanup[0]['state']=='unknown','journaled unknown unsubscribe must remain unresolved'
    assert row['document']['cleanup_stop_observation']['state']=='unknown' and 'discarded' in row['document']['cleanup_stop_observation']['reason']
    wait(lambda:query('snapshot')['occupied_slots']==1,10)
    change=os.environ.get('CORENIUS_TEST_CLEANUP_RESTART')
    if change:
     app=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':row['document']['server_execution']['binding']})['ok'];actual=app['runtime']
     if change=='connection':
      controller.kill();assert controller.wait(timeout=5)==-signal.SIGKILL
      controller=owned.spawn([sys.argv[1],str(r/'state.db'),str(r/'controller.sock')],'controller-cleanup-restarted',env)
      def reachable():
       assert controller.poll() is None
       try:return query('codex_registry')
       except (ConnectionRefusedError,FileNotFoundError,ConnectionResetError,json.JSONDecodeError):return False
      wait(reachable,15);replacement=actual
     else:
      descriptor=os.pidfd_open(actual['runtime_pid'])
      try:
       stat=pathlib.Path('/proc/'+str(actual['runtime_pid'])+'/stat').read_text().rsplit(') ',1)[1].split();namespace=int(os.readlink('/proc/'+str(actual['runtime_pid'])+'/ns/pid').split('[')[1][:-1]);poll=select.poll();poll.register(descriptor,select.POLLIN)
       assert stat[19]==actual['runtime_start'] and namespace==actual['pid_namespace'] and not poll.poll(0)
       signal.pidfd_send_signal(descriptor,signal.SIGKILL);assert poll.poll(5000)
      finally:os.close(descriptor)
      def replaced_cleanup_app():
       reply=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':row['document']['server_execution']['binding']})
       return reply['ok']['runtime'] if 'ok' in reply and reply['ok']['runtime']['incarnation']!=actual['incarnation'] else False
      replacement=wait(replaced_cleanup_app,20);assert replacement['store_id']==actual['store_id']
     (r/'hold-codex').unlink()
     def unresolved_cleanup():
      after=query('codex_registry');current=next(b for b in after['bindings'] if b['id']==row['id']);reason=current['document'].get('subscription_retirement',{}).get('reason','');return (current,after) if 'unresolved' in reason else False
     current,after=wait(unresolved_cleanup,15);ops=[op for op in after['operations'] if op['binding']==row['id'] and op['kind']=='thread/unsubscribe']
     assert current['state']=='revoking' and not current['document']['ui_input_ready'] and current['document']['turn_shutdown']==history['prior_terminal'] and len(ops)==1 and ops[0]['id']==cleanup[0]['id'] and ops[0]['state']=='unknown'
     assert len([op for op in after['operations'] if op['binding']==row['id'] and op['kind']=='turn/start'])==1 and not [op for op in after['operations'] if op['binding']==row['id'] and op['kind']=='turn/interrupt'] and query('snapshot')['occupied_slots']==1
     print('actual_cleanup_changed_qualification_unresolved',topology,json.dumps({'change':change,'old_runtime':actual,'current_runtime':replacement,'binding':current,'journal':ops}),flush=True)
     command('cancel_attempt',{'attempt':grants[1]['attempt']});wait(lambda:query('snapshot')['occupied_slots']==0);return
    (r/'hold-codex').unlink()
    def reconciled_cleanup():
     after=query('codex_registry');retired=next(row for row in after['bindings'] if row['id']==before['id']);ops=[op for op in after['operations'] if op['binding']==retired['id'] and op['kind']=='thread/unsubscribe'];return (retired,ops,after) if retired['state']=='revoked' else False
    retired,ops,after=wait(reconciled_cleanup,15);assert retired['document']['turn_shutdown']==history['prior_terminal'] and len(ops)==1 and ops[0]['id']==cleanup[0]['id'] and ops[0]['state']=='confirmed' and ops[0]['observation']['native']['status']=='notSubscribed' and ops[0]['observation']['previous_state']=='unknown'
    assert 'discarded' in retired['document'].get('cleanup_stop_uncertainty',{}).get('reason',''),'later successful reconciliation must retain the prior failed history observation separately'
    assert len([op for op in after['operations'] if op['binding']==retired['id'] and op['kind']=='turn/start'])==1 and not [op for op in after['operations'] if op['binding']==retired['id'] and op['kind']=='turn/interrupt']
    print('actual_cleanup_retry_terminal_immutable_reconciled',topology,json.dumps({'binding':retired,'journal':ops,'lost_acknowledgement':lost}),flush=True)
    command('cancel_attempt',{'attempt':grants[1]['attempt']});wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_RETIRED_EVENT_FIXTURE')=='1':
    assert topology=='shared'
    a=next(b for b in registry['bindings'] if b['attempt']==grants[0]['attempt']);b=next(b for b in registry['bindings'] if b['attempt']==grants[1]['attempt']);server=next(s for s in registry['servers'] if s['id']==a['document']['server_id'])
    app=request(r/'controlplanes.sock',{'token':CONTROL,'action':'native_lookup','binding':server['document']['effect_binding']})['ok'];actual=app['runtime']
    event_ws=WS.connect('localhost',8788,app['capability'],unix=actual['endpoint'],path='/?corenius_start='+actual['runtime_start']);event_ws.s.settimeout(8)
    fixture_rpc(event_ws,'initialize',{'clientInfo':{'name':'owned-retired-notification-setup-only','version':'0.1'},'capabilities':{'experimentalApi':True}});event_ws.send({'method':'initialized','params':{}})
    fixture_rpc(event_ws,'thread/resume',{'threadId':a['document']['thread_id']})
    (r/'lose-unsubscribe').write_text('declared loss only after actual native unsubscribe reply')
    (r/'retired-events').write_text('declared pause only after native stop disposition retained')
    errors=[]
    def produce_retired_events():
     try:
      wait(lambda:(r/'retired-events.observed').exists(),15);retired=json.loads((r/'retired-events.observed').read_text());assert retired['authority_state']=='revoked' and retired['turn_shutdown'].get('interrupt_receipt',retired['turn_shutdown'].get('terminal_receipt'))
      assert fixture_rpc(event_ws,'thread/read',{'threadId':a['document']['thread_id'],'includeTurns':True})['thread']['turns'][-1]['status'] in ['interrupted','completed','failed']
      receipts=[];start=time.monotonic()
      for first in range(0,1030,64):
       assert time.monotonic()-start<15,'fixed retired event setup deadline'
       receipts.extend(fixture_batch(event_ws,[('thread/name/set',{'threadId':a['document']['thread_id'],'name':'OWNED_RETIRED_SETUP_'+str(n)}) for n in range(first,min(first+64,1030))]))
      for n in range(3):receipts.append(fixture_rpc(event_ws,'thread/name/set',{'threadId':b['document']['thread_id'],'name':'OWNED_ACTIVE_PEER_'+str(n)}))
      proof={'retired_binding':retired,'native_rpc_receipts':len(receipts),'setup_seconds':round(time.monotonic()-start,3),'retired_notifications':[e for e in event_ws.name_frames if e['params']['threadId']==a['document']['thread_id']]}
      assert len(proof['retired_notifications'])==1030
      (r/'retired-events.setup').write_text(json.dumps(proof));print('actual_retired_native_notification_setup',topology,json.dumps(proof),flush=True)
     except Exception:errors.append(__import__('traceback').format_exc());print('actual_retired_setup_failure',topology,json.dumps({'error':errors[-1],'actual_frames':len(getattr(event_ws,'name_frames',[]))}),flush=True)
     finally:(r/'retired-events').unlink(missing_ok=True)
    threading.Thread(target=produce_retired_events,daemon=True).start();command('cancel_attempt',{'attempt':grants[0]['attempt']})
    try:wait(lambda:(r/'retired-events.peer').exists() or errors,20)
    except AssertionError:
     trace=[json.loads(line) for line in (r/'retired-events.received').read_text().splitlines()] if (r/'retired-events.received').exists() else []
     print('actual_retired_native_notification_failure',topology,json.dumps({'received_count':len(trace),'received':trace,'registry':query('codex_registry')}),flush=True);raise
    assert not errors,errors
    peer=json.loads((r/'retired-events.peer').read_text());events=[event for event in peer['native_activity'] if event.get('method')=='thread/name/updated'];assert len(events)==3 and len({event['params']['threadName'] for event in events})==3 and all(event['params']['threadId']==b['document']['thread_id'] for event in events)
    after=query('codex_registry');retired=next(row for row in after['bindings'] if row['id']==a['id']);assert retired['state']=='revoked' and not retired['document']['ui_input_ready'];assert query('snapshot')['occupied_slots']==1
    lost=json.loads((r/'lose-unsubscribe.observed').read_text());assert lost['status']=='unsubscribed'
    cleanup=[op for op in after['operations'] if op['binding']==a['id'] and op['kind']=='thread/unsubscribe'];assert len(cleanup)==1 and cleanup[0]['state']=='confirmed' and cleanup[0]['observation']['native']['status']=='notSubscribed' and cleanup[0]['observation']['previous_state']=='unknown'
    print('actual_native_unsubscribe_lost_ack_reconciled',topology,json.dumps({'lost_actual_reply':lost,'journal':cleanup}),flush=True)
    print('actual_retired_notifications_preserve_active_peer',topology,json.dumps({'peer':peer,'retired':retired}),flush=True)
    event_ws.close();(r/'hold-codex').unlink();release.set();command('cancel_attempt',{'attempt':grants[1]['attempt']});wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1':
    observation=json.loads((r/'crash-boundary.observed').read_text());row=next(b for b in registry['bindings'] if b['document'].get('thread_id')==observation['thread_id']);doc=row['document'];assert doc['turn_id']==observation['turn_id'] and doc['current_turn']['id']==observation['turn_id']
    assert len([o for o in registry['operations'] if o['binding']==row['id'] and o['kind']=='thread/start'])==1 and len([o for o in registry['operations'] if o['binding']==row['id'] and o['kind']=='turn/start'])==1
    history=doc['native_turns'];assert sum(any(item.get('clientId')==observation['client_user_message_id'] for item in turn['items']) for turn in history)==1
    assert not doc['ui_input_ready'] and doc['recovery_state']=='remote_registered_input_blocked','app restart must not silently authorize local input'
    print('actual_initial_turn_recovered_after_app_restart',topology,json.dumps({'receipt':observation,'binding':row}),flush=True)
    for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
    wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_EMPTY_RESTART')=='1':
    observation=json.loads((r/'crash-boundary.observed').read_text());empty=binding_for_thread(registry['bindings'],observation['thread_id']);assert empty['document'].get('turn_id') is None and not empty['document']['ui_input_ready']
    end=time.monotonic()+2
    samples=[]
    try:
     while time.monotonic()<end:
      current=query('codex_registry');row=next(b for b in current['bindings'] if b['id']==empty['id'])
      samples.append({'elapsed':time.monotonic()-(end-2),'state':row['state'],'turn_id':row['document'].get('turn_id'),'ui_input_ready':row['document']['ui_input_ready'],'server_incarnation':row['document'].get('server_incarnation'),'server_store':row['document'].get('server_store'),'recovery_state':row['document'].get('recovery_state'),'thread_starts':len([o for o in current['operations'] if o['binding']==empty['id'] and o['kind']=='thread/start']),'turn_starts':len([o for o in current['operations'] if o['binding']==empty['id'] and o['kind']=='turn/start'])})
      assert row['state']=='unknown' and row['document'].get('turn_id') is None and not row['document']['ui_input_ready']
      assert len([o for o in current['operations'] if o['binding']==empty['id'] and o['kind']=='thread/start'])==1 and not any(o['binding']==empty['id'] and o['kind']=='turn/start' for o in current['operations']);occupied=query('snapshot')['occupied_slots'];samples[-1]['occupied_slots']=occupied;assert occupied==2;time.sleep(.05)
    finally:
     metadata={'original_receipt':observation,'binding':empty['id'],'deadline_seconds':2,'original_assertions_preserved':True,'samples':samples,'qualification':'test-only allowed tuple metadata; no changed runtime branch or timeout'}
     (r/'empty-restart-diagnostic.json').write_text(json.dumps(metadata,indent=2));print('actual_empty_restart_original_hold_diagnostic',topology,json.dumps(metadata),flush=True)
    print('actual_empty_changed_incarnation_unknown_no_replay',topology,json.dumps(observation),flush=True);release.set()
    for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
    wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_PAGE_FIXTURE')=='1':
    assert len(padding)==202
    for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
    release.set();wait(lambda:query('snapshot')['occupied_slots']==0);page_ws.close();return
   if os.environ.get('CORENIUS_TEST_EMPTY_CRASH')=='1':
    observation=json.loads((r/'crash-boundary.observed').read_text());binding=binding_for_thread(registry['bindings'],observation['thread_id'])
    assert confirm_restarted and binding['document']['server_incarnation']==observation['server_incarnation']
    assert len([o for o in registry['operations'] if o['binding']==binding['id'] and o['kind']=='thread/start'])==1
    print('actual_empty_thread_binding_recovered',topology,json.dumps(observation),flush=True)
   if os.environ.get('CORENIUS_TEST_EVENT_FIXTURE')=='1':
    (r/'event-barrier').write_text('declared pause before A thread/read consumes actual B events')
    wait(lambda:(r/'event-barrier.observed').exists(),8);event_b_release.set()
    b_workspace=r/'workers'/grants[1]['attempt']/'workspace'
    wait(lambda:(b_workspace/'TASK_B-shell').exists() and (b_workspace/'TASK_B-patch-2').exists(),10)
    time.sleep(.2);(r/'event-barrier').unlink();wait(lambda:(r/'event-barrier.complete').exists(),10)
    projected=query('codex_registry');b=next(row for row in projected['bindings'] if row['attempt']==grants[1]['attempt']);a=next(row for row in projected['bindings'] if row['attempt']==grants[0]['attempt'])
    frames_a=json.loads((r/'event-barrier.native-A').read_text());frames_b=json.loads((r/'event-barrier.native-B').read_text());frames=frames_a if topology=='shared' else frames_b
    actual_b=[event for event in frames if event.get('method')=='item/completed' and event.get('params',{}).get('threadId')==b['document']['thread_id']]
    assert actual_b,'actual B native notification must be received before testing projection loss'
    print('actual_received_interleaved_events',topology,json.dumps(actual_b),flush=True)
    print('actual_interleaved_thread_events',topology,json.dumps({'A':a['document']['native_activity'],'B':b['document']['native_activity']}),flush=True)
    assert all(event in b['document']['native_activity'] for event in actual_b),'B actual completion notification must survive A draining shared connection'
    assert all(event.get('params',{}).get('threadId',event.get('params',{}).get('thread',{}).get('id'))==b['document']['thread_id'] for event in b['document']['native_activity'])
    (r/'hold-codex').unlink();release.set()
    for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
    wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_STORE_RESTART_FIXTURE'):
    mode=os.environ['CORENIUS_TEST_STORE_RESTART_FIXTURE'];release.set()
    if mode=='cancelled':
     for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
     wait(lambda:query('snapshot')['occupied_slots']==0)
     wait(lambda:all(b['state']=='revoked' for b in query('codex_registry')['bindings']))
    retained=query('codex_registry');controller.kill();controller.wait(timeout=5)
    for process in owned.processes:
     if process.poll() is None:process.kill();process.wait(timeout=5)
    for fd,execution in owned.namespace_fds.values():
     poll=select.poll();poll.register(fd,select.POLLIN)
     if not poll.poll(100):signal.pidfd_send_signal(fd,signal.SIGKILL);assert poll.poll(5000)
    for root,secret in [('workers',SUPERVISOR),('controlplanes',CONTROL)]:
     process=owned.spawn([str(supervisor),str(r/(root+'.json')),str(r/(root+'.sock'))],root+'-restarted',{'CORENIUS_SUPERVISOR_TOKEN':secret,'PATH':'/usr/bin:/bin'})
     def reopened():
      assert process.poll() is None,(r/(root+'-restarted.stderr')).read_text()
      try:value=request(r/(root+'.sock'),{'token':secret,'action':'lookup','binding':retained['bindings'][0]['document']['executor_execution']['binding'] if root=='workers' else retained['servers'][0]['document']['effect_binding']})
      except (ConnectionRefusedError,FileNotFoundError,ConnectionResetError):return False
      return value if 'ok' in value else False
     response=wait(reopened,30);print('actual_native_supervisor_reopened',topology,mode,root,json.dumps(response),flush=True)
     if root=='workers':
      expected='exited' if mode=='cancelled' else 'unknown';assert response['ok']['state']==expected
      if mode=='live':
       repeated=request(r/'workers.sock',{'token':SUPERVISOR,'action':'start','binding':response['ok']['binding']})
       assert repeated['ok']['state']=='unknown' and repeated['ok']['handle']==response['ok']['handle']
    if mode=='live':
     controller=owned.spawn([sys.argv[1],str(r/'state.db'),str(r/'controller.sock')],'controller-restarted-unknown',env)
     def held_unknown_capacity():
      assert controller.poll() is None,(r/'controller-restarted-unknown.stderr').read_text()
      try:snapshot=query('snapshot')
      except (ConnectionRefusedError,ConnectionResetError,FileNotFoundError):return False
      return snapshot['occupied_slots']==2 and all(a['state']=='unknown' for a in snapshot['attempts'])
     wait(held_unknown_capacity,20)
     rejected=request(r/'controller.sock',{'auth':auth,'project':'p','command':{'kind':'codex_turn','args':{'attempt':grants[0]['attempt'],'text':'UNKNOWN_OWNER_INPUT'},'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}})
     assert 'error' in rejected,'unknown owner must not dispatch a native turn'
    if mode=='cancelled':
     for process in owned.processes:
      if process.poll() is None:process.kill();process.wait(timeout=5)
     receipt=r/'workers'/('native-store-'+retained['bindings'][0]['document']['executor_execution']['binding']['operation']+'.json');original=receipt.read_bytes()
     corrupt=json.loads(original);corrupt['generation']=str(__import__('uuid').uuid4());receipt.write_text(json.dumps(corrupt))
     bad=owned.spawn([str(supervisor),str(r/'workers.json'),str(r/'workers.sock')],'workers-invalid-receipt',{'CORENIUS_SUPERVISOR_TOKEN':SUPERVISOR,'PATH':'/usr/bin:/bin'})
     wait(lambda:bad.poll() is not None,30);assert bad.returncode!=0 and 'native store receipt execution/generation mismatch' in (r/'workers-invalid-receipt.stderr').read_text()
     receipt.write_bytes(original);saved=r/'receipt-saved';receipt.rename(saved);receipt.symlink_to(saved)
     linked=owned.spawn([str(supervisor),str(r/'workers.json'),str(r/'workers.sock')],'workers-linked-receipt',{'CORENIUS_SUPERVISOR_TOKEN':SUPERVISOR,'PATH':'/usr/bin:/bin'})
     wait(lambda:linked.poll() is not None,30);assert linked.returncode!=0 and 'record symlink refused' in (r/'workers-linked-receipt.stderr').read_text()
     receipt.unlink();saved.rename(receipt)
     print('actual_invalid_native_receipt_refused',topology,flush=True)
    return
   first=registry['bindings'][0];before_ops=len([o for o in registry['operations'] if o['kind'].startswith('codex_')])
   for forbidden in [{'thread_id':'foreign'},{'server_id':'foreign'},{'history':[]},{'path':'/host'},{'environment_id':'local'},{'workspace':'/host'}]:
    value=request(r/'controller.sock',{'auth':auth,'project':'p','command':{'kind':'codex_turn','args':dict({'attempt':first['attempt'],'text':'forged'},**forbidden),'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}})
    assert 'error' in value,value
   for binding_guard in [{'thread_id':'foreign'},{'server_store':'foreign'},{'server_incarnation':'foreign'},{'environment_id':'local'}]:
    value=request(r/'controller.sock',{'auth':auth,'project':'p','command':{'kind':'codex_turn','args':{'attempt':first['attempt'],'text':'forged','expected_binding':binding_guard},'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}})
    assert 'error' in value,value
   worker_auth={'principal':grants[0]['principal'],'token':grants[0]['token']}
   value=request(r/'controller.sock',{'auth':worker_auth,'project':'p','command':{'kind':'codex_turn','args':{'attempt':first['attempt'],'text':'forged'},'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}})
   assert 'error' in value,value
   assert len([o for o in query('codex_registry')['operations'] if o['kind'].startswith('codex_')])==before_ops
   valid=command('codex_steer',{'attempt':first['attempt'],'text':'TASK_A MODEL_FREE_CURRENT_TURN_STEER','expected_turn':first['document']['turn_id']})
   wait(lambda:any(o['id']==valid['operation'] and o['state']=='confirmed' for o in query('codex_registry')['operations']))
   print('native_current_authority_and_selector_guards',topology,flush=True)
   if os.environ.get('CORENIUS_TEST_DISCONNECT_FIXTURE')=='1':
    (r/'hold-codex').write_text('declared test-only controller adaptation hold')
    executor=registry['bindings'][0]['document']['executor'];descriptor=os.pidfd_open(executor['runtime_pid'])
    try:
     stat=pathlib.Path('/proc/'+str(executor['runtime_pid'])+'/stat').read_text().rsplit(') ',1)[1].split()
     assert stat[19]==executor['runtime_start'];signal.pidfd_send_signal(descriptor,signal.SIGKILL)
    finally:os.close(descriptor)
    def child_restarted():
     path=r/'workers'/registry['bindings'][0]['attempt']/'scratch/native-runtime.json'
     return json.loads(path.read_text())['start_ticks']!=executor['runtime_start']
    wait(child_restarted)
    release.set();time.sleep(2)
    app_doc=next(s['document'] for s in registry['servers'] if s['id']==registry['bindings'][0]['document']['server_id']);runtime=app_doc['runtime']
    cap=(r/'controlplanes/native-authority'/app_doc['effect_binding']['attempt']).read_text()
    observer=WS.connect('127.0.0.1',8788,token=cap,unix=runtime['endpoint'],path='/?corenius_start='+runtime['runtime_start'])
    def observe(method,params,identifier):
     observer.send({'id':identifier,'method':method,'params':params})
     while True:
      response=observer.recv()
      if response.get('id')==identifier:return response
    observe('initialize',{'clientInfo':{'name':'corenius_readonly_fixture_observer','version':'0.1'},'capabilities':{'experimentalApi':True}},'observer-init')
    status=observe('environment/status',{'environmentId':registry['bindings'][0]['document']['environment_id']},'observer-status');assert status['result']['status']=='disconnected',status
    observation=observe('thread/read',{'threadId':registry['bindings'][0]['document']['thread_id'],'includeTurns':True},'observer-thread')
    assert observation['result']['thread']['environments'][0]['environmentId']==registry['bindings'][0]['document']['environment_id'],observation
    observer.close();print('native_disconnected_status',topology,json.dumps(status),flush=True)
    workspace=r/'workers'/registry['bindings'][0]['attempt']/'workspace'
    assert not (workspace/'TASK_A-shell').exists() and not any(workspace.glob('TASK_A-patch-*'))
    assert not any((p/'TASK_A-shell').exists() or list(p.glob('TASK_A-patch-*')) for p in (r/'controlplanes').glob('*/workspace'))
    print('native_disconnected_remote_no_local_fallback',topology,flush=True)
    (r/'hold-codex').unlink()
    wait(lambda:next(b['document'] for b in query('codex_registry')['bindings'] if b['attempt']==registry['bindings'][0]['attempt'])['executor']['incarnation']!=executor['incarnation'])
    stopped=command('codex_interrupt',{'attempt':registry['bindings'][0]['attempt'],'expected_turn':registry['bindings'][0]['document']['turn_id']})
    wait(lambda:any(o['id']==stopped['operation'] and o['state']=='confirmed' for o in query('codex_registry')['operations']))
    with counter_lock:counts['TASK_A']=3
    restored=command('codex_turn',{'attempt':registry['bindings'][0]['attempt'],'text':'TASK_A MODEL_FREE_EXECUTOR_RECOVERY'})
    wait(lambda:any(o['id']==restored['operation'] and o['state']=='confirmed' for o in query('codex_registry')['operations']))
    wait(lambda:(workspace/'TASK_A-shell-4').exists() and (workspace/'TASK_A-patch-5').exists())
    print('native_owned_executor_rebound',topology,flush=True)
    for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
    wait(lambda:query('snapshot')['occupied_slots']==0)
    return
   release.set()
   for grant,task in zip(grants,['TASK_A','TASK_B']):
    workspace=r/'workers'/grant['attempt']/'workspace'
    wait(lambda:(workspace/(task+'-shell')).exists() and any(workspace.glob(task+'-patch-*')),20)
    assert not any((workspace/(peer+'-shell')).exists() for peer in ['TASK_A','TASK_B'] if peer!=task)
    print('actual_native_effects',topology,task,(workspace/(task+'-probe')).read_text(),flush=True)
   if os.environ.get('CORENIUS_TEST_CORRECTION_AUTHORITY')=='1':
    lead=command('replace_lead',{})
    wait(lambda:any(b['role']=='lead' and b['document'].get('turn_id') for b in query('codex_registry')['bindings']))
    import correction_authority
    correction_authority.exercise(r,query,command,request,grants,lead,wait,topology,TOKEN)
    return
   # Controller connection restart must reconnect subscriptions without new threads/turns.
   durable=query('codex_registry');controller.kill();controller.wait(timeout=5)
   restarted_env=dict(env)
   if os.environ.get('CORENIUS_TEST_RETARGET_FIXTURE')=='1':
    changed=json.loads(runtime_config.read_text());changed['topology']='separate' if topology=='shared' else 'shared'
    changed_config=r/'retarget.json';changed_config.write_text(json.dumps(changed));restarted_env['CORENIUS_CODEX_CONFIG']=str(changed_config)
    if os.environ.get('CORENIUS_TEST_RETARGET_KIND')=='missing':restarted_env.pop('CORENIUS_CODEX_CONFIG')
   controller=owned.spawn([sys.argv[1],str(r/'state.db'),str(r/'controller.sock')],'controller-restarted',restarted_env)
   if os.environ.get('CORENIUS_TEST_RETARGET_FIXTURE')=='1':
    deadline=time.monotonic()+3
    while time.monotonic()<deadline and controller.poll() is None:time.sleep(.03)
    assert controller.poll() is not None and controller.returncode!=0,'durable native topology/config identity must refuse retargeted startup'
    print('durable_native_retarget_refused',topology,flush=True)
    return
   def reconnect():
    assert controller.poll() is None,(r/'controller-restarted.stderr').read_text()
    try:return request(r/'controller.sock',{'auth':auth,'project':'p','query':{'kind':'codex_registry','args':{}}}).get('ok')
    except (ConnectionRefusedError,FileNotFoundError):return False
   wait(reconnect)
   after=query('codex_registry')
   assert [b['document']['thread_id'] for b in after['bindings']]==[b['document']['thread_id'] for b in durable['bindings']]
   assert query('snapshot')['occupied_slots']==2
   print('controller_native_connection_reconciled',topology,flush=True)
   # Only owned fixture processes are killed; the controller remains the dispatcher.
   before=query('codex_registry'); old=before['bindings'][0]['document']; app=next(s for s in before['servers'] if s['id']==old['server_id'])['document']['runtime']
   native_pid=app['runtime_pid']; descriptor=os.pidfd_open(native_pid)
   try:
    stat=pathlib.Path('/proc/'+str(native_pid)+'/stat').read_text().rsplit(') ',1)[1].split()
    assert stat[19]==app['runtime_start']
    signal.pidfd_send_signal(descriptor,signal.SIGKILL)
   finally:os.close(descriptor)
   def resumed():
    row=next(b for b in query('codex_registry')['bindings'] if b['document']['thread_id']==old['thread_id'])
    d=row['document']
    return d if d.get('server_incarnation')!=old['server_incarnation'] and d.get('recovery_state')=='remote_registered_input_blocked' else False
   recovered=wait(resumed,20)
   assert recovered['server_store']==old['server_store'] and recovered['executor_execution']['handle']==old['executor_execution']['handle'] and not recovered['ui_input_ready']
   new=command('codex_turn',{'attempt':before['bindings'][0]['attempt'],'text':'TASK_A MODEL_FREE_RESTART_RESTORATION','expected_binding':{'thread_id':old['thread_id'],'server_store':old['server_store']}})
   wait(lambda:any(o['id']==new['operation'] and o['state']=='confirmed' for o in query('codex_registry')['operations']))
   restored=wait(lambda:next((b['document'] for b in query('codex_registry')['bindings'] if b['document']['thread_id']==old['thread_id'] and b['document']['ui_input_ready'] and b['document']['turn_id']!=old['turn_id']),False))
   recovered_workspace=r/'workers'/before['bindings'][0]['attempt']/'workspace'
   wait(lambda:(recovered_workspace/'TASK_A-shell-4').exists() and (recovered_workspace/'TASK_A-patch-5').exists(),20)
   print('native_restart_restored',topology,json.dumps({'thread_id':restored['thread_id'],'server_store':restored['server_store'],'server_incarnation_changed':True,'remote_restored':True}),flush=True)
   if os.environ.get('CORENIUS_TEST_ADOPT_CRASH'):
    old_turn=restored['turn_id'];(r/'crash-boundary').write_text('declared crash after actual native turn receipt')
    if os.environ['CORENIUS_TEST_ADOPT_CRASH']=='history_confirmation':(r/'lose-ack').write_text('declared actual native acknowledgement loss')
    new_turn=command('codex_turn',{'attempt':before['bindings'][0]['attempt'],'text':'TASK_A MODEL_FREE_ADOPTION_CRASH MODEL_FREE_ACTIVE_CANCEL'})
    wait(lambda:controller.poll()==79,20);observation=json.loads((r/'crash-boundary.observed').read_text());assert observation['operation']==new_turn['operation'] and observation['turn_id']!=old_turn
    print('actual_turn_crash_receipt',topology,os.environ['CORENIUS_TEST_ADOPT_CRASH'],json.dumps({'observation':observation,'old_turn':old_turn}),flush=True)
    with sqlite3.connect((r/'state.db').as_uri()+'?mode=ro',uri=True) as observer:
     durable=json.loads(observer.execute('SELECT document FROM codex_bindings WHERE id=?',(before['bindings'][0]['id'],)).fetchone()[0])
    print('actual_durable_turn_projection_at_crash',topology,json.dumps({'receipt':observation,'turn_id':durable.get('turn_id'),'current_turn':durable.get('current_turn')}),flush=True)
    assert durable['turn_id']==observation['turn_id'] and durable.get('current_turn',{}).get('id')==observation['turn_id'],'actual receipt and typed current turn must be atomically durable at crash'
    controller=owned.spawn([sys.argv[1],str(r/'state.db'),str(r/'controller.sock')],'controller-after-turn-crash',env)
    def adopted():
     try:projection=query('codex_registry')
     except (ConnectionRefusedError,FileNotFoundError,ConnectionResetError):return False
     row=next(b for b in projection['bindings'] if b['id']==before['bindings'][0]['id'])
     return row if row['document'].get('turn_id')==observation['turn_id'] else False
    actual=wait(adopted,25);projection=query('codex_registry');op=next(o for o in projection['operations'] if o['id']==new_turn['operation']);assert op['state']=='confirmed'
    wait(lambda:any(turn['id']==observation['turn_id'] and any(item.get('clientId')==new_turn['operation'] for item in turn['items']) for turn in next(b for b in query('codex_registry')['bindings'] if b['id']==actual['id'])['document']['native_turns']))
    history=next(b for b in query('codex_registry')['bindings'] if b['id']==actual['id'])['document']['native_turns']
    assert sum(any(item.get('clientId')==new_turn['operation'] for item in turn['items']) for turn in history)==1
    valid=command('codex_interrupt',{'attempt':before['bindings'][0]['attempt'],'expected_turn':observation['turn_id']})
    wait(lambda:any(o['id']==valid['operation'] and o['state']=='confirmed' for o in query('codex_registry')['operations']))
    print('actual_latest_turn_adopted_after_crash',topology,os.environ['CORENIUS_TEST_ADOPT_CRASH'],json.dumps(observation),flush=True)
    for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
    wait(lambda:query('snapshot')['occupied_slots']==0);return
   if os.environ.get('CORENIUS_TEST_ACK_FIXTURE')=='1':
    (r/'lose-ack').write_text('declared test-only acknowledgement loss')
    lost=command('codex_turn',{'attempt':before['bindings'][0]['attempt'],'text':'TASK_A MODEL_FREE_LOST_ACK'})
    wait(lambda:(r/'lose-ack.observed').exists())
    if os.environ.get('CORENIUS_TEST_ACK_FENCE_FIXTURE')=='1':
     wait(lambda:any(o['id']==lost['operation'] and o['state']=='unknown' for o in query('codex_registry')['operations']))
     value=request(r/'controller.sock',{'auth':auth,'project':'p','command':{'kind':'codex_turn','args':{'attempt':before['bindings'][0]['attempt'],'text':'TASK_A FORBIDDEN_WHILE_TURN_ACK_UNKNOWN'},'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}})
     assert 'error' in value,'unknown native start must fence new turns before reconciliation'
     (r/'hold-codex').unlink();print('native_unknown_start_fences_new_turn',topology,flush=True)
    history_printed=False
    def acknowledged():
     nonlocal history_printed
     projection=query('codex_registry');operation=next(o for o in projection['operations'] if o['id']==lost['operation'])
     if operation['state']=='unknown' and not history_printed:
      print('actual_unknown_turn_history',topology,json.dumps(projection['bindings'][0]['document'].get('native_turns')),flush=True);history_printed=True
     return operation if operation['state']=='confirmed' and operation['observation'].get('reconciled_from_native_history') else False
    reconciled=wait(acknowledged,20)
    assert reconciled['observation']['turn_id']!=restored['turn_id']
    print('native_lost_ack_reconciled',topology,json.dumps(reconciled),flush=True)
   readiness={};resumer=None
   if os.environ.get('CORENIUS_TEST_EXEC_READY_FIXTURE')=='1':
    (r/'pause-exec-ready').write_text('declared identity-qualified native executor pause before first authenticated readiness')
    def release_executor():
     observation=wait(lambda:json.loads((r/'pause-exec-ready.observed').read_text()) if (r/'pause-exec-ready.observed').exists() else False,15);pid=observation['runtime_pid'];descriptor=os.pidfd_open(pid)
     try:
      stat=pathlib.Path('/proc/'+str(pid)+'/stat').read_text().rsplit(') ',1)[1].split();namespace=int(os.readlink('/proc/'+str(pid)+'/ns/pid').split('[')[1][:-1]);poll=select.poll();poll.register(descriptor,select.POLLIN)
      assert stat[19]==observation['runtime_start'] and namespace==observation['pid_namespace'] and stat[0]=='T' and not poll.poll(0)
      stopped_at=time.monotonic();print('actual_native_exec_stopped_before_ready',topology,json.dumps(observation),flush=True)
      time.sleep(2.3)
      projection=query('codex_registry');binding=next(b for b in projection['bindings'] if b['document'].get('executor_effect_binding',{}).get('attempt')==observation['attempt']);readiness.update(observation=observation,binding=binding['id'],registrations_while_stopped=[o for o in projection['operations'] if o['binding']==binding['id'] and o['kind']=='environment/add'])
      print('actual_registration_while_exec_stopped',topology,json.dumps(readiness),flush=True)
      while time.monotonic()-stopped_at<3:time.sleep(.02)
     finally:
      stat=pathlib.Path('/proc/'+str(pid)+'/stat').read_text().rsplit(') ',1)[1].split();assert stat[19]==observation['runtime_start'] and not poll.poll(0);signal.pidfd_send_signal(descriptor,signal.SIGCONT);os.close(descriptor)
     readiness['actual_pause_seconds']=time.monotonic()-stopped_at
    resumer=threading.Thread(target=release_executor,daemon=True);resumer.start()
   lead=command('replace_lead',{})
   if resumer is not None:
    resumer.join(15);assert not resumer.is_alive() and 'actual_pause_seconds' in readiness,'owned native pause/resume must complete'
    assert readiness['registrations_while_stopped']==[],'unready actual executor must not create a native environment registration'
   wait(lambda:any(b['role']=='lead' and b['document'].get('thread_id') for b in query('codex_registry')['bindings']))
   assert query('snapshot')['occupied_slots']==2
   print('native_lead_no_worker_slot',topology,flush=True)
   if resumer is not None:
    projection=query('codex_registry');adds=[o for o in projection['operations'] if o['binding']==readiness['binding'] and o['kind']=='environment/add'];assert len(adds)==1 and adds[0]['state']=='confirmed'
    print('actual_exec_ready_recovered_one_registration',topology,json.dumps({'readiness':readiness,'registration':adds[0]}),flush=True)

   lead_rows=wait(lambda:[b for b in query('codex_registry')['bindings'] if b['role']=='lead' and b['document'].get('turn_id')] or False)
   if os.environ.get('CORENIUS_TEST_CONTROL_CENTER')=='1':
    if os.environ.get('CORENIUS_TEST_REPAIR3_UI')=='1':
     import repair3_native_ui
     repair3_native_ui.exercise(r,query,command,request,grants,lead,wait,topology,TOKEN)
    elif os.environ.get('CORENIUS_TEST_REPAIR2_UI')=='1':
     import repair2_native_ui
     repair2_native_ui.exercise(r,query,command,request,grants,lead,wait,topology,TOKEN)
    else:
     import control_center_native
     control_center_native.exercise(r,query,command,request,grants,lead,wait,topology,TOKEN)
   old_lead=lead_rows[0];command('replace_lead',{})
   wait(lambda:any(b['id']==old_lead['id'] and b['state']=='revoked' and b['document']['executor_execution']['state']=='exited' and not b['document']['ui_input_ready'] for b in query('codex_registry')['bindings']))
   if os.environ.get('CORENIUS_TEST_REPAIR3_UI')=='1':
    saved=json.loads((r/'repair3-ui-request.json').read_text());assert 'error' in request(r/'controller.sock',saved)
   if os.environ.get('CORENIUS_TEST_REPAIR2_UI')=='1':
    saved=json.loads((r/'repair2-native-ui-exact-retries.json').read_text());old_binding=saved['binding']
    def old_identity_counts():
     with sqlite3.connect((r/'state.db').as_uri()+'?mode=ro',uri=True) as db:return tuple(db.execute('SELECT (SELECT COUNT(*) FROM codex_operations WHERE binding=?),(SELECT COUNT(*) FROM control_ui_commands WHERE binding=?)',(old_binding,old_binding)).fetchone())
    before=old_identity_counts();negatives=[]
    for method,key in [('turn/start','lost'),('turn/steer','steer')]:
     negative=request(r/'controller.sock',{'auth':saved['auth'],'project':'p','native_ui':{'method':method,'params':saved[key]}});assert 'error' in negative,negative;negatives.append(negative)
    after=old_identity_counts();assert before==after,(before,after)
    print('actual_repair2_native_ui_revoked_exact_retries',topology,json.dumps({'responses':negatives,'old_identity_counts_before':before,'old_identity_counts_after':after,'no_effect':True}),flush=True)
    cursor=0;lead_history=[]
    while True:
     page=request(r/'controller.sock',{'auth':auth,'project':'p','query':{'kind':'control_records','args':{'collection':'lead_public_history','binding':old_binding,'after':cursor,'limit':20}}})
     print('actual_repair2_replaced_lead_public_history_page',topology,json.dumps({'ok':bool(page.get('ok')),'error':page.get('error'),'binding':old_binding,'native_retained_turns':len(next(x for x in query('codex_registry')['bindings'] if x['id']==old_binding)['document'].get('native_turns',[]))}),flush=True)
     assert 'ok' in page,page
     lead_history.extend(page['ok']['items'])
     if not page['ok']['page']['has_more']:break
     assert page['ok']['page']['after']>cursor;cursor=page['ok']['page']['after']
    assert lead_history and all(x['binding']==old_binding for x in lead_history)
    reference=next(x['input_reference'] for x in lead_history if x['kind']=='public_message')
    full=request(r/'controller.sock',{'auth':auth,'project':'p','query':reference});assert 'ok' in full and full['ok']['digest']==reference['args']['sha256'] and full['ok']['input']['type'] in ['userMessage','agentMessage'],full
    replacement=query('snapshot')['project']['lead_epoch']
    with sqlite3.connect((r/'state.db').as_uri()+'?mode=ro',uri=True) as db:
     reply=json.loads(db.execute("SELECT response FROM receipts WHERE project='p' AND json_extract(response,'$.result.role_epoch')=?",(replacement,)).fetchone()[0])['result']
    replacement_auth={k:reply[k] for k in ['principal','token']};assert 'ok' in request(r/'controller.sock',{'auth':replacement_auth,'project':'p','query':reference})
    for actor in [{'principal':grants[0]['principal'],'token':grants[0]['token']},saved['auth']]:assert 'error' in request(r/'controller.sock',{'auth':actor,'project':'p','query':reference})
    observer=command('create_monitor_observer',{'name':'native public input observer'});observer_auth={k:observer[k] for k in ['principal','token']}
    assert 'error' in request(r/'controller.sock',{'auth':observer_auth,'project':'p','query':reference})
    print('actual_repair2_replaced_lead_public_history_input',topology,json.dumps({'items':len(lead_history),'replacement_read':True,'worker_observer_revoked_refused':True,'private_reasoning_excluded':True,'no_analyzer_credit':True}),flush=True)

   value=request(r/'controller.sock',{'auth':{'principal':lead['principal'],'token':lead['token']},'project':'p','command':{'kind':'codex_turn','args':{'role':'lead','text':'REVOKED_LEAD_INPUT'},'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}})
   assert 'error' in value,'revoked lead must not submit native input'
   assert query('snapshot')['occupied_slots']==2
   print('native_replaced_lead_authority_refused',topology,flush=True)
   active=command('codex_turn',{'attempt':grants[0]['attempt'],'text':'TASK_A MODEL_FREE_ACTIVE_CANCEL'})
   wait(lambda:any(o['id']==active['operation'] and o['state']=='confirmed' for o in query('codex_registry')['operations']))
   wait(lambda:any(b['attempt']==grants[0]['attempt'] and b['document'].get('native_turns') and b['document']['native_turns'][-1]['status']=='inProgress' for b in query('codex_registry')['bindings']))
   for grant in grants:command('cancel_attempt',{'attempt':grant['attempt']})
   wait(lambda:query('snapshot')['occupied_slots']==0)
   retired=query('codex_registry')
   wait(lambda:all(b['state']=='revoked' and not b['document']['ui_input_ready'] for b in query('codex_registry')['bindings'] if b['role']=='attempt'))
   disposition=next(b['document'] for b in query('codex_registry')['bindings'] if b['attempt']==grants[0]['attempt']).get('turn_shutdown')
   assert disposition and disposition['state'] in ['completed','interrupted','failed','unknown','interrupt_acknowledged'], 'retained native turn shutdown must distinguish receipt from executor stop'
   print('native_active_cancellation_disposition',topology,json.dumps(disposition),flush=True)
   value=request(r/'controller.sock',{'auth':auth,'project':'p','command':{'kind':'codex_turn','args':{'attempt':grants[0]['attempt'],'text':'REVOKED_ATTEMPT_INPUT'},'expected_revision':query('snapshot')['revision'],'idempotency_key':str(__import__('uuid').uuid4())}})
   assert 'error' in value,'withdrawn attempt binding must not submit native input'
   print('controller_owned_cancellation_confirmed',topology,flush=True)
  finally:
   release.set();event_b_release.set();cancel_hold.set();owned.stop();provider.shutdown();provider.server_close();guard.close()
   if os.environ.get('CORENIUS_TEST_RETAIN_RAW')=='1' or os.environ.get('CORENIUS_TEST_EMPTY_RESTART')=='1' or os.environ.get('CORENIUS_TEST_ADOPT_EVENT_FIXTURE')=='1' or os.environ.get('CORENIUS_TEST_PRE_UNSUBSCRIBE_FIXTURE')=='1' or os.environ.get('CORENIUS_TEST_CONTROL_CENTER')=='1' or os.environ.get('CORENIUS_TEST_INITIAL_CONFIRM')=='1':
    import tarfile
    destination=HERE.parents[1]/'.scratch/sqlite-controller/native';destination.mkdir(parents=True,exist_ok=True)
    with tarfile.open(destination/(topology+'-'+r.name+'.tar.gz'),'w:gz') as archive:archive.add(r,arcname=r.name)
   for log in list(r.glob('*.stderr'))+list(r.glob('*/**/output.log')):
    if log.stat().st_size:print('owned_log',log.name,log.read_text()[-3000:],flush=True)
for topology in os.environ.get('CORENIUS_TEST_TOPOLOGY','shared,separate').split(','):run(topology)

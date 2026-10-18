"""MODEL_FREE actual public SQLite/Git/supervisor/bwrap contract qualification.
All resources are unique local fixtures. No source/mainline/model/session effects.
"""
import concurrent.futures, hashlib, json, os, pathlib, select, shutil, signal, socket, sqlite3, subprocess, sys, sysconfig, tarfile, tempfile, time, uuid

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)
BIN=pathlib.Path(sys.argv[1]).resolve(); HERE=pathlib.Path(__file__).resolve().parents[2]; REPO=_temp_policy_file.parents[1]
TOKEN='operator-git-secret'; SUPERVISOR='git-supervisor-secret'
def pin(path):
 p=pathlib.Path(path);return {'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def owned_tmp():
 p=pathlib.Path(os.environ.get('TMPDIR') or '/run/corenius-tmp');p.mkdir(parents=True,exist_ok=True);return str(p)
def copy_into_runtime(source,root):
 source=pathlib.Path(source)
 source_bytes=source.resolve() if source.is_symlink() else source
 text=str(source)
 if text.startswith('/usr/'):rel=pathlib.Path(text).relative_to('/usr')
 elif text.startswith('/lib64/'):rel=pathlib.Path('lib64')/pathlib.Path(text).relative_to('/lib64')
 elif text.startswith('/lib/'):rel=pathlib.Path('lib')/pathlib.Path(text).relative_to('/lib')
 else:raise AssertionError('runtime source outside supported toolchain roots: '+text)
 dest=root/rel;dest.parent.mkdir(parents=True,exist_ok=True)
 if dest.exists() and hashlib.sha256(dest.read_bytes()).hexdigest()==hashlib.sha256(source_bytes.read_bytes()).hexdigest():return
 shutil.copy2(source_bytes,dest)
def ldd_paths(paths):
 found=set()
 for path in paths:
  proc=subprocess.run(['/usr/bin/ldd',str(path)],capture_output=True,text=True,check=True)
  for line in proc.stdout.splitlines():
   parts=line.strip().split()
   candidates=[]
   if '=>' in parts:
    idx=parts.index('=>')
    if idx+1<len(parts):candidates.append(parts[idx+1])
   elif parts and parts[0].startswith('/'):
    candidates.append(parts[0])
   for candidate in candidates:
    if candidate.startswith(('/lib/','/lib64/','/usr/lib/')) and pathlib.Path(candidate).is_file():
     found.add(candidate)
 return sorted(found)
def write_runtime_manifest(root,manifest):
 entries=[]
 for path in sorted(root.rglob('*')):
  rel=path.relative_to(root).as_posix()
  st=path.lstat()
  if path.is_symlink():
   target=os.readlink(path)
   assert not pathlib.PurePosixPath(target).is_absolute()
   normalized=pathlib.PurePosixPath(rel).parent.joinpath(target)
   assert '..' not in normalized.parts
   entries.append({'path':rel,'kind':'symlink','target':target})
  elif path.is_file():
   entries.append({'path':rel,'kind':'regular','sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':st.st_size})
 doc={'schema_version':'corenius.verifier-runtime.v1','entries':entries}
 manifest.write_text(json.dumps(doc,sort_keys=True,separators=(',',':'))+'\n')
 return {'root':str(root),'manifest':pin(manifest),'control':str(root/'share/corenius-runtime-control/value.txt')}
def build_verifier_runtime(root):
 runtime=root/'runtime-snapshot/usr';runtime.mkdir(parents=True)
 python=pathlib.Path('/usr/bin/python3')
 copy_into_runtime(python,runtime)
 stdlib=pathlib.Path(sysconfig.get_path('stdlib'))
 shutil.copytree(stdlib,runtime/stdlib.relative_to('/usr'),dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__','*.pyc'))
 dynload=pathlib.Path(sysconfig.get_config_var('DESTSHARED') or stdlib/'lib-dynload')
 if dynload.exists():
  shutil.copytree(dynload,runtime/dynload.relative_to('/usr'),dirs_exist_ok=True,ignore=shutil.ignore_patterns('__pycache__','*.pyc'))
 so_paths=[python]
 so_paths.extend(path for path in dynload.glob('*.so') if path.is_file())
 for dep in ldd_paths(so_paths):copy_into_runtime(dep,runtime)
 control_source=root/'ambient-runtime-source/value.txt';control_source.parent.mkdir(parents=True);control_source.write_text('snapshotted\n')
 control=runtime/'share/corenius-runtime-control/value.txt';control.parent.mkdir(parents=True);shutil.copy2(control_source,control)
 (root/'ambient-runtime-source/ambient_only.py').write_text('raise AssertionError("ambient source imported")\n')
 control_source.write_text('ambient changed after snapshot\n')
 return write_runtime_manifest(runtime,root/'runtime-manifest.json')
def wait(f,seconds=25):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  value=f()
  if value:return value
  time.sleep(.02)
 raise AssertionError('public behavior deadline')
def request(path,value):
 with socket.socket(socket.AF_UNIX) as s:
  s.settimeout(40);s.connect(str(path));s.sendall(json.dumps(value).encode()+b'\n');return json.loads(s.makefile('rb').readline())
def git(path,*args):
 p=subprocess.run(['/usr/bin/git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-C',str(path),*args],env={'PATH':'/usr/bin:/bin','GIT_CONFIG_NOSYSTEM':'1','GIT_CONFIG_GLOBAL':'/dev/null','GIT_NO_REPLACE_OBJECTS':'1'},capture_output=True,text=True)
 assert p.returncode==0,(args,p.stderr);return p.stdout.strip()
class Fixture:
 def __init__(self,mode):
  self.temp=tempfile.TemporaryDirectory(prefix='g4-',dir=owned_tmp());self.r=pathlib.Path(self.temp.name);self.mode=mode;self.processes=[];self.records=[];self.namespaces={};self.runtime=None
  try:self.setup()
  except BaseException:
   self.close();raise
 def setup(self):
  for d in ['authority','workers','canonical','seed']:(self.r/d).mkdir()
  git(self.r/'canonical','init','--bare','--template=')
  git(self.r/'seed','init','--template=')
  (self.r/'seed/README').write_text('bounded fixture base\n')
  git(self.r/'seed','add','README');git(self.r/'seed','-c','user.name=Fixture','-c','user.email=fixture@invalid','commit','-m','B')
  self.base=git(self.r/'seed','rev-parse','HEAD');git(self.r/'seed','push',str(self.r/'canonical'),self.base+':refs/heads/protected')
  (self.r/'secret').write_text('outside effect must remain unchanged')
  self.suite=self.r/'verification.py';self.suite.write_text('''import importlib.util,pathlib,sys,os
root=pathlib.Path('/workspace');mode=sys.argv[1];host=sys.argv[2]
assert not pathlib.Path(host).exists(), 'verifier must not see controller root'
assert not pathlib.Path('/root/.ssh').exists()
assert 'CapEff:\\t0000000000000000' in pathlib.Path('/proc/self/status').read_text()
if len(sys.argv)>3:
 control=pathlib.Path('/usr/share/corenius-runtime-control/value.txt')
 assert control.read_text()=='snapshotted\\n', 'verifier consumed ambient runtime instead of snapshot'
 try:
  control.write_text('verifier wrote runtime')
  raise AssertionError('verifier runtime snapshot is writable')
 except OSError:
  pass
 try:
  __import__('ambient_only')
  raise AssertionError('omitted ambient module loaded')
 except ModuleNotFoundError:
  pass
for name in (['a','b'] if mode=='combined' else [mode]):
 spec=importlib.util.spec_from_file_location('task_'+name,root/('task_'+name+'.py'));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
 assert module.answer()==('A' if name=='a' else 'B'), 'fixed admitted task contract failed'
print('FIXED_CONTRACT_PASS',mode,'VERIFIER_OS_ISOLATED')
''')
  runtime_args=[]
  if self.mode=='runtime-snapshot':
   self.runtime=build_verifier_runtime(self.r);runtime_args=['snapshotted']
  suites=[]
  for task in ['a','b']:
   suites += [{'id':task+'-initial','task':task,'base':self.base,'script':pin(self.suite),'args':[task,str(self.r),*runtime_args]},{'id':task+'-combined','task':task,'base':None,'script':pin(self.suite),'args':['combined',str(self.r),*runtime_args]}]
  policy={'project':'p','root':str(self.r/'authority'),'attempts_root':str(self.r/'workers'),'canonical':str(self.r/'canonical'),'protected_ref':'refs/heads/protected','git':pin('/usr/bin/git'),'python':pin('/usr/bin/python3'),'bubblewrap':pin('/usr/bin/bwrap'),'suites':suites}
  if self.runtime:policy['verifier_runtime']={'root':self.runtime['root'],'manifest':self.runtime['manifest']}
  self.policy=self.r/'git.json';self.policy.write_text(json.dumps(policy))
  worker='''import pathlib,time,json,subprocess,os
s=pathlib.Path('/scratch');w=pathlib.Path('/workspace')
while not (s/'action.json').exists():time.sleep(.02)
a=json.loads((s/'action.json').read_text());name=a['task'];host=a['host'];pid=a['host_pid']
assert (w/'.git').is_dir(),'controller must provision complete Git before dispatch'
assert not (w/'.git/objects/info/alternates').exists()
assert not pathlib.Path(host).exists(),'canonical/controller path exposed'
try:os.kill(pid,0);raise AssertionError('controller signal authority exposed')
except ProcessLookupError:pass
code="import pathlib,os\\ndef answer():\\n try: pathlib.Path("+repr(host+'/secret')+").write_text('FORBIDDEN')\\n except (FileNotFoundError,PermissionError): print('CANDIDATE_OUTSIDE_EFFECT_DENIED')\\n else: raise AssertionError('verifier outside authority exposed')\\n return "+repr('A' if name=='a' else 'B')+"\\n"
if a.get('flood'):code=code.replace('def answer():','def answer():\\n try: print("x"*(26*1024*1024))\\n except BrokenPipeError: os.dup2(os.open("/dev/null",os.O_WRONLY),1)')
(w/('task_'+name+'.py')).write_text(code)
def git(*args):return subprocess.run(['/usr/bin/git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-C','/workspace',*args],check=True,capture_output=True,text=True).stdout.strip()
git('add','--all');git('-c','user.name=Worker','-c','user.email=worker@invalid','commit','-m','private '+name)
# Actual hostile worker-owned Git configuration and hook/upload-pack probes.
(w/'.git/hooks').mkdir(exist_ok=True);evil=w/'.git/hooks/evil';evil.write_text('#!/usr/bin/python3\\nimport pathlib\\ntry:pathlib.Path('+repr(host+'/secret')+').write_text("FORBIDDEN")\\nexcept (FileNotFoundError,PermissionError):pathlib.Path("/scratch/hostile-denied").write_text("outside denied")\\nelse:raise AssertionError("outside exposed")\\n');evil.chmod(0o700)
git('config','core.fsmonitor','/workspace/.git/hooks/evil')
subprocess.run(['/usr/bin/git','-C','/workspace','status','--porcelain'],capture_output=True)
subprocess.run(['/usr/bin/git','clone','--upload-pack=/workspace/.git/hooks/evil','/workspace','/scratch/hostile-clone'],capture_output=True)
assert (s/'hostile-denied').exists(),'malicious config/upload-pack probe must actually execute inside boundary'
(s/'ready').write_text(git('rev-parse','HEAD'))
while not (s/'finish').exists():
 if (s/'mutations.json').exists():
  m=json.loads((s/'mutations.json').read_text());exec(m['python']);(s/'mutations.json').unlink();(s/'mutations-done').write_text(m['id'])
 if (s/'hostile.json').exists():
  h=json.loads((s/'hostile.json').read_text());f=w/'.git/hooks/filter';f.write_text(h['filter_code']);f.chmod(0o700)
  git('config','filter.hostile.clean',h['command']);(w/'.gitattributes').write_text('task_a.py filter=hostile\\n')
  p=w/'task_a.py';p.write_text(p.read_text().replace("return 'A'","return 'Z'"));os.utime(p,(time.time()+5,time.time()+5))
  (s/'hostile.json').unlink();(s/'hostile-installed').write_text('owned worker configured hostile clean filter')
 time.sleep(.02)
'''
  config={'root':str(self.r/'workers'),'profile':{'executable':'/usr/bin/python3','args':['-c',worker],'codex':None}}
  self.supervisor_config=self.r/'supervisor.json';self.supervisor_config.write_text(json.dumps(config))
  self.supervisor=self.spawn([str(BIN.parent/'corenius-attempt-supervisor'),str(self.supervisor_config),str(self.r/'supervisor.sock')],'supervisor',{'PATH':'/usr/bin:/bin','CORENIUS_SUPERVISOR_TOKEN':SUPERVISOR,'CORENIUS_GIT_CONFIG':str(self.policy)})
  wait(lambda:self.alive(self.supervisor,'supervisor') and (self.r/'supervisor.sock').exists())
  self.env={'PATH':'/usr/bin:/bin','CORENIUS_OPERATOR_TOKEN':TOKEN,'CORENIUS_GIT_CONFIG':str(self.policy),'CORENIUS_SUPERVISOR_SOCKET':str(self.r/'supervisor.sock'),'CORENIUS_SUPERVISOR_TOKEN':SUPERVISOR}
  self.controller=self.start_controller('controller')
  self.command('create_project',{'name':'p','base':self.base})
 def spawn(self,argv,name,env):
  p=subprocess.Popen(argv,env=env,stdout=open(self.r/(name+'.stdout'),'w'),stderr=open(self.r/(name+'.stderr'),'w'));fd=os.pidfd_open(p.pid)
  start=pathlib.Path('/proc/'+str(p.pid)+'/stat').read_text().rsplit(') ',1)[1].split()[19]
  self.processes.append((p,fd));self.records.append({'name':name,'pid':p.pid,'start_ticks':start,'argv':argv});return p
 def alive(self,p,name):
  assert p.poll() is None,(name,(self.r/(name+'.stderr')).read_text());return True
 def start_controller(self,name):
  p=self.spawn([str(BIN),str(self.r/'state.db'),str(self.r/'controller.sock')],name,self.env)
  def ready():
   self.alive(p,name)
   try:return request(self.r/'controller.sock',{'auth':{'principal':'operator','token':TOKEN},'project':'p','query':{'kind':'storage_configuration','args':{}}}).get('ok')
   except (ConnectionRefusedError,FileNotFoundError,ConnectionResetError):return False
  wait(ready);return p
 def query(self,kind,args=None,auth=None):
  result=request(self.r/'controller.sock',{'auth':auth or {'principal':'operator','token':TOKEN},'project':'p','query':{'kind':kind,'args':args or {}}});assert 'ok' in result,result
  if kind=='snapshot':
   for a in result['ok']['attempts']:
    if a.get('observation'):
     try:e=json.loads(a['observation'])
     except ValueError:continue
     pid=e.get('namespace_init');start=e.get('init_start_ticks');key=(pid,start)
     if pid and key not in self.namespaces:
      try:
       fd=os.pidfd_open(pid);poll=select.poll();poll.register(fd,select.POLLIN);actual=pathlib.Path('/proc/'+str(pid)+'/stat').read_text().rsplit(') ',1)[1].split()[19]
       assert actual==start
       if not poll.poll(0):self.namespaces[key]=(fd,e)
       else:os.close(fd)
      except ProcessLookupError:pass
  return result['ok']
 def command(self,kind,args,auth=None,key=None,revision=None,allow_error=False):
  automatic=revision is None and key is None
  if revision is None:
   revision=0 if kind=='create_project' else self.query('snapshot')['revision']
  req={'auth':auth or {'principal':'operator','token':TOKEN},'project':'p','command':{'kind':kind,'args':args,'expected_revision':revision,'idempotency_key':key or str(uuid.uuid4())}}
  value=request(self.r/'controller.sock',req)
  for _ in range(25):
   if not automatic or 'revision conflict' not in value.get('error',''):break
   req['command']['expected_revision']=self.query('snapshot')['revision'];req['command']['idempotency_key']=str(uuid.uuid4());value=request(self.r/'controller.sock',req)
  if not allow_error:assert 'ok' in value,(kind,value)
  return value if allow_error else value['ok']['result']
 def admit(self,task):
  self.command('create_task',{'task':task,'work_ref':{'deployment':'fixture','project':'p','item':task},'source_revision':'r1','brief':'fixed bounded '+task+' contract; preserve existing peer work'})
  return self.command('admit',{'task':task})
 def ready(self,grant,task,flood=False):
  root=self.r/'workers'/grant['attempt'];wait(lambda:(root/'workspace/.git/objects').is_dir())
  (root/'scratch/action.json').write_text(json.dumps({'task':task,'host':str(self.r),'host_pid':self.controller.pid,'flood':flood}))
  wait(lambda:self.query('snapshot') and (root/'scratch/ready').exists());wait(lambda:next(a for a in self.query('snapshot')['attempts'] if a['id']==grant['attempt'])['state']=='running')
  return (root/'scratch/ready').read_text()
 def mutate(self,grant,code):
  root=self.r/'workers'/grant['attempt'];key=str(uuid.uuid4());(root/'scratch/mutations.json').write_text(json.dumps({'id':key,'python':code}));wait(lambda:(root/'scratch/mutations-done').exists() and (root/'scratch/mutations-done').read_text()==key)
 def capture(self,grant):return self.command('capture_candidate',{'report':'worker report is evidence content, never verifier authority'},auth={'principal':grant['principal'],'token':grant['token']})
 def accept(self,c):
  v=self.command('verify_candidate',{'candidate':c['candidate']});assert v['verdict']=='pass',v
  accepted=self.command('accept_candidate',{'candidate':c['candidate'],'verification':v['verification']});return v,accepted
 def stop_attempt(self,grant):
  (self.r/'workers'/grant['attempt']/'scratch/finish').write_text('owned deterministic executor finish')
  wait(lambda:next(a for a in self.query('snapshot')['attempts'] if a['id']==grant['attempt'])['state']=='exited')
  e=next(a for a in self.query('snapshot')['attempts'] if a['id']==grant['attempt']);assert json.loads(e['observation'])['state']=='exited'
 def terminate(self,p):
  fd=next(fd for child,fd in self.processes if child is p)
  if p.poll() is None:signal.pidfd_send_signal(fd,signal.SIGKILL)
  p.wait(timeout=5)
 def close(self):
  # Qualify only namespace identities from this fixture's supervisor records,
  # including an early setup failure before the first snapshot was obtained.
  for path in (self.r/'workers').glob('*.json'):
   try:
    e=json.loads(path.read_text());pid=e.get('namespace_init');start=e.get('init_start_ticks');key=(pid,start)
    if not pid or key in self.namespaces:continue
    fd=os.pidfd_open(pid);stat=pathlib.Path('/proc/'+str(pid)+'/stat').read_text().rsplit(') ',1)[1].split();namespace=int(os.readlink('/proc/'+str(pid)+'/ns/pid').split('[')[1][:-1]);poll=select.poll();poll.register(fd,select.POLLIN)
    if stat[19]==start and namespace==e['pid_namespace'] and not poll.poll(0):self.namespaces[key]=(fd,e)
    else:os.close(fd)
   except (FileNotFoundError,ProcessLookupError):pass
  for p,fd in reversed(self.processes):
   if p.poll() is None:signal.pidfd_send_signal(fd,signal.SIGKILL)
   p.wait(timeout=5);os.close(fd)
  dispositions=[]
  for (pid,start),(fd,e) in self.namespaces.items():
   poll=select.poll();poll.register(fd,select.POLLIN);stopped=bool(poll.poll(0))
   if not stopped:signal.pidfd_send_signal(fd,signal.SIGKILL)
   terminated=bool(poll.poll(5000));os.close(fd);assert terminated
   dispositions.append({'pid':pid,'start_ticks':start,'handle':e['handle'],'pid_namespace':e['pid_namespace'],'stopped_before_fixture_cleanup':stopped,'whole_process_terminated':terminated})
  for record,(p,_) in zip(self.records,self.processes):record['exit']=p.returncode
  (self.r/'process-dispositions.json').write_text(json.dumps({'processes':self.records,'namespaces':dispositions},indent=2))
  root=REPO/('.scratch/sqlite-controller/05/git-fixtures' if self.mode=='observer-authority' else '.scratch/sqlite-controller/04/fixtures');root.mkdir(parents=True,exist_ok=True)
  archive=root/(self.mode+'-'+self.r.name+'.tar.gz')
  with tarfile.open(archive,'w:gz') as a:a.add(self.r,arcname=self.r.name)
  print('retained_actual_fixture',json.dumps({'mode':self.mode,'archive':str(archive),'sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'processes':self.records,'namespaces':dispositions}),flush=True)
  self.temp.cleanup()
def attempt_progress():
 f=Fixture('attempt-progress')
 try:
  a=f.admit('a');f.ready(a,'a');old_auth={k:a[k] for k in ['principal','token']}
  f.command('report_progress',{'task':'a','text':'OLD_ATTEMPT_ONLY_CHECKPOINT'},auth=old_auth)
  c=f.capture(a);f.accept(c);f.stop_attempt(a)
  f.command('request_repair',{'task':'a','candidate':c['candidate'],'rationale':'new independent repair attempt'})
  b=f.command('admit',{'task':'a'});wait(lambda:next(x for x in f.query('snapshot')['attempts'] if x['id']==b['attempt'])['state']=='running')
  f.command('report_progress',{'task':'a','text':'OPERATOR_TASK_CHECKPOINT'})
  before=f.query('control_center');worker=next(x for x in before['workers'] if x['attempt']['id']==b['attempt'])
  old_denied=f.command('report_progress',{'task':'a','text':'REVOKED_PROGRESS'},auth=old_auth,allow_error=True);assert 'error' in old_denied
  assert worker['reported_progress'] is None, 'fresh Attempt must not inherit predecessor or task-level progress'
  history=before['progress_reports']
  assert any(x['attempt']==a['attempt'] and x['report_scope']=='attempt' and 'OLD_ATTEMPT_ONLY_CHECKPOINT' in json.dumps(x) for x in history)
  assert any(x['attempt'] is None and x['report_scope']=='task' and 'OPERATOR_TASK_CHECKPOINT' in json.dumps(x) for x in history)
  f.command('report_progress',{'task':'a','text':'NEW_ATTEMPT_CHECKPOINT'},auth={k:b[k] for k in ['principal','token']})
  after=f.query('control_center');reported=next(x for x in after['workers'] if x['attempt']['id']==b['attempt'])['reported_progress']
  assert reported['attribution']['attempt']==b['attempt'] and 'NEW_ATTEMPT_CHECKPOINT' in json.dumps(reported)
  events=f.query('events');assert any('OLD_ATTEMPT_ONLY_CHECKPOINT' in json.dumps(x) for x in events)
  (f.r/'attempt-progress-result.json').write_text(json.dumps({'old_attempt':a['attempt'],'new_attempt':b['attempt'],'before':before,'after':after,'old_denied':old_denied},indent=2))
  print('actual_current_attempt_progress_and_attributed_history_PASS',json.dumps({'old_attempt':a['attempt'],'new_attempt':b['attempt']}),flush=True)
 finally:f.close()
def normal():
 f=Fixture('normal')
 try:
  a=f.admit('a');b=f.admit('b');C=f.ready(a,'a');D=f.ready(b,'b')
  snapshot=f.query('snapshot');assert snapshot['occupied_slots']==2 and all(x['state']=='running' for x in snapshot['attempts'])
  for grant in [a,b]:
   repo=f.r/'workers'/grant['attempt']/'workspace';assert (repo/'.git').is_dir() and not (repo/'.git/objects/info/alternates').exists()
   assert os.stat(repo/'.git/objects').st_ino!=os.stat(f.r/'canonical/objects').st_ino
   assert all(os.stat(path).st_nlink==1 for path in (repo/'.git/objects').rglob('*') if path.is_file())
  assert os.stat(f.r/'workers'/a['attempt']/'workspace/.git').st_ino!=os.stat(f.r/'workers'/b['attempt']/'workspace/.git').st_ino
  inventory=f.query('workspace_inventory');assert len(inventory['workspaces'])==2 and not inventory['ref_drift'];assert all(w['ownership_drift'] is None for w in inventory['workspaces'])
  center=f.query('control_center');assert center['occupied_slots']==2 and len(center['filesystem']['workspaces'])==2
  (f.r/'normal-inventory-pair.json').write_text(json.dumps({'inventory':inventory,'center':center},indent=2))
  def stable_inspections(rows):
   stable=json.loads(json.dumps(rows))
   for row in stable:
    process=row['inspection']['process'];assert process['exit']==0 and process['waited']
    assert process.pop('pid')>0 and process.pop('start_ticks')
   return stable
  assert stable_inspections(center['filesystem']['workspaces'])==stable_inspections(inventory['workspaces']) and not center['projection_consistency']['refresh_required']
  ca=f.capture(a);cb=f.capture(b);assert ca['commit']==C and cb['commit']==D and ca['base']==cb['base']==f.base
  for grant,c in [(a,ca),(b,cb)]:
   ev=f.query('candidate_evidence',{'candidate':c['candidate']});assert ev['candidate']['commit']==c['commit'];assert not ev['verification']
   assert f.command('verify_candidate',{'candidate':c['candidate']},auth={'principal':grant['principal'],'token':grant['token']},allow_error=True).get('error')
  va,_=f.accept(ca);vb,_=f.accept(cb)
  center=f.query('control_center');assert {c['candidate'] for c in center['landing_queue']}=={ca['candidate'],cb['candidate']}
  assert {v['id'] for v in center['verification']}=={va['verification'],vb['verification']}
  assert va['document']['materialization']!=vb['document']['materialization'] and all('/workers/' not in v['document']['materialization'] for v in [va,vb])
  assert f.command('accept_candidate',{'candidate':ca['candidate'],'verification':vb['verification']},allow_error=True).get('error')
  for extra in [{'bundle':'forged'},{'workspace':str(f.r/'canonical')},{'report_digest':'forged'}]:
   assert f.command('capture_candidate',dict({'report':'x'},**extra),auth={'principal':a['principal'],'token':a['token']},allow_error=True).get('error')
  assert (f.r/'secret').read_text()=='outside effect must remain unchanged'
  land=f.command('land_candidate',{'candidate':ca['candidate'],'expected_trunk':f.base});assert land['state']=='landed' and git(f.r/'canonical','rev-parse','refs/heads/protected')==C
  stale=f.command('land_candidate',{'candidate':cb['candidate'],'expected_trunk':f.base});assert stale['state']=='stale_base' and git(f.r/'canonical','rev-parse','refs/heads/protected')==C
  f.stop_attempt(a);f.stop_attempt(b)
  d2=f.command('admit',{'task':'b'});assert d2['base']==C and d2['attempt']!=b['attempt']
  D2=f.ready(d2,'b');c2=f.capture(d2);assert c2['commit']==D2 and c2['base']==C
  assert f.command('accept_candidate',{'candidate':c2['candidate'],'verification':vb['verification']},allow_error=True).get('error')
  v2,_=f.accept(c2);assert v2['document']['suite_id']=='b-combined'
  assert f.command('land_candidate',{'candidate':c2['candidate'],'expected_trunk':C})['state']=='landed';assert git(f.r/'canonical','rev-parse','refs/heads/protected')==D2
  old=f.query('candidate_evidence',{'candidate':cb['candidate']});assert old['candidate']['commit']==D and old['verification'][0]['id']==vb['verification']
  f.stop_attempt(d2)
  # Actual dirty and untracked preservation; failure is separately retryable.
  root=f.r/'workers'/d2['attempt'];(root/'workspace/untracked').write_text('preserve me')
  denied=f.command('retire_workspace',{'attempt':d2['attempt'],'retire_refs':['refs/heads/attempt']},allow_error=True);assert 'dirty/untracked' in denied['error'] and (root/'workspace/untracked').exists()
  # The fixture owns this newly generated file and explicitly retains/removes it.
  (f.r/'retained-untracked').write_bytes((root/'workspace/untracked').read_bytes());(root/'workspace/untracked').unlink()
  retired=f.command('retire_workspace',{'attempt':d2['attempt'],'retire_refs':['refs/heads/attempt']});assert retired['state']=='retired' and not root.exists()
  assert f.query('workspace_inventory')['workspaces'][-1]['cleanup_disposition']=='retired'
  assert f.command('capture_candidate',{'report':'stale'},auth={'principal':d2['principal'],'token':d2['token']},allow_error=True).get('error')
  print('private_candidates_C_D_stale_D2_combined_verification_serialized_FF_PASS',json.dumps({'B':f.base,'C':C,'D':D,'D2':D2,'C_proof':va,'D_proof':vb,'D2_proof':v2,'retirement':retired}),flush=True)
 finally:f.close()
def hostile():
 f=Fixture('hostile-scratch')
 try:
  a=f.admit('a');f.ready(a,'a');root=f.r/'workers'/a['attempt'];repo=root/'workspace'
  sentinel='owned-capture-sentinel-'+uuid.uuid4().hex;(f.r/'secret').write_text(sentinel)
  filter_code="""#!/usr/bin/python3
import os,pathlib,json,sys
host=sys.argv[1]
for name in ['/scratch/candidate.bundle','/scratch/ancestor']:
 try:os.symlink(host,name)
 except FileExistsError:pass
receipt={name:os.path.islink(name) for name in ['/scratch/candidate.bundle','/scratch/ancestor']}
try:pathlib.Path(host).read_bytes()
except (FileNotFoundError,PermissionError):receipt['outside_read_denied']=True
else:receipt['outside_read_denied']=False
pathlib.Path('/workspace/.git/hostile-scratch-receipt').write_text(json.dumps(receipt))
sys.stdout.buffer.write(sys.stdin.buffer.read())
"""
  # The actual supervised worker remains the sole Git writer; the outer fixture
  # only supplies its declared adversarial action through private scratch.
  (root/'scratch/hostile.json').write_text(json.dumps({'filter_code':filter_code,'command':'/usr/bin/python3 /workspace/.git/hooks/filter '+str(f.r/'secret')}))
  wait(lambda:(root/'scratch/hostile-installed').exists())
  result=f.command('capture_candidate',{'report':'actual hostile clean filter scratch replacement probe'},auth={'principal':a['principal'],'token':a['token']},allow_error=True)
  print('actual_hostile_capture_result',json.dumps(result),flush=True)
  receipt=json.loads((repo/'.git/hostile-scratch-receipt').read_text());assert all(receipt.values()),receipt
  assert (f.r/'secret').read_text()==sentinel
  secret_digest=hashlib.sha256(sentinel.encode()).hexdigest()
  assert not (f.r/'authority/artifacts'/secret_digest).exists()
  assert all(sentinel.encode() not in path.read_bytes() for path in (f.r/'authority/artifacts').iterdir())
  assert sentinel not in json.dumps(result) and secret_digest not in json.dumps(result)
  print('actual_hostile_clean_filter_final_and_ancestor_replacements_NO_HOST_READ_PASS',json.dumps({'receipt':receipt,'result':result,'outside_digest_not_retained':True}),flush=True)
  f.stop_attempt(a)
 finally:f.close()
def cancel_race():
 f=Fixture('capture-cancel-race')
 try:
  a=f.admit('a');f.ready(a,'a');f.terminate(f.controller)
  f.env['CORENIUS_TEST_GIT_CAPTURE_BARRIER']=str(f.r/'capture-barrier');f.controller=f.start_controller('controller-capture-barrier')
  (f.r/'capture-barrier').write_text('declared retained bytes before candidate publication')
  with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
   future=pool.submit(f.command,'capture_candidate',{'report':'race'},auth={'principal':a['principal'],'token':a['token']},allow_error=True)
   wait(lambda:(f.r/'capture-barrier.observed').exists())
   started=time.monotonic();f.command('cancel_attempt',{'attempt':a['attempt']});assert time.monotonic()-started<2,'cancellation must remain responsive during external Git capture'
   (f.r/'capture-barrier').unlink();outcome=future.result(timeout=10)
  assert 'authority' in outcome['error'],outcome
  assert not f.query('workspace_inventory')['workspaces'][0]['current_candidate']
  wait(lambda:next(x for x in f.query('snapshot')['attempts'] if x['id']==a['attempt'])['state']=='exited')
  print('actual_capture_retention_cancel_race_NO_AUTHORITATIVE_CANDIDATE_PASS',json.dumps(outcome),flush=True)
 finally:f.close()
def cleanup_crash(point):
 f=Fixture(point)
 try:
  a=f.admit('a');f.ready(a,'a');c=f.capture(a);f.stop_attempt(a);f.terminate(f.controller)
  f.env.update(CORENIUS_TEST_GIT_CRASH_POINT=point,CORENIUS_TEST_GIT_CRASH_MARKER=str(f.r/'crash'));f.controller=f.start_controller('controller-cleanup-armed')
  (f.r/'crash').write_text('declared owned cleanup crash');revision=f.query('snapshot')['revision']
  try:f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']},key='cleanup-crash',revision=revision)
  except (AssertionError,json.JSONDecodeError):pass
  wait(lambda:f.controller.poll()==81);observed=json.loads((f.r/'crash.observed').read_text());root=f.r/'workers'/a['attempt']
  assert root.exists()==(point!='after_workspace_cleanup')
  if point=='partial_workspace_cleanup':assert not (root/'workspace/.git').exists() and not (root/'workspace-owner.json').exists()
  f.controller=f.start_controller('controller-cleanup-recovered')
  if point=='partial_workspace_cleanup':
   original=root.with_name(root.name+'-preserved');root.rename(original);root.mkdir();(root/'unknown-owner').write_text('replacement must survive')
   replacement=f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']},key='cleanup-crash',revision=revision,allow_error=True)
   assert replacement.get('error') and (root/'unknown-owner').read_text()=='replacement must survive'
   (root/'unknown-owner').unlink();root.rmdir();root.symlink_to(original,target_is_directory=True)
   alias=f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']},key='cleanup-crash',revision=revision,allow_error=True)
   assert alias.get('error') and root.is_symlink() and original.exists();root.unlink();original.rename(root)
   added=root/'unknown-new-work';added.write_text('new work must survive')
   unknown=f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']},key='cleanup-crash',revision=revision,allow_error=True)
   assert unknown.get('error') and added.read_text()=='new work must survive';added.unlink()
   print('actual_partial_cleanup_replacement_alias_unknown_preserved',json.dumps({'replacement':replacement,'alias':alias,'unknown':unknown}),flush=True)
  result=f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']},key='cleanup-crash',revision=revision)
  assert result['state']=='retired' and not root.exists() and result['operation']==observed['operation']
  assert f.query('workspace_inventory')['workspaces'][0]['cleanup_disposition']=='retired'
  print('actual_identity_fenced_cleanup_same_operation_recovery_PASS',json.dumps({'point':point,'operation':observed,'outcome':result}),flush=True)
 finally:f.close()
def crash(point,divergence=False):
 f=Fixture(point+('-divergence' if divergence else ''))
 try:
  a=f.admit('a');C=f.ready(a,'a');c=f.capture(a);v,_=f.accept(c)
  f.terminate(f.controller);f.env.update(CORENIUS_TEST_GIT_CRASH_POINT=point,CORENIUS_TEST_GIT_CRASH_MARKER=str(f.r/'crash'))
  f.controller=f.start_controller('controller-armed');(f.r/'crash').write_text('owned crash boundary')
  try:f.command('land_candidate',{'candidate':c['candidate'],'expected_trunk':f.base},key='crash-land')
  except (AssertionError,json.JSONDecodeError):pass
  wait(lambda:f.controller.poll()==81);observation=json.loads((f.r/'crash.observed').read_text());actual=git(f.r/'canonical','rev-parse','refs/heads/protected')
  assert actual==(f.base if point=='before_ref_update' else C)
  if divergence:
   # This is a deliberate write to the test's unique canonical fixture only.
   (f.r/'seed/divergence').write_text('unexplained fixture ref change');git(f.r/'seed','add','divergence');git(f.r/'seed','-c','user.name=Fixture','-c','user.email=fixture@invalid','commit','-m','unexplained divergence')
   changed=git(f.r/'seed','rev-parse','HEAD');git(f.r/'seed','push',str(f.r/'canonical'),changed+':refs/heads/protected')
  f.controller=f.start_controller('controller-recovered');snapshot=f.query('snapshot');ops=[o for o in snapshot['operations'] if o['id']==observation['operation']];assert len(ops)==1
  if divergence:
   assert ops[0]['state']=='unknown' and git(f.r/'canonical','rev-parse','refs/heads/protected')==changed
   f.command('create_task',{'task':'b','work_ref':{'deployment':'fixture','project':'p','item':'b'},'source_revision':'r1','brief':'b'})
   assert f.command('admit',{'task':'b'},allow_error=True).get('error')
   assert f.query('workspace_inventory')['ref_drift']
  else:
   assert ops[0]['state']=='confirmed' and snapshot['project']['base']==C and git(f.r/'canonical','rev-parse','refs/heads/protected')==C
  print('actual_journaled_conditional_ref_recovery_PASS',json.dumps({'point':point,'divergence':divergence,'observed_operation':observation,'protected_ref_before_restart':actual,'operations':ops}),flush=True)
  f.stop_attempt(a)
 finally:f.close()
def slow_inventory():
 f=Fixture('slow-inventory')
 try:
  a=f.admit('a');f.ready(a,'a');root=f.r/'workers'/a['attempt']
  slow="#!/usr/bin/python3\nimport pathlib,time,sys\npathlib.Path('/workspace/.git/slow-observed').write_text('actual clean filter entered');time.sleep(6);sys.stdout.buffer.write(sys.stdin.buffer.read())\n"
  code="(w/'.git/hooks/slow').write_text("+repr(slow)+");git('config','filter.slow.clean','/usr/bin/python3 /workspace/.git/hooks/slow');(w/'.gitattributes').write_text('task_a.py filter=slow\\n');p=w/'task_a.py';p.write_text(p.read_text().replace(\"return 'A'\",\"return 'Z'\"));os.utime(p,(time.time()+5,time.time()+5))"
  f.mutate(a,code)
  with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
   future=pool.submit(f.query,'workspace_inventory');wait(lambda:(root/'workspace/.git/slow-observed').exists())
   started=time.monotonic();status=f.query('snapshot');f.command('report_progress',{'task':'a','text':'independent control while inventory observes untrusted Git'});f.command('cancel_attempt',{'attempt':a['attempt']});seconds=time.monotonic()-started
   was_blocked=not future.done();inventory=future.result(timeout=15)
  print('actual_public_inventory_responsiveness',json.dumps({'seconds':seconds,'inventory_still_blocked_during_control':was_blocked,'snapshot':status,'inventory':inventory}),flush=True)
  assert seconds<2 and was_blocked,'external inventory must not hold the DB connection mutex'
  assert inventory['snapshot_revision']<=status['revision'] and inventory['state_drift'],'inventory must report its snapshot age and intervening state drift'
  wait(lambda:next(x for x in f.query('snapshot')['attempts'] if x['id']==a['attempt'])['state']=='exited')
 finally:f.close()
def recovery_publication(point):
 f=Fixture('publication-'+point)
 try:
  a=f.admit('a');C=f.ready(a,'a');c=f.capture(a);f.accept(c);f.stop_attempt(a);lead=f.command('replace_lead',{});actor={k:lead[k] for k in ['principal','token']};f.terminate(f.controller)
  f.env.update(CORENIUS_TEST_GIT_BARRIER_POINT='after_land_admission',CORENIUS_TEST_GIT_BARRIER_FILE=str(f.r/'admission-barrier'),CORENIUS_TEST_GIT_CRASH_POINT=point,CORENIUS_TEST_GIT_CRASH_MARKER=str(f.r/'crash'));f.controller=f.start_controller('controller-publication-armed');(f.r/'admission-barrier').write_text('qualified admission');(f.r/'crash').write_text('actual ref crash')
  revision=f.query('snapshot')['revision'];cursor=f.query('events')[-1]['seq']
  with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
   future=pool.submit(f.command,'land_candidate',{'candidate':c['candidate'],'expected_trunk':f.base},auth=actor,key='publication-land',revision=revision,allow_error=True)
   wait(lambda:(f.r/'admission-barrier.observed').exists());replacement=f.command('replace_lead',{});before=f.query('snapshot');(f.r/'admission-barrier').unlink()
   try:future.result(timeout=12)
   except json.JSONDecodeError:pass
  wait(lambda:f.controller.poll()==81);f.controller=f.start_controller('controller-publication-recovered');snapshot=f.query('snapshot');events=f.query('events',{'after':cursor});lands=[e for e in events if e['kind']=='land_candidate']
  print('actual_public_recovered_landing_publication',json.dumps({'point':point,'before':before,'after':snapshot,'events':events,'original_lead':lead['principal'],'replacement_lead':replacement['principal']}),flush=True)
  assert snapshot['project']['base']==C and snapshot['revision']==before['revision']+1 and len(lands)==1,'recovery must publish one landing revision/event without original client retry'
  f.command('reconcile_git',{});assert len([e for e in f.query('events',{'after':cursor}) if e['kind']=='land_candidate'])==1
  assert f.command('land_candidate',{'candidate':c['candidate'],'expected_trunk':f.base},auth=actor,key='publication-land',revision=revision,allow_error=True).get('error')
 finally:f.close()
def landing_authority(point):
 f=Fixture(point)
 try:
  a=f.admit('a');C=f.ready(a,'a');c=f.capture(a);v,_=f.accept(c);lead=f.command('replace_lead',{});actor={k:lead[k] for k in ['principal','token']}
  f.terminate(f.controller);f.env.update(CORENIUS_TEST_GIT_BARRIER_POINT=point,CORENIUS_TEST_GIT_BARRIER_FILE=str(f.r/'admission-barrier'));f.controller=f.start_controller('controller-barrier');(f.r/'admission-barrier').write_text('declared admission ordering')
  with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
   future=pool.submit(f.command,'land_candidate',{'candidate':c['candidate'],'expected_trunk':f.base},auth=actor,allow_error=True)
   wait(lambda:(f.r/'admission-barrier.observed').exists());replacement=f.command('replace_lead',{});(f.r/'admission-barrier').unlink();result=future.result(timeout=12)
  actual=git(f.r/'canonical','rev-parse','refs/heads/protected');snapshot=f.query('snapshot');print('actual_public_landing_authority_boundary',json.dumps({'point':point,'result':result,'actual_ref':actual,'snapshot':snapshot,'replacement':replacement['principal']}),flush=True)
  assert actual==(f.base if point=='before_land_admission' else C),'authority must be current at fully admitted intent, durable afterward'
  if point=='before_land_admission':assert result.get('error') and not any(t['state']=='landed' for t in snapshot['tasks'])
  else:assert snapshot['project']['base']==C
  f.stop_attempt(a)
 finally:f.close()
def terminal_lifecycle(mode):
 f=Fixture(mode)
 try:
  a=f.admit('a');C=f.ready(a,'a');grant={'principal':a['principal'],'token':a['token']};cr=f.query('snapshot')['revision'];c=f.command('capture_candidate',{'report':'terminal lifecycle'},auth=grant,key='stable-capture',revision=cr)
  v=f.command('verify_candidate',{'candidate':c['candidate']});ar=f.query('snapshot')['revision'];accept_args={'candidate':c['candidate'],'verification':v['verification']};accept=f.command('accept_candidate',accept_args,key='stable-accept',revision=ar)
  land_args={'candidate':c['candidate'],'expected_trunk':f.base};lr=f.query('snapshot')['revision']
  if mode!='accepted-capture':land=f.command('land_candidate',land_args,key='stable-land',revision=lr)
  before=f.query('snapshot');candidate_before=f.query('workspace_inventory')['workspaces'][0]['current_candidate']
  if mode=='terminal-land':outcome=f.command('land_candidate',land_args,allow_error=True)
  elif mode=='terminal-accept':outcome=f.command('accept_candidate',accept_args,allow_error=True)
  else:outcome=f.command('capture_candidate',{'report':'must refuse reopening'},auth=grant,allow_error=True)
  after=f.query('snapshot');print('actual_public_terminal_lifecycle_observation',json.dumps({'mode':mode,'outcome':outcome,'before':before,'after':after}),flush=True)
  assert outcome.get('error'),'fresh mutation must not reopen accepted/landed task'
  assert after['tasks']==before['tasks'] and f.query('workspace_inventory')['workspaces'][0]['current_candidate']==candidate_before
  assert f.command('capture_candidate',{'report':'terminal lifecycle'},auth=grant,key='stable-capture',revision=cr)==c
  assert f.command('accept_candidate',accept_args,key='stable-accept',revision=ar)==accept
  if mode=='terminal-land':assert f.command('land_candidate',land_args,key='stable-land',revision=lr)==land
  assert git(f.r/'canonical','rev-parse','refs/heads/protected')==(f.base if mode=='accepted-capture' else C)
  f.stop_attempt(a)
 finally:f.close()
def accepted_verification():
 f=Fixture('accepted-verification')
 try:
  a=f.admit('a');C=f.ready(a,'a');c=f.capture(a);args={'candidate':c['candidate']};revision=f.query('snapshot')['revision']
  proof=f.command('verify_candidate',args,key='stable-verification',revision=revision);assert proof['verdict']=='pass'
  acceptance=f.command('accept_candidate',{'candidate':c['candidate'],'verification':proof['verification']})
  before=f.query('snapshot');evidence=f.query('candidate_evidence',args);ref_before=git(f.r/'canonical','rev-parse','refs/heads/protected')
  fresh=f.command('verify_candidate',args,allow_error=True);after=f.query('snapshot');after_evidence=f.query('candidate_evidence',args)
  blocked=None
  if not fresh.get('error'):
   blocked=f.command('land_candidate',{'candidate':c['candidate'],'expected_trunk':f.base},allow_error=True)
   assert blocked.get('error') and git(f.r/'canonical','rev-parse','refs/heads/protected')==f.base,'old behavior must demonstrate accepted proof is now unlandable'
  print('actual_public_accepted_verification_admission',json.dumps({'candidate':c,'accepted_proof':proof,'acceptance':acceptance,'fresh':fresh,'old_composition_land_refusal':blocked,'before':before,'after':after,'before_evidence':evidence,'after_evidence':after_evidence,'ref_before':ref_before}),flush=True)
  assert fresh.get('error'),'fresh verification must refuse before journaling or superseding accepted proof'
  assert after['revision']==before['revision'] and after['tasks']==before['tasks'] and after['operations']==before['operations'],'refusal must not insert latest verification operation or alter lifecycle/revision'
  assert after_evidence==evidence,'accepted candidate/proof/report/decision must remain exact'
  assert f.command('verify_candidate',args,key='stable-verification',revision=revision)==proof,'stable prior verification must replay despite accepted state'
  landed=f.command('land_candidate',{'candidate':c['candidate'],'expected_trunk':f.base});assert landed['state']=='landed' and git(f.r/'canonical','rev-parse','refs/heads/protected')==C
  terminal=f.query('snapshot');landed_evidence=f.query('candidate_evidence',args);fresh_landed=f.command('verify_candidate',args,allow_error=True);terminal_after=f.query('snapshot')
  assert fresh_landed.get('error') and terminal_after['revision']==terminal['revision'] and terminal_after['tasks']==terminal['tasks'] and terminal_after['operations']==terminal['operations']
  assert f.query('candidate_evidence',args)==landed_evidence and f.command('verify_candidate',args,key='stable-verification',revision=revision)==proof
  print('actual_accepted_proof_stable_replay_original_landing_terminal_refusal_PASS',json.dumps({'proof':proof,'landed':landed,'fresh_landed':fresh_landed,'before':terminal,'after':terminal_after,'actual_ref':C}),flush=True)
  f.stop_attempt(a)
 finally:f.close()
def runtime_snapshot():
 f=Fixture('runtime-snapshot')
 try:
  assert f.runtime, 'fixture must prepare verifier runtime snapshot'
  a=f.admit('a');C=f.ready(a,'a');candidate=f.capture(a)
  proof=f.command('verify_candidate',{'candidate':candidate['candidate']});assert proof['verdict']=='pass',proof
  doc=proof['document'];assert doc['runtime']['status']=='pinned' and doc['runtime']['manifest']==f.runtime['manifest']['sha256']
  log=json.loads((f.r/'authority/artifacts'/doc['logs']).read_bytes())
  assert log['runtime']['status']=='pinned' and log['runtime']['manifest']==f.runtime['manifest']['sha256']
  pathlib.Path(f.runtime['control']).write_text('tampered after trusted proof\n')
  accept=f.command('accept_candidate',{'candidate':candidate['candidate'],'verification':proof['verification']},allow_error=True)
  fresh=f.command('verify_candidate',{'candidate':candidate['candidate']},key='verify-after-runtime-tamper',allow_error=True)
  assert fresh.get('error') and 'verifier runtime' in fresh['error'],fresh
  assert accept.get('error') and 'verifier runtime' in accept['error'],accept
  print('actual_verifier_runtime_snapshot_tamper_rejected_before_dispatch_and_trusted_proof_PASS',json.dumps({'candidate':candidate,'proof':proof,'fresh_after_tamper':fresh,'accept_after_tamper':accept,'runtime_manifest':f.runtime['manifest']}),flush=True)
  f.stop_attempt(a)
 finally:f.close()
def shadow_imports():
 f=Fixture('shadow-imports')
 try:
  a=f.admit('a');f.ready(a,'a');c=f.capture(a)
  shadow="""import importlib.util,pathlib,sysconfig
spec=importlib.util.spec_from_file_location('real_subprocess',pathlib.Path(sysconfig.get_path('stdlib'))/'subprocess.py');real=importlib.util.module_from_spec(spec);spec.loader.exec_module(real)
def check_output(args,**kw):
 pathlib.Path('/workspace/shadow-executed').write_text('actual worker shadow subprocess imported')
 if 'status' in args:return ''
 return real.check_output(args,**kw)
"""
  f.mutate(a,"(w/'subprocess.py').write_text("+repr(shadow)+");(w/'untracked-evidence').write_text('preserve')")
  inventory=f.query('workspace_inventory');row=inventory['workspaces'][0]
  print('actual_public_shadow_inventory_observation',json.dumps({'inventory':inventory,'shadow_executed':(f.r/'workers'/a['attempt']/'workspace/shadow-executed').exists()}),flush=True)
  assert 'subprocess.py' in row['dirty_untracked'] and 'untracked-evidence' in row['dirty_untracked'],'trusted inspection must retain actual untracked state despite worker import shadow'
  assert not (f.r/'workers'/a['attempt']/'workspace/shadow-executed').exists(),'worker module must not execute in trusted inspection'
  f.stop_attempt(a)
 finally:f.close()
def bounded_archive():
 f=Fixture('bounded-archive')
 try:
  a=f.admit('a');f.ready(a,'a');c=f.capture(a);f.stop_attempt(a)
  root=f.r/'workers'/a['attempt'];large=root/'scratch/owned-sparse'
  with large.open('wb') as stream:stream.truncate(64*1024*1024)
  assert large.stat().st_blocks*512<1024*1024,'fixture must be sparse'
  started=time.monotonic();outcome=f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']},allow_error=True)
  assert outcome.get('error') and root.exists(),outcome
  sizes={str(p.relative_to(f.r)):p.stat().st_size for p in (f.r/'authority/captures').rglob('workspace.tar')}
  print('actual_public_archive_limit',json.dumps({'outcome':outcome,'expanded_archive_bytes':sizes,'elapsed':time.monotonic()-started,'sparse_bytes':large.stat().st_size,'sparse_allocated':large.stat().st_blocks*512}),flush=True)
  assert all(n<=32*1024*1024 for n in sizes.values()),'retention producer expanded beyond its budget before rejection'
  assert 'output bound' in outcome['error'],outcome
  diagnostic=json.loads((f.r/'authority/artifacts'/outcome['error'].split()[-1]).read_bytes());assert diagnostic['process']['limit']=='output bound' and diagnostic['process']['waited'] and diagnostic['retained_archive_bytes']<=32*1024*1024
  print('actual_bounded_archive_producer_identity',json.dumps(diagnostic),flush=True)
  with large.open('wb') as stream:stream.truncate(0)
  retry=f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']})
  assert retry['state']=='retired' and not root.exists();assert (f.r/'authority/artifacts'/retry['retention']).is_file()
  print('actual_archive_bounded_failure_and_retry_PASS',json.dumps({'failed':outcome,'retry':retry}),flush=True)
 finally:f.close()
def bounded_output(final=False):
 f=Fixture('final-output' if final else 'bounded-output')
 try:
  a=f.admit('a');f.ready(a,'a',flood=True);c=f.capture(a)
  if final:
   f.terminate(f.controller);f.env['CORENIUS_TEST_GIT_FINAL_OUTPUT_WAIT']='1';f.controller=f.start_controller('controller-final-output')
  v=f.command('verify_candidate',{'candidate':c['candidate']});print('actual_public_final_output_verdict',json.dumps(v),flush=True);assert v['verdict']=='fail',v
  log=json.loads((f.r/'authority/artifacts'/v['document']['logs']).read_bytes());assert log['process']['limit']=='output bound';assert len(log['stdout'])<=24*1024*1024 and len(log['stderr'])<=1024*1024
  if final:assert log['exit']==0 and log['process']['exit']==0,'exit-zero scheduling control must remain actual'
  print('actual_verifier_limit_and_owned_identity',json.dumps({'process':log['process'],'stdout_bytes':len(log['stdout']),'stderr_bytes':len(log['stderr'])}),flush=True)
  evidence=f.query('candidate_evidence',{'candidate':c['candidate']});assert f.command('accept_candidate',{'candidate':c['candidate'],'verification':v['verification']},allow_error=True).get('error')
  print('actual_fixed_verifier_output_bound_FAIL_CLOSED_PASS',json.dumps({'verification':v,'candidate_evidence':evidence}),flush=True)
  f.stop_attempt(a)
 finally:f.close()
def observer_authority():
 f=Fixture('observer-authority')
 try:
  a=f.admit('a');C=f.ready(a,'a');candidate=f.capture(a);f.stop_attempt(a)
  observer=f.command('create_monitor_observer',{'name':'advisory-only Git probe'})
  auth={'principal':observer['principal'],'token':observer['token']};root=f.r/'workers'/a['attempt']
  def effects():
   with sqlite3.connect((f.r/'state.db').as_uri()+'?mode=ro',uri=True) as c:
    tables=['operations','receipts','events','git_requests','git_effect_history','verification_reports','acceptance_decisions']
    return {'counts':{t:c.execute('SELECT COUNT(*) FROM '+t).fetchone()[0] for t in tables},'revision':c.execute("SELECT revision FROM projects WHERE id='p'").fetchone()[0],'protected':git(f.r/'canonical','rev-parse','refs/heads/protected'),'workspace_present':root.exists()}
  attempts=[('capture_candidate',{'report':'observer cannot capture'}),('verify_candidate',{'candidate':candidate['candidate']}),('compare_candidates',{'candidate':candidate['candidate']}),('accept_candidate',{'candidate':candidate['candidate'],'verification':'untrusted-report'}),('land_candidate',{'candidate':candidate['candidate'],'expected_trunk':f.base}),('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']}),('reconcile_git',{})]
  for kind,args in attempts:
   before=effects();revision=before['revision']
   direct=request(f.r/'controller.sock',{'auth':auth,'project':'p','command':{'kind':kind,'args':args,'expected_revision':revision,'idempotency_key':'observer-direct-'+kind}})
   gateway=request(f.r/'controller.sock',{'auth':auth,'project':'p','gateway':{'tool':'corenius_command','arguments':{'kind':kind,'args':args,'expected_revision':revision,'operation_id':'observer-gateway-'+kind}}})
   after=effects();print('observer_Git_no_effect',json.dumps({'kind':kind,'direct':direct,'gateway':gateway,'before':before,'after':after}),flush=True)
   assert direct.get('error') and gateway.get('error') and before==after,'observer must be advisory-only before journals, receipts or physical effects'
  for kind,args in [('workspace_inventory',{}),('candidate_evidence',{'candidate':candidate['candidate']})]:
   before=effects();value=request(f.r/'controller.sock',{'auth':auth,'project':'p','query':{'kind':kind,'args':args}})
   assert value.get('error') and effects()==before,'observer evidence scope must stay bounded to monitor records'
  verification,accepted=f.accept(candidate);assert accepted and verification['verdict']=='pass'
  assert f.command('land_candidate',{'candidate':candidate['candidate'],'expected_trunk':f.base})['state']=='landed'
  assert git(f.r/'canonical','rev-parse','refs/heads/protected')==C
  retired=f.command('retire_workspace',{'attempt':a['attempt'],'retire_refs':['refs/heads/attempt']});assert retired['state']=='retired' and not root.exists()
  print('actual_observer_all_Git_mutations_and_evidence_DENIED_authorized_verification_landing_retirement_PASS',flush=True)
 finally:f.close()
def bounded_inventory():
 f=Fixture('bounded-inventory')
 try:
  f.command('create_task',{'task':'a','work_ref':{'deployment':'fixture','project':'p','item':'a'},'source_revision':'r1','brief':'fixed bounded a contract; preserve existing peer work'})
  f.terminate(f.controller)
  expected=set();large='é'*32768
  with sqlite3.connect(f.r/'state.db') as db:
   brief=db.execute("SELECT brief FROM tasks WHERE id='a'").fetchone()[0]
   for n in range(25):
    attempt=str(uuid.uuid4());expected.add(attempt);task='seeded-'+str(n);operation=str(uuid.uuid4())
    db.execute("INSERT INTO tasks VALUES('p',?,?,?,'aborted')",(task,brief,task))
    db.execute("INSERT INTO attempts(id,project,task,brief,base,state,scope_epoch,observation) VALUES(?,'p',?,?,?,'exited',2,'seeded historical fixture, no prior physical execution claimed')",(attempt,task,brief,f.base))
    db.execute("INSERT INTO operations(id,project,attempt,kind,state,payload) VALUES(?,'p',?,'start','failed',?)",(operation,attempt,json.dumps({'attempt':attempt,'base':f.base,'brief':brief,'scope_epoch':1})))
    db.execute("INSERT INTO git_effect_history(operation,state,observation) VALUES(?,'failed',?)",(operation,large if n==24 else 'seeded history '+str(n)))
  f.controller=f.start_controller('restarted-controller')
  a=f.command('admit',{'task':'a'});expected.add(a['attempt']);root=f.r/'workers'/a['attempt']
  wait(lambda:(root/'workspace/.git/objects').is_dir())
  (root/'scratch/action.json').write_text(json.dumps({'task':'a','host':str(f.r),'host_pid':f.controller.pid}))
  wait(lambda:(root/'scratch/ready').exists() and f.query('runtime_context')['attempts'][0]['state']=='running')
  f.mutate(a,"[(w/('large-untracked-'+str(n)+'-'+'x'*64)).write_text('owned bounded inventory input') for n in range(1000)]")
  initial=f.query('control_center');assert initial['filesystem']['coverage']['workspace_records']==26 and initial['filesystem']['pagination']['workspaces']['has_more']
  selected=f.query('control_center',{'cursors':{'workspaces':20,'git_history':20}});assert len(selected['filesystem']['workspaces'])==6 and selected['filesystem']['workspaces'][-1]['attempt']==a['attempt']
  seen=set();history=[];args={};pages=0;references=[]
  while True:
   inventory=f.query('workspace_inventory',args);pages+=1;assert len(inventory['workspaces'])<=20 and len(inventory['history'])<=20 and len(json.dumps(inventory).encode())<1_048_576
   for item in inventory['workspaces']:
    assert item['attempt'] not in seen;seen.add(item['attempt'])
    if 'input_reference' in item:references.append(item['input_reference'])
   for item in inventory['history']:
    history.append(item)
    if 'input_reference' in item:references.append(item['input_reference'])
   if not inventory['pagination']['workspaces']['has_more'] and not inventory['pagination']['history']['has_more']:break
   args=inventory['pagination']['continuation']['args']
  assert seen==expected and len(history)==25 and references
  originals=[]
  for reference in references:
   args=reference['args'];chunks=[]
   while True:
    part=f.query(reference['kind'],args);chunks.append(part['text'])
    if part['next_offset'] is None:break
    args=dict(args,offset=part['next_offset'])
   raw=''.join(chunks);assert hashlib.sha256(raw.encode()).hexdigest()==reference['args']['input'];originals.append(json.loads(raw))
  assert any(x.get('observation')==large for x in originals)
  assert any(x.get('attempt')==a['attempt'] and 'large-untracked-' in x.get('dirty_untracked','') for x in originals)
  observer=f.command('create_monitor_observer',{'name':'inventory observer'});worker={k:a[k] for k in ['principal','token']};observer_auth={k:observer[k] for k in ['principal','token']}
  for actor in [worker,observer_auth]:assert 'error' in request(f.r/'controller.sock',{'auth':actor,'project':'p','query':{'kind':'workspace_inventory','args':references[0]['args']}})
  f.command('cancel_attempt',{'attempt':a['attempt']});wait(lambda:not f.query('runtime_context')['attempts'])
  print('actual_bounded_inventory_history',json.dumps({'seeded_exited_rows':25,'actual_active_workspace':a['attempt'],'workspace_records':len(seen),'history_records':len(history),'pages':pages,'complete_large_inputs':len(originals),'scope_negatives':True,'current_cancelled':True}),flush=True)
 finally:f.close()
def repair_landing(mode):
 f=Fixture(mode)
 try:
  a=f.admit('a');C=f.ready(a,'a');c=f.capture(a);f.accept(c);f.stop_attempt(a)
  lead=f.command('replace_lead',{});actor={k:lead[k] for k in ['principal','token']}
  f.terminate(f.controller);barrier=f.r/'admission-barrier'
  f.env.update(CORENIUS_TEST_GIT_BARRIER_POINT='after_land_admission',CORENIUS_TEST_GIT_BARRIER_FILE=str(barrier));f.controller=f.start_controller('controller-barrier');barrier.write_text('actual admitted physical landing boundary')
  revision=f.query('snapshot')['revision'];args={'candidate':c['candidate'],'expected_trunk':f.base}
  with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
   future=pool.submit(f.command,'land_candidate',args,auth=actor,key='repair3-land',revision=revision,allow_error=True)
   wait(lambda:(f.r/'admission-barrier.observed').exists())
   operation=json.loads((f.r/'admission-barrier.observed').read_text())['operation']
   if mode=='repair3-unknown':
    outcome=future.result(timeout=12);assert outcome.get('error')
    assert next(x for x in f.query('snapshot')['operations'] if x['id']==operation)['state']=='unknown'
   before=f.query('snapshot');events=f.query('events')
   with sqlite3.connect((f.r/'state.db').as_uri()+'?mode=ro',uri=True) as db:receipts=db.execute('SELECT COUNT(*) FROM receipts').fetchone()[0]
   refusal=f.command('request_repair',{'task':'a','candidate':c['candidate'],'rationale':'repair cannot supersede admitted landing'},allow_error=True)
   after=f.query('snapshot');after_events=f.query('events')
   with sqlite3.connect((f.r/'state.db').as_uri()+'?mode=ro',uri=True) as db:after_receipts=db.execute('SELECT COUNT(*) FROM receipts').fetchone()[0]
   barrier.unlink();result=future.result(timeout=12)
  record={'mode':mode,'operation':operation,'refusal':refusal,'before':before,'after':after,'events_before':events,'events_after':after_events,'receipts':[receipts,after_receipts],'land_result':result,'physical_ref_before_recovery':git(f.r/'canonical','rev-parse','refs/heads/protected')}
  (f.r/'repair3-transition.json').write_text(json.dumps(record,indent=2));print('actual_repair3_landing_transition',json.dumps(record),flush=True)
  assert refusal.get('error'),'same-task repair must refuse unresolved admitted physical landing'
  assert before==after and events==after_events and receipts==after_receipts,'denied repair must have no durable mutation'
  replacement=f.command('replace_lead',{})
  f.command('reconcile_git',{})
  assert git(f.r/'canonical','rev-parse','refs/heads/protected')==C
  assert next(t for t in f.query('snapshot')['tasks'] if t['id']=='a')['state']=='landed'
  assert len([e for e in f.query('events') if e['kind']=='land_candidate'])==1
  assert f.command('land_candidate',args,auth=actor,key='repair3-land',revision=revision,allow_error=True).get('error')
  assert f.command('request_repair',{'task':'a','candidate':c['candidate'],'rationale':'terminal landing preserved'},allow_error=True).get('error')
  b=f.admit('b');f.ready(b,'b');cb=f.capture(b);f.accept(cb);f.stop_attempt(b)
  repair_args={'task':'b','candidate':cb['candidate'],'rationale':'before landing repair remains available'}
  assert f.command('request_repair',repair_args)['state']=='repair-required'
  assert f.command('land_candidate',{'candidate':cb['candidate'],'expected_trunk':C},allow_error=True).get('error')
  assert f.command('request_repair',repair_args)['state']=='repair-required'
 finally:f.close()
if __name__=='__main__':
 modes=os.environ.get('CORENIUS_TEST_GIT_MODES','normal,before_ref_update,after_ref_update,divergence,hostile,bounded-output,cancel,before_workspace_cleanup,after_workspace_cleanup,shadow-imports,before_land_admission,after_land_admission,publication-before_ref_update,publication-after_ref_update,terminal-capture,terminal-accept,terminal-land,accepted-capture,inventory-slow,final-output,bounded-archive,partial_workspace_cleanup,accepted-verification').split(',')
 for mode in modes:
  if mode in ['repair3-applying','repair3-unknown']:repair_landing(mode)
  elif mode=='bounded-inventory':bounded_inventory()
  elif mode=='observer-authority':observer_authority()
  elif mode=='attempt-progress':attempt_progress()
  elif mode=='normal':normal()
  elif mode=='hostile':hostile()
  elif mode=='bounded-output':bounded_output()
  elif mode=='final-output':bounded_output(True)
  elif mode=='bounded-archive':bounded_archive()
  elif mode=='shadow-imports':shadow_imports()
  elif mode=='runtime-snapshot':runtime_snapshot()
  elif mode=='accepted-verification':accepted_verification()
  elif mode=='inventory-slow':slow_inventory()
  elif mode in ['publication-before_ref_update','publication-after_ref_update']:recovery_publication(mode.removeprefix('publication-'))
  elif mode in ['before_land_admission','after_land_admission']:landing_authority(mode)
  elif mode in ['terminal-capture','terminal-accept','terminal-land','accepted-capture']:terminal_lifecycle(mode)
  elif mode=='cancel':cancel_race()
  elif mode in ['before_workspace_cleanup','after_workspace_cleanup','partial_workspace_cleanup']:cleanup_crash(mode)
  elif mode=='divergence':crash('before_ref_update',True)
  else:crash(mode)

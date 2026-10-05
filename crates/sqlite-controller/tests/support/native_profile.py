"""MODEL_FREE_ACTUAL_PROTOCOL trusted supervisor native-profile qualification."""
import pathlib,tempfile,json,subprocess,socket,hashlib,time,sys,uuid
from wsutil import WS
BIN=pathlib.Path('/data02/home/philip.yang/.codex/packages/standalone/releases/0.159.3-x86_64-unknown-linux-musl/bin')
HERE=pathlib.Path(__file__).resolve().parents[2]
ORIGIN=pathlib.Path('/data02/home/philip.yang/devx/tmp/corenius-sqlite-topology-sms522gg')
def pin(path):return {'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else '0'*64}
def request(path,value):
 with socket.socket(socket.AF_UNIX) as s:
  s.settimeout(5);s.connect(str(path));s.sendall(json.dumps(value).encode()+b'\n');return json.loads(s.makefile('rb').readline())
def wait(predicate,seconds=25):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  result=predicate()
  if result:return result
  time.sleep(.025)
 raise AssertionError('behavior deadline')
with tempfile.TemporaryDirectory(prefix='cnp-',dir='/tmp') as directory:
 r=pathlib.Path(directory);(r/'executions').mkdir();(r/'transport').mkdir()
 catalog=ORIGIN/'catalog.json'; cfg=r/'runtime.toml';cfg.write_text((ORIGIN/'codex/config.toml').read_text().replace(str(catalog),'/run/codex/catalog.json').replace('35361','8777'))
 provider=r/'provider.sock';listener=socket.socket(socket.AF_UNIX);listener.bind(str(provider));listener.listen()
 profile={'kind':'app','binary':pin(BIN/'codex'),'companion':pin(BIN/'codex-code-mode-host'),'catalog':pin(catalog),'config':pin(cfg),'launcher':pin(HERE/'support/codex_insula.py'),'transport_root':str(r/'transport'),'provider_socket':str(provider)}
 config=r/'supervisor.json';config.write_text(json.dumps({'root':str(r/'executions'),'profile':{'executable':'/usr/bin/python3','args':['/run/codex/launcher.py'],'codex':profile}}))
 supervisor=pathlib.Path(sys.argv[1]).parent/'corenius-attempt-supervisor';p=subprocess.Popen([str(supervisor),str(config),str(r/'supervisor.sock')],env={'CORENIUS_SUPERVISOR_TOKEN':'native-profile-secret','PATH':'/usr/bin:/bin'},stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 binding={'project':'native','attempt':str(uuid.uuid4()),'operation':str(uuid.uuid4()),'scope_epoch':1,'base':'0123456789abcdef0123456789abcdef01234567','brief':str(uuid.uuid4())}
 try:
  def ready():
   assert p.poll() is None, 'trusted pinned native profile must start: '+p.stderr.read()
   return (r/'supervisor.sock').exists()
  wait(ready)
  identity=request(r/'supervisor.sock',{'token':'native-profile-secret','action':'start','binding':binding})
  if 'ok' not in identity: print('native_setup_output', (r/'executions'/binding['attempt']/'output.log').read_text(),flush=True)
  assert 'ok' in identity,identity
  proxy=r/'transport'/binding['attempt']/'native.sock';wait(proxy.exists)
  token=(r/'executions/native-authority'/binding['attempt']).read_text()
  def lookup():
   value=request(r/'supervisor.sock',{'token':'native-profile-secret','action':'native_lookup','binding':binding})
   return value if 'ok' in value else False
  qualified=wait(lookup)
  def connect():
   try:return WS.connect('127.0.0.1',8788,token=token,unix=proxy,path='/?corenius_start='+qualified['ok']['runtime']['runtime_start'])
   except (OSError,AssertionError):return False
  ws=wait(connect)
  try:
   stale=WS.connect('127.0.0.1',8788,token=token,unix=proxy,path='/?corenius_start=0')
  except (OSError,AssertionError):stale=None
  assert stale is None, 'actual native bridge must fence a stale service incarnation before RPC'
  ws.send({'id':1,'method':'initialize','params':{'clientInfo':{'name':'corenius_private_profile','version':'0.1'},'capabilities':{'experimentalApi':True}}});native=ws.recv();assert '0.159.3' in native['result']['userAgent'],native
  print('actual_private_native_initialize',native,flush=True)
  assert qualified['ok']['runtime']['store_id']==identity['ok']['native_store']
  print('native_qualified_observation',qualified['ok']['runtime'],flush=True)
  store=r/'executions'/binding['attempt']/'home/.codex'
  saved=store.with_name('.codex-owned-saved');store.rename(saved);store.mkdir()
  try:
   replaced=request(r/'supervisor.sock',{'token':'native-profile-secret','action':'native_lookup','binding':binding})
   assert 'error' in replaced,'replacement native thread store must revoke qualification despite same execution UUID'
   print('native_store_replacement_refused',replaced,flush=True)
  finally:store.rmdir();saved.rename(store)
  cancelled=request(r/'supervisor.sock',{'token':'native-profile-secret','action':'cancel','binding':binding});print('native_profile_cancel_identity',cancelled,flush=True);assert cancelled['ok']['state']=='exited',cancelled
 finally:
  log=r/'executions'/binding['attempt']/'output.log'
  if log.exists(): print('native_owned_output',log.read_text(),flush=True)
  if p.poll() is None:p.kill()
  p.wait(timeout=5);listener.close()

"""MODEL_FREE_ACTUAL_PROTOCOL prelude: owned actual pin, no provider/model turn."""
import pathlib, tempfile, subprocess, json, selectors, hashlib, sys
binary=pathlib.Path('/data02/home/philip.yang/.codex/packages/standalone/releases/0.159.3-x86_64-unknown-linux-musl/bin/codex')
assert hashlib.sha256(binary.read_bytes()).hexdigest()=='8bf204b36a2f6dd0dab73aa2f639892e67ef9ac8befccb4a05b1496ebf25c479'
with tempfile.TemporaryDirectory(prefix='corenius-native-init-') as directory:
 r=pathlib.Path(directory); (r/'home/.codex').mkdir(parents=True);(r/'workspace').mkdir()
 argv=['/usr/bin/bwrap','--unshare-all','--new-session','--die-with-parent','--as-pid-1','--ro-bind','/usr','/usr','--symlink','usr/bin','/bin','--symlink','usr/lib','/lib','--symlink','usr/lib64','/lib64','--proc','/proc','--dev','/dev','--tmpfs','/tmp','--bind',str(r/'home'),'/home/worker','--bind',str(r/'workspace'),'/workspace','--ro-bind',str(binary),'/codex','--clearenv','--setenv','HOME','/home/worker','--setenv','CODEX_HOME','/home/worker/.codex','--setenv','PATH','/usr/bin:/bin','--chdir','/workspace','/codex','app-server','--stdio']
 p=subprocess.Popen(argv,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
 try:
  p.stdin.write(json.dumps({'id':1,'method':'initialize','params':{'clientInfo':{'name':'corenius_registry_red','version':'0.1'},'capabilities':{'experimentalApi':True}}})+'\n');p.stdin.flush()
  selector=selectors.DefaultSelector();selector.register(p.stdout,selectors.EVENT_READ);assert selector.select(10), 'native initialization deadline'
  line=p.stdout.readline()
  if not line: raise RuntimeError(p.stderr.read())
  response=json.loads(line);assert '0.159.3' in response['result']['userAgent'];print(json.dumps({'classification':'MODEL_FREE_ACTUAL_PROTOCOL','native_initialize':response,'private_network':True}),flush=True)
 finally:
  p.kill();p.wait(timeout=5)
  print(json.dumps({'owned_native_monitor':p.pid,'exit_code':p.returncode,'disposed':True}),flush=True)

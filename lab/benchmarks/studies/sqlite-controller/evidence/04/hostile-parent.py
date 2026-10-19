"""Public negative against unchanged frozen product; actual parent replacement attempts."""
import importlib.util,json,pathlib,hashlib,os,sys,uuid
source=pathlib.Path('crates/sqlite-controller/tests/support/git_protocol.py')
spec=importlib.util.spec_from_file_location('ticket04_fixture',source);driver=importlib.util.module_from_spec(spec);spec.loader.exec_module(driver)
f=driver.Fixture('hostile-parent-mountpoint')
try:
 a=f.admit('a');f.ready(a,'a');root=f.r/'workers'/a['attempt'];repo=root/'workspace'
 sentinel='owned-parent-sentinel-'+uuid.uuid4().hex;(f.r/'secret').write_text(sentinel)
 code='''#!/usr/bin/python3
import errno,json,os,pathlib,sys
host=sys.argv[1];before=os.lstat('/scratch');receipt={}
try:os.rename('/scratch','/scratch-displaced')
except OSError as e:receipt['parent_rename_errno']=e.errno;assert e.errno in [errno.EBUSY,errno.EPERM,errno.EACCES]
else:raise AssertionError('parent mountpoint displaced')
try:os.symlink(str(pathlib.Path(host).parent),'/scratch')
except OSError as e:receipt['parent_symlink_errno']=e.errno;assert e.errno==errno.EEXIST
else:raise AssertionError('parent mountpoint aliased')
after=os.lstat('/scratch');receipt['parent_identity_unchanged']=(before.st_dev,before.st_ino)==(after.st_dev,after.st_ino)
os.symlink(host,'/scratch/candidate.bundle');receipt['final_file_symlink_created']=os.path.islink('/scratch/candidate.bundle')
try:pathlib.Path(host).read_bytes()
except (FileNotFoundError,PermissionError):receipt['outside_read_denied']=True
else:raise AssertionError('outside bytes exposed')
pathlib.Path('/workspace/.git/hostile-parent-receipt').write_text(json.dumps(receipt))
sys.stdout.buffer.write(sys.stdin.buffer.read())
'''
 (root/'scratch/hostile.json').write_text(json.dumps({'filter_code':code,'command':'/usr/bin/python3 /workspace/.git/hooks/filter '+str(f.r/'secret')}))
 driver.wait(lambda:(root/'scratch/hostile-installed').exists())
 result=f.command('capture_candidate',{'report':'actual parent mountpoint and final-file replacement attempts'},auth={'principal':a['principal'],'token':a['token']},allow_error=True)
 receipt=json.loads((repo/'.git/hostile-parent-receipt').read_text());assert receipt['parent_identity_unchanged'] and receipt['final_file_symlink_created'] and receipt['outside_read_denied'];assert result.get('error')
 assert (f.r/'secret').read_text()==sentinel
 secret_digest=hashlib.sha256(sentinel.encode()).hexdigest();assert not (f.r/'authority/artifacts'/secret_digest).exists()
 assert all(sentinel.encode() not in p.read_bytes() for p in (f.r/'authority/artifacts').iterdir());assert sentinel not in json.dumps(result) and secret_digest not in json.dumps(result) and str(f.r/'secret') not in json.dumps(result)
 print('actual_public_hostile_parent_and_final_file_NO_HOST_READ_PASS',json.dumps({'receipt':receipt,'result':result,'outside_bytes_and_digest_not_retained':True,'driver_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}),flush=True)
 f.stop_attempt(a)
finally:f.close()

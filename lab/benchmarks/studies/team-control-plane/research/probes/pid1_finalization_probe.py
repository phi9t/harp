import os,sys,subprocess,tempfile,pathlib,time,json,select,signal
if len(sys.argv)>1:
 p=pathlib.Path(sys.argv[1]);f=open(p/'info','w');s=subprocess.Popen(['/usr/bin/bwrap','--unshare-user','--unshare-pid','--unshare-net','--die-with-parent','--new-session','--cap-drop','ALL','--disable-userns','--ro-bind','/','/','--bind',str(p),str(p),'--proc','/proc','--dev','/dev','--info-fd',str(f.fileno()),'--block-fd','0','--','/bin/sh','-c',f'(while :; do echo tick >> {p}/effects; sleep 0.01; done) & sleep 4'],pass_fds=(f.fileno(),),stdin=subprocess.PIPE);
 while not (p/'release').exists():time.sleep(.001)
 s.stdin.write(b'1');s.stdin.flush();s.stdin.close();s.wait();sys.exit()
def stat(pid):
 try:
  s=pathlib.Path(f'/proc/{pid}/stat').read_text();f=s.rsplit(')',1)[1].split();return {'state':f[0],'start':f[19]}
 except OSError as e:return {'errno':e.errno}
with tempfile.TemporaryDirectory(prefix='owned-pid1-') as d:
 p=pathlib.Path(d);c=subprocess.Popen([sys.executable,__file__,d]);fd=None
 try:
  end=time.monotonic()+2
  while time.monotonic()<end:
   try:i=json.loads((p/'info').read_text());pid=i['child-pid'];break
   except (OSError,ValueError,KeyError):time.sleep(.005)
  else:raise RuntimeError('no info')
  fd=os.pidfd_open(pid);pol=select.poll();pol.register(fd,select.POLLIN)
  original=stat(pid);original['nspid']=[l for l in pathlib.Path(f'/proc/{pid}/status').read_text().splitlines() if l.startswith('NSpid:')];ns=os.readlink(f'/proc/{pid}/ns/pid');assert original['nspid'][0].split()[-1]=='1';original['gated_effects_absent']=not (p/'effects').exists();(p/'release').touch();time.sleep(.04);before=(p/'effects').stat().st_size
  c.kill();rc=c.wait(timeout=1);rows=[];end=time.monotonic()+2
  while time.monotonic()<end:
   s=stat(pid)
   try:s['ns']=os.readlink(f'/proc/{pid}/ns/pid')
   except OSError as e:s['ns_errno']=e.errno
   s['pidfd_events']=pol.poll(0);rows.append(s)
   if s['pidfd_events']:break
   time.sleep(.001)
  after=(p/'effects').stat().st_size;time.sleep(.05);stable=after==(p/'effects').stat().st_size
  out={'kernel':os.uname().release,'controller_return':rc,'original':original,'namespace':ns,'effects_before':before,'effects_after_exit':after,'effects_stable_50ms':stable,'sequence':rows,'limitations':'No forced D-state, permission race or asynchronous external effect proof.'};print(json.dumps(out));assert rows[-1]['pidfd_events'] and stable
 finally:
  if c.poll() is None:c.kill();c.wait()
  if fd is not None:os.close(fd)

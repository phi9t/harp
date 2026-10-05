"""Authenticated real TCP gateway controls; no native runtime or paid provider."""
import hashlib,json,os,pathlib,signal,socket,struct,subprocess,sys,tempfile,time,tarfile

binary=pathlib.Path(sys.argv[1]);root=pathlib.Path(tempfile.mkdtemp(prefix='ws05-'))
env={'PATH':'/usr/bin:/bin','CORENIUS_OPERATOR_TOKEN':'operator-fixture-secret','CORENIUS_CLIENT_TOKEN':'operator-fixture-secret','CORENIUS_CLIENT_PRINCIPAL':'operator','CORENIUS_UI_TOKEN':'separate-ui-fixture-secret'}
owned=[]
def spawn(args,name):
 p=subprocess.Popen([str(binary),*args],env=env,stdout=(root/(name+'.stdout')).open('wb'),stderr=(root/(name+'.stderr')).open('wb'))
 owned.append((p,os.pidfd_open(p.pid),pathlib.Path(f'/proc/{p.pid}/stat').read_text(),args));return p
def exact(s,n):
 b=b''
 while len(b)<n:
  part=s.recv(n-len(b));assert part,'gateway disconnected before response';b+=part
 return b
def connect():
 s=socket.create_connection(address,timeout=3)
 s.sendall(b'GET / HTTP/1.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: YXV0aGVudGljYXRlZA==\r\nAuthorization: Bearer separate-ui-fixture-secret\r\n\r\n')
 response=b''
 while not response.endswith(b'\r\n\r\n'):response+=exact(s,1)
 assert response.startswith(b'HTTP/1.1 101');return s
def send(s,opcode,data=b'',masked=True,fin=True):
 with (root/'wire-calls.jsonl').open('a') as log:log.write(json.dumps({'opcode':opcode,'payload':data.hex(),'masked':masked,'fin':fin})+'\n')
 mask=b'abcd';n=len(data);header=bytes([(128 if fin else 0)|opcode,(128 if masked else 0)|(n if n<126 else 126)])
 if n>=126:header+=struct.pack('!H',n)
 s.sendall(header+(mask+bytes(b^mask[i%4] for i,b in enumerate(data)) if masked else data))
def receive(s):
 first,second=exact(s,2);assert second<128;n=second
 if n==126:n=struct.unpack('!H',exact(s,2))[0]
 if n==127:n=struct.unpack('!Q',exact(s,8))[0]
 return first,exact(s,n)
try:
 sock=root/'s';spawn([str(root/'db'),str(sock)],'controller')
 deadline=time.monotonic()+5
 while not sock.exists():assert time.monotonic()<deadline;time.sleep(.01)
 spawn(['--native-ui',str(sock),'p','127.0.0.1:0'],'gateway')
 while not (root/'gateway.stdout').read_text().strip():assert time.monotonic()<deadline;time.sleep(.01)
 host,port=(root/'gateway.stdout').read_text().strip().split()[-1].split(':');address=(host,int(port))
 with connect() as s:
  payload=b'exact\x00ping\xff';send(s,9,payload);reply=receive(s)
  (root/'ping-result.json').write_text(json.dumps({'sent':payload.hex(),'opcode':reply[0],'returned':reply[1].hex()}))
  assert reply==(0x8a,payload),reply
  send(s,10,b'unsolicited pong')
  send(s,1,json.dumps({'id':7,'method':'thread/list','params':{}}).encode())
  opcode,raw=receive(s);response=json.loads(raw);assert opcode==0x81 and response['id']==7
  # No native binding is configured: ordinary authenticated RPC reaches its existing refusal.
  assert 'error' in response,response
  send(s,8,struct.pack('!H',1000)+b'done');assert receive(s)==(0x88,struct.pack('!H',1000)+b'done')
 for opcode,data,masked,fin in [(9,b'a',False,True),(9,b'a'*126,True,True),(9,b'a',True,False),(8,b'x',True,True),(8,struct.pack('!H',1005),True,True),(8,struct.pack('!H',1000)+b'\xff',True,True)]:
  with connect() as s:
   send(s,opcode,data,masked,fin)
   try:result=s.recv(1)
   except ConnectionResetError:result=b''
   assert result==b'',(opcode,result)
 print('actual_authenticated_websocket_controls',json.dumps({'ping_echo':True,'pong_then_rpc':response,'close_echo':True,'malformed_refused':6,'native_configured':False}),flush=True)
finally:
 records=[]
 for p,fd,start,args in reversed(owned):
  if p.poll() is None:signal.pidfd_send_signal(fd,signal.SIGTERM)
  p.wait(timeout=5);os.close(fd);records.append({'pid':p.pid,'start_stat':start,'argv':args,'exit':p.returncode,'waited':True})
 (root/'resources.json').write_text(json.dumps(records,indent=2))
 destination=pathlib.Path(__file__).resolve().parents[4]/'.scratch/sqlite-controller/05-repair3/fixtures';destination.mkdir(parents=True,exist_ok=True)
 archive=destination/(root.name+'.tar.gz')
 with tarfile.open(archive,'w:gz') as out:out.add(root,arcname=root.name)
 print('retained_websocket_fixture',json.dumps({'root':str(root),'archive':str(archive),'sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'resources':records}),flush=True)

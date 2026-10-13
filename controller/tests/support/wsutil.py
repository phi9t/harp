import socket,os,base64,hashlib,struct,json
class WS:
 def __init__(self,s,client=True):self.s=s;self.client=client;self.f=s.makefile('rb')
 @classmethod
 def connect(cls,host,port,token=None,unix=None,path="/"):
  s=socket.socket(socket.AF_UNIX) if unix else socket.create_connection((host,port))
  if unix:s.connect(str(unix))
  s.settimeout(3)
  w=cls(s);key=base64.b64encode(os.urandom(16)).decode();auth=('Authorization: Bearer '+token+'\r\n') if token else ''
  s.sendall(f'GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n{auth}\r\n'.encode());first=w.f.readline()
  assert b'101' in first, first
  while w.f.readline()!=b'\r\n':pass
  return w
 @classmethod
 def accept(cls,s):
  w=cls(s,False);headers={};w.f.readline()
  while (ln:=w.f.readline())!=b'\r\n':
   k,v=ln.decode().split(':',1);headers[k.lower()]=v.strip()
  key=base64.b64encode(hashlib.sha1((headers['sec-websocket-key']+'258EAFA5-E914-47DA-95CA-C5AB0DC85B11').encode()).digest()).decode();s.sendall(f'HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {key}\r\n\r\n'.encode());return w
 def send(self,d,opcode=1):
  b=json.dumps(d).encode() if not isinstance(d,bytes) else d;n=len(b);head=bytes([128|opcode,(128 if self.client else 0)|(n if n<126 else 126 if n<65536 else 127)])
  if n>=126:head+=struct.pack('!H' if n<65536 else '!Q',n)
  if self.client:
   mask=os.urandom(4);head+=mask;b=bytes(v^mask[i%4] for i,v in enumerate(b))
  self.s.sendall(head+b)
 def recv(self):
  while True:
   h=self.f.read(2)
   if len(h)<2:raise EOFError()
   op=h[0]&15;n=h[1]&127
   if n==126:n=struct.unpack('!H',self.f.read(2))[0]
   if n==127:n=struct.unpack('!Q',self.f.read(8))[0]
   mask=self.f.read(4) if h[1]&128 else None;b=self.f.read(n)
   if mask:b=bytes(v^mask[i%4] for i,v in enumerate(b))
   if op==8:raise EOFError()
   if op==9:self.send(b,10);continue
   if op==1:return json.loads(b)
 def close(self):
  try:self.s.shutdown(socket.SHUT_RDWR)
  except OSError:pass
  self.s.close()
def port():
 s=socket.socket();s.bind(('127.0.0.1',0));p=s.getsockname()[1];s.close();return p

#!/usr/bin/python3
"""Trusted PID1 service launcher. Fixed Unix destinations; no host TCP fallback.
Plaintext native capabilities stay outside this namespace. This launcher and
native services receive only their SHA256 digest; control requests authenticate.
"""
import ctypes,fcntl,hashlib,json,os,pathlib,select,shutil,socket,struct,subprocess,threading,time,uuid,urllib.parse
HOME=pathlib.Path('/home/worker'); STORE=HOME/'.codex'; BRIDGE=pathlib.Path('/run/bridge')
KIND=os.environ['CORENIUS_NATIVE_KIND']; DIGEST=os.environ['CORENIUS_NATIVE_TOKEN_SHA256']
native_lock=threading.RLock();native_state={'child':None,'start':None}

def bootstrap():
    control=socket.socket();fcntl.ioctl(control,0x8914,struct.pack('16sh',b'lo',0x49));control.close()
    libc=ctypes.CDLL(None,use_errno=True)
    assert libc.prctl(38,1,0,0,0)==0 # PR_SET_NO_NEW_PRIVS
    assert libc.prctl(47,4,0,0,0)==0 # PR_CAP_AMBIENT_CLEAR_ALL
    class Header(ctypes.Structure):_fields_=[('version',ctypes.c_uint32),('pid',ctypes.c_int)]
    class Data(ctypes.Structure):_fields_=[('effective',ctypes.c_uint32),('permitted',ctypes.c_uint32),('inheritable',ctypes.c_uint32)]
    assert libc.capset(ctypes.byref(Header(0x20080522,0)),(Data*2)())==0
    status=pathlib.Path('/proc/self/status').read_text()
    for name in ('CapEff','CapPrm','CapInh','CapAmb'):
        assert int(next(line.split()[1] for line in status.splitlines() if line.startswith(name+':')),16)==0

def copy_stream(left,right):
    try:
        while True:
            ready,_,_=select.select([left,right],[],[],30)
            if not ready:continue
            for source in ready:
                data=source.recv(65536)
                if not data:return
                (right if source is left else left).sendall(data)
    except OSError:pass
    finally:left.close();right.close()

def serve(listener,connect):
    def loop():
        while True:
            left,_=listener.accept()
            try:right=connect()
            except OSError:left.close();continue
            threading.Thread(target=copy_stream,args=(left,right),daemon=True).start()
    threading.Thread(target=loop,daemon=True).start()

def unix_connect(path):
    stream=socket.socket(socket.AF_UNIX);stream.connect(str(path));return stream

def tcp_front(path,port=0):
    listener=socket.socket();listener.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1);listener.bind(('127.0.0.1',port));listener.listen(32)
    serve(listener,lambda:unix_connect(path));return listener.getsockname()[1]

def unix_front(path,port):
    if path.exists():path.unlink()
    listener=socket.socket(socket.AF_UNIX);listener.bind(str(path));os.chmod(path,0o600);listener.listen(32)
    def accept(stream):
        try:
            stream.settimeout(2);header=b''
            while b'\r\n\r\n' not in header:
                data=stream.recv(4096)
                if not data or len(header)+len(data)>16384:raise ValueError('native handshake limit')
                header+=data
            method,url,version=header.split(b'\r\n',1)[0].decode().split()
            expected=urllib.parse.parse_qs(urllib.parse.urlsplit(url).query).get('corenius_start')
            with native_lock:
                child=native_state['child']
                if method!='GET' or expected!=[native_state['start']] or child is None or child.poll() is not None:raise ValueError('native incarnation fence')
                peer=socket.create_connection(('127.0.0.1',port),timeout=2)
            peer.sendall(header);stream.settimeout(None);peer.settimeout(None)
            copy_stream(stream,peer)
        except Exception:stream.close()
    def loop():
        while True:
            stream,_=listener.accept();threading.Thread(target=accept,args=(stream,),daemon=True).start()
    threading.Thread(target=loop,daemon=True).start()

bootstrap();STORE.mkdir(exist_ok=True)
shutil.copyfile('/run/codex/config.toml',STORE/'config.toml')
if KIND=='app':tcp_front('/run/provider.sock',8777)
port=8788 if KIND=='app' else 8789
unix_front(BRIDGE/'native.sock',port)
bridges={}; bridge_lock=threading.Lock()

def control_client(stream):
    try:
        stream.settimeout(2);file=stream.makefile('rb');raw=file.readline(65537)
        if len(raw)>65536:raise ValueError('control framing limit')
        value=json.loads(raw)
        if set(value)!={'token','action','attempt'} or hashlib.sha256(value['token'].encode()).hexdigest()!=DIGEST:raise ValueError('control authentication refused')
        attempt=value['attempt'];assert str(uuid.UUID(attempt))==attempt
        if KIND!='app' or value['action']!='executor_bridge':raise ValueError('control action refused')
        own=os.environ.get('CORENIUS_NATIVE_OWN_EXECUTOR')
        if own and attempt!=own:raise ValueError('executor scope refused')
        path=pathlib.Path('/run/executors')/attempt/'native.sock'
        if not path.is_socket():raise ValueError('qualified executor socket absent')
        with bridge_lock:
            if attempt not in bridges:bridges[attempt]=tcp_front(path)
            reply={'ok':{'url':'ws://127.0.0.1:'+str(bridges[attempt])}}
        stream.sendall(json.dumps(reply).encode()+b'\n')
    except Exception:
        try:stream.sendall(b'{"error":"native scoped bridge refused"}\n')
        except OSError:pass
    finally:stream.close()

if KIND=='app':
    listener=socket.socket(socket.AF_UNIX);path=BRIDGE/'control.sock'
    if path.exists():path.unlink()
    listener.bind(str(path));os.chmod(path,0o600);listener.listen(16)
    def controls():
        while True:
            stream,_=listener.accept();threading.Thread(target=control_client,args=(stream,),daemon=True).start()
    threading.Thread(target=controls,daemon=True).start()

while True:
    args=['/run/codex/codex','app-server','--strict-config'] if KIND=='app' else ['/run/codex/codex','exec-server']
    args+=['--listen','ws://127.0.0.1:'+str(port),'--ws-auth','capability-token','--ws-token-sha256',DIGEST]
    with native_lock:
        child=subprocess.Popen(args,stdin=subprocess.DEVNULL)
        stat=pathlib.Path(f'/proc/{child.pid}/stat').read_text().rsplit(') ',1)[1].split()
        native_state.update(child=child,start=stat[19])
    try:
        identity={'kind':KIND,'pid':child.pid,'start_ticks':stat[19],'port':port,'capabilities_dropped':True}
        temporary=pathlib.Path('/scratch/.native-runtime.tmp');temporary.write_text(json.dumps(identity));temporary.replace('/scratch/native-runtime.json')
        print(json.dumps({'native_service':identity}),flush=True)
        child.wait()
    finally:
        if child.poll() is None:child.kill();child.wait()
        with native_lock:native_state.update(child=None,start=None)
    # Native service restart is observable via its independently checked start identity.
    time.sleep(.1)

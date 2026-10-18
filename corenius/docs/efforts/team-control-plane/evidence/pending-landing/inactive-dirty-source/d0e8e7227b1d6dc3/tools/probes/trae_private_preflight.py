#!/usr/bin/env python3
"""Owned synthetic-only Trae prerequisite probe; never inherits a runtime home.

Public evidence contains hashes and selected origins, not private stores/config.
An outer private PID/net namespace denies all host/provider egress. No adapter
adoption or live provider/recovery guarantee is implied by this preflight.
"""
import argparse
import base64
import struct
import hashlib
import http.server
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import threading
import time

# This executed envelope spent all four server incarnations. Further launch
# needs a separately reviewed envelope and budget enforcement; never reset here.
LAUNCH_ENABLED = False

BINARY = Path('/data02/home/philip.yang/.local/share/traex/releases/0.207.1-linux-x86_64.Cduewb/traex')
BINARY_SHA = '9f5531a1599cd7d81df35a9b6a7568f6652fbf58141d33da9853a62046caf8c2'
FEATURES = ['memories', 'goals', 'multi_agent', 'multi_agent_v2', 'code_mode_host', 'plugins', 'plugin_hooks', 'hooks']


def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def proc_identity(pid):
    proc = Path(f'/proc/{pid}')
    stat = (proc/'stat').read_text().rsplit(')', 1)[1].split()
    return {'pid':pid, 'start_ticks':int(stat[19]), 'pid_namespace':os.readlink(proc/'ns/pid'), 'network_namespace':os.readlink(proc/'ns/net')}


def members(namespace):
    result = []
    for p in Path('/proc').iterdir():
        if p.name.isdigit():
            try:
                if os.readlink(p/'ns/pid') == namespace:
                    result.append(proc_identity(int(p.name)))
            except (FileNotFoundError, ProcessLookupError, PermissionError):
                pass
    return result


def capsule(fixture):
    return ['/usr/bin/bwrap', '--unshare-all', '--die-with-parent', '--new-session', '--clearenv',
            '--ro-bind', '/usr', '/usr', '--ro-bind', '/lib', '/lib', '--ro-bind', '/lib64', '/lib64',
            '--ro-bind', str(BINARY), '/runtime/traex', '--ro-bind', str(Path(__file__).resolve()), '/preflight.py',
            '--bind', str(fixture), '/fixture', '--proc', '/proc', '--dev', '/dev', '--tmpfs', '/tmp',
            '--setenv', 'PATH', '/usr/bin:/bin', '--setenv', 'TRAE_HOME', '/fixture/store',
            '--setenv', 'CODEX_OSS_BASE_URL', 'http://127.0.0.1:1234/v1', '--setenv', 'CODEX_OSS_PORT', '1234',
            '--chdir', '/fixture/repository', '--', '/usr/bin/python3', '/preflight.py', '--inside']


class Synthetic(http.server.BaseHTTPRequestHandler):
    lock = threading.Lock()
    accepted = 0
    records = []
    maximum = 6

    def log_message(self, *args):
        pass

    def handle_request(self):
        length = int(self.headers.get('Content-Length', '0'))
        if length > 131072:
            self.send_error(413)
            return
        body = self.rfile.read(length)
        with self.lock:
            allowed = self.accepted < self.maximum
            if allowed:
                type(self).accepted += 1
            self.records.append({'method':self.command, 'path':self.path, 'body_bytes':length,
                                 'body_sha256':hashlib.sha256(body).hexdigest(), 'accepted':allowed,
                                 'json_keys':sorted(json.loads(body)) if body else []})
            write('/fixture/public/provider-requests.json', self.records)
        if not allowed:
            self.send_error(429, 'owned synthetic request budget exhausted')
            return
        if self.command == 'GET':
            data = json.dumps({'object':'list','data':[{'id':'synthetic-preflight','object':'model','owned_by':'owned-fixture'}]}).encode()
            self.send_response(200); self.send_header('Content-Type','application/json'); self.send_header('Content-Length',str(len(data))); self.end_headers(); self.wfile.write(data)
            return
        # Initial protocol observation deliberately refuses execution rather than
        # claiming an unverified streaming contract. No upstream/provider exists.
        self.send_error(501, 'synthetic streaming protocol not yet exercised')

    do_GET = handle_request
    do_POST = handle_request


class Rpc:
    def __init__(self, path):
        self.socket = socket.socket(socket.AF_UNIX)
        self.socket.settimeout(10)
        self.socket.connect(path)
        key = base64.b64encode(os.urandom(16)).decode()
        self.socket.sendall((f'GET / HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n').encode())
        self.buffer = b''
        while b'\r\n\r\n' not in self.buffer:
            part = self.socket.recv(4096)
            if not part: raise RuntimeError('Unix WebSocket handshake ended')
            self.buffer += part
        headers, self.buffer = self.buffer.split(b'\r\n\r\n',1)
        if not headers.startswith(b'HTTP/1.1 101'): raise RuntimeError('Unix WebSocket upgrade refused')
        self.sequence = 0
        self.events = []

    def exact(self, size):
        while len(self.buffer)<size:
            part=self.socket.recv(size-len(self.buffer))
            if not part: raise RuntimeError('RPC transport ended')
            self.buffer+=part
        value,self.buffer=self.buffer[:size],self.buffer[size:]
        return value

    def send(self, message):
        payload=json.dumps(message).encode();mask=os.urandom(4)
        length=len(payload)
        header=bytes([0x81,0x80|length]) if length<126 else bytes([0x81,0xfe])+struct.pack('!H',length)
        self.socket.sendall(header+mask+bytes(v^mask[i%4] for i,v in enumerate(payload)))

    def receive(self):
        first,second=self.exact(2);length=second&127
        if length==126: length=struct.unpack('!H',self.exact(2))[0]
        if length==127: length=struct.unpack('!Q',self.exact(8))[0]
        if length>1048576: raise RuntimeError('RPC frame exceeds bound')
        mask=self.exact(4) if second&128 else None
        payload=self.exact(length)
        if mask: payload=bytes(v^mask[i%4] for i,v in enumerate(payload))
        if first&15!=1: raise RuntimeError(f'unsupported RPC frame opcode {first&15}')
        return json.loads(payload)

    def call(self, method, params):
        self.sequence += 1
        self.send({'id':self.sequence,'method':method,'params':params})
        while True:
            message=self.receive()
            if message.get('id') == self.sequence:
                if 'error' in message: raise RuntimeError(f'{method}: {message["error"]}')
                return message['result']
            self.events.append(message.get('method'))


def inside():
    out = Path('/fixture/public')
    out.mkdir()
    write(out/'identity.json',proc_identity(os.getpid()))
    denied = []
    for endpoint in [('1.1.1.1',443),('127.0.0.1',11434),('169.254.169.254',80)]:
        s = socket.socket(); s.settimeout(1); code = s.connect_ex(endpoint); s.close()
        denied.append({'endpoint':endpoint,'errno':code})
        assert code != 0, 'unexpected alternate egress'
    write(out/'network-denial.json',denied)
    assert not Path('/data02').exists() and not Path('/home').exists()
    assert not Path('/etc').exists(), 'shared config access'
    server = http.server.ThreadingHTTPServer(('127.0.0.1',1234),Synthetic)
    thread = threading.Thread(target=server.serve_forever,daemon=True); thread.start()
    args = ['/runtime/traex','--oss','--local-provider','lmstudio','-m','synthetic-preflight']
    for feature in FEATURES:
        args += ['-c',f'features.{feature}=false']
    args += ['-c','model_provider="lmstudio"','-c','mcp_servers={}','-c','analytics.enabled=false','app-server','--listen','unix:///fixture/server.sock']
    write(out/'launch.json',{'argv':args,'environment_names':sorted(os.environ),'shared_home_mounted':False})
    with Path('/fixture/server.stderr').open('wb') as stderr:
        child = subprocess.Popen(args,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=stderr)
        try:
            deadline = time.monotonic()+10
            while not Path('/fixture/server.sock').exists():
                if child.poll() is not None: raise RuntimeError(f'private server exit {child.returncode}')
                if time.monotonic()>deadline: raise TimeoutError('private socket readiness')
                time.sleep(.05)
            rpc = Rpc('/fixture/server.sock')
            initialized = rpc.call('initialize',{'clientInfo':{'name':'owned-preflight','version':'1'},'capabilities':{'experimentalApi':True}})
            write(out/'initialize.json',initialized)
            assert initialized['codexHome']=='/fixture/store', 'store identity mismatch'
            rpc.send({'method':'initialized'})
            config = rpc.call('config/read',{'includeLayers':True,'cwd':'/fixture/repository'})
            write(out/'config-origins.json',{'origins':config.get('origins'),'layer_names':[v.get('name') for v in config.get('layers',[])], 'model_provider':config.get('config',{}).get('model_provider')})
            catalog = rpc.call('model/list', {'limit':20})
            choices = catalog.get('data', [])
            models = [item.get('model', item.get('id')) for item in choices]
            write(out/'model-list.json', {'models':models,'next_cursor':catalog.get('nextCursor')})
            if not models: raise RuntimeError('no advertised local model metadata; synthetic routing unsupported without another source')
            model = models[0]
            started = rpc.call('thread/start',{'cwd':'/fixture/repository','model':model,'modelProvider':'lmstudio','approvalPolicy':'never','sandbox':'read-only','experimentalRawEvents':False,'baseInstructions':'Respond only with owned synthetic preflight text. Do not use tools.'})
            write(out/'thread-start.json',{'thread_id':started['thread']['id'],'modelProvider':started.get('modelProvider')})
            rpc.call('turn/start',{'threadId':started['thread']['id'],'input':[{'type':'text','text':'Return synthetic preflight.'}]})
            time.sleep(2)
            write(out/'result.json',{'provider_requests':len(Synthetic.records),'events':rpc.events,'verdict':'routing observed; streaming unsupported until measured'})
            assert Synthetic.records, 'no synthetic endpoint discovery/request'
        finally:
            if child.poll() is None:
                child.terminate()
                try: child.wait(timeout=3)
                except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=3)
            write(out/'server-terminal.json',{'pid':child.pid,'waited':True,'exit':child.returncode})
            server.shutdown(); server.server_close()


def host(fixture):
    if not LAUNCH_ENABLED:
        raise RuntimeError('preflight envelope exhausted: four incarnations; further launch refused')
    fixture = Path(fixture).resolve()
    fixture.mkdir(mode=0o700)
    (fixture/'repository').mkdir(); (fixture/'store').mkdir(mode=0o700)
    assert hashlib.sha256(BINARY.read_bytes()).hexdigest()==BINARY_SHA
    write(fixture/'binary.json',{'path':str(BINARY),'sha256':BINARY_SHA})
    command = capsule(fixture)
    child = subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env={'PATH':'/usr/bin:/bin'})
    namespace = None
    deadline = time.monotonic()+10
    while child.poll() is None and time.monotonic()<deadline:
        p = fixture/'public/identity.json'
        if p.exists(): namespace=json.loads(p.read_text())['pid_namespace']; break
        time.sleep(.02)
    try:
        stdout,stderr=child.communicate(timeout=45)
    except subprocess.TimeoutExpired:
        child.kill(); stdout,stderr=child.communicate(timeout=5)
    (fixture/'capsule.stdout').write_bytes(stdout);(fixture/'capsule.stderr').write_bytes(stderr)
    remaining=members(namespace) if namespace else None
    write(fixture/'terminal.json',{'waited':True,'exit':child.returncode,'pid_namespace':namespace,'remaining_members':remaining,'disposition':'stopped' if namespace and not remaining else 'quarantined','fixture_retained':True})
    if not namespace or remaining: raise RuntimeError('incomplete namespace stop proof; retain quarantine')
    return child.returncode


if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--inside',action='store_true');parser.add_argument('--fixture');args=parser.parse_args()
    if args.inside:
        if not LAUNCH_ENABLED:
            raise RuntimeError('preflight envelope exhausted; direct inner launch refused')
        inside()
    elif args.fixture: sys.exit(host(args.fixture))
    else: parser.error('--fixture required')

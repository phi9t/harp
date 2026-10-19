#!/usr/bin/env python3
"""Deterministic public controller qualification against the exact real Kata daemon.
No models, production tasks, Kata SQL, or fabricated HTTP successes.
All fixture resources and failure evidence are retained in the assigned checkout.
"""
import base64
import hashlib
from http import client as httpclient
from http import server as httpserver
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.request

REPO = Path(__file__).resolve().parents[4]
PIN = Path(os.environ.get('KATA_PINNED_BINARY', '/data02/home/philip.yang/workspace/corenius-codex-kata-evaluation/.scratch/team-control-plane/kata17/kata-pinned'))
SHA = '8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6'
assert hashlib.sha256(PIN.read_bytes()).hexdigest() == SHA
ROOT = Path(tempfile.mkdtemp(prefix='k3-', dir=REPO / '.scratch'))
ROOT.chmod(0o700)
records = []
api_trace = open(ROOT/'public-api.jsonl','a')
def scrub(value):
    if isinstance(value,dict):
        return {k:('[fixture secret redacted]' if k in ('token','plaintext') else scrub(v)) for k,v in value.items()}
    if isinstance(value,list):
        return [scrub(v) for v in value]
    return value
processes = []
logs = []
bootstrap = secrets.token_hex(32)
operator = secrets.token_hex(32)
katadir = ROOT / 'kata'
katadir.mkdir(mode=0o700)
(katadir / 'config.toml').write_text('[auth]\ntoken="' + bootstrap + '"\nrequire_token_identity=true\n')
(katadir / 'config.toml').chmod(0o600)
def port():
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]
kport = port()
origin = 'http://127.0.0.1:' + str(kport)
kataenv = {k:v for k,v in os.environ.items() if not k.startswith('KATA_')}
kataenv['KATA_HOME'] = str(katadir)

def start_kata():
    log = open(ROOT / ('kata-' + str(len(processes)) + '.log'), 'ab')
    logs.append(log)
    p = subprocess.Popen([str(PIN), 'daemon', 'start', '--foreground', '--listen', '127.0.0.1:' + str(kport)], cwd=ROOT, env=kataenv, stdout=log, stderr=log, start_new_session=True)
    processes.append(p)
    identity(p,'kata')
    ready(p, lambda: http('GET', '/health', token=None))
    return p

def identity(p,kind):
    p.fixture_start_ticks = Path('/proc/'+str(p.pid)+'/stat').read_text().rsplit(')',1)[1].split()[19]
    records.append({'name':'owned-process-start','kind':kind,'pid':p.pid,'start_ticks':p.fixture_start_ticks,'boot_id':Path('/proc/sys/kernel/random/boot_id').read_text().strip()})

def stop(p):
    if p.poll() is None:
        assert Path('/proc/'+str(p.pid)+'/stat').read_text().rsplit(')',1)[1].split()[19] == p.fixture_start_ticks
        p.send_signal(signal.SIGTERM)
        try:
            p.wait(timeout=5)
        except subprocess.TimeoutExpired:
            p.kill()
            p.wait(timeout=5)
    records.append({'name':'owned-process-terminal','pid':p.pid,'exit_code':p.returncode})

def ready(p, probe):
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        assert p.poll() is None, ('startup exit', p.returncode)
        try:
            probe()
            return
        except (OSError, urllib.error.URLError, TimeoutError):
            time.sleep(.03)
    raise AssertionError('startup deadline')

def http(method, path, body=None, token=bootstrap, headers=None):
    h = {'Content-Type':'application/json'}
    if token:
        h['Authorization'] = 'Bearer ' + token
    h.update(headers or {})
    req = urllib.request.Request(origin + '/api/v1' + path, data=None if body is None else json.dumps(body).encode(), method=method, headers=h)
    try:
        with urllib.request.urlopen(req, timeout=3) as r:
            return json.load(r)
    except urllib.error.HTTPError as error:
        diagnostic = error.read().decode()
        records.append({'name':'actual-kata-http-error','path':path,'status':error.code,'body':diagnostic})
        raise AssertionError(diagnostic) from error

class Proxy(httpserver.BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    drop = None
    offline = False
    def log_message(self, *args):
        pass
    def serve(self):
        body = self.rfile.read(int(self.headers.get('Content-Length','0')))
        if Proxy.offline:
            self.connection.close()
            self.close_connection = True
            return
        c = httpclient.HTTPConnection('127.0.0.1', kport, timeout=3)
        try:
            c.request(self.command, self.path, body, {k:v for k,v in self.headers.items() if k.lower() not in ('host','connection')})
            r = c.getresponse()
            data = r.read()
            if self.command == 'POST' and Proxy.drop and self.path.endswith(Proxy.drop):
                records.append({'name':'actual-effect-reply-dropped','path':self.path,'status':r.status})
                Proxy.drop = None
                self.connection.close()
                self.close_connection = True
                return
            self.send_response(r.status)
            self.send_header('Content-Type','application/json')
            self.send_header('Content-Length',str(len(data)))
            self.send_header('Connection','close')
            self.end_headers()
            self.wfile.write(data)
        except (OSError, httpclient.HTTPException):
            self.connection.close()
        finally:
            c.close()
            self.close_connection = True
    do_GET = serve
    do_POST = serve

proxy = httpserver.ThreadingHTTPServer(('127.0.0.1',0),Proxy)
thread = threading.Thread(target=proxy.serve_forever,daemon=True)
thread.start()
csock = ROOT / 'controller.sock'
config = ROOT / 'controller-kata.json'
ctrl = None

def api(value):
    with socket.socket(socket.AF_UNIX) as s:
        s.settimeout(4)
        s.connect(str(csock))
        s.sendall((json.dumps(value)+'\n').encode())
        reply = json.loads(s.makefile().readline())
    api_trace.write(json.dumps({'request':scrub(value),'response':scrub(reply)})+'\n')
    api_trace.flush()
    if 'error' in reply:
        raise AssertionError(reply['error'])
    return reply['ok']

auth = {'principal':'operator','token':operator}
def query(kind='snapshot', args=None, actor=None):
    return api({'auth':actor or auth,'project':'p','query':{'kind':kind,'args':args or {}}})
def cmd(kind,args,key=None,actor=None,revision=None):
    req = {'auth':actor or auth,'project':'p','command':{'kind':kind,'args':args,'idempotency_key':key or secrets.token_hex(8),'expected_revision':query()['revision'] if revision is None else revision}}
    return api(req)
def denied(kind,args,actor,match='authorization'):
    try:
        cmd(kind,args,actor=actor)
    except AssertionError as e:
        assert match in str(e), str(e)
        records.append({'name':'refused-'+kind,'error':str(e)})
    else:
        raise AssertionError('forbidden command succeeded: ' + kind)
def reconcile():
    return api({'auth':auth,'project':'p','kata':{'kind':'reconcile'}})
def start_controller():
    log = open(ROOT / ('controller-'+str(len(processes))+'.log'),'ab')
    logs.append(log)
    env = os.environ.copy()
    env.update(CORENIUS_OPERATOR_TOKEN=operator,CORENIUS_KATA_CONFIG=str(config),CORENIUS_TEST_MANUAL_RUNTIME='1')
    for name in ('CORENIUS_CODEX_CONFIG','CORENIUS_SUPERVISOR_SOCKET','CORENIUS_SUPERVISOR_TOKEN'):
        env.pop(name,None)
    p = subprocess.Popen([os.environ['CONTROLLER_BINARY'],str(ROOT/'state.db'),str(csock)],env=env,cwd=ROOT,stdout=log,stderr=log)
    processes.append(p)
    identity(p,'controller')
    ready(p, lambda: api({'auth':auth,'project':'p','query':{'kind':'storage_configuration','args':{}}}))
    return p

try:
    kata = start_kata()
    health = http('GET','/health',token=None)
    assert health['api_schema_version'] == '0.24.0'
    token = http('POST','/tokens',{'actor':'corenius-fixture-controller','name':'controller-only'})['plaintext']
    (ROOT/'kata.token').write_text(token)
    (ROOT/'kata.token').chmod(0o600)
    project = http('POST','/projects',{'name':'ticket03-'+ROOT.name},token=token)['project']
    project_path = '/projects/'+str(project['id'])
    issue = http('POST',project_path+'/issues',{'title':'pinned intent','body':'Implement task.md, constrained by spec.md'},token=token)['issue']
    other = http('POST',project_path+'/issues',{'title':'peer task','body':'second bounded assignment'},token=token)['issue']
    events = http('GET',project_path+'/events?after_id=0',token=token)['events']
    instance = next(e['origin_instance_uid'] for e in events if e['type']=='issue.created' and e['issue_uid']==issue['uid'])
    ipath = project_path+'/issues/'+issue['uid']
    repo = ROOT/'input'
    repo.mkdir()
    def git(*args):
        return subprocess.check_output(['git','-C',str(repo),*args],text=True).strip()
    git('init','-b','fixture')
    git('config','user.name','Controller qualification')
    git('config','user.email','fixture@example.invalid')
    (repo/'spec.md').write_text('Acceptance: exact immutable intent and no closure authority.\n')
    (repo/'task.md').write_text('Read spec.md; implement pinned request.\n')
    git('add','.')
    git('commit','-m','fixture source')
    base = git('rev-parse','HEAD')
    (repo/'task.md').write_text('Read spec.md; explicit uncommitted source snapshot.\n')
    config.write_text(json.dumps({'schema_migration':'kata-v1','controller_id':'fixture03','actor':'corenius-fixture-controller','api_schema_version':'0.24.0','origin_instance_uid':instance,'origin':'http://127.0.0.1:'+str(proxy.server_port),'token_file':str(ROOT/'kata.token'),'project_id':project['id'],'project_uid':project['uid'],'repositories':{'fixture-input':str(repo)}}))
    config.chmod(0o600)
    ctrl = start_controller()
    cmd('create_project',{'name':'p','base':base},key='project',revision=0)
    files = {'storage':'fixture-input','mode':'snapshot','paths':['spec.md','task.md']}
    creation = {'task':'a','item':issue['uid'],'expected_source_revision':issue['revision'],'source_files':files}
    first = cmd('create_kata_task',creation,key='capture-a')
    assert first['result']['source']['work_ref']['deployment'] == instance
    assert first['result']['source']['files']['dirty'] is True
    artifacts = first['result']['source']['files']['artifacts']
    assert len(artifacts) == 2
    expected_files = {'spec.md':b'Acceptance: exact immutable intent and no closure authority.\n','task.md':b'Read spec.md; explicit uncommitted source snapshot.\n'}
    for artifact in artifacts:
        exact = base64.b64decode(artifact['content_base64'])
        assert exact == expected_files[artifact['path']]
        assert artifact['sha256'] == hashlib.sha256(exact).hexdigest()
    assert cmd('create_kata_task',creation,key='capture-a',revision=first['revision']-1) == first
    cmd('create_kata_task',{'task':'b','item':other['uid'],'expected_source_revision':other['revision'],'source_files':{'storage':'fixture-input','mode':'git','commit':base,'paths':['spec.md','task.md']}},key='capture-b')
    denied('create_task',{'task':'forged','work_ref':{'deployment':instance,'project':project['uid'],'item':'invented'},'source_revision':'rev-1','brief':'unverified caller content'},auth,match='qualified Kata')
    third = http('POST',project_path+'/issues',{'title':'changed-before-admission'},token=token)['issue']
    cmd('create_kata_task',{'task':'c','item':third['uid'],'expected_source_revision':third['revision'],'source_files':files})
    thirdpath = project_path+'/issues/'+third['uid']
    http('PATCH',thirdpath,{'body':'changed after capture before assignment'},token=token)
    denied('admit',{'task':'c'},auth,match='source amendment')
    assert any(v['task']=='c' and v['proposed'] for v in query('participant_context')['sources'])
    cmd('refresh_kata_source',{'task':'c'})
    git_source = query('participant_context')['sources'][1]['admitted']['files']
    assert git_source['commit'] == base and git_source['dirty'] is False
    assert base64.b64decode(git_source['artifacts'][1]['content_base64']) == b'Read spec.md; implement pinned request.\n'
    admitted = cmd('admit',{'task':'a'},key='admit-a')
    wa = {k:admitted['result'][k] for k in ('principal','token')}
    admitted_b = cmd('admit',{'task':'b'},key='admit-b')
    wb = {k:admitted_b['result'][k] for k in ('principal','token')}
    pinned = query('participant_context',actor=wa)['sources'][0]['admitted']
    assert token not in json.dumps(query('participant_context',actor=wa))
    assert len(query('participant_context',actor=wa)['tasks']) == 1
    assert pinned['revision'] == issue['revision']
    (ROOT/'admitted-source.json').write_text(json.dumps(pinned,indent=2)+'\n')
    for kind,args in [('admit',{'task':'b'}),('close_kata_task',{'task':'a'}),('create_kata_task',creation),('replace_lead',{}),('grant_worker',{'task':'b'}),('change_dependencies',{'task':'a'}),('report_progress',{'task':'b','text':'spoof'})]:
        denied(kind,args,wa)
    try:
        api({'auth':wa,'project':'p','kata':{'kind':'reconcile'}})
    except AssertionError as error:
        assert 'authorization' in str(error)
    else:
        raise AssertionError('worker obtained unrestricted Kata reconciliation')
    denied('send_message',{'recipient':wb['principal'],'sender':'operator','text':'spoof'},wa,match='sender')
    q = cmd('ask_question',{'task':'a','text':'Which acceptance evidence?'},actor=wa)['result']['question']
    proposal = cmd('propose_followup',{'task':'a','text':'Add a narrowly scoped validation followup'},actor=wa)['result']['proposal']
    message = cmd('send_message',{'recipient':wb['principal'],'text':'I am using the pinned acceptance contract'},actor=wa)['result']['message']
    cmd('ack_message',{'message':message},actor=wb)
    cmd('record_decision',{'text':'Keep intent immutable; task closure cannot accept candidates'})
    cmd('observe_attempt',{'attempt':admitted['result']['attempt'],'state':'running','observation':'storage fixture observation; runtime cleanup still pending'})
    cmd('sync_kata_task',{'task':'a'},key='sync-blocked')
    Proxy.drop = '/metadata'
    reconcile()
    assert any(e['state']=='unknown' for e in query('participant_context')['synchronization']['effects'])
    reconcile()
    Proxy.drop = '/comments'
    cmd('sync_kata_task',{'task':'a'},key='sync-lost-comment')
    reconcile()
    reconcile()
    remote = http('GET',ipath,token=token)
    summary = remote['issue']['metadata']['corenius.execution']
    assert summary['state']=='blocked'
    comments = remote['comments']
    ids = [c['body'] for c in comments if '[corenius operation ' in c['body']]
    assert len(ids)==len(set(ids)) and ids
    assert all(c['author']=='corenius-fixture-controller' for c in comments)
    synced = query('participant_context')['sources'][0]
    assert synced['acknowledged_kata_revision'] > pinned['revision']
    refreshed = cmd('refresh_kata_source',{'task':'a'},key='own-projection-refresh')['result']
    assert refreshed['disposition']=='unchanged', refreshed
    assert query('participant_context')['sources'][0]['admitted'] == pinned
    cmd('answer_question',{'question':q,'text':'Retain exact source and public command evidence'})
    stop(kata)
    cmd('report_progress',{'task':'a','text':'daemon offline; keep journaled sync'},actor=wa)
    cmd('sync_kata_task',{'task':'a'},key='offline-sync')
    reconcile()
    offline = query('participant_context')
    assert any(e['state']=='unknown' for e in offline['synchronization']['effects'])
    stop(ctrl)
    ctrl = start_controller()
    kata = start_kata()
    reconcile()
    assert all(e['state']=='confirmed' for e in query('participant_context')['synchronization']['effects'])
    assert http('GET',ipath,token=token)['issue']['metadata']['corenius.execution']['state']=='running'
    (repo/'spec.md').write_text('Changed external acceptance is a proposal.\n')
    http('PATCH',ipath,{'body':'externally amended requirement'},token=token)
    amendment = cmd('refresh_kata_source',{'task':'a'},key='amendment')['result']
    assert amendment['disposition']=='proposed', amendment
    duplicate = cmd('refresh_kata_source',{'task':'a'},key='amendment',revision=query()['revision']-1)
    assert duplicate['result']==amendment
    ctx = query('participant_context')
    assert ctx['sources'][0]['admitted']==pinned
    assert ctx['sources'][0]['current'] != pinned
    assert ctx['sources'][0]['proposed']
    http('PATCH',project_path+'/issues/'+other['uid'],{'body':'changed before next admission'},token=token)
    # Initial stale revision capture cannot quietly admit current text.
    denied('create_kata_task',{'task':'stale','item':other['uid'],'expected_source_revision':1,'source_files':files},auth,match='source revision')
    lead = cmd('replace_lead',{})['result']
    lead_auth = {k:lead[k] for k in ('principal','token')}
    recovered = query('participant_context',actor=lead_auth)
    assert recovered['questions'][0]['answer']
    assert recovered['followup_proposals'][0]['id']==proposal
    assert recovered['messages'][0]['acknowledged'] is True
    assert recovered['decisions'] and len(recovered['attempts'])==2
    assert recovered['runtime_cleanup']['qualified'] is False
    assert recovered['sources'][0]['admitted']==pinned
    # External closure is an amendment, never controller acceptance or observed stop.
    http('POST',ipath+'/actions/close',{'retry_protocol':'close-v1','reason':'done','message':'External fixture closure only; no Corenius acceptance or execution termination is implied.','evidence':[{'type':'test','command':'true'}]},token=token,headers={'Idempotency-Key':'external-fixture-close'})
    assert cmd('refresh_kata_source',{'task':'a'})['result']['disposition']=='proposed'
    assert query()['tasks'][0]['state']=='active'
    assert query()['attempts'][0]['state']=='running'
    # External visible-projection tampering is drift; never overwrite it blindly.
    r = http('GET',ipath,token=token)['issue']
    http('POST',ipath+'/metadata',{'patch':{'corenius.execution':{'controller':'external'}},'guard':{'key':'corenius.execution','if_value':json.dumps(r['metadata']['corenius.execution'])}},token=token,headers={'If-Match':'\"rev-'+str(r['revision'])+'\"'})
    cmd('sync_kata_task',{'task':'a'})
    reconcile()
    driftctx = query('participant_context',actor=lead_auth)
    assert driftctx['sources'][0]['drift']
    assert any(e['state']=='unknown' for e in driftctx['synchronization']['effects'])
    (ROOT/'reconstructed-context.json').write_text(json.dumps(driftctx,indent=2)+'\n')
    # Source/project/deployment mismatch must fail closed, with no outgoing effects.
    Proxy.offline = True
    denied('refresh_kata_source',{'task':'a'},auth,match='Kata')
    Proxy.offline = False
    stop(kata)
    alternate = ROOT/'alternate-kata'
    alternate.mkdir(mode=0o700)
    (alternate/'config.toml').write_text('[auth]\ntoken=\"'+token+'\"\nrequire_token_identity=false\n')
    (alternate/'config.toml').chmod(0o600)
    kataenv['KATA_HOME'] = str(alternate)
    kata = start_kata()
    denied('refresh_kata_source',{'task':'a'},auth,match='deployment identity')
    assert query()['tasks'][0]['state']=='active'
    records.append({'name':'alternate-deployment-fixture','credential':'fixture-only bootstrap accepts controller fixture token to isolate actual instance UID refusal','actual_instance_uid':http('GET','/instance',token=token)['instance_uid'],'expected_instance_uid':instance})
    records.append({'name':'qualification','status':'PASS','public_source_capture':True,'immutable_admitted':True,'real_metadata_comments':True,'duplicate_effect_readback':True,'offline_restart_recovery':True,'worker_boundary':True,'replacement_context':True,'runtime_cleanup_qualified':False})
except BaseException as error:
    records.append({'name':'qualification','status':'FAIL','error':str(error)})
    raise
finally:
    for p in processes:
        stop(p)
    proxy.shutdown()
    proxy.server_close()
    for log in logs:
        log.close()
    api_trace.close()
    manifest = {'fixture':str(ROOT),'binary_sha256':SHA,'kata_commit':'b70a4306464e319701df30a3b92860289613b15a','api_schema_version':'0.24.0','records':records}
    (ROOT/'qualification.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print('Retained real Kata fixture:',ROOT,flush=True)

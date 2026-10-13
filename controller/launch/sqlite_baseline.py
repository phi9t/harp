#!/usr/bin/env python3
"""Disposable real-model SQLite/Kata/Git baseline, retained in a new tmux session.

start builds once and launches the outer runner. Outer prepares immutable intent
and operator bootstrap, then sends ONLY literal lead TUI input. All assignment,
candidate capture, verification, acceptance and physical landing use the existing
controller and scoped real model tools. No mainline repository is a target.
"""
import argparse
import hashlib
import json
import os
import pathlib
import secrets
import shlex
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
import urllib.parse
import uuid

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

REPO = _temp_policy_file.parents[1]
SELF = pathlib.Path(__file__).resolve()
BIN = pathlib.Path('/data02/home/philip.yang/.codex/packages/standalone/releases/0.159.3-x86_64-unknown-linux-musl/bin')
KATA_SHA256 = '8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6'
KATA = pathlib.Path(os.environ.get(
    'CORENIUS_KATA_PINNED_BINARY',
    pathlib.Path.home() / '.local/share/corenius/pinned-tools/kata' / KATA_SHA256 / 'kata-pinned',
))
CATALOG = pathlib.Path('/data02/home/philip.yang/devx/tmp/corenius-sqlite-topology-sms522gg/catalog.json')
CONFIG = CATALOG.parent / 'codex/config.toml'
MODELS = ['gpt-6.1-sol', 'gpt-6-astra', 'gpt-6-sol', 'gpt-6-luna',
          'gpt-5.6-sol', 'gpt-5.6-terra', 'gpt-5.6-luna', 'gpt-5.5']
TARGET = REPO / '.scratch/sqlite06-happy-path/target'
BINARY_DIR = TARGET / 'debug'
CONTROLLER = BINARY_DIR / 'corenius-sqlite-controller'
REQUIRED_BINARIES = (
    'corenius-sqlite-controller',
    'corenius-attempt-supervisor',
    'corenius-attempt-gate',
)
SCHEMA = 'e587a3d5ffab12e528589e14d385b04d1192b9ae6914ce008cd3ea507e5aa455'


def temp_parent():
    override = os.environ.get('TMPDIR')
    if override:
        root = pathlib.Path(override)
    else:
        root = pathlib.Path(os.environ.get('CORENIUS_RUNS') or pathlib.Path.home() / '.cache/corenius') / 'tmp'
    root.mkdir(parents=True, exist_ok=True)
    return root


def new_run_root(run_id):
    return pathlib.Path(tempfile.mkdtemp(prefix='cb-' + run_id + '-', dir=str(temp_parent())))


def checked(argv, **kwargs):
    return subprocess.run([str(a) for a in argv], check=True, **kwargs)


def pin(path):
    path = pathlib.Path(path).resolve()
    return {'path':str(path), 'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}


def require_binary_dir(path):
    path = pathlib.Path(path).resolve()
    missing = []
    binaries = {}
    for name in REQUIRED_BINARIES:
        candidate = path / name
        binaries[name] = candidate
        if not candidate.is_file() or not os.access(candidate, os.X_OK):
            missing.append(name)
    if missing:
        raise RuntimeError('binary directory incomplete: missing executable ' + ', '.join(missing))
    return binaries


def scratch_root():
    work = REPO / '.scratch/sqlite06-happy-path'
    work.mkdir(parents=True, exist_ok=True)
    return work


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')
    path.chmod(0o600)


def safe(value):
    if isinstance(value, dict):
        return {k:('[redacted]' if any(w in k.lower() for w in ['token','capability','secret'])
                   else safe(v)) for k, v in value.items()
                if k not in ['reasoning','encrypted_content','native_turns']}
    if isinstance(value, list):
        return [safe(v) for v in value]
    return value


def append(root, name, value):
    with (root / name).open('a') as out:
        out.write(json.dumps(safe(value)) + '\n')


def git(path, *args):
    result = checked(['/usr/bin/git','-c','core.hooksPath=/dev/null',
                      '-c','core.fsmonitor=false','-C',path,*args],
                     env={'PATH':'/usr/bin:/bin','GIT_CONFIG_NOSYSTEM':'1',
                          'GIT_CONFIG_GLOBAL':'/dev/null','GIT_NO_REPLACE_OBJECTS':'1'},
                     capture_output=True, text=True)
    return result.stdout.strip()


def wait(root, description, predicate, seconds=180):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        try:
            result = predicate()
            if result:
                append(root, 'milestones.jsonl', {'at':time.time(),'milestone':description})
                return result
        except (ConnectionRefusedError, FileNotFoundError, TimeoutError, urllib.error.URLError):
            pass
        time.sleep(.5)
    raise TimeoutError(description)


def request(root, kind, args=None, command=False):
    authority = json.loads((root / 'private.json').read_text())
    value = {'auth':{'principal':'operator','token':authority['operator']}, 'project':'demo'}
    value['command' if command else 'query'] = {'kind':kind,'args':args or {}}
    if command:
        value['command'].update(expected_revision=0 if kind == 'create_project' else request(root,'snapshot')['revision'],
                                idempotency_key=str(uuid.uuid4()))
    with socket.socket(socket.AF_UNIX) as stream:
        stream.settimeout(40)
        stream.connect(str(root / 'controller.sock'))
        stream.sendall(json.dumps(value).encode() + b'\n')
        result = json.loads(stream.makefile('rb').readline())
    if 'error' in result:
        raise RuntimeError(kind + ': ' + result['error'])
    return result['ok']


def http(root, method, path, body=None, bootstrap=False):
    private = json.loads((root / 'private.json').read_text())
    token = private['kata_bootstrap'] if bootstrap else (root / 'kata.token').read_text()
    req = urllib.request.Request('http://127.0.0.1:' + str(private['kata_port']) + '/api/v1' + path,
                                 data=None if body is None else json.dumps(body).encode(), method=method,
                                 headers={'Authorization':'Bearer ' + token,'Content-Type':'application/json'})
    with urllib.request.urlopen(req, timeout=5) as response:
        return json.load(response)


def reuse_kata(root, configured):
    path = pathlib.Path(configured['path'])
    if pin(path) != configured or path.stat().st_mode & 0o077:
        raise RuntimeError('Kata reuse configuration pin/privacy changed')
    config = json.loads(path.read_text())
    origin = urllib.parse.urlparse(config['origin'])
    if (origin.scheme != 'http' or origin.hostname != '127.0.0.1' or not origin.port
            or origin.path or origin.query or origin.fragment or origin.username
            or config['api_schema_version'] != '0.24.0' or config['actor'] != 'corenius-controller'):
        raise RuntimeError('explicit pinned private loopback Kata reuse configuration required')
    token_file = pathlib.Path(config['token_file'])
    if not token_file.is_file() or token_file.stat().st_mode & 0o077:
        raise RuntimeError('private Kata reuse token file required')
    token = token_file.read_text()
    req = urllib.request.Request(config['origin'] + '/api/v1/instance',
                                 headers={'Authorization':'Bearer ' + token})
    with urllib.request.urlopen(req, timeout=5) as response:
        identity = json.load(response)
    if (identity['instance_uid'] != config['origin_instance_uid']
            or identity['auth']['actor'] != config['actor']):
        raise RuntimeError('Kata reuse identity mismatch')
    private = json.loads((root / 'private.json').read_text())
    private['kata_port'] = origin.port
    write(root / 'private.json', private)
    (root / 'kata.token').write_text(token)
    (root / 'kata.token').chmod(0o600)


def port():
    with socket.socket() as stream:
        stream.bind(('127.0.0.1', 0))
        return stream.getsockname()[1]


def window(root, name, argv):
    manifest = json.loads((root / 'manifest.json').read_text())
    pane = checked(['tmux','new-window','-d','-P','-F','#{pane_id}', '-t',manifest['session'],
                    '-n',manifest.get('window_prefix','') + name,shlex.join([str(a) for a in argv])],
                   capture_output=True, text=True).stdout.strip()
    checked(['tmux','set-option','-w','-t',pane,'remain-on-exit','on'])
    return pane


def service_window(root, name):
    return window(root, name, ['/usr/bin/python3',SELF,'_service',root,name])


def literal(root, text):
    manifest = json.loads((root / 'manifest.json').read_text())
    count = len(list(root.glob('lead-input-*.txt'))) + 1
    path = root / ('lead-input-' + str(count) + '.txt')
    path.write_text(text)
    buffer = 'corenius-' + manifest['run_id'] + '-' + str(count)
    paste = root / ('lead-paste-' + str(count) + '.bin')
    paste.write_bytes(b'\x1b[200~' + text.encode('utf-8') + b'\x1b[201~')
    checked(['tmux','load-buffer','-b',buffer,paste])
    checked(['tmux','paste-buffer','-d','-b',buffer,'-t',manifest['lead_pane']])
    checked(['tmux','send-keys','-t',manifest['lead_pane'],'Enter'])
    append(root,'ingress.jsonl',{'at':time.time(),'path':str(path),'sha256':hashlib.sha256(text.encode()).hexdigest(),
                               'destination':manifest['lead_pane'],'literal_bracketed_paste':True})
    print('Lead literal input', count, flush=True)


def service(root, name):
    private = json.loads((root / 'private.json').read_text())
    controller = pathlib.Path(json.loads((root / 'manifest.json').read_text())['controller']['path'])
    env = {'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TERM':'xterm-256color'}
    if name == 'kata':
        env['KATA_HOME'] = str(root / 'kata')
        argv = [KATA,'daemon','start','--foreground','--listen','127.0.0.1:' + str(private['kata_port'])]
    elif name == 'relay':
        env['HOME'] = str(pathlib.Path.home())
        argv = ['/usr/bin/python3', SELF.with_name('sqlite_responses_relay.py'), root / 'provider.sock',
                '--evidence',root / 'relay.jsonl']
    elif name in ['workers','controlplanes']:
        env['CORENIUS_SUPERVISOR_TOKEN'] = private['executor'] if name == 'workers' else private['app']
        if name == 'workers':
            env['CORENIUS_GIT_CONFIG'] = str(root / 'git.json')
        argv = [controller.parent / 'corenius-attempt-supervisor',root / (name + '.json'),root / (name + '.sock')]
    elif name == 'controller':
        env.update(CORENIUS_OPERATOR_TOKEN=private['operator'], CORENIUS_SUPERVISOR_TOKEN=private['executor'],
                   CONTROLPLANE_SECRET=private['app'], CORENIUS_SUPERVISOR_SOCKET=str(root / 'workers.sock'),
                   CORENIUS_CODEX_CONFIG=str(root / 'native.json'), CORENIUS_GIT_CONFIG=str(root / 'git.json'),
                   CORENIUS_KATA_CONFIG=str(root / 'kata.json'))
        argv = [controller,root / 'state.db',root / 'controller.sock']
    elif name in ['gateway','lead','dashboard']:
        lead = json.loads((root / 'lead.json').read_text())
        env.update(CORENIUS_CLIENT_PRINCIPAL=lead['principal'],CORENIUS_CLIENT_TOKEN=lead['token'],
                   CORENIUS_UI_TOKEN=private['ui'])
        if name == 'gateway':
            env['CORENIUS_UI_AUDIT'] = str(root / 'ui-audit.jsonl')
            argv = [controller,'--native-ui',root / 'controller.sock','demo','127.0.0.1:' + str(private['ui_port'])]
        elif name == 'dashboard':
            argv = [controller,'--dashboard',root / 'controller.sock','demo']
        else:
            home = root / 'ui-home'
            home.mkdir(exist_ok=True)
            (home / 'codex').mkdir(exist_ok=True)
            env.update(HOME=str(home), CODEX_HOME=str(home / 'codex'))
            manifest = json.loads((root / 'manifest.json').read_text())
            argv = [BIN / 'codex','--remote','ws://127.0.0.1:' + str(private['ui_port']),
                    '--remote-auth-token-env','CORENIUS_UI_TOKEN','resume',manifest['lead_thread'],'--no-alt-screen']
    else:
        raise ValueError('unknown owned service')
    interactive = name in ['lead','dashboard']
    log = None if interactive else open(root / (name + '.log'),'ab',buffering=0)
    process = subprocess.Popen([str(a) for a in argv], env=env, cwd=root, stdout=None if interactive else log,
                               stderr=None if interactive else subprocess.STDOUT)
    ticks = pathlib.Path('/proc/' + str(process.pid) + '/stat').read_text().rsplit(') ',1)[1].split()[19]
    append(root,'processes.jsonl',{'name':name,'pid':process.pid,'start_ticks':ticks,'at':time.time(),
                                 'argv':[str(a) for a in argv],'binary':pin(pathlib.Path(argv[0]))})
    print(name, 'owned PID', process.pid, 'start', ticks, flush=True)
    result = process.wait()
    append(root,'processes.jsonl',{'name':name,'pid':process.pid,'start_ticks':ticks,'exit':result,'at':time.time()})
    print(name,'actual exit',result,flush=True)
    raise SystemExit(result)


def role(root, task):
    while True:
        center = request(root,'control_center',{'limit':20})
        attempts = [a for a in center.get('attempts',[]) if a.get('task') == task]
        ids = {a['id'] for a in attempts}
        bindings = [b for b in center.get('runtime_bindings',[]) if b.get('attempt') in ids]
        output = {'task':task,'attempts':attempts,'bindings':bindings,
                  'progress':[p for p in center.get('progress_reports',[]) if p.get('task') == task],
                  'candidates':[c for c in center.get('candidates',[]) if c.get('attempt') in ids]}
        print('\x1b[2J\x1b[H' + json.dumps(safe(output),indent=2),flush=True)
        time.sleep(2)


def snapshot(root, name):
    center = request(root,'control_center',{'limit':20})
    write(root / (name + '-dashboard.json'),safe(center))
    manifest = json.loads((root / 'manifest.json').read_text())
    capture = checked(['tmux','capture-pane','-p','-t',manifest['session'] + ':' + manifest.get('window_prefix','') + 'dashboard'],capture_output=True,text=True)
    (root / (name + '-dashboard.txt')).write_text(capture.stdout)
    return center


def task_state(root, task, state):
    center = request(root,'control_center',{'limit':20})
    return center if any(t['id'] == task and t['state'] == state for t in center['tasks']) else None


def lead_terminal(root):
    registry = request(root,'codex_registry')
    return next((b for b in registry['bindings'] if b['role'] == 'lead' and
                 b['document'].get('current_turn',{}).get('state') == 'terminal'),None)


def assign_prompt(root, tasks):
    manifest = json.loads((root / 'manifest.json').read_text())
    selections = []
    for task in tasks:
        item = manifest['items'][task]
        selections.append({'task':task,'item':item['uid'],'expected_source_revision':item['revision'],
                           'source_files':{'storage':'intent','mode':'git','commit':manifest['intent_commit'],
                                           'paths':['spec.md',task + '.md']}})
    return ('You are authorized to coordinate this DISPOSABLE demo through scoped corenius tools. '
            'Query current state/revision. Create each pinned Kata task below using create_kata_task, then admit '
            'ALL of them before waiting, so both workers run concurrently when two are listed. '
            'Use corenius_command with fresh operation_id and exact current expected_revision; on a revision '
            'conflict requery and use a fresh key. Never supply identity selectors. Workers independently edit '
            'and capture their own candidate. Do not edit their files, fabricate a result, verify, accept, land '
            'or cancel in this turn. Return once the listed admissions are acknowledged.\n' + json.dumps(selections))


def landing_prompt(root, tasks, stale=False):
    return ('Coordinate only through scoped corenius_query/corenius_command. Query the current control center. '
            'For each task in this exact order ' + json.dumps(tasks) + ', find its current submitted candidate. '
            'Run verify_candidate, require actual verdict pass, then accept_candidate citing THAT candidate and '
            'THAT verification ID. Then land_candidate with expected_trunk equal to the candidate BASE retained '
            'before either landing. This is authorized FF landing only in the disposable canonical repository. '
            + ('The second concurrent candidate intentionally has the old base; retain its stale_base result. '
               if stale else '') +
            'Only after these results, cancel_attempt for each corresponding old attempt to release the actual '
            'executor slot. Do not abort tasks or delete evidence. Do not admit a new attempt yet. Requery '
            'revision before each mutation; return with exact candidate/proof/acceptance/landing IDs and states.')


def run(root):
    manifest = json.loads((root / 'manifest.json').read_text())
    try:
        for name in ['workers','controlplanes','t-exec','t-app','authority','canonical','seed','input','kata']:
            (root / name).mkdir(mode=0o700)
        private = {key:secrets.token_hex(32) for key in ['operator','executor','app','ui','kata_bootstrap']}
        private.update(kata_port=port(),ui_port=port())
        write(root / 'private.json',private)
        if not manifest.get('reuse_kata'):
            (root / 'kata/config.toml').write_text('[auth]\ntoken="' + private['kata_bootstrap'] + '"\nrequire_token_identity=true\n')
            (root / 'kata/config.toml').chmod(0o600)
        git(root / 'canonical','init','--bare','--template=')
        git(root / 'seed','init','--template=')
        (root / 'seed/README.md').write_text('Disposable real-model happy-path repository.\n')
        git(root / 'seed','add','README.md')
        git(root / 'seed','-c','user.name=Corenius demo','-c','user.email=demo@invalid','commit','-m','Disposable base')
        base = git(root / 'seed','rev-parse','HEAD')
        git(root / 'seed','push',str(root / 'canonical'),base + ':refs/heads/protected')
        git(root / 'input','init','--template=')
        spec = ('Use your own scoped corenius_query/corenius_command tools. Read the assigned task. Report '
                'progress, perform real filesystem edits in your complete /workspace Git repository, run the '
                'local function check, and commit all task changes using git -c user.name=Worker -c '
                'user.email=worker@invalid commit. Then capture_candidate with a report describing the actual '
                'commit/check. Never claim verifier or acceptance authority. Query own_assignment for revision '
                'before commands; on revision conflict requery and use a fresh operation_id. Preserve all '
                'files inherited from the admitted base. Once capture is acknowledged, return; the lead handles '
                'verification and termination. Do not start another thread or provider.\n')
        (root / 'input/spec.md').write_text(spec)
        for task in ['solo','a','b']:
            text = ('Implement /workspace/task_' + task + '.py with function answer() returning exactly ' +
                    repr(task.upper()) + '. Use real tools to write and test it. ' +
                    ('Before editing, run a single bounded 20-second sleep inside your executor; this creates '
                     'a measured concurrent episode and you must still produce the useful file/commit. ' if task != 'solo' else '') +
                    'Report your own progress, commit, then capture the exact candidate. ' +
                    'For a later integration Attempt, recreate only your own file on the newly admitted base '
                    'and preserve the peer files already there.\n')
            (root / ('input/' + task + '.md')).write_text(text)
        git(root / 'input','add','.')
        git(root / 'input','-c','user.name=Corenius intent','-c','user.email=intent@invalid','commit','-m','Immutable disposable intent')
        manifest['intent_commit'] = git(root / 'input','rev-parse','HEAD')
        manifest['base'] = base
        suite = root / 'verify.py'
        suite.write_text("import pathlib,sys,subprocess,importlib.util\nw=pathlib.Path('/workspace')\n"
                         "assert not pathlib.Path(sys.argv[2]).exists(), 'host authority absent'\n"
                         "assert 'CapEff:\\t0000000000000000' in pathlib.Path('/proc/self/status').read_text()\n"
                         "g=lambda *a:subprocess.check_output(['/usr/bin/git','-c','core.hooksPath=/dev/null','-C','/workspace',*a],text=True).strip()\n"
                         "required={sys.argv[1]}\n"
                         "for commit in g('rev-list','HEAD').splitlines():\n"
                         " for task in ('solo','a','b'):\n"
                         "  if 'task_'+task+'.py' in g('ls-tree','--name-only',commit).splitlines():required.add(task)\n"
                         "for task in required:\n"
                         " spec=importlib.util.spec_from_file_location('task_'+task,w/('task_'+task+'.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);assert m.answer()==task.upper()\n"
                         "print('INDEPENDENT_FIXED_CONTRACT_PASS',','.join(sorted(required)),'OS_ISOLATED')\n")
        write(root / 'git.json',{'project':'demo','root':str(root / 'authority'),'attempts_root':str(root / 'workers'),
                                'canonical':str(root / 'canonical'),'protected_ref':'refs/heads/protected',
                                'git':pin('/usr/bin/git'),'python':pin('/usr/bin/python3'),'bubblewrap':pin('/usr/bin/bwrap'),
                                'suites':[{'id':task + '-retained-contract','task':task,'base':None,
                                           'script':pin(suite),'args':[task,str(root)]} for task in ['solo','a','b']]})
        (root / 'catalog.json').write_bytes(CATALOG.read_bytes())
        (root / 'runtime.toml').write_text(CONFIG.read_text().replace(str(CATALOG),'/run/codex/catalog.json').replace('35361','8777'))
        assets = {'binary':pin(BIN / 'codex'),'companion':pin(BIN / 'codex-code-mode-host'),
                  'catalog':pin(root / 'catalog.json'),'config':pin(root / 'runtime.toml'),
                  'launcher':pin(REPO / 'controller/support/codex_insula.py')}
        for name, kind, transport in [('workers','exec','t-exec'),('controlplanes','app','t-app')]:
            profile = dict(assets,kind=kind,transport_root=str(root / transport))
            if kind == 'app':
                profile.update(provider_socket=str(root / 'provider.sock'),executor_root=str(root / 't-exec'),own_executor_only=True)
            write(root / (name + '.json'),{'root':str(root / name),
                                         'profile':{'executable':'/usr/bin/python3','args':['/run/codex/launcher.py'],'codex':profile}})
        write(root / 'native.json',{'schema_migration':'1-to-2','topology':'separate','transport_root':str(root / 't-app'),
                                   'model':manifest.get('selected_model','gpt-6-sol'),
                                   'servers':[{'id':'native','supervisor_socket':str(root / 'controlplanes.sock'),
                                               'auth_ref':'CONTROLPLANE_SECRET','transport_root':str(root / 't-app'),
                                               'worker_supervisor_socket':str(root / 'workers.sock'),
                                               'worker_auth_ref':'CORENIUS_SUPERVISOR_TOKEN',
                                               'worker_transport_root':str(root / 't-exec'),'schema_digest':SCHEMA}]})
        if manifest.get('reuse_kata'):
            reuse_kata(root, manifest['reuse_kata'])
            private = json.loads((root / 'private.json').read_text())
        else:
            service_window(root,'kata')
            wait(root,'real Kata ready',lambda:http(root,'GET','/health',bootstrap=True),30)
            token = http(root,'POST','/tokens',{'actor':'corenius-controller','name':'disposable-demo'},bootstrap=True)['plaintext']
            (root / 'kata.token').write_text(token)
            (root / 'kata.token').chmod(0o600)
        project = http(root,'POST','/projects',{'name':manifest['run_id']})['project']
        items = {}
        for task in ['solo','a','b']:
            items[task] = http(root,'POST','/projects/' + str(project['id']) + '/issues',
                               {'title':'Disposable ' + task,'body':'Implement ' + task + '.md constrained by spec.md.',
                                'force_new':True})['issue']
        instance = http(root,'GET','/instance')['instance_uid']
        write(root / 'kata.json',{'schema_migration':'kata-v1','controller_id':manifest['run_id'],'actor':'corenius-controller',
                                 'api_schema_version':'0.24.0','origin_instance_uid':instance,
                                 'origin':'http://127.0.0.1:' + str(private['kata_port']),'token_file':str(root / 'kata.token'),
                                 'project_id':project['id'],'project_uid':project['uid'],'repositories':{'intent':str(root / 'input')}})
        manifest.update(items=items,kata_instance=instance,kata_project=project,native_config_sha256=pin(root / 'native.json')['sha256'],
                        git_policy_sha256=pin(root / 'git.json')['sha256'],kata_config_sha256=pin(root / 'kata.json')['sha256'])
        write(root / 'manifest.json',manifest)
        for name in ['relay','workers','controlplanes']:
            service_window(root,name)
            wait(root,name + ' socket ready',lambda n=name:(root / ('provider.sock' if n == 'relay' else n + '.sock')).exists(),30)
        service_window(root,'controller')
        wait(root,'controller ready',lambda:(root / 'controller.sock').exists(),30)
        request(root,'create_project',{'name':'demo','base':base},command=True)
        lead = request(root,'replace_lead',{},command=True)['result']
        write(root / 'lead.json',lead)
        binding = wait(root,'real native lead bound',lambda:next((b for b in request(root,'codex_registry')['bindings']
                                                                 if b['role'] == 'lead' and b['document'].get('ui_input_ready')),None),90)
        manifest['lead_thread'] = binding['document']['thread_id']
        write(root / 'manifest.json',manifest)
        service_window(root,'gateway')
        wait(root,'scoped gateway ready',lambda:(root / 'gateway.log').exists() and 'Gateway listening' in (root / 'gateway.log').read_text(),30)
        manifest['lead_pane'] = service_window(root,'lead')
        write(root / 'manifest.json',manifest)
        checked(['tmux','resize-window','-t',manifest['lead_pane'],'-x','150','-y','45'])
        service_window(root,'dashboard')
        window(root,'workerA',['/usr/bin/python3',SELF,'_role',root,'a'])
        window(root,'workerB',['/usr/bin/python3',SELF,'_role',root,'b'])
        wait(root,'real lead TUI attached',lambda:(root / 'ui-audit.jsonl').exists() and any(
             json.loads(line).get('method') == 'thread/resume' and json.loads(line).get('accepted')
             for line in (root / 'ui-audit.jsonl').read_text().splitlines()),30)
        wait(root,'initial lead turn terminal',lambda:lead_terminal(root),120)
        literal(root,assign_prompt(root,['solo']))
        wait(root,'one worker exact candidate captured',lambda:task_state(root,'solo','submitted'),300)
        wait(root,'lead admission turn terminal',lambda:lead_terminal(root),120)
        snapshot(root,'one-candidate')
        literal(root,landing_prompt(root,['solo']))
        wait(root,'one worker disposable FF landed',lambda:task_state(root,'solo','landed'),300)
        wait(root,'one worker executor actual stop',lambda:request(root,'snapshot')['occupied_slots'] == 0,90)
        wait(root,'lead first landing turn terminal',lambda:lead_terminal(root),120)
        snapshot(root,'one-landed')
        literal(root,assign_prompt(root,['a','b']))
        overlap = wait(root,'two actual executor slots overlap',lambda:request(root,'snapshot') if request(root,'snapshot')['occupied_slots'] == 2 else None,180)
        write(root / 'overlap-start.json',safe(overlap))
        wait(root,'two useful candidates captured',lambda:task_state(root,'a','submitted') and task_state(root,'b','submitted'),360)
        wait(root,'lead concurrent admission terminal',lambda:lead_terminal(root),120)
        snapshot(root,'two-candidates')
        literal(root,landing_prompt(root,['a','b'],stale=True))
        wait(root,'first concurrent candidate landed',lambda:task_state(root,'a','landed'),300)
        wait(root,'stale second candidate retained',lambda:task_state(root,'b','integration-required'),300)
        wait(root,'both old executors actual stop',lambda:request(root,'snapshot')['occupied_slots'] == 0,90)
        wait(root,'lead stale landing turn terminal',lambda:lead_terminal(root),120)
        snapshot(root,'stale-retained')
        literal(root,'Query current state. Task b is integration-required with an immutable stale candidate. '
                'Admit a NEW attempt for task b using the current controller base. Preserve the old candidate/proof. '
                'The new real worker must recreate its own task_b.py on this new complete repository, preserve '
                'solo/a, commit and capture a new artifact. Only admit in this turn; no verification or landing yet.')
        wait(root,'fresh integration candidate captured',lambda:task_state(root,'b','submitted'),360)
        wait(root,'lead integration admission terminal',lambda:lead_terminal(root),120)
        literal(root,landing_prompt(root,['b']))
        wait(root,'fresh verified accepted integration FF landed',lambda:task_state(root,'b','landed'),300)
        wait(root,'integration executor actual stop',lambda:request(root,'snapshot')['occupied_slots'] == 0,90)
        final = snapshot(root,'final')
        registry = request(root,'codex_registry')
        write(root / 'bindings.json',safe(registry))
        final_ref = git(root / 'canonical','rev-parse','refs/heads/protected')
        result = {'label':'happy path demonstrated','baseline_qualified':False,'at':time.time(),'manifest':manifest,
                  'final_ref':final_ref,'tasks':final['tasks'],'candidates':final['candidates'],
                  'verification':final['verification'],'acceptance':final['acceptance'],
                  'bindings_file':str(root / 'bindings.json'),'overlap':'actual two occupied executor slots; relay model intervals retained',
                  'absent_metrics':'queue/presentation/explicitACK/applied latency unknown when not represented by retained events',
                  'live_services':'controller/Kata/relay/supervisors/lead/gateway/dashboard retained for owner observation',
                  'exit':0}
        write(root / 'result.json',safe(result))
        print(json.dumps(safe(result)),flush=True)
    except BaseException as error:
        write(root / 'failure.json',{'at':time.time(),'class':type(error).__name__,'message':str(error),
                                    'exit':1,'label':'initial setup or target failure; retained, no success claim'})
        print('Retained trial failure:',type(error).__name__,str(error),flush=True)
        raise


def start(args):
    global TARGET, BINARY_DIR, CONTROLLER
    if args.target_dir:
        TARGET = args.target_dir.resolve()
        BINARY_DIR = TARGET / 'debug'
        CONTROLLER = BINARY_DIR / 'corenius-sqlite-controller'
    if args.binary_dir:
        BINARY_DIR = args.binary_dir.resolve()
        CONTROLLER = BINARY_DIR / 'corenius-sqlite-controller'
        binaries = require_binary_dir(BINARY_DIR)
    else:
        work = scratch_root()
        with (work / 'retained-build.log').open('ab') as log:
            result = subprocess.run(['cargo','build','--manifest-path',str(REPO / 'controller/Cargo.toml'),
                                     '--target-dir',str(TARGET),'--locked','--offline'],stdout=log,stderr=subprocess.STDOUT)
        write(work / ('build-' + str(time.time_ns()) + '.json'),{'exit':result.returncode,'controller':pin(CONTROLLER) if result.returncode == 0 else None})
        if result.returncode:
            raise SystemExit(result.returncode)
        binaries = require_binary_dir(BINARY_DIR)
    if args.session:
        checked(['tmux','has-session','-t','=' + args.session])
    work = scratch_root()
    if pin(BIN / 'codex')['sha256'] != '8bf204b36a2f6dd0dab73aa2f639892e67ef9ac8befccb4a05b1496ebf25c479':
        raise RuntimeError('Codex pin changed')
    if pin(KATA)['sha256'] != KATA_SHA256:
        raise RuntimeError('Kata pin changed')
    run_id = time.strftime('%Y%m%dT%H%M%SZ',time.gmtime()) + '-' + uuid.uuid4().hex[:6]
    root = new_run_root(run_id)
    root.chmod(0o700)
    manifest = {'run_id':run_id,'root':str(root),'session':args.session or 'corenius-baseline-' + run_id,
                'selected_model':args.model or 'gpt-6-sol',
                'source_commit':git(REPO,'rev-parse','HEAD'),'source_tree':git(REPO,'rev-parse','HEAD^{tree}'),
                'runner':pin(SELF),'relay':pin(SELF.with_name('sqlite_responses_relay.py')),
                'controller':pin(binaries['corenius-sqlite-controller']),
                'supervisor':pin(binaries['corenius-attempt-supervisor']),
                'gate':pin(binaries['corenius-attempt-gate']),
                'codex':pin(BIN / 'codex'),'kata':pin(KATA),'launch':['python3',str(SELF),'start'],
                 'scope':'owner c9 happy-path-first; disposable targets only; baseline remains unqualified'}
    if args.session:
        manifest['window_prefix'] = 'trial-' + run_id[-6:] + '-'
    if args.reuse_kata:
        manifest['reuse_kata'] = pin(args.reuse_kata)
    manifest['launch'] = ['python3',str(SELF),*sys.argv[1:]]
    write(root / 'manifest.json',manifest)
    if args.session:
        manifest['outer_pane'] = window(root,'outer',['/usr/bin/python3',SELF,'_run',root])
    else:
        checked(['tmux','new-session','-d','-s',manifest['session'],'-n','outer',
                 shlex.join(['/usr/bin/python3',str(SELF),'_run',str(root)])])
        checked(['tmux','set-option','-t',manifest['session'],'remain-on-exit','on'])
    write(work / ('launch-' + run_id + '.json'),manifest)
    print(json.dumps({'run':str(root),'session':manifest['session'],'attach':'tmux attach -t ' + manifest['session'],
                      'status':'python3 ' + str(SELF) + ' status ' + str(root)}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command',required=True)
    p = commands.add_parser('start')
    p.add_argument('--session', help='use this existing tmux session with uniquely named trial windows')
    p.add_argument('--reuse-kata', type=pathlib.Path, help='reuse this exact private Kata configuration and authenticated instance')
    binaries = p.add_mutually_exclusive_group()
    binaries.add_argument('--target-dir', type=pathlib.Path, help='build continuation binaries separately')
    binaries.add_argument('--binary-dir', type=pathlib.Path, help='use an explicit directory containing all controller binaries')
    p.add_argument('--model', choices=MODELS, help='select an installed model for this new disposable run')
    for name in ['status','input','_run','_service','_role']:
        p = commands.add_parser(name)
        p.add_argument('run',type=pathlib.Path)
        if name == 'input':
            p.add_argument('--file',type=pathlib.Path,required=True)
        elif name in ['_service','_role']:
            p.add_argument('name')
    args = parser.parse_args()
    if args.command == 'start':
        start(args)
    elif args.command == 'status':
        for name in ['manifest.json','result.json','failure.json']:
            if (args.run / name).exists():
                print((args.run / name).read_text())
        if (args.run / 'controller.sock').exists():
            print(json.dumps(safe(request(args.run,'snapshot')),indent=2))
    elif args.command == 'input':
        literal(args.run,args.file.read_text())
    elif args.command == '_run':
        run(args.run)
    elif args.command == '_service':
        service(args.run,args.name)
    else:
        role(args.run,args.name)


if __name__ == '__main__':
    main()

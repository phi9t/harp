"""Pinned requests over OpenSSH; no retry loop or remote scheduling state."""
import hashlib
import json
from pathlib import Path
import re
import shlex
import subprocess
import os
import signal
import time

ROOT = Path(__file__).resolve().parents[1]


def source_identity(root=ROOT):
    files = {}
    for package in ('durable', 'insula', 'runner'):
        for path in sorted((root/package).rglob('*.py')):
            if path.name.startswith('test_') or path.name.endswith('_test.py'):continue
            files[str(path.relative_to(root))] = hashlib.sha256(path.read_bytes()).hexdigest()
    return hashlib.sha256(json.dumps(files,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def host(path, alias):
    import tomllib
    path = Path(path)
    if path.is_symlink():raise ValueError('inventory must not be symlinked')
    info = path.stat()
    if info.st_mode & 0o077:raise ValueError('real inventory must be private (0600)')
    data = tomllib.loads(path.read_text())
    entry = data['hosts'][alias]
    if set(entry) - {'ssh_alias','repository','state','execution','execution_contract','python'} or not {'ssh_alias','repository','state','execution','execution_contract'} <= set(entry):raise ValueError('invalid host profile fields')
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*',entry['ssh_alias']):raise ValueError('invalid SSH alias')
    if not isinstance(entry['execution_contract'],str) or not entry['execution_contract'].startswith('insula-durable-v1:'):
        raise ValueError('inventory must pin the admitted execution contract')
    for name in ('repository','state','execution'):
        value = entry[name]
        if not isinstance(value,str) or not value.startswith('/') or any(c in value for c in '\x00\n\r'):
            raise ValueError('host paths must be absolute and single-line')
    if 'python' in entry and (not isinstance(entry['python'],str) or not entry['python'].startswith('/') or any(c in entry['python'] for c in '\x00\n\r')):
        raise ValueError('controller Python must be an absolute executable path')
    return entry


def _exchange(argv, *, input, timeout):
    """Own the SSH transport group, including its short-lived proxy process."""
    process = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, start_new_session=True)
    deadline = time.monotonic() + timeout
    pending = input
    while True:
        try:
            stdout, stderr = process.communicate(pending, timeout=min(.25, max(.01, deadline-time.monotonic())))
            return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)
        except subprocess.TimeoutExpired:
            pending = None
            expired = time.monotonic() >= deadline
            if process.poll() is None and not expired:
                continue
            # A proxy can retain pipes after SSH itself exits. It belongs only to
            # this request; terminating it does not cancel a remote activity.
            try: os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            stdout, stderr = process.communicate(timeout=2)
            if expired: raise subprocess.TimeoutExpired(argv, timeout)
            return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)


def request(profile, arguments, *, identity=None, timeout=3600):
    if not isinstance(arguments,list) or not all(isinstance(a,str) and '\x00' not in a for a in arguments):
        raise ValueError('request arguments must be literal NUL-free strings')
    envelope = {'schema':1,'source':identity or source_identity(), 'state':profile['state'],
                'execution':profile['execution'],'contract':profile['execution_contract'],'arguments':arguments}
    command = 'cd '+shlex.quote(profile['repository'])+' && exec '+shlex.join([
        'env','PYTHONDONTWRITEBYTECODE=1','PYTHONNOUSERSITE=1',
        'PYTHONPATH='+profile['repository'],profile.get('python','python3'),'-m','runner.remote'])
    try:
        result = _exchange(['ssh','-T','-o','BatchMode=yes','-o','StrictHostKeyChecking=yes','-o','ForwardAgent=no','-o','ClearAllForwardings=yes','-o','ControlMaster=no','-o','ControlPath=none','-o','ControlPersist=no','--',profile['ssh_alias'],command],
            input=json.dumps(envelope),timeout=timeout)
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError('transport outcome unknown; query the same run identity before any new execution') from exc
    if result.returncode == 255 or result.returncode < 0:
        raise RuntimeError('SSH connection lost; remote effect may be unknown; query the same run identity')
    # Do not expose host paths/banners/credentials by reflecting arbitrary SSH stderr.
    if len(result.stdout)>(24*1024*1024 if arguments[:1]==['artifact'] else 4*1024*1024):raise RuntimeError('remote response exceeds control limit')
    try:response=json.loads(result.stdout)
    except json.JSONDecodeError as exc:raise RuntimeError('invalid remote response; reconcile by run identity') from exc
    if not isinstance(response,dict) or response.get('source') != envelope['source'] or response.get('contract') != envelope['contract']:raise RuntimeError('remote response source identity differs')
    if type(response.get('returncode')) is not int or response['returncode'] != result.returncode:
        raise RuntimeError('remote return status differs; reconcile by run identity')
    return response, result.returncode

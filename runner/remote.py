"""Bounded SSH endpoint; the durable runtime owns every admitted effect."""
from __future__ import annotations
import base64
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import shutil
import sys
from .transport import ROOT, source_identity

ALLOWED = {'status','events','check','admit','workload','start','cancel','stop','capture','verify'}


def preflight(request, identity):
    if set(request) != {'schema','source','state','execution','contract','arguments'} or type(request['schema']) is not int or request['schema'] != 1:
        raise ValueError('invalid transport envelope')
    if request['source'] != identity:
        raise ValueError('remote source identity differs; no action admitted')
    for name in ('state','execution'):
        if (not isinstance(request[name],str) or not Path(request[name]).is_absolute() or
                any(c in request[name] for c in '\x00\r\n')):
            raise ValueError('invalid project profile')
    profile = json.loads(Path(request['execution']).read_text())
    if profile['contract'] != request['contract'] or profile['state'] != str(Path(request['state']).resolve()):
        raise ValueError('remote admitted profile differs; no action admitted')
    code = Path(profile['code'])
    for package in ('durable','insula'):
        current = {str(p.relative_to(ROOT)):p.read_bytes() for p in (ROOT/package).rglob('*.py')
                   if not p.name.startswith('test_') and not p.name.endswith('_test.py')}
        pinned = {str(p.relative_to(code)):p.read_bytes() for p in (code/package).rglob('*.py')}
        if current != pinned:
            raise ValueError('remote admitted code differs from requested source')
    return profile


def fetch_artifact(request, status, relative):
    from durable.agents.admission import relative_path
    relative_path(relative)
    if status.get('workflow') != 'workload' or status.get('status') != 'SUCCEEDED':
        raise ValueError('artifacts require a completed accepted workload')
    result = status['result']
    declaration = next((a for a in result['artifacts'] if a['path'] == relative),None)
    if declaration is None:
        raise ValueError('artifact is not declared by the accepted result')
    attempt = result['attempt_id']
    if not isinstance(attempt,str) or not re.fullmatch(r'[a-f0-9]{32}',attempt):
        raise ValueError('invalid accepted attempt identity')
    root = Path(request['state'])/'attempts'/attempt/'payload'/'artifacts'
    path = root/relative
    if (any(p.is_symlink() for p in (path,*path.parents)) or not path.is_file() or
            not path.resolve().is_relative_to(root.resolve())):
        raise ValueError('artifact path is unsafe')
    if path.stat().st_size != declaration['size'] or path.stat().st_size > 16<<20:
        raise ValueError('artifact size differs from accepted result')
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != declaration['sha256']:
        raise ValueError('artifact bytes differ from accepted result')
    return {'artifact':declaration,'attempt_id':attempt,'content_base64':base64.b64encode(raw).decode()}


def main():
    identity = source_identity()
    contract = None
    try:
        raw = sys.stdin.buffer.read((1<<20)+1)
        if len(raw)>1<<20:
            raise ValueError('request exceeds limit')
        request = json.loads(raw)
        if not isinstance(request,dict):
            raise ValueError('transport envelope must be an object')
        contract = request.get('contract')
        admitted_profile = preflight(request,identity)
        os.environ['INSULA_CAPSULES'] = admitted_profile['capsule_store']
        args = request['arguments']
        if (not isinstance(args,list) or not args or args[0] not in ALLOWED|{'preflight','artifact'} or
                any(not isinstance(a,str) or '\x00' in a for a in args)):
            raise ValueError('unsupported remote operation')
        if args == ['preflight']:
            if shutil.which('systemd-run') is None or shutil.which('systemctl') is None:
                raise ValueError('remote runner requires a user-manager foreground scope')
            manager = subprocess.run(['systemctl','--user','show-environment'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            if manager.returncode:
                raise ValueError('remote user manager unavailable; no workload admitted')
            from durable.execution import InsulaExecution
            InsulaExecution(json.loads(Path(request['execution']).read_text())).verify()
            data = {'source':identity,'contract':contract,'cgroup_delegation':True,'platform':sys.platform}
            code = 0
        else:
            artifact = args[0]=='artifact'
            if artifact and len(args)!=3:
                raise ValueError('artifact requires exact run identity and declared relative path')
            arguments = ['status',args[1]] if artifact else args
            command = [sys.executable,'-m','durable','--state',request['state'],
                       '--execution',request['execution'],*arguments]
            if arguments[0] == 'start':
                # SSH sessions live outside the user's delegated manager subtree.
                # A transient foreground scope enters that subtree so the guardian
                # can move children into the pre-admitted attempt cgroup. The
                # existing durable runtime still owns all scheduling and cleanup.
                manager = shutil.which('systemd-run')
                if manager is None:
                    raise ValueError('remote start requires a user-manager foreground scope')
                command = [manager,'--user','--scope','--quiet','--collect','--',*command]
            result = subprocess.run(command,text=True,capture_output=True)
            if len(result.stdout)+len(result.stderr)>4<<20:
                raise ValueError('durable control response exceeds bound')
            data = json.loads(result.stdout if result.returncode==0 else result.stderr)
            code = result.returncode
            if artifact and code==0:
                data = fetch_artifact(request,data['data'],args[2])
        print(json.dumps({'source':identity,'contract':contract,'result':data,'returncode':code}))
        return code
    except (ValueError,OSError,KeyError,TypeError,RuntimeError) as exc:
        print(json.dumps({'source':identity,'contract':contract,'error':str(exc),'returncode':2}))
        return 2


if __name__=='__main__':
    raise SystemExit(main())

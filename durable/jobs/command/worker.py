"""Execute frozen workload argv and checks; publish bounded file identities."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from durable.common import digest, publish_result, read_json
from durable.jobs.command.admission import manifest

MAX_ARTIFACT = 16 << 20
MAX_ARTIFACTS = 64 << 20


def command(argv, *, cwd, env):
    # Drain the pipe while retaining only a bounded tail. The guardian owns
    # cancellation and all descendants; this worker adds no retry/cleanup loop.
    process = subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, close_fds=True)
    tail = bytearray()
    def drain():
        while chunk := process.stdout.read(32768):
            tail.extend(chunk)
            if len(tail) > 65536:
                del tail[:-65536]
    reader = threading.Thread(target=drain)
    reader.start()
    code = process.wait()
    reader.join()
    return {'exit_code':code, 'output_tail':tail.decode('utf-8', errors='replace')}


SOURCE = '/home/bench/run/source'
OUTPUT = '/home/bench/run/output'
ORACLE = '/home/bench/run/oracle'


def sandbox_argv(workspace, output, argv, admitted, *, oracle=None):
    """A command cannot retain processes or reach its owner's input/proof.

    Source/output are writable only during workload execution. Checks receive
    read-only projections and a frozen oracle; their scratch home is private.
    Bubblewrap's PID namespace kills detached descendants when the command
    exits, before any later check or artifact hash can run. The activity guardian
    still owns the single outer deadline and cgroup cleanup obligation.
    """
    bind = '--ro-bind' if oracle is not None else '--bind'
    command = ['/usr/bin/bwrap', '--die-with-parent', '--new-session',
               '--unshare-user', '--unshare-pid', '--unshare-ipc', '--unshare-uts',
               '--unshare-net', '--unshare-cgroup-try', '--cap-drop', 'ALL',
               '--ro-bind', '/', '/', '--tmpfs', '/home/bench',
               '--dir', '/home/bench/run', '--dir', '/home/bench/run/home',
               '--proc', '/proc', '--dev', '/dev', '--tmpfs', '/tmp',
               '--tmpfs', '/run', '--dir', '/run/insula-tmp',
               bind, str(workspace), SOURCE, bind, str(output), OUTPUT,
               '--clearenv']
    environment = {'PATH':'/usr/bin:/bin', 'LANG':'C.UTF-8', 'LC_ALL':'C.UTF-8',
        'HOME':'/home/bench/run/home', 'TMPDIR':'/run/insula-tmp',
        'TMP':'/run/insula-tmp', 'TEMP':'/run/insula-tmp',
        'PYTHONPATH':'/workspace', 'PYTHONNOUSERSITE':'1', 'PYTHONDONTWRITEBYTECODE':'1',
        'GIT_CONFIG_GLOBAL':'/dev/null', 'GIT_CONFIG_NOSYSTEM':'1',
        'GIT_TERMINAL_PROMPT':'0', 'GIT_OPTIONAL_LOCKS':'0',
        'INSULA_DURABLE_CONTRACT':admitted['execution_contract'],
        'HARP_WORKLOAD_SOURCE':SOURCE, 'HARP_WORKLOAD_OUTPUT':OUTPUT,
        'HARP_WORKLOAD_PROFILE':admitted['profile'], 'HARP_WORKLOAD_RUN_ID':admitted['run_id']}
    for name, value in environment.items():
        command += ['--setenv', name, value]
    if oracle is not None:
        command += ['--ro-bind', str(oracle), ORACLE]
    return command + ['--chdir', SOURCE, '--', *argv]


def artifacts(root, names):
    result = []
    total = 0
    for name in names:
        path = root / name
        if any(p.is_symlink() for p in (path, *path.parents)) or not path.is_file():
            raise ValueError('declared artifact missing or symlinked')
        size = path.stat().st_size
        total += size
        if size > MAX_ARTIFACT or total > MAX_ARTIFACTS:
            raise ValueError('workload artifact exceeds bounded retrieval contract')
        with path.open('rb') as stream:
            sha = hashlib.file_digest(stream, 'sha256').hexdigest()
        result.append({'path':name, 'size':size, 'sha256':sha})
    return result


def main():
    envelope = json.load(sys.stdin)
    admitted = read_json(Path('/home/bench/input/request.json'))
    if digest(admitted) != envelope['input']['identity']:
        raise ValueError('workload activity differs from admitted inputs')
    raw = Path('/home/bench/input/workload.json').read_bytes()
    if hashlib.sha256(raw).hexdigest() != admitted['manifest_sha256']:
        raise ValueError('workload manifest differs')
    declared = manifest(raw)
    attempt = Path(envelope['attempt_dir'])
    workspace, output, home = (attempt / name for name in ('workspace','artifacts','home'))
    output.mkdir(mode=0o700)
    home.mkdir(mode=0o700)
    environment = dict(os.environ, HOME=str(home), GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL='/dev/null',
                       PYTHONPATH='/workspace', PYTHONNOUSERSITE='1')
    from durable.estate.workspaces import materialize
    materialize('/home/bench/input/source.bundle', admitted['base'], workspace)
    binding = {**admitted, 'profile':declared['profile']}
    def render(argv, *, oracle=False):
        rendered = [a.replace('{source}', SOURCE).replace('{output}', OUTPUT) for a in argv]
        return [ORACLE+'/'+a[len('/oracle/'):] if oracle and a.startswith('/oracle/') else a for a in rendered]
    execution = command(sandbox_argv(workspace, output, render(declared['command']), binding),
                        cwd=attempt, env={'PATH':'/usr/bin:/bin'})
    # Always provide a private oracle directory, even for inline checks: this
    # selects the read-only source/output projection for every acceptance check.
    oracle = Path('/home/bench/input/oracles')
    if not oracle.exists():
        oracle = attempt/'empty-oracle'
        oracle.mkdir(mode=0o700)
    checks = [command(sandbox_argv(workspace, output, render(check,oracle=True), binding, oracle=oracle),
                      cwd=attempt, env={'PATH':'/usr/bin:/bin'}) for check in declared['checks']]
    passed = execution['exit_code'] == 0 and all(c['exit_code'] == 0 for c in checks)
    produced = artifacts(output, declared['artifacts'])
    publish_result(envelope, {'schema':1, 'identity':envelope['input']['identity'],
        'attempt_id':envelope['attempt_id'], 'source_commit':admitted['base'],
        'task_revision':admitted['task_revision'], 'manifest_sha256':admitted['manifest_sha256'],
        'execution_contract':admitted['execution_contract'], 'passed':passed,
        'command':execution, 'checks':checks, 'artifacts':produced})


if __name__ == '__main__':
    main()

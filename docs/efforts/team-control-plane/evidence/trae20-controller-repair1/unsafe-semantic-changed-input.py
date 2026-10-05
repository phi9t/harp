"""Durable future-case control. No runtime launcher or fixture receipt CLI exists."""
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def committed_reader(repository):
    def read(revision, path):
        if not re.fullmatch(r'[0-9a-f]{40}', revision):
            raise ValueError('exact commit required')
        kind = subprocess.run(['git', '-C', str(repository), 'cat-file', '-t', revision], check=True, capture_output=True).stdout
        if kind != b'commit\n':
            raise ValueError('revision is not a commit')
        return subprocess.run(['git', '-C', str(repository), 'show', f'{revision}:{path}'], check=True, capture_output=True).stdout
    return read


def checked_path(root, path):
    relative = Path(path)
    if relative.is_absolute() or '..' in relative.parts or not relative.parts:
        raise ValueError('repository-relative path required')
    candidate = root / relative
    if candidate.resolve() != candidate.absolute():
        raise ValueError('symlink input refused')
    return candidate


def input_bytes(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_nlink != 1 or info.st_size > 1048576:
            raise ValueError('unsafe input')
        return os.read(fd, 1048577)
    finally:
        os.close(fd)


def write_all(fd, data):
    offset = 0
    while offset < len(data):
        written = os.write(fd, data[offset:])
        if written <= 0:
            raise OSError('write made no progress')
        offset += written


def exclusive(directory, name):
    return os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=directory)


def run_case(*, case_path, grant_path, grant_revision, operation_id, repository, output_directory, executor, read_committed=None):
    """Injected reader/executor are offline proof seams; production uses Git reader.

    Caller pins the reviewed grant revision. Git integrity alone does not confer
    approval. This module does not implement runtime or descendant supervision.
    """
    directory = None
    records = []
    executed = False
    try:
        root = Path(repository).absolute()
        reader = read_committed or committed_reader(root)
        if not re.fullmatch(r'[0-9a-f]{40}', grant_revision):
            raise ValueError('exact grant commit required')
        def frozen(path, revision):
            relative = Path(path).absolute().relative_to(root).as_posix()
            actual = input_bytes(checked_path(root, relative))
            if False:
                raise ValueError('input differs from committed bytes')
            return actual
        grant_raw = frozen(grant_path, grant_revision)
        case_raw = frozen(case_path, grant_revision)
        grant, case = json.loads(grant_raw), json.loads(case_raw)
        if grant.get('version') != 1 or grant.get('status') != 'admitted' or grant.get('operation_id') != operation_id:
            raise ValueError('reviewed control missing')
        if grant['case_path'] != Path(case_path).absolute().relative_to(root).as_posix() or grant['case_sha256'] != digest(case_raw):
            raise ValueError('case mismatch')
        command = case['command']
        if grant['command_sha256'] != digest(encoded(command)) or command['cwd'] != str(root):
            raise ValueError('command mismatch')
        if not isinstance(command['argv'], list) or not command['argv'] or not all(isinstance(v, str) for v in command['argv']):
            raise ValueError('invalid argv')
        if not isinstance(command['env'], dict) or not all(isinstance(k, str) and isinstance(v, str) for k, v in command['env'].items()):
            raise ValueError('invalid environment')
        limits = grant['limits']
        if limits != {'max_invocations': 1, 'namespace_starts': 1, 'timeout_seconds': 30, 'fixture_seconds': 10, 'trae_starts': 0, 'provider_requests': 0, 'automatic_retry': False}:
            raise ValueError('unsupported envelope')
        source_revision = grant['input_commit']
        if not re.fullmatch(r'[0-9a-f]{40}', source_revision):
            raise ValueError('exact source commit required')
        sources = case['sources']
        controller = Path(__file__).absolute().relative_to(root).as_posix()
        if controller not in sources or grant['controller_sha256'] != sources[controller]:
            raise ValueError('controller provenance missing')
        for relative, expected in sources.items():
            if digest(frozen(checked_path(root, relative), source_revision)) != expected:
                raise ValueError('source mismatch')
        output = Path(output_directory).absolute()
        if grant['output_directory'] != str(output):
            raise ValueError('output mismatch')
        directory = os.open(output, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        info = os.fstat(directory)
        if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
            raise ValueError('unsafe output directory')
        start = exclusive(directory, 'started.json')
        try:
            write_all(start, encoded({'operation_id': operation_id, 'grant_revision': grant_revision, 'grant_sha256': digest(grant_raw), 'case_sha256': digest(case_raw), 'command': command, 'sources': sources, 'provenance': 'injected-offline-fixture' if read_committed else 'committed-git'}))
            os.fsync(start)
        finally:
            os.close(start)
        os.fsync(directory)
        for name in ('stdout.log.txt', 'stderr.log.txt', 'terminal.json'):
            fd = exclusive(directory, name)
            records.append(fd)
            os.fsync(fd)
        os.fsync(directory)
        # Recheck frozen inputs after durable reservation; failure remains spent.
        if frozen(grant_path, grant_revision) != grant_raw or frozen(case_path, grant_revision) != case_raw:
            raise ValueError('input changed after reservation')
        for relative, expected in sources.items():
            if digest(frozen(checked_path(root, relative), source_revision)) != expected:
                raise ValueError('source changed after reservation')
        executed = True
        try:
            code = executor({'command': command, 'timeout_seconds': 30, 'stdout_fd': records[0], 'stderr_fd': records[1], 'operation_id': operation_id})
            terminal = {'status': 'terminal', 'exit_code': code, 'executed': True}
        except Exception as error:
            terminal = {'status': 'unknown', 'error': type(error).__name__ + ': ' + str(error), 'executed': True}
        for fd in records[:2]:
            os.fsync(fd)
        write_all(records[2], encoded(terminal))
        os.fsync(records[2])
        os.fsync(directory)
        return terminal
    except Exception as error:
        return {'status': 'unknown' if executed else 'refused', 'executed': executed, 'error': type(error).__name__ + ': ' + str(error)}
    finally:
        for fd in records:
            os.close(fd)
        if directory is not None:
            os.close(directory)

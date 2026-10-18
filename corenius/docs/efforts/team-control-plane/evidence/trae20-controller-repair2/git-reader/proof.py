#!/usr/bin/env python3
"""Single-use real-Git integration proof; executor only writes recording logs."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
from unittest.mock import patch

PROOF = Path(__file__).resolve().parent
REPO = PROOF / 'fixture'
ROOT = Path('/data02/home/philip.yang/workspace/corenius-codex-reference-design')
REVIEWED = '16c15c86e9391939d5e6b5fc6c026e6a3a09fde1'
SOURCE_PATH = 'tools/probes/trae_case_control.py'
TEST_PATH = 'tools/probes/test_trae_case_control.py'
SOURCE_SHA = '9e6adfc1fe3c312776507676f29ba945eec75e96cc2bc73eeba86d63458bd207'
TEST_SHA = '7297a7a861acbd5c323d48070d31e80a0c700a7133f4533b861351c8e331db75'
OP = 'fixture-only-git-reader-20261003-NO-PHYSICAL-ADMISSION'
_real_run = subprocess.run
_real_fsync = os.fsync
commands = []
outcomes = []
calls = []
fsync_events = []
active_case = None


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def save(path, value):
    path.write_bytes(encoded(value) + b'\n')


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def process(argv, *, check=True, purpose='fixture-git'):
    # This proof permits actual subprocesses only for Git; command argv is data.
    require(argv[0] == 'git', 'non-Git process refused by proof')
    index = len(commands) + 1
    result = _real_run(argv, capture_output=True, check=False)
    stdout_path = PROOF / 'rawlogs' / f'{index:03d}.stdout'
    stderr_path = PROOF / 'rawlogs' / f'{index:03d}.stderr'
    stdout_path.write_bytes(result.stdout)
    stderr_path.write_bytes(result.stderr)
    event = {'index': index, 'argv': list(argv), 'purpose': purpose,
             'control_case': active_case, 'exit_code': result.returncode,
             'stdout': stdout_path.relative_to(PROOF).as_posix(),
             'stdout_sha256': sha(result.stdout),
             'stderr': stderr_path.relative_to(PROOF).as_posix(),
             'stderr_sha256': sha(result.stderr)}
    commands.append(event)
    with (PROOF / 'commands.jsonl').open('ab') as fd:
        fd.write(encoded(event) + b'\n')
    print(json.dumps({'git_command': index, 'argv': argv,
                      'exit_code': result.returncode, 'purpose': purpose}), flush=True)
    if check and result.returncode:
        raise subprocess.CalledProcessError(result.returncode, argv,
                                            result.stdout, result.stderr)
    return result


def git(*args, repository=REPO, check=True, purpose='fixture-git'):
    return process(['git', '-C', str(repository), *args], check=check,
                   purpose=purpose).stdout


def reader_process(argv, **kwargs):
    # Observe the default reader's actual Git calls, forwarding to real Git.
    # Neither committed_reader nor read_committed is replaced.
    require(argv[:3] == ['git', '-C', str(REPO)], 'reader escaped fixture')
    require(argv[3] in ('cat-file', 'show'), 'unexpected reader Git command')
    require(kwargs == {'check': True, 'capture_output': True},
            'unexpected reader process options')
    return process(argv, purpose='actual-default-committed-reader')


def observe_fsync(fd):
    result = _real_fsync(fd)
    info = os.fstat(fd)
    fsync_events.append({'fd': fd, 'path': os.readlink(f'/proc/self/fd/{fd}'),
                         'directory': stat.S_ISDIR(info.st_mode),
                         'before_callback': len(calls) == 0})
    return result


def invoke(name, **overrides):
    global active_case
    active_case = name
    before = len(commands)
    options = {'case_path': case_path, 'grant_path': grant_path,
               'grant_revision': grant_commit, 'operation_id': OP,
               'repository': REPO, 'output_directory': output,
               'executor': record}
    options.update(overrides)
    require('read_committed' not in options, 'injected reader is forbidden')
    with patch.object(module.subprocess, 'run', reader_process), \
         patch.object(module.os, 'fsync', observe_fsync):
        result = module.run_case(**options)
    event = {'name': name, 'result': result, 'callbacks_total': len(calls),
             'actual_reader_git_commands': len(commands) - before,
             'read_committed_argument': 'OMITTED'}
    outcomes.append(event)
    save(PROOF / 'outcomes' / (name + '.json'), event)
    print(json.dumps({'outcome': name, **event}), flush=True)
    active_case = None
    return result


def refuse(name, expected_error, **overrides):
    result = invoke(name, **overrides)
    require(result['status'] == 'refused' and result['executed'] is False,
            name + ' must refuse')
    require(expected_error in result['error'], name + ' wrong refusal cause')
    require(not calls, name + ' reached executor')
    require(not list(output.iterdir()), name + ' wrote operation records')


def record(invocation):
    require(not calls, 'recording callback budget exhausted')
    require(invocation['command'] == case['command'], 'callback command differs')
    require(invocation['operation_id'] == OP and invocation['timeout_seconds'] == 30,
            'callback identity/envelope differs')
    started = json.loads((output / 'started.json').read_bytes())
    require(started == expected_started, 'durable started binding differs')
    pre = [e for e in fsync_events if e['before_callback']]
    require(len(pre) >= 6, 'reservation/output fsyncs missing before callback')
    require(any(e['path'] == str(output / 'started.json') and not e['directory']
                for e in pre), 'started file not fsynced')
    require(sum(e['directory'] and e['path'] == str(output) for e in pre) >= 2,
            'reservation/output directory fsyncs missing')
    for key, filename in (('stdout_fd', 'stdout.log.txt'),
                          ('stderr_fd', 'stderr.log.txt')):
        require(os.readlink(f'/proc/self/fd/{invocation[key]}') == str(output / filename),
                'callback output descriptor mismatch')
    # Do not interpret or execute the command argv. This is the only executor.
    calls.append(dict(invocation))
    save(PROOF / 'recording-callback.json', invocation)
    module.write_all(invocation['stdout_fd'], b'RECORDED ONLY; CASE ARGV NEVER EXECUTED\n')
    module.write_all(invocation['stderr_fd'], b'FIXTURE ONLY; NO PHYSICAL ADMISSION\n')
    return 0


def main():
    global module, case_path, grant_path, case, grant, grant_commit, output, expected_started
    require(not REPO.exists(), 'fixture path must be exclusive and fresh')
    for name in ('rawlogs', 'outcomes', 'control-inputs', 'records', 'preimages'):
        (PROOF / name).mkdir(mode=0o700)
    # Ignore inherited Git configuration, hooks, signing, credential and exec options.
    for key in list(os.environ):
        if key.startswith('GIT_'):
            del os.environ[key]
    safe_env = {'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_SYSTEM': '/dev/null',
                'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_TERMINAL_PROMPT': '0',
                'GIT_CONFIG_COUNT': '4', 'GIT_CONFIG_KEY_0': 'core.hooksPath',
                'GIT_CONFIG_VALUE_0': '/dev/null', 'GIT_CONFIG_KEY_1': 'commit.gpgSign',
                'GIT_CONFIG_VALUE_1': 'false', 'GIT_CONFIG_KEY_2': 'tag.gpgSign',
                'GIT_CONFIG_VALUE_2': 'false', 'GIT_CONFIG_KEY_3': 'core.fsmonitor',
                'GIT_CONFIG_VALUE_3': 'false', 'GIT_AUTHOR_NAME': 'Corenius Fixture Only',
                'GIT_AUTHOR_EMAIL': 'fixture-only@invalid.example',
                'GIT_COMMITTER_NAME': 'Corenius Fixture Only',
                'GIT_COMMITTER_EMAIL': 'fixture-only@invalid.example',
                'GIT_AUTHOR_DATE': '2026-10-03T00:00:00+0000',
                'GIT_COMMITTER_DATE': '2026-10-03T00:00:00+0000'}
    os.environ.update(safe_env)
    save(PROOF / 'git-environment.json', safe_env)
    source = git('show', REVIEWED + ':' + SOURCE_PATH, repository=ROOT,
                 purpose='read-only-reviewed-source')
    tests = git('show', REVIEWED + ':' + TEST_PATH, repository=ROOT,
                purpose='read-only-reviewed-tests-hash')
    require(sha(source) == SOURCE_SHA, 'reviewed source SHA mismatch')
    require(sha(tests) == TEST_SHA, 'reviewed test SHA mismatch')
    (PROOF / 'preimages' / 'reviewed-source.py').write_bytes(source)
    (PROOF / 'preimages' / 'reviewed-tests.py').write_bytes(tests)
    REPO.mkdir(mode=0o700)
    git('init', '--initial-branch=fixture-only')
    module_path = REPO / SOURCE_PATH
    module_path.parent.mkdir(parents=True)
    module_path.write_bytes(source)
    git('add', '--', SOURCE_PATH)
    git('commit', '-m', 'Synthetic fixture source C; no physical admission')
    source_commit = git('rev-parse', 'HEAD').decode().strip()
    require(git('rev-list', '--parents', '-n', '1', source_commit).decode().strip()
            == source_commit, 'source C must have no shared parent/history')
    require(git('show', source_commit + ':' + SOURCE_PATH) == source,
            'source C differs')
    spec = importlib.util.spec_from_file_location('qualified_fixture_controller', module_path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    sys.dont_write_bytecode = True
    spec.loader.exec_module(module)
    require(Path(module.__file__) == module_path, 'loaded module path differs')
    output = PROOF / 'records' / 'operation'
    output.mkdir(mode=0o700)
    (REPO / 'fixtures').mkdir()
    case_path, grant_path = REPO / 'fixtures/case.json', REPO / 'fixtures/grant.json'
    case = {'fixture_only': True, 'physical_admission': False,
            'description': 'Synthetic binding fixture. argv is recorded data and never executed.',
            'command': {'argv': ['/fixture-recording-only/NEVER-EXECUTE'],
                        'env': {'FIXTURE_ONLY': 'NO_PHYSICAL_ADMISSION'}, 'cwd': str(REPO)},
            'sources': {SOURCE_PATH: SOURCE_SHA}}
    case_raw = encoded(case)
    case_path.write_bytes(case_raw)
    grant = {'version': 1, 'status': 'admitted', 'fixture_only': True,
             'physical_admission': False,
             'authority_note': 'Synthetic admitted marker exercises reader binding only; grants no physical admission.',
             'operation_id': OP, 'case_path': 'fixtures/case.json',
             'case_sha256': sha(case_raw), 'command_sha256': sha(encoded(case['command'])),
             'controller_sha256': SOURCE_SHA, 'input_commit': source_commit,
             'output_directory': str(output),
             'limits': {'max_invocations': 1, 'namespace_starts': 1, 'timeout_seconds': 30,
                        'fixture_seconds': 10, 'trae_starts': 0, 'provider_requests': 0,
                        'automatic_retry': False}}
    grant_raw = encoded(grant)
    grant_path.write_bytes(grant_raw)
    git('add', '--', 'fixtures/case.json', 'fixtures/grant.json')
    os.environ['GIT_AUTHOR_DATE'] = os.environ['GIT_COMMITTER_DATE'] = '2026-10-03T00:00:01+0000'
    git('commit', '-m', 'Synthetic case/grant G binds immutable C; fixture only')
    grant_commit = git('rev-parse', 'HEAD').decode().strip()
    require(git('rev-list', '--parents', '-n', '1', grant_commit).decode().strip()
            == grant_commit + ' ' + source_commit, 'G parent must be owned C')
    for label, path, original in (('changed-source', module_path, source),
                                 ('changed-case', case_path, case_raw),
                                 ('changed-grant', grant_path, grant_raw)):
        changed = original + b'\n'
        (PROOF / 'control-inputs' / (label + '.bytes')).write_bytes(changed)
        path.write_bytes(changed)
        try:
            refuse(label, 'input differs from committed bytes')
        finally:
            path.write_bytes(original)
        require(path.read_bytes() == original, 'own fixture input restore failed')
    refuse('wrong-operation', 'reviewed control missing', operation_id=OP + '-WRONG')
    refuse('stale-source-commit-as-grant', 'CalledProcessError', grant_revision=source_commit)
    refuse('unknown-grant-commit', 'CalledProcessError', grant_revision='0' * 40)
    grant_blob = git('rev-parse', grant_commit + ':fixtures/grant.json').decode().strip()
    refuse('blob-as-grant-commit', 'revision is not a commit', grant_revision=grant_blob)
    refuse('abbreviated-grant-commit', 'exact grant commit required', grant_revision=grant_commit[:12])
    refuse('wrong-output-binding', 'output mismatch', output_directory=PROOF / 'records/wrong-output')
    expected_started = {'operation_id': OP, 'grant_revision': grant_commit,
                        'grant_sha256': sha(grant_raw), 'case_sha256': sha(case_raw),
                        'command': case['command'], 'sources': case['sources'],
                        'provenance': 'committed-git'}
    binding = {key: expected_started[key] for key in ('operation_id', 'grant_revision',
               'grant_sha256', 'case_sha256', 'sources')}
    binding['command_sha256'] = sha(encoded(case['command']))
    terminal = invoke('positive-default-git-reader')
    require(terminal == {**binding, 'status': 'terminal', 'executed': True, 'exit_code': 0},
            'terminal binding/exit mismatch')
    require(len(calls) == 1, 'exactly one recording callback required')
    require(outcomes[-1]['actual_reader_git_commands'] == 12,
            'positive path must perform actual six Git reads (12 Git commands)')
    require(json.loads((output / 'terminal.json').read_bytes()) == terminal,
            'terminal record differs from result')
    require((output / 'stdout.log.txt').read_bytes() == b'RECORDED ONLY; CASE ARGV NEVER EXECUTED\n',
            'stdout record differs')
    require((output / 'stderr.log.txt').read_bytes() == b'FIXTURE ONLY; NO PHYSICAL ADMISSION\n',
            'stderr record differs')
    before_duplicate = {p.name: p.read_bytes() for p in output.iterdir()}
    save(PROOF / 'record-hashes-before-duplicate.json',
         {name: sha(data) for name, data in before_duplicate.items()})
    duplicate = invoke('duplicate-default-git-reader')
    require(duplicate['status'] == 'refused' and duplicate['executed'] is False,
            'duplicate must refuse')
    require('FileExistsError' in duplicate['error'], 'duplicate reservation cause differs')
    require(len(calls) == 1, 'duplicate reached callback')
    require({p.name: p.read_bytes() for p in output.iterdir()} == before_duplicate,
            'duplicate changed prior bytes or names')
    save(PROOF / 'record-hashes-after-duplicate.json',
         {name: sha(data) for name, data in before_duplicate.items()})
    save(PROOF / 'fsync-events.json', fsync_events)
    require(not git('status', '--porcelain'), 'fixture inputs must finish clean')
    refs = git('for-each-ref', '--format=%(objectname) %(refname)').decode().splitlines()
    require(refs == [grant_commit + ' refs/heads/fixture-only'], 'unexpected fixture refs')
    commits = git('rev-list', '--all').decode().splitlines()
    require(commits == [grant_commit, source_commit], 'unexpected fixture commits/history')
    paths = git('ls-tree', '-r', '--name-only', grant_commit).decode().splitlines()
    require(paths == ['fixtures/case.json', 'fixtures/grant.json', SOURCE_PATH],
            'bundle source tree contains unexpected paths')
    object_listing = git('rev-list', '--objects', '--all').decode()
    (PROOF / 'fixture-objects.txt').write_text(object_listing)
    git('fsck', '--full', '--no-reflogs')
    bundle = PROOF / 'fixture-only.bundle'
    git('bundle', 'create', str(bundle), '--all')
    git('bundle', 'verify', str(bundle))
    bundle_refs = git('bundle', 'list-heads', str(bundle)).decode().splitlines()
    require(sorted(bundle_refs) == sorted([grant_commit + ' HEAD',
                grant_commit + ' refs/heads/fixture-only']), 'unexpected bundle refs')
    bundle_header = bundle.read_bytes().split(b'\n\n', 1)[0].decode()
    require(not any(line.startswith('-') for line in bundle_header.splitlines()),
            'bundle unexpectedly requires shared history')
    save(PROOF / 'fixture-identities.json', {'reviewed_root_commit': REVIEWED,
         'source_commit_C': source_commit, 'grant_commit_G': grant_commit,
         'source_sha256': SOURCE_SHA, 'reviewed_tests_sha256': TEST_SHA,
         'case_sha256': sha(case_raw), 'grant_sha256': sha(grant_raw),
         'command_sha256': binding['command_sha256'], 'grant_blob': grant_blob,
         'bundle_sha256': sha(bundle.read_bytes()), 'bundle_refs': bundle_refs,
         'bundle_complete_history': True, 'fixture_refs': refs,
         'fixture_tree_paths': paths, 'physical_admission': False})
    save(PROOF / 'proof-result.json', {'status': 'PASS', 'controls_passed': 9,
         'controller_invocations': len(outcomes), 'recording_callbacks': len(calls),
         'actual_default_reader_git_commands': sum(e['purpose'] == 'actual-default-committed-reader' for e in commands),
         'total_git_commands': len(commands),
         'nonzero_git_exits_expected_controls': sum(e['exit_code'] != 0 for e in commands),
         'actual_reader_exercised': True, 'read_committed_argument': 'OMITTED',
         'started_provenance': 'committed-git', 'fixture_repositories': 1,
         'runtime_starts': 0, 'namespace_starts': 0, 'provider_requests': 0,
         'service_starts': 0, 'case_argv_executions': 0,
         'root_or_shared_mutations': 0, 'acceptance_authority': False,
         'limits': ['Only local Git-reader binding and durable recording boundary qualified.',
                    'No real runtime timeout or descendant enforcement demonstrated.',
                    'No physical runtime/provider/service/namespace qualification or admission.',
                    'Earlier chronological TDD gaps remain unchanged.',
                    'Original physical result remains PARTIAL; budgets/quarantines unchanged.',
                    'Runtime configuration, nested isolation, native turn-ID binding, terminal observation and request accounting remain unproved.']})
    print('PASS: 9 fail-closed controls; 11 controller calls; 1 recording callback; zero physical launches.', flush=True)


if __name__ == '__main__':
    main()

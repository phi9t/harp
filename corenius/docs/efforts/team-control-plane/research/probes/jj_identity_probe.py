#!/usr/bin/env python3
"""Model-free jj identity preflight in disposable Git-backed repositories."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

PIN = 'af45d57de7163cc5f17f3df178a31577e4951ea3'
BINARY_SHA256 = '15fe28e44398552177ce01ba69dd17b52151e8e604ccf82fe587acd40ba13be4'
BASE = '963f58b31e518a8162dfe719e386ef9d9843321f'


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def run(args):
    binary = Path(args.binary).resolve(strict=True)
    assert sha256(binary.read_bytes()) == BINARY_SHA256, 'unexpected jj executable'
    git = shutil.which('git')
    assert git, 'Git is required'
    result = {'corenius_base': BASE, 'jj_source_pin': PIN,
              'jj_binary_sha256': BINARY_SHA256, 'guard': args.guard, 'events': []}
    temporary = tempfile.TemporaryDirectory(prefix='corenius-jj-preflight-')
    failure = None
    try:
        root = Path(temporary.name)
        # A clean environment avoids inherited Git/JJ configuration and credentials.
        env = {'PATH': '/usr/bin:/bin', 'JJ_CONFIG': '',
               'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_SYSTEM': '/dev/null',
               'GIT_TERMINAL_PROMPT': '0', 'GIT_EDITOR': '/bin/false',
               'EDITOR': '/bin/false', 'VISUAL': '/bin/false',
               'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.invalid',
               'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.invalid',
               'XDG_CONFIG_HOME': str(root / 'config'), 'XDG_CACHE_HOME': str(root / 'cache')}

        def command(argv, cwd=root):
            output = subprocess.run(argv, cwd=cwd, env=env, capture_output=True,
                                    text=True, timeout=20)
            result['events'].append({'argv': argv, 'exit': output.returncode,
                                     'stdout': output.stdout, 'stderr': output.stderr})
            if output.returncode:
                raise RuntimeError('fixture command failed: ' + repr(argv))
            return output.stdout.strip()

        def jj(repo, *argv):
            return command([str(binary), '--config', 'user.name="Fixture"', '--config',
                            'user.email="fixture@example.invalid"', '--config',
                            'git.colocate=false', '--repository', str(repo), *argv])

        result['version'] = command([str(binary), '--version'])
        assert PIN in result['version'] and '0.44.0' in result['version']
        repo = root / 'source'
        command([str(binary), '--config', 'user.name="Fixture"', '--config',
                 'user.email="fixture@example.invalid"', 'git', 'init', '--no-colocate', str(repo)])
        store = repo / '.jj/repo/store/git'

        def git_store(*argv):
            return command([git, '-c', 'core.hooksPath=/dev/null', '--git-dir=' + str(store), *argv])

        def identity(workspace, revision='@'):
            values = jj(workspace, 'log', '--no-graph', '--revisions', revision,
                        '--template', 'commit_id ++ " " ++ change_id ++ " " ++ conflict').split()
            assert len(values) == 3 and values[2] == 'false', 'candidate must be conflict-free'
            return {'commit': values[0], 'change': values[1],
                    'tree': git_store('rev-parse', values[0] + '^{tree}'), 'conflict': False}

        (repo / '.gitignore').write_text('private.env\n')
        (repo / 'private.env').write_text('SYNTHETIC_PRIVATE_VALUE_DO_NOT_CAPTURE\n')
        (repo / 'artifact.txt').write_text('verified-original\n')
        before = identity(repo)
        captured = git_store('ls-tree', '--name-only', before['commit']).splitlines()
        assert captured == ['.gitignore', 'artifact.txt'], captured
        old_bytes = git_store('show', before['commit'] + ':artifact.txt') + '\n'
        result['before'] = before
        result['captured_paths'] = captured
        result['ignored_private_excluded'] = 'private.env' not in captured
        result['verified_file_sha256'] = sha256(old_bytes.encode())
        jj(repo, 'git', 'export')
        git_store('update-ref', 'refs/fixture/candidate', before['commit'])
        bundle = root / 'candidate.bundle'
        git_store('bundle', 'create', str(bundle), 'refs/fixture/candidate')
        result['bundle_sha256'] = sha256(bundle.read_bytes())

        (repo / 'artifact.txt').write_text('rewritten-source\n')
        after = identity(repo)
        result['after'] = after
        assert before['change'] == after['change'], 'fixture must exercise stable change identity'
        assert before['commit'] != after['commit'] and before['tree'] != after['tree']
        # RED mode intentionally exposes a weak evidence guard. It is not a CLI failure.
        guard_accepts_changed = (before['change'] == after['change'] if args.guard == 'change-id'
                                 else (before['commit'], before['tree']) == (after['commit'], after['tree']))
        result['guard_accepts_changed_candidate'] = guard_accepts_changed
        assert not guard_accepts_changed, 'evidence guard incorrectly accepted rewritten candidate'

        verifier = root / 'verifier'
        verifier.mkdir()
        command([git, '-c', 'core.hooksPath=/dev/null', 'init', str(verifier)])
        command([git, '-c', 'core.hooksPath=/dev/null', '-C', str(verifier), 'fetch',
                 str(bundle), 'refs/fixture/candidate'])
        retained_commit = command([git, '-C', str(verifier), 'rev-parse', 'FETCH_HEAD'])
        retained_tree = command([git, '-C', str(verifier), 'rev-parse', 'FETCH_HEAD^{tree}'])
        retained_bytes = command([git, '-C', str(verifier), 'show', 'FETCH_HEAD:artifact.txt']) + '\n'
        assert retained_commit == before['commit'] and retained_tree == before['tree']
        assert retained_bytes == old_bytes and retained_bytes != (repo / 'artifact.txt').read_text()
        retained_paths = command([git, '-C', str(verifier), 'ls-tree', '--name-only', 'FETCH_HEAD']).splitlines()
        assert retained_paths == captured
        assert sha256(bundle.read_bytes()) == result['bundle_sha256']
        result['independent_import'] = {'commit': retained_commit, 'tree': retained_tree,
                                        'file_sha256': sha256(retained_bytes.encode()),
                                        'captured_paths': retained_paths}

        peer = root / 'peer'
        jj(repo, 'workspace', 'add', '--name', 'peer', str(peer))
        observed_before = identity(peer, after['change'])
        jj(peer, 'describe', after['change'], '--message', 'rewritten by peer workspace')
        observed_after = identity(peer, after['change'])
        assert observed_before['commit'] == after['commit']
        assert observed_before['change'] == observed_after['change']
        assert observed_before['commit'] != observed_after['commit']
        assert observed_before['tree'] == observed_after['tree']
        result['peer_rewrite'] = {'before': observed_before, 'after': observed_after,
                                 'another_workspace_can_rewrite_change': True}
        result['outcome'] = 'passed'
    except Exception as error:
        result['outcome'] = 'failed'
        result['failure'] = str(error)
        failure = error
    finally:
        temporary.cleanup()
        result['fixture_removed'] = not Path(temporary.name).exists()
    Path(args.output).write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'events'}, indent=2))
    if failure:
        raise failure


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--guard', choices=['exact', 'change-id'], default='exact',
                        help='change-id intentionally demonstrates a failing identity guard')
    run(parser.parse_args())

#!/usr/bin/env python3
"""Disposable, model-free workspace-interface evaluation; not a production backend."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

PIN = 'af45d57de7163cc5f17f3df178a31577e4951ea3'
BINARY_SHA = '15fe28e44398552177ce01ba69dd17b52151e8e604ccf82fe587acd40ba13be4'
BASE = '264756edd550225449202b6f20ac4b560808d14e'
ROOT = Path(__file__).resolve().parents[5]


def digest(data):
    return hashlib.sha256(data).hexdigest()


class Refused(RuntimeError):
    pass


class Evaluation:
    def __init__(self, binary, unsafe):
        self.binary = Path(binary).resolve(strict=True)
        assert digest(self.binary.read_bytes()) == BINARY_SHA
        self.unsafe = unsafe
        self.tmp = tempfile.TemporaryDirectory(prefix='corenius-jj19-')
        self.root = Path(self.tmp.name)
        self.env = {'PATH': '/usr/bin:/bin', 'JJ_CONFIG': '',
                    'HOME': str(self.root / 'private-home'),
                    'XDG_CONFIG_HOME': str(self.root / 'config'),
                    'XDG_CACHE_HOME': str(self.root / 'cache'),
                    'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_SYSTEM': '/dev/null',
                    'GIT_CONFIG_NOSYSTEM': '1', 'GIT_TERMINAL_PROMPT': '0',
                    'GIT_EDITOR': '/bin/false', 'EDITOR': '/bin/false',
                    'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.invalid',
                    'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.invalid',
                    'PYTHONDONTWRITEBYTECODE': '1'}
        self.result = {'source_base': BASE, 'pin': PIN, 'binary_sha256': BINARY_SHA,
                       'fixture_sha256': digest(Path(__file__).read_bytes()),
                       'unsafe_control': unsafe, 'events': [], 'observations': {}}
        version = self.command([str(self.binary), '--version'])
        assert version == 'jj 0.44.0-' + PIN
        self.result['version'] = version

    def journal(self, entry):
        # Persist preparation before executing a command; replay observes facts, not rollback assumptions.
        with (self.root / 'operations.jsonl').open('a') as stream:
            stream.write(json.dumps(entry, sort_keys=True) + '\n')
            stream.flush()
            os.fsync(stream.fileno())

    def command(self, argv, cwd=None, check=True):
        self.journal({'phase': 'prepared', 'argv': [str(a) for a in argv],
                      'cwd': str(cwd or self.root)})
        p = subprocess.run([str(a) for a in argv], cwd=cwd or self.root, env=self.env,
                           capture_output=True, text=True, timeout=60)
        self.result['events'].append({'argv': [str(a) for a in argv], 'cwd': str(cwd or self.root),
                                      'exit': p.returncode, 'stdout': p.stdout, 'stderr': p.stderr})
        if check and p.returncode:
            raise RuntimeError(f'command failed ({p.returncode}): {argv!r}: {p.stderr}')
        return p.stdout.strip() if check else p

    def argv(self, repo, *args):
        return [str(self.binary), '--no-pager', '--color=never', '--config', 'user.name="Fixture"',
                '--config', 'user.email="fixture@example.invalid"', '--config', 'git.colocate=false',
                '--repository', str(repo), *args]

    def jj(self, repo, *args, check=True):
        return self.command(self.argv(repo, *args), cwd=repo, check=check)

    def git(self, repo, *args, check=True, bare=False):
        selector = '--git-dir=' + str(repo) if bare else '-C'
        prefix = ['git', '-c', 'core.hooksPath=/dev/null', selector]
        if not bare:
            prefix.append(str(repo))
        return self.command([*prefix, *args], check=check)

    def create_attempt(self, name):
        repo = self.root / name
        self.command([str(self.binary), '--config', 'user.name="Fixture"', '--config',
                      'user.email="fixture@example.invalid"', 'git', 'init', '--no-colocate', str(repo)])
        (repo / '.gitignore').write_text('private.env\n')
        (repo / 'artifact.txt').write_text('base\n')
        self.checkpoint(repo, snapshot_authority=True)
        self.jj(repo, 'new', '-m', 'isolated attempt at exact base')
        return repo

    def store(self, repo):
        repository = repo / '.jj/repo'
        if repository.is_file():
            repository = (repository.parent / repository.read_text().strip()).resolve()
        return repository / 'store/git'

    def operation(self, repo):
        return self.jj(repo, '--ignore-working-copy', '--at-operation=@', 'op', 'log', '--limit', '1', '--no-graph',
                       '--template', 'id')

    def inspect(self, repo, revision='@', operation=None):
        # Exact operation avoids both implicit snapshot and automatic head reconciliation.
        op = operation or self.operation(repo)
        answer = self.jj(repo, '--at-operation=' + op, 'log', '--no-graph', '-r', revision,
                         '-T', 'commit_id ++ " " ++ change_id ++ " " ++ conflict ++ "\\n"', check=False)
        if answer.returncode:
            if 'divergent' in answer.stderr:
                raise Refused('ambiguous change/revision')
            raise RuntimeError(answer.stderr)
        text = answer.stdout.strip()
        rows = [line.split() for line in text.splitlines()]
        if len(rows) != 1:
            raise Refused('ambiguous change/revision')
        commit, change, conflict = rows[0]
        tree = self.git(self.store(repo), 'rev-parse', commit + '^{tree}', bare=True)
        return {'backend': 'jj-git', 'store': str(self.store(repo).resolve()), 'workspace': str(repo),
                'operation': op, 'change': change, 'commit': commit, 'tree': tree,
                'conflict': conflict == 'true'}

    def checkpoint(self, repo, snapshot_authority):
        if not snapshot_authority:
            raise Refused('snapshot authority required')
        before = self.operation(repo)
        self.result['events'].append({'journal': 'checkpoint prepared', 'operation_before': before,
                                      'workspace': str(repo)})
        self.jj(repo, 'status')
        return self.inspect(repo)

    def freeze_candidate(self, repo, allowed=('.gitignore', 'artifact.txt'), revision='@'):
        candidate = self.inspect(repo, revision)
        paths = self.git(self.store(repo), 'ls-tree', '-r', '--name-only', candidate['commit'], bare=True).splitlines()
        bookmarks = self.jj(repo, '--at-operation=' + candidate['operation'], 'bookmark', 'list')
        if not self.unsafe and (candidate['conflict'] or '(conflicted)' in bookmarks):
            raise Refused('unresolved content/bookmark conflict')
        if not self.unsafe and (set(paths) - set(allowed) or any(p.startswith('private') for p in paths)):
            raise Refused('disallowed/private captured path')
        entries = self.git(self.store(repo), 'ls-tree', '-r', candidate['commit'], bare=True)
        if not self.unsafe and any(line.startswith(('120000 ', '160000 ')) for line in entries.splitlines()):
            raise Refused('symlink/submodule outside regular-file capture contract')
        candidate['captured_paths'] = paths
        candidate['base_commit'] = self.jj(repo, '--at-operation=' + candidate['operation'], 'log',
                                           '--no-graph', '-r', candidate['commit'] + '-', '-T', 'commit_id')
        return candidate

    def prepare_integration(self, repo, candidate, name='candidate'):
        # Exact immutable candidate, not a re-resolved change/bookmark/@ alias.
        self.jj(repo, 'git', 'export')
        store = self.store(repo)
        self.git(store, 'update-ref', 'refs/fixture/' + name, candidate['commit'], bare=True)
        bundle = self.root / (name + '.bundle')
        self.git(store, 'bundle', 'create', str(bundle), 'refs/fixture/' + name, bare=True)
        return bundle

    def release(self, repo):
        identity = self.inspect(repo)
        self.jj(repo, 'workspace', 'forget', 'default')
        self.result['observations'].setdefault('released', []).append(identity)

    def refused(self, fn, reason):
        try:
            fn()
        except Refused as error:
            self.result['observations'].setdefault('refusals', []).append({'expected': reason, 'actual': str(error)})
            return
        raise AssertionError('unsafe outcome accepted: ' + reason)

    def identity_slice(self):
        repo = self.create_attempt('identity')
        (repo / 'artifact.txt').write_text('verified-original\n')
        self.checkpoint(repo, True)
        original = self.freeze_candidate(repo)
        bundle = self.prepare_integration(repo, original)
        proof = digest(bundle.read_bytes())
        (repo / 'artifact.txt').write_text('rewritten-source\n')
        self.checkpoint(repo, True)
        changed = self.freeze_candidate(repo)
        assert original['change'] == changed['change']
        assert original['commit'] != changed['commit'] and original['tree'] != changed['tree']
        accepts = original['change'] == changed['change'] if self.unsafe else (original['commit'], original['tree']) == (changed['commit'], changed['tree'])
        self.result['observations']['rewrite'] = {'before': original, 'after': changed, 'guard_accepts': accepts}
        assert not accepts, 'mutable change identity accepted rewritten candidate'
        imported = self.root / 'verifier'; imported.mkdir()
        self.git(imported, 'init', '-q')
        self.git(imported, 'fetch', str(bundle), 'refs/fixture/candidate')
        assert self.git(imported, 'rev-parse', 'FETCH_HEAD') == original['commit']
        assert self.git(imported, 'rev-parse', 'FETCH_HEAD^{tree}') == original['tree']
        assert self.git(imported, 'show', 'FETCH_HEAD:artifact.txt') == 'verified-original'
        assert digest(bundle.read_bytes()) == proof
        # Ancestry/description-only rewrite also needs a new commit binding even with same tree.
        self.jj(repo, 'describe', '-m', 'new immutable candidate identity')
        metadata_change = self.freeze_candidate(repo)
        assert metadata_change['commit'] != changed['commit'] and metadata_change['tree'] == changed['tree']
        self.result['observations']['retained_bundle'] = {'sha256': proof, 'commit': original['commit'], 'tree': original['tree']}
        self.result['observations']['metadata_rewrite'] = metadata_change
        self.release(repo)

    def authority_slice(self):
        repo = self.create_attempt('authority'); other = self.create_attempt('isolated-other')
        other_before = self.inspect(other)
        before = self.inspect(repo)
        (repo / 'artifact.txt').write_text('unsnapshotted\n')
        if self.unsafe:
            self.jj(repo, 'status')
        else:
            self.inspect(repo, operation=before['operation'])
        after = self.inspect(repo)
        self.result['observations']['inspection'] = {'before': before, 'after': after}
        assert before == after, 'status inspection silently acquired snapshot authority'
        self.refused(lambda: self.checkpoint(repo, False), 'checkpoint without snapshot authority')
        self.checkpoint(repo, True)
        peer = self.root / 'peer'
        self.jj(repo, 'workspace', 'add', '--name', 'peer', str(peer))
        victim = self.inspect(repo)
        (peer / 'artifact.txt').write_text('trusted peer content rewrite\n')
        source = self.checkpoint(peer, True)
        self.jj(peer, 'restore', '--from', source['commit'], '--to', victim['commit'], 'artifact.txt')
        stale = self.jj(repo, 'status', check=False)
        assert stale.returncode != 0 and 'stale' in stale.stderr.lower(), 'expected actual stale workspace refusal'
        self.jj(repo, 'workspace', 'update-stale')
        repaired = self.inspect(repo)
        assert repaired['commit'] != victim['commit']
        # Raw shared store permits cross-workspace rewrites. Isolated stores preserve other attempt.
        assert self.inspect(other) == other_before
        op = self.operation(repo)
        current = self.inspect(repo)
        self.jj(repo, '--at-operation=' + op, 'describe', '-r', current['commit'], '-m', 'divergent-left')
        self.jj(repo, '--at-operation=' + op, 'describe', '-r', current['commit'], '-m', 'divergent-right')
        self.jj(repo, '--ignore-working-copy', 'status')
        merged = self.operation(repo)
        self.refused(lambda: self.freeze_candidate(repo, revision=current['change']), 'divergent change cannot freeze')
        assert merged != op and self.inspect(other) == other_before
        self.result['observations']['shared_store'] = {'victim': victim, 'repaired': repaired,
            'shared_rewrite_possible': True, 'isolated_other': other_before, 'divergent_base_operation': op,
            'reconciled_operation': merged, 'no_snapshot_status_can_reconcile_operations': True}
        self.jj(repo, 'workspace', 'forget', 'peer')
        self.release(other)

    def crash_slice(self):
        repo = self.create_attempt('crash'); other = self.create_attempt('crash-other')
        untouched = self.inspect(other)
        old = self.inspect(repo)
        self.jj(repo, 'new', '-m', 'changed checkout target')
        (repo / 'artifact.txt').write_text('after durable operation\n')
        changed = self.checkpoint(repo, True)
        calibration = self.root / 'calibration.strace'
        self.command(['strace', '-f', '-yy', '-e', 'trace=flock', '-o', str(calibration),
                      *self.argv(repo, 'edit', old['commit'])], cwd=repo)
        locks = [line for line in calibration.read_text().splitlines() if 'flock(' in line]
        wc = [n + 1 for n, line in enumerate(locks) if 'working_copy.lock' in line and 'LOCK_EX' in line]
        assert len(wc) == 2, 'calibration must locate snapshot and post-operation working-copy locks'
        boundary = wc[1]
        before = self.inspect(repo)
        before_bytes = (repo / 'artifact.txt').read_bytes()
        trace = self.root / 'crash.strace'
        argv = ['strace', '-f', '-yy', '-e', 'trace=flock', '-e',
                f'inject=flock:delay_enter=30s:when={boundary}', '-o', str(trace),
                *self.argv(repo, 'edit', changed['commit'])]
        self.result['events'].append({'journal': 'edit prepared', 'operation_before': before['operation'],
                                     'target_commit': changed['commit'], 'target_tree': changed['tree']})
        self.journal({'phase': 'prepared crash-boundary command', 'argv': argv,
                      'operation_before': before['operation'], 'target_commit': changed['commit']})
        child = subprocess.Popen(argv, cwd=repo, env=self.env, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, text=True, start_new_session=True)
        durable = None
        pids = []
        try:
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                assert child.poll() is None, 'crash boundary command must still be running'
                now = self.inspect(repo)
                if now['operation'] != before['operation']:
                    durable = now
                    pids = [int(p) for p in Path(f'/proc/{child.pid}/task/{child.pid}/children').read_text().split()]
                    break
                time.sleep(.02)
            assert durable is not None, 'repository operation did not reach durable pre-checkout boundary'
            assert durable['commit'] == changed['commit']
            assert (repo / 'artifact.txt').read_bytes() == before_bytes, 'working copy already updated before crash'
            wc_state = self.jj(repo, '--ignore-working-copy', '--at-operation=' + durable['operation'],
                               'debug', 'working-copy')
        finally:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate(timeout=10)
        self.result['events'].append({'argv': argv, 'cwd': str(repo), 'exit': child.returncode,
                                      'stdout': stdout, 'stderr': stderr})
        assert child.returncode == -signal.SIGKILL
        deadline = time.monotonic() + 3
        while any(Path(f'/proc/{pid}').exists() for pid in pids) and time.monotonic() < deadline:
            time.sleep(.02)
        self.result['observations']['crash'] = {'before': before, 'durable_before_kill': durable,
            'working_copy_before_kill': wc_state, 'disk_bytes_unchanged': True,
            'strace_lock_ordinal': boundary, 'calibration': calibration.read_text(),
            'fault_trace': trace.read_text(), 'owned_tracer_pid': child.pid, 'owned_jj_pids': pids,
            'terminal': child.returncode, 'tracer_waited': True,
            'jj_proc_entries_absent': all(not Path(f'/proc/{pid}').exists() for pid in pids)}
        assert self.result['observations']['crash']['jj_proc_entries_absent']
        if self.unsafe:
            assert self.operation(repo) == before['operation'], 'SIGKILL did not roll back durable repository operation'
        stale = self.jj(repo, 'status', check=False)
        assert stale.returncode != 0 and 'stale' in stale.stderr.lower()
        self.jj(repo, 'workspace', 'update-stale')
        recovered = self.inspect(repo)
        assert recovered['commit'] == changed['commit'] and recovered['tree'] == changed['tree']
        assert (repo / 'artifact.txt').read_text() == 'after durable operation\n'
        assert self.inspect(other) == untouched
        self.result['observations']['recovered'] = recovered
        self.release(repo); self.release(other)

    def exclusions_slice(self):
        repo = self.create_attempt('private')
        (repo / 'private.env').write_text('SYNTHETIC_PRIVATE_DO_NOT_EXPORT\n')
        self.checkpoint(repo, True)
        clean = self.freeze_candidate(repo)
        assert clean['captured_paths'] == ['.gitignore', 'artifact.txt']
        # Already tracked private content remains tracked after adding an ignore rule.
        (repo / '.gitignore').write_text('')
        self.checkpoint(repo, True)
        (repo / '.gitignore').write_text('private.env\n')
        self.checkpoint(repo, True)
        paths = self.git(self.store(repo), 'ls-tree', '-r', '--name-only', self.inspect(repo)['commit'], bare=True)
        assert 'private.env' in paths.splitlines()
        self.result['observations']['ignored_private'] = {'fresh_excluded': clean, 'tracked_private_still_captured': True}
        self.refused(lambda: self.freeze_candidate(repo), 'ignore rules alone cannot remove tracked private content')
        (repo / 'private.env').unlink()
        (repo / 'unexpected.txt').write_text('outside captured path contract\n')
        self.checkpoint(repo, True)
        self.refused(lambda: self.freeze_candidate(repo), 'nonignored out-of-scope file')
        (repo / 'unexpected.txt').unlink()
        self.checkpoint(repo, True)
        # A real capsule allows this workspace write and denies a sibling write.
        sibling = self.root / 'outside-sentinel'; sibling.write_text('unchanged\n')
        capsule = self.command(['bwrap', '--unshare-all', '--die-with-parent', '--ro-bind', '/', '/',
            '--bind', str(repo), str(repo), '--chdir', str(repo), '/bin/sh', '-c',
            "printf '%s\n' capsule-allowed > artifact.txt; if printf forbidden > \"$1\" 2>/dev/null; then exit 9; fi",
            'fixture', str(sibling)], check=False)
        assert capsule.returncode == 0, 'real capsule must execute and deny outside write'
        assert sibling.read_text() == 'unchanged\n'
        self.result['observations']['capsule'] = {'exit': capsule.returncode, 'outside_unchanged': True,
                                                'workspace_write': (repo / 'artifact.txt').read_text()}
        self.checkpoint(repo, True)
        self.freeze_candidate(repo)
        (repo / 'artifact.txt').unlink()
        (repo / 'artifact.txt').symlink_to(sibling)
        self.checkpoint(repo, True)
        self.refused(lambda: self.freeze_candidate(repo), 'allowed-name symlink escapes regular-file capture contract')
        conflict = self.create_attempt('content-conflict'); base = self.inspect(conflict)
        self.jj(conflict, 'new', base['commit'], '-m', 'left')
        (conflict / 'artifact.txt').write_text('left\n'); left = self.checkpoint(conflict, True)
        self.jj(conflict, 'new', base['commit'], '-m', 'right')
        (conflict / 'artifact.txt').write_text('right\n'); right = self.checkpoint(conflict, True)
        self.jj(conflict, 'new', left['commit'], right['commit'], '-m', 'unresolved merge')
        conflicted = self.inspect(conflict); assert conflicted['conflict']
        exported = self.jj(conflict, 'git', 'export', check=False)
        self.refused(lambda: self.freeze_candidate(conflict), 'content conflict before export/integration')
        self.result['observations']['content_conflict'] = {'candidate': conflicted, 'export_exit': exported.returncode,
            'git_tree_entries': self.git(self.store(conflict), 'ls-tree', '-r', '--name-only', conflicted['commit'], bare=True)}
        bookmarks = self.create_attempt('bookmark-conflict'); base = self.inspect(bookmarks)
        self.jj(bookmarks, 'new', base['commit'], '-m', 'bookmark-left'); left = self.inspect(bookmarks)
        self.jj(bookmarks, 'new', base['commit'], '-m', 'bookmark-right'); right = self.inspect(bookmarks)
        self.jj(bookmarks, 'bookmark', 'create', 'target', '-r', base['commit'])
        self.jj(bookmarks, 'git', 'export')
        old_git_ref = self.git(self.store(bookmarks), 'rev-parse', 'refs/heads/target', bare=True)
        op = self.operation(bookmarks)
        self.jj(bookmarks, '--at-operation=' + op, 'bookmark', 'set', 'target', '-r', left['commit'])
        self.jj(bookmarks, '--at-operation=' + op, 'bookmark', 'set', 'target', '-r', right['commit'])
        self.jj(bookmarks, '--ignore-working-copy', 'status')
        state = self.jj(bookmarks, '--ignore-working-copy', 'bookmark', 'list')
        assert '(conflicted)' in state
        exported = self.jj(bookmarks, 'git', 'export', check=False)
        self.refused(lambda: self.freeze_candidate(bookmarks), 'bookmark ambiguity before export/integration')
        actual_ref = self.git(self.store(bookmarks), 'rev-parse', 'refs/heads/target', bare=True)
        assert actual_ref == old_git_ref
        self.result['observations']['bookmark_conflict'] = {'state': state, 'export_exit': exported.returncode,
                                                           'old_git_ref': old_git_ref, 'actual_git_ref': actual_ref}

    def integration_slice(self):
        repo = self.create_attempt('export'); base = self.inspect(repo)
        self.jj(repo, 'new', base['commit'], '-m', 'linear exported candidate')
        (repo / 'artifact.txt').write_text('integration-verified\n')
        self.checkpoint(repo, True)
        candidate = self.freeze_candidate(repo)
        bundle = self.prepare_integration(repo, candidate, 'integration')
        imported = self.root / 'git-integration'; imported.mkdir()
        self.git(imported, 'init', '-q', '-b', 'main')
        self.git(imported, 'fetch', str(bundle), 'refs/fixture/integration')
        self.git(imported, 'update-ref', 'refs/heads/main', base['commit'])
        self.git(imported, 'checkout', '-q', 'main')
        self.git(imported, 'branch', 'candidate', candidate['commit'])
        self.git(imported, 'config', 'user.name', 'Fixture')
        self.git(imported, 'config', 'user.email', 'fixture@example.invalid')
        assert self.git(imported, 'rev-parse', 'candidate') == candidate['commit']
        assert self.git(imported, 'rev-parse', 'candidate^{tree}') == candidate['tree']
        self.git(imported, 'merge-base', '--is-ancestor', base['commit'], candidate['commit'])
        land = ROOT / 'tools/agents/land.py'
        failure = self.command(['python3', str(land), 'candidate', '--repo', str(imported),
                                '--gate', 'exit 42'], check=False)
        assert failure.returncode == 5
        assert self.git(imported, 'rev-parse', 'main') == base['commit']
        self.result['observations']['failed_gate'] = {'exit': failure.returncode,
            'mainline_after': base['commit'], 'candidate': candidate}
        if self.unsafe:
            assert self.git(imported, 'rev-parse', 'main') == candidate['commit'], 'failed gate did not authorize landing'
        gate = ('test "$LAND_TIP" = "' + candidate['commit'] + '" && '
                'test "$(git rev-parse HEAD^{tree})" = "' + candidate['tree'] + '" && '
                'test "$(cat artifact.txt)" = "integration-verified"')
        landed = self.command(['python3', str(land), 'candidate', '--repo', str(imported), '--gate', gate])
        assert landed == candidate['commit']
        assert self.git(imported, 'rev-parse', 'main') == candidate['commit']
        assert self.git(imported, 'rev-parse', 'main^{tree}') == candidate['tree']
        # Existing evidence recorder measures the actual exported candidate checkout.
        run = self.root / 'evidence-run'; run.mkdir()
        run_data = {'name': 'jj19-model-free-fixture', 'worktree': str(imported), 'branch': 'main',
                    'base_sha': base['commit'], 'allowed': ['artifact.txt'], 'model': 'none',
                    'effort': 'local-vcs-fixture', 'sandbox': 'isolated-private-config'}
        (run / 'run.json').write_text(json.dumps(run_data))
        (run / 'prompt.md').write_text('Verify exact exported candidate, no provider.\n')
        (run / 'events.jsonl').write_text(json.dumps({'candidate_commit': candidate['commit'],
                                                  'candidate_tree': candidate['tree']}) + '\n')
        (run / 'verdict.json').write_text(json.dumps({'verdict': 'CLEAN', 'exit_code': 0}))
        (run / 'last-message.md').write_text('exact candidate gate passed\n')
        (run / 'stderr.log').write_text('')
        evidence_dir = self.root / 'evidence'; self.env['CORENIUS_EVIDENCE'] = str(evidence_dir)
        tool = ROOT / 'tools/agents/evidence.py'
        evidence_id = self.command(['python3', str(tool), 'record', str(run), '--repo', str(imported)])
        self.command(['python3', str(tool), 'verify', evidence_id])
        record = json.loads((evidence_dir / (evidence_id.removeprefix('sha256:') + '.json')).read_text())
        assert record['fields']['base_sha']['value'] == candidate['commit']
        (run / 'events.jsonl').write_text('tampered candidate proof\n')
        tamper = self.command(['python3', str(tool), 'verify', evidence_id], check=False)
        assert tamper.returncode != 0
        self.result['observations']['evidence'] = {'id': evidence_id, 'record': record,
                                                   'tamper_exit': tamper.returncode}
        # GC is dry by default and refuses dirty resources; safe merged tree can be reclaimed.
        wt = ROOT / 'tools/agents/wt.py'
        managed = json.loads(self.command(['python3', str(wt), 'new', '--repo', str(imported),
                                          '--agent', 'fixture', '--topic', 'gc']))
        worktree = Path(managed['path'])
        dry = self.command(['python3', str(wt), 'gc', '--repo', str(imported)])
        assert worktree.exists() and managed['branch'] in dry
        (worktree / 'dirty.txt').write_text('not collectible\n')
        dirty = self.command(['python3', str(wt), 'gc', '--apply', '--repo', str(imported)])
        assert worktree.exists() and managed['branch'] not in dirty
        (worktree / 'dirty.txt').unlink()
        safe = self.command(['python3', str(wt), 'gc', '--apply', '--repo', str(imported)])
        assert not worktree.exists() and managed['branch'] in safe
        self.result['observations']['integration'] = {'candidate': candidate, 'bundle_sha256': digest(bundle.read_bytes()),
            'landed_exact_commit': landed, 'actual_git_tree': self.git(imported, 'rev-parse', 'main^{tree}'),
            'gc_dry_run_preserved': True, 'gc_dirty_preserved': True, 'gc_safe_removed': True}
        # Existing Python tooling regression fixtures also operate solely on disposable Git repositories.
        for name in ('test_wt.py', 'test_land.py', 'test_evidence.py'):
            self.command(['python3', str(ROOT / 'tools/agents' / name)])
        self.release(repo)

    def git_baseline_slice(self):
        repo = self.root / 'git-baseline'; repo.mkdir()
        self.git(repo, 'init', '-q', '-b', 'main')
        (repo / 'artifact.txt').write_text('git-base\n')
        self.git(repo, 'add', 'artifact.txt'); self.git(repo, 'commit', '-qm', 'base')
        base = self.git(repo, 'rev-parse', 'HEAD')
        self.git(repo, 'checkout', '-qb', 'attempt')
        (repo / 'artifact.txt').write_text('git-checkpoint\n')
        before_status = self.git(repo, 'rev-parse', 'HEAD')
        status = self.git(repo, 'status', '--porcelain')
        assert status and self.git(repo, 'rev-parse', 'HEAD') == before_status
        self.git(repo, 'add', 'artifact.txt'); self.git(repo, 'commit', '-qm', 'checkpoint')
        commit = self.git(repo, 'rev-parse', 'HEAD'); tree = self.git(repo, 'rev-parse', 'HEAD^{tree}')
        self.git(repo, 'update-ref', 'refs/fixture/verified', commit)
        bundle = self.root / 'git-baseline.bundle'
        self.git(repo, 'bundle', 'create', str(bundle), 'refs/fixture/verified')
        bundle_digest = digest(bundle.read_bytes())
        (repo / 'artifact.txt').write_text('git-rewrite\n')
        self.git(repo, 'add', 'artifact.txt'); rewritten_tree = self.git(repo, 'write-tree')
        rewritten_commit = self.git(repo, 'commit-tree', rewritten_tree, '-p', base, '-m', 'replacement candidate')
        self.git(repo, 'update-ref', 'refs/heads/attempt', rewritten_commit, commit)
        assert self.git(repo, 'rev-parse', 'HEAD') != commit
        assert self.git(repo, 'show', commit + ':artifact.txt') == 'git-checkpoint'
        assert tree != rewritten_tree
        # Deliberate public repo/WC separation, not an invented Git crash proof.
        self.git(repo, 'reset', '--hard', commit)
        self.journal({'phase': 'prepared Git repository-only transition', 'base': base,
                      'old_commit': commit, 'target_commit': rewritten_commit, 'target_tree': rewritten_tree})
        self.git(repo, 'update-ref', 'refs/heads/attempt', rewritten_commit, commit)
        separated = {'repository_commit': self.git(repo, 'rev-parse', 'HEAD'),
                     'working_copy_content': (repo / 'artifact.txt').read_text(),
                     'status': self.git(repo, 'status', '--porcelain')}
        assert separated['repository_commit'] == rewritten_commit
        assert separated['working_copy_content'] == 'git-checkpoint\n'
        assert separated['status']
        self.git(repo, 'reset', '--hard', rewritten_commit)
        assert (repo / 'artifact.txt').read_text() == 'git-rewrite\n'
        assert digest(bundle.read_bytes()) == bundle_digest
        verifier = self.root / 'git-baseline-verifier'; verifier.mkdir()
        self.git(verifier, 'init', '-q')
        self.git(verifier, 'fetch', str(bundle), 'refs/fixture/verified')
        assert self.git(verifier, 'rev-parse', 'FETCH_HEAD') == commit
        assert self.git(verifier, 'rev-parse', 'FETCH_HEAD^{tree}') == tree
        assert self.git(verifier, 'show', 'FETCH_HEAD:artifact.txt') == 'git-checkpoint'
        self.result['observations']['git_baseline'] = {'base': base, 'checkpoint_commit': commit,
            'checkpoint_tree': tree, 'rewritten_commit': rewritten_commit, 'rewritten_tree': rewritten_tree,
            'bundle_sha256': bundle_digest, 'status_does_not_snapshot': True,
            'repository_vs_working_copy': separated, 'explicit_reconciliation_exact': True,
            'fault_scope': 'public update-ref/reset separation only; no Git crash injection claimed'}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--slice', choices=['identity', 'authority', 'crash', 'exclusions', 'integration', 'git_baseline'], required=True)
    parser.add_argument('--unsafe-control', action='store_true', help='Behavioral falsification of unsafe assumptions')
    args = parser.parse_args()
    evaluation = Evaluation(args.binary, args.unsafe_control)
    code = 0
    try:
        getattr(evaluation, args.slice + '_slice')()
        evaluation.result['outcome'] = 'passed'
    except Exception as error:
        evaluation.result.update(outcome='failed', failure=str(error), failure_type=type(error).__name__)
        code = 1
    finally:
        evaluation.result['durable_preexecution_journal'] = (evaluation.root / 'operations.jsonl').read_text()
        evaluation.tmp.cleanup()
        evaluation.result.update(slice=args.slice, fixture_removed=not evaluation.root.exists())
        Path(args.output).write_text(json.dumps(evaluation.result, indent=2) + '\n')
    print(json.dumps({k: v for k, v in evaluation.result.items() if k not in ('events', 'durable_preexecution_journal')}, indent=2))
    return code


if __name__ == '__main__':
    raise SystemExit(main())

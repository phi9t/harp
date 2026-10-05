#!/usr/bin/env python3
"""Behavior checks use disposable tmux servers and a viewer without model calls."""
import importlib.util
import json
import os
from pathlib import Path
import shlex
import socket
import signal
import sys
import subprocess
import tempfile
import threading
import time
import unittest
import uuid

TOOL = Path(__file__).with_name('mac_tmux.py')
THREAD1 = '01a10a0d-f18a-7641-9ca5-1cb49cb81603'
THREAD2 = '01a0f51d-8b62-7930-862f-c4b44d7d0bb7'


class MaintenanceTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(TOOL.is_file(), 'maintainer tool must exist')
        spec = importlib.util.spec_from_file_location('mac_tmux', TOOL)
        self.tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.tool)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.tmux_argv = ['tmux', '-L', 'mac-test-' + uuid.uuid4().hex]
        self.addCleanup(lambda: subprocess.run(self.tmux_argv + ['kill-server'], capture_output=True))
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.bind(str(self.root / 'daemon.sock'))
        self.addCleanup(self.sock.close)
        self.viewer = self.root / 'fake viewer.py'
        self.log = self.root / 'argv.jsonl'
        self.viewer.write_text('#!/usr/bin/env python3\nimport json,sys,time\n'
                               + f'with open({str(self.log)!r},"a") as f: f.write(json.dumps(sys.argv[1:])+"\\n")\n'
                               + 'time.sleep(3600)\n')
        self.viewer.chmod(0o700)
        self.current = self.root / 'CURRENT.json'
        self.current.write_text(json.dumps({'thread': THREAD1, 'worktree': str(self.root), 'writer_state': 'active'}))
        self.config = {'version': 1, 'session': 'operator', 'owner': 'test-owner',
                       'state_dir': str(self.root / 'state'), 'current_path': str(self.current),
                       'luna_path': str(self.root / 'missing-luna'), 'codex_executable': str(self.viewer),
                       'endpoint': 'unix://' + str(self.root / 'daemon.sock'),
                       'coordinator': {'role': 'coordinator', 'thread': THREAD2, 'cwd': str(self.root)},
                       'extra_bindings': [], 'refresh_seconds': 3,
                       'dashboard_argv': [str(self.viewer), 'dashboard', 'literal $(touch HACKED); hi']}
        self.commands = []

    def tmux(self, *argv):
        return subprocess.run(self.tmux_argv + list(argv), check=True, capture_output=True, text=True).stdout.strip()

    def reconcile(self):
        def run(argv):
            self.commands.append(argv)
            return subprocess.run(argv, capture_output=True, text=True)
        return self.tool.reconcile(self.config, tmux_argv=self.tmux_argv, runner=run)

    def views(self):
        return self.tmux('list-windows', '-t', '=operator', '-F', '#{window_id}').splitlines()

    def wait_log(self, count):
        for _ in range(100):
            if self.log.exists():
                rows = [json.loads(line) for line in self.log.read_text().splitlines()]
                if len(rows) >= count:
                    return rows
            time.sleep(.01)
        self.fail('fake viewer did not record launch')

    def test_repeat_and_replacement_keep_clients_and_exact_resume_boundary(self):
        first = self.reconcile()
        self.assertEqual(first['health'], 'ok')
        self.assertEqual(len(first['bindings']), 2)
        ids = self.views()
        chosen = first['bindings'][0]['window']
        self.tmux('select-window', '-t', chosen)
        self.reconcile()
        self.assertEqual(self.views(), ids)
        self.assertEqual(self.tmux('display-message', '-p', '-t', '=operator:', '#{window_id}'), chosen)
        self.current.write_text(json.dumps({'thread': str(uuid.UUID(int=3)), 'worktree': str(self.root), 'writer_state': 'replacing'}))
        replaced = self.reconcile()
        self.assertTrue(set(ids).issubset(self.views()))
        old = next(b for b in replaced['bindings'] if b['thread'] == THREAD1)
        self.assertIs(old['current'], False)
        self.assertIn('historical', self.tmux('display-message', '-p', '-t', old['window'], '#{window_name}'))
        rows = self.wait_log(4)
        self.assertIn(['resume', '--remote', self.config['endpoint'], '--include-non-interactive', THREAD1], rows)
        self.assertIn(['dashboard', 'literal $(touch HACKED); hi'], rows)
        self.assertFalse((self.root / 'HACKED').exists())
        allowed = {'has-session', 'new-session', 'show-options', 'list-windows', 'new-window', 'set-option', 'rename-window', 'list-panes'}
        for command in self.commands:
            self.assertIn(command[len(self.tmux_argv)], allowed, command)
        manifest = Path(self.config['state_dir']) / 'manifest.json'
        self.assertEqual(json.loads(manifest.read_text())['bindings'], replaced['bindings'])
        self.assertEqual(manifest.stat().st_mode & 0o777, 0o600)

    def test_missing_and_malformed_current_preserve_views_and_report_unknown(self):
        self.reconcile()
        ids = self.views()
        for content in (None, '{', '{}', json.dumps({'thread': THREAD1, 'worktree': 42})):
            if content is None:
                self.current.unlink()
            else:
                self.current.write_text(content)
            report = self.reconcile()
            self.assertEqual(self.views(), ids)
            self.assertEqual(report['health'], 'degraded')
            self.assertTrue(any('CURRENT' in error for error in report['errors']))
            worker = next(b for b in report['bindings'] if b['thread'] == THREAD1)
            self.assertIsNone(worker['current'])

    def test_same_thread_different_endpoint_creates_separate_views(self):
        self.reconcile()
        other = socket.socket(socket.AF_UNIX)
        other.bind(str(self.root / 'other.sock'))
        self.addCleanup(other.close)
        self.config['extra_bindings'] = [{'role': 'operator-extra', 'thread': THREAD1,
            'cwd': str(self.root), 'endpoint': 'unix://' + str(self.root / 'other.sock')}]
        report = self.reconcile()
        matches = [b for b in report['bindings'] if b['thread'] == THREAD1]
        self.assertEqual(len(matches), 2)
        self.assertNotEqual(matches[0]['window'], matches[1]['window'])
        self.assertEqual(len(self.reconcile()['bindings']), 3)

    def test_unrelated_session_ownership_is_refused_without_changes(self):
        self.tmux('new-session', '-d', '-s', 'operator', 'sleep 3600')
        self.tmux('set-option', '-t', '=operator:', '@corenius_operator_owner', 'someone-else')
        ids = self.views()
        with self.assertRaisesRegex(ValueError, 'owner'):
            self.reconcile()
        self.assertEqual(self.views(), ids)
        self.assertTrue(all(c[len(self.tmux_argv)] in {'has-session', 'show-options'} for c in self.commands))

    def test_legacy_adoption_requires_live_endpoint_launch_evidence(self):
        self.tmux('new-session', '-d', '-s', 'operator', '-n', 'overview', 'sleep 3600')
        self.tmux('set-option', '-t', '=operator:', '@corenius_operator_owner', self.config['owner'])
        argv = [str(self.viewer), 'resume', '--remote', self.config['endpoint'], '--include-non-interactive', THREAD1]
        window = self.tmux('new-window', '-d', '-P', '-F', '#{window_id}', '-t', '=operator:',
                           '-n', 'worker-1', '-c', str(self.root), shlex.join(argv))
        for key, value in (('@corenius_thread', THREAD1), ('@corenius_role', 'worker-1')):
            self.tmux('set-option', '-w', '-t', window, key, value)
        legacy = self.root / 'legacy.json'
        legacy.write_text(json.dumps({'session': 'operator', 'daemon_endpoint': self.config['endpoint'],
            'bindings': [{'role': 'worker-1', 'thread': THREAD1, 'workspace': str(self.root), 'window': window, 'argv': argv}]}))
        self.config['adopt_manifest_path'] = str(legacy)
        report = self.reconcile()
        self.assertEqual(report['health'], 'ok')
        self.assertEqual(next(b['window'] for b in report['bindings'] if b['thread'] == THREAD1), window)
        self.assertEqual(self.tmux('show-options', '-w', '-v', '-t', window, '@corenius_endpoint'), self.config['endpoint'])
        self.assertEqual(len(self.views()), 3)

    def test_bare_legacy_thread_is_not_adopted_or_duplicated(self):
        self.tmux('new-session', '-d', '-s', 'operator', '-n', 'overview', 'sleep 3600')
        self.tmux('set-option', '-t', '=operator:', '@corenius_operator_owner', self.config['owner'])
        window = self.tmux('new-window', '-d', '-P', '-F', '#{window_id}', '-t', '=operator:', 'sleep 3600')
        for key, value in (('@corenius_thread', THREAD1), ('@corenius_role', 'worker-1')):
            self.tmux('set-option', '-w', '-t', window, key, value)
        argv = [str(self.viewer), 'resume', '--remote', self.config['endpoint'], '--include-non-interactive', THREAD1]
        legacy = self.root / 'legacy.json'
        legacy.write_text(json.dumps({'session': 'operator', 'daemon_endpoint': self.config['endpoint'],
            'bindings': [{'role': 'worker-1', 'thread': THREAD1, 'workspace': str(self.root), 'window': window, 'argv': argv}]}))
        self.config['adopt_manifest_path'] = str(legacy)
        report = self.reconcile()
        self.assertEqual(report['health'], 'degraded')
        self.assertFalse(any(b['thread'] == THREAD1 for b in report['bindings']))
        self.assertEqual(len(self.views()), 3)
        self.assertTrue(any('adoption' in e for e in report['errors']))

    def test_unowned_exact_endpoint_thread_is_pending_without_duplicate_or_mutation(self):
        self.tmux('new-session', '-d', '-s', 'operator', '-n', 'overview', 'sleep 3600')
        self.tmux('set-option', '-t', '=operator:', '@corenius_operator_owner', self.config['owner'])
        window = self.tmux('new-window', '-d', '-P', '-F', '#{window_id}', '-t', '=operator:', '-n', 'unowned', 'sleep 3600')
        for key, value in (('@corenius_thread', THREAD1), ('@corenius_role', 'worker-1'),
                           ('@corenius_endpoint', self.config['endpoint']), ('@corenius_cwd', str(self.root))):
            self.tmux('set-option', '-w', '-t', window, key, value)
        report = self.reconcile()
        self.assertFalse(any(b['thread'] == THREAD1 for b in report['bindings']))
        self.assertEqual(len(self.views()), 3)
        self.assertEqual(report['health'], 'degraded')
        self.assertTrue(any('pending validated adoption' in e for e in report['errors']))
        self.assertEqual(self.tmux('display-message', '-p', '-t', window, '#{window_name}'), 'unowned')
        mutations = {'set-option', 'rename-window'}
        self.assertFalse(any(c[len(self.tmux_argv)] in mutations and window in c for c in self.commands))

    def test_malformed_adoption_binding_reports_degraded_and_retains_existing_views(self):
        self.reconcile()
        ids = self.views()
        legacy = self.root / 'bad-legacy.json'
        self.config['adopt_manifest_path'] = str(legacy)
        for bindings in ([42], None, {'window': '@0'}):
            legacy.write_text(json.dumps({'session': 'operator', 'bindings': bindings}))
            report = self.reconcile()
            self.assertEqual(report['health'], 'degraded')
            self.assertTrue(any('adoption pending' in e for e in report['errors']))
            self.assertEqual(self.views(), ids)
            self.assertEqual(len(report['bindings']), 2)

    def test_completed_current_is_retained_without_launching_a_new_client(self):
        self.reconcile()
        old_ids = self.views()
        self.current.write_text(json.dumps({'thread': THREAD1, 'worktree': str(self.root), 'writer_state': 'completed'}))
        report = self.reconcile()
        self.assertIs(next(b['current'] for b in report['bindings'] if b['thread'] == THREAD1), False)
        self.current.write_text(json.dumps({'thread': str(uuid.UUID(int=4)), 'worktree': str(self.root), 'writer_state': 'quiescent'}))
        self.reconcile()
        self.assertEqual(self.views(), old_ids)

    def test_unavailable_daemon_and_exited_views_are_reported_without_replacement(self):
        report = self.reconcile()
        old_ids = self.views()
        window = next(b['window'] for b in report['bindings'] if b['thread'] == THREAD1)
        # Disposable test cleanup only: the maintainer is not allowed to do this.
        pid = int(self.tmux('display-message', '-p', '-t', window, '#{pane_pid}'))
        os.kill(pid, 15)
        for _ in range(100):
            panes = self.tmux('list-panes', '-s', '-t', '=operator', '-F', '#{window_id} #{pane_dead}').splitlines()
            if window + ' 1' in panes or window not in self.views():
                break
            time.sleep(.01)
        self.assertIn(window, self.views(), 'exited native view must remain without fixture options')
        report = self.reconcile()
        self.assertEqual(report['health'], 'degraded')
        self.assertTrue(any('exited' in e for e in report['errors']))
        self.assertEqual(self.views(), old_ids)
        self.current.write_text(json.dumps({'thread': str(uuid.UUID(int=5)), 'worktree': str(self.root), 'writer_state': 'active'}))
        (self.root / 'daemon.sock').unlink()
        report = self.reconcile()
        self.assertTrue(any('socket unavailable' in e for e in report['errors']))
        self.assertEqual(self.views(), old_ids)

    def test_immediately_exiting_view_is_retained_without_relaunch(self):
        self.viewer.write_text('#!/usr/bin/env python3\nimport sys\nsys.exit(1)\n')
        self.config['dashboard_argv'] = ['/bin/sleep', '3600']
        first = self.reconcile()
        ids = self.views()
        self.assertEqual(len(ids), 3)
        time.sleep(.05)
        second = self.reconcile()
        self.assertEqual(self.views(), ids)
        self.assertEqual({b['window'] for b in first['bindings']}, {b['window'] for b in second['bindings']})
        self.assertEqual(second['health'], 'degraded')
        self.assertEqual(len([e for e in second['errors'] if 'exited' in e]), 2)

    def test_explicit_no_active_author_marks_prior_view_historical_without_launch(self):
        self.reconcile()
        ids = self.views()
        self.current.write_text(json.dumps({'writer': None, 'thread': None,
            'writer_state': 'no active implementation writer; scoped independent reviews running',
            'last_writer': {'thread': THREAD1, 'worktree': str(self.root), 'state': 'completed/quiescent'}}))
        report = self.reconcile()
        self.assertEqual(report['health'], 'ok')
        self.assertEqual(self.views(), ids)
        self.assertIs(next(b['current'] for b in report['bindings'] if b['thread'] == THREAD1), False)
        self.current.write_text(json.dumps({'thread': None, 'writer_state': 'no active implementation writer'}))
        report = self.reconcile()
        self.assertEqual(report['health'], 'degraded')
        self.assertIsNone(next(b['current'] for b in report['bindings'] if b['thread'] == THREAD1))

    def test_cli_watch_holds_instance_lock_and_refreshes_without_reselection(self):
        config_path = self.root / 'config.json'
        self.config['refresh_seconds'] = 1
        config_path.write_text(json.dumps(self.config))
        wrapper_dir = self.root / 'bin'
        wrapper_dir.mkdir()
        wrapper = wrapper_dir / 'tmux'
        wrapper.write_text('#!/bin/sh\nexec ' + shlex.join(['/usr/bin/tmux'] + self.tmux_argv[1:]) + ' "$@"\n')
        wrapper.chmod(0o700)
        env = dict(os.environ, PATH=str(wrapper_dir) + os.pathsep + os.environ['PATH'])
        process = subprocess.Popen([sys.executable, str(TOOL), '--config', str(config_path), '--watch'],
                                   stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, env=env)
        self.addCleanup(lambda: process.poll() is None and process.kill())
        manifest = Path(self.config['state_dir']) / 'manifest.json'
        for _ in range(300):
            if manifest.exists():
                break
            time.sleep(.01)
        self.assertTrue(manifest.exists())
        first = json.loads(manifest.read_text())
        chosen = first['bindings'][0]['window']
        self.tmux('select-window', '-t', chosen)
        duplicate = subprocess.run([sys.executable, str(TOOL), '--config', str(config_path), '--once'],
                                   capture_output=True, text=True, env=env, timeout=3)
        self.assertEqual(duplicate.returncode, 1)
        self.assertIn('already running', duplicate.stderr)
        replacement = str(uuid.UUID(int=6))
        self.current.write_text(json.dumps({'thread': replacement, 'worktree': str(self.root), 'writer_state': 'active'}))
        for _ in range(300):
            if any(b['thread'] == replacement for b in json.loads(manifest.read_text())['bindings']):
                break
            time.sleep(.01)
        self.assertTrue(any(b['thread'] == replacement for b in json.loads(manifest.read_text())['bindings']))
        self.assertEqual(self.tmux('display-message', '-p', '-t', '=operator:', '#{window_id}'), chosen)
        process.send_signal(signal.SIGINT)
        _, error = process.communicate(timeout=3)
        self.assertEqual(process.returncode, 0, error)
        once = subprocess.run([sys.executable, str(TOOL), '--config', str(config_path), '--once'],
                              capture_output=True, text=True, env=env, timeout=3)
        self.assertEqual(once.returncode, 0, once.stderr)
        self.assertEqual(len(json.loads(once.stdout)['bindings']), 3)

    def test_concurrent_reconciliation_does_not_duplicate(self):
        reports, errors = [], []
        def once():
            try:
                reports.append(self.reconcile())
            except self.tool.AlreadyRunning:
                pass
            except Exception as exc:
                errors.append(exc)
        threads = [threading.Thread(target=once) for _ in range(4)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()
        self.assertEqual(errors, [])
        self.assertTrue(reports)
        self.assertEqual(len(self.reconcile()['bindings']), 2)
        self.assertEqual(len(self.views()), 3)


if __name__ == '__main__':
    unittest.main()

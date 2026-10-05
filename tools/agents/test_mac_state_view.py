#!/usr/bin/env python3
"""Behavior checks for read-only public state panes; no provider calls."""
import datetime as dt
import fcntl
import importlib.util
import json
import os
import pathlib
import pty
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
import unittest

SCRIPT = pathlib.Path(__file__).with_name('mac_state_view.py')


class StateViewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.now = dt.datetime(2026, 10, 5, 4, tzinfo=dt.timezone.utc)
        self.stamp = self.now.isoformat()
        self.config = {'version': 1, 'session': 'test-team', 'owner': 'operator',
                       'state_dir': str(self.root), 'current_path': str(self.root / 'CURRENT.json'),
                       'luna_path': '/never-read', 'codex_executable': '/never-run',
                       'endpoint': 'unix:///test', 'coordinator': {}, 'extra_bindings': [],
                       'dashboard_argv': [], 'refresh_seconds': 0.25,
                       'controller_snapshot_path': str(self.root / 'controller.json'),
                       'kata_snapshot_path': str(self.root / 'kata.json')}
        self.current = {'observed_at': self.stamp, 'writer': 'implementation-author',
                        'writer_state': 'review pending', 'thread': 'author-thread',
                        'worktree': '/work/author', 'base': 'base-sha', 'source': 'source-sha',
                        'tickets': {'03': 'state panes'}, 'focused_results': {'view': 'GREEN'}}
        self.manifest = {'session': 'test-team', 'observed_at': self.stamp,
                         'bindings': [{'role': 'product-lead', 'thread': 'lead-thread',
                                       'endpoint': 'unix:///product', 'cwd': '/work/lead',
                                       'window': '@4', 'current': True}]}
        self.controller = {'observed_at': self.stamp, 'revision': 7, 'event_cursor': 19,
                           'occupied_slots': 2,
                           'mission': {'project': 'trial-project', 'status': 'running',
                                       'lead_runtime': 'runtime-lead', 'role_epoch': 3},
                           'workers': [{'slot': 1, 'runtime': 'runtime-one', 'task': 'K-1',
                                        'attempt': 'A-1', 'status': 'working', 'epoch': 2},
                                       {'slot': 2, 'runtime': 'runtime-two', 'task': 'K-2',
                                        'attempt': 'A-2', 'status': 'waiting', 'epoch': 4}],
                           'tasks': [{'id': 'K-1', 'state': 'running', 'attempt': 'A-1'}],
                           'questions': [{'id': 'Q-1', 'status': 'open', 'question': 'Choose gate'}],
                           'unknown_effects': [{'id': 'op-1', 'state': 'unknown'}],
                           'filesystem': [{'workspace': '/work/attempt', 'base': 'attempt-base',
                                           'source': 'attempt-source'}],
                           'landing_queue': [{'candidate': 'candidate-1', 'status': 'queued'}]}
        self.kata = {'observed_at': self.stamp, 'deployment': 'kata-trial', 'project': 'trial-project',
                     'issues': [{'id': 'K-1', 'uid': 'uid-1', 'title': 'First task', 'state': 'open',
                                 'owner': 'worker-one', 'revision': 4, 'dependencies': ['K-0'],
                                 'blocked_by': ['K-0']}],
                     'synchronization': {'status': 'behind', 'revision': 3}}
        self.write_inputs()

    def write_inputs(self):
        for name, obj in [('config.json', self.config), ('CURRENT.json', self.current),
                          ('manifest.json', self.manifest), ('controller.json', self.controller),
                          ('kata.json', self.kata)]:
            (self.root / name).write_text(json.dumps(obj))

    def module(self):
        self.assertTrue(SCRIPT.exists(), 'state view implementation is missing')
        spec = importlib.util.spec_from_file_location('mac_state_view', SCRIPT)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def rendered(self, view):
        module = self.module()
        config = module.load_config(self.root / 'config.json')
        return '\n'.join(module.render(module.load_snapshot(config, now=self.now), view))

    def test_controller_exposes_both_slots_with_product_and_implementation_labels(self):
        text = self.rendered('controller')
        for expected in ['trial-project', 'revision: 7', 'event_cursor: 19', 'runtime-lead',
                         'Worker slot 1', 'runtime-one', 'Worker slot 2', 'runtime-two',
                         'A-1', 'A-2', 'op-1', 'Choose gate', 'candidate-1',
                         'Implementation writer (CURRENT)', 'implementation-author',
                         'Tasks (CURRENT claims)', 'state panes', 'Native bindings (manager)',
                         'lead-thread', 'unix:///product', 'sha256:', self.stamp, 'fresh']:
            self.assertIn(expected, text)

    def test_nested_physical_slots_one_and_two_and_unmapped_workers_remain_visible(self):
        self.controller['workers'] = [
            {'attempt': {'id': 'attempt-one', 'slot': 1, 'task': 'task-one'},
             'runtime': {'id': 'runtime-physical-one', 'state': 'bound'}},
            {'attempt': {'id': 'attempt-two', 'slot': 2, 'task': 'task-two'},
             'runtime': {'id': 'runtime-physical-two', 'state': 'bound'}},
            {'attempt': {'id': 'attempt-unmapped', 'slot': 7},
             'runtime': {'id': 'runtime-unmapped', 'state': 'bound'}}]
        self.write_inputs()
        text = self.rendered('controller')
        self.assertIn('Worker slot 2:', text)
        slot_one = text.split('Worker slot 1:', 1)[1].split('Worker slot 2:', 1)[0]
        self.assertIn('attempt-one', slot_one)
        self.assertIn('runtime-physical-one', slot_one)
        slot_two = text.split('Worker slot 2:', 1)[1].split('Unmapped workers', 1)[0]
        self.assertIn('attempt-two', slot_two)
        self.assertIn('runtime-physical-two', slot_two)
        self.assertIn('Unmapped workers', text)
        self.assertIn('attempt-unmapped', text)
        self.assertIn('runtime-unmapped', text)
        self.assertNotIn('Worker slot 0', text)

    def test_kata_has_exact_identity_dependencies_and_sync_without_daemon_guess(self):
        text = self.rendered('kata')
        for expected in ['kata-trial', 'trial-project', 'uid-1', 'First task',
                         'dependencies', 'K-0', 'blocked_by', 'behind', 'revision: 3']:
            self.assertIn(expected, text)
        del self.config['kata_snapshot_path']
        self.write_inputs()
        text = self.rendered('kata')
        self.assertIn('Kata: unavailable', text)
        self.assertIn('not configured', text)
        self.assertNotIn('kata-trial', text)

    def test_workspace_claims_and_observed_inventory_are_separate(self):
        text = self.rendered('workspaces')
        for expected in ['CURRENT claims', '/work/author', 'base-sha', 'source-sha', 'GREEN',
                         'Controller filesystem (published inventory)', '/work/attempt',
                         'attempt-base', 'candidate-1', 'Native view cwd (manager)', '/work/lead']:
            self.assertIn(expected, text)

    def test_official_nested_projection_keeps_runtime_and_filesystem_metadata_only(self):
        self.controller['project'] = {'id': 'official-project', 'base': 'official-base',
                                      'revision': 11, 'lead_epoch': 8, 'paused': False}
        self.controller['mission']['constraints'] = 'Owner admits landing'
        self.controller['workers'] = [{
            'attempt': {'id': 'nested-attempt', 'task': 'nested-task', 'base': 'nested-base',
                        'state': 'running', 'scope_epoch': 9},
            'runtime': {'id': 'native-runtime', 'role': 'attempt', 'authority_epoch': 9,
                        'state': 'bound', 'document': {'auth': 'HIDDEN-RUNTIME'}},
            'last_activity': {'visible_stream': {'document': 'HIDDEN-TRANSCRIPT'}},
            'reported_progress': {'body': 'HIDDEN-REPORT'},
            'observed': {'state': 'running', 'stop_observed': False},
            'cancellation': {'requested': False, 'observed_stopped': False}}]
        self.controller['filesystem'] = {
            'configured': True, 'backend': 'git', 'actual_trunk': 'trunk-sha',
            'snapshot_revision': 11, 'current_revision': 12, 'state_drift': True,
            'workspaces': [{'path': '/work/official', 'attempt': 'nested-attempt',
                            'base': 'nested-base', 'commit': 'candidate-sha',
                            'observed_attempt_state': 'running', 'ownership_drift': False,
                            'refs': {'secret': 'HIDDEN-REFS'}}]}
        self.write_inputs()
        text = self.rendered('controller')
        for expected in ['official-project', 'Owner admits landing', 'nested-attempt',
                         'native-runtime', 'authority_epoch: 9', 'physical slot IDs unavailable',
                         'Worker slot 1: unknown', 'Worker slot 2: unknown']:
            self.assertIn(expected, text)
        workspaces = self.rendered('workspaces')
        for expected in ['trunk-sha', '/work/official', 'candidate-sha', 'state_drift: True']:
            self.assertIn(expected, workspaces)
        self.assertNotIn('HIDDEN-', text + workspaces)

    def test_missing_and_malformed_sources_preserve_two_unknown_slots(self):
        del self.config['controller_snapshot_path']
        self.write_inputs()
        text = self.rendered('controller')
        for expected in ['Product controller: unavailable', 'Worker slot 1: unknown',
                         'Worker slot 2: unknown', 'implementation-author']:
            self.assertIn(expected, text)
        self.config['controller_snapshot_path'] = str(self.root / 'controller.json')
        self.write_inputs()
        for raw in ['{bad', '[]', '{"workers":"not-a-list"}',
                    '{"observed_at":"bad","workers":[]}', '{"workers":[false]}']:
            (self.root / 'controller.json').write_text(raw)
            text = self.rendered('controller')
            self.assertIn('Product controller: unavailable', text)
            self.assertIn('malformed', text)
            self.assertIn('Worker slot 2: unknown', text)
        (self.root / 'kata.json').unlink()
        self.assertIn('missing', self.rendered('kata'))

    def test_stale_future_and_untimestamped_sources_are_truthful(self):
        for stamp, expected in [('2026-10-05T03:00:00Z', 'STALE'),
                                ('2026-10-05T05:00:00Z', 'future timestamp')]:
            self.controller['observed_at'] = stamp
            self.write_inputs()
            self.assertIn(expected, self.rendered('controller'))
        del self.controller['observed_at']
        self.write_inputs()
        text = self.rendered('controller')
        self.assertIn('freshness unavailable', text)
        self.assertIn('runtime-one', text)

    def test_nested_secrets_controls_and_large_values_never_reach_output(self):
        sentinel = 'DO-NOT-DISPLAY-SECRET'
        for obj in [self.controller, self.controller['mission'], self.controller['workers'][0],
                    self.controller['questions'][0], self.controller['filesystem'][0],
                    self.current, self.kata, self.kata['issues'][0], self.manifest['bindings'][0]]:
            obj.update({'credentials': sentinel, 'private_reasoning': sentinel, 'api_key': sentinel})
        self.controller['mission']['status'] = {'token': sentinel}
        self.current['tickets']['secret_key'] = sentinel
        self.current['focused_results']['secret_key'] = sentinel
        self.kata['issues'][0]['title'] = 'Visible\x1b[31m\x00\n\u202e' + 'x' * 10000
        self.kata['issues'] *= 50
        self.write_inputs()
        for view in ['controller', 'kata', 'workspaces']:
            text = self.rendered(view)
            self.assertNotIn(sentinel, text)
            for control in ['\x1b', '\x00', '\u202e']:
                self.assertNotIn(control, text)
            self.assertLess(len(text), 120000)
        self.assertIn('display limited', self.rendered('kata'))

    def test_render_is_read_only_and_config_paths_are_validated(self):
        before = {p.name: p.read_bytes() for p in self.root.iterdir()}
        for view in ['controller', 'kata', 'workspaces']:
            result = subprocess.run([sys.executable, str(SCRIPT), '--config', str(self.root / 'config.json'),
                                     '--view', view, '--render'], capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('sha256:', result.stdout)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.root.iterdir()})
        self.config['kata_snapshot_path'] = ['wrong']
        self.write_inputs()
        result = subprocess.run([sys.executable, str(SCRIPT), '--config', str(self.root / 'config.json'),
                                 '--view', 'kata', '--render'], capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 2)

    def test_narrow_real_pty_refresh_navigation_and_exit(self):
        master, slave = pty.openpty()
        self.addCleanup(os.close, master)
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 8, 24, 0, 0))
        process = subprocess.Popen([sys.executable, str(SCRIPT), '--config', str(self.root / 'config.json'),
                                    '--view', 'controller'], stdin=slave, stdout=slave, stderr=slave,
                                   env=dict(os.environ, TERM='xterm'))
        os.close(slave)
        self.addCleanup(lambda: process.kill() if process.poll() is None else None)
        output = bytearray()

        def drain(duration):
            until = time.monotonic() + duration
            while time.monotonic() < until:
                if select.select([master], [], [], 0.03)[0]:
                    try:
                        output.extend(os.read(master, 65536))
                    except OSError:
                        break

        drain(0.3)
        for key in [b'2', b'3', b'1', b'j', b'k', b'r', b'?', b'?']:
            os.write(master, key)
            drain(0.08)
        os.write(master, b'q')
        drain(0.1)
        self.assertEqual(process.wait(timeout=3), 0, output.decode(errors='replace'))
        for label in [b'Controller', b'Kata', b'Workspaces']:
            self.assertIn(label, output)


if __name__ == '__main__':
    unittest.main()

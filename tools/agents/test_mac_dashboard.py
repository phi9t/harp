#!/usr/bin/env python3
"""Focused operator dashboard behavior checks; no product or provider calls."""
import datetime as dt
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
import fcntl

SCRIPT = pathlib.Path(__file__).with_name('mac_dashboard.py')


class DashboardTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.now = dt.datetime(2026, 10, 5, 4, tzinfo=dt.timezone.utc)
        self.stamp = self.now.isoformat()
        self.current = {
            'observed_at': self.stamp, 'goal': 'Repair then trial', 'next': 'Review candidate',
            'version': 'happy-path-1', 'source': 'abc123', 'base': 'def456',
            'thread': 'author-thread', 'writer': '/root/author', 'worktree': '/workspace/author',
            'writer_state': 'explicitly acknowledged', 'tickets': {'05': 'review pending'},
            'unknowns': ['Provider entitlement'], 'focused_results': {'C1': 'RED then GREEN'},
            'pending_gates': ['Independent review'],
            'context': {'author_latest_observed_input': 111203, 'cumulative_tokens': 900000,
                        'compaction_control': 'not exposed'},
        }
        self.luna = {'checked_at': self.stamp, 'phase': 'Focused review',
                     'progress': {'projection_says': 'C1 complete'},
                     'observed_services': {'controller_or_demo': 'not declared'},
                     'actionable_findings': [], 'source_identities': {'CURRENT.json': 'sha256:abc'}}
        self.manifest = {'session': 'test-team', 'observed_at': self.stamp,
                         'health': 'ok', 'errors': [], 'bindings': [
            {'role': 'implementation-author', 'thread': 'author-thread', 'endpoint': 'unix:///test',
             'cwd': '/workspace/author', 'window': '@4', 'current': True},
            {'role': 'old-author', 'thread': 'older-thread', 'endpoint': 'unix:///test',
             'cwd': '/workspace/old', 'window': '@2', 'current': False}]}
        self.config = {'version': 1, 'session': 'test-team', 'owner': 'operator',
                       'state_dir': str(self.root), 'current_path': str(self.root / 'CURRENT.json'),
                       'luna_path': str(self.root / 'luna.json'), 'codex_executable': '/unused/codex',
                       'endpoint': 'unix:///test', 'coordinator': {'role': 'coordinator',
                       'thread': 'coordinator-thread', 'cwd': '/workspace', 'endpoint': 'unix:///test'},
                       'extra_bindings': [], 'refresh_seconds': 3, 'dashboard_argv': []}
        self.write_inputs()

    def write_inputs(self):
        for name, obj in [('config.json', self.config), ('CURRENT.json', self.current),
                          ('luna.json', self.luna), ('manifest.json', self.manifest)]:
            (self.root / name).write_text(json.dumps(obj))

    def module(self):
        self.assertTrue(SCRIPT.exists(), 'dashboard implementation is missing')
        spec = importlib.util.spec_from_file_location('mac_dashboard', SCRIPT)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def snapshot(self):
        module = self.module()
        return module, module.load_snapshot(module.load_config(self.root / 'config.json'), now=self.now)

    def fake_tmux(self):
        fake = self.root / 'tmux'
        log = self.root / 'tmux-call.jsonl'
        fake.write_text('#!' + sys.executable + '\nimport json,pathlib,sys\n'
                        + 'with pathlib.Path(' + repr(str(log)) + ').open("a") as handle:\n'
                        + '    handle.write(json.dumps(sys.argv[1:])+"\\n")\n')
        fake.chmod(0o700)
        return log, dict(os.environ, TERM='xterm', PATH=str(self.root) + os.pathsep + os.defpath)

    def test_populated_snapshot_has_reachable_sections_and_truthful_provenance(self):
        module, snapshot = self.snapshot()
        sections = module.sections(snapshot)
        self.assertEqual(list(sections), ['Overview', 'Tasks', 'Agents', 'Work', 'Evidence', 'VCS', 'Infra'])
        rendered = '\n'.join(module.render(snapshot))
        for text in ['Repair then trial', '05: review pending', 'source: abc123',
                     'RED then GREEN', 'Independent review', 'window @4',
                     'historical', 'implementation author', 'Product controller worker slots: unavailable',
                     'Questions (CURRENT): unavailable', 'Findings (Luna): none',
                     'latest observed input', 'cumulative_tokens', '900000', '111203',
                     'America/Los_Angeles', '2026-10-05T04:00:00+00:00', 'sha256:abc']:
            self.assertIn(text, rendered)
        historical = next(line for line in sections['Agents'] if 'old-author' in line)
        self.assertIn('historical', historical)
        self.assertNotIn('current |', historical)

    def test_bad_sources_and_stale_data_never_look_healthy(self):
        module = self.module()
        (self.root / 'CURRENT.json').write_text('[]')
        (self.root / 'luna.json').write_text('{broken')
        (self.root / 'manifest.json').unlink()
        _, snapshot = self.snapshot()
        rendered = '\n'.join(module.render(snapshot))
        for name in ['CURRENT', 'Luna', 'Manifest']:
            self.assertIn(name + ': unavailable', rendered)
        self.assertIn('Questions (CURRENT): unavailable', rendered)
        self.current['observed_at'] = '2026-10-05T03:00:00Z'
        self.write_inputs()
        _, snapshot = self.snapshot()
        self.assertIn('STALE', '\n'.join(module.render(snapshot)))
        self.current['tickets'] = ['not-a-map']
        self.luna['actionable_findings'] = 'not-a-list'
        self.manifest['bindings'][0]['current'] = 'true'
        self.write_inputs()
        _, snapshot = self.snapshot()
        rendered = '\n'.join(module.render(snapshot))
        for name in ['CURRENT', 'Luna', 'Manifest']:
            self.assertIn(name + ': unavailable', rendered)

    def test_operator_roster_is_separate_and_does_not_invent_native_links(self):
        self.config['agent_roster_path'] = str(self.root / 'roster.json')
        (self.root / 'roster.json').write_text(json.dumps({'observed_at': self.stamp, 'agents': [
            {'name': 'Luna', 'scope': 'operator observation', 'status': 'watching', 'model': 'Luna'},
            {'name': 'dashboard author', 'scope': 'operator tooling', 'status': 'implementing',
             'model': 'lead-declared', 'ticket': '01-dashboard'}]}))
        self.write_inputs()
        module, snapshot = self.snapshot()
        agents = '\n'.join(module.sections(snapshot)['Agents'])
        self.assertIn('Operator roster (lead-owned)', agents)
        self.assertIn('Luna', agents)
        self.assertIn('01-dashboard', agents)
        self.assertIn('native view: unavailable', agents)
        self.assertEqual(len(module.bindings(snapshot)), 2)
        (self.root / 'roster.json').write_text('{"observed_at":"bad","agents":[]}')
        _, snapshot = self.snapshot()
        self.assertIn('Roster: unavailable', '\n'.join(module.render(snapshot)))

    def test_plain_render_strips_controls_and_has_no_tmux_side_effect(self):
        self.current['next'] = 'Review\x1b[31m candidate\x00\n' + '\u202e' + 'x' * 10000
        self.write_inputs()
        log, env = self.fake_tmux()
        before = {name: (self.root / name).read_bytes() for name in
                  ('config.json', 'CURRENT.json', 'luna.json', 'manifest.json')}
        result = subprocess.run([sys.executable, str(SCRIPT), '--config', str(self.root / 'config.json'),
                                 '--render'], capture_output=True, text=True, timeout=5, env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('=== Infra ===', result.stdout)
        for control in ['\x1b', '\x00', '\u202e']:
            self.assertNotIn(control, result.stdout)
        self.assertFalse(log.exists())
        self.assertEqual(before, {name: (self.root / name).read_bytes() for name in before})
        help_result = subprocess.run([sys.executable, str(SCRIPT), '--help'], capture_output=True,
                                     text=True, timeout=5)
        self.assertEqual(help_result.returncode, 0)

    def test_narrow_pty_navigation_only_selects_explicit_native_window(self):
        self.current['next'] = 'Long next action ' * 700
        self.write_inputs()
        log, env = self.fake_tmux()
        master, slave = pty.openpty()
        self.addCleanup(os.close, master)
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 8, 24, 0, 0))
        process = subprocess.Popen([sys.executable, str(SCRIPT), '--config', str(self.root / 'config.json')],
                                   stdin=slave, stdout=slave, stderr=slave, env=env)
        os.close(slave)
        self.addCleanup(lambda: process.kill() if process.poll() is None else None)
        output = bytearray()

        def drain(duration):
            until = time.monotonic() + duration
            while time.monotonic() < until:
                if select.select([master], [], [], 0.05)[0]:
                    try:
                        output.extend(os.read(master, 65536))
                    except OSError:
                        break

        drain(0.4)
        for key in [b'2', b'4', b'5', b'6', b'7', b'1', b'j', b'j', b'k', b'r', b'?', b'?', b'3']:
            os.write(master, key)
            drain(0.06)
        self.assertFalse(log.exists(), 'navigation must not call tmux before explicit view jump')
        os.write(master, b'\r')
        drain(0.3)
        os.write(master, b'q')
        drain(0.2)
        self.assertEqual(process.wait(timeout=3), 0, output.decode(errors='replace'))
        self.assertEqual([json.loads(line) for line in log.read_text().splitlines()],
                         [['select-window', '-t', '@4']])
        for label in [b'CORENIUS', b'Overview', b'Tasks', b'Agents', b'Work', b'Evidence', b'VCS', b'Infra']:
            self.assertIn(label, output)

    def test_input_bounds_and_scalar_shapes_do_not_admit_false_source_data(self):
        self.current['next'] = ['wrong-shape']
        self.write_inputs()
        module, snapshot = self.snapshot()
        self.assertIn('CURRENT: unavailable', '\n'.join(module.render(snapshot)))
        (self.root / 'CURRENT.json').write_bytes(b' ' * (1024 * 1024 + 1))
        _, snapshot = self.snapshot()
        self.assertIn('exceeds 1 MiB', '\n'.join(module.render(snapshot)))

    def test_missing_current_goal_uses_explicit_operator_config_goal(self):
        del self.current['goal']
        self.config['goal'] = 'Prove the permanent MAC happy path'
        self.write_inputs()
        module, snapshot = self.snapshot()
        overview = '\n'.join(module.sections(snapshot)['Overview'])
        self.assertIn('Goal (CURRENT): unavailable', overview)
        self.assertIn('Goal (operator config): Prove the permanent MAC happy path', overview)
        self.current['goal'] = 'Updated source goal'
        self.write_inputs()
        _, snapshot = self.snapshot()
        self.assertNotIn('Goal (operator config)', '\n'.join(module.sections(snapshot)['Overview']))

    def test_quiescent_writer_and_unknown_manifest_binding_remain_distinct(self):
        self.current['writer'] = None
        self.current['thread'] = None
        self.manifest['bindings'][0]['current'] = None
        self.write_inputs()
        module, snapshot = self.snapshot()
        agents = '\n'.join(module.sections(snapshot)['Agents'])
        self.assertIn('Writer (CURRENT): none', agents)
        self.assertIn('implementation-author | unknown | window @4', agents)
        self.assertNotIn('CURRENT: unavailable', '\n'.join(module.render(snapshot)))
        self.assertNotIn('implementation author (CURRENT binding)', agents)
        del self.current['writer']
        self.write_inputs()
        _, snapshot = self.snapshot()
        self.assertIn('Writer (CURRENT): unavailable', '\n'.join(module.sections(snapshot)['Agents']))


if __name__ == '__main__':
    unittest.main()

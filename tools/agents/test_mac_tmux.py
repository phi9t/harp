#!/usr/bin/env python3
"""Behavior checks use disposable tmux servers and a viewer without model calls."""
import importlib.util
from datetime import datetime, timedelta, timezone
import hashlib
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

    def publish_runtime(self, bindings, age_seconds=0):
        path = self.root / 'runtime.json'
        self.config['runtime_bindings_path'] = str(path)
        value = {'observed_at': (datetime.now(timezone.utc) - timedelta(seconds=age_seconds)).isoformat(),
                 'bindings': bindings}
        path.write_text(json.dumps(value))
        return path

    def runtime_worker(self, thread, role='product-worker-1', **fields):
        return dict(role=role, thread=thread, cwd=str(self.root),
                    endpoint=self.config['endpoint'], current=True, **fields)

    def publish_external(self, bindings, age_seconds=0):
        path = self.root / 'external.json'
        self.config['external_views_path'] = str(path)
        path.write_text(json.dumps({'observed_at': time.time() - age_seconds, 'bindings': bindings}))
        return path

    def external_observer(self, thread=None):
        thread = thread or str(uuid.UUID(int=80))
        window = self.tmux('new-window', '-d', '-P', '-F', '#{window_id}', '-t', '=operator:',
                           '-n', 'trial-lead-original', 'sleep 3600')
        pane = self.tmux('display-message', '-p', '-t', window, '#{pane_id}')
        self.tmux('set-option', '-w', '-t', window, '@core_private_grant', 'PRIVATE_GRANT_SENTINEL')
        return {'role': 'lead', 'thread': thread, 'endpoint': 'ws://127.0.0.1:17189',
                'cwd': '/workspace', 'window': window, 'pane': pane}

    def observer_metadata(self, observer):
        return (self.tmux('display-message', '-p', '-t', observer['window'], '#{window_name}'),
                self.tmux('show-options', '-w', '-t', observer['window']),
                self.tmux('display-message', '-p', '-t', '=operator:', '#{window_id}'))

    def publish_observer_runtime(self, observer, age_seconds=0, **fields):
        runtime = {key: observer[key] for key in ('role', 'thread', 'endpoint', 'cwd')}
        path = self.publish_runtime([dict(runtime, current=True, **fields)], age_seconds)
        value = json.loads(path.read_text())
        value['observed_at'] = time.time() - age_seconds
        value['projection_kind'] = 'coordinator snapshot; refresh not claimed'
        path.write_text(json.dumps(value))
        return path

    def assert_observer_not_mutated(self, observer, commands):
        mutations = {'new-window', 'set-option', 'rename-window', 'select-window', 'respawn-pane',
                     'respawn-window', 'kill-window', 'kill-pane', 'send-keys'}
        for command in commands:
            verb = command[len(self.tmux_argv)]
            if verb in mutations:
                self.assertNotIn(observer['window'], command)
                self.assertNotIn(observer['pane'], command)
        self.assertNotIn('PRIVATE_GRANT_SENTINEL', json.dumps(commands))

    def test_external_observer_reused_readonly_despite_role_changes_and_host_cwd(self):
        self.reconcile()
        observer = self.external_observer()
        path = self.publish_external([observer])
        runtime_path = self.publish_observer_runtime(observer)
        before = self.observer_metadata(observer)
        ids, count = self.views(), len(self.commands)
        first = self.reconcile()
        self.assertEqual(first['health'], 'ok')
        view = next(b for b in first['bindings'] if b['thread'] == observer['thread'])
        self.assertIs(view.get('managed'), False)
        self.assertEqual((view['window'], view['pane']), (observer['window'], observer['pane']))
        self.assertEqual(view['external_reference']['digest'], hashlib.sha256(path.read_bytes()).hexdigest())
        self.assertEqual(view['external_reference']['observed_at'], json.loads(path.read_text())['observed_at'])
        self.assertEqual(view['sources'][0]['projection_kind'], 'coordinator snapshot; refresh not claimed')
        value = json.loads(runtime_path.read_text())
        value['bindings'][0]['role'] = 'replacement-lead'
        runtime_path.write_text(json.dumps(value))
        second = self.reconcile()
        self.assertEqual(next(b for b in second['bindings'] if b['thread'] == observer['thread'])['role'], 'replacement-lead')
        self.assertEqual(self.views(), ids)
        self.assertEqual(self.observer_metadata(observer), before)
        self.assert_observer_not_mutated(observer, self.commands[count:])
        self.assertFalse(any(c[len(self.tmux_argv)] == 'new-window' for c in self.commands[count:]))
        self.assertNotIn('PRIVATE_GRANT_SENTINEL', json.dumps(second))

    def test_bad_external_refs_fail_closed_and_preserve_prior_unknown_observer(self):
        self.reconcile()
        observer = self.external_observer()
        path = self.publish_external([observer])
        self.publish_observer_runtime(observer)
        first = self.reconcile()
        prior = next(b for b in first['bindings'] if b['thread'] == observer['thread'])
        ids, before = self.views(), self.observer_metadata(observer)
        variants = [(None, 'missing'), ('{', 'degraded'),
                    (json.dumps({'observed_at': time.time() - 360, 'bindings': [observer]}), 'stale'),
                    (json.dumps({'observed_at': time.time(), 'bindings': [observer, observer]}), 'degraded'),
                    (json.dumps({'observed_at': time.time(), 'bindings': [observer],
                                 'auth': 'SECRET_SENTINEL'}), 'degraded')]
        for content, status in variants:
            with self.subTest(status=status):
                if content is None:
                    path.unlink()
                else:
                    path.write_text(content)
                count = len(self.commands)
                report = self.reconcile()
                self.assertEqual(report['health'], 'degraded')
                self.assertEqual(report['external_views_source']['status'], status)
                view = next(b for b in report['bindings'] if b['thread'] == observer['thread'])
                self.assertIs(view['current'], None)
                self.assertIs(view['managed'], False)
                self.assertEqual(view['sources'], [dict(prior['sources'][0], current=None)])
                self.assertEqual(self.views(), ids)
                self.assertEqual(self.observer_metadata(observer), before)
                self.assertFalse(any(c[len(self.tmux_argv)] == 'new-window' for c in self.commands[count:]))
                self.assert_observer_not_mutated(observer, self.commands[count:])
                self.assertNotIn('SECRET_SENTINEL', json.dumps(report))

    def test_vanished_external_pane_reports_pending_and_never_replaces_client(self):
        self.reconcile()
        observer = self.external_observer()
        self.publish_external([observer])
        self.publish_observer_runtime(observer)
        self.reconcile()
        self.tmux('split-window', '-d', '-t', observer['window'], 'sleep 3600')
        self.tmux('kill-pane', '-t', observer['pane'])  # Disposable fixture only.
        ids, count = self.views(), len(self.commands)
        report = self.reconcile()
        view = next(b for b in report['bindings'] if b['thread'] == observer['thread'])
        self.assertIsNone(view['current'])
        self.assertIs(view['managed'], False)
        self.assertTrue(any('external observer pending' in error for error in report['errors']))
        self.assertEqual(self.views(), ids)
        self.assertFalse(any(c[len(self.tmux_argv)] == 'new-window' for c in self.commands[count:]))
        self.assert_observer_not_mutated(observer, self.commands[count:])

    def test_external_refs_never_admit_without_fresh_runtime_and_owned_view_wins(self):
        self.reconcile()
        observer = self.external_observer()
        self.publish_external([observer])
        runtime_path = self.publish_observer_runtime(observer, age_seconds=360)
        ids = self.views()
        report = self.reconcile()
        self.assertFalse(any(b['thread'] == observer['thread'] for b in report['bindings']))
        self.assertEqual(self.views(), ids)
        self.publish_observer_runtime(observer)
        first = self.reconcile()
        runtime_path.write_text(json.dumps({'observed_at': time.time(), 'status': 'unavailable', 'bindings': []}))
        report = self.reconcile()
        view = next(b for b in report['bindings'] if b['thread'] == observer['thread'])
        self.assertIsNone(view['current'])
        self.assertEqual(view['sources'], [dict(next(b for b in first['bindings']
                         if b['thread'] == observer['thread'])['sources'][0], current=None)])
        self.assertEqual(self.views(), ids)
        duplicate = dict(observer, thread=THREAD1, endpoint=self.config['endpoint'])
        self.publish_external([duplicate])
        self.publish_observer_runtime(duplicate)
        count, before = len(self.commands), self.observer_metadata(observer)
        report = self.reconcile()
        managed = [b for b in report['bindings'] if b['thread'] == THREAD1]
        self.assertEqual(len(managed), 1)
        self.assertIs(managed[0]['managed'], True)
        self.assertNotEqual(managed[0]['window'], observer['window'])
        self.assertEqual(self.observer_metadata(observer), before)
        self.assert_observer_not_mutated(observer, self.commands[count:])

    def test_invalid_or_absent_external_ref_cannot_launch_first_runtime_client(self):
        self.reconcile()
        observer = self.external_observer()
        self.publish_observer_runtime(observer)
        path = self.publish_external([])
        ids = self.views()
        values = [{'observed_at': time.time(), 'bindings': []},
                  {'observed_at': time.time(), 'bindings': [dict(observer, pane='%999999')]},
                  {'observed_at': time.time(), 'bindings': [dict(observer, current=True)]},
                  {'observed_at': time.time(), 'bindings': [dict(observer, endpoint='ws://u:SECRET_SENTINEL@host/')]},
                  {'observed_at': time.time(), 'bindings': [dict(observer, thread=THREAD2)]}]
        for value in values:
            with self.subTest(fields=sorted(value['bindings'][0]) if value['bindings'] else []):
                path.write_text(json.dumps(value))
                count = len(self.commands)
                report = self.reconcile()
                self.assertFalse(any(b['thread'] == observer['thread'] for b in report['bindings']))
                self.assertEqual(report['health'], 'degraded')
                self.assertEqual(self.views(), ids)
                self.assertFalse(any(c[len(self.tmux_argv)] == 'new-window' for c in self.commands[count:]))
                self.assertNotIn('SECRET_SENTINEL', json.dumps(report))

    def test_external_mode_does_not_probe_legacy_core_launch_metadata(self):
        self.reconcile()
        observer = self.external_observer()
        for field in ('role', 'thread'):
            self.tmux('set-option', '-w', '-t', observer['window'], '@corenius_' + field, observer[field])
        argv = self.tool.resume_argv(self.config, observer)
        legacy = self.root / 'core-legacy.json'
        legacy.write_text(json.dumps({'session': 'operator', 'bindings': [dict(observer, argv=argv)]}))
        self.config['adopt_manifest_path'] = str(legacy)
        path = self.publish_external([observer])
        path.unlink()
        self.publish_observer_runtime(observer)
        before, count = self.observer_metadata(observer), len(self.commands)
        self.reconcile()
        self.assertFalse(any('#{pane_start_command}' in c for c in self.commands[count:]))
        self.assertEqual(self.observer_metadata(observer), before)
        self.assert_observer_not_mutated(observer, self.commands[count:])

    def test_published_lead_and_two_product_workers_have_exact_native_clients(self):
        self.current.write_text(json.dumps({'writer': None, 'thread': None,
            'writer_state': 'no active implementation writer'}))
        worker2 = str(uuid.UUID(int=20))
        path = self.publish_runtime([
            self.runtime_worker(THREAD2, 'product-lead'),
            self.runtime_worker(THREAD1), self.runtime_worker(worker2, 'product-worker-2')])
        report = self.reconcile()
        self.assertEqual(len(report['bindings']), 3)
        self.assertTrue(all(b['current'] is True for b in report['bindings']))
        self.assertEqual(len(self.views()), 4)
        source = report['runtime_bindings_source']
        self.assertEqual(source['status'], 'ok')
        self.assertEqual(source['digest'], hashlib.sha256(path.read_bytes()).hexdigest())
        self.assertEqual(source['observed_at'], json.loads(path.read_text())['observed_at'])
        lead = next(b for b in report['bindings'] if b['thread'] == THREAD2)
        self.assertIn({'source': 'runtime_bindings', 'role': 'product-lead', 'current': True,
                       'observed_at': source['observed_at'], 'digest': source['digest']}, lead['sources'])
        rows = self.wait_log(4)
        for thread in (THREAD1, THREAD2, worker2):
            self.assertEqual(rows.count(['resume', '--remote', self.config['endpoint'],
                                         '--include-non-interactive', thread]), 1)

    def test_runtime_dedup_keeps_source_roles_and_endpoint_identity(self):
        other = socket.socket(socket.AF_UNIX)
        other.bind(str(self.root / 'other.sock'))
        self.addCleanup(other.close)
        extra = str(uuid.UUID(int=21))
        self.config['extra_bindings'] = [{'role': 'operator-extra', 'thread': extra, 'cwd': str(self.root)}]
        bindings = [self.runtime_worker(THREAD1), self.runtime_worker(THREAD2, 'product-lead'),
                    self.runtime_worker(extra, 'product-worker-2'), self.runtime_worker(THREAD1)]
        bindings.append(dict(self.runtime_worker(THREAD1), endpoint='unix://' + str(self.root / 'other.sock')))
        self.publish_runtime(bindings)
        report = self.reconcile()
        self.assertEqual(len(report['bindings']), 4)
        same_store = next(b for b in report['bindings'] if b['thread'] == THREAD1 and b['endpoint'] == self.config['endpoint'])
        self.assertEqual({s['role'] for s in same_store['sources']}, {'worker-1', 'product-worker-1'})
        self.assertEqual(len(same_store['sources']), 2)
        ids = self.views()
        self.assertEqual(len(self.reconcile()['bindings']), 4)
        self.assertEqual(self.views(), ids)

    def test_runtime_replacement_and_valid_empty_retain_historical_clients(self):
        old, new = str(uuid.UUID(int=22)), str(uuid.UUID(int=23))
        self.publish_runtime([self.runtime_worker(old)])
        first = self.reconcile()
        self.assertIn(old, [b['thread'] for b in first['bindings']])
        ids = self.views()
        self.publish_runtime([self.runtime_worker(new)])
        report = self.reconcile()
        self.assertTrue(set(ids).issubset(self.views()))
        self.assertIs(next(b['current'] for b in report['bindings'] if b['thread'] == old), False)
        self.assertIs(next(b['current'] for b in report['bindings'] if b['thread'] == new), True)
        ids = self.views()
        self.publish_runtime([])
        report = self.reconcile()
        self.assertEqual(report['health'], 'ok')
        self.assertEqual(self.views(), ids)
        self.assertTrue(all(b['current'] is False for b in report['bindings'] if b['thread'] in (old, new)))

    def test_unavailable_runtime_preserves_unknown_membership_without_admission(self):
        published, unpublished = str(uuid.UUID(int=24)), str(uuid.UUID(int=25))
        path = self.publish_runtime([self.runtime_worker(published)])
        first = self.reconcile()
        self.assertIn(published, [b['thread'] for b in first['bindings']])
        provenance = next(b for b in first['bindings'] if b['thread'] == published)['sources'][0]
        ids = self.views()
        for content, expected in ((None, 'missing'), ('{', 'degraded'), ('{}', 'degraded'),
                                  (json.dumps({'observed_at': (datetime.now(timezone.utc) - timedelta(minutes=6)).isoformat(),
                                     'bindings': [self.runtime_worker(unpublished)]}), 'stale')):
            with self.subTest(status=expected, content=content):
                if content is None:
                    path.unlink()
                else:
                    path.write_text(content)
                report = self.reconcile()
                self.assertEqual(self.views(), ids)
                self.assertEqual(report['health'], 'degraded')
                self.assertEqual(report['runtime_bindings_source']['status'], expected)
                prior = next(b for b in report['bindings'] if b['thread'] == published)
                self.assertIsNone(prior['current'])
                self.assertEqual(prior['sources'], [dict(provenance, current=None)])
                self.assertFalse(any(b['thread'] == unpublished for b in report['bindings']))
                self.assertIs(next(b['current'] for b in report['bindings'] if b['thread'] == THREAD1), True)

    def test_invalid_public_runtime_fields_are_rejected_without_secret_output(self):
        thread = str(uuid.UUID(int=26))
        path = self.publish_runtime([])
        self.reconcile()
        ids = self.views()
        good = self.runtime_worker(thread)
        variants = [dict(good, grant={'token': 'SECRET_SENTINEL'}), dict(good, current=1),
                    dict(good, thread=thread.upper()), dict(good, endpoint='ws://user:SECRET_SENTINEL@host/'),
                    dict(good, endpoint='ws://host/?token=SECRET_SENTINEL'),
                    {k: v for k, v in good.items() if k != 'endpoint'}]
        # UUID(int=26) contains letters, so uppercase is noncanonical.
        for item in variants:
            with self.subTest(item=item):
                self.publish_runtime([item])
                report = self.reconcile()
                self.assertEqual(report['health'], 'degraded')
                self.assertEqual(self.views(), ids)
                self.assertNotIn('SECRET_SENTINEL', json.dumps(report))
                self.assertNotIn('SECRET_SENTINEL', json.dumps(self.commands))
        path.write_text(json.dumps({'observed_at': datetime.now(timezone.utc).isoformat(),
                                    'bindings': [good], 'auth': 'SECRET_SENTINEL'}))
        report = self.reconcile()
        self.assertEqual(report['health'], 'degraded')
        self.assertEqual(self.views(), ids)
        self.assertNotIn('SECRET_SENTINEL', json.dumps(report))

    def test_unavailable_envelope_retains_provenance_and_unknown_prior_membership(self):
        published, unpublished = str(uuid.UUID(int=40)), str(uuid.UUID(int=41))
        path = self.publish_runtime([self.runtime_worker(published)])
        first = self.reconcile()
        prior_source = next(b for b in first['bindings'] if b['thread'] == published)['sources'][0]
        ids = self.views()
        for status, items in [('unavailable', []), ('unavailable', [self.runtime_worker(unpublished)]),
                              ('pending', [self.runtime_worker(unpublished)])]:
            with self.subTest(status=status, contradictory=bool(items)):
                stamp = datetime.now(timezone.utc).isoformat()
                value = {'observed_at': stamp, 'status': status, 'bindings': items,
                         'reason': 'Original native namespaces gone; restored admission not yet validated. '
                                   'Operator clients remain separate. SECRET_SENTINEL'}
                path.write_text(json.dumps(value))
                before_commands = len(self.commands)
                report = self.reconcile()
                self.assertEqual(report['health'], 'degraded')
                source = report['runtime_bindings_source']
                self.assertEqual(source['status'], 'unavailable')
                self.assertEqual(source['declared_status'], status)
                self.assertEqual(source['observed_at'], stamp)
                self.assertEqual(source['digest'], hashlib.sha256(path.read_bytes()).hexdigest())
                prior = next(b for b in report['bindings'] if b['thread'] == published)
                self.assertIsNone(prior['current'])
                self.assertEqual(prior['sources'], [dict(prior_source, current=None)])
                self.assertFalse(any(b['thread'] == unpublished for b in report['bindings']))
                self.assertEqual(self.views(), ids)
                self.assertFalse(any(c[len(self.tmux_argv)] == 'new-window'
                                     for c in self.commands[before_commands:]))
                self.assertNotIn('SECRET_SENTINEL', json.dumps(report))
                self.assertNotIn('SECRET_SENTINEL', json.dumps(self.commands))

    def test_numeric_projection_time_preserves_source_and_unknown_membership(self):
        thread = str(uuid.UUID(int=70))
        path = self.publish_runtime([self.runtime_worker(thread)])
        stamp = time.time()
        kind = 'coordinator snapshot of author public scoped roster; refresh not claimed'
        path.write_text(json.dumps({'observed_at': stamp, 'projection_kind': kind,
                                    'bindings': [self.runtime_worker(thread)]}))
        first = self.reconcile()
        self.assertEqual(first['health'], 'ok')
        source = first['runtime_bindings_source']
        self.assertEqual(source['observed_at'], stamp)
        self.assertEqual(source['projection_kind'], kind)
        self.assertEqual(source['digest'], hashlib.sha256(path.read_bytes()).hexdigest())
        view = next(b for b in first['bindings'] if b['thread'] == thread)
        self.assertEqual(view['sources'][0]['observed_at'], stamp)
        self.assertEqual(view['sources'][0]['projection_kind'], kind)
        ids = self.views()
        path.unlink()
        report = self.reconcile()
        retained = next(b for b in report['bindings'] if b['thread'] == thread)
        self.assertIsNone(retained['current'])
        self.assertEqual(retained['sources'], [dict(view['sources'][0], current=None)])
        self.assertEqual(self.views(), ids)

    def test_numeric_time_and_projection_kind_reject_unsafe_envelopes(self):
        path = self.publish_runtime([])
        self.reconcile()
        ids = self.views()
        valid = {'observed_at': time.time(), 'projection_kind': 'public snapshot',
                 'bindings': [self.runtime_worker(str(uuid.UUID(int=71)))]}
        variants = [dict(valid, observed_at=value) for value in (True, float('nan'), float('inf'), 1e100)]
        variants += [dict(valid, projection_kind=value) for value in ('x' * 4097, {'auth': 'SECRET_SENTINEL'})]
        variants += [dict(valid, auth='SECRET_SENTINEL')]
        for value in variants:
            with self.subTest(timestamp_type=type(value['observed_at']).__name__):
                path.write_text(json.dumps(value))
                report = self.reconcile()
                self.assertEqual(report['runtime_bindings_source']['status'], 'degraded')
                self.assertEqual(self.views(), ids)
                self.assertNotIn('SECRET_SENTINEL', json.dumps(report))

    def test_real_scoped_row_observation_is_evidence_under_global_admission_gate(self):
        self.reconcile()
        # Exact public row read from the configured coordinator snapshot; no private fields.
        row = {'role': 'lead', 'thread': '01a10a6f-b309-7a92-854d-25092fc7a92b',
               'endpoint': 'ws://127.0.0.1:17189', 'cwd': '/workspace', 'current': True,
               'observed_at': 1791177266.0653453}
        observer = self.external_observer(row['thread'])
        self.publish_external([observer])
        path = self.publish_runtime([])
        envelope = {'observed_at': 1791177266.0653453,
                    'projection_kind': 'coordinator snapshot of author public scoped roster; refresh not claimed',
                    'bindings': [row]}
        path.write_text(json.dumps(envelope))
        report = self.reconcile()
        self.assertEqual(report['runtime_bindings_source']['status'], 'stale')
        self.assertFalse(any(b['thread'] == row['thread'] for b in report['bindings']))
        # Only the fixture's global observation advances; old row observations do not gain a TTL.
        envelope['observed_at'] = time.time()
        for observation in (row['observed_at'], '2026-10-05T04:00:00+00:00'):
            row['observed_at'] = observation
            path.write_text(json.dumps(envelope))
            report = self.reconcile()
            self.assertEqual(report['health'], 'ok')
            view = next(b for b in report['bindings'] if b['thread'] == row['thread'])
            self.assertIs(view['managed'], False)
            self.assertEqual(view['sources'][0]['binding_observed_at'], observation)
            self.assertEqual(view['sources'][0]['observed_at'], envelope['observed_at'])
            self.assertEqual(view['sources'][0]['projection_kind'], envelope['projection_kind'])
        prior_source = view['sources'][0]
        row['observed_at'] = True
        path.write_text(json.dumps(envelope))
        report = self.reconcile()
        retained = next(b for b in report['bindings'] if b['thread'] == row['thread'])
        self.assertIsNone(retained['current'])
        self.assertEqual(retained['sources'], [dict(prior_source, current=None)])

    def test_invalid_row_observation_and_external_row_extras_remain_rejected(self):
        row = self.runtime_worker(str(uuid.UUID(int=90)))
        path = self.publish_runtime([])
        invalid = (True, float('nan'), float('inf'), 1e100, '2026-10-05T04:00:00',
                   '2026-10-05T04:00:00.' + '0' * 4097 + '+00:00')
        for observation in invalid:
            with self.subTest(timestamp_type=type(observation).__name__):
                path.write_text(json.dumps({'observed_at': time.time(),
                    'bindings': [dict(row, observed_at=observation)]}))
                errors = []
                views, source = self.tool.runtime_bindings(self.config, errors)
                self.assertEqual(views, [])
                self.assertEqual(source['status'], 'degraded')
                self.assertTrue(errors)
        for extra in ({'auth': 'SECRET_SENTINEL'}, {'unexpected': True},
                      {'endpoint': 'ws://u:SECRET_SENTINEL@host/'}):
            path.write_text(json.dumps({'observed_at': time.time(),
                'bindings': [dict(row, observed_at=time.time(), **extra)]}))
            errors = []
            views, source = self.tool.runtime_bindings(self.config, errors)
            self.assertEqual(views, [])
            self.assertEqual(source['status'], 'degraded')
            self.assertNotIn('SECRET_SENTINEL', json.dumps((views, source, errors)))
        external = {key: row[key] for key in ('role', 'thread', 'endpoint', 'cwd')}
        with self.assertRaises(ValueError):
            self.tool.public_binding(dict(external, window='@4', pane='%5', observed_at=time.time()), external=True)

    def test_positive_envelope_and_legacy_rosters_admit_valid_public_clients(self):
        for number, status in enumerate((None, 'observed', 'available', 'ok'), 50):
            with self.subTest(status=status):
                thread = str(uuid.UUID(int=number))
                path = self.publish_runtime([self.runtime_worker(thread)])
                value = json.loads(path.read_text())
                if status is not None:
                    value.update(status=status, reason='Public coordinator projection')
                    path.write_text(json.dumps(value))
                report = self.reconcile()
                self.assertEqual(report['health'], 'ok')
                self.assertEqual(report['runtime_bindings_source']['status'], 'ok')
                admitted = next(b for b in report['bindings'] if b['thread'] == thread)
                self.assertIs(admitted['current'], True)
                self.assertEqual(admitted['sources'][0]['source'], 'runtime_bindings')
                self.assertEqual(admitted['sources'][0]['digest'], hashlib.sha256(path.read_bytes()).hexdigest())
                self.assertIn(['resume', '--remote', self.config['endpoint'], '--include-non-interactive', thread],
                              self.wait_log(number - 46))

    def test_envelope_extras_unknown_status_and_credentials_reject_without_private_output(self):
        thread = str(uuid.UUID(int=60))
        path = self.publish_runtime([])
        self.reconcile()
        ids = self.views()
        valid = {'observed_at': datetime.now(timezone.utc).isoformat(), 'status': 'available',
                 'bindings': [self.runtime_worker(thread)], 'reason': 'SECRET_SENTINEL'}
        variants = [dict(valid, auth='SECRET_SENTINEL'), dict(valid, unexpected=True),
                    dict(valid, status='healthy'), dict(valid, status=True),
                    dict(valid, reason={'private': 'SECRET_SENTINEL'}),
                    dict(valid, bindings=[dict(self.runtime_worker(thread),
                                              endpoint='ws://user:SECRET_SENTINEL@host/')])]
        for value in variants:
            with self.subTest(fields=sorted(value)):
                path.write_text(json.dumps(value))
                report = self.reconcile()
                self.assertEqual(report['health'], 'degraded')
                self.assertEqual(report['runtime_bindings_source']['status'], 'degraded')
                self.assertEqual(self.views(), ids)
                self.assertNotIn('SECRET_SENTINEL', json.dumps(report))
                self.assertNotIn('SECRET_SENTINEL', json.dumps(self.commands))

    def test_source_unknown_states_are_independent_even_for_shared_roles(self):
        product = str(uuid.UUID(int=27))
        path = self.publish_runtime([self.runtime_worker(product, 'worker-1'),
                                     self.runtime_worker(THREAD1, 'product-worker-2')])
        self.reconcile()
        self.current.unlink()
        report = self.reconcile()
        self.assertIs(next(b['current'] for b in report['bindings'] if b['thread'] == product), True)
        shared = next(b for b in report['bindings'] if b['thread'] == THREAD1)
        self.assertIs(shared['current'], True)
        self.assertEqual({s['source']: s['current'] for s in shared['sources']},
                         {'implementation': None, 'runtime_bindings': True})
        self.publish_runtime([])
        report = self.reconcile()
        self.assertIs(next(b['current'] for b in report['bindings'] if b['thread'] == product), False)
        self.assertIsNone(next(b['current'] for b in report['bindings'] if b['thread'] == THREAD1))
        path.unlink()
        report = self.reconcile()
        self.assertIsNone(next(b['current'] for b in report['bindings'] if b['thread'] == product))

    def test_runtime_false_is_not_admitted_and_true_duplicate_wins(self):
        product = str(uuid.UUID(int=28))
        historical = dict(self.runtime_worker(product), current=False)
        self.publish_runtime([historical])
        report = self.reconcile()
        self.assertFalse(any(b['thread'] == product for b in report['bindings']))
        self.publish_runtime([self.runtime_worker(product), historical])
        report = self.reconcile()
        view = next(b for b in report['bindings'] if b['thread'] == product)
        self.assertIs(view['current'], True)
        self.assertEqual(len(view['sources']), 1)

    def test_bounded_runtime_inputs_reject_entire_roster_before_admitting_clients(self):
        product = str(uuid.UUID(int=29))
        path = self.publish_runtime([])
        self.reconcile()
        ids = self.views()
        variants = [{'observed_at': datetime.now(timezone.utc).isoformat(),
                     'bindings': [self.runtime_worker(product)] * 257},
                    {'observed_at': datetime.now(timezone.utc).isoformat(),
                     'bindings': [self.runtime_worker(product), dict(self.runtime_worker(product), role='x' * 4097)]},
                    {'observed_at': '2036-01-01T00:00:00+00:00', 'bindings': [self.runtime_worker(product)]},
                    {'observed_at': '2026-01-01T00:00:00', 'bindings': [self.runtime_worker(product)]}]
        for value in variants:
            with self.subTest(value_type='structured'):
                path.write_text(json.dumps(value))
                report = self.reconcile()
                self.assertEqual(report['runtime_bindings_source']['status'], 'degraded')
                self.assertEqual(self.views(), ids)
        for content in (' ' * (1024 * 1024 + 1), '[' * 1000 + '0' + ']' * 1000):
            with self.subTest(value_type='bounded'):
                path.write_text(content)
                report = self.reconcile()
                self.assertEqual(report['runtime_bindings_source']['status'], 'degraded')
                self.assertEqual(self.views(), ids)

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

"""Exercise the correction fixture when delivery is acknowledged before polling.

The scripted public API adapter stops at the fairness checks. It does not claim
native runtime qualification; it covers the fixture's delivery prerequisite.
"""
import importlib.util
import os
import pathlib
import runpy
import tempfile
import unittest

REPO = pathlib.Path(__file__).resolve().parents[4]
TEMP_ROOT = str(runpy.run_path(str(REPO / 'tools/temp_root.py'))['temporary_root']())
tempfile.tempdir = TEMP_ROOT
os.environ['TMPDIR'] = TEMP_ROOT
spec = importlib.util.spec_from_file_location(
    'correction_authority', pathlib.Path(__file__).with_name('correction_authority.py'))
correction_authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(correction_authority)


class ReachedFairnessChecks(Exception):
    pass


class CorrectionAuthorityPeerWait(unittest.TestCase):
    def test_delivered_peer_reaches_fairness_checks(self):
        for delivered in ('acknowledged', 'presented'):
            with self.subTest(delivered=delivered), tempfile.TemporaryDirectory() as directory:
                self.exercise_peer(pathlib.Path(directory), delivered)

    def exercise_peer(self, root, delivered):
        findings = [
            {'id': 'f0', 'epoch': 1, 'attempt': 'a0', 'issue': 'independent-0-0'},
            {'id': 'f1', 'epoch': 1, 'attempt': 'a0', 'issue': 'independent-0-1'},
            {'id': 'f2', 'epoch': 1, 'attempt': 'a1', 'issue': 'independent-1-0'},
        ]
        rows = []
        registry_calls = 0

        def query(kind, args=None):
            nonlocal registry_calls
            if kind == 'snapshot':
                return {'revision': 1}
            if kind == 'monitor_context':
                return {'findings': findings}
            if kind == 'monitor_records':
                return {'items': rows}
            if kind == 'codex_registry':
                registry_calls += 1
                if registry_calls > 2:
                    raise ReachedFairnessChecks()
                return {'bindings': [
                    {'attempt': attempt, 'document': {
                        'ui_input_ready': True, 'current_turn': {'state': 'terminal'}}}
                    for attempt in ('a0', 'a1')]}
            raise AssertionError(kind)

        def request(path, value):
            kind = value['command']['kind']
            result = {}
            if kind == 'register_monitor_stream':
                result = {'stream': 's'}
            elif kind == 'seal_monitor_batch':
                result = {'batch': 'b'}
            elif kind == 'replace_lead':
                result = {'principal': 'newlead', 'token': 'fixture'}
            elif kind == 'authorize_correction':
                identifier = str(len(rows))
                rows.append({'id': identifier, 'state': 'requested' if not rows else delivered})
                result = {'correction': identifier}
            return {'ok': {'result': result}}

        def wait(predicate, seconds=20):
            # Virtual polling preserves the deadline verdict without wall-clock sleep.
            for _ in range(round(seconds / 0.04)):
                if predicate():
                    return True
            raise AssertionError('behavior deadline')

        with self.assertRaises(ReachedFairnessChecks):
            correction_authority.exercise(
                root, query, None, request,
                [{'attempt': 'a0'}, {'attempt': 'a1'}],
                {'principal': 'lead', 'token': 'fixture'}, wait, 'fixture', 'fixture')


if __name__ == '__main__':
    unittest.main()

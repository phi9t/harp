"""Controller executor-boundary fixtures only; no runtime/service/process launch."""
import json
from pathlib import Path
import tempfile
import unittest
from trae_case_control import run_case

class ControllerBehavior(unittest.TestCase):
    def test_missing_reviewed_control_never_reaches_executor(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);case=root/'case.json';grant=root/'grant.json';out=root/'operation';out.mkdir()
            case.write_text('{}');grant.write_text(json.dumps({'status':'admitted','operation_id':'owned-test'}))
            calls=[]
            result=run_case(case_path=case,grant_path=grant,grant_revision='a'*40,operation_id='owned-test',repository=root,output_directory=out,executor=lambda invocation:calls.append(invocation),read_committed=lambda revision,path:b'')
            self.assertEqual(calls,[],'missing control must refuse at the executor boundary')
            self.assertFalse(result['executed'])

class DurableControllerBehavior(unittest.TestCase):
    def setUp(self):
        import trae_case_control as control
        self.control = control
        self.root = Path(control.__file__).resolve().parents[2]
        self.temp = tempfile.TemporaryDirectory(dir=self.root / '.scratch')
        self.addCleanup(self.temp.cleanup)
        self.folder = Path(self.temp.name)
        self.out = self.folder / 'operation'
        self.out.mkdir(mode=0o700)
        self.case = self.folder / 'case.json'
        self.grant = self.folder / 'grant.json'
        source = Path(control.__file__).relative_to(self.root).as_posix()
        body = {'command': {'argv': ['/recording-only'], 'env': {}, 'cwd': str(self.root)}, 'sources': {source: control.digest(Path(control.__file__).read_bytes())}}
        self.case.write_bytes(control.encoded(body))
        grant = {'version': 1, 'status': 'admitted', 'operation_id': 'fixture-operation', 'case_path': self.case.relative_to(self.root).as_posix(), 'case_sha256': control.digest(self.case.read_bytes()), 'command_sha256': control.digest(control.encoded(body['command'])), 'controller_sha256': body['sources'][source], 'input_commit': 'b'*40, 'output_directory': str(self.out), 'limits': {'max_invocations':1, 'namespace_starts':1, 'timeout_seconds':30, 'fixture_seconds':10, 'trae_starts':0, 'provider_requests':0, 'automatic_retry':False}}
        self.grant.write_bytes(control.encoded(grant))
        self.frozen = {('a'*40, self.case.relative_to(self.root).as_posix()): self.case.read_bytes(), ('a'*40, self.grant.relative_to(self.root).as_posix()): self.grant.read_bytes(), ('b'*40, source): Path(control.__file__).read_bytes()}
        self.calls = []

    def run_controller(self, executor=None):
        return run_case(case_path=self.case, grant_path=self.grant, grant_revision='a'*40, operation_id='fixture-operation', repository=self.root, output_directory=self.out, executor=executor or self.record, read_committed=lambda revision, path: self.frozen[(revision, path)])

    def record(self, invocation):
        import os
        self.assertTrue((self.out / 'started.json').exists())
        self.assertEqual(json.loads((self.out / 'started.json').read_text())['provenance'], 'injected-offline-fixture')
        os.write(invocation['stdout_fd'], b'recorded-only\n')
        self.calls.append(invocation)
        return 0

    def test_durable_boundary_and_duplicate_refusal(self):
        self.assertEqual(self.run_controller()['status'], 'terminal')
        retained = {p.name:p.read_bytes() for p in self.out.iterdir()}
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)
        self.assertEqual(retained, {p.name:p.read_bytes() for p in self.out.iterdir()})

    def test_concurrent_operation_executes_once(self):
        import concurrent.futures
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(lambda _: self.run_controller(), range(2)))
        self.assertEqual(sum(r['executed'] for r in results), 1)
        self.assertEqual(len(self.calls), 1)

    def test_changed_case_grant_and_source_refuse(self):
        for path in (self.case, self.grant):
            original = path.read_bytes()
            path.write_bytes(original+b' ')
            self.assertFalse(self.run_controller()['executed'])
            path.write_bytes(original)
        key = next(key for key in self.frozen if key[0] == 'b'*40)
        self.frozen[key] += b'changed'
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(self.calls, [])

    def test_failed_write_spends_reservation_without_execution(self):
        from unittest.mock import patch
        with patch('trae_case_control.os.write', side_effect=OSError('injected write fault')):
            self.assertFalse(self.run_controller()['executed'])
        self.assertTrue((self.out / 'started.json').exists())
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(self.calls, [])

    def test_fsync_failure_never_execute(self):
        from unittest.mock import patch
        with patch('trae_case_control.os.fsync', side_effect=OSError('injected fsync')):
            self.assertFalse(self.run_controller()['executed'])
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(self.calls, [])

    def test_short_writes_complete_and_fsync_precedes_executor(self):
        import os
        from unittest.mock import patch
        original_write, original_sync = os.write, os.fsync
        syncs = []
        def short(fd, data):
            return original_write(fd, data[:7])
        def sync(fd):
            syncs.append(os.fstat(fd).st_mode)
            return original_sync(fd)
        def execute(invocation):
            import stat
            self.assertTrue(any(stat.S_ISDIR(mode) for mode in syncs))
            self.assertGreaterEqual(len(syncs), 6)
            return self.record(invocation)
        with patch('trae_case_control.os.write', side_effect=short), patch('trae_case_control.os.fsync', side_effect=sync):
            self.assertEqual(self.run_controller(execute)['exit_code'], 0)

    def test_timeout_and_exception_are_retained_unknown_without_retry(self):
        def failure(invocation):
            self.calls.append(invocation)
            raise TimeoutError('recording executor only')
        result = self.run_controller(failure)
        self.assertEqual(result['status'], 'unknown')
        self.assertEqual(json.loads((self.out/'terminal.json').read_text()), result)
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)

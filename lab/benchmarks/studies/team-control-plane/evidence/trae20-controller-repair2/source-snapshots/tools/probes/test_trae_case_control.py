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
            self.assertEqual(self.run_controller(execute).get('exit_code'), 0)

    def test_timeout_and_exception_are_retained_unknown_without_retry(self):
        def failure(invocation):
            self.calls.append(invocation)
            raise TimeoutError('recording executor only')
        result = self.run_controller(failure)
        self.assertEqual(result['status'], 'unknown')
        self.assertEqual(json.loads((self.out/'terminal.json').read_text()), result)
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)

    def test_post_reservation_input_change_retains_hash_bound_refusal(self):
        grant_key = ('a'*40, self.grant.relative_to(self.root).as_posix())
        reads = 0
        def reader(revision, path):
            nonlocal reads
            if (revision, path) == grant_key:
                reads += 1
                if reads == 2:
                    self.case.write_bytes(self.case.read_bytes()+b' ')
            return self.frozen[(revision, path)]
        result = run_case(case_path=self.case, grant_path=self.grant, grant_revision='a'*40, operation_id='fixture-operation', repository=self.root, output_directory=self.out, executor=self.record, read_committed=reader)
        self.assertFalse(result['executed'])
        self.assertEqual(self.calls, [])
        terminal_bytes = (self.out/'terminal.json').read_bytes()
        self.assertTrue(terminal_bytes, 'healthy-storage refusal must retain terminal bytes')
        retained = json.loads(terminal_bytes)
        self.assertEqual(retained, result)
        started = json.loads((self.out/'started.json').read_text())
        self.assertEqual(retained['operation_id'], started['operation_id'])
        self.assertEqual(retained['case_sha256'], started['case_sha256'])
        self.assertEqual(retained['grant_sha256'], started['grant_sha256'])
        self.assertEqual(retained['command_sha256'], self.control.digest(self.control.encoded(started['command'])))

    def test_changed_command_with_updated_case_digest_is_refused(self):
        case = json.loads(self.case.read_bytes())
        case['command']['argv'] = ['/changed-recording-only']
        self.case.write_bytes(self.control.encoded(case))
        grant = json.loads(self.grant.read_bytes())
        grant['case_sha256'] = self.control.digest(self.case.read_bytes())
        self.grant.write_bytes(self.control.encoded(grant))
        self.frozen[('a'*40, self.case.relative_to(self.root).as_posix())] = self.case.read_bytes()
        self.frozen[('a'*40, self.grant.relative_to(self.root).as_posix())] = self.grant.read_bytes()
        result = self.run_controller()
        self.assertFalse(result['executed'])
        self.assertIn('command mismatch', result['error'])
        self.assertEqual(self.calls, [])

    def test_later_output_and_directory_fsync_faults_retain_refusal(self):
        import os
        from unittest.mock import patch
        original = os.fsync
        # Pure fixture outputs are separate operations; no runtime retry occurs.
        for fail_at in (2, 3, 4, 5, 6):
            with self.subTest(fsync_call=fail_at):
                self.out = self.folder / ('fsync-'+str(fail_at))
                self.out.mkdir(mode=0o700)
                grant = json.loads(self.grant.read_bytes())
                grant['output_directory'] = str(self.out)
                self.grant.write_bytes(self.control.encoded(grant))
                self.frozen[('a'*40, self.grant.relative_to(self.root).as_posix())] = self.grant.read_bytes()
                calls = 0
                def fail_once(fd):
                    nonlocal calls
                    calls += 1
                    if calls == fail_at:
                        raise OSError('injected later fsync')
                    return original(fd)
                with patch('trae_case_control.os.fsync', side_effect=fail_once):
                    result = self.run_controller()
                self.assertFalse(result['executed'])
                self.assertEqual(json.loads((self.out/'terminal.json').read_bytes()), result)
                self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(self.calls, [])

    def test_executor_exception_is_hash_bound_unknown_and_spent(self):
        def error(invocation):
            self.calls.append(invocation)
            raise RuntimeError('injected executor exception')
        result = self.run_controller(error)
        self.assertEqual(result['status'], 'unknown')
        self.assertEqual(result['operation_id'], 'fixture-operation')
        self.assertEqual(result['error'], 'RuntimeError: injected executor exception')
        self.assertEqual(json.loads((self.out/'terminal.json').read_bytes()), result)
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)

    def test_terminal_write_failure_preserves_partial_and_retains_unknown(self):
        import os
        from unittest.mock import patch
        original = os.write
        failed = False
        partial = b'{"partial":'
        def fault(fd, data):
            nonlocal failed
            target = os.readlink('/proc/self/fd/'+str(fd))
            if target.endswith('/terminal.json') and not failed:
                failed = True
                original(fd, partial)
                raise OSError('injected partial terminal write')
            return original(fd, data)
        with patch('trae_case_control.os.write', side_effect=fault):
            result = self.run_controller()
        self.assertTrue(result['executed'])
        self.assertEqual(result['status'], 'unknown')
        self.assertEqual((self.out/'terminal.json').read_bytes(), partial)
        start = json.loads((self.out/'started.json').read_bytes())
        for key in ('operation_id', 'grant_revision', 'grant_sha256', 'case_sha256', 'sources'):
            self.assertEqual(result[key], start[key])
        self.assertEqual(json.loads((self.out/'uncertainty.json').read_bytes()), result)
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)

    def test_terminal_fsync_failure_retains_separate_unknown(self):
        import os
        from unittest.mock import patch
        original = os.fsync
        seen = 0
        def fault(fd):
            nonlocal seen
            if os.readlink('/proc/self/fd/'+str(fd)).endswith('/terminal.json'):
                seen += 1
                if seen == 2:
                    raise OSError('injected terminal fsync')
            return original(fd)
        with patch('trae_case_control.os.fsync', side_effect=fault):
            result = self.run_controller()
        self.assertEqual(result['status'], 'unknown')
        self.assertEqual(json.loads((self.out/'terminal.json').read_bytes())['status'], 'terminal')
        self.assertEqual(json.loads((self.out/'uncertainty.json').read_bytes()), result)
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)

    def test_final_directory_fsync_failure_retains_unknown(self):
        import os
        from unittest.mock import patch
        original = os.fsync
        count = 0
        def fault(fd):
            nonlocal count
            count += 1
            if count == 10:
                raise OSError('injected final directory fsync')
            return original(fd)
        with patch('trae_case_control.os.fsync', side_effect=fault):
            result = self.run_controller()
        self.assertEqual(result['status'], 'unknown')
        self.assertEqual(json.loads((self.out/'uncertainty.json').read_bytes()), result)
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)

    def test_post_dispatch_output_fsync_faults_retain_unknown(self):
        import os
        from unittest.mock import patch
        original = os.fsync
        for fail_at in (7, 8):
            with self.subTest(fsync_call=fail_at):
                self.out = self.folder / ('post-output-'+str(fail_at))
                self.out.mkdir(mode=0o700)
                grant = json.loads(self.grant.read_bytes())
                grant['output_directory'] = str(self.out)
                self.grant.write_bytes(self.control.encoded(grant))
                self.frozen[('a'*40, self.grant.relative_to(self.root).as_posix())] = self.grant.read_bytes()
                count = 0
                def fault(fd):
                    nonlocal count
                    count += 1
                    if count == fail_at:
                        raise OSError('injected output persistence failure')
                    return original(fd)
                with patch('trae_case_control.os.fsync', side_effect=fault):
                    result = self.run_controller()
                self.assertTrue(result['executed'])
                self.assertEqual(result['status'], 'unknown')
                self.assertEqual(json.loads((self.out/'terminal.json').read_bytes()), result)
                self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 2)

    def test_persistent_terminal_storage_fault_reports_unknown_and_stays_spent(self):
        import os
        from unittest.mock import patch
        original = os.write
        def fault(fd, data):
            name = os.readlink('/proc/self/fd/'+str(fd))
            if name.endswith(('/terminal.json', '/uncertainty.json')):
                raise OSError('persistent injected terminal storage fault')
            return original(fd, data)
        with patch('trae_case_control.os.write', side_effect=fault):
            result = self.run_controller()
        self.assertTrue(result['executed'])
        self.assertEqual(result['status'], 'unknown')
        self.assertIn('persistent injected terminal storage fault', result['retention_error'])
        self.assertTrue((self.out/'started.json').read_bytes())
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(len(self.calls), 1)

    def test_prior_terminal_without_start_retains_new_refusal_and_spends_start(self):
        prior = b'{"prior_operation":"historical"}\n'
        (self.out/'terminal.json').write_bytes(prior)
        self.assertFalse((self.out/'started.json').exists())
        result = self.run_controller()
        self.assertFalse(result['executed'])
        self.assertEqual(result['status'], 'refused')
        self.assertEqual(self.calls, [])
        self.assertEqual((self.out/'terminal.json').read_bytes(), prior)
        self.assertTrue((self.out/'started.json').read_bytes())
        self.assertTrue((self.out/'uncertainty.json').exists(), 'spent reservation must retain new refusal separately from prior terminal')
        retained = json.loads((self.out/'uncertainty.json').read_bytes())
        self.assertEqual(retained, result)
        start = json.loads((self.out/'started.json').read_bytes())
        for key in ('operation_id', 'grant_revision', 'grant_sha256', 'case_sha256', 'sources'):
            self.assertEqual(retained[key], start[key])
        snapshot = {p.name:p.read_bytes() for p in self.out.iterdir()}
        self.assertFalse(self.run_controller()['executed'])
        self.assertEqual(self.calls, [])
        self.assertEqual({p.name:p.read_bytes() for p in self.out.iterdir()}, snapshot)

    def test_output_collisions_preserve_prior_bytes_and_retain_spent_refusal(self):
        for name in ('stdout.log.txt', 'stderr.log.txt', 'terminal.json'):
            with self.subTest(prior_record=name):
                self.out = self.folder / ('collision-'+name)
                self.out.mkdir(mode=0o700)
                grant = json.loads(self.grant.read_bytes())
                grant['output_directory'] = str(self.out)
                self.grant.write_bytes(self.control.encoded(grant))
                self.frozen[('a'*40, self.grant.relative_to(self.root).as_posix())] = self.grant.read_bytes()
                prior = b'prior evidence, never parse or overwrite\n'
                (self.out/name).write_bytes(prior)
                result = self.run_controller()
                self.assertEqual(result['status'], 'refused')
                self.assertFalse(result['executed'])
                self.assertNotIn('retention_error', result)
                self.assertEqual((self.out/name).read_bytes(), prior)
                self.assertTrue((self.out/'started.json').read_bytes())
                record = 'uncertainty.json' if name == 'terminal.json' else 'terminal.json'
                self.assertEqual(json.loads((self.out/record).read_bytes()), result)
                self.assertEqual(result['operation_id'], 'fixture-operation')
                self.assertEqual(result['command_sha256'], self.control.digest(self.control.encoded(json.loads(self.case.read_bytes())['command'])))
                snapshot = {p.name:p.read_bytes() for p in self.out.iterdir()}
                self.assertFalse(self.run_controller()['executed'])
                self.assertEqual(snapshot, {p.name:p.read_bytes() for p in self.out.iterdir()})
        self.assertEqual(self.calls, [])

    def test_occupied_uncertainty_output_refuses_before_reservation(self):
        prior = {'terminal.json': b'prior-terminal', 'uncertainty.json': b'prior-uncertainty'}
        for name, content in prior.items():
            (self.out/name).write_bytes(content)
        result = self.run_controller()
        self.assertFalse(result['executed'])
        self.assertEqual(result['status'], 'refused')
        self.assertFalse((self.out/'started.json').exists(), 'incompatible prior uncertainty must refuse before reservation')
        self.assertEqual({p.name:p.read_bytes() for p in self.out.iterdir()}, prior)
        self.assertEqual(self.calls, [])

"""Qualification must reject incomplete evidence, even when unittest says OK."""
import io
import json
import os
import signal
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from durable.qualification import run_suite, scope_root


class QualificationTests(unittest.TestCase):
    def test_provenance_failure_cannot_publish_passing_report(self):
        from durable.qualification import main
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'report.json'
            with patch('durable.qualification.preflight', return_value={}), \
                 patch('durable.qualification.run_suite', return_value={'status': 'passed'}), \
                 patch('durable.qualification.source_identity',
                       side_effect=[{}, subprocess.CalledProcessError(128, 'git')]):
                self.assertEqual(main(['--output', str(output)]), 2)
            self.assertNotEqual(json.loads(output.read_text())['status'], 'passed')

    def test_scope_cancellation_stops_exact_unit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fake = root / 'systemctl'
            marker = root / 'stopped'
            ready = root / 'ready'
            fake.write_text('#!' + sys.executable + '\nimport pathlib,sys\n'
                            + f'pathlib.Path({str(marker)!r}).write_text(" ".join(sys.argv[1:]))\n')
            fake.chmod(0o700)
            command = [sys.executable, '-c',
                       f'import pathlib,time; pathlib.Path({str(ready)!r}).touch(); time.sleep(60)']
            code = ('from durable.qualification import run_scope; '
                    f'raise SystemExit(run_scope({command!r}, "owned.scope"))')
            env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ['PATH'])
            proc = subprocess.Popen([sys.executable, '-c', code], env=env,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                deadline = time.monotonic() + 5
                while not ready.exists() and proc.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue(ready.exists())
                proc.send_signal(signal.SIGTERM)
                _, errors = proc.communicate(timeout=10)
                self.assertEqual(proc.returncode, 128 + signal.SIGTERM, errors.decode())
                self.assertEqual(marker.read_text(), '--user stop owned.scope')
            finally:
                if proc.poll() is None:
                    proc.kill()
                proc.communicate(timeout=5)

    def test_scope_root_requires_exact_owned_unit(self):
        unit = 'harp-qualification-' + 'a' * 32 + '.scope'
        self.assertEqual(scope_root(unit, '0::/user.slice/app.slice/' + unit + '\n'),
                         Path('/sys/fs/cgroup/user.slice/app.slice') / unit)
        for actual in ['0::/unrelated.scope', '0::/x/' + unit + '/../other',
                       '1:cpu:/x/' + unit]:
            with self.assertRaises(ValueError):
                scope_root(unit, actual)
        with self.assertRaises(ValueError):
            scope_root('other.scope', '0::/other.scope')

    def test_complete_pass_records_executed_case(self):
        class Passing(unittest.TestCase):
            def runTest(self):
                self.assertEqual(2 + 2, 4)
        report = run_suite(unittest.TestSuite([Passing()]), stream=io.StringIO())
        self.assertEqual(report['status'], 'passed')
        self.assertEqual(report['tests_run'], 1)
        self.assertEqual(len(report['executed']), 1)

    def test_empty_skipped_expected_failure_and_failure_do_not_qualify(self):
        class Skipped(unittest.TestCase):
            def runTest(self):
                self.skipTest('missing capability')
        class ExpectedFailure(unittest.TestCase):
            @unittest.expectedFailure
            def runTest(self):
                self.fail('known failure')
        class Failed(unittest.TestCase):
            def runTest(self):
                self.fail('broken')
        for cases in [[], [Skipped()], [ExpectedFailure()], [Failed()]]:
            with self.subTest(cases=cases):
                report = run_suite(unittest.TestSuite(cases), stream=io.StringIO())
                self.assertEqual(report['status'], 'failed')

    def test_missing_delegation_writes_nonpassing_cli_report(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'qualification.json'
            env = dict(os.environ)
            env.pop('INSULA_TEST_CGROUP_ROOT', None)
            result = subprocess.run([sys.executable, '-m', 'durable.qualification',
                                     '--output', str(output)], env=env, capture_output=True,
                                    text=True, timeout=10)
            self.assertEqual(result.returncode, 2, result.stderr)
            report = json.loads(output.read_text())
            self.assertEqual(report['status'], 'unavailable')
            self.assertIn('INSULA_TEST_CGROUP_ROOT', report['reason'])
            self.assertEqual(report['tests_run'], 0)


if __name__ == '__main__':
    unittest.main()

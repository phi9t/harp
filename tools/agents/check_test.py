"""Behavioral CLI tests against disposable gate registrations (never the full gate)."""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]


class BoundedGateTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="gate-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "tools/agents/checks.d").mkdir(parents=True)
        for name in ('tools/agents/check.py', 'tools/agents/check.sh', 'tools/temp_root.py'):
            shutil.copyfile(ROOT / name, self.root / name)
        self.summary = self.root / "summary.json"
        self.pids = self.root / "pids"
        self.addCleanup(self.cleanup_children)

    def cleanup_children(self):
        if self.pids.exists():
            for pid in self.pids.read_text().split():
                try:
                    os.kill(int(pid), signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def register(self, name, script):
        (self.root / f"tools/agents/checks.d/{name}.sh").write_text(script)

    def command(self, *args):
        return [sys.executable, str(self.root / 'tools/temp_root.py'), '--', '/bin/sh',
                str(self.root / 'tools/agents/check.sh'), '--json', str(self.summary), *args]

    def hanging_suite(self):
        (self.root / 'hang.py').write_text("""import os, signal, time
from pathlib import Path
child = os.fork()
if child == 0:
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    Path('ready').write_text('yes')
    while True: time.sleep(1)
Path('pids').write_text(str(os.getpid()) + ' ' + str(child))
while True: time.sleep(1)
""")
        self.register('hang', f'exec {sys.executable} hang.py')

    def assert_children_stopped(self):
        for pid in self.pids.read_text().split():
            stat = Path('/proc') / pid / 'stat'
            if stat.exists():
                self.assertEqual(stat.read_text().split()[2], 'Z', f'live child: {pid}')

    def test_timeout_kills_term_ignoring_descendant_and_continues(self):
        self.hanging_suite()
        self.register('later', 'exit 0')
        process = subprocess.Popen(self.command('--suite', 'hang', '--suite', 'later', '--timeout', '0.3'),
                                   cwd=self.root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            out, err = process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            self.cleanup_children()
            process.kill()
            process.communicate()
            self.fail('gate did not clean descendants and finish within five seconds')
        self.assertEqual(process.returncode, 1, out + err)
        summary = json.loads(self.summary.read_text())
        self.assertEqual([row['status'] for row in summary['results']], ['timeout', 'passed'])
        self.assert_children_stopped()

    def test_interrupt_preserves_partial_summary_and_kills_children(self):
        for sig in (signal.SIGINT, signal.SIGTERM):
            with self.subTest(signal=sig):
                self.hanging_suite()
                (self.root / 'ready').unlink(missing_ok=True)
                self.register('later', 'exit 0')
                process = subprocess.Popen(self.command('--suite', 'hang', '--suite', 'later'),
                                           cwd=self.root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                try:
                    deadline = time.monotonic() + 3
                    while not (self.root / 'ready').exists() and time.monotonic() < deadline:
                        time.sleep(0.01)
                    self.assertTrue((self.root / 'ready').exists())
                    process.send_signal(sig)
                    out, err = process.communicate(timeout=5)
                    self.assertEqual(process.returncode, 128 + sig, out + err)
                    summary = json.loads(self.summary.read_text())
                    self.assertEqual(summary['status'], 'interrupted')
                    self.assertEqual(summary['results'][0]['status'], 'interrupted')
                    self.assertEqual(summary['remaining_suites'], ['later'])
                    self.assert_children_stopped()
                finally:
                    self.cleanup_children()
                    if process.poll() is None:
                        process.kill()
                    process.communicate()

    def test_list_and_invalid_options_never_execute_suites(self):
        self.register('marker', 'touch executed')
        for args, code in ((['--list'], 0), (['--suite', 'unknown'], 2),
                           (['--timeout', 'nan'], 2), (['--timeout', '0'], 2)):
            with self.subTest(args=args):
                result = subprocess.run(self.command(*args), capture_output=True, text=True)
                self.assertEqual(result.returncode, code, result.stderr)
                self.assertFalse((self.root / 'executed').exists())
                if '--list' in args:
                    self.assertIn('marker', result.stdout)

    def test_failure_and_timing_are_reported(self):
        self.register('failure', 'exit 7')
        self.register('success', 'sleep 0.03')
        result = subprocess.run(self.command('--suite', 'failure', '--suite', 'success'),
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        summary = json.loads(self.summary.read_text())
        self.assertEqual([row['status'] for row in summary['results']], ['failed', 'passed'])
        self.assertEqual(summary['results'][0]['returncode'], 7)
        self.assertGreaterEqual(summary['results'][1]['duration_seconds'], 0.03)
        self.assertGreaterEqual(summary['duration_seconds'], sum(row['duration_seconds'] for row in summary['results']))


if __name__ == '__main__':
    unittest.main()

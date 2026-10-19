"""Reproduce the supplied Rust suite against Harp's actual workspace lock."""
from pathlib import Path
import os
import re
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(sys.platform == 'linux' and os.environ.get('INSULA_OWNER_NATIVE') == '1',
                     'requires explicit native supervisor qualification')
class WorkspaceQualification(unittest.TestCase):
    def test_locked_linux_supervisor_and_transport_suites(self):
        self.assertEqual(sys.platform, 'linux', 'native Linux qualification required')
        result = subprocess.run(['cargo', 'test', '--offline', '--locked', '--profile', 'test-small',
                                 '-p', 'insula-supervisor', '--all-targets', '--', '--test-threads=1'],
                                cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                timeout=1200)
        print(result.stdout, flush=True)
        self.assertEqual(result.returncode, 0, result.stdout[-8192:])
        summaries = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', result.stdout)
        self.assertGreaterEqual(sum(int(row[0]) for row in summaries), 53, 'missing supplied test suites')
        self.assertTrue(all(row[1:] == ('0', '0') for row in summaries), 'required tests were skipped or failed')

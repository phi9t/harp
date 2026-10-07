"""The temp-path gate rejects host paths while retaining capsule paths."""
from pathlib import Path
import subprocess
import tempfile
import unittest

GATE = Path(__file__).resolve().parent / "checks.d/90-no-host-tmp.sh"

class TempGateTest(unittest.TestCase):
    def check(self, source, expected):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / "tools/agents/fixture.py"
            fixture.parent.mkdir(parents=True)
            fixture.write_text(source)
            result = subprocess.run(["sh", str(GATE)], cwd=root, capture_output=True, text=True)
            self.assertEqual(result.returncode, expected, result.stderr)
            return result

    def test_host_tmp_descendants_are_refused(self):
        result = self.check("target = " + repr("/" + "tmp/forbidden-host-path") + "\n", 1)
        self.assertIn("forbidden-host-path", result.stderr)

    def test_private_capsule_mount_descendants_remain_allowed(self):
        self.check("target = " + repr("/" + "tmp/run/tmp/probe") + "\n", 0)

if __name__ == "__main__":
    unittest.main()

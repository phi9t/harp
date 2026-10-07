"""The temp-path gate rejects host and former private sandbox paths."""
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
        self.assertIn("tools/agents/fixture.py:1", result.stderr)

    def test_legacy_private_capsule_mount_descendants_are_refused(self):
        self.check("target = " + repr("/" + "tmp/run/tmp/probe") + "\n", 1)

    def test_historical_display_exception_cannot_allow_operational_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / "tools/agents/test_lead_launch.py"
            fixture.parent.mkdir(parents=True)
            historic = "/" + "tmp/r-worker-demo-01-hello"
            fixture.write_text("display = \"\"\"\n" + historic + "\n\"\"\"\n")
            allowed = subprocess.run(["sh", str(GATE)], cwd=root, capture_output=True, text=True)
            self.assertEqual(allowed.returncode, 0, allowed.stderr)
            fixture.write_text("target = " + repr(historic) + "\n")
            refused = subprocess.run(["sh", str(GATE)], cwd=root, capture_output=True, text=True)
            self.assertEqual(refused.returncode, 1, refused.stderr)

if __name__ == "__main__":
    unittest.main()

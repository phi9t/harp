import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class DeployCtlTests(unittest.TestCase):
    def run_status(self, csv_text: str, output_format: str | None = None) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "jobs.csv"
            path.write_text(csv_text)
            cmd = [sys.executable, "-m", "deployctl", "status", str(path)]
            if output_format is not None:
                cmd.extend(["--format", output_format])
            return subprocess.run(cmd, text=True, capture_output=True, check=False)

    def test_status_json_is_machine_readable(self):
        result = self.run_status(
            "id,state\n1,ok\n2,failed\n3,queued\n4,ok\n5,ignored\n",
            "json",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertEqual(json.loads(result.stdout), {"failed": 1, "ok": 2, "queued": 1})

    def test_status_json_is_not_hard_coded_to_one_fixture(self):
        result = self.run_status(
            "id,state\n10,queued\n11,queued\n12,ok\n13,unknown\n",
            "json",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), {"failed": 0, "ok": 1, "queued": 2})

    def test_status_text_is_the_default_format(self):
        result = self.run_status("id,state\n1,failed\n2,ok\n3,ignored\n")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "ok: 1\nfailed: 1\nqueued: 0\n")


if __name__ == "__main__":
    unittest.main()

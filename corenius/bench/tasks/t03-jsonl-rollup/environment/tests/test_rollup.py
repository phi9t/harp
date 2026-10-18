import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class RollupTests(unittest.TestCase):
    def run_rollup(self, records, service, status):
        tmp_root = ROOT / ".test-tmp"
        tmp_root.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="rollup.", dir=tmp_root) as tmp:
            events = Path(tmp) / "events.jsonl"
            with events.open("w", encoding="utf-8") as handle:
                for record in records:
                    handle.write(json.dumps(record) + "\n")
            result = subprocess.run(
                ["python3", str(ROOT / "rollup.py"), str(events), service, status],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout.strip()

    def test_filters_by_service_and_status(self):
        records = [
            {"service": "api", "status": "ok", "amount": 7},
            {"service": "worker", "status": "ok", "amount": 100},
            {"service": "api", "status": "error", "amount": 50},
            {"service": "api", "status": "ok", "amount": 8},
            {"service": "billing", "status": "ok", "amount": 3},
        ]
        self.assertEqual(self.run_rollup(records, "api", "ok"), "count=2 total=15")

    def test_does_not_count_same_status_from_other_services(self):
        records = [
            {"service": "api", "status": "ok", "amount": 9},
            {"service": "api", "status": "retry", "amount": 100},
            {"service": "queue", "status": "ok", "amount": 8},
            {"service": "api", "status": "ok", "amount": -2},
        ]
        self.assertEqual(self.run_rollup(records, "api", "ok"), "count=2 total=7")


if __name__ == "__main__":
    unittest.main()

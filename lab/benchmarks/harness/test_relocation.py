"""Exercise relocated task materialization and grader/reference boundaries.

HARP_BENCHMARK_NATIVE=1 runs real offline Linux capsules. Missing native
prerequisites fail that explicit qualification instead of silently skipping.
"""
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasklib


@unittest.skipUnless(os.environ.get("HARP_BENCHMARK_NATIVE") == "1", "explicit Linux benchmark qualification required")
class RelocatedReferenceReplay(unittest.TestCase):
    def test_existing_terminal_tasks_grade_red_then_green_in_real_capsules(self):
        self.assertEqual(sys.platform, "linux")
        for executable in ("bwrap", "bash", "tar", "python3"):
            self.assertIsNotNone(shutil.which(executable), executable)
        with tempfile.TemporaryDirectory(prefix="bench-relocation-") as temporary:
            root = Path(temporary)
            previous = os.environ.get("INSULA_CAPSULES")
            os.environ["INSULA_CAPSULES"] = str(root / "capsules")
            try:
                request = root / "request.json"
                request.write_text(json.dumps({"layers": [{"kind": "base", "source": "host-usr"}], "policy": {"net": "none", "gpu": False}}))
                record = tasklib.capsule.build(str(request))
                for name in ("t01-log-window", "t02-quoting-archive", "t03-jsonl-rollup"):
                    with self.subTest(task=name):
                        task = tasklib.load_tasks(only=[name])[0]
                        workspace = root / name
                        tasklib.materialize(task, workspace)
                        before = tasklib.verify(task, workspace, record)
                        self.assertFalse(before["passed"], before)
                        self.assertFalse(before["timed_out"], before)
                        applied = tasklib.apply_solution(task, workspace, record)
                        self.assertEqual(applied["rc"], 0, applied)
                        after = tasklib.verify(task, workspace, record)
                        self.assertTrue(after["passed"], after)
                        repeated = tasklib.verify(task, workspace, record)
                        self.assertTrue(repeated["passed"], repeated)
            finally:
                if previous is None:
                    os.environ.pop("INSULA_CAPSULES", None)
                else:
                    os.environ["INSULA_CAPSULES"] = previous


if __name__ == "__main__":
    unittest.main()

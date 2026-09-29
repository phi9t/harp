"""validate.py must reject broken tasks, not only accept good ones (real bwrap, base-only capsule)."""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasklib  # noqa: E402
import validate  # noqa: E402


class ValidateTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=str(tasklib.ROOT / ".scratch"))
        self.root = Path(self.tmp.name)
        os.environ["CORENIUS_CAPSULES"] = str(self.root / "store")
        req = self.root / "req.json"
        req.write_text(json.dumps({"name": "t", "layers": [{"kind": "base"}], "policy": {"net": "none"}}))
        self.record = tasklib.capsule.build(str(req))

    def tearDown(self):
        os.environ.pop("CORENIUS_CAPSULES", None)
        self.tmp.cleanup()

    def make(self, name="t", verifier="sh tests/test.sh", test="grep -q fixed value.txt", start="broken", fixed="fixed",
             protected=None, flaky=False):
        d = self.root / name
        (d / "environment/tests").mkdir(parents=True)
        (d / "tests").mkdir()
        (d / "solution/overlay").mkdir(parents=True)
        (d / "environment/value.txt").write_text(start + "\n")
        if flaky:
            test = 'if [ -e .seen ]; then exit 1; fi; : > .seen; exit 0'
        for base in (d / "tests", d / "environment/tests"):
            (base / "test.sh").write_text(test + "\n")
        (d / "solution/overlay/value.txt").write_text(fixed + "\n")
        (d / "solution/solve.sh").write_text("cp -r /tmp/solution/overlay/. /workspace/\n")
        prot = "" if protected is None else f"protected = {json.dumps(protected)}\n"
        (d / "task.toml").write_text(f'schema_version = "1.1"\n[task]\nname = "{name}"\n[bench]\nnet = "none"\n'
                                     f'[verifier]\ncommand = "{verifier}"\n{prot}')
        return tasklib.Task(d)

    def checks(self, task):
        return validate.check_task(task, self.record)

    def test_good_task_passes_every_check(self):
        result = self.checks(self.make())
        self.assertEqual([k for k, v in result.items() if not v["ok"]], [], result)

    def test_verifier_that_always_passes_is_rejected(self):
        result = self.checks(self.make(test="exit 0"))
        self.assertFalse(result["start-fails"]["ok"])
        self.assertFalse(result["empty-patch-fails"]["ok"])

    def test_solution_that_does_not_fix_is_rejected(self):
        result = self.checks(self.make(fixed="still broken"))
        self.assertFalse(result["solution-passes"]["ok"])

    def test_unprotected_tests_are_flagged_by_the_cheat_checks(self):
        result = self.checks(self.make(protected=[]))
        self.assertFalse(result["cheat-stub-tests"]["ok"])

    def test_flaky_verifier_is_rejected(self):
        result = self.checks(self.make(flaky=True, fixed="fixed"))
        self.assertFalse(result["deterministic"]["ok"] and result["solution-passes"]["ok"] and result["start-fails"]["ok"])

    def test_restore_discards_agent_edits_to_tests(self):
        task = self.make()
        ws = self.root / "ws"
        tasklib.materialize(task, ws)
        (ws / "tests/test.sh").write_text("exit 0\n")
        (ws / "tests/extra").write_text("x")
        tasklib.restore_protected(task, ws)
        self.assertEqual((ws / "tests/test.sh").read_text(), "grep -q fixed value.txt\n")
        self.assertFalse((ws / "tests/extra").exists())

    def test_protected_source_files_are_restored_from_the_environment(self):
        task = self.make(protected=["tests", "value.txt"])
        # value.txt is part of the fix, so protecting it makes the task unsolvable: the check must flag that
        result = self.checks(task)
        self.assertFalse(result["solution-passes"]["ok"])
        ws = self.root / "ws2"
        tasklib.materialize(task, ws)
        (ws / "value.txt").write_text("hacked\n")
        tasklib.restore_protected(task, ws)
        self.assertEqual((ws / "value.txt").read_text(), "broken\n")

    def test_tamper_check_passes_when_protected_sources_are_untouched_by_the_fix(self):
        task = self.make(test="grep -q fixed value.txt && grep -q keep helper.txt", protected=["tests", "helper.txt"])
        (task.path / "environment/helper.txt").write_text("keep\n")
        result = self.checks(task)
        self.assertTrue(result["tamper-protected-restored"]["ok"], result)

    def test_protected_path_cannot_escape(self):
        task = self.make(protected=["../etc"])
        with self.assertRaises(ValueError):
            tasklib.restore_protected(task, self.root / "ws")


if __name__ == "__main__":
    unittest.main()

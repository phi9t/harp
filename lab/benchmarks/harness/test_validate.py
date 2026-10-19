"""validate.py must reject broken tasks, not only accept good ones (real bwrap, base-only capsule)."""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from io import StringIO
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasklib  # noqa: E402
import validate  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "insula" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "insula" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


class ValidateTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=str(tasklib.ROOT / ".scratch"))
        self.root = Path(self.tmp.name)
        os.environ["INSULA_CAPSULES"] = str(self.root / "store")
        req = self.root / "req.json"
        req.write_text(json.dumps({"name": "t", "layers": [{"kind": "base"}], "policy": {"net": "none"}}))
        self.record = tasklib.capsule.build(str(req))

    def tearDown(self):
        os.environ.pop("INSULA_CAPSULES", None)
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
        (d / "solution/solve.sh").write_text("cp -r /run/insula-tmp/solution/overlay/. /workspace/\n")
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

    def test_materialize_preserves_execute_only_files(self):
        task = self.make()
        exe = task.path / "environment/executable"
        exe.write_text("#!/bin/sh\necho opaque\n")
        exe.chmod(0o111)
        ws = self.root / "ws-execute-only"
        tasklib.materialize(task, ws)
        self.assertEqual(exe.stat().st_mode & 0o777, 0o111)
        self.assertEqual((ws / "executable").stat().st_mode & 0o777, 0o111)

    def test_materialize_preserves_restricted_directories(self):
        task = self.make()
        restricted = task.path / "environment/data/output"
        restricted.mkdir(parents=True)
        (restricted / "kept.txt").write_text("kept\n")
        restricted.chmod(0)
        ws = self.root / "ws-restricted-dir"
        try:
            tasklib.materialize(task, ws)
            self.assertEqual(restricted.stat().st_mode & 0o777, 0)
            self.assertEqual((ws / "data/output").stat().st_mode & 0o777, 0)
        finally:
            restricted.chmod(0o700)
            if (ws / "data/output").exists():
                (ws / "data/output").chmod(0o700)

    def test_apply_solution_receives_agent_env(self):
        task = self.make(test="grep -q env-value value.txt")
        with (task.path / "task.toml").open("a") as handle:
            handle.write('[agent.env]\nEXPECTED_VALUE = "env-value"\n')
        task = tasklib.Task(task.path)
        (task.path / "solution/solve.sh").write_text("printf '%s\\n' \"$EXPECTED_VALUE\" > /workspace/value.txt\n")
        ws = self.root / "ws-agent-env"
        tasklib.materialize(task, ws)
        result = tasklib.apply_solution(task, ws, self.record)
        self.assertEqual(result["rc"], 0, result)
        self.assertEqual((ws / "value.txt").read_text(), "env-value\n")

    def test_apply_solution_honors_bash_solutions_when_available(self):
        task = self.make(test="grep -q bash value.txt")
        (task.path / "solution/solve.sh").write_text(
            "#!/bin/bash\nset -euo pipefail\nprintf 'bash\\n' > /workspace/value.txt\n"
        )
        ws = self.root / "ws-bash-solution"
        tasklib.materialize(task, ws)
        result = tasklib.apply_solution(task, ws, self.record)
        self.assertEqual(result["rc"], 0, result)
        self.assertEqual((ws / "value.txt").read_text(), "bash\n")

    def test_run_in_capsule_starts_declared_service(self):
        ws = self.root / "ws-service"
        (ws / "site").mkdir(parents=True)
        (ws / "site/index.html").write_text("ready\n")
        result = tasklib.run_in_capsule(
            self.record,
            ws,
            "python3 -c \"import urllib.request; print(urllib.request.urlopen('http://server:8123', timeout=2).read().decode().strip())\"",
            {},
            "none",
            30,
            services=[
                {
                    "name": "server",
                    "command": "python3 -m http.server 8123",
                    "cwd": "/workspace/site",
                    "ready_command": "python3 -c \"import urllib.request; urllib.request.urlopen('http://server:8123', timeout=1).read(1)\"",
                    "aliases": ["server"],
                    "startup_timeout_sec": 5,
                }
            ],
        )
        self.assertEqual(result["rc"], 0, result)
        self.assertIn("ready", result["tail"])

    def test_run_in_capsule_runs_same_capsule_service_stop_command(self):
        ws = self.root / "ws-service-stop"
        ws.mkdir()
        result = tasklib.run_in_capsule(
            self.record,
            ws,
            "exec true",
            {},
            "none",
            30,
            services=[
                {
                    "name": "server",
                    "command": "while :; do sleep 1; done",
                    "cwd": "/workspace",
                    "ready_command": "true",
                    "stop_command": "printf stopped > /workspace/stopped.txt",
                    "aliases": [],
                    "startup_timeout_sec": 5,
                }
            ],
        )
        self.assertEqual(result["rc"], 0, result)
        self.assertEqual((ws / "stopped.txt").read_text(), "stopped")

    def test_run_in_capsule_starts_isolated_service_capsule(self):
        req = self.root / "service_req.json"
        req.write_text(json.dumps({"name": "svc", "layers": [{"kind": "base"}], "policy": {"net": "none"}}))
        ws = self.root / "ws-isolated-service"
        (ws / "site").mkdir(parents=True)
        (ws / "site/index.html").write_text("isolated-ready\n")
        result = tasklib.run_in_capsule(
            self.record,
            ws,
            "python3 -c \"import urllib.request; print(urllib.request.urlopen('http://isolated:18123', timeout=2).read().decode().strip())\"",
            {},
            "host",
            30,
            services=[
                {
                    "name": "isolated",
                    "command": "python3 -m http.server 18123 --bind 127.0.0.1",
                    "cwd": "/workspace/site",
                    "ready_command": "python3 -c \"import urllib.request; urllib.request.urlopen('http://isolated:18123', timeout=1).read(1)\"",
                    "aliases": ["isolated"],
                    "startup_timeout_sec": 5,
                    "phases": ["agent"],
                    "capsule_request": str(req),
                    "net": "host",
                    "env": {},
                    "ro_workspace": True,
                }
            ],
        )
        self.assertEqual(result["rc"], 0, result)
        self.assertIn("isolated-ready", result["tail"])

    def test_isolated_service_ready_output_is_captured(self):
        req = self.root / "service_req.json"
        req.write_text(json.dumps({"name": "svc", "layers": [{"kind": "base"}], "policy": {"net": "none"}}))
        ws = self.root / "ws-isolated-ready-output"
        ws.mkdir()
        stdout = StringIO()
        stderr = StringIO()
        with redirect_stdout(stdout), redirect_stderr(stderr):
            result = tasklib.run_in_capsule(
                self.record,
                ws,
                "true",
                {},
                "none",
                30,
                services=[
                    {
                        "name": "isolated",
                        "command": "sleep 10",
                        "cwd": "/workspace",
                        "ready_command": "printf READY_STDOUT; printf READY_STDERR >&2; exit 1",
                        "aliases": ["isolated"],
                        "startup_timeout_sec": 1,
                        "phases": ["agent"],
                        "capsule_request": str(req),
                        "net": "none",
                        "env": {},
                        "ro_workspace": True,
                    }
                ],
            )
        self.assertEqual(result["rc"], 1, result)
        self.assertIn("service isolated did not become ready", result["tail"])
        self.assertIn("READY_STDOUT", result["tail"])
        self.assertIn("READY_STDERR", result["tail"])
        self.assertNotIn("READY_STDOUT", stdout.getvalue())
        self.assertNotIn("READY_STDERR", stderr.getvalue())

    def test_service_phases_default_and_validate(self):
        task = self.make()
        with (task.path / "task.toml").open("a") as handle:
            handle.write(
                "[bench.services.good]\n"
                "command = \"true\"\n"
                "phases = [\"verifier\"]\n"
                "[bench.services.default]\n"
                "command = \"true\"\n"
                "stop_command = \"printf stopped > /workspace/stopped\"\n"
                "[bench.services.isolated]\n"
                "command = \"true\"\n"
                "capsule_request = \"service_capsule.json\"\n"
                "net = \"host\"\n"
                "ro_workspace = true\n"
                "[bench.services.isolated.env]\n"
                "EXAMPLE = \"1\"\n"
            )
        loaded = tasklib.Task(task.path)
        phases = {service["name"]: service["phases"] for service in loaded.services}
        self.assertEqual(phases["good"], ["verifier"])
        self.assertEqual(phases["default"], ["agent", "verifier"])
        default = next(service for service in loaded.services if service["name"] == "default")
        self.assertEqual(default["stop_command"], "printf stopped > /workspace/stopped")
        isolated = next(service for service in loaded.services if service["name"] == "isolated")
        self.assertEqual(isolated["capsule_request"], str((task.path / "service_capsule.json").resolve()))
        self.assertEqual(isolated["net"], "host")
        self.assertTrue(isolated["ro_workspace"])
        self.assertEqual(isolated["env"], {"EXAMPLE": "1"})
        with (task.path / "task.toml").open("a") as handle:
            handle.write("[bench.services.bad]\ncommand = \"true\"\nphases = [\"setup\"]\n")
        with self.assertRaises(ValueError):
            tasklib.Task(task.path)

    def test_agent_net_default_and_validate(self):
        task = self.make()
        with (task.path / "task.toml").open("a") as handle:
            handle.write('[agent]\nnet = "host"\n')
        self.assertEqual(tasklib.Task(task.path).agent["net"], "host")
        with (task.path / "task.toml").open("a") as handle:
            handle.write('[agent_bad]\n')
        (task.path / "task.toml").write_text((task.path / "task.toml").read_text().replace('net = "host"', 'net = "bridge"'))
        with self.assertRaises(ValueError):
            tasklib.Task(task.path)

    def test_service_net_is_validated(self):
        task = self.make()
        with (task.path / "task.toml").open("a") as handle:
            handle.write('[bench.services.badnet]\ncommand = "true"\nnet = "bridge"\n')
        with self.assertRaises(ValueError):
            tasklib.Task(task.path)

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

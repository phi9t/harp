from __future__ import annotations
import copy
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from controller.durable import Activity, Context, Runtime, Store
from controller.durable.common import atomic_json, digest, read_json
from controller.durable.engine import CleanupUnproven, ControllerBusy
from controller.durable.workflows import DEMO_CONTRACT, DEMO_VERSION, REGISTRY, command
from tools.temp_root import temporary_root
ROOT = Path(__file__).resolve().parents[2]


def registered_cli_workflow(ctx, args):
    return {"received": args}


# This importable test registry proves the CLI is independent of demo versions.
REGISTRY = {**REGISTRY, "registered-test": ("registered-v2", registered_cli_workflow)}


def wait_for(fn, timeout=15):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        value = fn()
        if value:
            return value
        time.sleep(0.02)
    raise AssertionError("condition did not become true")


def alive(pid):
    try:
        return Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[0] != "Z"
    except FileNotFoundError:
        return False


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=temporary_root())
        self.root = Path(self.tmp.name)
        self.store = Store(self.root / "state", allow_unpatched_sqlite=True)
        self.children = []

    def tearDown(self):
        for p in self.children:
            if p.poll() is None:
                p.kill()
            p.wait(timeout=5)
        self.store.close()
        self.tmp.cleanup()

    def submit(self, args=None, workflow="one", run="r"):
        self.store.submit(run, workflow, DEMO_VERSION, args or {}, DEMO_CONTRACT)

    def runtime(self, **kwargs):
        return Runtime(self.store, REGISTRY, DEMO_CONTRACT, **kwargs)

    def drive(self, **kwargs):
        with self.runtime(**kwargs) as rt:
            rt.drive(timeout=25)
        return self.store.status("r")

    def cli(self, *args, wait=True):
        p = subprocess.Popen([sys.executable, "-m", "controller.durable", "--state", str(self.store.root), "--allow-unpatched-sqlite", *args],
                             cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.children.append(p)
        if wait:
            p.wait(timeout=25)
        return p

    def started(self):
        files = list((self.store.root / "attempts").glob("*/started.json"))
        return files[0] if files else None

    def test_pipeline_fanout_join(self):
        self.submit({"value": 3}, "pipeline")
        s = self.drive(max_active=2)
        self.assertEqual(s["status"], "SUCCEEDED")
        self.assertEqual(s["result"]["sum"], 181)
        self.assertEqual(len(s["attempts"]), 4)

    def test_submission_dedup_and_conflict(self):
        self.submit({"x": 1})
        self.submit({"x": 1})
        with self.assertRaises(ValueError):
            self.submit({"x": 2})
        self.assertEqual(self.store.db.execute("SELECT count(*) FROM runs").fetchone()[0], 1)

    def test_cli_loads_registry_and_submits_its_workflow_version(self):
        prefix = ("--registry", "controller.durable.runtime_test")
        process = self.cli(*prefix, "submit", "r", "--workflow", "registered-test",
                           "--args", '{"value": 23}')
        self.assertEqual(process.returncode, 0)
        self.assertEqual(self.store.status("r")["version"], "registered-v2")
        self.assertEqual(self.cli(*prefix, "run", "--timeout", "10").returncode, 0)
        self.assertEqual(self.store.status("r")["result"], {"received": {"value": 23}})

    def test_completed_activity_is_not_reexecuted(self):
        effect = self.root / "effects"
        self.submit({"input": {"value": 7, "effect_file": str(effect)}})
        self.assertEqual(self.drive()["result"]["square"], 49)
        self.drive()
        self.assertEqual(len(effect.read_text().splitlines()), 1)

    def test_retry_count_and_stable_idempotency_key(self):
        effect = self.root / "effects"
        self.submit({"input": {"fail_attempts": 1, "effect_file": str(effect)}})
        s = self.drive()
        self.assertEqual(s["status"], "SUCCEEDED")
        self.assertEqual(len(s["attempts"]), 2)
        lines = [json.loads(s) for s in effect.read_text().splitlines()]
        self.assertEqual(lines[0]["key"], lines[1]["key"])

    def test_retry_exhaustion(self):
        self.submit({"input": {"fail_attempts": 20}, "max_attempts": 2})
        s = self.drive()
        self.assertEqual(s["status"], "FAILED")
        self.assertEqual(len(s["attempts"]), 2)

    def test_attempt_timeout(self):
        self.submit({"input": {"sleep": 10}, "timeout": 0.05, "max_attempts": 1})
        s = self.drive()
        self.assertEqual(s["status"], "FAILED")
        self.assertIn("timeout", s["error"])

    def test_manual_effect_blocks_and_can_adopt(self):
        effect = self.root / "effects"
        self.submit({"input": {"fail_attempts": 20, "effect_file": str(effect)}, "recovery": "manual"})
        self.assertEqual(self.drive()["status"], "BLOCKED")
        self.assertEqual(self.drive()["status"], "BLOCKED")
        self.assertEqual(len(effect.read_text().splitlines()), 1)
        self.store.resolve("r", "work", "succeed", "external effect independently verified", {"adopted": True})
        self.assertEqual(self.drive()["result"], {"adopted": True})
        self.assertEqual(len(effect.read_text().splitlines()), 1)

    def test_manual_retry_requires_explicit_resolution(self):
        self.submit({"input": {"fail_attempts": 1}, "recovery": "manual"})
        self.assertEqual(self.drive()["status"], "BLOCKED")
        self.store.resolve("r", "work", "retry", "effect inspected; retry safe")
        self.assertEqual(self.drive()["status"], "SUCCEEDED")

    def test_signal_delivery_and_duplicate_conflict(self):
        def wf(ctx, _args):
            return ctx.wait_signal("approve")
        reg = {"wait": ("v1", wf)}
        self.store.submit("r", "wait", "v1", {}, DEMO_CONTRACT)
        with Runtime(self.store, reg, DEMO_CONTRACT) as rt:
            self.assertEqual(rt.drive()["r"], "RUNNING")
        self.store.signal("r", "approve", {"yes": True})
        self.store.signal("r", "approve", {"yes": True})
        with self.assertRaises(ValueError):
            self.store.signal("r", "approve", {"yes": False})
        with Runtime(self.store, reg, DEMO_CONTRACT) as rt:
            rt.drive()
        self.assertEqual(self.store.status("r")["result"], {"yes": True})

    def test_timer_deadline_survives_restart(self):
        def wf(ctx, _args):
            ctx.sleep("wait", 0.15)
            return 42
        reg = {"timer": ("v1", wf)}
        self.store.submit("r", "timer", "v1", {}, DEMO_CONTRACT)
        with Runtime(self.store, reg, DEMO_CONTRACT) as rt:
            rt.tick()
        due = self.store.status("r")["operations"][0]["due"]
        time.sleep(0.2)
        with Runtime(self.store, reg, DEMO_CONTRACT) as rt:
            rt.drive()
        s = self.store.status("r")
        self.assertEqual(s["operations"][0]["due"], due)
        self.assertEqual(s["result"], 42)

    def test_replay_argument_mismatch_fails(self):
        def before(ctx, _args):
            ctx.sleep("wait", 60)
        def after(ctx, _args):
            ctx.sleep("wait", 61)
        self.store.submit("r", "timer", "v1", {}, DEMO_CONTRACT)
        with Runtime(self.store, {"timer": ("v1", before)}, DEMO_CONTRACT) as rt:
            rt.tick()
        with Runtime(self.store, {"timer": ("v1", after)}, DEMO_CONTRACT) as rt:
            rt.drive()
        s = self.store.status("r")
        self.assertEqual(s["status"], "FAILED")
        self.assertIn("ReplayMismatch", s["error"])

    def test_replay_truncated_tail_fails(self):
        def before(ctx, _args):
            ctx.sleep("first", 0)
            ctx.sleep("second", 60)
        def after(ctx, _args):
            ctx.sleep("first", 0)
            return "wrong"
        self.store.submit("r", "timer", "v1", {}, DEMO_CONTRACT)
        with Runtime(self.store, {"timer": ("v1", before)}, DEMO_CONTRACT) as rt:
            rt.tick()
        with Runtime(self.store, {"timer": ("v1", after)}, DEMO_CONTRACT) as rt:
            rt.drive()
        self.assertIn("recorded operation tail", self.store.status("r")["error"])

    def test_contract_mismatch_refuses_start(self):
        self.submit()
        with self.assertRaisesRegex(RuntimeError, "contract mismatch"):
            with Runtime(self.store, REGISTRY, "wrong"):
                pass
        self.assertEqual(len(self.store.status("r")["attempts"]), 0)

    def test_version_mismatch_refuses_start(self):
        self.submit()
        reg = {"one": ("v2", REGISTRY["one"][1])}
        with self.assertRaisesRegex(RuntimeError, "version unavailable"):
            with Runtime(self.store, reg, DEMO_CONTRACT):
                pass

    def test_second_controller_rejected(self):
        other = Store(self.store.root, allow_unpatched_sqlite=True)
        try:
            with self.runtime():
                with self.assertRaises(ControllerBusy):
                    with Runtime(other, REGISTRY, DEMO_CONTRACT, lock_timeout=0):
                        pass
        finally:
            other.close()

    def test_parallel_capacity_and_gpu_assignment(self):
        # Synthetic identifiers: scheduler accounting, NOT real CUDA execution.
        self.submit({"items": [{"sleep": 0.1, "value": i} for i in range(4)], "gpus": 1}, "batch")
        peak = 0
        with self.runtime(max_active=2, cpu_slots=2, gpu_devices=("test-gpu-a", "test-gpu-b")) as rt:
            end = time.monotonic() + 25
            while self.store.status("r")["status"] == "RUNNING":
                rt.tick()
                peak = max(peak, len(rt.live))
                devices = [d for job in rt.live.values() for d in job.devices]
                self.assertEqual(len(devices), len(set(devices)))
                self.assertLessEqual(len(rt.live), 2)
                self.assertLess(time.monotonic(), end)
                time.sleep(0.01)
        self.assertEqual(peak, 2)
        self.assertEqual(self.store.status("r")["status"], "SUCCEEDED")

    def test_unsatisfiable_capacity_fails(self):
        self.submit({"gpus": 1})
        s = self.drive(gpu_devices=())
        self.assertEqual(s["status"], "FAILED")
        self.assertEqual(len(s["attempts"]), 0)

    def test_cancellation_kills_child_group(self):
        child_file = self.root / "child.json"
        self.submit({"input": {"sleep": 10, "child_pid_file": str(child_file)}})
        with self.runtime() as rt:
            wait_for(lambda: (rt.tick(), child_file.exists())[1])
            pid = read_json(child_file)["pid"]
            self.assertTrue(alive(pid))
            self.store.cancel("r")
            rt.drive(timeout=10)
        wait_for(lambda: not alive(pid))
        self.assertEqual(self.store.status("r")["status"], "CANCELLED")

    def test_cancelled_started_manual_effect_requires_resolution(self):
        effect = self.root / "manual-effects"
        self.submit({"input": {"sleep": 10, "effect_file": str(effect)},
                     "recovery": "manual"})
        with self.runtime() as rt:
            wait_for(lambda: (rt.tick(), effect.exists())[1])
            self.store.cancel("r")
            rt.drive(timeout=10)
        state = self.store.status("r")
        self.assertEqual(state["status"], "BLOCKED")
        self.assertEqual(state["operations"][0]["status"], "UNCERTAIN")
        self.assertEqual(self.drive()["status"], "BLOCKED")
        self.assertEqual(len(effect.read_text().splitlines()), 1)
        self.store.resolve("r", "work", "succeed", "effect independently reconciled", {"adopted": True})
        self.assertEqual(self.drive()["status"], "CANCELLED")
        self.assertEqual(len(effect.read_text().splitlines()), 1)

    def test_failed_sibling_started_manual_effect_requires_resolution(self):
        effect = self.root / "manual-effects"
        def workflow(ctx, _args):
            return ctx.parallel([
                ("failure", command({"sleep": 0.2, "fail_attempts": 1},
                                    recovery="retry_safe", max_attempts=1)),
                ("manual", command({"sleep": 10, "effect_file": str(effect)},
                                   recovery="manual")),
            ])
        registry = {"siblings": ("v1", workflow)}
        self.store.submit("r", "siblings", "v1", {}, DEMO_CONTRACT)
        with Runtime(self.store, registry, DEMO_CONTRACT) as rt:
            rt.drive(timeout=10)
        state = self.store.status("r")
        self.assertEqual(state["status"], "BLOCKED")
        self.assertEqual(state["operations"][1]["status"], "UNCERTAIN")
        with Runtime(self.store, registry, DEMO_CONTRACT) as rt:
            self.assertEqual(rt.drive(timeout=10)["r"], "BLOCKED")
        self.assertEqual(len(effect.read_text().splitlines()), 1)
        self.store.resolve("r", "manual", "succeed", "effect independently reconciled", {})
        with Runtime(self.store, registry, DEMO_CONTRACT) as rt:
            self.assertEqual(rt.drive(timeout=10)["r"], "FAILED")
        self.assertEqual(len(effect.read_text().splitlines()), 1)

    def test_resolving_one_effect_preserves_other_uncertainty(self):
        effects = [self.root / f"effects-{i}" for i in range(2)]
        def workflow(ctx, _args):
            return ctx.parallel([
                (f"manual/{i}", command({"sleep": 10, "effect_file": str(effect)},
                                        recovery="manual"))
                for i, effect in enumerate(effects)
            ])
        registry = {"manuals": ("v1", workflow)}
        self.store.submit("r", "manuals", "v1", {}, DEMO_CONTRACT)
        with Runtime(self.store, registry, DEMO_CONTRACT) as rt:
            wait_for(lambda: (rt.tick(), all(p.exists() for p in effects))[1])
            self.store.cancel("r")
            rt.drive(timeout=10)
        self.store.resolve("r", "manual/0", "succeed", "first effect reconciled", {})
        self.assertEqual(self.store.status("r")["status"], "BLOCKED")
        with Runtime(self.store, registry, DEMO_CONTRACT) as rt:
            self.assertEqual(rt.drive(timeout=10)["r"], "BLOCKED")
        self.store.resolve("r", "manual/1", "succeed", "second effect reconciled", {})
        with Runtime(self.store, registry, DEMO_CONTRACT) as rt:
            self.assertEqual(rt.drive(timeout=10)["r"], "CANCELLED")
        self.assertTrue(all(len(p.read_text().splitlines()) == 1 for p in effects))

    def test_sigkill_controller_cleans_and_retries(self):
        effect = self.root / "effects"
        self.submit({"input": {"sleep": 0.5, "effect_file": str(effect)}})
        p = self.cli("run", wait=False)
        first = wait_for(self.started)
        pid = read_json(first)["pid"]
        wait_for(lambda: effect.exists() and len(effect.read_text().splitlines()) == 1)
        p.kill()
        p.wait(timeout=5)
        s = self.drive()
        self.assertEqual(s["status"], "SUCCEEDED")
        self.assertEqual(len(s["attempts"]), 2)
        self.assertFalse(alive(pid))
        lines = [json.loads(s) for s in effect.read_text().splitlines()]
        self.assertEqual(len(lines), 2)
        self.assertEqual(lines[0]["key"], lines[1]["key"])

    def test_guard_retains_lock_until_cleanup(self):
        self.submit({"input": {"sleep": 5}})
        p = self.cli("run", wait=False)
        started = wait_for(self.started)
        aid = started.parent.name
        def guardian():
            for path in Path("/proc").glob("[0-9]*/cmdline"):
                try:
                    text = path.read_bytes()
                    if b"controller.durable.guard" in text and aid.encode() in text:
                        return int(path.parent.name)
                except (FileNotFoundError, ProcessLookupError):
                    pass
            return None
        guard_pid = wait_for(guardian)
        os.kill(guard_pid, signal.SIGSTOP)
        try:
            p.kill()
            p.wait(timeout=5)
            with self.assertRaises(ControllerBusy):
                with self.runtime(lock_timeout=0):
                    pass
        finally:
            os.kill(guard_pid, signal.SIGCONT)
        self.assertEqual(self.drive()["status"], "SUCCEEDED")

    def test_stale_receipt_cannot_overwrite_result(self):
        self.submit()
        self.drive()
        s = self.store.status("r")
        att = s["attempts"][0]
        forged = copy.deepcopy(read_json(Path(att["directory"]) / "receipt.json"))
        forged["result"] = {"wrong": True}
        forged["result_digest"] = digest(forged["result"])
        with self.runtime() as rt:
            self.assertFalse(rt._accept(att["id"], forged))
        self.assertEqual(self.store.status("r")["result"]["square"], 1)

    def test_missing_receipt_blocks_until_cleanup_and_effect_resolution(self):
        self.submit()
        op = Context(self.store, "r")._op("work", "ACTIVITY", command({}, timeout=10, recovery="retry_safe", backoff=0.02).spec())
        directory = self.store.root / "attempts" / "lost-attempt"
        directory.mkdir(parents=True)
        with self.store.transaction() as db:
            db.execute("INSERT INTO attempts(id,operation_id,number,epoch,spec_digest,directory,released,created) VALUES(?,?,?,?,?,?,?,?)",
                       ("lost-attempt", op["id"], 1, 1, op["spec_digest"], str(directory), 1, time.time()))
            db.execute("UPDATE operations SET status='RUNNING',active_attempt='lost-attempt',attempt_count=1 WHERE id=?", (op["id"],))
        with self.assertRaises(CleanupUnproven):
            with self.runtime():
                pass
        with self.assertRaises(CleanupUnproven):
            self.store.resolve("r", "work", "retry", "cleanup still unknown")
        self.store.acknowledge_cleanup("lost-attempt", "test has no actual child processes")
        self.assertEqual(self.store.status("r")["status"], "BLOCKED")
        self.store.resolve("r", "work", "retry", "input operation known safe to retry")
        self.assertEqual(self.drive()["status"], "SUCCEEDED")

    def test_sqlite_durability_settings(self):
        self.assertEqual(self.store.db.execute("PRAGMA journal_mode").fetchone()[0], "wal")
        self.assertEqual(self.store.db.execute("PRAGMA synchronous").fetchone()[0], 2)
        self.assertEqual(self.store.db.execute("PRAGMA foreign_keys").fetchone()[0], 1)
        self.assertEqual(self.store.db.execute("PRAGMA integrity_check").fetchone()[0], "ok")

    def test_invalid_result_is_not_accepted(self):
        self.submit({"input": {"invalid_result": True}, "max_attempts": 1})
        s = self.drive()
        self.assertEqual(s["status"], "FAILED")
        self.assertIsNone(s["result"])

    def test_invalid_receipt_types_block_recovery(self):
        mutations = [
            lambda receipt: None,
            lambda receipt: [],
            lambda receipt: {**receipt, "status": []},
            lambda receipt: {**receipt, "status": "FAILED", "reason": {}},
            lambda receipt: {**receipt, "protocol": True},
            lambda receipt: {key: value for key, value in
                             {**receipt, "result_digest": digest(None)}.items()
                             if key != "result"},
        ]
        for i, mutate in enumerate(mutations):
            with self.subTest(receipt=i):
                store = Store(self.root / f"invalid-{i}", allow_unpatched_sqlite=True)
                try:
                    store.submit("r", "one", DEMO_VERSION, {}, DEMO_CONTRACT)
                    process = subprocess.Popen([
                        sys.executable, "-m", "controller.durable", "--state", str(store.root),
                        "--allow-unpatched-sqlite", "run", "--test-crash-at", "after_receipt",
                    ], cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    self.children.append(process)
                    self.assertEqual(process.wait(timeout=10), 86)
                    attempt = store.status("r")["attempts"][0]
                    path = Path(attempt["directory"]) / "receipt.json"
                    atomic_json(path, mutate(read_json(path)))
                    with self.assertRaises(CleanupUnproven):
                        with Runtime(store, REGISTRY, DEMO_CONTRACT):
                            pass
                    state = store.status("r")
                    self.assertEqual(state["status"], "BLOCKED")
                    self.assertEqual(state["operations"][0]["status"], "UNCERTAIN")
                    store.acknowledge_cleanup(attempt["id"], "completed child independently verified")
                    self.assertEqual(store.status("r")["status"], "BLOCKED")
                    store.resolve("r", "work", "fail", "effect independently reconciled")
                    with Runtime(store, REGISTRY, DEMO_CONTRACT) as rt:
                        self.assertEqual(rt.drive(timeout=10)["r"], "FAILED")
                finally:
                    store.close()

    def test_relative_executable_rejected(self):
        with self.assertRaises(ValueError):
            Activity(argv=("python", "something.py")).spec()

    def test_activity_temporary_environment_uses_approved_host_root(self):
        keys = ("TMPDIR", "TMP", "TEMP", "TMUX_TMPDIR", "TMPPREFIX")
        root = self.root / "temporary"
        mux = self.root / "mux"
        prefix = self.root / "prefix-"
        code = (
            "import json, os, sys; from controller.durable.common import publish_result; "
            "request=json.load(sys.stdin); "
            f"publish_result(request, {{key: os.environ.get(key) for key in {keys!r}}})"
        )
        def workflow(ctx, _args):
            return ctx.activity("environment", Activity(
                argv=(sys.executable, "-c", code), cwd=str(ROOT)))
        registry = {"environment": ("v1", workflow)}
        self.store.submit("r", "environment", "v1", {}, DEMO_CONTRACT)
        with patch.dict(os.environ, {"TMPDIR": str(root), "TMP": str(root), "TEMP": str(root),
                                     "TMUX_TMPDIR": str(mux), "TMPPREFIX": str(prefix)}):
            with Runtime(self.store, registry, DEMO_CONTRACT) as rt:
                rt.drive(timeout=10)
        self.assertEqual(self.store.status("r")["result"], {
            "TMPDIR": str(root), "TMP": str(root), "TEMP": str(root),
            "TMUX_TMPDIR": str(mux), "TMPPREFIX": str(prefix),
        })

    def test_activity_rejects_managed_temporary_overrides(self):
        for key in ("TMPDIR", "TMP", "TEMP", "TMUX_TMPDIR", "TMPPREFIX"):
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, "managed temporary"):
                command({}, env={key: str(self.root)}).spec()

    def test_activity_spec_does_not_depend_on_host_temporary_root(self):
        activity = command({})
        with patch.dict(os.environ, {"TMPDIR": str(self.root / "first")}):
            first = activity.spec()
        with patch.dict(os.environ, {"TMPDIR": str(self.root / "second")}):
            self.assertEqual(activity.spec(), first)

    def test_nonfinite_control_data_rejected(self):
        with self.assertRaises(ValueError):
            self.submit({"bad": float("nan")})
        with self.assertRaises(ValueError):
            command({}, timeout=float("inf")).spec()


    def test_sqlite_version_gate(self):
        from unittest.mock import patch
        with patch("sqlite3.sqlite_version_info", (3, 46, 1)):
            with self.assertRaisesRegex(RuntimeError, "WAL-reset fix not verified"):
                Store(self.root / "version-test")

    def test_stale_attempt_cannot_commit_over_new_attempt(self):
        self.submit({"input": {"fail_attempts": 1, "sleep": 0.08}})
        with self.runtime() as rt:
            def second_live():
                rt.tick()
                s = self.store.status("r")
                return s if len(s["attempts"]) == 2 and s["operations"][0]["status"] == "RUNNING" else None
            s = wait_for(second_live)
            old = s["attempts"][0]
            active = s["operations"][0]["active_attempt"]
            forged = read_json(Path(old["directory"]) / "receipt.json")
            forged.update(status="SUCCEEDED", result={"stale": True}, result_digest=digest({"stale": True}))
            self.assertFalse(rt._accept(old["id"], forged))
            self.assertEqual(self.store.status("r")["operations"][0]["active_attempt"], active)
            rt.drive(timeout=15)
        self.assertNotIn("stale", self.store.status("r")["result"])

    def test_guard_sigkill_blocks_instead_of_blind_retry(self):
        self.submit({"input": {"sleep": 10}})
        pgid = None
        try:
            with self.runtime() as rt:
                wait_for(lambda: (rt.tick(), self.started())[1])
                job = next(iter(rt.live.values()))
                att = self.store.status("r")["attempts"][0]
                pgid = wait_for(lambda: read_json(Path(att["directory"]) / "process.json")["pgid"])
                job.process.kill()
                job.process.wait(timeout=5)
                with self.assertRaises(CleanupUnproven):
                    rt.drive(timeout=10)
            self.assertEqual(self.store.status("r")["status"], "BLOCKED")
            self.assertEqual(len(self.store.status("r")["attempts"]), 1)
            with self.assertRaises(CleanupUnproven):
                with self.runtime():
                    pass
        finally:
            if pgid is not None:
                try:
                    os.killpg(pgid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                wait_for(lambda: not alive(pgid))


def crash_test(point):
    def test(self):
        effect = self.root / "effects"
        self.submit({"input": {"value": 4, "effect_file": str(effect)}})
        p = self.cli("run", "--test-crash-at", point)
        self.assertEqual(p.returncode, 86)
        self.assertTrue((self.store.root / "test-crash.json").exists())
        s = self.drive()
        self.assertEqual(s["status"], "SUCCEEDED")
        self.assertEqual(s["result"]["square"], 16)
        if point in {"after_claim", "after_guard_spawn", "before_release", "after_receipt", "after_commit"}:
            self.assertEqual(len(effect.read_text().splitlines()), 1)
        if point in {"after_receipt", "after_commit"}:
            self.assertEqual(len(s["attempts"]), 1)
        self.assertEqual(self.store.db.execute("PRAGMA integrity_check").fetchone()[0], "ok")
    return test


for _point in ["after_claim", "after_guard_spawn", "before_release", "after_release", "after_receipt", "after_commit"]:
    setattr(RuntimeTests, f"test_crash_{_point}", crash_test(_point))

if __name__ == "__main__":
    unittest.main()

"""Admission and real Insula activity boundary checks."""
import json
import os
import py_compile
from contextlib import closing
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

from insula import capsule
from durable.execution import InsulaExecution, snapshot_code
from durable.runtime import Runtime, Store
from durable.runtime.demo_workflows import REGISTRY, DEMO_VERSION


class AdmissionTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.previous_store = os.environ.get("INSULA_CAPSULES")
        os.environ["INSULA_CAPSULES"] = str(self.root / "store")
        self.code = snapshot_code(self.root / "source")
        request = self.root / "capsule.json"
        request.write_text(json.dumps({"layers": [{"kind": "base", "source": "host-usr"}]}))
        self.record = capsule.build(str(request))

    def tearDown(self):
        if self.previous_store is None:
            os.environ.pop("INSULA_CAPSULES", None)
        else:
            os.environ["INSULA_CAPSULES"] = self.previous_store
        self.tmp.cleanup()

    def execution(self, **kwargs):
        return InsulaExecution.create(self.record, self.code, self.root / "state", **kwargs)

    def test_contract_binds_backend_code_and_state(self):
        first = self.execution()
        self.assertEqual(first.contract, self.execution().contract)
        self.assertNotEqual(first.contract, InsulaExecution.create(self.record, self.code, self.root / "other").contract)
        (self.code / "extra.py").write_text("x=1\n")
        self.assertNotEqual(first.contract, self.execution().contract)
        with self.assertRaisesRegex(ValueError, "code.*changed"):
            first.verify()
        with self.assertRaisesRegex(ValueError, "code.*changed"):
            first.verify(require_backend=False)

    def test_configuration_tampering_refused(self):
        config = self.execution().config
        config["backend"] = "cgroup"
        with self.assertRaises(ValueError):
            InsulaExecution(config).verify()

    def test_bytecode_cannot_escape_source_binding(self):
        workflow = self.root / "workflow"
        workflow.mkdir()
        source = workflow / "registry.py"
        source.write_text("REGISTRY={}\n")
        bytecode = workflow / "registry.pyc"
        py_compile.compile(str(source), cfile=str(bytecode), doraise=True)
        with self.assertRaisesRegex(ValueError, "bytecode"):
            snapshot_code(self.root / "with-workflow", workflow)
        execution = self.execution()
        py_compile.compile(str(source), cfile=str(self.code / "registry.pyc"), doraise=True)
        with self.assertRaisesRegex(ValueError, "bytecode"):
            execution.verify()

    def test_digest_excluded_executable_cannot_be_admitted(self):
        workflow = self.root / "excluded-workflow"
        workflow.mkdir()
        executable = workflow / ".DS_Store"
        executable.write_text("#!/usr/bin/python3\nprint('unbound executable')\n")
        executable.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "digest-excluded"):
            snapshot_code(self.root / "with-excluded-workflow", workflow)
        execution = self.execution()
        (self.code / ".DS_Store").write_text(executable.read_text())
        with self.assertRaisesRegex(ValueError, "digest-excluded"):
            execution.verify()

    @unittest.skipUnless(sys.platform == "linux", "Linux namespace test; native qualification mandatory")
    def test_sysfs_projection_masks_cgroup_controls(self):
        alias = self.root / "sysfs-alias"
        alias.symlink_to("/sys", target_is_directory=True)
        chained = self.root / "sysfs-chained"
        chained.symlink_to(alias, target_is_directory=True)
        paths = ("/sys", "/sys/", "/sys/.", str(alias), str(alias) + "/",
                 str(chained), str(chained) + "/")
        for index, path in enumerate(paths):
            with self.subTest(projected=path):
                request = self.root / f"sysfs-{index}.json"
                request.write_text(json.dumps({"layers": [{"kind": "base", "source": "host-usr"},
                    {"kind": "host-ro", "name": "sysfs", "paths": [path]}]}))
                record = capsule.build(str(request))
                execution = InsulaExecution.create(record, self.code, self.root / f"state-{index}")
                attempt = self.root / f"sysfs-attempt-{index}"
                attempt.mkdir()
                argv, _ = execution.payload(attempt, {"spec": {"cwd": str(self.code), "argv": [], "env": {}}})
                result = subprocess.run(argv + ["--", "/usr/bin/python3", "-c",
                    "from pathlib import Path; "
                    f"assert (Path({path!r}) / 'devices').is_dir(); "
                    f"assert not (Path({path!r}) / 'fs/cgroup/cgroup.procs').exists(); "
                    "assert not Path('/sys/fs/cgroup/cgroup.procs').exists()"], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)

    @unittest.skipUnless(sys.platform == "linux", "Linux namespace test; native qualification mandatory")
    def test_missing_cgroup_membership_blocks_without_kernel_cleanup_or_retry(self):
        from durable.runtime import Activity, Context
        from durable.runtime.engine import CleanupUnproven
        # Seed durable released state without a membership marker, reproducing
        # the guardian-death gap before the fork child confirms its join. The
        # recovery check must block before consulting any kernel subgroup.
        with closing(Store(self.root / "state", allow_unpatched_sqlite=True)) as store:
            store.submit("r", "one", DEMO_VERSION, {}, "test-contract")
            operation = Context(store, "r")._op("work", "ACTIVITY", Activity(argv=("/usr/bin/true",)).spec())
            directory = store.root / "attempts/lost"
            directory.mkdir(parents=True)
            (directory / "cgroup.json").write_text("{}")
            with store.transaction() as database:
                database.execute("INSERT INTO attempts(id,operation_id,number,epoch,spec_digest,directory,released,created) VALUES(?,?,?,?,?,?,?,?)",
                                 ("lost", operation["id"], 1, 1, operation["spec_digest"], str(directory), 1, time.time()))
                database.execute("UPDATE operations SET status='RUNNING',active_attempt='lost',attempt_count=1 WHERE id=?", (operation["id"],))
            runtime = Runtime(store, REGISTRY, "test-contract")
            # No fake cgroup filesystem or kernel-success claim. This exercises
            # the explicit precondition on a real persisted attempt ledger.
            runtime.execution = InsulaExecution({"backend": "cgroup"})
            attempt = store.db.execute("SELECT * FROM attempts WHERE id='lost'").fetchone()
            runtime._recover_missing_receipt(attempt)
            with self.assertRaises(CleanupUnproven):
                runtime._assert_clean()
            self.assertEqual(store.status("r")["status"], "BLOCKED")
            self.assertEqual(len(store.status("r")["attempts"]), 1)
            self.assertFalse((directory / "receipt.json").exists())

    def test_payload_mounts_only_attempt_and_private_temp(self):
        execution = self.execution()
        attempt = self.root / "attempt"
        attempt.mkdir()
        argv, request = execution.payload(attempt, {"attempt_dir": str(attempt), "spec": {"cwd": str(self.code), "argv": ["/usr/bin/python3"], "env": {}}})
        self.assertEqual(request["attempt_dir"], "/home/bench/run")
        self.assertIn("--ro-bind", argv)
        self.assertIn("/run/insula-tmp", argv)

    @unittest.skipUnless(sys.platform == "linux", "Linux namespace test; native qualification mandatory")
    def test_admitted_inputs_are_readonly_and_other_bundles_and_state_are_absent(self):
        import hashlib
        from durable.common import atomic_json, digest
        inputs = self.root / "inputs"
        inputs.mkdir(mode=0o700)
        manifest = {"schema": 1, "files": {"task.md": {"sha256": hashlib.sha256(b"pinned").hexdigest(), "size": 6}}}
        key = digest(manifest)
        bundle = inputs / key
        bundle.mkdir()
        (bundle / "task.md").write_bytes(b"pinned")
        atomic_json(bundle / "manifest.json", manifest)
        (inputs / "another-bundle").mkdir()
        execution = InsulaExecution.create(self.record, self.code, self.root / "state", inputs_root=inputs)
        self.assertNotEqual(execution.contract, self.execution().contract)
        attempt = self.root / "input-attempt"
        attempt.mkdir()
        request = {"attempt_dir": str(attempt), "spec": {"cwd": None, "env": {}, "inputs": key}}
        argv, _ = execution.payload(attempt, request)
        result = subprocess.run(argv + ["--", "/usr/bin/python3", "-c",
            "from pathlib import Path; "
            "assert Path('/home/bench/input/task.md').read_bytes() == b'pinned'; "
            "assert not Path('/home/bench/input/another-bundle').exists(); "
            f"assert not Path({str(inputs)!r}).exists(); assert not Path({str(self.root / 'state')!r}).exists(); "
            "Path('/home/bench/run/output').write_text('owned'); "
            "\ntry: Path('/home/bench/input/task.md').write_text('forged')"
            "\nexcept OSError: pass"
            "\nelse: raise AssertionError('input mount is writable')"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((attempt / "output").read_text(), "owned")
        self.assertEqual((bundle / "task.md").read_bytes(), b"pinned")
        with self.assertRaises(ValueError):
            self.execution().payload(attempt, request)
        (bundle / "task.md").write_bytes(b"forged")
        with self.assertRaises(ValueError):
            execution.payload(attempt, request)
        from durable.runtime import Activity
        def workflow(ctx, _args):
            return ctx.activity("coding", Activity(argv=("/usr/bin/true",), inputs=key))
        with closing(Store(self.root / "state", allow_unpatched_sqlite=True)) as store:
            store.submit("bad-input", "coding", "v1", {}, execution.contract)
            with Runtime(store, {"coding": ("v1", workflow)}, execution.contract, execution=execution) as runtime:
                with self.assertRaises(ValueError):
                    runtime.drive(timeout=1)
            self.assertEqual(store.status("bad-input")["attempts"], [])

    def test_old_execution_contract_is_unchanged_and_inputs_root_is_bound(self):
        config = self.execution().config
        from insula import capsule
        expected = "insula-durable-v1:" + capsule.sha(capsule.canonical({
            key: config[key] for key in ("capsule", "capsule_store", "code", "code_digest", "state", "backend", "cgroup_root", "cgroup_identity")
        }))
        self.assertEqual(config["contract"], expected)
        self.assertNotIn("inputs_root", config)
        inputs = self.root / "inputs"
        inputs.mkdir(mode=0o700)
        execution = InsulaExecution.create(self.record, self.code, self.root / "state", inputs_root=inputs)
        execution.config["inputs_root"] = str(self.root / "substitute")
        with self.assertRaises(ValueError):
            execution.verify()
        with self.assertRaises(ValueError):
            execution.verify(require_backend=False)

    @unittest.skipUnless(sys.platform == "linux", "Linux namespace test; native qualification mandatory")
    def test_real_insula_activity_environment_and_write_boundary(self):
        execution = self.execution()
        attempt = self.root / "attempt"
        attempt.mkdir()
        req = {"attempt_dir": str(attempt), "spec": {"cwd": str(self.code), "argv": ["/usr/bin/python3"], "env": {}}}
        argv, _ = execution.payload(attempt, req)
        result = subprocess.run(argv + ["--", "/usr/bin/python3", "-c",
            "import os,pathlib,json,tempfile; p=tempfile.NamedTemporaryFile(); "
            "assert p.name.startswith('/run/insula-tmp/'); "
            "assert not pathlib.Path('/sys/fs/cgroup/cgroup.procs').exists(); "
            "pathlib.Path('/home/bench/run/result.json').write_text(json.dumps(dict(tmp=os.environ['TMPDIR'])))"],
            capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads((attempt / "result.json").read_text())["tmp"], "/run/insula-tmp")

    @unittest.skipUnless(sys.platform == "linux", "Linux namespace test; native qualification mandatory")
    def test_runtime_pipeline_in_real_insula(self):
        execution = self.execution()
        with closing(Store(self.root / "state", allow_unpatched_sqlite=True)) as store:
            store.submit("r", "pipeline", DEMO_VERSION, {"value": 3}, execution.contract)
            with Runtime(store, REGISTRY, execution.contract, execution=execution) as runtime:
                runtime.drive()
            status = store.status("r")
            self.assertEqual(status["status"], "SUCCEEDED", status)
            self.assertEqual(status["result"]["sum"], 181)
            self.assertEqual(len(status["attempts"]), 4)
            for attempt in status["attempts"]:
                directory = Path(attempt["directory"])
                self.assertTrue((directory / "payload/result.json").is_file())
                self.assertFalse((directory / "payload/receipt.json").exists())

    @unittest.skipUnless(sys.platform == "linux", "Linux launcher; native qualification mandatory")
    def test_launcher_prepares_and_executes_admitted_snapshot(self):
        config = self.root / "execution.json"
        record_id = self.record["capsule_digest"]
        result = self.launch("prepare", "--capsule", record_id, "--state", str(self.root / "state"),
                             "--execution", str(config))
        self.assertEqual(result.returncode, 0, result.stderr)
        binding = json.loads(config.read_text())
        self.assertEqual(binding["capsule"]["capsule_digest"], record_id)
        self.assertNotIn("runtime_test.py", [p.name for p in Path(binding["code"]).rglob("*.py")])
        for arguments in [["submit", "r", "--args", '{"value":3}'], ["run", "--max-active", "2"]]:
            result = self.launch("exec", "--execution", str(config), "--", "--allow-unpatched-sqlite", *arguments)
            self.assertEqual(result.returncode, 0, result.stderr)
        result = self.launch("exec", "--execution", str(config), "--", "--allow-unpatched-sqlite", "status", "r")
        self.assertEqual(json.loads(result.stdout)["result"]["sum"], 181)

    def test_cgroup_admission_refuses_fake_delegation_before_submission(self):
        result = self.launch("prepare", "--capsule", self.record["capsule_digest"],
                             "--state", str(self.root / "state"), "--execution", str(self.root / "execution.json"),
                             "--backend", "cgroup", "--cgroup-root", str(self.root))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cgroup v2", result.stderr)
        self.assertFalse((self.root / "execution.json").exists())
        self.assertFalse((self.root / "state/runtime.sqlite").exists())

    @unittest.skipUnless(sys.platform == "linux", "Linux namespace test; native qualification mandatory")
    def test_payload_cannot_publish_authoritative_guardian_receipt(self):
        from durable.runtime import Activity
        execution = self.execution()
        code = "import json,sys,pathlib; r=json.load(sys.stdin); p=pathlib.Path(r['attempt_dir']); " \
               "(p/'receipt.json').write_text('forged'); (p/'result.json').write_text('{\"ok\":true}')"
        def workflow(ctx, _args):
            return ctx.activity("forge", Activity(argv=("/usr/bin/python3", "-c", code), recovery="retry_safe"))
        with closing(Store(self.root / "state", allow_unpatched_sqlite=True)) as store:
            store.submit("r", "forge", "v1", {}, execution.contract)
            with Runtime(store, {"forge": ("v1", workflow)}, execution.contract, execution=execution) as runtime:
                runtime.drive()
            status = store.status("r")
            self.assertEqual(status["result"], {"ok": True})
            base = Path(status["attempts"][0]["directory"])
            self.assertEqual((base / "payload/receipt.json").read_text(), "forged")
            self.assertEqual(json.loads((base / "receipt.json").read_text())["status"], "SUCCEEDED")

    def test_fifo_control_file_is_rejected_without_blocking_guardian(self):
        fifo = self.root / "result.json"
        os.mkfifo(fifo)
        result = subprocess.run([sys.executable, "-c",
                                 "from pathlib import Path; from durable.common import read_json; "
                                 "read_json(Path(__import__('sys').argv[1]))", str(fifo)],
                                capture_output=True, text=True, timeout=2)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("regular file", result.stderr)

    def launch(self, *args):
        return subprocess.run([sys.executable, "-m", "durable.runtime.admit", *args],
                              capture_output=True, text=True, timeout=30)


def insula_crash_test(point):
    def test(self):
        execution = self.execution()
        configuration = self.root / "execution.json"
        configuration.write_text(json.dumps(execution.config))
        with closing(Store(self.root / "state", allow_unpatched_sqlite=True)) as store:
            store.submit("r", "one", DEMO_VERSION, {"input": {"value": 7}}, execution.contract)
            command = [sys.executable, "-m", "durable.runtime", "--state", str(store.root),
                       "--contract", execution.contract, "--execution", str(configuration),
                       "--allow-unpatched-sqlite", "run", "--test-crash-at", point]
            result = subprocess.run(command, capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 86, result.stderr)
            with Runtime(store, REGISTRY, execution.contract, execution=execution) as runtime:
                runtime.drive()
            status = store.status("r")
            self.assertEqual(status["status"], "SUCCEEDED", status)
            self.assertEqual(status["result"]["square"], 49)
            if point in {"after_receipt", "after_commit"}:
                self.assertEqual(len(status["attempts"]), 1)
    return unittest.skipUnless(sys.platform == "linux", "Linux crash test; native qualification mandatory")(test)


for _point in ["after_claim", "after_guard_spawn", "before_release", "after_release", "after_receipt", "after_commit"]:
    setattr(AdmissionTests, f"test_insula_crash_{_point}", insula_crash_test(_point))


class DelegatedRuntimeTests(unittest.TestCase):
    def setUp(self):
        delegated = os.environ.get("INSULA_TEST_CGROUP_ROOT")
        if not delegated:
            self.skipTest("cgroup Runtime requires explicit INSULA_TEST_CGROUP_ROOT delegation")
        self.delegated = Path(delegated)
        AdmissionTests.setUp(self)
        self.addCleanup(AdmissionTests.tearDown, self)
        self.execution = InsulaExecution.create(self.record, self.code, self.root / "state",
                                               backend="cgroup", cgroup_root=self.delegated)

    def remove_attempt_groups(self, store):
        from insula.cgroup import Cgroup
        for attempt in store.db.execute("SELECT id,directory FROM attempts"):
            file = Path(attempt["directory"]) / "cgroup.json"
            if file.exists():
                group = Cgroup.resume(json.loads(file.read_text()), attempt_id=attempt["id"])
                group.kill_and_wait()
                group.path.rmdir()

    def test_cgroup_runtime_pipeline_accepts_kernel_cleanup_receipts(self):
        with closing(Store(self.root / "state", allow_unpatched_sqlite=True)) as store:
            try:
                store.submit("r", "pipeline", DEMO_VERSION, {"value": 3}, self.execution.contract)
                with Runtime(store, REGISTRY, self.execution.contract, execution=self.execution) as runtime:
                    runtime.drive()
                status = store.status("r")
                self.assertEqual(status["result"]["sum"], 181)
                for attempt in status["attempts"]:
                    receipt = json.loads((Path(attempt["directory"]) / "receipt.json").read_text())
                    self.assertEqual(receipt["cleanup_proof"]["populated"], 0)
            finally:
                self.remove_attempt_groups(store)

    def test_killed_cgroup_guardian_cleans_but_keeps_manual_effect_uncertain(self):
        with closing(Store(self.root / "state", allow_unpatched_sqlite=True)) as store:
            try:
                store.submit("r", "one", DEMO_VERSION,
                             {"input": {"sleep": 10}, "recovery": "manual"}, self.execution.contract)
                with Runtime(store, REGISTRY, self.execution.contract, execution=self.execution) as runtime:
                    deadline = time.monotonic() + 10
                    while not list((store.root / "attempts").glob("*/payload/started.json")):
                        runtime.tick()
                        self.assertLess(time.monotonic(), deadline)
                        time.sleep(0.01)
                    job = next(iter(runtime.live.values()))
                    job.process.kill()
                    job.process.wait(timeout=5)
                    runtime.drive()
                status = store.status("r")
                self.assertEqual(status["status"], "BLOCKED")
                self.assertEqual(status["operations"][0]["status"], "UNCERTAIN")
                self.assertEqual(len(status["attempts"]), 1)
                self.assertFalse(store.db.execute("SELECT 1 FROM cleanup_blocks WHERE acknowledged IS NULL").fetchone())
            finally:
                self.remove_attempt_groups(store)


if __name__ == "__main__":
    unittest.main()

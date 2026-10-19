"""Disposable Git/SQLite owner decisions; live guardian qualification is separate."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import unittest

from controller.durable.common import atomic_json, digest
from . import test_candidates as fixtures
from .admission import git
from .candidates import capture, request_verification
from .landing import approve, land, owner_init


class LandingTests(unittest.TestCase):
    setUp = fixtures.CaptureTests.setUp
    accept_result = fixtures.CaptureTests.accept_result

    def prepare_owner_and_verification(self):
        captured = capture(self.store, "coding-run")
        store = self.store
        class Execution:
            backend = "cgroup"
            contract = "verification-fixture"
            config = {"state": str(store.root), "inputs_root": str(store.root / "inputs")}
            def verify(self):
                pass
        requested = request_verification(store, Execution(), "coding-run")
        self.verification_id = requested["run_id"]
        directory = self.state / "attempts" / ("b" * 32)
        (directory / "payload/checks").mkdir(parents=True)
        logs = {}
        for stream in ("stdout", "stderr"):
            data = b"verified\n" if stream == "stdout" else b""
            name = f"checks/000.{stream}.log"
            (directory / "payload" / name).write_bytes(data)
            logs[stream] = {"path": name, "sha256": hashlib.sha256(data).hexdigest(),
                            "retained_bytes": len(data), "output_bytes": len(data), "truncated": False}
        proof = {"schema": 1, "subject": captured["subject"], **{
            key: requested[key] for key in ("inputs", "subject_digest", "checks_digest", "execution_contract")},
            "verification_attempt": "b" * 32, "passed": True,
            "checks": [{"argv": ["/usr/bin/true"], "returncode": 0,
                        "tracked_source_unchanged": True, "logs": logs}]}
        # This fixture represents accepted terminal records only. Real receipt
        # creation and namespace isolation are exercised by the Linux CLI suite.
        receipt = {"protocol": 1, "attempt_id": "b" * 32, "operation_id": "verification-op",
                   "spec_digest": "verification-spec", "status": "SUCCEEDED", "started": True,
                   "cleanup": True, "reason": None, "result": proof, "result_digest": digest(proof)}
        self.verification_receipt = directory / "receipt.json"
        atomic_json(self.verification_receipt, receipt)
        with self.store.transaction() as db:
            db.execute("INSERT INTO operations(id,run_id,seq,key,kind,spec,spec_digest,status,attempt_count,result) "
                       "VALUES('verification-op',?,0,'verify','ACTIVITY','{}','verification-spec','SUCCEEDED',1,?)",
                       (self.verification_id, json.dumps(proof)))
            db.execute("INSERT INTO attempts(id,operation_id,number,epoch,spec_digest,directory,status,created,receipt_digest) "
                       "VALUES(?,'verification-op',1,1,'verification-spec',?,'SUCCEEDED',0,?)",
                       ("b" * 32, str(directory), digest(receipt)))
            db.execute("UPDATE runs SET status='SUCCEEDED',result=? WHERE id=?", (json.dumps(proof), self.verification_id))
        self.destination = self.root / "destination"
        git(self.root, "clone", "-q", "--no-checkout", str(self.repo), str(self.destination))
        git(self.destination, "checkout", "-q", "--detach", self.base)
        git(self.destination, "checkout", "-qb", "operator")
        git(self.destination, "config", "user.name", "Disposable operator")
        git(self.destination, "config", "user.email", "operator@harp.local")
        private = self.root / "operator-private"
        private.mkdir(mode=0o700)
        self.capability = private / "owner.json"
        initialized = owner_init(self.store, self.capability)
        self.assertTrue(initialized["registered"])
        self.assertNotIn(json.loads(self.capability.read_text())["token"], json.dumps(initialized))
        self.assertEqual(owner_init(self.store, self.capability), initialized)
        return captured

    def approval(self):
        return approve(self.store, "coding-run", self.verification_id, self.destination, self.capability)["approval_id"]

    def snapshot(self):
        return (git(self.destination, "rev-parse", "HEAD"),
                (self.destination / ".git/index").read_bytes(),
                {str(p.relative_to(self.destination)): p.read_bytes() for p in self.destination.rglob("*")
                 if ".git" not in p.relative_to(self.destination).parts and p.is_file() and not p.is_symlink()})

    def test_owner_approval_physically_lands_branch_index_and_bytes(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        result = land(self.store, approval_id, self.capability)
        self.assertEqual(result["status"], "LANDED")
        self.assertFalse(result["reconciled"])
        self.assertEqual(git(self.destination, "rev-parse", "HEAD").decode().strip(), self.commit)
        self.assertEqual(git(self.destination, "write-tree").decode().strip(), self.tree)
        self.assertEqual((self.destination / "answer.txt").read_text(), "42\n")
        self.assertEqual(git(self.destination, "status", "--porcelain"), b"")
        self.assertTrue(land(self.store, approval_id, self.capability)["reconciled"])

    def test_branch_squash_default_cannot_replace_exact_fast_forward_landing(self):
        self.prepare_owner_and_verification()
        git(self.destination, "config", "branch.operator.mergeOptions", "--squash")
        approval_id = self.approval()
        result = land(self.store, approval_id, self.capability)
        self.assertEqual(result["status"], "LANDED")
        self.assertEqual(git(self.destination, "rev-parse", "HEAD").decode().strip(), self.commit)
        self.assertEqual(git(self.destination, "write-tree").decode().strip(), self.tree)
        self.assertEqual((self.destination / "answer.txt").read_text(), "42\n")
        self.assertEqual(git(self.destination, "status", "--porcelain"), b"")
        self.assertEqual(git(self.destination, "config", "branch.operator.mergeOptions").decode().strip(),
                         "--squash")

    def test_role_credentials_and_replaced_or_public_owner_file_are_refused(self):
        self.prepare_owner_and_verification()
        bad = self.capability.with_name("worker.json")
        bad.write_text(json.dumps({"schema": 1, "role": "worker", "state": str(self.state), "token": "c" * 64}))
        bad.chmod(0o600)
        with self.assertRaisesRegex(ValueError, "worker/lead"):
            approve(self.store, "coding-run", self.verification_id, self.destination, bad)
        self.capability.chmod(0o644)
        with self.assertRaisesRegex(ValueError, "private"):
            self.approval()
        self.capability.chmod(0o600)
        data = self.capability.read_bytes()
        replacement = self.capability.with_name("replacement")
        replacement.write_bytes(data)
        replacement.chmod(0o600)
        replacement.replace(self.capability)
        with self.assertRaisesRegex(ValueError, "identity changed"):
            self.approval()

    def test_owner_path_cannot_be_initialized_in_state_or_projected_profile(self):
        self.prepare_owner_and_verification()
        with self.assertRaisesRegex(ValueError, "projections"):
            owner_init(self.store, self.state / "owner.json")
        store, private = self.store, self.capability.parent
        class Execution:
            config = {"state": str(store.root), "capsule": {"layers": [
                {"kind": "host-ro", "paths": [str(private)]}]}}
            def verify(self, **kwargs):
                pass
        with self.assertRaisesRegex(ValueError, "projections"):
            owner_init(self.store, self.capability, Execution())
        alias = self.capability.with_name("alias")
        alias.symlink_to(self.capability)
        with self.assertRaisesRegex(ValueError, "symlinks"):
            approve(self.store, "coding-run", self.verification_id, self.destination, alias)

    def test_dirty_untracked_and_ignored_collisions_are_preserved(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        (self.destination / "task.md").write_text("operator dirty bytes\n")
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "dirty|tracked bytes"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)
        # Restore only this fixture's known original bytes, without Git reset/stash.
        (self.destination / "task.md").write_bytes((self.repo / "task.md").read_bytes())
        (self.destination / "notes.txt").write_text("untracked user work\n")
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "dirty"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)
        (self.destination / "notes.txt").unlink()
        (self.destination / ".git/info/exclude").write_text("answer.txt\n")
        (self.destination / "answer.txt").write_text("ignored user work\n")
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "ignored"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)
        self.assertEqual(self.store.db.execute("SELECT count(*) FROM harp_approvals").fetchone()[0], 1)
        self.assertTrue(self.store.db.execute("SELECT 1 FROM events WHERE kind='LANDING_REFUSED'").fetchone())

    def test_unrelated_ignored_scratch_survives_landing(self):
        self.prepare_owner_and_verification()
        (self.destination / ".git/info/exclude").write_text("cache/\n")
        (self.destination / "cache").mkdir()
        (self.destination / "cache/note").write_text("retained cache")
        land(self.store, self.approval(), self.capability)
        self.assertEqual((self.destination / "cache/note").read_text(), "retained cache")

    def test_ignored_file_appearing_after_preflight_is_never_overwritten(self):
        self.prepare_owner_and_verification()
        (self.destination / ".git/info/exclude").write_text("answer.txt\n")
        approval_id = self.approval()
        expected = []
        def race(point, detail=None):
            if point == "after_intent":
                (self.destination / "answer.txt").write_text("concurrent operator work\n")
                expected.append(self.snapshot())
        with self.assertRaises(ValueError):
            land(self.store, approval_id, self.capability, failpoint=race)
        self.assertEqual(self.snapshot(), expected[0])
        self.assertEqual((self.destination / "answer.txt").read_text(), "concurrent operator work\n")

    def test_stat_cache_cannot_hide_dirty_tracked_bytes(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        path = self.destination / "task.md"
        original = path.read_bytes()
        git(self.destination, "config", "core.trustctime", "false")
        git(self.destination, "config", "core.checkStat", "minimal")
        before_time = path.stat().st_mtime_ns - 10_000_000_000
        os.utime(path, ns=(before_time, before_time))
        git(self.destination, "update-index", "--refresh", "--", "task.md")
        path.write_bytes(b"Q" + original[1:])
        os.utime(path, ns=(before_time, before_time))
        self.assertEqual(git(self.destination, "status", "--porcelain"), b"")
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "tracked bytes"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)

    def test_local_checkout_conversions_refuse_before_mutating_destination(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        for name, value in (("core.autocrlf", "true"), ("core.eol", "crlf"),
                            ("core.symlinks", "false"), ("core.sparseCheckout", "true")):
            with self.subTest(name=name):
                git(self.destination, "config", name, value)
                before = self.snapshot()
                with self.assertRaises(ValueError):
                    land(self.store, approval_id, self.capability)
                self.assertEqual(self.snapshot(), before)

                self.assertFalse(self.store.db.execute("SELECT 1 FROM harp_landing_intents").fetchone())
                git(self.destination, "config", "--unset", name)
        marker = self.root / "filter-executed"
        git(self.destination, "config", "filter.evil.clean", "/usr/bin/touch " + str(marker))
        (self.destination / ".git/info/attributes").write_text("task.md filter=evil\n")
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "filters"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)
        self.assertFalse(marker.exists())
        git(self.destination, "config", "--unset", "filter.evil.clean")
        for conversion in ("*.txt eol=crlf", "*.txt ident", "*.txt working-tree-encoding=UTF-16"):
            with self.subTest(conversion=conversion):
                (self.destination / ".git/info/attributes").write_text(conversion + "\n")
                before = self.snapshot()
                with self.assertRaisesRegex(ValueError, "info/attributes"):
                    land(self.store, approval_id, self.capability)
                self.assertEqual(self.snapshot(), before)

    def test_external_attributes_are_refused_or_neutralized_without_partial_checkout(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        attributes = self.root / "external-attributes"
        attributes.write_text("*.txt eol=crlf\n")
        git(self.destination, "config", "core.attributesFile", str(attributes))
        before = self.snapshot()
        try:
            result = land(self.store, approval_id, self.capability)
        except ValueError:
            self.assertEqual(self.snapshot(), before)
        else:
            self.assertEqual(result["status"], "LANDED")
            self.assertEqual((self.destination / "answer.txt").read_bytes(), b"42\n")
            self.assertEqual(git(self.destination, "write-tree").decode().strip(), self.tree)

    def test_advanced_and_substituted_destination_refuse_without_mutation(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        (self.destination / "operator.txt").write_text("new independent work\n")
        git(self.destination, "add", "operator.txt")
        git(self.destination, "commit", "-qm", "Independent advance")
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "advanced"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)
        self.assertEqual(self.store.db.execute("SELECT status FROM harp_approvals WHERE id=?", (approval_id,)).fetchone()[0], "STALE")

    def test_same_path_repository_substitution_does_not_gain_approval(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        old = self.root / "previous-destination"
        self.destination.rename(old)
        git(self.root, "clone", "-q", str(old), str(self.destination))
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "substituted"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)

    def test_changed_verification_receipt_or_logs_cannot_authorize_landing(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        log = self.verification_receipt.parent / "payload/checks/000.stdout.log"
        log.write_text("substituted evidence")
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "logs changed"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)
        log.write_bytes(b"verified\n")
        self.verification_receipt.write_text("{}")
        with self.assertRaisesRegex(ValueError, "receipt"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)

    def process(self, approval_id, script="", *args):
        code = ("import json,os,sys,time; from pathlib import Path; "
                "from controller.durable.engine import Store; from harp_durable.landing import land; "
                "store=Store(sys.argv[1],allow_unpatched_sqlite=True); " + script +
                "\nprint(json.dumps(land(store,sys.argv[2],Path(sys.argv[3]),failpoint=point)),flush=True)")
        env = {**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parents[1] / "corenius") + ":" + str(Path(__file__).resolve().parents[1]),
               "PYTHONDONTWRITEBYTECODE": "1"}
        child = subprocess.Popen([sys.executable, "-c", code, str(self.state), approval_id,
                                  str(self.capability), *args], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def cleanup():
            if child.poll() is None:
                child.kill()
            child.communicate(timeout=10)
        self.addCleanup(cleanup)
        return child

    def test_process_crashes_before_and_after_git_are_reconciled(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        script = "\ndef point(name,detail=None):\n if name==sys.argv[4]: os._exit(86)\n"
        first = self.process(approval_id, script, "after_intent")
        self.assertEqual(first.wait(timeout=20), 86)
        self.assertEqual(git(self.destination, "rev-parse", "HEAD").decode().strip(), self.base)
        second = self.process(approval_id, script, "after_git")
        self.assertEqual(second.wait(timeout=20), 86)
        self.assertEqual(git(self.destination, "rev-parse", "HEAD").decode().strip(), self.commit)
        self.assertTrue(land(self.store, approval_id, self.capability)["reconciled"])
        self.assertEqual((self.destination / "answer.txt").read_text(), "42\n")

    def test_consumed_approval_cannot_be_reapplied_after_external_restoration(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        land(self.store, approval_id, self.capability)
        # Simulate an external operator restoring branch/index/worktree via
        # plumbing, rather than letting this module perform a reset or force.
        git(self.destination, "read-tree", "-u", "-m", self.commit, self.base)
        git(self.destination, "update-ref", "refs/heads/operator", self.base, self.commit)
        before = self.snapshot()
        with self.assertRaisesRegex(ValueError, "consumed"):
            land(self.store, approval_id, self.capability)
        self.assertEqual(self.snapshot(), before)

    def test_concurrent_requests_serialize_one_physical_branch_transition(self):
        self.prepare_owner_and_verification()
        approval_id = self.approval()
        marker, release = self.root / "intent-ready", self.root / "release"
        first_script = ("\ndef point(name,detail=None):\n if name=='after_intent':\n"
                        "  Path(sys.argv[4]).write_text('ready')\n"
                        "  deadline=time.monotonic()+10\n"
                        "  while not Path(sys.argv[5]).exists():\n"
                        "   if time.monotonic()>deadline: raise TimeoutError('fixture release missing')\n"
                        "   time.sleep(.01)\n")
        first = self.process(approval_id, first_script, str(marker), str(release))
        deadline = time.monotonic() + 10
        while not marker.exists():
            self.assertIsNone(first.poll())
            self.assertLess(time.monotonic(), deadline)
            time.sleep(.01)
        second_ready = self.root / "second-ready"
        second = self.process(approval_id, "Path(sys.argv[4]).write_text('ready')\ndef point(name,detail=None): pass\n",
                              str(second_ready))
        while not second_ready.exists():
            self.assertIsNone(second.poll())
            self.assertLess(time.monotonic(), deadline)
            time.sleep(.01)
        time.sleep(.1)
        self.assertIsNone(second.poll())
        release.write_text("release")
        results = []
        for child in (first, second):
            out, err = child.communicate(timeout=20)
            self.assertEqual(child.returncode, 0, err)
            results.append(json.loads(out))
        self.assertEqual(sorted(r["reconciled"] for r in results), [False, True])
        history = git(self.destination, "reflog", "show", "--format=%H", "refs/heads/operator").decode().splitlines()
        self.assertEqual(history.count(self.commit), 1)


if __name__ == "__main__":
    unittest.main()

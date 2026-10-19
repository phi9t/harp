"""Real Git/SQLite capture boundaries; namespace qualification runs on Linux."""
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from durable.common import atomic_json, digest, verify_input_bundle
from durable.runtime.engine import Store
from durable.agents.admission import git
from durable.git.candidates import capture, check_git_subject, copy_file, request_verification
from durable.git.verifier import MAX_LOG_BYTES, run_check, tracked_tree, unchanged, verify


def publish(root, contents):
    stage = root / "staging"
    stage.mkdir()
    files = {}
    for name, value in contents.items():
        (stage / name).write_bytes(value)
        files[name] = {"sha256": hashlib.sha256(value).hexdigest(), "size": len(value)}
    manifest = {"schema": 1, "files": files}
    identity = digest(manifest)
    atomic_json(stage / "manifest.json", manifest)
    stage.rename(root / identity)
    return identity


class CaptureTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.repo = self.root / "source"
        self.repo.mkdir()
        git(self.repo, "init", "-q")
        git(self.repo, "config", "user.name", "Fixture")
        git(self.repo, "config", "user.email", "fixture@harp.local")
        task = b"Produce answer.txt containing 42.\n"
        checks = b'{"allowed_paths": ["**"], "checks":[["/usr/bin/true"]]}'
        (self.repo / "task.md").write_bytes(task)
        (self.repo / "acceptance.json").write_bytes(checks)
        git(self.repo, "add", "task.md", "acceptance.json")
        git(self.repo, "commit", "-qm", "Admitted task")
        self.base = git(self.repo, "rev-parse", "HEAD").decode().strip()
        (self.repo / "answer.txt").write_text("42\n")
        (self.repo / "link").symlink_to("answer.txt")
        git(self.repo, "add", "answer.txt", "link")
        git(self.repo, "commit", "-qm", "Proposed answer")
        self.commit = git(self.repo, "rev-parse", "HEAD").decode().strip()
        self.tree = git(self.repo, "rev-parse", "HEAD^{tree}").decode().strip()
        self.state = self.root / "state"
        self.state.mkdir(mode=0o700)
        self.store = Store(self.state, allow_unpatched_sqlite=True)
        self.addCleanup(self.store.close)
        inputs = self.state / "inputs"
        inputs.mkdir(mode=0o700)
        request = {"schema": 1, "run_id": "coding-run", "base": self.base,
                   "provider": "deterministic", "workflow_version": "coding-v2",
                   "execution_contract": "fixture-contract",
                   "task_sha256": hashlib.sha256(task).hexdigest(),
                   "acceptance_sha256": hashlib.sha256(checks).hexdigest()}
        identity = digest(request)
        self.original = publish(inputs, {"request.json": json.dumps(request).encode(),
                                         "task.md": task, "acceptance.json": checks})
        args = {"identity": identity, "inputs": self.original, "provider": "deterministic"}
        self.store.submit("coding-run", "coding", "coding-v2", args, "fixture-contract")
        directory = self.state / "attempts" / ("a" * 32)
        (directory / "payload").mkdir(parents=True)
        self.bundle = directory / "payload/candidate.bundle"
        git(self.repo, "bundle", "create", str(self.bundle), "HEAD")
        self.result = {"schema": 1, "identity": identity, "attempt_id": "a" * 32,
                       "base": self.base, "provider": "deterministic", "commit": self.commit,
                       "tree": self.tree, "bundle": "candidate.bundle",
                       "bundle_sha256": hashlib.sha256(self.bundle.read_bytes()).hexdigest()}
        self.receipt = directory / "receipt.json"
        self.directory = directory
        self.accept_result()

    def accept_result(self):
        # Materialize the durable records produced by a successful guardian.
        # Capture must authenticate these records, not a free-standing result file.
        receipt = {"protocol": 1, "attempt_id": "a" * 32, "operation_id": "coding-op",
                   "spec_digest": "spec", "status": "SUCCEEDED", "started": True,
                   "cleanup": True, "reason": None, "result": self.result,
                   "result_digest": digest(self.result)}
        atomic_json(self.receipt, receipt)
        with self.store.transaction() as db:
            db.execute("DELETE FROM attempts")
            db.execute("DELETE FROM operations")
            db.execute("INSERT INTO operations(id,run_id,seq,key,kind,spec,spec_digest,status,attempt_count,result) "
                       "VALUES('coding-op','coding-run',0,'coding','ACTIVITY','{}','spec','SUCCEEDED',1,?)",
                       (json.dumps(self.result),))
            db.execute("INSERT INTO attempts(id,operation_id,number,epoch,spec_digest,directory,status,created,receipt_digest) "
                       "VALUES(?,'coding-op',1,1,'spec',?,'SUCCEEDED',0,?)",
                       ("a" * 32, str(self.directory), digest(receipt)))
            db.execute("UPDATE runs SET status='SUCCEEDED', result=? WHERE id='coding-run'",
                       (json.dumps(self.result),))

    def test_capture_survives_worker_workspace_and_payload_removal(self):
        before_refs = git(self.repo, "show-ref")
        first = capture(self.store, "coding-run")
        frozen = verify_input_bundle(self.state / "inputs", first["inputs"])
        self.assertEqual((frozen / "candidate.bundle").read_bytes(), self.bundle.read_bytes())
        (self.repo / "answer.txt").write_text("uncommitted later edits\n")
        self.bundle.unlink()
        self.receipt.unlink()
        self.assertEqual(capture(self.store, "coding-run"), first)
        self.assertEqual(git(self.repo, "show-ref"), before_refs)
        check_git_subject(frozen / "candidate.bundle", first["subject"], self.root / "independent")
        self.assertEqual((self.root / "independent/answer.txt").read_text(), "42\n")

    def test_capture_refuses_symlinked_payload_and_missing_receipt(self):
        original = self.root / "outside.bundle"
        self.bundle.rename(original)
        self.bundle.symlink_to(original)
        with self.assertRaisesRegex(ValueError, "symlinks"):
            capture(self.store, "coding-run")
        self.bundle.unlink()
        original.rename(self.bundle)
        self.receipt.unlink()
        with self.assertRaises(OSError):
            capture(self.store, "coding-run")

    def test_capture_refuses_changed_bundle_and_forged_commit_or_tree(self):
        original = self.bundle.read_bytes()
        self.bundle.write_bytes(original + b"extra")
        with self.assertRaisesRegex(ValueError, "digest"):
            capture(self.store, "coding-run")
        self.bundle.write_bytes(original)
        for field in ("commit", "tree", "base"):
            with self.subTest(field=field):
                old = self.result[field]
                self.result[field] = "f" * 40
                self.accept_result()
                with self.assertRaises(ValueError):
                    capture(self.store, "coding-run")
                self.result[field] = old
        self.accept_result()

    def test_free_standing_or_stale_result_is_not_capture_authority(self):
        with self.store.transaction() as db:
            db.execute("UPDATE runs SET status='RUNNING' WHERE id='coding-run'")
        with self.assertRaisesRegex(ValueError, "guardian-accepted"):
            capture(self.store, "coding-run")
        with self.store.transaction() as db:
            db.execute("UPDATE runs SET status='SUCCEEDED' WHERE id='coding-run'")
        self.receipt.write_text("{}")
        with self.assertRaisesRegex(ValueError, "evidence"):
            capture(self.store, "coding-run")

    def test_changed_captured_bundle_cannot_reuse_evidence(self):
        record = capture(self.store, "coding-run")
        frozen = self.state / "inputs" / record["inputs"]
        (frozen / "candidate.bundle").chmod(0o600)
        (frozen / "candidate.bundle").write_bytes(b"substituted")
        with self.assertRaises(ValueError):
            capture(self.store, "coding-run")

    def test_changed_admitted_task_and_substituted_capture_are_rejected(self):
        record = capture(self.store, "coding-run")
        path = self.state / "candidates" / (digest("coding-run") + ".json")
        changed = json.loads(path.read_text())
        changed["subject"]["commit"] = self.base
        atomic_json(path, changed)
        with self.assertRaisesRegex(ValueError, "stale evidence"):
            capture(self.store, "coding-run")
        atomic_json(path, record)
        (self.state / "inputs" / self.original / "task.md").write_text("different task")
        with self.assertRaises(ValueError):
            capture(self.store, "coding-run")

    def test_regular_copy_rejects_fifo_symlink_and_oversized_files(self):
        fifo = self.root / "fifo"
        os.mkfifo(fifo)
        with self.assertRaisesRegex(ValueError, "regular"):
            copy_file(fifo, self.root / "copy-fifo")
        alias = self.root / "alias"
        alias.symlink_to(self.bundle)
        with self.assertRaisesRegex(ValueError, "symlinks"):
            copy_file(alias, self.root / "copy-link")
        with self.assertRaisesRegex(ValueError, "bounded"):
            copy_file(self.bundle, self.root / "copy-large", limit=1)

    def test_verification_submission_is_durable_and_exact_subject_idempotent(self):
        class Execution:
            backend = "cgroup"
            contract = "verification-fixture"
            config = {"state": str(self.state), "inputs_root": str(self.state / "inputs")}

            def verify(self):
                pass
        first = request_verification(self.store, Execution(), "coding-run")
        second = request_verification(self.store, Execution(), "coding-run")
        self.assertEqual(first["run_id"], second["run_id"])
        self.assertFalse(first["reused"])
        self.assertTrue(second["reused"])
        admitted = self.store.status(first["run_id"])
        self.assertEqual(admitted["workflow"], "verification")
        self.assertEqual(admitted["contract"], "verification-fixture")
        self.assertEqual(admitted["version"], "verification-v2")

    def test_verifier_rejects_substituted_environment_or_checks_before_execution(self):
        record = capture(self.store, "coding-run")
        inputs = self.state / "inputs" / record["inputs"]
        binding = json.loads((inputs / "verification.json").read_text())
        envelope = {"input": {"subject_digest": record["subject_digest"], "inputs": record["inputs"],
                               "checks_digest": binding["checks_digest"], "execution_contract": "expected"},
                    "attempt_id": "verification-attempt"}
        attempt = self.root / "verifier"
        attempt.mkdir()
        with patch.dict(os.environ, INSULA_DURABLE_CONTRACT="other"):
            with self.assertRaisesRegex(ValueError, "execution binding"):
                verify(envelope, inputs, attempt)
        (inputs / "acceptance.json").chmod(0o600)
        (inputs / "acceptance.json").write_text('{"allowed_paths": ["**"], "checks":[["/usr/bin/false"]]}')
        with patch.dict(os.environ, INSULA_DURABLE_CONTRACT="expected"):
            with self.assertRaisesRegex(ValueError, "checks or admitted"):
                verify(envelope, inputs, attempt)
        self.assertEqual(list(attempt.iterdir()), [])

    def test_tracked_byte_check_ignores_untrusted_index_flags(self):
        roster = tracked_tree(self.repo, self.commit)
        self.assertTrue(unchanged(self.repo, roster))
        git(self.repo, "update-index", "--assume-unchanged", "answer.txt")
        (self.repo / "answer.txt").write_text("different\n")
        self.assertFalse(unchanged(self.repo, roster))
        (self.repo / "answer.txt").write_text("42\n")
        self.assertTrue(unchanged(self.repo, roster))
        (self.repo / "link").unlink()
        (self.repo / "link").symlink_to("task.md")
        self.assertFalse(unchanged(self.repo, roster))


class LogTests(unittest.TestCase):
    def test_failed_command_retains_bounded_logs_and_exact_exit_status(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            outcome = run_check([sys.executable, "-c",
                f"import sys; sys.stdout.write('x'*{MAX_LOG_BYTES + 17}); sys.stderr.write('failure'); sys.exit(7)"], root, 0)
            self.assertEqual(outcome["returncode"], 7)
            self.assertTrue(outcome["logs"]["stdout"]["truncated"])
            self.assertEqual(outcome["logs"]["stdout"]["retained_bytes"], MAX_LOG_BYTES)
            self.assertEqual((root / "000.stderr.log").read_text(), "failure")


if __name__ == "__main__":
    unittest.main()

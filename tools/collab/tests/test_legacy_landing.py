"""Legacy landing CLI behavior in real disposable Git repositories."""
from __future__ import annotations

import fcntl
import os
import json
import shlex
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "tools/agents/land.py"


class LegacyLandingTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.root = Path(self.tmp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "owner")
        self.git("config", "user.email", "owner@example.invalid")
        self.git("config", "--add", "collab.overrideOwners", "owner")
        (self.repo / "README").write_text("base\n")
        self.git("add", "README")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.worker = self.root / "worker"
        self.git("worktree", "add", "-q", "-b", "topic", str(self.worker))
        (self.worker / "result").write_text("candidate\n")
        self.git("add", "result", cwd=self.worker)
        self.git("commit", "-qm", "candidate", cwd=self.worker)
        self.tip = self.git("rev-parse", "topic").strip()

    def tearDown(self):
        self.tmp.cleanup()

    def git(self, *args, cwd=None):
        result = subprocess.run(["git", "-C", str(cwd or self.repo), *args], capture_output=True, text=True)
        if result.returncode:
            self.fail(result.stderr)
        return result.stdout

    def call(self, *args, env=None):
        environment = os.environ.copy()
        environment.update(env or {})
        return subprocess.run(
            [sys.executable, str(SCRIPT), "topic", "--repo", str(self.repo),
             "--owner-mainline-override", "owner", "--gate", "true", *args],
            capture_output=True, text=True, env=environment,
        )

    def remote(self):
        remote = self.root / "origin.git"
        self.git("init", "-q", "--bare", str(remote))
        self.git("remote", "add", "origin", str(remote))
        self.git("push", "-q", "origin", "main")
        return remote

    def test_unknown_owner_cannot_authorize_a_mainline_landing(self):
        result = self.call("--owner-mainline-override", "intruder")
        self.assertEqual(result.returncode, 8, result.stderr)
        self.assertIn("configured", result.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)

    def test_expected_tip_refuses_a_moved_candidate_before_gate(self):
        marker = self.root / "gate-ran"
        (self.worker / "later").write_text("later\n")
        self.git("add", "later", cwd=self.worker)
        self.git("commit", "-qm", "later", cwd=self.worker)
        result = self.call("--expected-tip", self.tip, "--gate", f"touch {marker}")
        self.assertEqual(result.returncode, 6, result.stderr)
        self.assertIn("expected tip", result.stderr)
        self.assertFalse(marker.exists())
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)

    def test_ignored_file_collision_is_refused_before_push(self):
        remote = self.remote()
        (self.repo / ".git/info/exclude").write_text("result\n")
        (self.repo / "result").write_text("local ignored work\n")
        result = self.call()
        self.assertEqual(result.returncode, 3, result.stderr)
        self.assertIn("result", result.stderr)
        self.assertEqual((self.repo / "result").read_text(), "local ignored work\n")
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)

    def test_collision_appearing_during_gate_is_refused_before_push(self):
        remote = self.remote()
        path = shlex.quote(str(self.repo / "result"))
        result = self.call("--gate", f"printf local > {path}")
        self.assertEqual(result.returncode, 3, result.stderr)
        self.assertEqual((self.repo / "result").read_text(), "local")
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)

    def test_interrupted_push_blocks_new_landing_and_recovers_exact_tip(self):
        remote = self.remote()
        result = self.call(env={"COLLAB_FAILPOINT": "after_legacy_push"})
        self.assertEqual(result.returncode, 86, result.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.tip)
        paths = list((self.repo / ".git/legacy-land").glob("*.json"))
        self.assertEqual(len(paths), 1)
        record = json.loads(paths[0].read_text())
        blocked = self.call()
        self.assertEqual(blocked.returncode, 64, blocked.stderr)
        self.assertIn("recover", blocked.stderr)
        recovered = self.call("--recover-landing", record["id"])
        self.assertEqual(recovered.returncode, 0, recovered.stderr)
        self.assertEqual(recovered.stdout.strip(), self.tip)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
        record = json.loads(paths[0].read_text())
        self.assertEqual(record["state"], "landed")
        self.assertTrue(any(event["kind"] == "mainline_override" for event in record["events"]))

    def test_gate_receipt_binds_argv_outcome_and_log_to_exact_tip(self):
        result = self.call("--expected-tip", self.tip)
        self.assertEqual(result.returncode, 0, result.stderr)
        path = next((self.repo / ".git/legacy-land").glob("*.json"))
        record = json.loads(path.read_text())
        receipts = [event for event in record["events"] if event["kind"] == "gate_receipt"]
        self.assertEqual(len(receipts), 1)
        receipt = receipts[0]
        self.assertEqual(receipt["subject"], self.tip)
        self.assertEqual(receipt["argv"], [["sh", "-c", "true"]])
        self.assertEqual(receipt["outcome"], "passed")
        self.assertTrue(Path(receipt["log"]).is_file())

    def test_gate_timeout_records_distinct_outcome_and_preserves_target(self):
        result = self.call("--gate", "sleep 10", "--gate-timeout", "0.1")
        self.assertEqual(result.returncode, 5, result.stderr)
        path = next((self.repo / ".git/legacy-land").glob("*.json"))
        receipt = next(event for event in json.loads(path.read_text())["events"] if event["kind"] == "gate_receipt")
        self.assertEqual(receipt["outcome"], "timeout")
        self.assertEqual(receipt["timeout_seconds"], 0.1)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)

    def test_gate_runs_without_holding_the_landing_lock(self):
        lock = str(self.repo / ".git/land.lock")
        code = "\n".join([
            "import fcntl",
            f"handle=open({lock!r}, 'a+')",
            "fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)",
        ])
        result = self.call("--gate", f"{shlex.quote(sys.executable)} -c {shlex.quote(code)}")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_initialized_legacy_landing_honors_busy_collab_meta_lock_before_publication(self):
        self.collab("init", "--trunk", "main", "--check-json", '["true"]')
        remote = self.remote()
        lock = self.repo / ".git/collab/meta.lock"

        with lock.open("a+") as handle:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            result = self.call()

        self.assertEqual(result.returncode, 64, result.stderr)
        self.assertIn("meta operation busy", result.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)
        self.assertFalse((self.repo / ".git/legacy-land").exists())

    def test_staged_unrelated_change_refuses_local_fast_forward_before_push(self):
        remote = self.remote()
        (self.repo / "README").write_text("staged local work\n")
        self.git("add", "README")
        result = self.call()
        self.assertEqual(result.returncode, 10, result.stderr)
        self.assertIn("staged", result.stderr)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)
        self.assertEqual(self.git("show", ":README"), "staged local work\n")

    def test_empty_cherry_pick_marker_refuses_before_push(self):
        remote = self.remote()
        cherry_pick = subprocess.run(
            ["git", "-C", str(self.repo), "cherry-pick", self.base],
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(cherry_pick.returncode, 0, cherry_pick.stderr)
        self.assertTrue((self.repo / ".git/CHERRY_PICK_HEAD").exists())
        self.assertEqual(self.git("status", "--porcelain=v1"), "")
        result = self.call()
        self.assertEqual(result.returncode, 10, result.stderr)
        self.assertIn("in-progress Git operation", result.stderr)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)

    def test_recover_after_local_move_requires_its_attributed_exact_tip(self):
        result = self.call(env={"COLLAB_FAILPOINT": "after_legacy_local"})
        self.assertEqual(result.returncode, 86, result.stderr)
        path = next((self.repo / ".git/legacy-land").glob("*.json"))
        operation = json.loads(path.read_text())
        self.assertEqual(operation["state"], "local_started")
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
        repaired = self.call("--recover-landing", operation["id"])
        self.assertEqual(repaired.returncode, 0, repaired.stderr)
        self.assertEqual(json.loads(path.read_text())["state"], "landed")

    def test_recover_refuses_unrelated_pending_legacy_landing(self):
        result = self.call(env={"COLLAB_FAILPOINT": "after_legacy_local"})
        self.assertEqual(result.returncode, 86, result.stderr)
        path = next((self.repo / ".git/legacy-land").glob("*.json"))
        operation = json.loads(path.read_text())
        other = self.repo / ".git/legacy-land/other-landing.json"
        other.write_text(json.dumps({
            "id": "other-landing",
            "state": "pushed",
            "events": [],
        }))

        repaired = self.call("--recover-landing", operation["id"])

        self.assertEqual(repaired.returncode, 64, repaired.stderr)
        self.assertIn("Unresolved legacy landing other-landing", repaired.stderr)
        self.assertEqual(json.loads(path.read_text())["state"], "local_started")

    def test_recover_repairs_attributed_local_success_while_remote_offline(self):
        remote = self.remote()
        result = self.call(env={"COLLAB_FAILPOINT": "after_legacy_local"})
        self.assertEqual(result.returncode, 86, result.stderr)
        path = next((self.repo / ".git/legacy-land").glob("*.json"))
        operation = json.loads(path.read_text())
        self.assertEqual(operation["state"], "local_started")
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.tip)
        remote.rename(remote.with_name("origin-offline.git"))

        repaired = self.call("--recover-landing", operation["id"])

        self.assertEqual(repaired.returncode, 0, repaired.stderr)
        self.assertEqual(json.loads(path.read_text())["state"], "landed")
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)

    def test_push_rejection_reports_actual_cause_without_a_stale_candidate_claim(self):
        remote = self.remote()
        hook = remote / "hooks/pre-receive"
        hook.write_text("#!/bin/sh\necho remote-policy-denied >&2\nexit 1\n")
        hook.chmod(0o755)
        result = self.call()
        self.assertEqual(result.returncode, 6, result.stderr)
        self.assertIn("push failed", result.stderr)
        self.assertIn("remote-policy-denied", result.stderr)
        self.assertNotIn("new Candidate on the new base", result.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)

    def test_help_is_deprecated_and_standalone_landing_never_loads_effort_module(self):
        help_result = self.call("--help")
        self.assertEqual(help_result.returncode, 0)
        self.assertIn("Deprecated", help_result.stdout)
        code = "\n".join([
            "import sys",
            f"sys.path.insert(0, {str(ROOT / 'tools/agents')!r})",
            "import land",
            f"result=land.main(['topic', '--repo', {str(self.repo)!r}, "
            "'--owner-mainline-override', 'owner', '--gate', 'true'])",
            "assert 'effort_records' not in sys.modules",
            "sys.exit(result)",
        ])
        result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_in_tree_effort_tracker_does_not_load_record_branch_extension(self):
        path = self.repo / "docs/efforts/new/EFFORT.md"
        path.parent.mkdir(parents=True)
        path.write_text("# Effort: new\n\nBase: main\nIntegration branch: main\n")
        (path.parent / "tickets").mkdir()
        code = "\n".join([
            "import sys",
            f"sys.path.insert(0, {str(ROOT / 'tools/agents')!r})",
            "import tracker",
            f"result=tracker.main(['--repo', {str(self.repo)!r}, 'list', 'new'])",
            "assert 'effort_records' not in sys.modules",
            "sys.exit(result)",
        ])
        result = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def record_effort(self):
        effort = self.worker / "docs/efforts/sample"
        (effort / "tickets").mkdir(parents=True)
        (effort / "evidence").mkdir()
        (effort / "EFFORT.md").write_text(
            "# Effort: sample\n\nStatus: active\nBase: main\nIntegration branch: main\n"
            "Record branch: effort/sample-records\nLead: lead\n"
        )
        (effort / "tickets/01-one.md").write_text(
            "# 01 one: One\n\nType: feature\nStatus: claimed\nOwner: worker-01\nBlocked by:\n"
            "Claimed: 2026-10-05T10:00:00Z expires 2026-10-06T10:00:00Z\n\n"
            "## Acceptance\n- [ ] works\n\n## Comments\n"
        )
        self.git("add", "docs", cwd=self.worker)
        self.git("commit", "-qm", "effort records", cwd=self.worker)
        self.tip = self.git("rev-parse", "topic").strip()
        records = self.root / "records"
        self.git("worktree", "add", "-q", "-b", "effort/sample-records", str(records), self.tip)
        path = records / "docs/efforts/sample/evidence/acceptance-01.jsonl"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps({
            "schema": "corenius.acceptance.v1", "effort": "sample", "ticket": "01", "candidate": self.tip,
            "reviewer": "lead", "reviewer_context": "lead", "axes": {"standards": "passed", "spec": "passed"},
            "required_checks": [{"name": "unit", "result": "passed"}], "owner_waiver": None, "accepted": True,
        }) + "\n")
        self.git("add", str(path), cwd=records)
        self.git("commit", "-qm", "acceptance", cwd=records)
        return records

    def test_post_landing_record_failure_is_partial_success_and_repair_is_bound(self):
        records = self.record_effort()
        evidence = records / "docs/efforts/sample/evidence"
        retained = records / "docs/efforts/sample/evidence-retained"
        gate = f"mv {shlex.quote(str(evidence))} {shlex.quote(str(retained))}; touch {shlex.quote(str(evidence))}"
        result = self.call("--effort", "sample", "--ticket", "01", "--gate", gate)
        self.assertEqual(result.returncode, 12, result.stderr)
        self.assertIn("landed", result.stderr)
        self.assertIn("records incomplete", result.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
        path = next((self.repo / ".git/legacy-land").glob("*.json"))
        operation = json.loads(path.read_text())
        self.assertEqual(operation["state"], "landed_records_incomplete")
        evidence.unlink()
        retained.rename(evidence)
        repaired = self.call("--recover-landing", operation["id"])
        self.assertEqual(repaired.returncode, 0, repaired.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
        self.assertEqual(json.loads(path.read_text())["state"], "landed")
        self.assertTrue((evidence / "landing-01.jsonl").is_file())

    def collab(self, *args):
        result = subprocess.run(
            [sys.executable, str(ROOT / "tools/collab/collab.py"), "--repo", str(self.repo), *args],
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)

    def test_registered_attempt_cannot_bypass_lifecycle_through_legacy_route(self):
        self.collab("init", "--trunk", "main", "--check-json", '["true"]')
        self.collab("new", "01", "One")
        self.git("add", ".tickets")
        self.git("commit", "-qm", "ticket")
        self.git("branch", "staging")
        attempt = self.collab("open-attempt", "01", "--owner", "worker")
        checkout = Path(attempt["path"])
        (checkout / "new-result").write_text("candidate\n")
        self.git("add", "new-result", cwd=checkout)
        self.git("commit", "-qm", "work", cwd=checkout)
        before = self.git("rev-parse", "staging").strip()
        result = subprocess.run(
            [sys.executable, str(SCRIPT), attempt["branch"], "--repo", str(self.repo),
             "--mainline", "staging", "--gate", "true", "--no-push"],
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 64, result.stderr)
        self.assertIn("collab lifecycle", result.stderr)
        self.assertEqual(self.git("rev-parse", "staging").strip(), before)


if __name__ == "__main__":
    unittest.main()

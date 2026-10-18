"""Refresh and ticket-source contracts exercised through the public collab CLI."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

CLI = Path(__file__).resolve().parents[1] / "collab.py"
sys.path.insert(0, str(CLI.parents[1]))
from temp_root import temporary_root


class RefreshSourceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="refresh-source-", dir=temporary_root())
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.env = dict(
            os.environ,
            GIT_CONFIG_NOSYSTEM="1",
            GIT_CONFIG_GLOBAL=os.devnull,
            GIT_AUTHOR_NAME="Test",
            GIT_AUTHOR_EMAIL="test@example.invalid",
            GIT_COMMITTER_NAME="Test",
            GIT_COMMITTER_EMAIL="test@example.invalid",
        )
        self.git("init", "-b", "main")
        (self.repo / "code.txt").write_text("base\n")
        self.commit("initial")

    def git(self, *args, cwd=None):
        result = subprocess.run(["git", *args], cwd=cwd or self.repo, env=self.env,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout.strip()

    def commit(self, message, cwd=None):
        self.git("add", "--all", cwd=cwd)
        self.git("commit", "-m", message, cwd=cwd)
        return self.git("rev-parse", "HEAD", cwd=cwd)

    def cli(self, *args, ok=True):
        result = subprocess.run([sys.executable, str(CLI), "--repo", str(self.repo), *args],
                                cwd=self.repo, env=self.env, capture_output=True, text=True)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr)
            return json.loads(result.stdout)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def init(self, source=None):
        args = ["init", "--trunk", "main", "--check-json", '["true"]',
                "--workspace-root", str(self.root / "workers")]
        if source:
            args.extend(["--ticket-source", source])
        return self.cli(*args)

    def ticket(self, tid="T-one", deps=(), status="open"):
        path = self.repo / ".tickets" / (tid + ".md")
        path.parent.mkdir(exist_ok=True)
        path.write_text(f"---\nid: {tid}\nstatus: {status}\ndeps: [{', '.join(deps)}]\n---\n\n# {tid}\n")
        return path

    def open(self, ticket="T-one", *args):
        return self.cli("open-attempt", ticket, "--owner", "worker", *args)

    def effort_ticket(self, tid="01", status="open", blockers="", owner=""):
        directory = self.repo / "docs/efforts/sample/tickets"
        directory.mkdir(parents=True, exist_ok=True)
        (directory.parent / "EFFORT.md").write_text("# Sample\n\nBase: main\nIntegration branch: main\n")
        path = directory / f"{tid}-work.md"
        path.write_text(
            f"# {tid} work: Do work\n\nType: feature\nStatus: {status}\nOwner: {owner}\n"
            f"Blocked by: {blockers}\nClaimed:\n\nTriage: ready-for-agent\n\n"
            "## What to build\nDo work.\n\n## Acceptance\n- [ ] observable result\n\n## Comments\n"
        )
        return path

    def test_corenius_source_lists_tracker_tickets_and_opens_qualified_identity(self):
        self.init("corenius")
        self.effort_ticket()
        self.effort_ticket("02", blockers="01")
        self.effort_ticket("03", status="resolved")
        self.ticket("T-hidden")
        self.commit("both layouts")
        rows = {row["id"]: row for row in self.cli("status")["tickets"]}
        self.assertEqual(set(rows), {"sample:01", "sample:02", "sample:03"})
        self.assertEqual([row["id"] for row in self.cli("ready")], ["sample:01"])
        self.assertIn("01", rows["sample:02"]["blocker"])
        self.assertFalse(rows["sample:03"]["ready"])
        attempt = self.open("sample:01")
        self.assertEqual(attempt["ticket"], "sample:01")
        ticket = self.repo / "docs/efforts/sample/tickets/01-work.md"
        self.assertIn("Status: claimed", ticket.read_text())
        self.assertIn("Owner: worker", ticket.read_text())
        rows = {row["id"]: row for row in self.cli("status")["tickets"]}
        self.assertEqual(rows["sample:01"]["active_attempt"], attempt["id"])
        self.assertFalse(rows["sample:01"]["ready"])
        refused = self.cli("open-attempt", "T-hidden", "--owner", "worker", ok=False)
        self.assertIn("effort", refused.stderr.lower())

    def test_corenius_source_directs_ticket_creation_to_the_tracker(self):
        self.init("corenius")
        result = self.cli("new", "T-one", "Work", ok=False)
        self.assertIn("tracker.py", result.stderr)
        self.assertFalse((self.repo / ".tickets").exists())

    def test_corenius_claim_refuses_foreign_integration_before_touching_other_worktree(self):
        self.init("corenius")
        ticket = self.effort_ticket()
        effort = ticket.parent.parent / "EFFORT.md"
        effort.write_text("# Sample\n\nBase: main\nIntegration branch: foreign-owner\n")
        self.commit("effort routed to foreign owner")
        foreign = self.root / "foreign"
        self.git("worktree", "add", "-b", "foreign-owner", str(foreign), "HEAD")
        foreign_ticket = foreign / ticket.relative_to(self.repo)
        original = foreign_ticket.read_bytes()
        result = self.cli("open-attempt", "01", "--effort", "sample", "--owner", "worker", ok=False)
        self.assertIn("optional effort-record", result.stderr)
        self.assertEqual(foreign_ticket.read_bytes(), original)
        self.assertEqual(self.git("status", "--porcelain", cwd=foreign), "")
        self.assertEqual(ticket.read_bytes(), original)
        self.assertEqual(self.cli("status")["attempts"], [])

    def test_corenius_claim_refuses_record_branch_before_touching_record_worktree(self):
        self.init("corenius")
        ticket = self.effort_ticket()
        effort = ticket.parent.parent / "EFFORT.md"
        effort.write_text("# Sample\n\nBase: main\nIntegration branch: main\nRecord branch: foreign-records\n")
        self.commit("effort records owned elsewhere")
        foreign = self.root / "foreign"
        self.git("worktree", "add", "-b", "foreign-records", str(foreign), "HEAD")
        foreign_ticket = foreign / ticket.relative_to(self.repo)
        original = foreign_ticket.read_bytes()
        result = self.cli("open-attempt", "sample:01", "--owner", "worker", ok=False)
        self.assertIn("optional effort-record", result.stderr)
        self.assertEqual(foreign_ticket.read_bytes(), original)
        self.assertEqual(self.git("status", "--porcelain", cwd=foreign), "")
        self.assertEqual(self.cli("status")["attempts"], [])

    def test_standalone_source_is_selected_at_init_and_runs_dependency_lifecycle(self):
        config = self.init("standalone")
        self.assertEqual(config["ticket_source"], "standalone")
        self.ticket("T-dep")
        self.ticket(deps=["T-dep"])
        self.commit("dependent tickets")
        self.assertEqual([row["id"] for row in self.cli("ready")], ["T-dep"])
        blocked = self.cli("open-attempt", "T-one", "--owner", "worker", ok=False)
        self.assertIn("Dependency not closed", blocked.stderr)
        dependency = self.open("T-dep")
        path = Path(dependency["path"])
        (path / "dep.txt").write_text("dependency complete\n")
        self.commit("dependency", cwd=path)
        self.cli("submit", dependency["id"])
        self.cli("check", dependency["id"])
        self.cli("land", dependency["id"], "--no-push")
        self.assertEqual([row["id"] for row in self.cli("ready")], ["T-one"])
        attempt = self.open()
        self.assertEqual(attempt["ticket"], "T-one")
        self.assertEqual(self.cli("ready"), [])

    def test_refresh_replays_only_contribution_onto_directly_moved_trunk_and_drops_old_receipt(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        old = self.open()
        path = Path(old["path"])
        (path / "one.txt").write_text("first\n")
        first = self.commit("first contribution", cwd=path)
        (path / "two.txt").write_text("second\n")
        tip = self.commit("second contribution", cwd=path)
        self.cli("submit", old["id"])
        checked = self.cli("check", old["id"])
        (self.repo / "direct.txt").write_text("direct landing\n")
        trunk = self.commit("another tool lands")
        fresh = self.cli("refresh", old["id"], "--confirm-stopped")
        self.assertNotEqual(fresh["id"], old["id"])
        self.assertEqual(fresh["base"], trunk)
        self.assertEqual(fresh["state"], "working")
        self.assertIsNone(fresh["check_receipt"])
        self.assertIsNone(fresh["candidate"])
        self.assertIsNone(fresh["integration"])
        self.assertEqual(self.cli("show", old["id"])["state"], "abandoned")
        self.assertEqual(self.cli("show", old["id"])["check_receipt"], checked["check_receipt"])
        self.assertEqual(self.git("rev-parse", old["branch"]), tip)
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=path), tip)
        new_path = Path(fresh["path"])
        self.assertEqual((new_path / "one.txt").read_text(), "first\n")
        self.assertEqual((new_path / "two.txt").read_text(), "second\n")
        self.assertEqual((new_path / "direct.txt").read_text(), "direct landing\n")
        self.assertEqual(self.git("log", "--format=%s", f"{trunk}..HEAD", cwd=new_path).splitlines(),
                         ["second contribution", "first contribution"])
        self.assertNotEqual(self.git("rev-parse", "HEAD~1", cwd=new_path), first)
        self.cli("land", fresh["id"], ok=False)

    def test_refresh_conflict_reports_failure_and_preserves_first_conflict_and_unapplied_suffix(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        old = self.open()
        path = Path(old["path"])
        (path / "prefix.txt").write_text("clean prefix\n")
        self.commit("clean prefix", cwd=path)
        (path / "code.txt").write_text("worker change\n")
        conflict = self.commit("conflicting contribution", cwd=path)
        (path / "suffix.txt").write_text("must not apply\n")
        old_tip = self.commit("unapplied suffix", cwd=path)
        (self.repo / "code.txt").write_text("trunk change\n")
        self.commit("direct conflicting landing")
        result = self.cli("refresh", old["id"], "--confirm-stopped", ok=False)
        attempts = self.cli("status")["attempts"]
        fresh = next(row for row in attempts if row["id"] != old["id"])
        new_path = Path(fresh["path"])
        self.assertEqual(result.returncode, 2)
        self.assertIn(fresh["id"], result.stderr)
        self.assertIn(conflict, result.stderr)
        self.assertEqual(self.git("rev-parse", "CHERRY_PICK_HEAD", cwd=new_path), conflict)
        self.assertIn("<<<<<<<", (new_path / "code.txt").read_text())
        self.assertEqual((new_path / "prefix.txt").read_text(), "clean prefix\n")
        self.assertFalse((new_path / "suffix.txt").exists())
        self.assertEqual(self.git("log", "-1", "--format=%s", cwd=new_path), "clean prefix")
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=path), old_tip)
        self.cli("submit", fresh["id"], ok=False)

    def test_corenius_refresh_transfers_owned_tracker_claim_to_new_attempt(self):
        self.init("corenius")
        ticket = self.effort_ticket()
        self.commit("ticket")
        old = self.open("sample:01")
        claimed = ticket.read_bytes()
        path = Path(old["path"])
        (path / "code.txt").write_text("worker change\n")
        tip = self.commit("worker change", cwd=path)
        (self.repo / "direct.txt").write_text("trunk movement\n")
        trunk = self.commit("direct landing")
        fresh = self.cli("refresh", old["id"], "--confirm-stopped")
        self.assertEqual(fresh["ticket"], "sample:01")
        self.assertEqual(fresh["owner"], "worker")
        self.assertEqual(fresh["base"], trunk)
        self.assertEqual(ticket.read_bytes(), claimed)
        self.assertEqual(self.git("rev-parse", old["branch"]), tip)
        self.assertEqual((Path(fresh["path"]) / "code.txt").read_text(), "worker change\n")
        self.assertEqual(self.cli("show", old["id"])["state"], "abandoned")
        self.cli("submit", fresh["id"])

    def test_refresh_excludes_trunk_commits_already_present_in_worker_history(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        old = self.open()
        path = Path(old["path"])
        (self.repo / "direct.txt").write_text("direct commit\n")
        trunk = self.commit("direct landing")
        self.git("merge", "--ff-only", trunk, cwd=path)
        (path / "worker.txt").write_text("worker contribution\n")
        self.commit("only worker contribution", cwd=path)
        fresh = self.cli("refresh", old["id"], "--confirm-stopped")
        new_path = Path(fresh["path"])
        self.assertEqual(self.git("log", "--format=%s", f"{trunk}..HEAD", cwd=new_path),
                         "only worker contribution")
        self.assertEqual((new_path / "direct.txt").read_text(), "direct commit\n")

    def test_refresh_refuses_missing_stop_confirmation_dirty_work_and_changed_contract_before_abandon(self):
        self.init()
        ticket = self.ticket()
        self.commit("ticket")
        old = self.open()
        path = Path(old["path"])
        (path / "worker.txt").write_text("committed contribution\n")
        tip = self.commit("contribution", cwd=path)
        result = self.cli("refresh", old["id"], ok=False)
        self.assertIn("confirm-stopped", result.stderr)
        (path / "worker.txt").write_text("unsaved contribution\n")
        result = self.cli("refresh", old["id"], "--confirm-stopped", ok=False)
        self.assertIn("not clean", result.stderr)
        self.assertEqual((path / "worker.txt").read_text(), "unsaved contribution\n")
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=path), tip)
        tip = self.commit("preserve unsaved contribution", cwd=path)
        ticket.write_text(ticket.read_text() + "\nChanged acceptance contract.\n")
        self.commit("changed contract")
        result = self.cli("refresh", old["id"], "--confirm-stopped", ok=False)
        self.assertIn("Ticket changed", result.stderr)
        self.assertEqual(self.cli("show", old["id"])["state"], "working")
        self.assertEqual(self.git("rev-parse", old["branch"]), tip)
        self.assertEqual(len(self.cli("status")["attempts"]), 1)


if __name__ == "__main__":
    unittest.main()

"""Public CLI proof using real disposable Git repositories and process death."""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import tokenize
import unittest

CLI = Path(__file__).resolve().parents[1] / "collab.py"
sys.path.insert(0, str(CLI.parents[1]))
from temp_root import temporary_root

SYSTEM_TMP = str(Path("/") / "tmp")
COLLAB_ROOT = CLI.parent


class CollabTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="collab-test-", dir=temporary_root())
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "project"
        self.repo.mkdir()
        self.env = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                        GIT_AUTHOR_NAME="Test", GIT_AUTHOR_EMAIL="test@example.invalid",
                        GIT_COMMITTER_NAME="Test", GIT_COMMITTER_EMAIL="test@example.invalid")
        self.git("init", "-b", "main")
        self.git("config", "user.name", "Collab Test")
        self.git("config", "user.email", "collab@example.invalid")
        (self.repo / "code.txt").write_text("base\n")
        (self.repo / ".gitignore").write_text("build/\n")
        self.commit("initial")

    def run_process(self, argv, cwd=None, env=None):
        return subprocess.run(argv, cwd=cwd or self.repo, env=env or self.env,
                              text=True, capture_output=True)

    def git(self, *args, cwd=None, ok=True):
        result = self.run_process(["git", *args], cwd=cwd)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout.strip()

    def commit(self, message, cwd=None):
        self.git("add", "--all", cwd=cwd)
        self.git("commit", "-m", message, cwd=cwd)
        return self.git("rev-parse", "HEAD", cwd=cwd)

    def cli(self, *args, repo=None, ok=True, failpoint=None, env=None):
        child_env = dict(env or self.env)
        if failpoint:
            child_env["COLLAB_FAILPOINT"] = failpoint
        result = self.run_process([sys.executable, str(CLI), "--repo", str(repo or self.repo),
                                   *args], env=child_env)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr)
            return json.loads(result.stdout)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def init(self, argv=None, protected_paths=None, timeout_seconds=None):
        args = [
            "init",
            "--trunk",
            "main",
            "--check-json",
            json.dumps(argv or [sys.executable, "-c", "print('checked')"]),
            "--workspace-root",
            str(self.root / "workspaces"),
        ]
        if protected_paths is not None:
            args.extend(["--protected-paths-json", json.dumps(protected_paths)])
        if timeout_seconds is not None:
            args.extend(["--check-timeout-seconds", str(timeout_seconds)])
        return self.cli(*args)

    def ticket(self, tid="T-one", deps=(), status="open", crlf=False):
        path = self.repo / ".tickets" / (tid + ".md")
        path.parent.mkdir(exist_ok=True)
        raw = (
            f"---\nid: {tid}\nstatus: {status}\ndeps: [{', '.join(deps)}]\n---\n\n"
            f"# {tid}\n\nDo the thing.\n"
        ).encode()
        path.write_bytes(raw.replace(b"\n", b"\r\n") if crlf else raw)
        return path

    def effort_ticket(
        self,
        tid="01",
        slug="do-work",
        status="open",
        owner="",
        blocked_by="",
        claimed="",
        effort="sample-effort",
    ):
        directory = self.repo / "docs" / "efforts" / effort / "tickets"
        directory.mkdir(parents=True, exist_ok=True)
        (directory.parent / "EFFORT.md").write_text(
            f"# {effort}\n\n"
            "Base: main\n"
            "Integration branch: main\n"
        )
        path = directory / f"{tid}-{slug}.md"
        path.write_text(
            f"# {tid} {slug}: Do work\n\n"
            "Type: feature\n"
            f"Status: {status}\n"
            f"Owner: {owner}\n"
            f"Blocked by: {blocked_by}\n"
            f"Claimed: {claimed}\n\n"
            "Triage: ready-for-agent\n\n"
            "## What to build\n"
            "Do the thing.\n\n"
            "## Acceptance\n"
            "- [ ] observable result\n\n"
            "## Comments\n"
        )
        return path

    def claim(self, tid="T-one"):
        return self.cli("open-attempt", tid, "--owner", "test-worker")

    def open_corenius_attempt(self, tid="01", owner="agent-a", effort="sample-effort", failpoint=None, ok=True):
        env = dict(self.env, CORENIUS_TRACKER_NOW="2026-10-12T00:00:00Z")
        return self.cli("open-attempt", tid, "--effort", effort, "--owner", owner,
                        failpoint=failpoint, env=env, ok=ok)

    def contribute(self, attempt, content="contribution\n"):
        path = Path(attempt["path"])
        (path / "code.txt").write_text(content)
        return self.commit("worker contribution", cwd=path)

    def unlock_worktree(self, attempt):
        self.git("worktree", "unlock", attempt["path"])

    def prepared(self, argv=None, timeout_seconds=None):
        self.init(argv, timeout_seconds=timeout_seconds)
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        self.contribute(a)
        return self.cli("submit", a["id"])

    def test_checked_commit_and_ticket_closure_land_together(self):
        a = self.prepared()
        checked = self.cli("check", a["id"])
        self.assertEqual(checked["state"], "verified")
        self.assertEqual(self.git("rev-parse", "HEAD"), a["base"])
        landed = self.cli("land", a["id"])
        self.assertEqual(landed["state"], "landed")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])
        self.assertEqual(self.git("rev-parse", "HEAD^"), a["base"])
        self.assertIn("status: closed", (self.repo / ".tickets/T-one.md").read_text())
        self.assertEqual((self.repo / "code.txt").read_text(), "contribution\n")

    def test_init_rejects_non_array_check_argv(self):
        for value in ('"true"', '{"true": "value"}', '[]', '[""]', '[1]', '["bad\\u0000arg"]'):
            with self.subTest(value=value):
                result = self.cli("init", "--trunk", "main", "--check-json", value, ok=False)
                self.assertIn("argv", result.stderr)
                self.assertFalse((self.repo / ".git/collab/state.sqlite3").exists())

    def test_init_rejects_nonpositive_check_timeout(self):
        for value in ("0", "-1", "nan", "inf"):
            with self.subTest(value=value):
                result = self.cli(
                    "init",
                    "--trunk",
                    "main",
                    "--check-json",
                    '["true"]',
                    "--check-timeout-seconds",
                    value,
                    ok=False,
                )
                self.assertIn("timeout", result.stderr)
                self.assertFalse((self.repo / ".git/collab/state.sqlite3").exists())

    def test_claim_hashes_exact_crlf_ticket_and_check_preserves_line_endings(self):
        self.init()
        ticket = self.ticket(crlf=True)
        original = ticket.read_bytes()
        self.commit("CRLF ticket")
        a = self.claim()
        self.assertEqual(a["ticket_hash"], hashlib.sha256(original).hexdigest())
        self.contribute(a)
        self.cli("submit", a["id"])
        checked = self.cli("check", a["id"])
        self.assertEqual((Path(checked["check_path"]) / ".tickets/T-one.md").read_bytes(),
                         original.replace(b"status: open", b"status: closed"))
        self.assertEqual(checked["check_receipt"]["outcome"], "passed")

    def test_check_death_before_receipt_commit_cannot_publish_verified(self):
        a = self.prepared()
        result = self.cli("check", a["id"], failpoint="before_check_receipt_commit", ok=False)
        self.assertEqual(result.returncode, 86)
        current = self.cli("show", a["id"])
        self.assertEqual(current["state"], "checking")
        self.assertIsNone(current["check_receipt"])
        self.cli("land", a["id"], ok=False)
        self.assertEqual(self.cli("recover", a["id"], "--confirm-stopped")["state"], "check_failed")

    def test_check_receipt_is_durable_with_verified_state(self):
        a = self.prepared()
        result = self.cli("check", a["id"], failpoint="after_check_receipt_commit", ok=False)
        self.assertEqual(result.returncode, 86)
        checked = self.cli("show", a["id"])
        self.assertEqual(checked["state"], "verified")
        receipt = checked["check_receipt"]
        self.assertEqual(receipt["version"], 2)
        self.assertEqual(receipt["outcome"], "passed")
        self.assertEqual(receipt["subject"], checked["integration"])
        self.assertEqual(receipt["base"], checked["base"])
        self.assertEqual(receipt["log_sha256"], hashlib.sha256(Path(checked["log"]).read_bytes()).hexdigest())
        self.assertTrue(receipt["passed"])
        self.assertEqual(self.cli("land", a["id"])["state"], "landed")

    def test_prepared_landing_at_base_can_recover_verified(self):
        a = self.prepared()
        self.cli("check", a["id"])
        result = self.cli("land", a["id"], failpoint="before_git_land", ok=False)
        self.assertEqual(result.returncode, 86)
        self.assertEqual(self.cli("show", a["id"])["state"], "landing_prepared")
        self.cli("recover", a["id"], ok=False)
        self.assertEqual(self.cli("recover", a["id"], "--confirm-stopped")["state"], "verified")
        self.assertEqual(self.cli("land", a["id"])["state"], "landed")

    def test_started_landing_without_attribution_stays_ambiguous_at_base(self):
        a = self.prepared()
        self.cli("check", a["id"])
        result = self.cli("land", a["id"], failpoint="after_land_started", ok=False)
        self.assertEqual(result.returncode, 86)
        self.assertEqual(self.cli("show", a["id"])["state"], "landing_started")
        self.assertEqual(self.git("rev-parse", "HEAD"), a["base"])
        result = self.cli("recover", a["id"], "--confirm-stopped", ok=False)
        self.assertIn("Ambiguous", result.stderr)
        self.cli("abandon", a["id"], "--confirm-stopped", ok=False)
        self.cli("land", a["id"], ok=False)

    def test_every_unresolved_landing_phase_blocks_other_check_and_land(self):
        self.init()
        self.ticket()
        self.ticket("T-two")
        self.commit("two tickets")
        first, second = self.claim(), self.claim("T-two")
        for a in (first, second):
            self.contribute(a, a["id"])
            self.cli("submit", a["id"])
            self.cli("check", a["id"])
        self.cli("land", first["id"], failpoint="before_git_land", ok=False)
        for command in ("check", "land"):
            result = self.cli(command, second["id"], ok=False)
            self.assertIn("Unresolved landing", result.stderr)
        self.cli("recover", first["id"], "--confirm-stopped")
        self.cli("land", first["id"], failpoint="after_land_started", ok=False)
        for command in ("check", "land"):
            result = self.cli(command, second["id"], ok=False)
            self.assertIn("Unresolved landing", result.stderr)

    def test_direct_cli_rejects_unsafe_temp_and_output_roots(self):
        for value in (SYSTEM_TMP, "relative", "/"):
            with self.subTest(value=value):
                result = self.cli("init", "--trunk", "main", "--check-json", '["true"]',
                                  env=dict(self.env, TMPDIR=value), ok=False)
                self.assertIn("TMPDIR", result.stderr)
                self.assertFalse((self.repo / ".git/collab/state.sqlite3").exists())
        result = self.cli("init", "--trunk", "main", "--check-json", '["true"]',
                          "--workspace-root", SYSTEM_TMP + "/collab-must-not-create", ok=False)
        self.assertIn("temporary", result.stderr)
        self.init()
        self.cli("backup", SYSTEM_TMP + "/collab-must-not-create.sqlite3", ok=False)

    def test_open_attempt_uses_agent_topic_branch_and_child_inherits_safe_temp(self):
        code = ("import os,tempfile; from pathlib import Path; p=Path(tempfile.gettempdir()); "
                "assert p==Path(os.environ['TMPDIR']); "
                f"assert not p.is_relative_to(Path({SYSTEM_TMP!r})); print(p)")
        a = self.prepared([sys.executable, "-c", code])
        self.assertTrue(a["branch"].startswith("test-worker/"))
        self.assertEqual(self.cli("check", a["id"])["state"], "verified")

    def substitute_workspace(self, attempt):
        path = Path(attempt["path"])
        path.rename(path.with_name(path.name + "-preserved"))
        path.mkdir()
        self.git("init", "-b", attempt["branch"], cwd=path)
        (path / "unrelated.txt").write_text("valuable\n")
        self.commit("unrelated repository", cwd=path)
        return path

    def test_submit_rejects_other_repository_at_assigned_path(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        path = self.substitute_workspace(a)
        result = self.cli("submit", a["id"], ok=False)
        self.assertIn("identity", result.stderr)
        self.assertEqual((path / "unrelated.txt").read_text(), "valuable\n")

    def test_gc_rejects_substituted_repository_before_unlocking(self):
        a = self.prepared()
        self.cli("abandon", a["id"], "--confirm-stopped")
        path = self.substitute_workspace(a)
        result = self.cli("gc", a["id"], "--confirm-stopped", ok=False)
        self.assertIn("identity", result.stderr)
        self.assertTrue(path.exists())
        self.assertIn("locked collab:", self.git("worktree", "list", "--porcelain"))

    def test_gc_refuses_changed_check_head(self):
        a = self.prepared()
        checked = self.cli("check", a["id"])
        self.cli("abandon", a["id"], "--confirm-stopped")
        path = Path(checked["check_path"])
        self.git("commit", "--allow-empty", "-m", "check changed after verification", cwd=path)
        result = self.cli("gc", a["id"], "--confirm-stopped", ok=False)
        self.assertIn("identity", result.stderr)
        self.assertTrue(Path(a["path"]).exists())
        self.assertTrue(path.exists())

    def test_allocation_adoption_requires_registered_exact_base_head(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        self.cli("open-attempt", "T-one", "--owner", "worker", failpoint="after_worktree_add", ok=False)
        a = self.cli("status")["attempts"][0]
        self.git("commit", "--allow-empty", "-m", "unexpected moved HEAD", cwd=Path(a["path"]))
        result = self.cli("recover", a["id"], "--confirm-stopped", ok=False)
        self.assertIn("identity", result.stderr)
        self.assertEqual(self.cli("show", a["id"])["state"], "allocating")

    def test_racing_claims_have_one_durable_winner(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        processes = [subprocess.Popen([sys.executable, str(CLI), "--repo", str(self.repo),
                                      "open-attempt", "T-one", "--owner", owner], env=self.env,
                                     stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                     for owner in ("worker-a", "worker-b")]
        results = [(process.wait(), *process.communicate()) for process in processes]
        self.assertEqual(sum(code == 0 for code, _, _ in results), 1, results)
        attempts = self.cli("status")["attempts"]
        self.assertEqual(len(attempts), 1)
        self.assertEqual(attempts[0]["state"], "working")
        self.assertIn("locked collab:" + attempts[0]["id"], self.git("worktree", "list", "--porcelain"))

    def test_new_is_not_dispatchable_until_committed_and_worktrees_share_claims(self):
        self.init()
        authored = self.cli("new", "T-one", "An authored ticket")
        self.assertTrue(Path(authored["path"]).is_file())
        self.assertEqual(self.cli("ready"), [])
        self.cli("open-attempt", "T-one", "--owner", "worker", ok=False)
        self.commit("authored ticket")
        self.assertEqual([ticket["id"] for ticket in self.cli("ready")], ["T-one"])
        a = self.claim()
        self.assertEqual(self.cli("show", a["id"], repo=Path(a["path"]))["id"], a["id"])
        self.assertEqual(self.cli("ready", repo=Path(a["path"])), [])
        self.cli("open-attempt", "T-one", "--owner", "second", repo=Path(a["path"]), ok=False)
        self.cli("init", "--trunk", "main", "--check-json", '["true"]', ok=False)

    def test_old_claim_command_is_renamed_to_open_attempt(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        result = self.cli("claim", "T-one", "--owner", "worker", ok=False)
        self.assertIn("invalid choice", result.stderr)
        self.assertEqual(self.cli("status")["attempts"], [])

    def test_open_attempt_claims_corenius_ticket_and_creates_worktree(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("tracker ticket")
        a = self.open_corenius_attempt()
        self.assertEqual(a["ticket"], "sample-effort:01")
        self.assertEqual(a["owner"], "agent-a")
        self.assertEqual(a["state"], "working")
        self.assertTrue(a["branch"].startswith("agent-a/"))
        self.assertEqual(self.git("branch", "--show-current", cwd=Path(a["path"])), a["branch"])
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=Path(a["path"])), a["base"])
        claimed = ticket.read_text()
        self.assertIn("Status: claimed", claimed)
        self.assertIn("Owner: agent-a", claimed)
        self.assertIn("Claimed: 2026-10-12T00:00:00Z expires 2026-10-13T00:00:00Z", claimed)
        self.assertIn("agent-a: claimed until 2026-10-13T00:00:00Z", claimed)
        self.assertEqual(self.git("status", "--porcelain=v1"), "")
        self.assertEqual(self.git("rev-parse", "HEAD"), a["base"])
        self.assertEqual(a["ticket_hash"], hashlib.sha256(ticket.read_bytes()).hexdigest())

    def test_open_attempt_refuses_unready_corenius_tracker_tickets(self):
        self.init()
        self.effort_ticket("01", "blocker")
        self.effort_ticket("02", "blocked", blocked_by="01")
        self.effort_ticket(
            "03",
            "claimed",
            status="claimed",
            owner="other",
            claimed="2026-10-12T00:00:00Z expires 2026-10-13T00:00:00Z",
        )
        self.effort_ticket("04", "resolved", status="resolved", owner="done")
        self.commit("tracker tickets")
        for tid, reason in (("02", "blocked"), ("03", "claimed"), ("04", "resolved")):
            with self.subTest(ticket=tid):
                result = self.open_corenius_attempt(tid, owner="agent-a", ok=False)
                self.assertIn(reason, result.stderr)
                self.assertEqual(self.cli("status")["attempts"], [])

    def test_open_attempt_recover_completes_corenius_allocation_after_tracker_claim(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("tracker ticket")
        result = self.open_corenius_attempt(failpoint="after_tracker_claim", ok=False)
        self.assertEqual(result.returncode, 86)
        a = self.cli("status")["attempts"][0]
        self.assertEqual(a["state"], "allocating")
        self.assertIn("Status: claimed", ticket.read_text())
        self.assertFalse(Path(a["path"]).exists())
        recovered = self.cli("recover", a["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "working")
        self.assertTrue(Path(recovered["path"]).is_dir())
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=Path(recovered["path"])), recovered["base"])

    def test_open_attempt_recover_claims_corenius_ticket_after_state_record_crash(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("tracker ticket")
        result = self.open_corenius_attempt(failpoint="after_claim_record", ok=False)
        self.assertEqual(result.returncode, 86)
        a = self.cli("status")["attempts"][0]
        self.assertEqual(a["state"], "allocating")
        self.assertIn("Status: open", ticket.read_text())
        recovered = self.cli("recover", a["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "working")
        self.assertIn("Status: claimed", ticket.read_text())
        self.assertIn("Owner: agent-a", ticket.read_text())
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=Path(recovered["path"])), recovered["base"])

    def test_open_attempt_recover_adopts_corenius_worktree_after_creation_crash(self):
        self.init()
        self.effort_ticket()
        self.commit("tracker ticket")
        result = self.open_corenius_attempt(failpoint="after_worktree_add", ok=False)
        self.assertEqual(result.returncode, 86)
        a = self.cli("status")["attempts"][0]
        self.assertEqual(a["state"], "allocating")
        before = self.git("worktree", "list", "--porcelain")
        recovered = self.cli("recover", a["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "working")
        self.assertEqual(self.git("worktree", "list", "--porcelain"), before)

    def test_dependencies_missing_cycles_closed_and_regular_ticket_modes(self):
        self.init()
        self.ticket("T-dep", status="closed")
        self.ticket("T-ready", deps=["T-dep"])
        self.ticket("T-blocked", deps=["T-ready"])
        self.ticket("T-missing", deps=["does-not-exist"])
        self.ticket("T-cycle-a", deps=["T-cycle-b"])
        self.ticket("T-cycle-b", deps=["T-cycle-a"])
        self.ticket("T-executable").chmod(0o755)
        self.commit("dependency graph")
        self.assertEqual([t["id"] for t in self.cli("ready")], ["T-ready"])
        for tid in ("T-dep", "T-blocked", "T-missing", "T-cycle-a", "T-executable", "T-rea", "../T-ready"):
            with self.subTest(ticket=tid):
                self.cli("open-attempt", tid, "--owner", "worker", ok=False)
        a = self.claim("T-ready")
        self.assertEqual(a["state"], "working")

    def test_stale_claim_cannot_submit_even_after_private_branch_fast_forward(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        (self.repo / "operator.txt").write_text("new trunk\n")
        self.commit("trunk advances")
        self.git("merge", "--ff-only", "main", cwd=Path(a["path"]))
        self.contribute(a)
        result = self.cli("submit", a["id"], ok=False)
        self.assertIn("Trunk moved", result.stderr)

    def test_landing_rejects_replaced_primary_checkout(self):
        self.ticket()
        self.commit("ticket")
        self.git("switch", "-c", "source")
        operator = self.root / "operator"
        self.git("worktree", "add", str(operator), "main")
        self.cli("init", "--trunk", "main", "--check-json", '["true"]',
                 "--workspace-root", str(self.root / "workspaces"), repo=operator)
        a = self.cli("open-attempt", "T-one", "--owner", "worker", repo=operator)
        self.contribute(a)
        self.cli("submit", a["id"], repo=Path(a["path"]))
        self.cli("check", a["id"], repo=Path(a["path"]))
        operator.rename(self.root / "preserved-operator")
        self.git("clone", "--no-hardlinks", "--branch", "main", str(self.repo), str(operator))
        result = self.cli("land", a["id"], repo=Path(a["path"]), ok=False)
        self.assertIn("identity", result.stderr)
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=operator), a["base"])
        self.assertEqual(self.cli("show", a["id"], repo=Path(a["path"]))["state"], "verified")

    def test_protected_coordination_and_policy_paths_cannot_submit(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        worker = Path(a["path"])
        protected_paths = (
            ".github/workflow.yml",
            ".corenius/config",
            "docs/agents/policy.md",
            "AGENTS.md",
            "CLAUDE.md",
            ".tickets/T-extra.md",
        )
        for name in protected_paths:
            with self.subTest(path=name):
                target = worker / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("worker policy edit\n")
                self.commit("protected path", cwd=worker)
                result = self.cli("submit", a["id"], ok=False)
                self.assertIn("coordination/policy", result.stderr)
                target.unlink()
                self.commit("withdraw protected edit", cwd=worker)

    def test_renaming_a_protected_path_cannot_submit(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        gate = Path(self.repo) / "tools" / "agents" / "check.sh"
        gate.parent.mkdir(parents=True)
        gate.write_text("#!/bin/sh\nexit 0\n")
        self.commit("gate entry point on trunk")
        a = self.claim()
        worker = Path(a["path"])
        (worker / "moved").mkdir()
        self.git("mv", "tools/agents/check.sh", "moved/check.sh", cwd=worker)
        self.commit("rename protected file", cwd=worker)
        result = self.cli("submit", a["id"], ok=False)
        self.assertIn("coordination/policy", result.stderr)

    def test_default_protected_paths_include_gate_and_constitution(self):
        config = self.init()
        self.assertEqual(
            config["protected_paths"],
            [
                ".tickets/",
                ".github/",
                ".corenius/",
                "docs/agents/",
                "AGENTS.md",
                "CLAUDE.md",
                "tools/agents/checks.d/",
                "tools/agents/check.sh",
                "CONSTITUTION.md",
            ],
        )
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        worker = Path(a["path"])
        protected_paths = (
            "tools/agents/checks.d/65-collab.sh",
            "tools/agents/check.sh",
            "CONSTITUTION.md",
        )
        for name in protected_paths:
            with self.subTest(path=name):
                target = worker / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("worker gate edit\n")
                self.commit("protected path", cwd=worker)
                result = self.cli("submit", a["id"], ok=False)
                self.assertIn("protected path", result.stderr)
                target.unlink()
                self.commit("withdraw protected edit", cwd=worker)

    def test_repository_specific_protected_paths_are_set_at_init(self):
        config = self.init(protected_paths=["project-policy/", "OWNERS"])
        self.assertEqual(config["protected_paths"], ["project-policy/", "OWNERS"])
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        worker = Path(a["path"])
        (worker / "project-policy").mkdir()
        (worker / "project-policy/rules.md").write_text("worker policy edit\n")
        self.commit("protected project policy", cwd=worker)
        result = self.cli("submit", a["id"], ok=False)
        self.assertIn("protected path", result.stderr)

    def test_init_rejects_invalid_protected_path_json(self):
        for value in ('"OWNERS"', '[1]', '[""]', '["../outside"]', '["/absolute"]', '["bad\\u0000path"]'):
            with self.subTest(value=value):
                result = self.cli(
                    "init",
                    "--trunk",
                    "main",
                    "--check-json",
                    '["true"]',
                    "--protected-paths-json",
                    value,
                    ok=False,
                )
                self.assertIn("protected", result.stderr)
                self.assertFalse((self.repo / ".git/collab/state.sqlite3").exists())

    def test_dirty_worker_and_operator_files_are_preserved(self):
        a = self.prepared()
        worker = Path(a["path"])
        (worker / "code.txt").write_text("uncommitted worker work\n")
        self.cli("submit", a["id"], ok=False)
        self.assertEqual((worker / "code.txt").read_text(), "uncommitted worker work\n")
        checked = self.cli("check", a["id"])
        (self.repo / "code.txt").write_text("uncommitted operator work\n")
        self.cli("land", a["id"], ok=False)
        self.assertEqual((self.repo / "code.txt").read_text(), "uncommitted operator work\n")
        self.assertEqual(self.cli("show", a["id"])["state"], "verified")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])

    def test_uncommitted_contract_is_ignored_but_committed_byte_change_invalidates_claim(self):
        a = self.prepared()
        ticket = self.repo / ".tickets/T-one.md"
        original = ticket.read_bytes()
        ticket.write_bytes(original + b"Uncommitted proposal.\n")
        checked = self.cli("check", a["id"])
        self.assertEqual(checked["state"], "verified")
        ticket.write_bytes(original.replace(b"\n", b"\r\n"))
        self.commit("contract line-ending change")
        for command in ("submit", "check", "land"):
            self.cli(command, a["id"], ok=False)
        self.assertEqual(self.cli("show", a["id"])["ticket_hash"], a["ticket_hash"])

    def test_failing_mutating_and_head_moving_checks_never_verify(self):
        mode_file = self.root / "check-mode"
        code = ("import subprocess,sys; from pathlib import Path; "
                f"mode=Path({str(mode_file)!r}).read_text(); "
                "print('mode='+mode); "
                "Path('code.txt').write_text('mutation') if mode=='tracked' else None; "
                "Path('extra.txt').write_text('untracked') if mode=='untracked' else None; "
                "subprocess.run(['git','commit','--allow-empty','-m','mutated HEAD'],"
                "check=True) if mode=='head' else None; "
                "sys.exit(7 if mode=='fail' else 0)")
        a = self.prepared([sys.executable, "-c", code])
        for mode in ("fail", "tracked", "untracked", "head"):
            with self.subTest(mode=mode):
                mode_file.write_text(mode)
                self.cli("check", a["id"], ok=False)
                current = self.cli("show", a["id"])
                self.assertEqual(current["state"], "check_failed")
                self.assertFalse(current["check_receipt"]["passed"])
                self.assertEqual(
                    current["check_receipt"]["outcome"],
                    {
                        "fail": "failed",
                        "tracked": "dirty",
                        "untracked": "dirty",
                        "head": "changed_head",
                    }[mode],
                )
                self.assertIn("mode=" + mode, Path(current["log"]).read_text())
                self.cli("land", a["id"], ok=False)
        mode_file.write_text("pass")
        checked = self.cli("check", a["id"])
        self.assertEqual(checked["state"], "verified")
        self.assertEqual(len(list((self.root / "workspaces").glob("check-*"))), 5)

    def test_hung_check_times_out_with_distinct_receipt_and_live_log(self):
        marker = self.root / "check-started"
        code = ("import time\nfrom pathlib import Path\n"
                "print('before hang', flush=True)\n"
                f"Path({str(marker)!r}).write_text('started')\n"
                "time.sleep(30)\n")
        a = self.prepared([sys.executable, "-c", code], timeout_seconds=0.2)
        result = self.cli("check", a["id"], ok=False)
        self.assertIn("timed out", result.stderr)
        current = self.cli("show", a["id"])
        self.assertEqual(current["state"], "check_failed")
        receipt = current["check_receipt"]
        self.assertEqual(receipt["version"], 2)
        self.assertEqual(receipt["argv"], [sys.executable, "-c", code])
        self.assertEqual(receipt["outcome"], "timeout")
        self.assertEqual(receipt["exit_code"], None)
        self.assertEqual(receipt["timeout_seconds"], 0.2)
        self.assertFalse(receipt["passed"])
        self.assertEqual(receipt["log_sha256"], hashlib.sha256(Path(current["log"]).read_bytes()).hexdigest())
        self.assertIn("before hang", Path(current["log"]).read_text())
        self.assertTrue(marker.exists())

    def test_live_check_log_is_readable_before_command_exits(self):
        ready, release = self.root / "check-log-ready", self.root / "release-check"
        code = ("import time\nfrom pathlib import Path\n"
                "print('streamed line', flush=True)\n"
                f"Path({str(ready)!r}).write_text('ready')\n"
                "deadline=time.monotonic()+10\n"
                f"while not Path({str(release)!r}).exists() and time.monotonic()<deadline: time.sleep(.01)\n")
        a = self.prepared([sys.executable, "-c", code])
        process = subprocess.Popen([sys.executable, str(CLI), "--repo", str(self.repo), "check", a["id"]],
                                   env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.monotonic() + 5
            while not ready.exists() and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(ready.exists(), "check child did not write its ready marker")
            current = self.cli("show", a["id"])
            log = Path(current["log"])
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline and "streamed line" not in log.read_text():
                time.sleep(.01)
            self.assertIn("streamed line", log.read_text())
        finally:
            release.touch()
            out, err = process.communicate(timeout=5)
        self.assertEqual(process.returncode, 0, err)
        self.assertEqual(json.loads(out)["state"], "verified")

    def test_check_uses_literal_argv_and_land_rejects_changed_receipt_log(self):
        marker = self.root / "shell-must-not-execute"
        literal = f"; touch {marker}; $(false)"
        argv = [sys.executable, "-c", "import sys; print(sys.argv[1])", literal]
        a = self.prepared(argv)
        checked = self.cli("check", a["id"])
        self.assertFalse(marker.exists())
        self.assertEqual(checked["check_receipt"]["argv"], argv)
        self.assertIn(literal, Path(checked["log"]).read_text())
        Path(checked["log"]).write_text("replaced verification output\n")
        result = self.cli("land", a["id"], ok=False)
        self.assertIn("receipt", result.stderr)
        self.assertEqual(self.git("rev-parse", "HEAD"), a["base"])

    def test_zero_exit_with_trunk_movement_is_check_failed(self):
        argv = [sys.executable, "-c", "import subprocess; subprocess.run(['git','-C'," +
                repr(str(self.repo)) + ",'commit','--allow-empty','-m','operator moved trunk'],check=True)"]
        a = self.prepared(argv)
        self.cli("check", a["id"], ok=False)
        current = self.cli("show", a["id"])
        self.assertEqual(current["state"], "check_failed")
        self.assertEqual(current["check_receipt"]["exit_code"], 0)
        self.assertEqual(current["check_receipt"]["outcome"], "trunk_moved")
        self.assertTrue(current["check_receipt"]["clean"])
        self.assertNotEqual(current["check_receipt"]["trunk"], current["base"])
        self.cli("land", a["id"], ok=False)

    def test_refresh_abandons_stale_attempt_and_cherry_picks_into_fresh_claim(self):
        a = self.prepared()
        self.cli("check", a["id"])
        (self.repo / "operator.txt").write_text("new trunk\n")
        new_base = self.commit("advance trunk")
        for command in ("submit", "check", "land"):
            self.cli(command, a["id"], ok=False)
        self.cli("abandon", a["id"], ok=False)
        self.cli("abandon", a["id"], "--confirm-stopped")
        self.cli("submit", a["id"], ok=False)
        fresh = self.claim()
        self.assertEqual(fresh["base"], new_base)
        self.assertNotEqual(fresh["path"], a["path"])
        self.git("cherry-pick", a["candidate"], cwd=Path(fresh["path"]))
        self.cli("submit", fresh["id"])
        self.cli("check", fresh["id"])
        self.assertEqual(self.cli("land", fresh["id"])["state"], "landed")
        self.assertTrue(Path(a["path"]).exists())

    def test_interrupted_and_failed_allocations_retain_claim_until_explicit_abandon(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        result = self.cli("open-attempt", "T-one", "--owner", "worker", failpoint="after_claim_record", ok=False)
        self.assertEqual(result.returncode, 86)
        a = self.cli("status")["attempts"][0]
        self.assertEqual(a["state"], "allocating")
        self.cli("recover", a["id"], "--confirm-stopped", ok=False)
        self.cli("open-attempt", "T-one", "--owner", "second", ok=False)
        self.cli("abandon", a["id"], "--confirm-stopped")
        (self.root / "workspaces").write_text("operator owns this file\n")
        self.cli("open-attempt", "T-one", "--owner", "worker", ok=False)
        second = self.cli("status")["attempts"][-1]
        self.assertEqual(second["state"], "allocating")
        self.assertEqual((self.root / "workspaces").read_text(), "operator owns this file\n")
        self.cli("open-attempt", "T-one", "--owner", "third", ok=False)

    def test_existing_allocation_is_adopted_without_duplicate_worktree(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        result = self.cli("open-attempt", "T-one", "--owner", "worker", failpoint="after_worktree_add", ok=False)
        self.assertEqual(result.returncode, 86)
        a = self.cli("status")["attempts"][0]
        before = self.git("worktree", "list", "--porcelain")
        self.cli("recover", a["id"], ok=False)
        recovered = self.cli("recover", a["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "working")
        self.assertEqual(self.git("worktree", "list", "--porcelain"), before)

    def test_lost_landing_acknowledgment_recovers_attributed_integration_ancestry(self):
        a = self.prepared()
        checked = self.cli("check", a["id"])
        result = self.cli("land", a["id"], failpoint="after_git_land", ok=False)
        self.assertEqual(result.returncode, 86)
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])
        (self.repo / "operator.txt").write_text("after landing\n")
        head = self.commit("later operator commit")
        recovered = self.cli("recover", a["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "landed")
        self.assertEqual(self.git("rev-parse", "HEAD"), head)
        self.assertEqual(self.git("rev-parse", "refs/collab/landed/" + a["id"]), checked["integration"])

    def test_matching_integration_without_attribution_cannot_reconcile_landing(self):
        a = self.prepared()
        checked = self.cli("check", a["id"])
        self.cli("land", a["id"], failpoint="after_land_started", ok=False)
        self.git("merge", "--ff-only", checked["integration"])
        result = self.cli("recover", a["id"], "--confirm-stopped", ok=False)
        self.assertIn("Ambiguous", result.stderr)
        self.assertEqual(self.cli("show", a["id"])["state"], "landing_started")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])

    def test_lost_reflog_never_establishes_started_landing_nonexecution(self):
        a = self.prepared()
        self.cli("check", a["id"])
        self.cli("land", a["id"], failpoint="after_land_started", ok=False)
        self.git("reflog", "expire", "--expire=all", "--all")
        result = self.cli("recover", a["id"], "--confirm-stopped", ok=False)
        self.assertIn("Ambiguous", result.stderr)
        self.assertEqual(self.cli("show", a["id"])["state"], "landing_started")

    def test_prepared_landing_with_unrelated_trunk_movement_stays_unresolved(self):
        a = self.prepared()
        self.cli("check", a["id"])
        self.cli("land", a["id"], failpoint="before_git_land", ok=False)
        (self.repo / "operator.txt").write_text("unrelated movement\n")
        head = self.commit("unrelated trunk commit")
        self.cli("recover", a["id"], "--confirm-stopped", ok=False)
        self.assertEqual(self.cli("show", a["id"])["state"], "landing_prepared")
        self.assertEqual(self.git("rev-parse", "HEAD"), head)
        self.cli("abandon", a["id"], "--confirm-stopped", ok=False)

    def test_cleanup_preserves_all_dirty_content_and_retains_history_and_old_checks(self):
        a = self.prepared()
        old = self.cli("check", a["id"])
        checked = self.cli("check", a["id"])
        self.cli("gc", a["id"], "--confirm-stopped", ok=False)
        self.cli("abandon", a["id"], "--confirm-stopped")
        self.cli("gc", a["id"], ok=False)
        worker, verification = Path(a["path"]), Path(checked["check_path"])
        ignored = verification / "build/output"
        ignored.parent.mkdir()
        ignored.write_text("ignored but valuable\n")
        self.cli("gc", a["id"], "--confirm-stopped", ok=False)
        self.assertEqual(ignored.read_text(), "ignored but valuable\n")
        self.assertTrue(worker.exists())
        ignored.unlink()
        ignored.parent.rmdir()
        (worker / "code.txt").write_text("tracked unsaved\n")
        self.cli("gc", a["id"], "--confirm-stopped", ok=False)
        self.assertEqual((worker / "code.txt").read_text(), "tracked unsaved\n")
        (worker / "code.txt").write_text("contribution\n")
        untracked = worker / "operator-notes"
        untracked.write_text("untracked valuable\n")
        self.cli("gc", a["id"], "--confirm-stopped", ok=False)
        self.assertEqual(untracked.read_text(), "untracked valuable\n")
        untracked.unlink()
        cleaned = self.cli("gc", a["id"], "--confirm-stopped")
        self.assertEqual(cleaned["removed"], [str(worker), str(verification)])
        self.assertTrue(Path(old["check_path"]).is_dir())
        self.assertTrue(Path(checked["log"]).is_file())
        self.assertEqual(self.git("rev-parse", a["branch"]), a["candidate"])
        self.assertEqual(
            self.git("rev-parse", "refs/collab/candidates/" + a["id"] + "/" + a["candidate"]),
            a["candidate"],
        )
        self.assertEqual(self.cli("show", a["id"])["state"], "abandoned")

    def test_sweep_dry_run_lists_candidates_without_removing(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.claim()
        worker = Path(attempt["path"])
        self.cli("abandon", attempt["id"], "--confirm-stopped")
        self.unlock_worktree(attempt)
        result = self.cli("sweep")
        self.assertFalse(result["applied"])
        self.assertEqual([item["attempt"] for item in result["candidates"]], [attempt["id"]])
        self.assertEqual(result["candidates"][0]["reason"], "terminal Attempt has a clean merged branch")
        self.assertTrue(worker.is_dir())
        self.assertEqual(self.git("rev-parse", attempt["branch"]), attempt["base"])

    def test_sweep_apply_removes_merged_terminal_attempt(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.claim()
        worker = Path(attempt["path"])
        self.cli("abandon", attempt["id"], "--confirm-stopped")
        self.unlock_worktree(attempt)
        result = self.cli("sweep", "--apply")
        self.assertTrue(result["applied"])
        self.assertEqual(result["removed"][0]["attempt"], attempt["id"])
        self.assertFalse(worker.exists())
        branch = self.run_process(["git", "rev-parse", "--verify", attempt["branch"]])
        self.assertNotEqual(branch.returncode, 0)

    def test_sweep_apply_never_removes_fresh_attempt(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.claim()
        worker = Path(attempt["path"])
        result = self.cli("sweep", "--apply")
        self.assertEqual(result["candidates"], [])
        self.assertEqual(result["skipped"][0]["attempt"], attempt["id"])
        self.assertIn("working", result["skipped"][0]["reason"])
        self.assertTrue(worker.is_dir())
        self.assertEqual(self.git("rev-parse", attempt["branch"]), attempt["base"])

    def test_sweep_reports_locked_terminal_worktree_as_skipped(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.claim()
        self.cli("abandon", attempt["id"], "--confirm-stopped")
        result = self.cli("sweep", "--apply")
        self.assertEqual(result["candidates"], [])
        self.assertEqual(result["skipped"][0]["attempt"], attempt["id"])
        self.assertIn("worktree is locked: collab:" + attempt["id"], result["skipped"][0]["reason"])
        self.assertTrue(Path(attempt["path"]).is_dir())
        self.assertEqual(self.git("rev-parse", attempt["branch"]), attempt["base"])

    def test_sweep_archives_unmerged_clean_terminal_branch_before_removal(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.claim()
        worker = Path(attempt["path"])
        head = self.contribute(attempt)
        self.cli("abandon", attempt["id"], "--confirm-stopped")
        self.unlock_worktree(attempt)
        result = self.cli("sweep", "--apply")
        candidate = result["candidates"][0]
        self.assertEqual(candidate["action"], "archive_then_remove_worktree_and_branch")
        self.assertEqual(candidate["head"], head)
        self.assertFalse(worker.exists())
        self.assertEqual(self.git("rev-parse", candidate["archive_ref"]), head)
        branch = self.run_process(["git", "rev-parse", "--verify", attempt["branch"]])
        self.assertNotEqual(branch.returncode, 0)

    def test_sweep_lists_foreign_unmerged_branch_with_owner_and_leaves_it(self):
        self.init()
        self.git("switch", "-c", "other-worker/feature")
        (self.repo / "foreign.txt").write_text("foreign work\n")
        head = self.commit("foreign work")
        self.git("switch", "main")
        result = self.cli("sweep", "--apply")
        foreign = [item for item in result["skipped"] if item["kind"] == "foreign_branch"]
        self.assertEqual(len(foreign), 1)
        self.assertEqual(foreign[0]["branch"], "other-worker/feature")
        self.assertEqual(foreign[0]["owner"], "other-worker")
        self.assertEqual(foreign[0]["head"], head)
        self.assertIn("unmerged", foreign[0]["reason"])
        self.assertEqual(self.git("rev-parse", "other-worker/feature"), head)

    def test_live_check_child_retains_check_lock_after_cli_parent_death(self):
        started, release, done = [self.root / name for name in ("child-started", "release-child", "child-done")]
        code = ("import os,time\nfrom pathlib import Path\n"
                f"Path({str(started)!r}).write_text(str(os.getpid()))\n"
                "deadline=time.monotonic()+10\n"
                f"while not Path({str(release)!r}).exists() and time.monotonic()<deadline: time.sleep(.01)\n"
                f"Path({str(done)!r}).write_text('done')\n")
        a = self.prepared([sys.executable, "-c", code])
        process = subprocess.Popen([sys.executable, str(CLI), "--repo", str(self.repo), "check", a["id"]],
                                   env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.monotonic() + 5
            while not started.exists() and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(started.exists(), "check child did not start")
            process.kill()
            process.communicate(timeout=5)
            result = self.cli("recover", a["id"], "--confirm-stopped", ok=False)
            self.assertIn("check operation busy", result.stderr)
            self.assertEqual(self.cli("show", a["id"])["state"], "checking")
        finally:
            release.touch()
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=5)
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            result = self.run_process([sys.executable, str(CLI), "--repo", str(self.repo),
                                       "recover", a["id"], "--confirm-stopped"])
            if result.returncode == 0:
                break
            self.assertIn("check operation busy", result.stderr)
            time.sleep(.01)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["state"], "check_failed")
        self.assertTrue(done.exists())

    def test_running_check_does_not_hold_landing_lane_lock(self):
        self.init([sys.executable, "check_gate"])
        (self.repo / "check_gate").write_text("print('checked')\n")
        self.ticket("T-one")
        self.ticket("T-two")
        self.commit("two tickets and check gate")
        first = self.claim("T-one")
        self.contribute(first, "first\n")
        first = self.cli("submit", first["id"])
        first = self.cli("check", first["id"])
        self.assertEqual(first["state"], "verified")

        second = self.claim("T-two")
        worker = Path(second["path"])
        gate = worker / "check_gate"
        (worker / "code.txt").write_text("second\n")
        self.commit("second contribution", cwd=worker)
        ready, release = self.root / "long-check-ready", self.root / "release-long-check"
        code = ("import time\nfrom pathlib import Path\n"
                f"Path({str(ready)!r}).write_text('ready')\n"
                "deadline=time.monotonic()+10\n"
                f"while not Path({str(release)!r}).exists() and time.monotonic()<deadline: time.sleep(.01)\n")
        (worker / "check_gate").write_text(code)
        self.commit("add long check gate", cwd=worker)
        self.cli("submit", second["id"])
        process = subprocess.Popen([sys.executable, str(CLI), "--repo", str(self.repo), "check", second["id"]],
                                   env=self.env,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            deadline = time.monotonic() + 5
            while not ready.exists() and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(ready.exists(), "long check did not start")
            landed = self.cli("land", first["id"])
            self.assertEqual(landed["state"], "landed")
        finally:
            release.touch()
            out, err = process.communicate(timeout=5)
        self.assertNotEqual(process.returncode, 0, out)
        self.assertIn("trunk moved", err.lower())
        self.assertEqual(self.cli("show", second["id"])["state"], "check_failed")

    def test_backup_creates_snapshot_and_never_overwrites_existing_or_dangling_link(self):
        self.init()
        target = self.root / "snapshot.sqlite3"
        result = self.cli("backup", str(target))
        self.assertEqual(result["database_backup"], str(target))
        self.assertIn("refs, workspaces and logs", result["note"])
        before = target.read_bytes()
        self.assertTrue(before.startswith(b"SQLite format 3\0"))
        self.cli("backup", str(target), ok=False)
        self.assertEqual(target.read_bytes(), before)
        dangling, missing = self.root / "dangling.sqlite3", self.root / "missing.sqlite3"
        dangling.symlink_to(missing)
        self.cli("backup", str(dangling), ok=False)
        self.assertTrue(dangling.is_symlink())
        self.assertFalse(missing.exists())

    def test_independent_repositories_competing_for_backup_destination_have_one_winner(self):
        self.init()
        other = self.root / "other"
        other.mkdir()
        self.git("init", "-b", "main", cwd=other)
        self.git("commit", "--allow-empty", "-m", "other", cwd=other)
        self.cli("init", "--trunk", "main", "--check-json", '["true"]', repo=other)
        target = self.root / "shared-backup.sqlite3"
        processes = [subprocess.Popen([sys.executable, str(CLI), "--repo", str(repo), "backup", str(target)],
                                      env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                     for repo in (self.repo, other)]
        results = []
        for process in processes:
            out, err = process.communicate(timeout=5)
            results.append((process.returncode, out, err))
        self.assertEqual(sum(code == 0 for code, _, _ in results), 1, results)
        self.assertTrue(target.read_bytes().startswith(b"SQLite format 3\0"))

    def test_submission_requires_nonempty_linear_contribution(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        worker = Path(a["path"])
        self.cli("submit", a["id"], ok=False)
        self.git("commit", "--allow-empty", "-m", "empty worker commit", cwd=worker)
        result = self.cli("submit", a["id"], ok=False)
        self.assertIn("Empty contribution", result.stderr)
        self.git("switch", "-c", "side", cwd=worker)
        (worker / "side.txt").write_text("side\n")
        self.commit("side contribution", cwd=worker)
        self.git("switch", a["branch"], cwd=worker)
        self.contribute(a)
        self.git("merge", "--no-ff", "-m", "merge side", "side", cwd=worker)
        result = self.cli("submit", a["id"], ok=False)
        self.assertIn("linear", result.stderr)

    def test_landing_never_overwrites_ignored_operator_content(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        a = self.claim()
        worker = Path(a["path"])
        (worker / "build").mkdir()
        (worker / "build/ship.txt").write_text("tracked contribution\n")
        self.git("add", "--force", "build/ship.txt", cwd=worker)
        self.commit("track formerly ignored output", cwd=worker)
        self.cli("submit", a["id"])
        self.cli("check", a["id"])
        (self.repo / "build").mkdir()
        operator_file = self.repo / "build/ship.txt"
        operator_file.write_text("ignored operator content\n")
        self.cli("land", a["id"], ok=False)
        self.assertEqual(operator_file.read_text(), "ignored operator content\n")
        self.assertEqual(self.git("rev-parse", "HEAD"), a["base"])
        self.assertEqual(self.cli("show", a["id"])["state"], "landing_started")
        self.cli("recover", a["id"], "--confirm-stopped", ok=False)

    def test_interrupted_check_before_command_is_failed_on_explicit_recovery(self):
        a = self.prepared()
        result = self.cli("check", a["id"], failpoint="before_check_command", ok=False)
        self.assertEqual(result.returncode, 86)
        interrupted = self.cli("show", a["id"])
        self.assertEqual(interrupted["state"], "checking")
        self.assertIsNone(interrupted["check_receipt"])
        self.assertFalse(Path(interrupted["log"]).exists())
        self.cli("recover", a["id"], ok=False)
        self.assertEqual(self.cli("recover", a["id"], "--confirm-stopped")["state"], "check_failed")
        self.assertEqual(self.cli("check", a["id"])["state"], "verified")

    def test_submit_refuses_skip_worktree_unsaved_content_without_clearing_flag(self):
        a = self.prepared()
        worker = Path(a["path"])
        self.git("update-index", "--skip-worktree", "code.txt", cwd=worker)
        (worker / "code.txt").write_text("hidden unsaved worker work\n")
        result = self.cli("submit", a["id"], ok=False)
        self.assertIn("hidden index", result.stderr)
        self.assertEqual((worker / "code.txt").read_text(), "hidden unsaved worker work\n")
        self.assertIn("S code.txt", self.git("ls-files", "-v", cwd=worker))

    def test_gc_preserves_assume_unchanged_unsaved_content_and_flag(self):
        a = self.prepared()
        worker = Path(a["path"])
        self.cli("abandon", a["id"], "--confirm-stopped")
        self.git("update-index", "--assume-unchanged", "code.txt", cwd=worker)
        (worker / "code.txt").write_text("hidden unsaved worker work\n")
        result = self.cli("gc", a["id"], "--confirm-stopped", ok=False)
        self.assertIn("hidden index", result.stderr)
        self.assertEqual((worker / "code.txt").read_text(), "hidden unsaved worker work\n")
        self.assertIn("h code.txt", self.git("ls-files", "-v", cwd=worker))

    def assert_hidden_check_fails(self, flag):
        code = ("import subprocess; from pathlib import Path; "
                f"subprocess.run(['git','update-index',{flag!r},'code.txt'],check=True); "
                "Path('code.txt').write_text('hidden check mutation\\n')")
        a = self.prepared([sys.executable, "-c", code])
        self.cli("check", a["id"], ok=False)
        checked = self.cli("show", a["id"])
        self.assertEqual(checked["state"], "check_failed")
        self.assertFalse(checked["check_receipt"]["clean"])
        self.assertFalse(checked["check_receipt"]["passed"])
        self.assertEqual((Path(checked["check_path"]) / "code.txt").read_text(), "hidden check mutation\n")
        self.cli("land", a["id"], ok=False)

    def test_check_cannot_verify_assume_unchanged_hidden_writes(self):
        self.assert_hidden_check_fails("--assume-unchanged")

    def test_check_cannot_verify_skip_worktree_hidden_writes(self):
        self.assert_hidden_check_fails("--skip-worktree")

    def test_land_refuses_hidden_primary_content_before_starting_git(self):
        a = self.prepared()
        self.cli("check", a["id"])
        self.git("update-index", "--assume-unchanged", "code.txt")
        (self.repo / "code.txt").write_text("hidden operator work\n")
        result = self.cli("land", a["id"], ok=False)
        self.assertIn("hidden index", result.stderr)
        self.assertEqual(self.cli("show", a["id"])["state"], "verified")
        self.assertEqual((self.repo / "code.txt").read_text(), "hidden operator work\n")

    def test_git_check_observes_integration_despite_inherited_repository_redirection(self):
        a = self.prepared(["git", "rev-parse", "HEAD"])
        redirected = dict(self.env, GIT_DIR=str(self.repo / ".git"), GIT_WORK_TREE=str(self.repo),
                          GIT_COMMON_DIR=str(self.repo / ".git"), GIT_INDEX_FILE=str(self.repo / ".git/index"))
        checked = self.cli("check", a["id"], env=redirected)
        self.assertEqual(checked["state"], "verified")
        self.assertEqual(Path(checked["log"]).read_text().splitlines()[-1], checked["integration"])
        self.assertEqual(self.git("rev-parse", "HEAD"), a["base"])

    def test_standalone_ticket_delimiters_preserve_triple_hyphens_and_crlf_bytes(self):
        self.init()
        authored = self.cli("new", "T---one", "A scalar---title")
        ticket = Path(authored["path"])
        original = (ticket.read_bytes().replace(b"deps: []\n", b"deps: []\ncontext: scalar---content\n") +
                    b"## Meaning\nKeep --- inside prose.\n\n---\nFollowing horizontal rule.\n")
        original = original.replace(b"\n", b"\r\n")
        ticket.write_bytes(original)
        self.commit("ticket with literal triple hyphens")
        self.assertEqual([t["id"] for t in self.cli("ready")], ["T---one"])
        a = self.claim("T---one")
        self.assertEqual(a["ticket_hash"], hashlib.sha256(original).hexdigest())
        self.contribute(a)
        self.cli("submit", a["id"])
        self.cli("check", a["id"])
        self.cli("land", a["id"])
        self.assertEqual(ticket.read_bytes(), original.replace(b"status: open", b"status: closed"))

    def test_completed_check_with_removed_workspace_records_failed_receipt(self):
        code = "import shutil; from pathlib import Path; shutil.rmtree(Path.cwd())"
        a = self.prepared([sys.executable, "-c", code])
        self.cli("check", a["id"], ok=False)
        current = self.cli("show", a["id"])
        self.assertEqual(current["state"], "check_failed")
        receipt = current["check_receipt"]
        self.assertEqual(receipt["exit_code"], 0)
        self.assertFalse(receipt["passed"])
        for condition in ("clean", "same_head", "identity"):
            self.assertFalse(receipt[condition])
        self.assertTrue(receipt["inspection_error"])
        self.assertEqual(receipt["subject"], current["integration"])
        self.assertEqual(receipt["trunk"], current["base"])
        self.assertFalse(Path(current["check_path"]).exists())
        self.cli("land", a["id"], ok=False)

    def test_new_rejects_duplicate_and_self_dependencies_without_dirtying_operator(self):
        self.init()
        for deps in (["T-new"], ["T-parent", "T-parent"]):
            with self.subTest(deps=deps):
                result = self.cli("new", "T-new", "Invalid dependencies", "--deps", *deps, ok=False)
                self.assertIn("dependency", result.stderr)
                self.assertEqual(self.git("status", "--porcelain=v1"), "")
                self.assertFalse((self.repo / ".tickets").exists())


class CollabStructureTests(unittest.TestCase):
    def test_collab_toolset_has_named_modules_and_single_sqlite_owner(self):
        expected = {
            "check_runner.py",
            "cli.py",
            "landing.py",
            "policy.py",
            "state_store.py",
            "ticket_source.py",
            "workspace.py",
        }
        present = {path.name for path in COLLAB_ROOT.glob("*.py")}
        self.assertTrue(expected <= present, expected - present)

        for path in sorted(COLLAB_ROOT.glob("*.py")):
            source = path.read_text()
            if path.name == "state_store.py":
                self.assertIn("sqlite3", source)
            else:
                self.assertNotIn("import sqlite3", source, path.name)
                self.assertNotIn("sqlite3.", source, path.name)

        long_lines = []
        for path in sorted(COLLAB_ROOT.glob("*.py")):
            for line_number, line in enumerate(path.read_text().splitlines(), start=1):
                if len(line) > 120:
                    long_lines.append(f"{path.name}:{line_number}")
        self.assertEqual(long_lines, [])

    def test_cli_entry_uses_command_table_without_packed_statements(self):
        source = (COLLAB_ROOT / "cli.py").read_text()
        self.assertIn("COMMANDS =", source)
        self.assertNotIn("elif args.command", source)
        tokens = tokenize.generate_tokens(io.StringIO(source).readline)
        self.assertFalse(any(token.string == ";" for token in tokens))


if __name__ == "__main__":
    unittest.main()

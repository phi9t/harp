"""lead_launch.py against throwaway git repositories and fake workers."""

from __future__ import annotations

import contextlib
import io
import json
import os
import signal
import stat
import subprocess
import sys
import tempfile
import textwrap
import time
import unittest
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import lead_launch  # noqa: E402


FAKE_WORKER = textwrap.dedent(
    """\
    #!/usr/bin/env python3
    import json, os, subprocess, sys, time
    print(json.dumps({"type": "thread.started"}), flush=True)
    mode = os.environ.get("FAKE_MODE", "sleep")
    if mode == "spawn_child":
        subprocess.Popen(["sh", "-c", "sleep 30"])
        time.sleep(0.2)
        sys.exit(0)
    if mode == "sleep":
        time.sleep(30)
    sys.exit(0)
    """
)

TRUST_PROMPT = textwrap.dedent(
    """\
    Folder access
    /tmp/r-worker-demo-01-hello
    Do you trust the contents of this directory? Working with untrusted contents comes with higher risk of
    prompt injection. Trusting the directory allows project-local config, hooks, and exec policies to load.
    > 1. Yes, continue
      2. No, quit
    enter continue  |  esc quit
    """
)

REAL_TRAECLI_IDLE_PANE_TAIL = textwrap.dedent(
    """\
        Checked: test -f hello.txt && printf 'hello\\n' | cmp -s - hello.txt passed, and the worktree is clean.

        Not checked: no broader suite was run because the ticket only required the file-content acceptance check.
        Checked: test -f hello.txt && printf 'hello\\n' | cmp -s - hello.txt passed, and the worktree is clean.

        5:33 PM

    \u2500\u2500\u2500 Complete demo-01-hello worker task \u2500
    \u276f Run /review on my current changes
    \u2500\u2500\u2500
      GPT-5.5 \u00b7 Context 95% left \u00b7 ~/devx/tmp/lwr18probe/r-worker-demo-01-hello \u00b7 worker/demo-01-hello \u00b7 +48 -0      \u2622 Full Access (shift+tab to cycle) \u00b7 \u2190 for agents
    """
)


def git(cwd: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


def live_session_pids(session_id: int) -> list[int]:
    pids: list[int] = []
    proc = Path("/proc")
    if not proc.is_dir():
        return pids
    for child in proc.iterdir():
        if not child.name.isdigit():
            continue
        try:
            raw = (child / "stat").read_text()
        except OSError:
            continue
        end = raw.rfind(")")
        if end == -1:
            continue
        fields = raw[end + 2 :].split()
        if len(fields) < 4:
            continue
        state = fields[0]
        sid = int(fields[3])
        if sid == session_id and state != "Z":
            pids.append(int(child.name))
    return sorted(pids)


def ticket_text(
    number: str,
    slug: str,
    *,
    status: str = "open",
    owner: str = "",
    blocked_by: str = "",
    claimed: str = "",
    triage: str | None = "ready-for-agent",
    description: str = "",
    acceptance: str = "- [ ] works\n",
    declared: list[str] | None = None,
    for_later: str | None = None,
) -> str:
    triage_text = "" if triage is None else f"Triage: {triage}\n\n"
    description_text = description.rstrip() + "\n\n" if description else ""
    declared_text = "".join(f"- `{path}`\n" for path in (declared or ["work.txt"]))
    for_later_text = "" if for_later is None else f"## For later tickets\n{for_later}\n"
    return (
        f"# {number} {slug}: Ticket {number}\n\n"
        "Type: feature\n"
        f"Status: {status}\n"
        f"Owner: {owner}\n"
        f"Blocked by: {blocked_by}\n"
        f"Claimed: {claimed}\n\n"
        "## Description\n"
        f"{triage_text}"
        f"{description_text}"
        "## Declared paths\n"
        f"{declared_text}\n"
        "## Acceptance\n"
        f"{acceptance}\n"
        f"{for_later_text}"
        "## Comments\n"
    )


class LeadLaunchTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        git(self.repo, "init", "-q", "-b", "main")
        git(self.repo, "config", "user.email", "lead@corenius")
        git(self.repo, "config", "user.name", "lead")
        (self.repo / "docs/efforts/demo/tickets").mkdir(parents=True)
        (self.repo / "docs/agents").mkdir(parents=True)
        (self.repo / "docs/efforts/demo/EFFORT.md").write_text(
            "# Effort: demo\n\n"
            "Status: active\n"
            "Base: main\n"
            "Integration branch: effort/demo\n"
            "Lead: lead\n"
        )
        (self.repo / "docs/efforts/demo/SPEC.md").write_text("spec v1\n")
        (self.repo / "AGENTS.md").write_text("agents v1\n")
        (self.repo / "CONSTITUTION.md").write_text("constitution v1\n")
        (self.repo / "docs/agents/agentic-engineering.md").write_text("workflow v1\n")
        git(self.repo, "add", ".")
        git(self.repo, "commit", "-qm", "main base")
        git(self.repo, "checkout", "-qb", "effort/demo")
        (self.repo / "integration.txt").write_text("integration only\n")
        git(self.repo, "add", "integration.txt")
        git(self.repo, "commit", "-qm", "integration tip")
        self.fake = self.root / "fake-worker"
        self.fake.write_text(FAKE_WORKER)
        self.fake.chmod(self.fake.stat().st_mode | stat.S_IXUSR)
        os.environ["CORENIUS_TRAECLI"] = str(self.fake)
        os.environ["CORENIUS_CODEX"] = str(self.fake)
        os.environ["CORENIUS_RUNS"] = str(self.root / "runs")
        os.environ["CORENIUS_TRACKER_NOW"] = "2026-10-05T12:00:00Z"
        os.environ["FAKE_MODE"] = "sleep"
        self.runs: list[Path] = []

    def tearDown(self):
        for run_dir in self.runs:
            try:
                run_data = json.loads((run_dir / "run.json").read_text())
                os.killpg(os.getpgid(run_data["pid"]), signal.SIGTERM)
                time.sleep(0.1)
            except (FileNotFoundError, ProcessLookupError, PermissionError):
                pass
        os.environ.pop("CORENIUS_TRAECLI", None)
        os.environ.pop("CORENIUS_CODEX", None)
        os.environ.pop("CORENIUS_RUNS", None)
        os.environ.pop("CORENIUS_TRACKER_NOW", None)
        os.environ.pop("FAKE_MODE", None)
        self.tmp.cleanup()

    def tree_listing(self, root: Path) -> list[str]:
        return sorted(str(path.relative_to(root)) for path in root.rglob("*"))

    def write_ticket(self, name: str, text: str) -> None:
        (self.repo / "docs/efforts/demo/tickets" / name).write_text(text)

    def write_report(self, name: str, text: str) -> None:
        path = self.repo / "docs/efforts/demo/evidence" / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def write_landed_candidate_records(self, ticket: str, candidate: str) -> None:
        evidence = self.repo / "docs/efforts/demo/evidence"
        evidence.mkdir(parents=True, exist_ok=True)
        acceptance = {
            "schema": "corenius.acceptance.v1",
            "effort": "demo",
            "ticket": ticket,
            "candidate": candidate,
            "reviewer": "lead",
            "reviewer_context": "lead",
            "axes": {"standards": "passed", "spec": "passed"},
            "required_checks": [{"name": "unit", "result": "passed"}],
            "accepted": True,
        }
        landing = {
            "schema": "corenius.ticket_landing.v1",
            "effort": "demo",
            "ticket": ticket,
            "candidate": candidate,
            "target": "effort/demo",
            "landed_at": "2026-10-05T12:30:00Z",
        }
        (evidence / f"acceptance-{ticket}.jsonl").write_text(json.dumps(acceptance, sort_keys=True) + "\n")
        (evidence / f"landing-{ticket}.jsonl").write_text(json.dumps(landing, sort_keys=True) + "\n")

    def commit_tickets(self):
        git(self.repo, "add", "docs")
        git(self.repo, "commit", "-qm", "tickets")

    def move_integration_branch_to_worktree(self) -> Path:
        checkout = self.root / "integration"
        git(self.repo, "checkout", "-q", "main")
        git(self.repo, "worktree", "add", "-q", str(checkout), "effort/demo")
        return checkout

    def move_effort_records_to_branch(self, integration: Path) -> Path:
        records = self.root / "records"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/demo-records", str(records), "effort/demo")
        effort_file = records / "docs/efforts/demo/EFFORT.md"
        effort_file.write_text(effort_file.read_text() + "Record branch: effort/demo-records\n")
        git(records, "add", "docs/efforts/demo/EFFORT.md")
        git(records, "commit", "-qm", "move effort records")
        git(self.repo, "rm", "-qr", "docs/efforts/demo")
        git(self.repo, "commit", "-qm", "remove effort records from main")
        git(integration, "rm", "-qr", "docs/efforts/demo")
        git(integration, "commit", "-qm", "remove effort records from code branch")
        return records

    def call(self, argv: list[str]) -> str:
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            self.code = lead_launch.main(["--repo", str(self.repo)] + argv)
        self.err = err.getvalue()
        return out.getvalue()

    def acceptance_call(self, argv: list[str]) -> str:
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            self.acceptance_code = lead_launch.acceptance.main(["--repo", str(self.repo)] + argv)
        self.acceptance_err = err.getvalue()
        return out.getvalue()

    def worker_call(self, argv: list[str]) -> str:
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            self.worker_code = lead_launch.worker.main(argv)
        self.worker_err = err.getvalue()
        return out.getvalue()

    def launch(self, ticket: str, cap: str = "3", extra: list[str] | None = None) -> dict:
        out = self.call(["launch", "demo", ticket, "--integration", "effort/demo", "--parallel-cap", cap] + (extra or []))
        self.assertEqual(self.code, lead_launch.OK, self.err)
        launched = json.loads(out)
        self.runs.append(Path(launched["run_dir"]))
        return launched

    def pin_candidate(self, launched: dict, *, session: bool) -> str:
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        worker_tree = Path(run_data["worktree"])
        (worker_tree / "work.txt").write_text("first candidate\n")
        git(worker_tree, "add", "work.txt")
        git(worker_tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker_tree, "rev-parse", "HEAD").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": git(worker_tree, "rev-parse", "HEAD^{tree}").strip(),
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        if session:
            run_data["session_id"] = "lead-rework-session"
        else:
            run_data.pop("session_id", None)
            (run_dir / "events.jsonl").write_text("")
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        return candidate

    def test_refuses_unresolved_unparseable_non_agent_and_over_cap_launches(self):
        self.write_ticket("01-blocker.md", ticket_text("01", "blocker"))
        self.write_ticket("02-blocked.md", ticket_text("02", "blocked", blocked_by="01"))
        self.write_ticket("03-bad-blocker.md", ticket_text("03", "bad-blocker", blocked_by="oops"))
        self.write_ticket("04-human.md", ticket_text("04", "human", triage="ready-for-human"))
        self.write_ticket("05-missing.md", ticket_text("05", "missing", triage=None))
        self.write_ticket("06-ready.md", ticket_text("06", "ready"))
        self.write_ticket("07-other-ready.md", ticket_text("07", "other-ready"))
        self.commit_tickets()

        self.call(["launch", "demo", "02", "--integration", "effort/demo"])
        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("unresolved blocker", self.err)

        self.call(["launch", "demo", "03", "--integration", "effort/demo"])
        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("unparseable blocker", self.err)

        self.call(["launch", "demo", "04", "--integration", "effort/demo"])
        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("triage is ready-for-human", self.err)

        self.call(["launch", "demo", "05", "--integration", "effort/demo"])
        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("triage is missing", self.err)

        self.launch("06", cap="1")
        self.call(["launch", "demo", "07", "--integration", "effort/demo", "--parallel-cap", "1"])
        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("parallel cap", self.err)

    def test_refuses_status_only_resolved_blocker_without_accepted_landed_candidate(self):
        self.write_ticket(
            "01-blocker.md",
            ticket_text("01", "blocker", status="resolved", owner="lead", acceptance="- [x] done\n"),
        )
        self.write_ticket("02-blocked.md", ticket_text("02", "blocked", blocked_by="01"))
        self.commit_tickets()

        self.call(["launch", "demo", "02", "--integration", "effort/demo"])

        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("unresolved blocker", self.err)

    def test_launch_starts_from_integration_tip_and_records_pinned_sources(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        main_tip = git(self.repo, "rev-parse", "main").strip()

        launched = self.launch("01")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        integration_tip = git(self.repo, "rev-parse", "effort/demo").strip()
        branch_tip = git(Path(launched["worktree"]), "rev-parse", "HEAD").strip()
        self.assertNotEqual(main_tip, integration_tip)
        self.assertEqual(run_data["base_sha"], integration_tip)
        self.assertEqual(branch_tip, integration_tip)
        missing_shared = subprocess.run(
            ["git", "-C", str(self.repo), "rev-parse", "--verify", "-q", launched["branch"]],
            capture_output=True,
            text=True,
        )
        self.assertEqual(missing_shared.stdout.strip(), "")

        record = run_data["lead_launch"]
        self.assertEqual(record["ticket"], "01")
        self.assertEqual(record["integration"], "effort/demo")
        ticket_source = record["sources"]["ticket"]
        self.assertEqual(ticket_source["commit"], integration_tip)
        self.assertEqual(ticket_source["path"], "docs/efforts/demo/tickets/01-ready.md")
        self.assertEqual(len(ticket_source["sha256"]), 64)
        self.assertEqual(record["sources"]["spec"]["path"], "docs/efforts/demo/SPEC.md")
        instruction_sources = record["sources"]["worker_instructions"]
        self.assertEqual(
            [source["path"] for source in instruction_sources],
            ["AGENTS.md", "CONSTITUTION.md", "docs/agents/agentic-engineering.md"],
        )
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("Ticket 01", prompt)
        self.assertIn(integration_tip, prompt)
        self.assertIn("ticket 01: ready for review", prompt)

        old_agents = instruction_sources[0]
        (self.repo / "AGENTS.md").write_text("agents v2\n")
        git(self.repo, "add", "AGENTS.md")
        git(self.repo, "commit", "-qm", "change instructions")
        refreshed = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(refreshed["lead_launch"]["sources"]["worker_instructions"][0], old_agents)
        old_bytes = subprocess.check_output(["git", "-C", str(self.repo), "show", f"{old_agents['commit']}:AGENTS.md"])
        self.assertEqual(lead_launch.sha256(old_bytes), old_agents["sha256"])

    def test_launch_reads_integration_branch_from_effort_record_when_flag_is_omitted(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()

        proc = subprocess.run(
            [
                sys.executable,
                str(Path(lead_launch.__file__)),
                "--repo",
                str(self.repo),
                "launch",
                "demo",
                "01",
            ],
            capture_output=True,
            text=True,
        )

        self.assertEqual(proc.returncode, lead_launch.OK, proc.stderr)
        launched = json.loads(proc.stdout)
        self.runs.append(Path(launched["run_dir"]))
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        integration_tip = git(self.repo, "rev-parse", "effort/demo").strip()
        self.assertEqual(run_data["base_sha"], integration_tip)
        self.assertEqual(run_data["lead_launch"]["integration"], "effort/demo")

    def test_launch_from_main_checkout_finds_effort_record_in_integration_worktree(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        integration = self.move_integration_branch_to_worktree()
        git(self.repo, "rm", "-qr", "docs/efforts")
        git(self.repo, "commit", "-qm", "main without effort record")
        main_tip = git(self.repo, "rev-parse", "main").strip()

        out = self.call(["launch", "demo", "01"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        launched = json.loads(out)
        self.runs.append(Path(launched["run_dir"]))
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        integration_tip = git(self.repo, "rev-parse", "effort/demo").strip()
        self.assertEqual(run_data["base_sha"], integration_tip)
        self.assertEqual(run_data["lead_launch"]["integration"], "effort/demo")
        self.assertIn("Status: claimed", (integration / "docs/efforts/demo/tickets/01-ready.md").read_text())
        self.assertEqual(git(self.repo, "rev-parse", "main").strip(), main_tip)
        self.assertFalse((self.repo / "docs/efforts/demo/EFFORT.md").exists())
        self.assertEqual(git(self.repo, "status", "--porcelain", "--untracked-files=all"), "")

    def test_launch_finds_integration_branch_in_a_linked_worktree(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        main_tip = git(self.repo, "rev-parse", "main").strip()
        integration = self.move_integration_branch_to_worktree()
        before_integration = git(integration, "rev-parse", "HEAD").strip()

        launched = self.launch("01")

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        after_integration = git(integration, "rev-parse", "HEAD").strip()
        branch_tip = git(Path(launched["worktree"]), "rev-parse", "HEAD").strip()
        self.assertNotEqual(before_integration, after_integration)
        self.assertEqual(git(integration, "log", "-1", "--format=%s").strip(), "Claim ticket 01 for worker")
        self.assertIn("Status: claimed", (integration / "docs/efforts/demo/tickets/01-ready.md").read_text())
        self.assertEqual(run_data["base_sha"], after_integration)
        self.assertEqual(run_data["lead_launch"]["base"], after_integration)
        self.assertEqual(branch_tip, after_integration)
        self.assertEqual(git(self.repo, "rev-parse", "main").strip(), main_tip)
        self.assertEqual(git(self.repo, "status", "--porcelain", "--untracked-files=all"), "")

    def test_launch_uses_record_branch_for_claims_and_code_branch_for_worker_base(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-ready.md", ticket_text("02", "ready"))
        self.write_ticket("03-ready.md", ticket_text("03", "ready"))
        self.commit_tickets()
        integration = self.move_integration_branch_to_worktree()
        records = self.move_effort_records_to_branch(integration)
        code_tip = git(self.repo, "rev-parse", "effort/demo").strip()

        out = self.call(["launch", "demo", "01", "02", "03", "--parallel-cap", "3"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        launched = json.loads(out)["launched"]
        self.runs.extend(Path(item["run_dir"]) for item in launched)
        self.assertEqual({json.loads((Path(item["run_dir"]) / "run.json").read_text())["base_sha"] for item in launched}, {code_tip})
        self.assertEqual({git(Path(item["worktree"]), "rev-parse", "HEAD").strip() for item in launched}, {code_tip})
        self.assertEqual(git(self.repo, "rev-parse", "effort/demo").strip(), code_tip)
        self.assertIn("Claim tickets 01, 02, 03 for workers", git(records, "log", "-1", "--format=%s").strip())
        for ticket_id in ("01", "02", "03"):
            self.assertIn("Status: claimed", (records / f"docs/efforts/demo/tickets/{ticket_id}-ready.md").read_text())
        run_data = json.loads((Path(launched[0]["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["lead_launch"]["sources"]["ticket"]["commit"], git(records, "rev-parse", "HEAD").strip())
        self.assertEqual(run_data["lead_launch"]["sources"]["spec"]["commit"], git(records, "rev-parse", "HEAD").strip())
        self.assertFalse((integration / "docs/efforts/demo").exists())
        self.assertEqual(git(integration, "status", "--porcelain", "--untracked-files=all"), "")
        self.assertEqual(git(records, "status", "--porcelain", "--untracked-files=all"), "")

    def test_failed_batch_launch_releases_tickets_that_did_not_launch_with_bad_tmpdir(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-ready.md", ticket_text("02", "ready"))
        self.commit_tickets()
        bad_tmp = self.root / "bad-tmp"
        bad_tmp.write_text("not a directory\n")
        old_tempdir = tempfile.tempdir
        try:
            tempfile.tempdir = None
            with mock.patch.dict(os.environ, {"TMPDIR": str(bad_tmp)}, clear=False):
                self.call(["launch", "demo", "01", "02", "--integration", "effort/demo", "--parallel-cap", "2"])
        finally:
            tempfile.tempdir = old_tempdir

        self.assertNotEqual(self.code, lead_launch.OK)
        first = (self.repo / "docs/efforts/demo/tickets/01-ready.md").read_text()
        second = (self.repo / "docs/efforts/demo/tickets/02-ready.md").read_text()
        self.assertIn("Status: open", first)
        self.assertIn("Status: open", second)
        self.assertFalse((self.root / "runs").exists())

    def test_launch_brief_temp_workspace_uses_runs_root_when_tmpdir_is_unset(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        forbidden = self.root / "forbidden-host-tmp"
        forbidden.mkdir()
        before = self.tree_listing(forbidden)
        old_tempdir = tempfile.tempdir
        try:
            tempfile.tempdir = str(forbidden)
            with mock.patch.dict(os.environ, {"CORENIUS_RUNS": str(self.root / "runs")}, clear=False):
                os.environ.pop("TMPDIR", None)
                launched = self.launch("01")
        finally:
            tempfile.tempdir = old_tempdir

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertTrue(Path(run_data["temp"]["tmpdir"]).is_relative_to(Path(launched["run_dir"])))
        self.assertEqual(self.tree_listing(forbidden), before)

    def test_launch_includes_accepted_report_findings_from_record_branch_after_move(self):
        blocker_candidate = git(self.repo, "rev-parse", "HEAD").strip()
        self.write_ticket(
            "01-done.md",
            ticket_text("01", "done", status="resolved", owner="lead", acceptance="- [x] done\n"),
        )
        self.write_ticket("02-ready.md", ticket_text("02", "ready", blocked_by="01"))
        self.write_landed_candidate_records("01", blocker_candidate)
        self.commit_tickets()
        integration = self.move_integration_branch_to_worktree()
        records = self.move_effort_records_to_branch(integration)
        evidence = records / "docs" / "efforts" / "demo" / "evidence"
        evidence.mkdir(parents=True, exist_ok=True)
        (evidence / "01-report-round-1.md").write_text(
            "# Ticket 01 report\n\n"
            "## For later tickets\n"
            "- to 02: Bring accepted report context.\n"
        )
        git(records, "add", "docs/efforts/demo/evidence/01-report-round-1.md")
        git(records, "commit", "-qm", "import ticket 01 report")

        launched = self.launch("02")

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertEqual(run_data["lead_launch"]["findings"][0]["sources"][0]["label"], "report 01 round 1")
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("- from report 01 round 1: Bring accepted report context.", prompt)
        self.assertFalse((integration / "docs/efforts/demo").exists())

    def test_refuses_when_integration_branch_is_checked_out_nowhere(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        git(self.repo, "checkout", "-q", "main")

        self.call(["launch", "demo", "01", "--integration", "effort/demo"])

        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("integration branch effort/demo is not checked out in any worktree", self.err)

    def test_launch_passes_and_records_access_profile_options(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        external = self.root / "external"
        external.mkdir()
        protected = self.root / "protected"
        protected.mkdir()
        (protected / "state.txt").write_text("keep\n")

        launched = self.launch(
            "01",
            extra=[
                "--network",
                "off",
                "--command-timeout",
                "77",
                "--external-add-path",
                str(external),
                "--external-protect-path",
                str(protected),
            ],
        )

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        expected = {
            "network": "off",
            "command_timeout_seconds": 77,
            "external_add_paths": [str(external.resolve())],
            "external_protect_paths": [str(protected.resolve())],
        }
        self.assertEqual(run_data["access_profile"]["network"], expected["network"])
        self.assertEqual(run_data["access_profile"]["command_timeout_seconds"], expected["command_timeout_seconds"])
        self.assertEqual(run_data["access_profile"]["external_add_paths"], expected["external_add_paths"])
        self.assertEqual(run_data["external_state"]["protected_paths"], expected["external_protect_paths"])
        self.assertEqual(run_data["lead_launch"]["access_profile"], expected)

    def test_launch_passes_and_records_runtime_option(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-ready.md", ticket_text("02", "ready"))
        self.commit_tickets()

        default = self.launch("01")
        default_run = json.loads((Path(default["run_dir"]) / "run.json").read_text())
        self.assertEqual(default_run["runtime"], "traecli")
        self.assertEqual(default_run["lead_launch"]["runtime"], "traecli")

        codex = self.launch("02", extra=["--runtime", "codex"])
        codex_run = json.loads((Path(codex["run_dir"]) / "run.json").read_text())
        self.assertEqual(codex_run["runtime"], "codex")
        self.assertEqual(codex_run["lead_launch"]["runtime"], "codex")

    def test_launch_records_missing_tmux_window_without_failing(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text("#!/bin/sh\necho tmux unavailable >&2\nexit 127\n")
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        try:
            launched = self.launch("01", extra=["--tmux-session", "demo-watch"])
        finally:
            os.environ["PATH"] = old_path

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        window = run_data["lead_launch"]["tmux_window"]
        self.assertFalse(window["available"])
        self.assertEqual(window["session"], "demo-watch")
        self.assertEqual(window["window"], "01-ready")
        self.assertIn("tmux unavailable", window["error"])

    def test_launch_creates_tmux_session_and_one_window_per_worker(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-other.md", ticket_text("02", "other"))
        self.commit_tickets()
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        log = self.root / "tmux.log"
        session_file = self.root / "tmux-session-created"
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text(textwrap.dedent(
            f"""\
            #!/bin/sh
            printf '%s\\n' "$*" >> {log}
            if [ "$1" = has-session ]; then
              test -e {session_file}
              exit $?
            fi
            if [ "$1" = new-session ]; then
              touch {session_file}
            fi
            exit 0
            """
        ))
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        try:
            out = self.call([
                "launch",
                "demo",
                "01",
                "02",
                "--integration",
                "effort/demo",
                "--parallel-cap",
                "2",
                "--tmux-session",
                "demo-watch",
            ])
        finally:
            os.environ["PATH"] = old_path

        self.assertEqual(self.code, lead_launch.OK, self.err)
        launched = json.loads(out)["launched"]
        self.runs.extend(Path(item["run_dir"]) for item in launched)
        for item in launched:
            window = json.loads((Path(item["run_dir"]) / "run.json").read_text())["lead_launch"]["tmux_window"]
            self.assertTrue(window["available"])
            self.assertEqual(window["session"], "demo-watch")
            self.assertEqual(window["target"], f"demo-watch:{window['window']}")
        calls = log.read_text().splitlines()
        self.assertEqual(sum(1 for line in calls if line.startswith("has-session -t demo-watch")), 2)
        self.assertEqual(sum(1 for line in calls if line.startswith("new-session -d -s demo-watch")), 1)
        self.assertEqual(sum(1 for line in calls if line.startswith("new-window -d -t demo-watch:")), 2)

    def test_interactive_launch_refuses_without_tmux_but_noninteractive_watch_still_records_failure(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-other.md", ticket_text("02", "other"))
        self.commit_tickets()
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text("#!/bin/sh\necho tmux unavailable >&2\nexit 127\n")
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        try:
            self.call([
                "launch",
                "demo",
                "01",
                "--integration",
                "effort/demo",
                "--interactive",
                "--tmux-session",
                "demo-live",
            ])
            self.assertEqual(self.code, lead_launch.REFUSED)
            self.assertIn("interactive launch requires tmux", self.err)

            launched = self.launch("02", extra=["--tmux-session", "demo-watch"])
        finally:
            os.environ["PATH"] = old_path

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertFalse(run_data["lead_launch"]["tmux_window"]["available"])
        self.assertIn("tmux unavailable", run_data["lead_launch"]["tmux_window"]["error"])

    def test_interactive_launch_records_tmux_target_runtime_command_and_shared_brief(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        log = self.root / "tmux-interactive.log"
        session_file = self.root / "tmux-session-created"
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text(textwrap.dedent(
            f"""\
            #!/bin/sh
            printf '%s\\n' "$*" >> {log}
            if [ "$1" = has-session ]; then
              test -e {session_file}
              exit $?
            fi
            if [ "$1" = new-session ]; then
              touch {session_file}
            fi
            if [ "$1" = new-window ]; then
              printf '%%42 %%pane-42\\n'
            fi
            exit 0
            """
        ))
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        try:
            out = self.call([
                "launch",
                "demo",
                "01",
                "--integration",
                "effort/demo",
                "--interactive",
                "--tmux-session",
                "demo-live",
                "--model",
                "test-model",
                "--reasoning-effort",
                "medium",
            ])
        finally:
            os.environ["PATH"] = old_path

        self.assertEqual(self.code, lead_launch.OK, self.err)
        launched = json.loads(out)
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        window = run_data["lead_launch"]["tmux_window"]
        self.assertTrue(run_data["lead_launch"]["interactive"])
        self.assertTrue(window["available"])
        self.assertEqual(window["target"], "demo-live:01-ready")
        self.assertEqual(window["pane_id"], "%pane-42")
        self.assertIsNone(run_data.get("pid"))
        calls = log.read_text()
        self.assertIn("new-window", calls)
        self.assertIn(f"-e TMPDIR={run_data['temp']['tmpdir']}", calls)
        self.assertIn(f"-e TMUX_TMPDIR={run_data['temp']['tmux_tmpdir']}", calls)
        self.assertIn(f"-e TMPPREFIX={run_data['temp']['tmpprefix']}", calls)
        script = Path(run_data["interactive_script"]).read_text()
        self.assertIn("timeout --foreground", script)
        self.assertIn(f"export TMPDIR={run_data['temp']['tmpdir']}", script)
        self.assertIn(f"export TMUX_TMPDIR={run_data['temp']['tmux_tmpdir']}", script)
        self.assertIn(f"export TMPPREFIX={run_data['temp']['tmpprefix']}", script)
        self.assertIn(f"{self.fake} -m test-model -c model_reasoning_effort=medium", script)
        self.assertIn("-c model_reasoning_summary=detailed", script)
        self.assertIn("--yolo", script)
        self.assertIn("exec ${SHELL:-sh}", script)
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("Read first, in this order:", prompt)
        self.assertIn("Report and hand-off:", prompt)
        self.assertIn("Do not edit your ticket file", prompt)
        self.assertIn("The tree must be clean after it", prompt)
        self.assertIn("mainline checkout, the integration worktree, the record worktree", prompt)

    def test_shared_brief_marks_ticket_specific_permissions_and_prohibitions(self):
        self.write_ticket(
            "01-ready.md",
            ticket_text(
                "01",
                "ready",
                description=(
                    "You may make real model runs for this ticket, and only these: one low-effort probe.\n"
                    "Do not run cargo for this ticket."
                ),
            ),
        )
        self.commit_tickets()

        launched = self.launch("01")

        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("## Ticket-Specific Permissions and Prohibitions", prompt)
        self.assertIn("[ticket] You may make real model runs for this ticket", prompt)
        self.assertIn("[ticket] Do not run cargo for this ticket.", prompt)
        self.assertIn("Ticket-specific permissions and prohibitions replace conflicting standing defaults.", prompt)

    def test_tmux_window_record_handles_missing_tmux_binary(self):
        with mock.patch.object(lead_launch.subprocess, "run", side_effect=FileNotFoundError("tmux")):
            record = lead_launch.tmux_window_record("demo-watch", "01-ready", self.repo, "watch")

        self.assertFalse(record["available"])
        self.assertEqual(record["session"], "demo-watch")
        self.assertIn("tmux", record["error"])

    def test_accounting_lists_stopped_and_retained_workers_and_slots(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-other.md", ticket_text("02", "other"))
        self.commit_tickets()
        first = self.launch("01")
        second = self.launch("02")
        first_temp = Path(json.loads((Path(first["run_dir"]) / "run.json").read_text())["temp"]["tmpdir"])
        second_temp = Path(json.loads((Path(second["run_dir"]) / "run.json").read_text())["temp"]["tmpdir"])
        self.stop_run(Path(first["run_dir"]))

        retain = self.call(["retain-worker", second["run_dir"], "--reason", "review round pending"])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        self.assertIn("retained", retain)
        out = self.call(["account", "demo", "--parallel-cap", "2", "--json"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        report = json.loads(out)
        by_ticket = {row["ticket"]: row for row in report["workers"]}
        self.assertEqual(report["available_slots"], 1)
        self.assertEqual(by_ticket["01"]["process_state"], "stopped")
        self.assertEqual(by_ticket["02"]["process_state"], "retained")
        self.assertTrue(by_ticket["02"]["slot_held"])
        self.assertGreater(by_ticket["02"]["disk_usage"]["worktree_bytes"], 0)
        self.assertGreater(by_ticket["02"]["disk_usage"]["git_dir_bytes"], 0)
        self.assertGreater(by_ticket["02"]["disk_usage"]["worker_repo_bytes"], by_ticket["02"]["disk_usage"]["worktree_bytes"])
        self.assertFalse(first_temp.exists())
        self.assertTrue(second_temp.exists())
        first_run = json.loads((Path(first["run_dir"]) / "run.json").read_text())
        self.assertIn("removed_at", first_run["temp"])

    def test_accounting_preserves_stopped_run_temp_when_keep_temp_is_set(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01", extra=["--keep-temp"])
        temp_dir = Path(json.loads((Path(launched["run_dir"]) / "run.json").read_text())["temp"]["tmpdir"])
        self.stop_run(Path(launched["run_dir"]))

        out = self.call(["account", "demo", "--parallel-cap", "1", "--json"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        row = json.loads(out)["workers"][0]
        self.assertEqual(row["process_state"], "stopped")
        self.assertFalse(row["slot_held"])
        self.assertTrue(temp_dir.exists())
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        self.assertTrue(run_data["temp"]["keep"])
        self.assertNotIn("removed_at", run_data["temp"])

    def test_accounting_reports_running_while_session_child_survives_wrapper(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        os.environ["FAKE_MODE"] = "spawn_child"
        launched = self.launch("01")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        session_id = os.getsid(run_data["pid"])
        try:
            deadline = time.time() + 5
            exit_file = Path(launched["run_dir"]) / "exit_code"
            while not exit_file.exists() and time.time() < deadline:
                time.sleep(0.05)
            self.assertTrue(exit_file.exists())
            self.assertTrue(live_session_pids(session_id))

            out = self.call(["account", "demo", "--parallel-cap", "1", "--json"])

            self.assertEqual(self.code, lead_launch.OK, self.err)
            row = json.loads(out)["workers"][0]
            self.assertEqual(row["process_state"], "running")
            self.assertTrue(row["slot_held"])
        finally:
            try:
                os.killpg(session_id, signal.SIGTERM)
            except ProcessLookupError:
                pass

    def test_monitor_wait_account_and_retain_use_record_branch_from_main_checkout(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-other.md", ticket_text("02", "other"))
        self.commit_tickets()
        integration = self.move_integration_branch_to_worktree()
        records = self.move_effort_records_to_branch(integration)
        first = self.launch("01")
        second = self.launch("02")
        self.stop_run(Path(first["run_dir"]))

        retain_text = self.call(["retain-worker", second["run_dir"], "--reason", "review round pending"])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        retain = json.loads(retain_text)
        self.assertTrue(retain["retained"])
        monitor_text = self.call(["monitor-tickets", "demo", "--json"])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        monitor = json.loads(monitor_text)
        wait_text = self.call(["wait", "demo", "01", "--timeout", "0", "--json"])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        wait = json.loads(wait_text)
        account_text = self.call(["account", "demo", "--parallel-cap", "2", "--json"])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        account = json.loads(account_text)

        by_ticket = {row["ticket"]: row for row in monitor["tickets"]}
        self.assertEqual(by_ticket["01"]["worker_state"], "stopped")
        self.assertEqual(by_ticket["02"]["worker_state"], "running")
        self.assertTrue(wait["triggered"])
        self.assertEqual(wait["states"]["01"], "stopped")
        account_by_ticket = {row["ticket"]: row for row in account["workers"]}
        self.assertEqual(account["available_slots"], 1)
        self.assertEqual(account_by_ticket["01"]["process_state"], "stopped")
        self.assertEqual(account_by_ticket["02"]["process_state"], "retained")
        self.assertFalse((self.repo / "docs" / "efforts" / "demo").exists())
        self.assertFalse((integration / "docs" / "efforts" / "demo").exists())
        self.assertIn("Status: claimed", (records / "docs/efforts/demo/tickets/01-ready.md").read_text())

    def test_ticket_monitor_marks_alive_candidate_as_finishing_and_wait_ignores_it(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        tree = Path(run_data["worktree"])
        (tree / "work.txt").write_text("candidate\n")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        head = git(tree, "rev-parse", "HEAD").strip()
        run_data["candidate"] = {
            "commit": head,
            "tree": git(tree, "rev-parse", "HEAD^{tree}").strip(),
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        (Path(launched["run_dir"]) / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")

        monitor = json.loads(self.call(["monitor-tickets", "demo", "--json"]))
        row = next(row for row in monitor["tickets"] if row["ticket"] == "01")
        self.assertEqual(row["worker_state"], "finishing")
        waited = json.loads(self.call(["wait", "demo", "01", "--timeout", "0.2", "--json"]))
        self.assertFalse(waited["triggered"])
        self.assertEqual(waited["states"]["01"], "finishing")

    def test_monitor_uses_recorded_integration_commit_and_supervisor_verdict_for_private_clone(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        tree = Path(run_data["worktree"])
        (tree / "work.txt").write_text("candidate\n")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        head = git(tree, "rev-parse", "HEAD").strip()
        run_data["candidate"] = {
            "commit": head,
            "tree": git(tree, "rev-parse", "HEAD^{tree}").strip(),
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        git(tree, "branch", "-D", "effort/demo")
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        (run_dir / "verdict.json").write_text(json.dumps({
            "verdict": "VIOLATED",
            "ready_for_review": False,
            "not_ready": ["boundary-violation"],
            "candidate": None,
        }) + "\n")

        monitor = json.loads(self.call(["monitor-tickets", "demo", "--json"]))

        self.assertEqual(self.code, lead_launch.OK, self.err)
        row = next(row for row in monitor["tickets"] if row["ticket"] == "01")
        self.assertEqual(row["ahead"], 1)
        self.assertTrue(row["branch_contains_integration"])
        self.assertNotEqual(row["worker_state"], "ready")
        self.assertEqual(row["worker_state"], "not-ready")
        missing_shared = subprocess.run(
            ["git", "-C", str(self.repo), "rev-parse", "--verify", "-q", launched["branch"]],
            capture_output=True,
            text=True,
        )
        self.assertEqual(missing_shared.stdout.strip(), "")

    def test_worker_state_vocabulary_includes_rework_waiting_needs_info_stopped_and_landed(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.write_ticket("02-other.md", ticket_text("02", "other"))
        self.commit_tickets()
        launched = self.launch("01")
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        tree = Path(run_data["worktree"])
        self.assertEqual(lead_launch.run_worker_state(run_data), "running")

        (tree / "work.txt").write_text("candidate\n")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(tree, "rev-parse", "HEAD").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": git(tree, "rev-parse", "HEAD^{tree}").strip(),
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        with mock.patch.object(lead_launch.worker, "process_state", return_value=(True, None)):
            self.assertEqual(lead_launch.run_worker_state(run_data), "finishing")

        with mock.patch.object(lead_launch.worker, "process_state", return_value=(False, 0)):
            self.assertEqual(lead_launch.run_worker_state(run_data), "ready")

        run_data["interactive"] = True
        run_data["lead_launch"]["tmux_window"] = {"available": True, "target": "demo-live:01-ready"}
        (run_dir / "exit_code").unlink(missing_ok=True)
        with mock.patch.object(lead_launch.worker, "process_state", return_value=(True, None)), mock.patch.object(
            lead_launch,
            "tmux_pane_text",
            return_value="Current activity: idle after marker\nContext remaining: 84%\n",
        ):
            self.assertEqual(lead_launch.run_worker_state(run_data), "ready")
        with mock.patch.object(lead_launch.worker, "process_state", return_value=(True, None)), mock.patch.object(
            lead_launch,
            "tmux_pane_text",
            return_value="Current activity: editing files\nContext remaining: 80%\nesc to interrupt\n",
        ):
            self.assertEqual(lead_launch.run_worker_state(run_data), "finishing")
        (run_dir / "exit_code").write_text("0\n")

        run_data["candidates"][-1]["review"] = {"accepted": False}
        self.assertEqual(lead_launch.run_worker_state(run_data), "rework")

        run_data["rework_deliveries"] = [{
            "id": "review-1",
            "kind": "review",
            "round": 1,
            "counts_toward_cap": True,
            "candidate": candidate,
            "receipt": {"acknowledged": True},
        }]
        (tree / "work.txt").write_text("fix started\n")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "fix started")
        self.assertEqual(lead_launch.run_worker_state(run_data), "waiting for re-integration")
        with mock.patch.object(lead_launch.worker, "process_state", return_value=(True, None)), mock.patch.object(
            lead_launch,
            "tmux_pane_text",
            return_value="Current activity: idle after marker\nContext remaining: 84%\n",
        ):
            self.assertEqual(lead_launch.run_worker_state(run_data), "waiting for re-integration")
        with mock.patch.object(lead_launch.worker, "process_state", return_value=(True, None)), mock.patch.object(
            lead_launch,
            "tmux_pane_text",
            return_value="Current activity: editing files\nContext remaining: 80%\nesc to interrupt\n",
        ):
            self.assertEqual(lead_launch.run_worker_state(run_data), "waiting for re-integration")
        (tree / "work.txt").write_text("fix still editing\n")
        self.assertEqual(lead_launch.run_worker_state(run_data), "rework")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "finish dirty edit")

        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "--allow-empty", "-qm", "ticket 01: needs-info")
        run_data.pop("candidate", None)
        run_data["candidates"] = []
        run_data["rework_deliveries"] = []
        self.assertEqual(lead_launch.run_worker_state(run_data), "needs-info")

        stopped = self.launch("02")
        stopped_run = json.loads((Path(stopped["run_dir"]) / "run.json").read_text())
        self.stop_run(Path(stopped["run_dir"]))
        (Path(stopped["run_dir"]) / "exit_code").write_text("143\n")
        self.assertEqual(lead_launch.run_worker_state(stopped_run), "stopped")
        self.assertEqual(lead_launch.run_worker_state(stopped_run, mock.Mock(status="resolved")), "landed")

    def test_wait_does_not_return_on_rework_but_returns_on_waiting_for_reintegration(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        tree = Path(run_data["worktree"])
        (tree / "work.txt").write_text("candidate\n")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(tree, "rev-parse", "HEAD").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": git(tree, "rev-parse", "HEAD^{tree}").strip(),
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"], "review": {"accepted": False}}]
        run_data["rework_deliveries"] = [{
            "id": "review-1",
            "kind": "review",
            "round": 1,
            "counts_toward_cap": True,
            "candidate": candidate,
            "receipt": {"acknowledged": True},
        }]
        (run_dir / "exit_code").write_text("0\n")
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")

        rework_wait = json.loads(self.call(["wait", "demo", "01", "--timeout", "0", "--json"]))
        self.assertFalse(rework_wait["triggered"])
        self.assertEqual(rework_wait["states"], {"01": "rework"})
        self.assertEqual(self.code, lead_launch.REFUSED)

        (tree / "work.txt").write_text("fix\n")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "fix started")
        reintegration_wait = json.loads(self.call(["wait", "demo", "01", "--timeout", "0", "--json"]))
        self.assertTrue(reintegration_wait["triggered"])
        self.assertEqual(reintegration_wait["trigger"]["state"], "waiting for re-integration")
        self.assertEqual(self.code, lead_launch.OK, self.err)

    def test_ticket_monitor_reports_counts_diff_size_and_private_clone_tip_state(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready", declared=["work.txt"]))
        self.commit_tickets()
        launched = self.launch("01")
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        tree = Path(run_data["worktree"])
        (tree / "work.txt").write_text("candidate\n")
        git(tree, "add", "work.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(tree, "rev-parse", "HEAD").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": git(tree, "rev-parse", "HEAD^{tree}").strip(),
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"], "review": {"accepted": False}}]
        (tree / "outside.txt").write_text("outside\n")
        git(tree, "add", "outside.txt")
        git(tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "post-review work")
        git(tree, "branch", "-D", "effort/demo")
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")

        monitor = json.loads(self.call(["monitor-tickets", "demo", "--json"]))

        self.assertEqual(self.code, lead_launch.OK, self.err)
        row = next(row for row in monitor["tickets"] if row["ticket"] == "01")
        self.assertEqual(row["ahead"], 2)
        self.assertTrue(row["branch_contains_integration"])
        self.assertEqual(row["outside_declared_count"], 1)
        self.assertEqual(row["diff_since_reviewed_candidate_size"], 1)
        self.assertFalse(row["tip_rewritten_since_pinned_or_reviewed"])
        self.assertEqual(row["outside_declared_paths"], ["outside.txt"])
        missing_shared = subprocess.run(
            ["git", "-C", str(self.repo), "rev-parse", "--verify", "-q", launched["branch"]],
            capture_output=True,
            text=True,
        )
        self.assertEqual(missing_shared.stdout.strip(), "")

    def test_wait_timeout_returns_refusal_when_no_ticket_leaves_running_states(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        self.launch("01")

        waited = json.loads(self.call(["wait", "demo", "01", "--timeout", "0", "--json"]))

        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertFalse(waited["triggered"])
        self.assertEqual(waited["states"]["01"], "running")

    def test_worker_monitor_reports_pane_activity_disk_and_last_delivery(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        run_data["lead_launch"]["tmux_window"] = {"available": True, "target": "demo-live:01-ready"}
        run_data["rework_deliveries"] = [{
            "id": "review-1",
            "delivered_at": "2026-10-05T12:01:00Z",
            "receipt": {"transport": "terminal", "sent_at": "2026-10-05T12:01:00Z", "acknowledged": False},
        }]
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")

        with mock.patch.object(
            lead_launch,
            "tmux_pane_text",
            return_value="Current activity: editing files\nContext remaining: 81%\n",
        ):
            out = self.call(["monitor-workers", "demo", "--json"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        report = json.loads(out)
        row = next(row for row in report["workers"] if row["ticket"] == "01")
        self.assertEqual(row["process"]["state"], "running")
        self.assertGreaterEqual(row["process"]["uptime_seconds"], 0)
        self.assertEqual(row["current_activity"], "editing files")
        self.assertEqual(row["context_remaining"], "81%")
        self.assertGreater(row["disk_usage"]["worker_repo_bytes"], 0)
        self.assertEqual(row["last_delivery"]["id"], "review-1")
        self.assertFalse(row["last_delivery"]["acknowledged"])

    def test_runtime_prompt_pane_is_blocked_and_wakes_wait(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        run_data["interactive"] = True
        run_data["lead_launch"]["interactive"] = True
        run_data["lead_launch"]["tmux_window"] = {"available": True, "target": "demo-live:01-ready"}
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")

        with mock.patch.object(lead_launch, "tmux_pane_text", return_value=TRUST_PROMPT):
            monitor = json.loads(self.call(["monitor-tickets", "demo", "--json"]))
            wait = json.loads(self.call(["wait", "demo", "01", "--timeout", "0", "--json"]))

        row = next(row for row in monitor["tickets"] if row["ticket"] == "01")
        self.assertEqual(row["worker_state"], "blocked-on-prompt")
        self.assertIn("Do you trust the contents", row["blocked_prompt"])
        self.assertTrue(wait["triggered"])
        self.assertEqual(wait["trigger"]["state"], "blocked-on-prompt")
        self.assertIn("Do you trust the contents", wait["trigger"]["prompt"])

    def test_real_traecli_idle_pane_is_ready_and_context_is_percent_only(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        run_dir = Path(launched["run_dir"])
        run_data = json.loads((run_dir / "run.json").read_text())
        worktree = Path(run_data["worktree"])
        (worktree / "work.txt").write_text("done\n")
        git(worktree, "add", "work.txt")
        git(worktree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        run_data["interactive"] = True
        run_data["runtime"] = "traecli"
        run_data["lead_launch"]["interactive"] = True
        run_data["lead_launch"]["tmux_window"] = {"available": True, "target": "demo-live:01-ready"}
        (run_dir / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")

        with mock.patch.object(lead_launch, "tmux_pane_text", return_value=REAL_TRAECLI_IDLE_PANE_TAIL):
            tickets = json.loads(self.call(["monitor-tickets", "demo", "--json"]))
            workers = json.loads(self.call(["monitor-workers", "demo", "--json"]))

        ticket_row = next(row for row in tickets["tickets"] if row["ticket"] == "01")
        worker_row = next(row for row in workers["workers"] if row["ticket"] == "01")
        self.assertEqual(ticket_row["worker_state"], "ready")
        self.assertEqual(worker_row["state"], "ready")
        self.assertEqual(worker_row["context_remaining"], "95%")
        self.assertEqual(lead_launch.parse_pane_activity(REAL_TRAECLI_IDLE_PANE_TAIL)[1], "95%")

    def test_open_session_creates_only_ticket_and_worker_monitor_windows(self):
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        log = self.root / "tmux-open-session.log"
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text(textwrap.dedent(
            f"""\
            #!/bin/sh
            printf '%s\\n' "$*" >> {log}
            exit 0
            """
        ))
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        try:
            out = self.call(["open-session", "demo", "--tmux-session", "demo-live", "--interval", "7"])
        finally:
            os.environ["PATH"] = old_path

        self.assertEqual(self.code, lead_launch.OK, self.err)
        opened = json.loads(out)
        self.assertEqual(opened["windows"], ["tickets", "workers"])
        calls = log.read_text().splitlines()
        self.assertEqual(len([line for line in calls if line.startswith("new-session ")]), 1)
        self.assertEqual(len([line for line in calls if line.startswith("new-window ")]), 1)
        self.assertIn("monitor-tickets demo --interval 7", calls[0])
        self.assertIn("monitor-workers demo --interval 7", calls[1])

    def test_launch_includes_findings_addressed_to_the_ticket(self):
        blocker_candidate = git(self.repo, "rev-parse", "HEAD").strip()
        self.write_ticket(
            "01-source.md",
            ticket_text(
                "01",
                "source",
                status="resolved",
                owner="lead",
                acceptance="- [x] done\n",
                for_later="- to 02: Preserve the parser boundary.\n",
            ),
        )
        self.write_ticket("02-ready.md", ticket_text("02", "ready", blocked_by="01"))
        self.write_landed_candidate_records("01", blocker_candidate)
        self.write_report(
            "01-report.md",
            "# Ticket 01 report\n\n"
            "## For later tickets\n"
            "- to 02: Report source context.\n"
        )
        self.write_report("lead-handoffs.md", "# Lead handoffs\n\n- to 02: Lead source context.\n")
        self.commit_tickets()

        launched = self.launch("02")

        run_data = json.loads((Path(launched["run_dir"]) / "run.json").read_text())
        findings = run_data["lead_launch"]["findings"]
        self.assertEqual([finding["text"] for finding in findings], [
            "Preserve the parser boundary.",
            "Report source context.",
            "Lead source context.",
        ])
        self.assertEqual([source["label"] for source in findings[0]["sources"]], ["ticket 01-source"])
        self.assertEqual([source["label"] for source in findings[1]["sources"]], ["report 01"])
        self.assertEqual([source["label"] for source in findings[2]["sources"]], ["lead"])
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("## Findings From Earlier Tickets", prompt)
        self.assertIn("- from ticket 01-source: Preserve the parser boundary.", prompt)
        self.assertIn("- from report 01: Report source context.", prompt)
        self.assertIn("- from lead: Lead source context.", prompt)

    def test_question_subcommand_moves_ticket_to_needs_info_and_lints(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()

        out = self.call(["question", "demo", "01", "--owner", "worker-01", "--question", "Need credentials."])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        self.assertIn("01-ready.md", out)
        text = (self.repo / "docs/efforts/demo/tickets/01-ready.md").read_text()
        self.assertIn("Triage: needs-info", text)
        self.assertIn("worker-01: question: Need credentials.", text)
        self.assertEqual(
            subprocess.run(
                [sys.executable, str(Path(lead_launch.__file__).with_name("tracker.py")), "--repo", str(self.repo), "lint", "demo"],
                capture_output=True,
                text=True,
            ).returncode,
            0,
        )

    def test_rework_starts_fresh_worker_when_original_session_cannot_resume(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        run_data = json.loads((original / "run.json").read_text())
        worker_tree = Path(run_data["worktree"])
        (worker_tree / "work.txt").write_text("first candidate\n")
        git(worker_tree, "add", "work.txt")
        git(worker_tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker_tree, "rev-parse", "HEAD").strip()
        tree = git(worker_tree, "rev-parse", "HEAD^{tree}").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": tree,
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        run_data.pop("session_id", None)
        (original / "events.jsonl").write_text("")
        (original / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        findings = self.root / "review-1.md"
        findings.write_text("Fix the result.\n")
        self.stop_run(original)

        out = self.call(["rework", str(original), "--findings-file", str(findings)])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        result = json.loads(out)
        self.assertEqual(result["mode"], "fresh-worker")
        self.assertIn("no session id", result["reason"])
        fresh = Path(result["fresh_run_dir"])
        self.runs.append(fresh)
        prompt = (fresh / "prompt.md").read_text()
        self.assertIn(candidate, prompt)
        self.assertIn(str(findings.resolve()), prompt)
        refreshed = json.loads((original / "run.json").read_text())
        self.assertEqual(refreshed["rework_deliveries"][-1]["receipt"]["transport"], "fresh-worker")
        self.assertEqual(refreshed["rework_deliveries"][-1]["receipt"]["fresh_run_dir"], str(fresh))

    def test_rework_fresh_worker_carries_launch_boundary_profile(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        external = self.root / "external-add"
        external.mkdir()
        protected = self.root / "external-protected"
        protected.mkdir()
        (protected / "state.txt").write_text("before\n")
        other = self.root / "other"
        other.mkdir()
        git(other, "init", "-q", "-b", "main")
        git(other, "config", "user.email", "other@corenius")
        git(other, "config", "user.name", "other")
        (other / "state.txt").write_text("protected\n")
        git(other, "add", "state.txt")
        git(other, "commit", "-qm", "protected")
        launched = self.launch(
            "01",
            extra=[
                "--sandbox",
                "read-only",
                "--network",
                "off",
                "--command-timeout",
                "77",
                "--external-add-path",
                str(external),
                "--external-protect-path",
                str(protected),
                "--protect",
                str(other),
            ],
        )
        original = Path(launched["run_dir"])
        self.pin_candidate(launched, session=False)
        findings = self.root / "review-profile.md"
        findings.write_text("Fix the result.\n")
        self.stop_run(original)

        out = self.call(["rework", str(original), "--findings-file", str(findings)])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        result = json.loads(out)
        fresh = Path(result["fresh_run_dir"])
        self.runs.append(fresh)
        fresh_data = json.loads((fresh / "run.json").read_text())
        self.assertEqual(fresh_data["sandbox"], "read-only")
        self.assertEqual(fresh_data["handoff_mode"], "marker")
        self.assertEqual(fresh_data["handoff_marker"], "ticket 01: ready for review")
        self.assertEqual(fresh_data["access_profile"]["network"], "off")
        self.assertEqual(fresh_data["access_profile"]["command_timeout_seconds"], 77)
        self.assertEqual(fresh_data["access_profile"]["external_add_paths"], [str(external.resolve())])
        self.assertEqual(fresh_data["external_state"]["protected_paths"], [str(protected.resolve())])
        self.assertIn(str(other.resolve()), [entry["path"] for entry in fresh_data["protected"]])

    def test_fourth_lead_rework_fresh_worker_round_is_refused_and_triaged(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        self.pin_candidate(launched, session=False)
        findings = self.root / "review-cap.md"
        findings.write_text("Fix the result.\n")
        self.stop_run(original)

        for _ in range(3):
            out = self.call(["rework", str(original), "--findings-file", str(findings)])
            self.assertEqual(self.code, lead_launch.OK, self.err)
            self.runs.append(Path(json.loads(out)["fresh_run_dir"]))

        out = self.call(["rework", str(original), "--findings-file", str(findings)])

        self.assertEqual(out, "")
        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("review-driven rework cap exhausted", self.err)
        ticket = self.repo / "docs/efforts/demo/tickets/01-ready.md"
        self.assertIn("Triage: ready-for-human", ticket.read_text())
        records = [
            json.loads(line)
            for line in (self.repo / "docs/efforts/demo/evidence/rework-01.jsonl").read_text().splitlines()
        ]
        self.assertEqual([record["round"] for record in records], [1, 2, 3])

    def test_fourth_lead_rework_same_session_round_is_refused_before_resume(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        candidate = self.pin_candidate(launched, session=True)
        findings = self.root / "review-resume-cap.md"
        findings.write_text("Fix the result.\n")
        for _ in range(3):
            self.acceptance_call([
                "request-rework",
                "demo",
                "01",
                "--candidate",
                candidate,
                "--findings-file",
                str(findings),
                "--actor",
                "lead",
            ])
            self.assertEqual(self.acceptance_code, lead_launch.acceptance.OK, self.acceptance_err)

        out = self.call(["rework", str(original), "--findings-file", str(findings)])

        self.assertEqual(out, "")
        self.assertEqual(self.code, lead_launch.REFUSED)
        self.assertIn("review-driven rework cap exhausted", self.err)

    def test_mixed_rework_commands_share_one_ticket_counter(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        candidate = self.pin_candidate(launched, session=False)
        findings = self.root / "review-mixed.md"
        findings.write_text("Fix the result.\n")
        self.stop_run(original)

        self.acceptance_call([
            "request-rework",
            "demo",
            "01",
            "--candidate",
            candidate,
            "--findings-file",
            str(findings),
            "--actor",
            "lead",
        ])
        self.assertEqual(self.acceptance_code, lead_launch.acceptance.OK, self.acceptance_err)
        self.worker_call([
            "deliver-findings",
            str(original),
            "--findings-file",
            str(findings),
            "--terminal-input-file",
            str(self.root / "terminal.txt"),
            "--ack-timeout",
            "0",
        ])
        self.assertEqual(self.worker_code, lead_launch.worker.DEGRADED, self.worker_err)
        out = self.call(["rework", str(original), "--findings-file", str(findings)])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        self.runs.append(Path(json.loads(out)["fresh_run_dir"]))

        self.acceptance_call([
            "request-rework",
            "demo",
            "01",
            "--candidate",
            candidate,
            "--findings-file",
            str(findings),
            "--actor",
            "lead",
        ])

        self.assertEqual(self.acceptance_code, lead_launch.acceptance.ERROR)
        self.assertIn("review-driven rework cap exhausted", self.acceptance_err)
        records = [
            json.loads(line)
            for line in (self.repo / "docs/efforts/demo/evidence/rework-01.jsonl").read_text().splitlines()
        ]
        self.assertEqual([record["round"] for record in records], [1, 2, 3])

    def test_worker_deliver_findings_refuses_fourth_shared_round_and_triages(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        candidate = self.pin_candidate(launched, session=False)
        findings = self.root / "review-worker-cap.md"
        findings.write_text("Fix the result.\n")
        for _ in range(3):
            self.acceptance_call([
                "request-rework",
                "demo",
                "01",
                "--candidate",
                candidate,
                "--findings-file",
                str(findings),
                "--actor",
                "lead",
            ])
            self.assertEqual(self.acceptance_code, lead_launch.acceptance.OK, self.acceptance_err)

        out = self.worker_call([
            "deliver-findings",
            str(original),
            "--findings-file",
            str(findings),
            "--terminal-input-file",
            str(self.root / "terminal-cap.txt"),
            "--ack-timeout",
            "0",
        ])

        self.assertEqual(out, "")
        self.assertEqual(self.worker_code, 64)
        self.assertIn("review-driven rework cap exhausted", self.worker_err)
        self.assertIn("Triage: ready-for-human", (self.repo / "docs/efforts/demo/tickets/01-ready.md").read_text())

    def test_rework_delivers_findings_to_interactive_tmux_pane(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        run_data = json.loads((original / "run.json").read_text())
        worker_tree = Path(run_data["worktree"])
        (worker_tree / "work.txt").write_text("first candidate\n")
        git(worker_tree, "add", "work.txt")
        git(worker_tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker_tree, "rev-parse", "HEAD").strip()
        tree = git(worker_tree, "rev-parse", "HEAD^{tree}").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": tree,
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        run_data["interactive"] = True
        run_data["session_id"] = "resume-available-but-pane-is-authoritative"
        run_data["lead_launch"]["tmux_window"] = {
            "available": True,
            "target": "demo-live:01-ready",
            "pane_id": "%pane-1",
        }
        (original / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        findings = self.root / "review-1.md"
        findings.write_text("Fix the result.\n")
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        log = self.root / "tmux-rework.log"
        pane = self.root / "pane.txt"
        pane.write_text("")
        fake_tmux = fake_bin / "tmux"
        fake_tmux.write_text(textwrap.dedent(
            f"""\
            #!/bin/sh
            printf '%s\\n' "$*" >> {log}
            if [ "$1" = capture-pane ]; then
              cat {pane}
              exit 0
            fi
            if [ "$1" = send-keys ] && [ "$4" = -l ]; then
              printf '%s\\n' "$5" > {pane}
            fi
            exit 0
            """
        ))
        fake_tmux.chmod(fake_tmux.stat().st_mode | stat.S_IXUSR)
        old_path = os.environ["PATH"]
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{old_path}"
        try:
            out = self.call([
                "rework",
                str(original),
                "--findings-file",
                str(findings),
                "--ack-timeout",
                "0.2",
            ])
        finally:
            os.environ["PATH"] = old_path

        self.assertEqual(self.code, lead_launch.OK, self.err)
        result = json.loads(out)
        self.assertEqual(result["mode"], "terminal")
        self.assertEqual(result["delivery"]["receipt"]["transport"], "terminal")
        self.assertEqual(result["delivery"]["receipt"]["terminal_target"], "%pane-1")
        self.assertFalse(result["delivery"]["receipt"]["acknowledged"])
        self.assertIn("no worker acknowledgement", result["delivery"]["receipt"]["reason"])
        self.assertNotIn("resume", result)
        calls = log.read_text()
        self.assertIn("send-keys -t %pane-1 -l", calls)
        self.assertIn(str(findings.resolve()), pane.read_text())
        refreshed = json.loads((original / "run.json").read_text())
        self.assertTrue(refreshed["rework_deliveries"][-1]["counts_toward_cap"])

    def test_stale_base_rework_starts_fresh_worker_from_moved_integration_tip(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        run_data = json.loads((original / "run.json").read_text())
        worker_tree = Path(run_data["worktree"])
        (worker_tree / "work.txt").write_text("first candidate\n")
        git(worker_tree, "add", "work.txt")
        git(worker_tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker_tree, "rev-parse", "HEAD").strip()
        tree = git(worker_tree, "rev-parse", "HEAD^{tree}").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": tree,
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        run_data.pop("session_id", None)
        (original / "events.jsonl").write_text("")
        (original / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        (self.repo / "integration-moved.txt").write_text("moved\n")
        git(self.repo, "add", "integration-moved.txt")
        git(self.repo, "commit", "-qm", "move integration")
        moved_tip = git(self.repo, "rev-parse", "effort/demo").strip()
        findings = self.root / "stale-base.md"
        findings.write_text("Re-integrate on the moved base.\n")
        self.stop_run(original)

        out = self.call(["rework", str(original), "--findings-file", str(findings), "--kind", "stale-base"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        result = json.loads(out)
        fresh = Path(result["fresh_run_dir"])
        self.runs.append(fresh)
        fresh_data = json.loads((fresh / "run.json").read_text())
        refreshed = json.loads((original / "run.json").read_text())
        self.assertEqual(result["mode"], "fresh-worker")
        self.assertEqual(fresh_data["base_sha"], moved_tip)
        self.assertEqual(fresh_data["lead_launch"]["previous_candidate"], candidate)
        self.assertEqual(fresh_data["lead_launch"]["rework_base"], moved_tip)
        self.assertEqual(git(self.repo, "rev-parse", launched["branch"]).strip(), candidate)
        self.assertEqual(refreshed["rework_deliveries"][-1]["kind"], "stale-base")
        self.assertFalse(refreshed["rework_deliveries"][-1]["counts_toward_cap"])

    def test_stale_base_rework_resumes_available_worker_without_counting_review_cap(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        run_data = json.loads((original / "run.json").read_text())
        worker_tree = Path(run_data["worktree"])
        (worker_tree / "work.txt").write_text("first candidate\n")
        git(worker_tree, "add", "work.txt")
        git(worker_tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker_tree, "rev-parse", "HEAD").strip()
        tree = git(worker_tree, "rev-parse", "HEAD^{tree}").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": tree,
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        run_data["session_id"] = "stale-resume-session"
        (original / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        (self.repo / "integration-moved.txt").write_text("moved\n")
        git(self.repo, "add", "integration-moved.txt")
        git(self.repo, "commit", "-qm", "move integration")
        moved_tip = git(self.repo, "rev-parse", "effort/demo").strip()
        findings = self.root / "stale-base.md"
        findings.write_text("Re-integrate on the moved base.\n")
        os.environ["FAKE_MODE"] = "done"

        out = self.call(["rework", str(original), "--findings-file", str(findings), "--kind", "stale-base"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        result = json.loads(out)
        self.assertEqual(result["mode"], "resumed")
        self.assertEqual(result["delivery"]["kind"], "stale-base")
        self.assertFalse(result["delivery"]["counts_toward_cap"])
        self.assertEqual(result["resume"]["refreshed_integration"], {"branch": "effort/demo", "commit": moved_tip})
        refreshed = json.loads((original / "run.json").read_text())
        self.assertEqual(refreshed["branch_baselines"]["effort/demo"], moved_tip)
        self.assertEqual(refreshed["shared_branch_baselines"]["effort/demo"], moved_tip)

    def test_rework_from_main_checkout_finds_rejected_review_in_effort_worktree(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        run_data = json.loads((original / "run.json").read_text())
        worker_tree = Path(run_data["worktree"])
        (worker_tree / "work.txt").write_text("first candidate\n")
        git(worker_tree, "add", "work.txt")
        git(worker_tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker_tree, "rev-parse", "HEAD").strip()
        tree = git(worker_tree, "rev-parse", "HEAD^{tree}").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": tree,
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        run_data.pop("session_id", None)
        (original / "events.jsonl").write_text("")
        (original / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        integration = self.move_integration_branch_to_worktree()
        git(self.repo, "rm", "-qr", "docs/efforts")
        git(self.repo, "commit", "-qm", "main without effort record")
        findings = integration / "docs" / "efforts" / "demo" / "evidence" / "review-main.md"
        findings.parent.mkdir(parents=True, exist_ok=True)
        findings.write_text("Fix the result.\n")
        review = {
            "schema": "corenius.acceptance.v1",
            "effort": "demo",
            "ticket": "01",
            "candidate": candidate,
            "reviewer": "lead",
            "reviewer_context": "lead",
            "axes": {"standards": "passed", "spec": "failed"},
            "required_checks": [],
            "owner_waiver": None,
            "claim_sampling": None,
            "accepted": False,
            "round": 1,
            "findings": {"path": "docs/efforts/demo/evidence/review-main.md", "sha256": lead_launch.worker.digest_file(findings)},
            "recorded_at": "2026-10-05T12:00:00Z",
        }
        with (findings.parent / "acceptance-01.jsonl").open("a") as handle:
            handle.write(json.dumps(review, sort_keys=True) + "\n")
        self.stop_run(original)

        out = self.call(["rework", str(original), "--findings-file", str(findings)])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        result = json.loads(out)
        self.assertEqual(result["mode"], "fresh-worker")
        self.runs.append(Path(result["fresh_run_dir"]))
        refreshed = json.loads((original / "run.json").read_text())
        self.assertEqual(refreshed["candidates"][0]["review"]["accepted"], False)
        self.assertEqual(refreshed["candidates"][0]["review"]["findings"]["sha256"], review["findings"]["sha256"])
        self.assertFalse((self.repo / "docs" / "efforts" / "demo").exists())
        self.assertEqual(git(self.repo, "status", "--porcelain", "--untracked-files=all"), "")

    def test_rework_from_main_checkout_finds_rejected_review_on_record_branch(self):
        self.write_ticket("01-ready.md", ticket_text("01", "ready"))
        self.commit_tickets()
        launched = self.launch("01")
        original = Path(launched["run_dir"])
        run_data = json.loads((original / "run.json").read_text())
        worker_tree = Path(run_data["worktree"])
        (worker_tree / "work.txt").write_text("first candidate\n")
        git(worker_tree, "add", "work.txt")
        git(worker_tree, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker_tree, "rev-parse", "HEAD").strip()
        tree = git(worker_tree, "rev-parse", "HEAD^{tree}").strip()
        run_data["candidate"] = {
            "commit": candidate,
            "tree": tree,
            "base": run_data["base_sha"],
            "brief_digest": run_data["brief_digest"],
        }
        run_data["candidates"] = [{"round": 0, "kind": "initial", "candidate": run_data["candidate"]}]
        run_data.pop("session_id", None)
        (original / "events.jsonl").write_text("")
        (original / "run.json").write_text(json.dumps(run_data, indent=2) + "\n")
        integration = self.move_integration_branch_to_worktree()
        records = self.move_effort_records_to_branch(integration)
        findings = records / "docs" / "efforts" / "demo" / "evidence" / "review-main.md"
        findings.parent.mkdir(parents=True, exist_ok=True)
        findings.write_text("Fix the result.\n")
        review = {
            "schema": "corenius.acceptance.v1",
            "effort": "demo",
            "ticket": "01",
            "candidate": candidate,
            "reviewer": "lead",
            "reviewer_context": "lead",
            "axes": {"standards": "passed", "spec": "failed"},
            "required_checks": [],
            "owner_waiver": None,
            "claim_sampling": None,
            "accepted": False,
            "round": 1,
            "findings": {"path": "docs/efforts/demo/evidence/review-main.md", "sha256": lead_launch.worker.digest_file(findings)},
            "recorded_at": "2026-10-05T12:00:00Z",
        }
        with (findings.parent / "acceptance-01.jsonl").open("a") as handle:
            handle.write(json.dumps(review, sort_keys=True) + "\n")
        self.stop_run(original)

        out = self.call(["rework", str(original), "--findings-file", str(findings)])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        result = json.loads(out)
        self.assertEqual(result["mode"], "fresh-worker")
        self.runs.append(Path(result["fresh_run_dir"]))
        refreshed = json.loads((original / "run.json").read_text())
        self.assertEqual(refreshed["candidates"][0]["review"]["accepted"], False)
        self.assertEqual(refreshed["candidates"][0]["review"]["findings"]["sha256"], review["findings"]["sha256"])
        self.assertFalse((self.repo / "docs" / "efforts" / "demo").exists())
        self.assertFalse((integration / "docs" / "efforts" / "demo").exists())

    def stop_run(self, run_dir: Path) -> None:
        try:
            run_data = json.loads((run_dir / "run.json").read_text())
            session_id = run_data.get("process_session_id")
            if session_id is not None:
                for pid in live_session_pids(int(session_id)):
                    try:
                        os.kill(pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
            if run_data.get("pid") is not None:
                try:
                    os.killpg(os.getpgid(run_data["pid"]), signal.SIGTERM)
                except ProcessLookupError:
                    pass
            deadline = time.time() + 3
            while lead_launch.worker.process_state(run_data)[0] and time.time() < deadline:
                time.sleep(0.05)
            if lead_launch.worker.process_state(run_data)[0] and session_id is not None:
                for pid in live_session_pids(int(session_id)):
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
            (run_dir / "exit_code").write_text("143\n")
        except (FileNotFoundError, ProcessLookupError, PermissionError, TypeError):
            pass

    def test_batch_launch_reports_common_declared_paths(self):
        self.write_ticket("01-alpha.md", ticket_text("01", "alpha", declared=["alpha.txt", "shared.txt"]))
        self.write_ticket("02-beta.md", ticket_text("02", "beta", declared=["beta.txt", "shared.txt"]))
        self.commit_tickets()

        out = self.call(["launch", "demo", "01", "02", "--integration", "effort/demo", "--parallel-cap", "2"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        launched = json.loads(out)
        for item in launched["launched"]:
            self.runs.append(Path(item["run_dir"]))
        findings = launched["overlap"]["findings"]
        self.assertEqual(
            [finding for finding in findings if finding["kind"] == "declared-path-overlap"],
            [{
                "severity": "NOTE",
                "kind": "declared-path-overlap",
                "tickets": ["01", "02"],
                "paths": ["shared.txt"],
                "detail": "tickets 01 and 02 both declare: shared.txt",
            }],
        )

    def test_overlap_command_reports_running_changed_path_conflicts_from_main_checkout(self):
        self.write_ticket("01-alpha.md", ticket_text("01", "alpha", declared=["alpha.txt"]))
        self.write_ticket("02-beta.md", ticket_text("02", "beta", declared=["beta.txt"]))
        self.commit_tickets()
        integration = self.move_integration_branch_to_worktree()
        records = self.move_effort_records_to_branch(integration)

        out = self.call(["launch", "demo", "01", "02", "--parallel-cap", "2"])
        self.assertEqual(self.code, lead_launch.OK, self.err)
        launched = json.loads(out)
        runs = []
        for item in launched["launched"]:
            run_dir = Path(item["run_dir"])
            self.runs.append(run_dir)
            runs.append(json.loads((run_dir / "run.json").read_text()))
        by_ticket = {run["lead_launch"]["ticket"]: run for run in runs}
        worker_01 = Path(by_ticket["01"]["worktree"])
        worker_02 = Path(by_ticket["02"]["worktree"])
        (worker_01 / "outside.txt").write_text("outside\n")
        (worker_01 / "beta.txt").write_text("other declared\n")
        (worker_01 / "shared-undecided.txt").write_text("one\n")
        (worker_02 / "shared-undecided.txt").write_text("two\n")
        (worker_01 / "docs/efforts/demo/evidence").mkdir(parents=True, exist_ok=True)
        (worker_01 / "docs/efforts/demo/evidence/01-report.md").write_text("own report\n")
        (integration / "integration-moved.txt").write_text("new tip\n")
        git(integration, "add", "integration-moved.txt")
        git(integration, "commit", "-qm", "move integration")

        report_text = self.call(["overlap", "demo"])

        self.assertEqual(self.code, lead_launch.OK, self.err)
        report = json.loads(report_text)
        self.assertEqual(report["integration"], "effort/demo")
        self.assertFalse((self.repo / "docs" / "efforts" / "demo").exists())
        self.assertFalse((integration / "docs" / "efforts" / "demo").exists())
        self.assertIn("Status: claimed", (records / "docs/efforts/demo/tickets/01-alpha.md").read_text())
        self.assertIn("Status: claimed", (records / "docs/efforts/demo/tickets/02-beta.md").read_text())
        self.assertTrue(all(entry["alive"] for entry in report["active"]))
        self.assertEqual({entry["ticket"]: entry["behind_integration"] for entry in report["active"]}, {"01": True, "02": True})
        self.assertEqual(report["common_changed_paths"], [{"tickets": ["01", "02"], "path": "shared-undecided.txt"}])
        findings = report["findings"]
        self.assertIn(
            {
                "severity": "VIOLATION",
                "kind": "changed-outside-declared",
                "ticket": "01",
                "path": "outside.txt",
                "detail": "ticket 01 changed outside its declared paths: outside.txt",
            },
            findings,
        )
        self.assertIn(
            {
                "severity": "VIOLATION",
                "kind": "changed-path-declared-by-other",
                "tickets": ["01", "02"],
                "path": "beta.txt",
                "detail": "ticket 01 changed beta.txt, declared by running ticket 02",
            },
            findings,
        )
        self.assertIn(
            {
                "severity": "VIOLATION",
                "kind": "common-undeclared-changed-path",
                "tickets": ["01", "02"],
                "path": "shared-undecided.txt",
                "detail": "running tickets 01 and 02 both changed undeclared path shared-undecided.txt",
            },
            findings,
        )
        self.assertNotIn("01-report.md", json.dumps(findings))


if __name__ == "__main__":
    unittest.main()

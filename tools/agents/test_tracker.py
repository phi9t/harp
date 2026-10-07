"""tracker.py against throwaway ticket repositories."""

from __future__ import annotations

import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tracker  # noqa: E402


def git(cwd: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


def ticket_text(
    number: str,
    slug: str,
    *,
    title: str = "title",
    type_: str = "task",
    status: str = "open",
    owner: str = "",
    blocked_by: str = "",
    claimed: str = "",
    triage: str | None = None,
    acceptance: str = "- [ ] criterion\n",
    for_later: str | None = None,
    comments: str = "",
) -> str:
    triage_text = "" if triage is None else f"Triage: {triage}\n\n"
    for_later_text = "" if for_later is None else f"## For later tickets\n{for_later}\n"
    return (
        f"# {number} {slug}: {title}\n\n"
        f"Type: {type_}\n"
        f"Status: {status}\n"
        f"Owner: {owner}\n"
        f"Blocked by: {blocked_by}\n"
        f"Claimed: {claimed}\n\n"
        f"{triage_text}"
        "## Acceptance\n"
        f"{acceptance}\n"
        f"{for_later_text}"
        "## Comments\n"
        f"{comments}"
    )


class TrackerTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name) / "repo"
        self.repo.mkdir()
        git(self.repo, "init", "-q", "-b", "main")
        (self.repo / "docs/efforts/demo/tickets").mkdir(parents=True)
        (self.repo / "README").write_text("demo\n")
        git(self.repo, "add", ".")
        git(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "init")

    def tearDown(self):
        self.tmp.cleanup()

    def ticket_path(self, name: str) -> Path:
        return self.repo / "docs/efforts/demo/tickets" / name

    def write_ticket(self, name: str, text: str) -> Path:
        path = self.ticket_path(name)
        path.write_text(text)
        return path

    def write_report(self, name: str, text: str) -> Path:
        path = self.repo / "docs/efforts/demo/evidence" / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def write_acceptance_record(self, ticket: str, candidate: str, *, accepted: bool = True) -> Path:
        path = self.repo / "docs/efforts/demo/evidence" / f"acceptance-{ticket}.jsonl"
        path.parent.mkdir(parents=True, exist_ok=True)
        record = {
            "schema": "corenius.acceptance.v1",
            "effort": "demo",
            "ticket": ticket,
            "candidate": candidate,
            "reviewer": "lead",
            "reviewer_context": "lead",
            "axes": {"standards": "passed", "spec": "passed"},
            "required_checks": [{"name": "unit", "result": "passed"}],
            "accepted": accepted,
        }
        with path.open("a") as handle:
            handle.write(json.dumps(record, sort_keys=True) + "\n")
        return path

    def write_landing_record(self, ticket: str, candidate: str, target: str = "effort/demo") -> Path:
        path = self.repo / "docs/efforts/demo/evidence" / f"landing-{ticket}.jsonl"
        path.parent.mkdir(parents=True, exist_ok=True)
        record = {
            "schema": "corenius.ticket_landing.v1",
            "effort": "demo",
            "ticket": ticket,
            "candidate": candidate,
            "target": target,
            "landed_at": "2026-09-30T01:30:00Z",
        }
        with path.open("a") as handle:
            handle.write(json.dumps(record, sort_keys=True) + "\n")
        return path

    def write_lead_handoffs(self, text: str) -> Path:
        return self.write_report("lead-handoffs.md", text)

    def integration_worktree(self) -> Path:
        integration = self.repo.parent / "integration"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/demo", str(integration), "main")
        return integration

    def move_records_to_branch(self) -> Path:
        effort = self.repo / "docs/efforts/demo"
        (effort / "EFFORT.md").write_text(
            "# Effort: demo\n\n"
            "Status: active\n"
            "Base: main\n"
            "Integration branch: effort/demo\n"
            "Record branch: effort/demo-records\n"
            "Lead: lead\n"
        )
        git(self.repo, "add", "docs/efforts/demo")
        git(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "effort records")
        records = self.repo.parent / "records"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/demo-records", str(records), "main")
        git(self.repo, "rm", "-qr", "docs/efforts/demo")
        git(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "remove records from code")
        return records

    def move_records_with_code_worktree(self) -> tuple[Path, Path]:
        effort = self.repo / "docs/efforts/demo"
        (effort / "EFFORT.md").write_text(
            "# Effort: demo\n\n"
            "Status: active\n"
            "Base: main\n"
            "Integration branch: effort/demo\n"
            "Record branch: effort/demo-records\n"
            "Lead: lead\n"
        )
        git(self.repo, "add", "docs/efforts/demo")
        git(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "effort records")
        code_tip = git(self.repo, "rev-parse", "HEAD").strip()
        records = self.repo.parent / "records"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/demo-records", str(records), code_tip)
        integration = self.repo.parent / "integration"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/demo", str(integration), code_tip)
        git(self.repo, "rm", "-qr", "docs/efforts/demo")
        git(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "remove records from main")
        git(integration, "rm", "-qr", "docs/efforts/demo")
        git(integration, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "remove records from integration")
        return records, integration

    def call(self, argv: list[str], now: str | None = None) -> str:
        return self.call_from(self.repo, argv, now=now)

    def call_from(self, repo: Path, argv: list[str], now: str | None = None) -> str:
        out, err = io.StringIO(), io.StringIO()
        env = {}
        if now:
            env["CORENIUS_TRACKER_NOW"] = now
        with patch.dict(os.environ, env, clear=False):
            with redirect_stdout(out), redirect_stderr(err):
                self.code = tracker.main(["--repo", str(repo)] + argv)
        self.err = err.getvalue()
        return out.getvalue()

    def test_new_creates_next_ticket_with_template_and_blockers(self):
        self.write_ticket("01-base.md", ticket_text("01", "base", acceptance="- [x] done\n"))
        out = self.call(["new", "demo", "next-step", "--title", "Next step", "--type", "feature", "--blocked-by", "01"])
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertIn("02-next-step.md", out)
        created = self.ticket_path("02-next-step.md").read_text()
        self.assertIn("# 02 next-step: Next step", created)
        self.assertIn("Type: feature", created)
        self.assertIn("Blocked by: 01", created)
        self.assertIn("Triage: needs-triage", created)
        self.assertIn("- [ ] _TODO_", created)
        self.call(["lint", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)

    def test_claim_refuses_active_claim_and_unresolved_blocker(self):
        self.write_ticket(
            "01-active.md",
            ticket_text(
                "01",
                "active",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
            ),
        )
        self.write_ticket("02-blocker.md", ticket_text("02", "blocker"))
        self.write_ticket("03-blocked.md", ticket_text("03", "blocked", blocked_by="02"))

        self.call(["claim", "demo", "01", "--owner", "bob"], now="2026-09-30T12:00:00Z")
        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("already claimed by alice", self.err)

        self.call(["claim", "demo", "03", "--owner", "bob"], now="2026-09-30T12:00:00Z")
        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("blocked", self.err)

    def test_claim_refuses_status_only_resolved_blocker_without_landed_candidate(self):
        self.write_ticket(
            "01-blocker.md",
            ticket_text("01", "blocker", status="resolved", owner="alice", acceptance="- [x] done\n"),
        )
        self.write_ticket(
            "02-blocked.md",
            ticket_text("02", "blocked", blocked_by="01", triage="ready-for-agent"),
        )

        self.call(["claim", "demo", "02", "--owner", "bob"], now="2026-09-30T12:00:00Z")
        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("blocked", self.err)

        out = self.call(["launchable", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual(out, "")

    def test_accepted_landing_record_unblocks_dependant(self):
        candidate = "1" * 40
        self.write_ticket(
            "01-blocker.md",
            ticket_text("01", "blocker", status="resolved", owner="alice", acceptance="- [x] done\n"),
        )
        self.write_ticket(
            "02-blocked.md",
            ticket_text("02", "blocked", blocked_by="01", triage="ready-for-agent"),
        )
        self.write_acceptance_record("01", candidate)
        self.write_landing_record("01", candidate)

        out = self.call(["launchable", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual([line.split()[0] for line in out.splitlines()], ["02"])

        out = self.call(["claim", "demo", "02", "--owner", "bob"], now="2026-09-30T12:00:00Z")
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertIn("docs/efforts/demo/tickets/02-blocked.md", out)

    def test_accepted_candidate_in_integration_history_unblocks_dependant_from_main_checkout(self):
        (self.repo / "docs/efforts/demo/EFFORT.md").write_text(
            "# Effort: demo\n\n"
            "Status: active\n"
            "Base: main\n"
            "Integration branch: effort/demo\n"
            "Record branch:\n"
            "Lead: lead\n"
        )
        (self.repo / "candidate.txt").write_text("candidate\n")
        git(self.repo, "add", ".")
        git(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "candidate")
        candidate = git(self.repo, "rev-parse", "HEAD").strip()
        git(self.repo, "branch", "effort/demo", candidate)
        self.write_ticket(
            "01-blocker.md",
            ticket_text("01", "blocker", status="resolved", owner="alice", acceptance="- [x] done\n"),
        )
        self.write_ticket(
            "02-blocked.md",
            ticket_text("02", "blocked", blocked_by="01", triage="ready-for-agent"),
        )
        self.write_acceptance_record("01", candidate)

        out = self.call(["launchable", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual([line.split()[0] for line in out.splitlines()], ["02"])

    def test_expired_claim_can_be_taken_over_with_injected_time(self):
        self.write_ticket(
            "01-expired.md",
            ticket_text(
                "01",
                "expired",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-09-30T01:00:00Z",
            ),
        )
        self.call(["claim", "demo", "01", "--owner", "bob", "--ttl-hours", "2"], now="2026-09-30T03:00:00Z")
        self.assertEqual(self.code, tracker.OK, self.err)
        text = self.ticket_path("01-expired.md").read_text()
        self.assertIn("Owner: bob", text)
        self.assertIn("Claimed: 2026-09-30T03:00:00Z expires 2026-09-30T05:00:00Z", text)
        self.assertIn("- 2026-09-30T03:00:00Z bob: claimed until 2026-09-30T05:00:00Z", text)

    def test_claim_and_resolve_use_record_branch_when_code_checkout_has_no_effort(self):
        candidate = "3" * 40
        self.write_ticket("01-open.md", ticket_text("01", "open", triage="ready-for-agent"))
        self.write_ticket(
            "02-claimed.md",
            ticket_text(
                "02",
                "claimed",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
                acceptance="- [x] done\n",
                triage="ready-for-agent",
            ),
        )
        self.write_acceptance_record("02", candidate)
        self.write_landing_record("02", candidate)
        records = self.move_records_to_branch()

        out = self.call(["claim", "demo", "01", "--owner", "bob"], now="2026-09-30T03:00:00Z")

        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertIn("docs/efforts/demo/tickets/01-open.md", out)
        self.assertIn("Owner: bob", (records / "docs/efforts/demo/tickets/01-open.md").read_text())
        self.assertFalse((self.repo / "docs/efforts/demo").exists())

        out = self.call(["resolve", "demo", "02", "--owner", "alice"], now="2026-09-30T04:00:00Z")

        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertIn("docs/efforts/demo/tickets/02-claimed.md", out)
        self.assertIn("Status: resolved", (records / "docs/efforts/demo/tickets/02-claimed.md").read_text())
        self.assertFalse((self.repo / "docs/efforts/demo").exists())

    def test_list_uses_record_branch_from_every_checkout(self):
        self.write_ticket("01-open.md", ticket_text("01", "open", triage="ready-for-agent"))
        self.write_ticket("02-other.md", ticket_text("02", "other", triage="ready-for-agent"))
        git(self.repo, "add", "docs/efforts/demo/tickets")
        git(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "tickets")
        records = self.move_records_to_branch()
        stale = self.repo.parent / "stale-copy"
        git(self.repo, "worktree", "add", "-q", "-b", "agent/stale-copy", str(stale), "effort/demo-records")
        self.call_from(records, ["claim", "demo", "01", "--owner", "worker-01"], now="2026-09-30T03:00:00Z")
        self.assertEqual(self.code, tracker.OK, self.err)

        rows = []
        for checkout in (self.repo, records, stale):
            out = self.call_from(checkout, ["list", "demo", "--json"])
            self.assertEqual(self.code, tracker.OK, (checkout, self.err))
            rows.append(json.loads(out))

        self.assertEqual(rows[0], rows[1])
        self.assertEqual(rows[1], rows[2])
        by_id = {row["id"]: row for row in rows[0]}
        self.assertEqual(by_id["01"]["status"], "claimed")

    def test_malformed_owner_waiver_cannot_unlock_landed_blocker(self):
        candidate = "2" * 40
        self.write_ticket("01-done.md", ticket_text("01", "done", status="resolved"))
        self.write_ticket("02-next.md", ticket_text("02", "next", blocked_by="01", triage="ready-for-agent"))
        path = self.write_acceptance_record("01", candidate)
        record = json.loads(path.read_text())
        record["required_checks"][0]["result"] = "failed"
        self.write_landing_record("01", candidate)
        for waiver in ("approved", True, {"owner": "alice"}, {"reason": "approved"}):
            with self.subTest(waiver=waiver):
                record["owner_waiver"] = waiver
                path.write_text(json.dumps(record) + "\n")
                out = self.call(["launchable", "demo", "--json"])
                self.assertEqual(self.code, tracker.OK, self.err)
                self.assertEqual(json.loads(out), [])
        record["owner_waiver"] = {"owner": "alice", "reason": "known fixture failure"}
        path.write_text(json.dumps(record) + "\n")
        out = self.call(["launchable", "demo", "--json"])
        self.assertEqual([row["id"] for row in json.loads(out)], ["02"])

    def test_resolve_by_owner_requires_checked_acceptance(self):
        candidate = "2" * 40
        self.write_ticket(
            "01-work.md",
            ticket_text(
                "01",
                "work",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
                acceptance="- [x] done\n",
            ),
        )
        self.write_acceptance_record("01", candidate)
        self.write_landing_record("01", candidate)
        self.call(["resolve", "demo", "01", "--owner", "bob"], now="2026-09-30T02:00:00Z")
        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("claimed by alice", self.err)

        self.call(["resolve", "demo", "01", "--owner", "alice"], now="2026-09-30T02:00:00Z")
        self.assertEqual(self.code, tracker.OK, self.err)
        text = self.ticket_path("01-work.md").read_text()
        self.assertIn("Status: resolved", text)
        self.assertIn("Owner: alice", text)
        self.assertIn("Claimed:\n", text)
        self.assertIn("- 2026-09-30T02:00:00Z alice: resolved", text)

    def test_resolve_refuses_checked_acceptance_without_accepted_landed_candidate_and_force_logs_override(self):
        self.write_ticket(
            "01-work.md",
            ticket_text(
                "01",
                "work",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
                acceptance="- [x] done\n",
            ),
        )
        self.call(["resolve", "demo", "01", "--owner", "alice"], now="2026-09-30T02:00:00Z")
        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("no accepted, landed Candidate", self.err)

        self.call(["resolve", "demo", "01", "--owner", "alice", "--force"], now="2026-09-30T02:00:00Z")
        self.assertEqual(self.code, tracker.OK, self.err)
        text = self.ticket_path("01-work.md").read_text()
        self.assertIn("Status: resolved", text)
        self.assertIn("resolved with --force despite missing accepted, landed Candidate", text)

    def test_resolve_refuses_unchecked_acceptance_and_force_logs_override(self):
        self.write_ticket(
            "01-work.md",
            ticket_text(
                "01",
                "work",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
            ),
        )
        self.call(["resolve", "demo", "01", "--owner", "alice"], now="2026-09-30T02:00:00Z")
        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("unchecked acceptance", self.err)

        self.call(["resolve", "demo", "01", "--owner", "alice", "--force"], now="2026-09-30T02:00:00Z")
        self.assertEqual(self.code, tracker.OK, self.err)
        text = self.ticket_path("01-work.md").read_text()
        self.assertIn("Status: resolved", text)
        self.assertIn("resolved with --force despite unchecked acceptance", text)

    def test_lint_reports_contract_errors(self):
        self.write_ticket(
            "01-header.md",
            "# 01 header: title\n\nStatus: open\nType: task\nOwner:\nBlocked by:\nClaimed:\n\n## Acceptance\n- [ ] ok\n\n## Comments\n",
        )
        self.write_ticket("02-values.md", ticket_text("02", "values", type_="research", status="waiting"))
        self.write_ticket("03-claimed.md", ticket_text("03", "claimed", status="claimed"))
        self.write_ticket("04-blockers.md", ticket_text("04", "blockers", blocked_by="99, 04"))
        self.write_ticket("05-cycle-a.md", ticket_text("05", "cycle-a", blocked_by="06"))
        self.write_ticket("06-cycle-b.md", ticket_text("06", "cycle-b", blocked_by="05"))
        self.write_ticket("07-empty.md", ticket_text("07", "empty", acceptance=""))
        self.write_ticket("08-comment.md", ticket_text("08", "comment", comments="not a comment\n"))
        self.write_ticket("09-one.md", ticket_text("09", "one"))
        self.write_ticket("09-two.md", ticket_text("09", "two"))
        self.write_ticket(
            "10-no-owner.md",
            ticket_text(
                "10",
                "no-owner",
                status="claimed",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
            ),
        )
        self.write_ticket(
            "11-no-claimed.md",
            ticket_text("11", "no-claimed", status="claimed", owner="alice"),
        )
        self.write_ticket(
            "12-bad-claimed.md",
            ticket_text(
                "12",
                "bad-claimed",
                status="claimed",
                owner="alice",
                claimed="2026-99-30T00:00:00Z expires 2026-10-01T00:00:00Z",
            ),
        )

        out = self.call(["lint", "demo"])
        self.assertEqual(self.code, tracker.REFUSED)
        for message in (
            "missing or reordered header line",
            "bad Type",
            "bad Status",
            "Status claimed requires Owner and Claimed",
            "unknown blocker 99",
            "self blocker 04",
            "cyclic blocker",
            "empty Acceptance",
            "malformed comment line",
            "duplicate id 09",
            "malformed Claimed line",
        ):
            self.assertIn(message, out)

    def test_list_prints_frontier_ordering_and_json(self):
        self.write_ticket("01-open.md", ticket_text("01", "open", title="Open"))
        self.write_ticket(
            "02-claimed.md",
            ticket_text(
                "02",
                "claimed",
                title="Claimed",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
            ),
        )
        self.write_ticket("03-blocked.md", ticket_text("03", "blocked", title="Blocked", blocked_by="02"))
        self.write_ticket(
            "04-resolved.md",
            ticket_text("04", "resolved", title="Resolved", status="resolved", owner="alice", acceptance="- [x] done\n"),
        )

        out = self.call(["list", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)
        ids = [line.split()[0] for line in out.splitlines()]
        self.assertEqual(ids, ["01", "02", "03", "04"])
        self.assertIn("01 open", out)
        self.assertIn("02 claimed", out)
        self.assertIn("03 blocked", out)
        self.assertIn("04 resolved", out)

        data = json.loads(self.call(["list", "demo", "--json"]))
        self.assertEqual([row["id"] for row in data], ["01", "02", "03", "04"])
        self.assertEqual([row["category"] for row in data], ["open", "claimed", "blocked", "resolved"])

    def test_launchable_prints_open_unblocked_ready_for_agent_tickets(self):
        self.write_ticket(
            "01-done.md",
            ticket_text("01", "done", status="resolved", owner="alice", triage="ready-for-agent", acceptance="- [x] done\n"),
        )
        self.write_ticket("02-ready.md", ticket_text("02", "ready", title="Ready", triage="ready-for-agent"))
        self.write_ticket("03-blocked.md", ticket_text("03", "blocked", blocked_by="02", triage="ready-for-agent"))
        self.write_ticket("04-human.md", ticket_text("04", "human", triage="ready-for-human"))
        self.write_ticket("05-missing.md", ticket_text("05", "missing"))
        self.write_ticket(
            "06-claimed.md",
            ticket_text(
                "06",
                "claimed",
                status="claimed",
                owner="alice",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
                triage="ready-for-agent",
            ),
        )

        out = self.call(["launchable", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual([line.split()[0] for line in out.splitlines()], ["02"])
        self.assertIn("02 ready-for-agent ready: Ready", out)

        data = json.loads(self.call(["launchable", "demo", "--json"]))
        self.assertEqual([row["id"] for row in data], ["02"])
        self.assertEqual(data[0]["triage"], "ready-for-agent")
        self.assertEqual(data[0]["blocked_by"], [])

    def test_lint_validates_triage_when_present_but_allows_legacy_missing_triage(self):
        self.write_ticket("01-missing.md", ticket_text("01", "missing"))
        self.write_ticket("02-bad.md", ticket_text("02", "bad", triage="delegate-to-moon"))

        out = self.call(["lint", "demo"])
        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("bad Triage 'delegate-to-moon'", out)
        self.assertNotIn("01-missing", out)

    def test_lint_validates_for_later_ticket_findings(self):
        self.write_ticket("01-source.md", ticket_text("01", "source", for_later="- to 02: carry this forward\n"))
        self.write_ticket("02-target.md", ticket_text("02", "target"))
        self.write_ticket("03-malformed.md", ticket_text("03", "malformed", for_later="- for 02: bad prefix\n"))
        self.write_ticket("04-unknown.md", ticket_text("04", "unknown", for_later="- to 99: missing target\n"))
        self.write_ticket("05-self.md", ticket_text("05", "self", for_later="- to 05: self target\n"))

        out = self.call(["lint", "demo"])

        self.assertEqual(self.code, tracker.REFUSED)
        self.assertNotIn("01-source", out)
        self.assertIn("malformed later-ticket finding", out)
        self.assertIn("unknown later-ticket target 99", out)
        self.assertIn("self later-ticket target 05", out)

    def test_findings_command_appends_and_lists_text_and_json(self):
        self.write_ticket("01-source.md", ticket_text("01", "source"))
        self.write_ticket("02-target.md", ticket_text("02", "target"))

        out = self.call(["add-finding", "demo", "01", "--to", "02", "--text", "Use the parser seam." ])

        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertIn("01-source.md", out)
        text = self.ticket_path("01-source.md").read_text()
        self.assertIn("## For later tickets\n- to 02: Use the parser seam.\n\n## Comments", text)
        self.call(["lint", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)

        out = self.call(["findings", "demo", "02"])
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual(out.strip(), "ticket 01-source: Use the parser seam.")

        data = json.loads(self.call(["findings", "demo", "02", "--json"]))
        self.assertEqual(data[0]["target"], "02")
        self.assertEqual(data[0]["text"], "Use the parser seam.")
        self.assertEqual(
            data[0]["sources"],
            [
                {
                    "kind": "ticket",
                    "id": "01",
                    "slug": "source",
                    "label": "ticket 01-source",
                    "path": "docs/efforts/demo/tickets/01-source.md",
                    "line": 13,
                }
            ],
        )

    def test_findings_command_reads_reports_lead_handoffs_and_deduplicates_sources(self):
        self.write_ticket("01-source.md", ticket_text("01", "source", for_later="- to 04: Ticket insight.\n"))
        self.write_ticket("02-other.md", ticket_text("02", "other"))
        self.write_ticket("03-report-source.md", ticket_text("03", "report-source"))
        self.write_ticket("04-target.md", ticket_text("04", "target"))
        self.write_report(
            "03-report.md",
            "# Ticket 03 report\n\n"
            "## For later tickets\n"
            "- to 04: Shared duplicate.\n"
            "- to 04: Report insight.\n",
        )
        self.write_lead_handoffs(
            "# Lead handoffs\n\n"
            "- to 04: Shared duplicate.\n"
            "- to 04: Lead insight.\n"
        )

        out = self.call(["findings", "demo", "04"])
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual(
            out.splitlines(),
            [
                "ticket 01-source: Ticket insight.",
                "report 03, lead: Shared duplicate.",
                "report 03: Report insight.",
                "lead: Lead insight.",
            ],
        )

        data = json.loads(self.call(["findings", "demo", "04", "--json"]))
        self.assertEqual([row["text"] for row in data], ["Ticket insight.", "Shared duplicate.", "Report insight.", "Lead insight."])
        self.assertEqual([source["label"] for source in data[1]["sources"]], ["report 03", "lead"])

    def test_findings_on_moved_effort_reads_accepted_report_copy_with_other_sources(self):
        self.write_ticket("01-source.md", ticket_text("01", "source", for_later="- to 04: Ticket insight.\n"))
        self.write_ticket("02-other.md", ticket_text("02", "other"))
        self.write_ticket("03-report-source.md", ticket_text("03", "report-source"))
        self.write_ticket("04-target.md", ticket_text("04", "target"))
        records, _ = self.move_records_with_code_worktree()
        evidence = records / "docs" / "efforts" / "demo" / "evidence"
        evidence.mkdir(parents=True, exist_ok=True)
        (evidence / "03-report-round-1.md").write_text(
            "# Ticket 03 report\n\n"
            "## For later tickets\n"
            "- to 04: Shared duplicate.\n"
            "- to 04: Report insight.\n"
        )
        (evidence / "lead-handoffs.md").write_text(
            "# Lead handoffs\n\n"
            "- to 04: Shared duplicate.\n"
            "- to 04: Lead insight.\n"
        )

        out = self.call(["findings", "demo", "04"])

        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual(
            out.splitlines(),
            [
                "ticket 01-source: Ticket insight.",
                "report 03 round 1, lead: Shared duplicate.",
                "report 03 round 1: Report insight.",
                "lead: Lead insight.",
            ],
        )
        data = json.loads(self.call(["findings", "demo", "04", "--json"]))
        self.assertEqual([row["text"] for row in data], ["Ticket insight.", "Shared duplicate.", "Report insight.", "Lead insight."])
        self.assertEqual([source["label"] for source in data[1]["sources"]], ["report 03 round 1", "lead"])

    def test_findings_on_moved_effort_reads_unaccepted_code_branch_report_as_fallback(self):
        self.write_ticket("01-source.md", ticket_text("01", "source"))
        self.write_ticket("02-target.md", ticket_text("02", "target"))
        _, integration = self.move_records_with_code_worktree()
        report = integration / "docs" / "efforts" / "demo" / "evidence" / "01-report.md"
        report.parent.mkdir(parents=True)
        report.write_text(
            "# Ticket 01 report\n\n"
            "## For later tickets\n"
            "- to 02: Report waiting for acceptance.\n"
        )
        git(integration, "add", "docs/efforts/demo/evidence/01-report.md")
        git(integration, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "worker report landed")

        out = self.call(["findings", "demo", "02"])

        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertEqual(out.strip(), "report 01 (not yet accepted): Report waiting for acceptance.")
        data = json.loads(self.call(["findings", "demo", "02", "--json"]))
        self.assertEqual(data[0]["sources"][0]["label"], "report 01 (not yet accepted)")

    def test_lint_validates_report_and_lead_handoff_findings(self):
        self.write_ticket("01-source.md", ticket_text("01", "source"))
        self.write_ticket("02-target.md", ticket_text("02", "target"))
        self.write_ticket("03-report-source.md", ticket_text("03", "report-source"))
        self.write_report("01-report.md", "# Report\n\n## For later tickets\nnone\n")
        self.write_report("03-report.md", "# Report\n\n## For later tickets\n- for 02: bad prefix\n- to 99: missing\n")
        self.write_lead_handoffs("# Lead handoffs\n\n- for 02: bad prefix\n- to 99: missing\n")

        out = self.call(["lint", "demo"])

        self.assertEqual(self.code, tracker.REFUSED)
        self.assertIn("03-report.md:4: malformed later-ticket finding", out)
        self.assertIn("03-report.md:5: unknown later-ticket target 99", out)
        self.assertIn("lead-handoffs.md:3: malformed later-ticket finding", out)
        self.assertIn("lead-handoffs.md:4: unknown later-ticket target 99", out)

    def test_claim_and_resolve_preserve_for_later_sections(self):
        candidate = "4" * 40
        self.write_ticket(
            "01-work.md",
            ticket_text(
                "01",
                "work",
                acceptance="- [x] done\n",
                for_later="- to 02: keep this for later\n",
            ),
        )
        self.write_ticket("02-target.md", ticket_text("02", "target"))
        self.write_acceptance_record("01", candidate)
        self.write_landing_record("01", candidate)

        self.call(["claim", "demo", "01", "--owner", "alice"], now="2026-09-30T01:00:00Z")
        self.assertEqual(self.code, tracker.OK, self.err)
        text = self.ticket_path("01-work.md").read_text()
        self.assertIn("## For later tickets\n- to 02: keep this for later\n\n## Comments", text)
        self.assertIn("- 2026-09-30T01:00:00Z alice: claimed until", text)

        self.call(["resolve", "demo", "01", "--owner", "alice"], now="2026-09-30T02:00:00Z")

        self.assertEqual(self.code, tracker.OK, self.err)
        text = self.ticket_path("01-work.md").read_text()
        self.assertIn("## For later tickets\n- to 02: keep this for later\n\n## Comments", text)
        self.assertIn("- 2026-09-30T02:00:00Z alice: resolved", text)
        self.write_ticket("03-no-findings.md", ticket_text("03", "no-findings"))
        self.call(["lint", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)

    def test_question_moves_triage_to_needs_info_and_appends_question(self):
        self.write_ticket(
            "01-work.md",
            ticket_text(
                "01",
                "work",
                status="claimed",
                owner="worker-01",
                claimed="2026-09-30T00:00:00Z expires 2026-10-01T00:00:00Z",
                triage="ready-for-agent",
            ),
        )

        out = self.call(
            ["question", "demo", "01", "--owner", "worker-01", "--question", "Need the API token scope."],
            now="2026-09-30T02:00:00Z",
        )
        self.assertEqual(self.code, tracker.OK, self.err)
        self.assertIn("01-work.md", out)
        text = self.ticket_path("01-work.md").read_text()
        self.assertIn("Triage: needs-info", text)
        self.assertIn("- 2026-09-30T02:00:00Z worker-01: question: Need the API token scope.", text)
        self.call(["lint", "demo"])
        self.assertEqual(self.code, tracker.OK, self.err)

    def test_real_agentic_engineering_tickets_lint_after_copy(self):
        source = Path(__file__).resolve().parents[2] / "docs/efforts/agentic-engineering/tickets"
        target = self.repo / "docs/efforts/agentic-engineering/tickets"
        target.parent.mkdir(parents=True)
        shutil.copytree(source, target)
        out = self.call(["lint", "agentic-engineering"])
        self.assertEqual(self.code, tracker.OK, out + self.err)

    def test_repo_option_is_accepted_after_subcommand(self):
        self.write_ticket("01-open.md", ticket_text("01", "open"))
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            code = tracker.main(["lint", "demo", "--repo", str(self.repo)])
        self.assertEqual(code, tracker.OK, out.getvalue() + err.getvalue())

    def test_lint_without_args_ignores_legacy_effort_only_in_other_worktree(self):
        self.write_ticket("01-open.md", ticket_text("01", "open"))
        other = self.repo.parent / "other"
        git(self.repo, "worktree", "add", "-q", "-b", "work/other", str(other), "main")
        legacy = other / "docs/efforts/legacy"
        (legacy / "tickets").mkdir(parents=True)
        (legacy / "EFFORT.md").write_text(
            "# Effort: legacy\n\n"
            "Status: active\n"
            "Base: main\n"
            "Lead: lead\n"
        )
        (legacy / "tickets/01-legacy.md").write_text(ticket_text("01", "legacy"))
        git(other, "add", "docs/efforts/legacy")
        git(other, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "legacy")

        out = self.call(["lint"])

        self.assertEqual(self.code, tracker.OK, out + self.err)

    def test_lint_without_args_ignores_integration_effort_only_in_other_worktree(self):
        self.write_ticket("01-open.md", ticket_text("01", "open"))
        other = self.repo.parent / "other"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/unrelated", str(other), "main")
        unrelated = other / "docs/efforts/unrelated"
        (unrelated / "tickets").mkdir(parents=True)
        (unrelated / "EFFORT.md").write_text(
            "# Effort: unrelated\n\n"
            "Status: active\n"
            "Base: main\n"
            "Integration branch: effort/unrelated\n"
            "Lead: lead\n"
        )
        (unrelated / "tickets/01-unrelated.md").write_text("# malformed\n")
        git(other, "add", "docs/efforts/unrelated")
        git(other, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "unrelated")

        out = self.call(["lint"])

        self.assertEqual(self.code, tracker.OK, out + self.err)


if __name__ == "__main__":
    unittest.main()

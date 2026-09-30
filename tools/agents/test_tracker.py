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
    acceptance: str = "- [ ] criterion\n",
    comments: str = "",
) -> str:
    return (
        f"# {number} {slug}: {title}\n\n"
        f"Type: {type_}\n"
        f"Status: {status}\n"
        f"Owner: {owner}\n"
        f"Blocked by: {blocked_by}\n"
        f"Claimed: {claimed}\n\n"
        "## Acceptance\n"
        f"{acceptance}\n"
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

    def call(self, argv: list[str], now: str | None = None) -> str:
        out, err = io.StringIO(), io.StringIO()
        env = {}
        if now:
            env["CORENIUS_TRACKER_NOW"] = now
        with patch.dict(os.environ, env, clear=False):
            with redirect_stdout(out), redirect_stderr(err):
                self.code = tracker.main(["--repo", str(self.repo)] + argv)
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

    def test_resolve_by_owner_requires_checked_acceptance(self):
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


if __name__ == "__main__":
    unittest.main()

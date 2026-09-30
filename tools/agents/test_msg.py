"""msg.py messages and handoffs against throwaway git repos."""

from __future__ import annotations

import io
import json
import os
import shlex
import stat
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import msg  # noqa: E402


def run(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


class MsgTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.repo = root / "proj"
        self.repo.mkdir()
        run(self.repo, "init", "-q", "-b", "main")
        run(self.repo, "config", "user.email", "a@b")
        run(self.repo, "config", "user.name", "a")
        (self.repo / "README").write_text("hi\n")
        run(self.repo, "add", "README")
        run(self.repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "init")

    def tearDown(self):
        self.tmp.cleanup()

    def call(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            self.code = msg.main(argv)
        self.err = err.getvalue()
        return out.getvalue()

    def send(self, *extra):
        out = self.call(
            [
                "send",
                "--root",
                str(self.repo),
                "--effort",
                "demo",
                "--from",
                "lead",
                "--to",
                "worker",
                "--type",
                "info",
                "--body",
                "hello",
                *extra,
            ]
        )
        self.assertEqual(self.code, 0, self.err)
        return Path(out.strip())

    def inbox(self):
        out = self.call(["inbox", "--root", str(self.repo), "--effort", "demo", "--to", "worker", "--json"])
        self.assertEqual(self.code, 0, self.err)
        return json.loads(out)

    def write_message(self, name, lines):
        directory = self.repo / ".scratch" / "demo" / "messages" / "worker"
        directory.mkdir(parents=True, exist_ok=True)
        path = directory / name
        path.write_text("\n".join(lines) + "\n\nbody\n")
        return path

    def test_inbox_orders_oldest_first_with_same_second_sends(self):
        first = self.send("--timestamp", "2026-01-01T00:00:00Z", "--body", "first")
        second = self.send("--timestamp", "2026-01-01T00:00:00Z", "--body", "second")
        self.assertNotEqual(first.name, second.name)
        items = self.inbox()
        self.assertEqual([item["file"] for item in items], [first.name, second.name])

    def test_ack_removes_message_without_editing_it(self):
        path = self.send("--timestamp", "2026-01-01T00:00:00Z")
        before = path.read_bytes()
        self.call(
            [
                "ack",
                "--root",
                str(self.repo),
                "--effort",
                "demo",
                "--to",
                "worker",
                path.name,
                "--by",
                "lead",
                "--timestamp",
                "2026-01-01T00:01:00Z",
            ]
        )
        self.assertEqual(self.code, 0, self.err)
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(self.inbox(), [])

    def test_needs_ack_stays_reported_flagged_until_ack(self):
        path = self.send("--timestamp", "2026-01-01T00:00:00Z", "--needs-ack")
        for _ in range(2):
            items = self.inbox()
            self.assertEqual(len(items), 1)
            self.assertTrue(items[0]["needs_ack"])
            self.assertEqual(items[0]["flags"], ["NEEDS-ACK"])
        self.call(["ack", "--root", str(self.repo), "--effort", "demo", "--to", "worker", path.name, "--by", "worker"])
        self.assertEqual(self.code, 0, self.err)
        self.assertEqual(self.inbox(), [])

    def test_lint_reports_each_header_problem(self):
        valid_tail = ["Type: info", "Re:", "Needs-ack: no"]
        self.write_message("missing-header.md", ["From: lead", *valid_tail])
        self.write_message("wrong-order.md", ["To: worker", "From: lead", *valid_tail])
        self.write_message("bad-type.md", ["From: lead", "To: worker", "Type: note", "Re:", "Needs-ack: no"])
        self.write_message("bad-needs-ack.md", ["From: lead", "To: worker", "Type: info", "Re:", "Needs-ack: maybe"])
        self.write_message(
            "unknown-extra.md",
            ["From: lead", "To: worker", "Type: info", "Re:", "Needs-ack: no", "Subject: extra"],
        )
        out = self.call(["lint", "--root", str(self.repo), "--effort", "demo"])
        self.assertEqual(self.code, 1)
        self.assertIn("missing-header.md: missing header To", out)
        self.assertIn("wrong-order.md: wrong header order", out)
        self.assertIn("bad-type.md: bad Type", out)
        self.assertIn("bad-needs-ack.md: bad Needs-ack", out)
        self.assertIn("unknown-extra.md: unknown extra header Subject", out)

    def test_lint_accepts_sent_messages(self):
        self.send("--timestamp", "2026-01-01T00:00:00Z", "--needs-ack")
        out = self.call(["lint", "--root", str(self.repo), "--effort", "demo"])
        self.assertEqual(self.code, 0, out)
        self.assertEqual(out, "")

    def test_send_refuses_effort_and_to_path_traversal(self):
        cases = [
            ("bad/name", "worker"),
            ("bad..name", "worker"),
            ("demo", "worker/name"),
            ("demo", "worker..name"),
        ]
        for effort, recipient in cases:
            with self.subTest(effort=effort, recipient=recipient):
                self.call(
                    [
                        "send",
                        "--root",
                        str(self.repo),
                        "--effort",
                        effort,
                        "--from",
                        "lead",
                        "--to",
                        recipient,
                        "--type",
                        "info",
                        "--body",
                        "hello",
                    ]
                )
                self.assertEqual(self.code, 64)
                self.assertIn("single path component", self.err)
        self.assertFalse((self.repo / ".scratch" / "bad").exists())

    def test_check_script_propagates_suite_failure(self):
        fake_python = Path(self.tmp.name) / "python3"
        marker = Path(self.tmp.name) / "called"
        fake_python.write_text(f"#!/bin/sh\nprintf '%s\\n' \"$*\" > {shlex.quote(str(marker))}\nexit 7\n")
        fake_python.chmod(fake_python.stat().st_mode | stat.S_IXUSR)
        env = os.environ.copy()
        env["PYTHON"] = str(fake_python)
        result = subprocess.run(
            ["sh", "tools/agents/checks.d/40-msg.sh"],
            cwd=Path(__file__).resolve().parents[2],
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 7)
        self.assertEqual(marker.read_text().strip(), "tools/agents/test_msg.py")


if __name__ == "__main__":
    unittest.main()

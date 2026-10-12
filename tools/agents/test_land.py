"""land.py against throwaway git repositories."""

from __future__ import annotations

import argparse
import io
import json
import os
import shlex
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest import mock

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import acceptance  # noqa: E402
import land  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


SCRIPT = Path(__file__).resolve().with_name("land.py")


def run(cwd, *args, check=True):
    result = subprocess.run(["git", "-C", str(cwd), *args], capture_output=True, text=True)
    if check and result.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed in {cwd}:\n{result.stderr}")
    return result.stdout


def q(path: Path) -> str:
    return shlex.quote(str(path))


class LandTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def repo(self, with_remote=False):
        repo = self.root / f"repo-{len(list(self.root.glob('repo-*')))}"
        repo.mkdir()
        run(repo, "init", "-q", "-b", "main")
        run(repo, "config", "user.email", "a@b")
        run(repo, "config", "user.name", "a")
        run(repo, "config", "--add", "collab.overrideOwners", "owner")
        (repo / "README").write_text("base\n")
        run(repo, "add", "README")
        run(repo, "commit", "-qm", "base")
        if not with_remote:
            return repo, None
        remote = self.root / f"remote-{len(list(self.root.glob('remote-*')))}.git"
        run(self.root, "init", "-q", "--bare", "-b", "main", str(remote))
        run(repo, "remote", "add", "origin", str(remote))
        run(repo, "push", "-q", "-u", "origin", "main")
        return repo, remote

    def topic_worktree(self, repo: Path, branch="topic", base="main", path="topic.txt", text="topic\n"):
        safe_branch = branch.replace("/", "-")
        worktree = self.root / f"{safe_branch}-{len(list(self.root.glob(f'{safe_branch}-*')))}"
        run(repo, "worktree", "add", "-q", "-b", branch, str(worktree), base)
        (worktree / path).write_text(text)
        run(worktree, "add", path)
        run(worktree, "commit", "-qm", "topic")
        return worktree, run(repo, "rev-parse", branch).strip()

    def topic(self, repo: Path, branch="topic", base="main", path="topic.txt", text="topic\n"):
        worktree, tip = self.topic_worktree(repo, branch=branch, base=base, path=path, text=text)
        run(repo, "worktree", "remove", "--force", str(worktree))
        return tip

    def effort_ticket(self, repo: Path, ticket="01", integration_branch="effort/sample"):
        effort = repo / "docs" / "efforts" / "sample"
        (effort / "tickets").mkdir(parents=True)
        (effort / "evidence").mkdir()
        (effort / "EFFORT.md").write_text(
            "# Effort: sample\n\n"
            "Status: active\n"
            "Base: main\n"
            f"Integration branch: {integration_branch}\n"
            "Lead: lead\n"
        )
        (effort / "tickets" / f"{ticket}-one.md").write_text(
            f"# {ticket} one: One\n\n"
            "Type: feature\n"
            "Status: claimed\n"
            "Owner: worker-01\n"
            "Blocked by:\n"
            "Claimed: 2026-10-05T10:00:00Z expires 2026-10-06T10:00:00Z\n\n"
            "## Acceptance\n"
            "- [ ] works\n\n"
            "## Comments\n"
        )
        run(repo, "add", ".")
        run(repo, "commit", "-qm", "ticket")
        if integration_branch != "main":
            run(repo, "branch", integration_branch, "main")

    def effort_ticket_worktree(self, repo: Path, ticket="01", integration_branch="effort/sample") -> Path:
        checkout = self.root / f"{integration_branch.replace('/', '-')}-wt"
        run(repo, "worktree", "add", "-q", "-b", integration_branch, str(checkout), "main")
        effort = checkout / "docs" / "efforts" / "sample"
        (effort / "tickets").mkdir(parents=True)
        (effort / "evidence").mkdir()
        (effort / "EFFORT.md").write_text(
            "# Effort: sample\n\n"
            "Status: active\n"
            "Base: main\n"
            f"Integration branch: {integration_branch}\n"
            "Lead: lead\n"
        )
        (effort / "tickets" / f"{ticket}-one.md").write_text(
            f"# {ticket} one: One\n\n"
            "Type: feature\n"
            "Status: claimed\n"
            "Owner: worker-01\n"
            "Blocked by:\n"
            "Claimed: 2026-10-05T10:00:00Z expires 2026-10-06T10:00:00Z\n\n"
            "## Acceptance\n"
            "- [ ] works\n\n"
            "## Comments\n"
        )
        run(checkout, "add", ".")
        run(checkout, "commit", "-qm", "ticket")
        return checkout

    def write_acceptance(self, repo: Path, candidate: str, **overrides):
        args = [
            "--repo", str(repo),
            "record",
            "sample",
            overrides.pop("ticket", "01"),
            "--candidate", overrides.pop("record_candidate", candidate),
            "--reviewer", overrides.pop("reviewer", "lead"),
            "--reviewer-context", overrides.pop("reviewer_context", "lead"),
            "--standards", overrides.pop("standards", "passed"),
            "--spec", overrides.pop("spec", "passed"),
            "--required-check", "unit",
            "--check", f"unit={overrides.pop('check', 'passed')}",
        ]
        if overrides.pop("waiver", False):
            args += ["--waiver-owner", "owner", "--waiver-reason", "accepted risk"]
        self.assertFalse(overrides, overrides)
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            code = acceptance.main(args)
        self.assertEqual(code, acceptance.OK)

    def write_acceptance_json(self, repo: Path, record: dict, ticket="01"):
        path = repo / "docs" / "efforts" / "sample" / "evidence" / f"acceptance-{ticket}.jsonl"
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a") as handle:
            handle.write(json.dumps(record, sort_keys=True) + "\n")

    def write_raw_acceptance(self, repo: Path, candidate: str, ticket="01"):
        self.write_acceptance_json(
            repo,
            {
                "schema": "corenius.acceptance.v1",
                "effort": "sample",
                "ticket": ticket,
                "candidate": candidate,
                "reviewer": "lead",
                "reviewer_context": "lead",
                "axes": {"standards": "passed", "spec": "passed"},
                "required_checks": [{"name": "unit", "result": "passed"}],
                "owner_waiver": None,
                "accepted": True,
            },
            ticket=ticket,
        )

    def record_worktree(self, repo: Path) -> Path:
        records = self.root / f"records-{len(list(self.root.glob('records-*')))}"
        run(repo, "worktree", "add", "-q", "-b", "effort/sample-records", str(records), "effort/sample")
        effort_file = records / "docs/efforts/sample/EFFORT.md"
        effort_file.write_text(effort_file.read_text() + "Record branch: effort/sample-records\n")
        run(records, "add", "docs/efforts/sample/EFFORT.md")
        run(records, "commit", "-qm", "move records")
        return records

    def commit_acceptance(self, repo: Path, ticket="01"):
        path = f"docs/efforts/sample/evidence/acceptance-{ticket}.jsonl"
        run(repo, "add", path)
        run(repo, "commit", "-qm", f"acceptance {ticket}")

    def call(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            code = land.main(argv)
        return code, out.getvalue(), err.getvalue()

    def rev(self, repo: Path, ref: str) -> str:
        return run(repo, "rev-parse", ref).strip()

    def test_exact_candidate_commit_lands_without_moving_worker_branch_and_gate_sees_it(self):
        repo, _ = self.repo()
        worker, candidate = self.topic_worktree(repo)
        (worker / "later.txt").write_text("later\n")
        run(worker, "add", "later.txt")
        run(worker, "commit", "-qm", "later")
        worker_tip = self.rev(repo, "topic")
        worker_status = run(worker, "status", "--porcelain", "--untracked-files=all")
        seen_tip = self.root / "seen-tip"

        code, out, err = self.call([
            candidate,
            "--repo", str(repo),
            "--owner-mainline-override", "owner",
            "--gate", f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {q(seen_tip)}',
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), candidate)
        self.assertEqual(seen_tip.read_text(), candidate)
        self.assertEqual(self.rev(repo, "main"), candidate)
        self.assertEqual(self.rev(repo, "topic"), worker_tip)
        self.assertEqual(run(worker, "status", "--porcelain", "--untracked-files=all"), worker_status)
        self.assertEqual(run(repo, "status", "--porcelain", "--untracked-files=all"), "")
        self.assertEqual((repo / "topic.txt").read_text(), "topic\n")

    def test_gate_worktree_uses_runs_root_when_tmpdir_is_unset(self):
        repo, _ = self.repo()
        candidate = self.topic(repo)
        forbidden = self.root / "forbidden-host-tmp"
        forbidden.mkdir()
        seen_checkout = self.root / "seen-checkout"
        runs_root = self.root / "runs"
        old_tempdir = tempfile.tempdir
        try:
            tempfile.tempdir = str(forbidden)
            with mock.patch.dict(os.environ, {"CORENIUS_RUNS": str(runs_root)}, clear=False):
                os.environ.pop("TMPDIR", None)
                code, _, err = self.call([
                    candidate,
                    "--repo", str(repo),
                    "--owner-mainline-override", "owner",
                    "--gate", f'printf %s "$PWD" > {q(seen_checkout)}',
                ])
        finally:
            tempfile.tempdir = old_tempdir

        self.assertEqual(code, land.OK, err)
        self.assertTrue(Path(seen_checkout.read_text()).is_relative_to(runs_root))
        self.assertEqual(list(forbidden.rglob("*")), [])

    def test_landing_checked_out_target_keeps_checkout_clean_and_populated(self):
        repo, _ = self.repo()
        candidate = self.topic(repo)

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--owner-mainline-override", "owner",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), candidate)
        self.assertEqual(self.rev(repo, "main"), candidate)
        self.assertEqual(run(repo, "status", "--porcelain", "--untracked-files=all"), "")
        self.assertEqual((repo / "topic.txt").read_text(), "topic\n")

    def test_checked_out_target_overlap_refuses_without_moving_anything(self):
        repo, _ = self.repo()
        (repo / "shared.txt").write_text("base\n")
        run(repo, "add", "shared.txt")
        run(repo, "commit", "-qm", "shared")
        candidate = self.topic(repo, path="shared.txt", text="topic\n")
        main_before = self.rev(repo, "main")
        topic_before = self.rev(repo, "topic")
        (repo / "shared.txt").write_text("dirty\n")

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--owner-mainline-override", "owner",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.OVERLAP, err)
        self.assertEqual(out, "")
        self.assertIn("shared.txt", err)
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertEqual(self.rev(repo, "topic"), topic_before)
        self.assertEqual(topic_before, candidate)
        self.assertEqual(run(repo, "status", "--porcelain", "--untracked-files=all"), " M shared.txt\n")
        self.assertEqual((repo / "shared.txt").read_text(), "dirty\n")

    def test_linked_target_worktree_overlap_refuses_without_moving_anything(self):
        repo, _ = self.repo()
        (repo / "shared.txt").write_text("base\n")
        run(repo, "add", "shared.txt")
        run(repo, "commit", "-qm", "shared")
        target = self.root / "integration"
        run(repo, "worktree", "add", "-q", "-b", "integration", str(target), "main")
        worktree, candidate = self.topic_worktree(
            repo,
            branch="topic",
            base="integration",
            path="shared.txt",
            text="topic\n",
        )
        run(repo, "worktree", "remove", "--force", str(worktree))
        target_before = self.rev(repo, "integration")
        topic_before = self.rev(repo, "topic")
        (target / "shared.txt").write_text("dirty\n")

        code, out, err = self.call(["topic", "--repo", str(repo), "--mainline", "integration", "--no-push", "--gate", "true"])

        self.assertEqual(code, land.OVERLAP, err)
        self.assertEqual(out, "")
        self.assertIn("shared.txt", err)
        self.assertEqual(self.rev(repo, "integration"), target_before)
        self.assertEqual(self.rev(repo, "topic"), topic_before)
        self.assertEqual(topic_before, candidate)
        self.assertEqual(run(target, "status", "--porcelain", "--untracked-files=all"), " M shared.txt\n")
        self.assertEqual((target / "shared.txt").read_text(), "dirty\n")

    def test_stale_branch_candidate_refuses_without_touching_worker_worktree(self):
        repo, _ = self.repo()
        worker, candidate = self.topic_worktree(repo)
        worker_status = run(worker, "status", "--porcelain", "--untracked-files=all")
        (repo / "main.txt").write_text("main moved\n")
        run(repo, "add", "main.txt")
        run(repo, "commit", "-qm", "main moved")
        main_before = self.rev(repo, "main")
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--owner-mainline-override", "owner",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, getattr(land, "STALE_CANDIDATE", land.LOST_RACE), err)
        self.assertEqual(out, "")
        self.assertIn("new Candidate on the new base", err)
        self.assertFalse(gate_seen.exists())
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertEqual(self.rev(repo, "topic"), candidate)
        self.assertEqual(run(worker, "status", "--porcelain", "--untracked-files=all"), worker_status)

    def test_failing_gate_leaves_local_mainline_and_remote_unchanged(self):
        repo, remote = self.repo(with_remote=True)
        topic = self.topic(repo)
        local_before = self.rev(repo, "main")
        remote_before = self.rev(remote, "main")
        seen_tip = self.root / "seen-tip"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--owner-mainline-override", "owner",
            "--gate", f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {q(seen_tip)}; exit 42',
        ])

        self.assertEqual(code, land.GATE_FAILED, err)
        self.assertEqual(out, "")
        self.assertEqual(seen_tip.read_text(), topic)
        self.assertEqual(self.rev(repo, "main"), local_before)
        self.assertEqual(self.rev(remote, "main"), remote_before)
        self.assertEqual(self.rev(repo, "topic"), topic)

    def test_concurrent_landings_leave_exactly_one_landed_and_one_stale(self):
        repo, _ = self.repo()
        left = self.topic(repo, branch="left", path="left.txt", text="left\n")
        right = self.topic(repo, branch="right", path="right.txt", text="right\n")
        gate = "sleep 0.2"

        processes = [
            subprocess.Popen(
                [
                    sys.executable,
                    str(SCRIPT),
                    branch,
                    "--repo",
                    str(repo),
                    "--owner-mainline-override",
                    "owner",
                    "--gate",
                    gate,
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
            )
            for branch in ("left", "right")
        ]
        results = [p.communicate(timeout=20) for p in processes]
        codes = [p.returncode for p in processes]

        self.assertEqual(sorted(codes), [land.OK, getattr(land, "STALE_CANDIDATE", land.LOST_RACE)], results)
        landed = [out.strip() for out, _ in results if out.strip()]
        self.assertEqual(len(landed), 1, results)
        self.assertIn(landed[0], {left, right})
        self.assertEqual(self.rev(repo, "main"), landed[0])
        stale_errors = [err for (out, err), code in zip(results, codes) if code != land.OK]
        self.assertEqual(len(stale_errors), 1, results)
        self.assertIn("new Candidate on the new base", stale_errors[0])
        self.assertEqual(self.rev(repo, "left"), left)
        self.assertEqual(self.rev(repo, "right"), right)

    def test_overlap_with_tracked_main_change_exits_3_and_lists_only_tracked_path(self):
        repo, _ = self.repo()
        (repo / "shared.txt").write_text("base\n")
        run(repo, "add", "shared.txt")
        run(repo, "commit", "-qm", "shared")
        worktree = self.root / "overlap-topic"
        run(repo, "worktree", "add", "-q", "-b", "topic", str(worktree), "main")
        (worktree / "shared.txt").write_text("topic\n")
        run(worktree, "add", "shared.txt")
        run(worktree, "commit", "-qm", "topic")
        run(repo, "worktree", "remove", "--force", str(worktree))
        (repo / "shared.txt").write_text("dirty\n")
        (repo / "untracked.txt").write_text("not blocking\n")

        code, out, err = self.call(["topic", "--repo", str(repo), "--owner-mainline-override", "owner", "--gate", "true"])

        self.assertEqual(code, land.OVERLAP, err)
        self.assertEqual(out, "")
        self.assertIn("shared.txt", err)
        self.assertNotIn("untracked.txt", err)

    def test_merge_commit_in_branch_exits_4_and_mainline_stays_linear(self):
        repo, _ = self.repo()
        main_before = self.rev(repo, "main")
        side = self.root / "side"
        topic = self.root / "merge-topic"
        run(repo, "worktree", "add", "-q", "-b", "side", str(side), "main")
        (side / "side.txt").write_text("side\n")
        run(side, "add", "side.txt")
        run(side, "commit", "-qm", "side")
        run(repo, "worktree", "add", "-q", "-b", "topic", str(topic), "main")
        (topic / "topic.txt").write_text("topic\n")
        run(topic, "add", "topic.txt")
        run(topic, "commit", "-qm", "topic")
        run(topic, "merge", "--no-ff", "-m", "merge side", "side")
        run(repo, "worktree", "remove", "--force", str(topic))
        run(repo, "worktree", "remove", "--force", str(side))

        code, out, err = self.call(["topic", "--repo", str(repo), "--owner-mainline-override", "owner", "--gate", "true"])

        self.assertEqual(code, land.MERGE_COMMIT, err)
        self.assertEqual(out, "")
        self.assertIn("merge commits", err)
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertEqual(run(repo, "rev-list", "--merges", "main").strip(), "")

    def test_rejected_push_changes_no_ref(self):
        repo, remote = self.repo(with_remote=True)
        topic = self.topic(repo)
        before = self.rev(repo, "main")
        hook = remote / "hooks" / "pre-receive"
        hook.write_text(
            "#!/bin/sh\n"
            "exit 1\n"
        )
        hook.chmod(0o755)

        code, out, err = self.call([
            "topic",
            "--repo",
            str(repo),
            "--owner-mainline-override",
            "owner",
            "--gate",
            "true",
        ])

        self.assertEqual(code, getattr(land, "STALE_CANDIDATE", land.LOST_RACE), err)
        self.assertEqual(out, "")
        self.assertEqual(self.rev(repo, "main"), before)
        self.assertEqual(self.rev(remote, "main"), before)
        self.assertEqual(self.rev(repo, "topic"), topic)

    def test_attempts_option_is_not_exposed(self):
        repo, _ = self.repo()
        topic = self.topic(repo)

        code, out, err = self.call(["topic", "--repo", str(repo), "--attempts", "2"])

        self.assertEqual(code, land.USAGE)
        self.assertEqual(out, "")
        self.assertIn("unrecognized arguments: --attempts 2", err)

    def test_no_remote_standalone_cli_fast_forwards_local_mainline_and_prints_sha(self):
        repo, _ = self.repo()
        topic = self.topic(repo)

        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "topic",
                "--repo",
                str(repo),
                "--owner-mainline-override",
                "owner",
                "--allow-no-gate",
            ],
            capture_output=True,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
            text=True,
        )

        self.assertEqual(result.returncode, land.OK, result.stderr)
        self.assertEqual(result.stdout.strip(), topic)
        self.assertEqual(self.rev(repo, "main"), topic)

    def test_no_push_with_remote_fast_forwards_only_local_mainline(self):
        repo, remote = self.repo(with_remote=True)
        topic = self.topic(repo)
        remote_before = self.rev(remote, "main")

        code, out, err = self.call([
            "topic",
            "--repo",
            str(repo),
            "--no-push",
            "--owner-mainline-override",
            "owner",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), topic)
        self.assertEqual(self.rev(repo, "main"), topic)
        self.assertEqual(self.rev(remote, "main"), remote_before)

    def test_add_parser_registers_land_subcommand_with_run_function(self):
        repo, _ = self.repo()
        topic = self.topic(repo)
        parser = argparse.ArgumentParser()
        sub = parser.add_subparsers(dest="command", required=True)
        land.add_parser(sub)
        args = parser.parse_args([
            "land",
            "topic",
            "--repo",
            str(repo),
            "--owner-mainline-override",
            "owner",
            "--allow-no-gate",
        ])
        out, err = io.StringIO(), io.StringIO()

        with redirect_stdout(out), redirect_stderr(err):
            code = args.func(args)

        self.assertEqual(code, land.OK, err.getvalue())
        self.assertEqual(out.getvalue().strip(), topic)
        self.assertEqual(self.rev(repo, "main"), topic)

    def test_ticket_help_states_acceptance_record_order(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--help"],
            capture_output=True,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        help_text = " ".join(result.stdout.split())
        self.assertIn("acceptance record", result.stdout)
        self.assertIn("ticket landings need effort records on a record branch", help_text)
        self.assertIn("effort.py move-records <effort>", help_text)
        self.assertNotIn("or integration branch", result.stdout)
        self.assertNotIn("will be committed after landing", result.stdout)
        self.assertIn("--allow-no-gate", result.stdout)

    def test_standalone_mainline_landing_requires_owner_and_gate_or_explicit_no_gate(self):
        repo, _ = self.repo()
        topic = self.topic(repo)
        main_before = self.rev(repo, "main")

        code, out, err = self.call(["topic", "--repo", str(repo), "--no-push"])

        self.assertEqual(code, land.MAINLINE_REFUSED, err)
        self.assertEqual(out, "")
        self.assertIn("owner mainline override", err)
        self.assertEqual(self.rev(repo, "main"), main_before)

        code, out, err = self.call([
            "topic",
            "--repo",
            str(repo),
            "--no-push",
            "--owner-mainline-override",
            "owner",
        ])

        self.assertEqual(code, land.NO_GATE_REFUSED, err)
        self.assertEqual(out, "")
        self.assertIn("no gate", err)
        self.assertEqual(self.rev(repo, "main"), main_before)

        code, out, err = self.call([
            "topic",
            "--repo",
            str(repo),
            "--no-push",
            "--owner-mainline-override",
            "owner",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), topic)
        self.assertEqual(self.rev(repo, "main"), topic)

    def test_ticket_landing_refuses_missing_acceptance_record_without_running_gate(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        topic = self.topic(repo)
        self.record_worktree(repo)
        main_before = self.rev(repo, "main")
        integration_before = self.rev(repo, "effort/sample")
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertEqual(out, "")
        self.assertIn("acceptance record missing", err)
        self.assertFalse(gate_seen.exists())
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertEqual(self.rev(repo, "effort/sample"), integration_before)
        self.assertEqual(self.rev(repo, "topic"), topic)

    def test_ticket_landing_uses_effort_integration_branch_by_default(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        topic = self.topic(repo, base="effort/sample")
        main_before = self.rev(repo, "main")
        records = self.record_worktree(repo)
        self.write_acceptance(records, topic)
        self.commit_acceptance(records)
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), topic)
        self.assertTrue(gate_seen.exists())
        self.assertEqual(self.rev(repo, "effort/sample"), topic)
        self.assertEqual(self.rev(repo, "main"), main_before)

    def test_ticket_landing_from_main_checkout_finds_effort_record_in_integration_worktree(self):
        repo, _ = self.repo()
        integration = self.effort_ticket_worktree(repo)
        worker, topic = self.topic_worktree(repo, branch="topic", base="effort/sample")
        run(repo, "worktree", "remove", "--force", str(worker))
        records = self.record_worktree(repo)
        self.write_acceptance(records, topic)
        self.commit_acceptance(records)
        main_before = self.rev(repo, "main")
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), topic)
        self.assertTrue(gate_seen.exists())
        self.assertEqual(self.rev(repo, "effort/sample"), topic)
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertEqual(run(repo, "status", "--porcelain", "--untracked-files=all"), "")

    def test_repo_effort_record_wins_over_other_worktree_copies(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        duplicate = self.root / "duplicate"
        run(repo, "worktree", "add", "-q", "-b", "work/duplicate", str(duplicate), "main")
        topic = self.topic(repo, base="main")
        self.record_worktree(repo)

        code, out, err = self.call([
            "topic",
            "--repo",
            str(repo),
            "--no-push",
            "--effort",
            "sample",
            "--ticket",
            "01",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertEqual(out, "")
        self.assertIn("acceptance record missing", err)

    def test_ticket_landing_refuses_mainline_target_without_owner_override_and_records_override(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        topic = self.topic(repo, base="main")
        main_before = self.rev(repo, "main")
        integration_before = self.rev(repo, "effort/sample")
        records = self.record_worktree(repo)
        self.write_acceptance(records, topic)
        self.commit_acceptance(records)
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--mainline", "main",
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, 8, err)
        self.assertEqual(out, "")
        self.assertIn("owner mainline override", err)
        self.assertFalse(gate_seen.exists())
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertEqual(self.rev(repo, "effort/sample"), integration_before)

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--mainline", "main",
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--owner-mainline-override", "owner",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), topic)
        self.assertEqual(self.rev(repo, "main"), topic)
        self.assertEqual(self.rev(repo, "effort/sample"), integration_before)
        record_path = records / "docs/efforts/sample/evidence/mainline-landing-overrides.jsonl"
        record = json.loads(record_path.read_text().splitlines()[-1])
        self.assertEqual(record["owner"], "owner")
        self.assertEqual(record["effort"], "sample")
        self.assertEqual(record["ticket"], "01")
        self.assertEqual(record["target"], "main")
        self.assertEqual(record["candidate"], topic)

    def test_effort_without_base_line_treats_main_as_base_for_owner_override(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        effort = repo / "docs/efforts/sample/EFFORT.md"
        effort.write_text(effort.read_text().replace("Base: main\n", ""))
        run(repo, "add", "docs/efforts/sample/EFFORT.md")
        run(repo, "commit", "-qm", "remove base line")
        topic = self.topic(repo, base="main")
        records = self.record_worktree(repo)
        self.write_acceptance(records, topic)
        self.commit_acceptance(records)
        main_before = self.rev(repo, "main")

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--mainline", "main",
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", "true",
        ])

        self.assertEqual(code, land.MAINLINE_REFUSED, err)
        self.assertEqual(out, "")
        self.assertIn("owner mainline override", err)
        self.assertEqual(self.rev(repo, "main"), main_before)

    def test_ticket_landing_refuses_record_for_another_commit_and_missing_axis(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        topic = self.topic(repo)
        records = self.record_worktree(repo)
        self.write_acceptance(records, self.rev(repo, "main"))
        self.commit_acceptance(records)

        code, _, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertIn("record names another commit", err)

        (records / "docs" / "efforts" / "sample" / "evidence" / "acceptance-01.jsonl").write_text("")
        self.write_acceptance_json(
            records,
            {
                "schema": "corenius.acceptance.v1",
                "effort": "sample",
                "ticket": "01",
                "candidate": topic,
                "reviewer": "lead",
                "reviewer_context": "lead",
                "axes": {"standards": "passed"},
                "required_checks": [{"name": "unit", "result": "passed"}],
                "owner_waiver": None,
                "accepted": True,
            },
        )
        self.commit_acceptance(records)

        code, _, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertIn("missing review axis: spec", err)

    def test_ticket_landing_refuses_failing_check_unless_owner_waived(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        topic = self.topic(repo)
        integration_before = self.rev(repo, "effort/sample")
        records = self.record_worktree(repo)
        self.write_acceptance_json(
            records,
            {
                "schema": "corenius.acceptance.v1",
                "effort": "sample",
                "ticket": "01",
                "candidate": topic,
                "reviewer": "lead",
                "reviewer_context": "lead",
                "axes": {"standards": "passed", "spec": "passed"},
                "required_checks": [{"name": "unit", "result": "failed"}],
                "owner_waiver": None,
                "accepted": True,
            },
        )
        self.commit_acceptance(records)

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertEqual(out, "")
        self.assertIn("failing required check without owner waiver: unit", err)
        self.assertEqual(self.rev(repo, "effort/sample"), integration_before)

        (records / "docs" / "efforts" / "sample" / "evidence" / "acceptance-01.jsonl").write_text("")
        self.write_acceptance(records, topic, check="failed", waiver=True)
        self.commit_acceptance(records)

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), topic)
        self.assertEqual(self.rev(repo, "effort/sample"), topic)

    def test_ticket_file_status_and_checked_acceptance_items_do_not_satisfy_landing(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        topic = self.topic(repo)
        self.record_worktree(repo)
        ticket = next((repo / "docs" / "efforts" / "sample" / "tickets").glob("01-*.md"))
        ticket.write_text(ticket.read_text().replace("Status: claimed", "Status: resolved").replace("- [ ] works", "- [x] works"))
        main_before = self.rev(repo, "main")

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--allow-no-gate",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertEqual(out, "")
        self.assertIn("acceptance record missing", err)
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertNotEqual(self.rev(repo, "effort/sample"), topic)

    def test_ticket_landing_without_record_branch_refuses_before_gate(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        target = self.root / "integration"
        run(repo, "worktree", "add", "-q", str(target), "effort/sample")
        worker, topic = self.topic_worktree(repo, branch="topic", base="effort/sample")
        run(repo, "worktree", "remove", "--force", str(worker))
        self.write_acceptance(target, topic)
        self.commit_acceptance(target)
        before = self.rev(repo, "effort/sample")
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(target),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, 11, err)
        self.assertEqual(out, "")
        self.assertIn("ticket landings need effort records on a record branch", err)
        self.assertIn("effort.py move-records sample", err)
        self.assertEqual(self.rev(repo, "effort/sample"), before)
        self.assertFalse(gate_seen.exists())

    def test_ticket_landing_ignores_uncommitted_acceptance_from_target_worktree(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        target = self.root / "integration"
        run(repo, "worktree", "add", "-q", str(target), "effort/sample")
        self.record_worktree(repo)
        worker, topic = self.topic_worktree(repo, branch="topic", base="effort/sample")
        run(repo, "worktree", "remove", "--force", str(worker))
        self.write_raw_acceptance(target, topic)
        record_rel = Path("docs/efforts/sample/evidence/acceptance-01.jsonl")
        record_before = (target / record_rel).read_text()
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(target),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertEqual(out, "")
        self.assertIn("acceptance record missing", err)
        self.assertNotEqual(self.rev(repo, "effort/sample"), topic)
        self.assertFalse(gate_seen.exists())
        self.assertEqual((target / record_rel).read_text(), record_before)
        self.assertEqual(
            run(target, "status", "--porcelain", "--untracked-files=all"),
            f"?? {record_rel.as_posix()}\n",
        )

    def test_untracked_acceptance_in_third_worktree_does_not_authorize_landing(self):
        repo, _ = self.repo()
        self.effort_ticket(repo)
        integration = self.root / "integration"
        third = self.root / "third"
        run(repo, "worktree", "add", "-q", str(integration), "effort/sample")
        run(repo, "worktree", "add", "-q", "-b", "agent/third", str(third), "effort/sample")
        self.record_worktree(repo)
        worker, topic = self.topic_worktree(repo, branch="topic", base="effort/sample")
        run(repo, "worktree", "remove", "--force", str(worker))
        self.write_raw_acceptance(third, topic)
        before = self.rev(repo, "effort/sample")

        code, out, err = self.call([
            "topic",
            "--repo", str(third),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", "true",
        ])

        self.assertEqual(code, land.ACCEPTANCE_FAILED, err)
        self.assertEqual(out, "")
        self.assertIn("acceptance record missing", err)
        self.assertEqual(self.rev(repo, "effort/sample"), before)

    def test_ticket_landing_allows_committed_acceptance_record_on_record_branch(self):
        repo, _ = self.repo()
        target = self.effort_ticket_worktree(repo)
        records = self.root / "records"
        run(repo, "worktree", "add", "-q", "-b", "effort/sample-records", str(records), "effort/sample")
        effort_file = records / "docs/efforts/sample/EFFORT.md"
        effort_file.write_text(effort_file.read_text() + "Record branch: effort/sample-records\n")
        run(records, "add", "docs/efforts/sample/EFFORT.md")
        run(records, "commit", "-qm", "move records")
        run(target, "rm", "-qr", "docs/efforts/sample")
        run(target, "commit", "-qm", "remove effort records from code branch")
        worker, topic = self.topic_worktree(repo, branch="topic", base="effort/sample")
        run(repo, "worktree", "remove", "--force", str(worker))
        self.write_acceptance(records, topic)
        record_rel = Path("docs/efforts/sample/evidence/acceptance-01.jsonl")
        run(records, "add", record_rel.as_posix())
        run(records, "commit", "-qm", "acceptance record")
        draft = records / "docs/efforts/sample/evidence/draft-notes-in-progress.md"
        draft.write_text("not ready\n")
        ticket = records / "docs/efforts/sample/tickets/01-one.md"
        ticket.write_text(ticket.read_text() + "- local uncommitted note\n")
        target_before = self.rev(repo, "effort/sample")
        gate_seen = self.root / "gate-ran"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--no-push",
            "--effort", "sample",
            "--ticket", "01",
            "--gate", f"touch {q(gate_seen)}",
        ])

        self.assertEqual(code, land.OK, err)
        self.assertEqual(out.strip(), topic)
        self.assertEqual(self.rev(repo, "effort/sample"), topic)
        self.assertNotEqual(target_before, topic)
        self.assertTrue(gate_seen.exists())
        self.assertEqual(run(target, "status", "--porcelain", "--untracked-files=all"), "")
        self.assertEqual(
            run(records, "status", "--porcelain", "--untracked-files=all"),
            " M docs/efforts/sample/tickets/01-one.md\n"
            "?? docs/efforts/sample/evidence/draft-notes-in-progress.md\n",
        )
        committed_paths = run(records, "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD").splitlines()
        self.assertEqual(committed_paths, ["docs/efforts/sample/evidence/landing-01.jsonl"])
        landing_record = json.loads((records / "docs/efforts/sample/evidence/landing-01.jsonl").read_text().splitlines()[-1])
        self.assertEqual(landing_record["ticket"], "01")
        self.assertEqual(landing_record["candidate"], topic)
        self.assertEqual(landing_record["target"], "effort/sample")

    def test_untracked_file_that_candidate_would_overwrite_refuses_before_push(self):
        repo, remote = self.repo(with_remote=True)
        topic = self.topic(repo, path="new.txt", text="candidate\n")
        before = self.rev(repo, "main")
        (repo / "new.txt").write_text("local\n")

        code, out, err = self.call(["topic", "--repo", str(repo), "--owner-mainline-override", "owner", "--gate", "true"])

        self.assertEqual(code, land.OVERLAP, err)
        self.assertEqual(out, "")
        self.assertIn("new.txt", err)
        self.assertEqual(self.rev(remote, "main"), before)
        self.assertEqual(self.rev(repo, "main"), before)


if __name__ == "__main__":
    unittest.main()

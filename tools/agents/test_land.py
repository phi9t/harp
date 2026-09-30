"""land.py against throwaway git repositories."""

from __future__ import annotations

import argparse
import io
import os
import shlex
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import land  # noqa: E402


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

    def topic(self, repo: Path, path="topic.txt", text="topic\n"):
        worktree = self.root / f"topic-{len(list(self.root.glob('topic-*')))}"
        run(repo, "worktree", "add", "-q", "-b", "topic", str(worktree), "main")
        (worktree / path).write_text(text)
        run(worktree, "add", path)
        run(worktree, "commit", "-qm", "topic")
        run(repo, "worktree", "remove", "--force", str(worktree))
        return run(repo, "rev-parse", "topic").strip()

    def call(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            code = land.main(argv)
        return code, out.getvalue(), err.getvalue()

    def rev(self, repo: Path, ref: str) -> str:
        return run(repo, "rev-parse", ref).strip()

    def test_failing_gate_leaves_local_mainline_and_remote_unchanged(self):
        repo, remote = self.repo(with_remote=True)
        topic = self.topic(repo)
        local_before = self.rev(repo, "main")
        remote_before = self.rev(remote, "main")
        seen_tip = self.root / "seen-tip"

        code, out, err = self.call([
            "topic",
            "--repo", str(repo),
            "--gate", f'test "$LAND_TIP" = "$(git rev-parse HEAD)" && printf %s "$LAND_TIP" > {q(seen_tip)}; exit 42',
        ])

        self.assertEqual(code, land.GATE_FAILED, err)
        self.assertEqual(out, "")
        self.assertEqual(seen_tip.read_text(), topic)
        self.assertEqual(self.rev(repo, "main"), local_before)
        self.assertEqual(self.rev(remote, "main"), remote_before)
        self.assertEqual(self.rev(repo, "topic"), topic)

    def test_lost_race_rebases_again_reruns_gates_and_lands_pushed_tip(self):
        repo, remote = self.repo(with_remote=True)
        self.topic(repo)
        tips = self.root / "gate-tips"
        marker = self.root / "race-done"
        race_sha = self.root / "race-sha"
        gate = (
            f'printf "%s\\n" "$LAND_TIP" >> {q(tips)}; '
            f"if [ ! -f {q(marker)} ]; then "
            "git checkout -q -B land-race origin/main && "
            "printf race > race.txt && "
            "git add race.txt && "
            "git -c user.email=a@b -c user.name=a commit -qm race && "
            f"git rev-parse HEAD > {q(race_sha)} && "
            "git push -q origin HEAD:main && "
            f"touch {q(marker)}; "
            "fi"
        )

        code, out, err = self.call(["topic", "--repo", str(repo), "--gate", gate, "--attempts", "3"])

        self.assertEqual(code, land.OK, err)
        landed = out.strip()
        observed = tips.read_text().splitlines()
        self.assertEqual(len(observed), 2)
        self.assertNotEqual(observed[0], observed[1])
        self.assertEqual(landed, observed[1])
        self.assertEqual(self.rev(repo, "main"), landed)
        self.assertEqual(self.rev(repo, "topic"), landed)
        self.assertEqual(self.rev(remote, "main"), landed)
        run(repo, "merge-base", "--is-ancestor", race_sha.read_text().strip(), "main")

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

        code, out, err = self.call(["topic", "--repo", str(repo)])

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

        code, out, err = self.call(["topic", "--repo", str(repo)])

        self.assertEqual(code, land.MERGE_COMMIT, err)
        self.assertEqual(out, "")
        self.assertIn("merge commits", err)
        self.assertEqual(self.rev(repo, "main"), main_before)
        self.assertEqual(run(repo, "rev-list", "--merges", "main").strip(), "")

    def test_rejected_first_push_does_not_move_local_main_before_retry(self):
        repo, remote = self.repo(with_remote=True)
        self.topic(repo)
        before = self.rev(repo, "main")
        first_seen = self.root / "first-push-local-main"
        marker = self.root / "hook-rejected-once"
        hook = remote / "hooks" / "pre-receive"
        hook.write_text(
            "#!/bin/sh\n"
            f"if [ ! -f {q(marker)} ]; then\n"
            f"  env -u GIT_DIR -u GIT_WORK_TREE git -C {q(repo)} rev-parse main > {q(first_seen)}\n"
            f"  touch {q(marker)}\n"
            "  exit 1\n"
            "fi\n"
            "exit 0\n"
        )
        hook.chmod(0o755)

        code, out, err = self.call(["topic", "--repo", str(repo), "--attempts", "2"])

        self.assertEqual(code, land.OK, err)
        landed = out.strip()
        self.assertEqual(first_seen.read_text().strip(), before)
        self.assertEqual(self.rev(repo, "main"), landed)
        self.assertEqual(self.rev(remote, "main"), landed)

    def test_no_remote_standalone_cli_fast_forwards_local_mainline_and_prints_sha(self):
        repo, _ = self.repo()
        topic = self.topic(repo)

        result = subprocess.run(
            [sys.executable, str(SCRIPT), "topic", "--repo", str(repo)],
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

        code, out, err = self.call(["topic", "--repo", str(repo), "--no-push"])

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
        args = parser.parse_args(["land", "topic", "--repo", str(repo)])
        out, err = io.StringIO(), io.StringIO()

        with redirect_stdout(out), redirect_stderr(err):
            code = args.func(args)

        self.assertEqual(code, land.OK, err.getvalue())
        self.assertEqual(out.getvalue().strip(), topic)
        self.assertEqual(self.rev(repo, "main"), topic)


if __name__ == "__main__":
    unittest.main()

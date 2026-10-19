"""milano.scm against real throwaway git repositories (no network, no Insula)."""

from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

from milano.scm import (
    MAINLINE,
    ZERO,
    ScmError,
    check_push,
    finish,
    integrate,
    new_worktree,
    setup,
    worktrees,
)


def run(cwd: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def commit(repo: Path, name: str, content: str, message: str | None = None) -> str:
    (repo / name).write_text(content)
    run(repo, "add", name)
    run(repo, "commit", "-q", "-m", message or f"change {name}")
    return run(repo, "rev-parse", "HEAD")


@pytest.fixture
def origin_and_clone(tmp_path: Path) -> tuple[Path, Path]:
    origin = tmp_path / "origin.git"
    run(tmp_path, "init", "-q", "--bare", "-b", MAINLINE, str(origin))
    seed = tmp_path / "seed"
    run(tmp_path, "clone", "-q", str(origin), str(seed))
    for repo in (seed,):
        run(repo, "config", "user.email", "agent@example.com")
        run(repo, "config", "user.name", "agent")
    run(seed, "checkout", "-q", "-b", MAINLINE)
    commit(seed, "README", "root\n")
    run(seed, "push", "-q", "origin", MAINLINE)
    clone = tmp_path / "milano"
    run(tmp_path, "clone", "-q", "-b", MAINLINE, str(origin), str(clone))
    run(clone, "config", "user.email", "agent@example.com")
    run(clone, "config", "user.name", "agent")
    return origin, clone


def gates_seen() -> tuple[list[tuple[str, str]], object]:
    seen: list[tuple[str, str]] = []

    def fake_gate(worktree: Path, revision: str, gate: str) -> None:
        seen.append((revision, gate))

    return seen, fake_gate


def test_new_worktree_branches_from_origin_mainline(origin_and_clone, tmp_path: Path) -> None:
    _, clone = origin_and_clone
    created = new_worktree(clone, "claude", "m4-orchestrator", root=tmp_path / "trees")

    assert created.branch == "claude/m4-orchestrator"
    assert created.path == tmp_path / "trees" / "claude-m4-orchestrator"
    assert created.head == run(clone, "rev-parse", f"origin/{MAINLINE}")
    assert {entry.branch for entry in worktrees(clone)} == {MAINLINE, "claude/m4-orchestrator"}
    with pytest.raises(ScmError, match="<agent>/<topic>"):
        new_worktree(clone, "Claude", "x y", root=tmp_path / "trees")


def test_integrate_rebases_gates_the_exact_tip_and_fast_forwards(origin_and_clone, tmp_path: Path) -> None:
    origin, clone = origin_and_clone
    task = new_worktree(clone, "claude", "feature", root=tmp_path / "trees")
    commit(task.path, "feature.txt", "feature\n")

    # Another agent integrates first; our branch must be rebased onto it.
    other = tmp_path / "other"
    run(tmp_path, "clone", "-q", "-b", MAINLINE, str(origin), str(other))
    run(other, "config", "user.email", "other@example.com")
    run(other, "config", "user.name", "other")
    theirs = commit(other, "other.txt", "other\n")
    run(other, "push", "-q", "origin", MAINLINE)

    seen, fake_gate = gates_seen()
    tip = integrate(clone, "claude/feature", gates=["repo", "rtest"], run_gate=fake_gate)

    assert seen == [(tip, "repo"), (tip, "rtest")]
    assert run(clone, "rev-parse", MAINLINE) == tip
    assert run(origin, "rev-parse", MAINLINE) == tip
    assert run(clone, "rev-parse", f"{tip}~1") == theirs
    assert run(clone, "rev-list", "--merges", tip) == ""


def test_integrate_linearises_merges_and_refuses_dirty_worktrees(origin_and_clone, tmp_path: Path) -> None:
    _, clone = origin_and_clone
    task = new_worktree(clone, "claude", "messy", root=tmp_path / "trees")
    commit(task.path, "a.txt", "a\n")
    run(task.path, "checkout", "-q", "-b", "side", f"origin/{MAINLINE}")
    commit(task.path, "b.txt", "b\n")
    run(task.path, "checkout", "-q", "claude/messy")
    run(task.path, "merge", "-q", "--no-ff", "side", "-m", "merge side")

    (task.path / "a.txt").write_text("dirty\n")
    with pytest.raises(ScmError, match="uncommitted changes"):
        integrate(clone, "claude/messy", gates=[], run_gate=gates_seen()[1])
    run(task.path, "checkout", "--", "a.txt")

    # A plain rebase flattens the merge; the result is linear and integrates.
    tip = integrate(clone, "claude/messy", gates=[], run_gate=gates_seen()[1], push=False)
    assert run(clone, "rev-list", "--merges", tip) == ""


def test_failed_gate_leaves_mainline_untouched(origin_and_clone, tmp_path: Path) -> None:
    origin, clone = origin_and_clone
    before = run(origin, "rev-parse", MAINLINE)
    task = new_worktree(clone, "claude", "broken", root=tmp_path / "trees")
    commit(task.path, "broken.txt", "x\n")

    def failing_gate(worktree: Path, revision: str, gate: str) -> None:
        raise ScmError(f"gate {gate!r} failed on {revision[:12]}")

    with pytest.raises(ScmError, match="gate 'repo' failed"):
        integrate(clone, "claude/broken", gates=["repo"], run_gate=failing_gate)
    assert run(origin, "rev-parse", MAINLINE) == before
    assert run(clone, "rev-parse", MAINLINE) == before


def test_pre_push_rules_for_mainline(origin_and_clone) -> None:
    origin, clone = origin_and_clone
    base = run(clone, "rev-parse", "HEAD")
    ahead = commit(clone, "x.txt", "x\n")
    ref = f"refs/heads/{MAINLINE}"

    assert check_push(clone, ref, ahead, base) == []
    assert check_push(clone, "refs/heads/claude/topic", base, ahead) == []
    assert check_push(clone, ref, base, ahead) == [f"{MAINLINE} only moves by fast-forward (rebase first)"]
    assert check_push(clone, ref, ZERO, base) == [f"{MAINLINE} cannot be deleted"]

    run(clone, "checkout", "-q", "-b", "side", base)
    side = commit(clone, "y.txt", "y\n")
    run(clone, "checkout", "-q", MAINLINE)
    run(clone, "merge", "-q", "--no-ff", side, "-m", "merge")
    merged = run(clone, "rev-parse", "HEAD")
    problems = check_push(clone, ref, merged, base)
    assert len(problems) == 1 and "merge commits" in problems[0]


def test_finish_removes_only_integrated_work(origin_and_clone, tmp_path: Path) -> None:
    _, clone = origin_and_clone
    task = new_worktree(clone, "claude", "done", root=tmp_path / "trees")
    commit(task.path, "done.txt", "d\n")
    with pytest.raises(ScmError, match="integrate it first"):
        finish(clone, "claude/done")

    integrate(clone, "claude/done", gates=[], run_gate=gates_seen()[1])
    finish(clone, "claude/done")
    assert not task.path.exists()
    assert "claude/done" not in run(clone, "branch", "--list", "claude/done")


def test_setup_is_idempotent(origin_and_clone) -> None:
    _, clone = origin_and_clone

    assert "core.hooksPath=.githooks" in setup(clone)
    assert setup(clone) == []
    assert run(clone, "config", "--get", "pull.rebase") == "true"


def test_finish_works_when_run_from_the_task_worktree(origin_and_clone, tmp_path: Path) -> None:
    _, clone = origin_and_clone
    task = new_worktree(clone, "claude", "self", root=tmp_path / "trees")
    commit(task.path, "self.txt", "s\n")
    integrate(task.path, "claude/self", gates=[], run_gate=gates_seen()[1])

    finish(task.path, "claude/self")

    assert not task.path.exists()
    assert run(clone, "branch", "--list", "claude/self") == ""

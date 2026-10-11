#!/usr/bin/env python3
"""Read-only consistency checks for coordinator-owned CURRENT projections."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess


def check_projection(projection, registered, now=None):
    problems = []
    worktree = projection.get("worktree")
    if worktree and worktree not in registered:
        problems.append(f"unregistered worktree: {worktree}")
    if projection.get("execution_state") == "quiescent":
        if projection.get("writer") or worktree:
            problems.append("quiescent projection carries active writer/worktree")
        if (projection.get("next_launch") or {}).get("command"):
            problems.append("quiescent projection carries executable next launch")
    claim = projection.get("claim")
    if isinstance(claim, dict) and claim.get("state") == "active":
        try:
            expires = datetime.fromisoformat(claim["expires"].replace("Z", "+00:00"))
            if expires.tzinfo is None:
                raise ValueError("claim expiry requires timezone")
            if expires <= (now or datetime.now(timezone.utc)):
                problems.append("active claim has expired")
        except (KeyError, ValueError, TypeError) as exc:
            problems.append(f"invalid active claim expiry: {exc}")
    return problems


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    args = parser.parse_args()
    repo = args.repo.resolve()
    output = subprocess.check_output(["git", "-C", str(repo), "worktree", "list", "--porcelain"], text=True)
    registered = {line[9:] for line in output.splitlines() if line.startswith("worktree ")}
    problems = []
    projections = sorted((repo / "docs/efforts").glob("*/CURRENT.json"))
    for path in projections:
        try:
            projection = json.loads(path.read_text())
            if not isinstance(projection, dict):
                raise ValueError("projection must be an object")
            findings = check_projection(projection, registered)
        except (ValueError, TypeError, AttributeError) as exc:
            findings = [f"invalid projection: {exc}"]
        problems.extend(f"{path.relative_to(repo)}: {item}" for item in findings)
    for problem in problems:
        print(f"FAIL {problem}")
    if not problems:
        print(f"PASS operational state ({len(projections)} current projections)")
    return int(bool(problems))


if __name__ == "__main__":
    raise SystemExit(main())

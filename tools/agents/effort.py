#!/usr/bin/env python3
"""Durable efforts: decisions and state in the repo, execution detail in scratch.

An effort has two homes (docs/agents/agentic-engineering.md, "Efforts"):

* ``docs/efforts/<slug>/`` tracked: EFFORT.md (charter), STATE.md (the
  dehydrated snapshot), decisions/, tickets/, evidence/.
* ``.scratch/<slug>/`` gitignored: worktrees' build output, run logs, drafts.

Commands
  new <slug>          create the effort branch, worktree and durable files
  decide <slug> <t>   add a decision file (append-only; supersede, never edit)
  dehydrate <slug>    refresh STATE.md's machine block from git and run dirs,
                      require the human part, refuse dirty worktrees, commit
  rehydrate <slug>    verify every pointer in STATE.md, restore missing
                      worktrees, print a briefing and a drift report
  status              list efforts with status and staleness

Exit codes: 0 ok, 1 warnings (for example lost scratch), 2 errors. Standard
library only; reuses the git helpers in worker.py.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, git, main_worktree, now  # noqa: E402

OK, WARN, ERROR = 0, 1, 2
START, END = "<!-- effort:machine:start -->", "<!-- effort:machine:end -->"
SLUG = re.compile(r"^[a-z0-9][a-z0-9-]*$")
HUMAN_REQUIRED = ("Where we are", "Next actions")
HUMAN_SECTIONS = ("Where we are", "Next actions", "Open questions", "Traps", "Do not redo")

EFFORT_TEMPLATE = """\
# Effort: {slug}

Status: active
Base: {base}
Lead:

## Intent

_TODO: what this effort is for, in a paragraph._

## Scope and non-goals

## Milestones and exit criteria

- [ ] _TODO: each milestone with a command or document that checks it._
"""

STATE_TEMPLATE = """\
# State: {slug}

{start}
{end}

## Where we are

_TODO_

## Next actions

_TODO_

## Open questions

## Traps

Things a fresh agent would get wrong.

## Do not redo
"""

DECISION_TEMPLATE = """\
# {title}

Date: {date}
Supersedes:

## Decision

## Why

## Consequences
"""


def repo_root(repo: str | None) -> Path:
    return Path(git(repo or ".", "rev-parse", "--show-toplevel").strip())


def effort_dir(root: Path, slug: str) -> Path:
    if not SLUG.match(slug):
        raise WorkerError(f"bad effort slug {slug!r}: use lowercase letters, digits, hyphens")
    return root / "docs" / "efforts" / slug


def scratch_dir(root: Path, slug: str) -> Path:
    return main_worktree(root) / ".scratch" / slug


def rev(repo: Path | str, ref: str) -> str | None:
    out = git(repo, "rev-parse", "--verify", "-q", ref + "^{commit}", check=False).strip()
    return out or None


def is_ancestor(repo: Path | str, older: str, newer: str) -> bool:
    import subprocess

    return subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", older, newer], capture_output=True
    ).returncode == 0


def only_effort_changes(repo: Path, old: str, new: str, slug: str) -> bool:
    """True if old..new touches nothing outside the effort's own directory.

    The effort branch's recorded head predates the dehydrate commit itself, so
    that one commit must not count as drift.
    """
    names = git(repo, "diff", "--name-only", old, new).split()
    return all(n.startswith(f"docs/efforts/{slug}/") for n in names)


# --- STATE.md -----------------------------------------------------------------


def split_state(text: str) -> tuple[dict, str]:
    """Return (machine dict, text with the machine block removed)."""
    if START not in text or END not in text:
        raise WorkerError("STATE.md has no machine block markers")
    head, rest = text.split(START, 1)
    block, tail = rest.split(END, 1)
    match = re.search(r"```json\n(.*?)\n```", block, re.S)
    machine = json.loads(match.group(1)) if match else {}
    return machine, head + START + "\n" + END + tail


def join_state(text: str, machine: dict) -> str:
    block = f"{START}\n```json\n{json.dumps(machine, indent=2, sort_keys=True)}\n```\n{END}"
    head, rest = text.split(START, 1)
    _, tail = rest.split(END, 1)
    return head + block + tail


def human_sections(text: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for match in re.finditer(r"^## (.+?)\n(.*?)(?=^## |\Z)", text, re.M | re.S):
        out[match.group(1).strip()] = match.group(2).strip()
    return out


def human_problems(text: str) -> list[str]:
    sections = human_sections(text)
    problems = []
    for name in HUMAN_REQUIRED:
        body = sections.get(name, "")
        if not body or "_TODO" in body:
            problems.append(f"STATE.md section '{name}' is empty or still a placeholder")
    return problems


def dirty_paths(worktree: Path) -> list[str]:
    paths = []
    for line in git(worktree, "status", "--porcelain", "-uall").splitlines():
        paths.append(line[3:].split(" -> ")[-1].strip().strip('"'))
    return paths


def registered_worktrees(root: Path) -> dict[str, dict]:
    """Map path -> {branch, head} for every registered worktree."""
    out: dict[str, dict] = {}
    current: dict = {}
    for line in git(root, "worktree", "list", "--porcelain").splitlines() + [""]:
        if not line:
            if current.get("path"):
                out[current["path"]] = current
            current = {}
        elif line.startswith("worktree "):
            current["path"] = line[9:]
        elif line.startswith("HEAD "):
            current["head"] = line[5:]
        elif line.startswith("branch "):
            current["branch"] = line[7:].removeprefix("refs/heads/")
    return out


def describe_worktree(root: Path, path: str, own_effort: Path | None) -> dict:
    p = Path(path)
    if not p.exists():
        raise WorkerError(f"worktree {path} does not exist")
    branch = git(p, "rev-parse", "--abbrev-ref", "HEAD").strip()
    head = git(p, "rev-parse", "HEAD").strip()
    dirty = dirty_paths(p)
    pushed = bool(git(p, "branch", "-r", "--contains", head, check=False).strip())
    return {"path": str(p), "branch": branch, "head": head, "dirty": dirty, "pushed": pushed}


def read_run(run_dir: str) -> dict:
    d = Path(run_dir)
    info: dict = {"run_dir": str(d)}
    verdict = d / "verdict.json"
    if verdict.exists():
        data = json.loads(verdict.read_text())
        info["verdict"] = data.get("verdict")
        info["exit_code"] = data.get("exit_code")
    else:
        info["verdict"] = None
    return info


# --- commands -----------------------------------------------------------------


def cmd_new(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    slug = args.slug
    base = args.base
    if not rev(root, base):
        raise WorkerError(f"base branch {base!r} not found")
    worktree = root.parent / f"{root.name}-{slug}"
    branch = f"effort/{slug}"
    if worktree.exists():
        raise WorkerError(f"{worktree} already exists")
    if rev(root, branch):
        raise WorkerError(f"branch {branch} already exists")
    git(root, "worktree", "add", "-q", "-b", branch, str(worktree), base)
    edir = effort_dir(worktree, slug)
    (edir / "decisions").mkdir(parents=True)
    (edir / "tickets").mkdir()
    (edir / "evidence").mkdir()
    for keep in ("decisions", "tickets", "evidence"):
        (edir / keep / ".gitkeep").write_text("")
    (edir / "EFFORT.md").write_text(EFFORT_TEMPLATE.format(slug=slug, base=base))
    (edir / "STATE.md").write_text(STATE_TEMPLATE.format(slug=slug, start=START, end=END))
    scratch_dir(root, slug).mkdir(parents=True, exist_ok=True)
    print(json.dumps({"effort": slug, "worktree": str(worktree), "branch": branch,
                      "durable": str(edir), "scratch": str(scratch_dir(root, slug))}))
    return OK


def cmd_decide(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    edir = effort_dir(root, args.slug)
    if not edir.is_dir():
        raise WorkerError(f"no effort {args.slug} in {root}")
    words = re.sub(r"[^a-z0-9]+", "-", args.title.lower()).strip("-")[:60]
    date = datetime.now(timezone.utc).strftime("%Y%m%d")
    path = edir / "decisions" / f"{date}-{words}.md"
    if path.exists():
        raise WorkerError(f"{path.name} exists; decisions are append-only, pick a new title")
    path.write_text(DECISION_TEMPLATE.format(title=args.title, date=date))
    print(str(path))
    return OK


def cmd_dehydrate(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    edir = effort_dir(root, args.slug)
    state_path = edir / "STATE.md"
    if not state_path.exists():
        raise WorkerError(f"no STATE.md at {state_path}")
    machine_old, stripped = split_state(state_path.read_text())
    problems = human_problems(stripped)

    wanted = {str(root)}
    wanted.update(str(Path(w).resolve()) for w in args.worktree)
    wanted.update(w["path"] for w in machine_old.get("worktrees", []))
    worktrees = []
    own_prefix = f"docs/efforts/{args.slug}/"
    for path in sorted(wanted):
        if path in args.forget:
            continue
        if not Path(path).exists():
            problems.append(f"recorded worktree {path} is gone; dehydrate refuses to drop it silently (use --forget)")
            continue
        info = describe_worktree(root, path, edir)
        stray = [p for p in info["dirty"] if not (path == str(root) and p.startswith(own_prefix))]
        if stray:
            problems.append(f"worktree {path} has uncommitted changes: {', '.join(stray[:5])}")
        info["dirty"] = []
        worktrees.append(info)

    runs = {r["run_dir"]: r for r in machine_old.get("runs", [])}
    for run_dir in args.run:
        runs[str(Path(run_dir).resolve())] = read_run(run_dir)
    for key, run in list(runs.items()):
        if run.get("verdict") is None and Path(key).exists():
            runs[key] = read_run(key)

    if problems:
        for p in problems:
            print(f"BLOCKED: {p}", file=sys.stderr)
        return ERROR

    base_branch = args.base
    base_commit = git(root, "merge-base", base_branch, "HEAD").strip()
    machine = {
        "effort": args.slug,
        "dehydrated": now(),
        "base_branch": base_branch,
        "base_commit": base_commit,
        "worktrees": worktrees,
        "runs": sorted(runs.values(), key=lambda r: r["run_dir"]),
    }
    state_path.write_text(join_state(stripped, machine))
    git(root, "add", f"docs/efforts/{args.slug}")
    if git(root, "diff", "--cached", "--name-only", "--", f"docs/efforts/{args.slug}").strip():
        git(root, "commit", "-q", "-m", f"Dehydrate effort {args.slug}", "--", f"docs/efforts/{args.slug}")
    machine["state_commit"] = git(root, "rev-parse", "HEAD").strip()
    print(json.dumps({"dehydrated": args.slug, "worktrees": len(worktrees), "runs": len(machine["runs"]),
                      "commit": machine["state_commit"]}))
    return OK


def cmd_rehydrate(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    edir = effort_dir(root, args.slug)
    if not (edir / "STATE.md").exists():
        raise WorkerError(f"no STATE.md in {edir}; is this checkout at a commit that has the effort?")
    machine, stripped = split_state((edir / "STATE.md").read_text())
    if not machine:
        raise WorkerError("STATE.md has never been dehydrated (empty machine block)")
    errors: list[str] = []
    warnings: list[str] = []
    repaired: list[str] = []

    base_branch, base_commit = machine["base_branch"], machine["base_commit"]
    if not rev(root, base_commit):
        errors.append(f"base commit {base_commit[:12]} is not in this repository")
    elif not rev(root, base_branch):
        errors.append(f"base branch {base_branch} not found")
    elif not is_ancestor(root, base_commit, base_branch):
        errors.append(f"base commit {base_commit[:12]} is not an ancestor of {base_branch}: history was rewritten")
    else:
        behind = git(root, "rev-list", "--count", f"{base_commit}..{base_branch}").strip()
        if behind != "0":
            warnings.append(f"{base_branch} moved {behind} commit(s) since dehydrate")

    registered = registered_worktrees(root)
    for w in machine.get("worktrees", []):
        path, branch, head = w["path"], w["branch"], w["head"]
        if Path(path).exists():
            cur = registered.get(path) or registered.get(str(Path(path).resolve()))
            if cur is None:
                errors.append(f"{path} exists but is not a registered worktree")
                continue
            if cur.get("branch") != branch:
                errors.append(f"{path} is on {cur.get('branch')}, expected {branch}")
            elif cur["head"] != head and not (
                is_ancestor(root, head, cur["head"]) and only_effort_changes(root, head, cur["head"], args.slug)
            ):
                (warnings if is_ancestor(root, head, cur["head"]) else errors).append(
                    f"{branch} at {cur['head'][:12]}, recorded {head[:12]}"
                    + ("" if is_ancestor(root, head, cur["head"]) else " (diverged: recorded work is not reachable)")
                )
            if dirty_paths(Path(path)):
                warnings.append(f"{path} has uncommitted changes")
            continue
        tip = rev(root, branch)
        if tip is None:
            if rev(root, head):
                git(root, "worktree", "add", "-q", "-b", branch, path, head)
                repaired.append(f"recreated branch {branch} at {head[:12]} and worktree {path}")
            else:
                errors.append(f"branch {branch} and commit {head[:12]} are both gone: unlanded work is lost")
            continue
        if tip != head and not is_ancestor(root, head, tip):
            errors.append(f"{branch} at {tip[:12]} does not contain recorded {head[:12]}: diverged")
            continue
        if tip != head and not only_effort_changes(root, head, tip, args.slug):
            warnings.append(f"{branch} advanced from {head[:12]} to {tip[:12]}")
        git(root, "worktree", "add", "-q", path, branch)
        repaired.append(f"restored worktree {path} on {branch}")

    lost = []
    for r in machine.get("runs", []):
        if not Path(r["run_dir"]).exists():
            lost.append(f"run {Path(r['run_dir']).name}: logs lost, recorded verdict {r.get('verdict')}")
    warnings.extend(lost)
    scratch_dir(root, args.slug).mkdir(parents=True, exist_ok=True)

    sections = human_sections(stripped)
    effort_text = (edir / "EFFORT.md").read_text() if (edir / "EFFORT.md").exists() else ""
    status = re.search(r"^Status:\s*(.*)$", effort_text, re.M)
    decisions = sorted(p.name for p in (edir / "decisions").glob("*.md")) if (edir / "decisions").is_dir() else []
    print(f"# Briefing: {args.slug}  (status: {status.group(1).strip() if status else 'unknown'}, "
          f"dehydrated {machine['dehydrated']})\n")
    for name in HUMAN_SECTIONS:
        body = sections.get(name)
        if body:
            print(f"## {name}\n{body}\n")
    print(f"## Decisions ({len(decisions)})")
    for d in decisions:
        print(f"- {d}")
    print("\n## Drift report")
    for label, items in (("repaired", repaired), ("warning", warnings), ("error", errors)):
        for item in items:
            print(f"- {label}: {item}")
    if not (repaired or warnings or errors):
        print("- none")
    return ERROR if errors else (WARN if warnings else OK)


def cmd_status(args: argparse.Namespace) -> int:
    root = repo_root(args.repo)
    efforts = root / "docs" / "efforts"
    rows = []
    for edir in sorted(p for p in efforts.iterdir() if p.is_dir()) if efforts.is_dir() else []:
        text = (edir / "EFFORT.md").read_text() if (edir / "EFFORT.md").exists() else ""
        status = re.search(r"^Status:\s*(.*)$", text, re.M)
        stale = "never dehydrated"
        state = edir / "STATE.md"
        if state.exists():
            machine, _ = split_state(state.read_text())
            if machine:
                when = datetime.strptime(machine["dehydrated"], "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
                stale = f"{(datetime.now(timezone.utc) - when).days}d ago"
        rows.append({"effort": edir.name, "status": status.group(1).strip() if status else "unknown",
                     "dehydrated": stale, "decisions": len(list((edir / "decisions").glob("*.md")))
                     if (edir / "decisions").is_dir() else 0})
    for row in rows:
        print(json.dumps(row))
    return OK


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", help="a checkout of the repository (default: cwd)")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("new")
    p.add_argument("slug")
    p.add_argument("--base", default="main")
    p.set_defaults(fn=cmd_new)
    p = sub.add_parser("decide")
    p.add_argument("slug")
    p.add_argument("title")
    p.set_defaults(fn=cmd_decide)
    p = sub.add_parser("dehydrate")
    p.add_argument("slug")
    p.add_argument("--base", default="main")
    p.add_argument("--worktree", action="append", default=[])
    p.add_argument("--forget", action="append", default=[], help="drop a recorded worktree that is gone")
    p.add_argument("--run", action="append", default=[], help="worker run dir to record")
    p.set_defaults(fn=cmd_dehydrate)
    p = sub.add_parser("rehydrate")
    p.add_argument("slug")
    p.set_defaults(fn=cmd_rehydrate)
    p = sub.add_parser("status")
    p.set_defaults(fn=cmd_status)
    args = parser.parse_args(argv)
    try:
        return args.fn(args)
    except WorkerError as error:
        print(f"error: {error}", file=sys.stderr)
        return 64


if __name__ == "__main__":
    sys.exit(main())

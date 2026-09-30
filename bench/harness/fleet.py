"""Multi-ticket runs: one git worktree per ticket, agent inside a capsule, lead lands by rebase + fast-forward.

A multi task has tickets/<id>/{ticket.toml, instruction.md, overlay/}. ticket.toml:
    allow = ["a.py"]        paths the ticket may change (anything else is contamination)
    depends = []            ticket ids that should land first (advisory; landing order is the run's choice)
The overlay is the reference change for that ticket, used by the reference agent.

An agent is a callable agent(ticket, workspace, record) -> dict run against a ticket worktree. It executes
task code, so it must run inside the capsule (see run_in_capsule); the fleet never runs it on the host.
"""

from __future__ import annotations

import itertools
import shutil
import threading
from concurrent.futures import ThreadPoolExecutor
import subprocess
import tempfile
import time
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

import tasklib

GIT_ENV = ["-c", "user.email=bench@corenius", "-c", "user.name=bench", "-c", "commit.gpgsign=false"]
IGNORED = (".uvcache/", "target/", "build/", "__pycache__/", ".pytest_cache/", ".blade_tools/", "build64_release/")


@dataclass
class Ticket:
    id: str
    path: Path
    allow: list[str]
    depends: list[str] = field(default_factory=list)

    @property
    def instruction(self) -> str:
        return (self.path / "instruction.md").read_text()


def load_tickets(task: tasklib.Task) -> list[Ticket]:
    root = task.path / "tickets"
    tickets = []
    for path in sorted(p for p in root.iterdir() if (p / "ticket.toml").is_file()):
        cfg = tomllib.loads((path / "ticket.toml").read_text())
        tickets.append(Ticket(path.name, path, list(cfg["allow"]), list(cfg.get("depends", []))))
    return tickets


def git(cwd, *args: str, check: bool = True) -> subprocess.CompletedProcess:
    proc = subprocess.run(["git", *GIT_ENV, "-C", str(cwd), *args], capture_output=True, text=True)
    if check and proc.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed in {cwd}: {proc.stderr.strip() or proc.stdout.strip()}")
    return proc


def allowed(path: str, allow: list[str]) -> bool:
    return any(path == a.rstrip("/") or path.startswith(a.rstrip("/") + "/") for a in allow)


def init_repo(task: tasklib.Task, repo: Path) -> str:
    tasklib.materialize(task, repo)
    (repo / ".gitignore").write_text("\n".join(IGNORED) + "\n")
    git(repo, "init", "-q", "-b", "main")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "start")
    return git(repo, "rev-parse", "HEAD").stdout.strip()


def changed_paths(ws: Path) -> list[str]:
    out = git(ws, "status", "--porcelain", "--untracked-files=all").stdout
    paths = []
    for line in out.splitlines():
        path = line[3:].split(" -> ")[-1].strip('"')
        paths.append(path)
    return paths


def reference_agent(ticket: Ticket, ws: Path, record: dict) -> dict:
    """Applies the ticket's apply.sh, or copies its overlay, inside the capsule (what a perfect agent would do)."""
    command = ("sh /tmp/ticket/apply.sh" if (ticket.path / "apply.sh").exists()
               else "cp -r /tmp/ticket/overlay/. /workspace/")
    return tasklib.run_in_capsule(record, ws, command, {}, "none", 120.0, ro=[(str(ticket.path), "/tmp/ticket")])


GIT_LOCK = threading.Lock()


def run_ticket(repo: Path, ticket: Ticket, base: str, agent, record: dict, tag: str = "") -> dict:
    wt = repo.parent / f"wt-{ticket.id}{tag}"
    branch = f"ticket/{ticket.id}{tag}"
    with GIT_LOCK:
        git(repo, "worktree", "add", "-q", "-b", branch, str(wt), base)
    started = time.time()
    result = agent(ticket, wt, record)  # agents for different tickets may run concurrently
    seconds = round(time.time() - started, 3)
    with GIT_LOCK:
        return finish_ticket(ticket, wt, branch, result, seconds)


def finish_ticket(ticket: Ticket, wt: Path, branch: str, result: dict, seconds: float) -> dict:
    changed = changed_paths(wt)
    contamination = sorted(p for p in changed if not allowed(p, ticket.allow))
    inside = [p for p in changed if allowed(p, ticket.allow)]
    for p in inside:
        git(wt, "add", "--", p)
    if inside:
        git(wt, "commit", "-q", "-m", f"ticket {ticket.id}")
    # Contamination is reported above and then discarded: only allowed paths ever land.
    git(wt, "checkout", "-q", "--", ".")
    git(wt, "clean", "-fdq")
    return {"ticket": ticket.id, "branch": branch, "worktree": str(wt), "seconds": seconds, "agent_rc": result.get("rc"),
            "changed": sorted(changed), "contamination": contamination, "empty": not inside}


def land(repo: Path, run: dict) -> dict:
    """Rebase the ticket branch onto main, then fast-forward main. A conflict aborts the rebase and is reported."""
    wt = run["worktree"]
    rebase = git(wt, "rebase", "main", check=False)
    if rebase.returncode != 0:
        files = git(wt, "diff", "--name-only", "--diff-filter=U", check=False).stdout.split()
        git(wt, "rebase", "--abort", check=False)
        if not files:
            return {"landed": False, "conflict": False, "files": [], "error": rebase.stderr.strip()[:200]}
        return {"landed": False, "conflict": True, "files": sorted(files)}
    ff = git(repo, "merge", "--ff-only", run["branch"], check=False)
    return {"landed": ff.returncode == 0, "conflict": False, "files": [],
            **({} if ff.returncode == 0 else {"error": ff.stderr.strip()[:200]})}


def verify_repo(task: tasklib.Task, repo: Path, record: dict) -> dict:
    with tempfile.TemporaryDirectory(prefix="fleet-verify-", dir=str(tasklib.ROOT / ".scratch")) as tmp:
        ws = Path(tmp) / "ws"
        shutil.copytree(repo, ws, ignore=shutil.ignore_patterns(".git"))
        return tasklib.verify(task, ws, record)


def run_fleet(task: tasklib.Task, agent, record: dict, order: list[str] | None = None, redo=True,
              include: list[str] | None = None, workers: int = 1) -> dict:
    """All tickets start from the same base (parallel workers), then land one by one in `order`."""
    tickets = {t.id: t for t in load_tickets(task) if include is None or t.id in include}
    order = [t for t in (order or list(tickets)) if t in tickets]
    started = time.time()
    with tempfile.TemporaryDirectory(prefix=f"fleet-{task.id}-", dir=str(tasklib.ROOT / ".scratch")) as tmp:
        repo = Path(tmp) / "repo"
        base = init_repo(task, repo)
        with ThreadPoolExecutor(max_workers=max(1, workers)) as pool:
            futures = {tid: pool.submit(run_ticket, repo, tickets[tid], base, agent, record) for tid in tickets}
            runs = {tid: f.result() for tid, f in futures.items()}
        landings, conflicts, redos = [], 0, 0
        for tid in order:
            outcome = land(repo, runs[tid])
            if outcome["conflict"]:
                conflicts += 1
                if redo:
                    redos += 1
                    head = git(repo, "rev-parse", "main").stdout.strip()
                    rerun = run_ticket(repo, tickets[tid], head, agent, record, tag="-redo")
                    runs[tid + "-redo"] = rerun
                    outcome = {**land(repo, rerun), "first_conflict_files": outcome["files"]}
                else:
                    outcome["first_conflict_files"] = outcome["files"]
            landings.append({"ticket": tid, **outcome})
        verdict = verify_repo(task, repo, record)
        contamination = sorted({p for r in runs.values() for p in r["contamination"]})
        return {
            "task": task.id, "order": order, "landings": landings, "conflicts": conflicts, "redos": redos,
            "contamination": contamination, "passed": verdict["passed"], "verify_seconds": verdict["seconds"],
            "wall_seconds": round(time.time() - started, 3),
            "ticket_seconds": {tid: r["seconds"] for tid, r in runs.items()},
        }


def all_orders(task: tasklib.Task, limit: int = 24) -> list[list[str]]:
    ids = [t.id for t in load_tickets(task)]
    return [list(p) for p in itertools.islice(itertools.permutations(ids), limit)]

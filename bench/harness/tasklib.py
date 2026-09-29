"""Benchmark tasks: load, materialize, apply the reference solution, verify inside a capsule.

Layout (Harbor-compatible authoring shape): task.toml, instruction.md, environment/ (the starting
workspace, visible to the agent), tests/ (verifier-owned, restored from here before every
verification), solution/ (solve.sh plus overlay/, never mounted for the agent).
"""

from __future__ import annotations

import shutil
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "insula"))
import capsule  # noqa: E402

TASKS = ROOT / "bench" / "tasks"
REQUESTS = ROOT / "bench" / "insula" / "requests"


class Task:
    def __init__(self, path: Path):
        self.path = path
        self.id = path.name
        self.cfg = tomllib.loads((path / "task.toml").read_text())
        self.bench = self.cfg.get("bench", {})
        self.verifier = self.cfg["verifier"]
        self.protected = self.verifier.get("protected", ["tests"])

    @property
    def tags(self) -> list[str]:
        return self.cfg.get("metadata", {}).get("tags", [])


def load_tasks(root: Path = TASKS, only: list[str] | None = None) -> list[Task]:
    tasks = [Task(p) for p in sorted(root.iterdir()) if (p / "task.toml").is_file()]
    if only:
        unknown = set(only) - {t.id for t in tasks}
        if unknown:
            raise SystemExit(f"unknown task(s): {', '.join(sorted(unknown))}")
        tasks = [t for t in tasks if t.id in only]
    return tasks


def capsule_for(task: Task) -> dict:
    return capsule.build(str(REQUESTS / f"{task.bench.get('capsule', 'toolchains')}.json"))


def materialize(task: Task, ws: Path) -> None:
    shutil.copytree(task.path / "environment", ws, symlinks=True, dirs_exist_ok=True)


def restore_protected(task: Task, ws: Path) -> None:
    """Replace verifier-owned paths from the pristine task copy; the agent's edits there are discarded."""
    for rel in task.protected:
        if ".." in Path(rel).parts or Path(rel).is_absolute():
            raise ValueError(f"protected path {rel} escapes the workspace")
        target = ws / rel
        if target.is_symlink() or target.is_file():
            target.unlink()
        elif target.is_dir():
            shutil.rmtree(target)
        source = task.path / rel
        if source.is_dir():
            shutil.copytree(source, target, symlinks=False)
            for path in target.rglob("*"):
                if path.is_symlink():
                    raise ValueError(f"symlink in pristine {rel}: {path}")
        elif source.is_file():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)


def run_in_capsule(record: dict, ws: Path, command: str, env: dict, net: str, timeout: float,
                   ro: list[tuple[str, str]] | None = None, gpu: bool = False) -> dict:
    with tempfile.TemporaryFile() as out:
        started = capsule.time.time()
        code = capsule.run(record, ["/bin/sh", "-c", command], str(ws), [], env, net, gpu, False, timeout,
                           "/workspace", None, ro, stdout=out, stderr=capsule.subprocess.STDOUT)
        elapsed = capsule.time.time() - started
        out.seek(0)
        text = out.read().decode("utf-8", "replace")
    return {"rc": code, "timed_out": code == 124, "seconds": round(elapsed, 3), "tail": "\n".join(text.splitlines()[-25:])}


def verify(task: Task, ws: Path, record: dict) -> dict:
    restore_protected(task, ws)
    result = run_in_capsule(record, ws, task.verifier["command"], dict(task.verifier.get("env", {})),
                            task.bench.get("net", "none"), float(task.verifier.get("timeout_sec", 300)),
                            gpu=bool(task.bench.get("gpu", False)))
    result["passed"] = result["rc"] == 0
    return result


def apply_solution(task: Task, ws: Path, record: dict) -> dict:
    solution = task.path / "solution"
    return run_in_capsule(record, ws, "sh /tmp/solution/solve.sh", {}, "none", 120.0,
                          ro=[(str(solution), "/tmp/solution")])

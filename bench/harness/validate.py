#!/usr/bin/env python3
"""Offline validation of the benchmark itself: tasks and verifiers are sound before any agent runs.

For every task: the starting state fails the verifier; the reference solution passes; cheats
(deleted tests, stubbed tests) do not turn a failing state into a pass; the passing verdict is
deterministic; the capsule probe is clean. Every step runs inside the task's capsule.
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasklib  # noqa: E402


def cheat_paths(task: tasklib.Task) -> list[str]:
    """Where an agent would tamper: the declared protected paths plus the conventional tests/ directory."""
    return sorted(set(task.protected) | {"tests"})


def stub_out(ws: Path, task: tasklib.Task) -> None:
    for rel in cheat_paths(task):
        for path in (ws / rel).rglob("*") if (ws / rel).is_dir() else [ws / rel]:
            if path.is_file():
                path.write_text("#!/bin/sh\nexit 0\n" if path.suffix in ("", ".sh") else "")


def check_task(task: tasklib.Task, record: dict) -> dict:
    checks: dict[str, dict] = {}
    with tempfile.TemporaryDirectory(prefix=f"bench-{task.id}-", dir=str(tasklib.ROOT / ".scratch")) as tmp:
        base = Path(tmp)

        def fresh(name: str) -> Path:
            ws = base / name
            tasklib.materialize(task, ws)
            return ws

        start = tasklib.verify(task, fresh("start"), record)
        checks["start-fails"] = {"ok": not start["passed"] and not start["timed_out"], "rc": start["rc"], "seconds": start["seconds"]}

        ws = fresh("solved")
        applied = tasklib.apply_solution(task, ws, record)
        solved = tasklib.verify(task, ws, record)
        checks["solution-applies"] = {"ok": applied["rc"] == 0, "rc": applied["rc"]}
        checks["solution-passes"] = {"ok": solved["passed"], "rc": solved["rc"], "seconds": solved["seconds"],
                                     "tail": "" if solved["passed"] else solved["tail"]}
        again = tasklib.verify(task, ws, record)
        checks["deterministic"] = {"ok": again["passed"] == solved["passed"] and again["rc"] == solved["rc"]}

        deleted = fresh("cheat-delete")
        for rel in cheat_paths(task):
            target = deleted / rel
            shutil.rmtree(target) if target.is_dir() else target.unlink(missing_ok=True)
        r = tasklib.verify(task, deleted, record)
        checks["cheat-delete-tests"] = {"ok": not r["passed"]}

        stubbed = fresh("cheat-stub")
        stub_out(stubbed, task)
        r = tasklib.verify(task, stubbed, record)
        checks["cheat-stub-tests"] = {"ok": not r["passed"]}

        empty = fresh("empty")
        checks["empty-patch-fails"] = {"ok": not tasklib.verify(task, empty, record)["passed"]}
    return checks


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--task", action="append", default=[])
    p.add_argument("--json", help="write the full report here")
    args = p.parse_args(argv)
    (tasklib.ROOT / ".scratch").mkdir(exist_ok=True)
    tasks = tasklib.load_tasks(only=args.task)
    report, failed = {}, 0
    records: dict[str, dict] = {}
    started = time.time()
    for task in tasks:
        name = task.bench.get("capsule", "toolchains")
        if name not in records:
            records[name] = tasklib.capsule_for(task)
            probe = tasklib.capsule.probe(records[name])
            report.setdefault("_capsules", {})[name] = {"digest": records[name]["capsule_digest"], "probe": probe}
            if not probe["ok"]:
                print(f"capsule {name}: probe failed: {probe}")
                failed += 1
        checks = check_task(task, records[name])
        report[task.id] = checks
        bad = [k for k, v in checks.items() if not v["ok"]]
        failed += bool(bad)
        print(f"{'FAIL' if bad else 'ok  '} {task.id:28s} " + (", ".join(bad) if bad else f"{len(checks)} checks"))
        for k in bad:
            tail = checks[k].get("tail")
            if tail:
                print("    " + tail.replace("\n", "\n    "))
    print(f"{len(tasks)} task(s), {failed} failing, {time.time() - started:.1f}s")
    if args.json:
        Path(args.json).write_text(json.dumps(report, indent=2) + "\n")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

"""State files: the only interface between the scheduler, the panes and the dashboard.

Layout under a run root:
  runs/<run_id>/state.json     one benchmark run (a solo run or a fleet run): written by the scheduler
  units/<unit_id>/state.json   one agent pane: written by the pane process
  units/<unit_id>/{job.json,events.jsonl,pane.log,result.json}
Writes are atomic (temp file + rename), so a reader never sees a half-written file.
"""

from __future__ import annotations

import json
import os
import tempfile
import time
from pathlib import Path

RUN_PHASES = ("queued", "agents", "landing", "verify", "done", "crashed")
UNIT_PHASES = ("queued", "agent", "done", "crashed")
FINAL = ("done", "crashed")


def now() -> float:
    return time.time()


def write_atomic(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".state-", dir=str(path.parent))
    try:
        with os.fdopen(fd, "w") as handle:
            json.dump(data, handle, indent=1, sort_keys=True)
            handle.write("\n")
        os.replace(tmp, path)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def read(path: Path) -> dict | None:
    try:
        return json.loads(Path(path).read_text())
    except (OSError, ValueError):
        return None


def update(path: Path, **fields) -> dict:
    """Merge fields into the state file (creating it) and stamp `updated`."""
    data = read(path) or {}
    data.update(fields)
    data["updated"] = now()
    write_atomic(path, data)
    return data


def run_state_path(root: Path, run_id: str) -> Path:
    return Path(root) / "runs" / run_id / "state.json"


def unit_dir(root: Path, unit: str) -> Path:
    return Path(root) / "units" / unit


def unit_state_path(root: Path, unit: str) -> Path:
    return unit_dir(root, unit) / "state.json"


def collect(root: Path) -> dict:
    """Snapshot for the dashboard: {'runs': [...], 'units': [...]}, each a state dict, sorted by start."""
    root = Path(root)
    runs = [s for p in sorted((root / "runs").glob("*/state.json")) if (s := read(p))]
    units = [s for p in sorted((root / "units").glob("*/state.json")) if (s := read(p))]
    runs.sort(key=lambda r: (r.get("created", 0), r.get("run_id", "")))
    units.sort(key=lambda u: (u.get("created", 0), u.get("unit", "")))
    return {"runs": runs, "units": units, "taken": now()}

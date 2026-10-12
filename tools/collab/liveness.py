"""Worker process liveness checks for collab Attempts."""
from __future__ import annotations

import json
from pathlib import Path

from common import Refused, output_path


def _stat(pid: int) -> dict | None:
    proc = Path("/proc") / str(pid) / "stat"
    try:
        raw = proc.read_text()
    except FileNotFoundError:
        return None
    except PermissionError:
        return {"state": "?", "session_id": None, "start_ticks": None, "permission_denied": True}
    end = raw.rfind(")")
    if end == -1:
        return None
    fields = raw[end + 2 :].split()
    if len(fields) < 20:
        return None
    try:
        session_id = int(fields[3])
    except ValueError:
        session_id = None
    return {
        "state": fields[0],
        "session_id": session_id,
        "start_ticks": fields[19],
    }


def _int_or_none(value: object) -> int | None:
    if value is None:
        return None
    try:
        parsed = int(value)
    except (TypeError, ValueError) as exc:
        raise Refused(f"Invalid worker process id: {value!r}") from exc
    if parsed <= 0:
        raise Refused("--worker-pid must be positive")
    return parsed


def normalize_worker_binding(worker_pid: int | str | None, worker_run_dir: str | None) -> dict:
    pid = _int_or_none(worker_pid)
    run_dir = None
    if worker_run_dir is not None:
        run_dir = output_path(worker_run_dir)
        if not run_dir.is_dir():
            raise Refused("Worker run directory does not exist")
    return {
        "worker_pid": pid,
        "worker_pid_start_ticks": proc_start_ticks(pid) if pid is not None else None,
        "worker_run_dir": str(run_dir) if run_dir is not None else None,
    }


def proc_start_ticks(pid: int) -> str | None:
    stat = _stat(pid)
    return None if stat is None else stat["start_ticks"]


def pid_alive(pid: int, start_ticks: str | None = None) -> bool:
    stat = _stat(pid)
    if stat is None:
        return False
    if stat["state"] == "Z":
        return False
    if stat.get("permission_denied"):
        return True
    if start_ticks is not None and stat["start_ticks"] != start_ticks:
        return False
    return True


def live_session_pids(session_id: int | None) -> list[int]:
    if session_id is None:
        return []
    proc = Path("/proc")
    if not proc.is_dir():
        return []
    pids: list[int] = []
    for child in proc.iterdir():
        if not child.name.isdigit():
            continue
        try:
            raw = (child / "stat").read_text()
        except OSError:
            continue
        end = raw.rfind(")")
        if end == -1:
            continue
        fields = raw[end + 2 :].split()
        if len(fields) < 4:
            continue
        try:
            state = fields[0]
            sid = int(fields[3])
        except (ValueError, IndexError):
            continue
        if sid == session_id and state != "Z":
            pids.append(int(child.name))
    return sorted(pids)


def _run_dir_liveness(run_dir: Path) -> dict:
    source = {"source": "worker_run_dir", "worker_run_dir": str(run_dir), "alive": False}
    try:
        run = json.loads((run_dir / "run.json").read_text())
    except (OSError, json.JSONDecodeError) as exc:
        return {**source, "state": "unknown", "reason": str(exc)}
    session_id = _optional_int(run.get("process_session_id"))
    pids = live_session_pids(session_id)
    if pids:
        return {**source, "alive": True, "state": "running", "process_session_id": session_id, "live_pids": pids}
    exit_file = run_dir / "exit_code"
    if exit_file.exists() and exit_file.read_text().strip():
        return {**source, "state": "exited", "exit_code": exit_file.read_text().strip()}
    pid = _optional_int(run.get("pid"))
    alive = pid_alive(pid) if pid is not None else False
    return {**source, "alive": alive, "state": "running" if alive else "stopped", "pid": pid}


def _optional_int(value: object) -> int | None:
    try:
        return int(value) if value is not None else None
    except (TypeError, ValueError):
        return None


def worker_liveness(attempt: dict) -> dict:
    sources = []
    run_dir = attempt.get("worker_run_dir")
    if run_dir:
        sources.append(_run_dir_liveness(Path(run_dir)))
    pid = attempt.get("worker_pid")
    if pid is not None:
        pid = int(pid)
        alive = pid_alive(pid, attempt.get("worker_pid_start_ticks"))
        sources.append({"source": "worker_pid", "pid": pid, "alive": alive, "state": "running" if alive else "stopped"})
    live = [source for source in sources if source["alive"]]
    return {"alive": bool(live), "sources": sources}


def liveness_blockers(attempt: dict) -> list[dict]:
    status = worker_liveness(attempt)
    blockers = []
    for source in status["sources"]:
        if not source["alive"] and source.get("state") != "unknown":
            continue
        blocker = {"kind": "worker_liveness", **source}
        blockers.append(blocker)
    return blockers


def require_worker_stopped(attempt: dict) -> None:
    blockers = liveness_blockers(attempt)
    if blockers:
        raise Refused(f"recorded worker running or unknown for Attempt {attempt['id']}: {blockers[0]}")

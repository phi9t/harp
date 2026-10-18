"""Worker process liveness checks for collab Attempts."""
from __future__ import annotations

import json
from pathlib import Path
import re

from common import Refused, output_path

DECISION_RUNNING = "running"
DECISION_STOPPED = "stopped"
DECISION_UNKNOWN = "unknown"
DECISION_UNBOUND = "unbound"


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
    if isinstance(value, bool) or not isinstance(value, (int, str)):
        raise Refused(f"Invalid worker process id: {value!r}")
    if isinstance(value, str) and not re.fullmatch(r"[1-9][0-9]*", value):
        raise Refused(f"Invalid worker process id: {value!r}")
    parsed = int(value)
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


def _source(source: str, decision: str, **fields) -> dict:
    return {
        "source": source,
        "decision": decision,
        "state": decision,
        "alive": decision == DECISION_RUNNING,
        **fields,
    }


def _strict_positive_int(value: object, field: str) -> tuple[int | None, str | None]:
    if value is None:
        return None, None
    if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
        return None, f"{field} must be a positive integer"
    return value, None


def _completion(run_dir: Path) -> tuple[int | None, str | None]:
    exit_file = run_dir / "exit_code"
    if not exit_file.exists():
        return None, None
    try:
        text = exit_file.read_text().strip()
    except OSError as exc:
        return None, f"completion unreadable: {exc}"
    if not re.fullmatch(r"\d+", text):
        return None, "completion exit_code must be a non-negative integer"
    return int(text), None


def _run_dir_liveness(run_dir: Path) -> dict:
    source = {"worker_run_dir": str(run_dir)}
    try:
        run = json.loads((run_dir / "run.json").read_text())
    except (OSError, json.JSONDecodeError) as exc:
        return _source("worker_run_dir", DECISION_UNKNOWN, **source, reason=f"run.json unusable: {exc}")
    if not isinstance(run, dict):
        return _source("worker_run_dir", DECISION_UNKNOWN, **source, reason="run.json must be a JSON object")
    pid, pid_error = _strict_positive_int(run.get("pid"), "pid")
    session_id, session_error = _strict_positive_int(run.get("process_session_id"), "process_session_id")
    if pid_error or session_error:
        return _source(
            "worker_run_dir",
            DECISION_UNKNOWN,
            **source,
            pid=run.get("pid"),
            process_session_id=run.get("process_session_id"),
            reason=pid_error or session_error,
        )
    if pid is None and session_id is None:
        return _source("worker_run_dir", DECISION_UNKNOWN, **source, reason="run.json has no usable pid or session")
    if session_id is not None:
        pids = live_session_pids(session_id)
        if pids:
            return _source(
                "worker_run_dir",
                DECISION_RUNNING,
                **source,
                process_session_id=session_id,
                live_pids=pids,
            )
    if pid is not None and pid_alive(pid):
        return _source("worker_run_dir", DECISION_RUNNING, **source, pid=pid, process_session_id=session_id)
    exit_code, completion_error = _completion(run_dir)
    if completion_error:
        return _source(
            "worker_run_dir",
            DECISION_UNKNOWN,
            **source,
            pid=pid,
            process_session_id=session_id,
            reason=completion_error,
        )
    if exit_code is not None:
        return _source(
            "worker_run_dir",
            DECISION_STOPPED,
            **source,
            pid=pid,
            process_session_id=session_id,
            exit_code=exit_code,
            reason="valid completion evidence",
        )
    return _source(
        "worker_run_dir",
        DECISION_STOPPED,
        **source,
        pid=pid,
        process_session_id=session_id,
        reason="no live process observed for usable identity",
    )


def _pid_liveness(pid: object, start_ticks: str | None) -> dict:
    parsed, error = _strict_positive_int(pid, "worker_pid")
    if error:
        return _source("worker_pid", DECISION_UNKNOWN, pid=pid, reason=error)
    alive = pid_alive(parsed, start_ticks)
    return _source(
        "worker_pid",
        DECISION_RUNNING if alive else DECISION_STOPPED,
        pid=parsed,
        reason="process is live" if alive else "process is not live",
    )


def worker_liveness(attempt: dict) -> dict:
    sources = []
    run_dir = attempt.get("worker_run_dir")
    if run_dir:
        sources.append(_run_dir_liveness(Path(run_dir)))
    pid = attempt.get("worker_pid")
    if pid is not None:
        sources.append(_pid_liveness(pid, attempt.get("worker_pid_start_ticks")))
    if not sources:
        return {
            "decision": DECISION_UNBOUND,
            "state": DECISION_UNBOUND,
            "alive": False,
            "reason": "no worker binding",
            "sources": [],
        }
    if any(source["decision"] == DECISION_RUNNING for source in sources):
        decision = DECISION_RUNNING
    elif any(source["decision"] == DECISION_UNKNOWN for source in sources):
        decision = DECISION_UNKNOWN
    else:
        decision = DECISION_STOPPED
    return {"decision": decision, "state": decision, "alive": decision == DECISION_RUNNING, "sources": sources}


def liveness_blockers(attempt: dict, status: dict | None = None) -> list[dict]:
    status = status or worker_liveness(attempt)
    blockers = []
    for source in status["sources"]:
        if source["decision"] not in (DECISION_RUNNING, DECISION_UNKNOWN):
            continue
        blocker = {"kind": "worker_liveness", **source}
        blockers.append(blocker)
    return blockers


def require_worker_stopped(attempt: dict) -> None:
    status = worker_liveness(attempt)
    blockers = liveness_blockers(attempt, status)
    if blockers:
        state = "still running" if status["decision"] == DECISION_RUNNING else "state unknown"
        raise Refused(f"recorded worker {state} for Attempt {attempt['id']}: {blockers[0]}")

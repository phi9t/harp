"""Project-scoped foreground runtime. No installed service or PID-based control."""
from __future__ import annotations

import argparse
import ctypes
import fcntl
import json
import math
import os
from pathlib import Path
import signal
import sqlite3
import sys
import time
import uuid

from controller.durable.common import atomic_json, read_json, temporary_environment
from controller.durable.engine import Runtime, Store
from controller.durable.insula import InsulaExecution
from controller.durable.workflows import DEMO_CONTRACT, REGISTRY


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--format", choices=("json", "text"), default="json")
    p.add_argument("--state", type=Path, required=True)
    p.add_argument("--execution", type=Path)
    sub = p.add_subparsers(dest="command", required=True)
    submit = sub.add_parser("submit", help="submit a trusted deterministic demonstration")
    submit.add_argument("run_id")
    submit.add_argument("--value", type=int, default=3)
    submit.add_argument("--sleep", type=float, default=0)
    start = sub.add_parser("start", help="run in the foreground, then drain on stop")
    start.add_argument("--capacity", type=int, choices=range(1, 5), default=1)
    start.add_argument("--timeout", type=float, default=3600)
    start.add_argument("--until-idle", action="store_true")
    status = sub.add_parser("status")
    status.add_argument("run_id", nargs="?")
    sub.add_parser("stop", help="request a specific current instance to stop and clean up")
    sub.add_parser("events")
    sub.add_parser("check")
    cancel = sub.add_parser("cancel")
    cancel.add_argument("run_id")
    return p


def instance(root):
    """The inherited runtime lock also covers guardians after controller death."""
    path = root / "instance.json"
    record = read_json(path) if path.exists() else None
    fd = os.open(root / "controller.lock", os.O_RDWR | os.O_CREAT, 0o600)
    try:
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return {"locked": True, "instance": record}
        if record is not None:
            record["status"] = "stopped"
        return {"locked": False, "instance": record}
    finally:
        os.close(fd)


def start(store, args, execution, contract):
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        raise ValueError("timeout must be finite and positive")
    # Validate host temporary storage before any guardian can be launched.
    temporary_environment()
    stopped = False

    def stop_signal(_number, _frame):
        nonlocal stopped
        stopped = True

    old = {s: signal.signal(s, stop_signal) for s in (signal.SIGINT, signal.SIGTERM)}
    token = uuid.uuid4().hex
    record = {"token": token, "pid": os.getpid(), "state": str(store.root), "status": "running"}
    try:
        with Runtime(store, REGISTRY, contract, max_active=args.capacity,
                     cpu_slots=args.capacity, lock_timeout=0, execution=execution) as runtime:
            atomic_json(store.root / "instance.json", record)
            deadline = time.monotonic() + args.timeout
            try:
                while not stopped:
                    stop_path = store.root / ("stop-" + token + ".json")
                    if stop_path.exists() and read_json(stop_path) == {"token": token}:
                        break
                    runtime.tick()
                    running = store.db.execute("SELECT count(*) FROM runs WHERE status='RUNNING'").fetchone()[0]
                    if args.until_idle and not runtime.live and not running:
                        break
                    if time.monotonic() >= deadline:
                        raise TimeoutError("runtime deadline reached; stopping active guardians")
                    time.sleep(0.02)
            finally:
                # Runtime.__exit__ proves guardian cleanup before releasing its lock.
                record["status"] = "stopping"
                atomic_json(store.root / "instance.json", record)
        record["status"] = "stopped"
        # Ownership has ended. A replacement may already have published its
        # token, so never write shared metadata after releasing the lock.
    finally:
        for sig, previous in old.items():
            signal.signal(sig, previous)
    return {"instance": record, "runs": runs(store)}


def runs(store):
    return [{"id": row[0], "status": row[1]} for row in store.db.execute("SELECT id,status FROM runs ORDER BY created,id")]


def run_status(store, run_id):
    data = store.status(run_id)
    data["phase"] = data["status"].lower()
    if data["status"] == "RUNNING" and not data["attempts"]:
        data["phase"] = "pending"
    for attempt in data["attempts"]:
        attempt["logs"] = {stream: str(Path(attempt["directory"]) / (stream + ".log"))
                           for stream in ("stdout", "stderr")}
    return data


def local_state(path):
    """Admit owned storage on a known local Linux filesystem before opening SQLite."""
    for parent in (path, *path.parents):
        if parent.is_symlink():
            raise ValueError("state path must not contain symlinks")
    path.mkdir(parents=True, mode=0o700, exist_ok=True)
    info = path.stat()
    if info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise ValueError("state must be owned by this user and private (mode 0700)")
    for name in ("runtime.sqlite", "runtime.sqlite-wal", "runtime.sqlite-shm",
                 "runtime.sqlite-journal", "controller.lock", "attempts", "instance.json"):
        if (path / name).is_symlink():
            raise ValueError("state targets must not be symlinks: " + name)
    libc = ctypes.CDLL(None, use_errno=True)
    buffer = ctypes.create_string_buffer(256)
    if libc.statfs(os.fsencode(path), buffer) != 0:
        raise OSError(ctypes.get_errno(), "cannot identify state filesystem")
    magic = ctypes.c_long.from_buffer(buffer).value & 0xffffffff
    if magic not in {0xef53, 0x58465342, 0x9123683e, 0x01021994, 0x794c7630, 0x2fc12fc1}:
        raise ValueError(f"unsupported state filesystem type: {magic:#x}; use local ext4/xfs/btrfs/tmpfs/overlay/zfs storage")


def execute(args):
    if sys.platform != "linux":
        raise RuntimeError("durable execution currently requires Linux")
    if not args.state.is_absolute():
        raise ValueError("--state must be an absolute project-owned directory")
    local_state(args.state)
    execution = InsulaExecution(read_json(args.execution)) if args.execution else None
    contract = execution.contract if execution else DEMO_CONTRACT
    if execution:
        execution.verify()
        if execution.config["state"] != str(args.state.resolve()):
            raise ValueError("state disagrees with admitted execution")
    store = Store(args.state)
    try:
        if args.command == "submit":
            if not math.isfinite(args.sleep) or not 0 <= args.sleep <= 3600:
                raise ValueError("sleep must be between zero and 3600 seconds")
            return store.submit(args.run_id, "one", REGISTRY["one"][0],
                                {"input": {"value": args.value, "sleep": args.sleep}}, contract)
        if args.command == "start":
            return start(store, args, execution, contract)
        if args.command == "status":
            return run_status(store, args.run_id) if args.run_id else {**instance(store.root), "runs": runs(store)}
        if args.command == "stop":
            current = instance(store.root)
            record = current["instance"]
            if not current["locked"] or not record or record["status"] != "running":
                return {"stop_requested": False, **current}
            # No process signalling: a stale request cannot stop a replacement instance.
            atomic_json(store.root / ("stop-" + record["token"] + ".json"), {"token": record["token"]})
            return {"stop_requested": True, "token": record["token"]}
        if args.command == "cancel":
            store.cancel(args.run_id)
            return {"cancel_requested": True}
        if args.command == "events":
            return [dict(row) for row in store.db.execute("SELECT * FROM events ORDER BY id")]
        return {"integrity_check": store.db.execute("PRAGMA integrity_check").fetchone()[0],
                "sqlite_version": sqlite3.sqlite_version, "wal_reset_fix_version_verified": store.sqlite_fix_verified,
                "journal_mode": store.db.execute("PRAGMA journal_mode").fetchone()[0],
                "synchronous": store.db.execute("PRAGMA synchronous").fetchone()[0]}
    finally:
        store.close()


def main():
    args = parser().parse_args()
    try:
        result = execute(args)
        print(json.dumps({"schema_version": 1, "command": "durable." + args.command,
                          "status": "ok", "data": result}, sort_keys=True, allow_nan=False))
        return 0
    except Exception as exc:
        print(json.dumps({"schema_version": 1, "command": "durable." + args.command,
                          "status": "error", "error": {"code": type(exc).__name__, "message": str(exc)}}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

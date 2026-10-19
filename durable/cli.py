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

from durable.common import atomic_json, read_json, temporary_environment
from durable.runtime.engine import Runtime, Store
from durable.runtime.execution import InsulaExecution
from durable.fixtures.workflows import DEMO_CONTRACT


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--format", choices=("json", "text"), default="json")
    p.add_argument("--state", type=Path, required=True)
    p.add_argument("--execution", type=Path)
    sub = p.add_subparsers(dest="command", required=True)
    prepare = sub.add_parser("prepare", help="admit a rootfs, frozen runtime code and delegated cgroup")
    prepare.add_argument("--rootfs", type=Path, required=True)
    prepare.add_argument("--cgroup-root", type=Path, required=True)
    prepare.add_argument("--network", choices=("none", "host"), default="none")
    prepare.add_argument("--providers-bin", type=Path)
    prepare.add_argument("--codex-auth", type=Path)
    prepare.add_argument("--trae-auth", type=Path)
    admit = sub.add_parser("admit", help="freeze committed task inputs and submit isolated coding work")
    admit.add_argument("run_id")
    admit.add_argument("--repository", type=Path, required=True)
    admit.add_argument("--task", required=True)
    admit.add_argument("--acceptance", required=True)
    admit.add_argument("--base", default="HEAD")
    admit.add_argument("--task-revision", default="HEAD")
    admit.add_argument("--provider", choices=("deterministic", "codex", "trae"), required=True)
    admit.add_argument("--model")
    admit.add_argument("--fixture-path")
    admit.add_argument("--fixture-content", default="Harp managed worker\n")
    admit.add_argument("--fixture-sleep", type=float, default=0)
    workload = sub.add_parser("workload", help="freeze and admit a committed lab workload")
    workload.add_argument("run_id")
    workload.add_argument("--repository", type=Path, required=True)
    workload.add_argument("--manifest", required=True)
    workload.add_argument("--base", default="HEAD")
    workload.add_argument("--task-revision", default="HEAD")
    submit = sub.add_parser("submit", help="submit a trusted deterministic demonstration")
    submit.add_argument("run_id")
    submit.add_argument("--value", type=int, default=3)
    submit.add_argument("--sleep", type=float, default=0)
    start = sub.add_parser("start", help="run in the foreground, then drain on stop")
    start.add_argument("--capacity", type=int, choices=range(1, 5), default=1)
    start.add_argument("--timeout", type=float, default=3600)
    start.add_argument("--until-idle", action="store_true")
    start.add_argument("--test-crash-at", choices=("after_claim", "before_release", "after_release_started",
                      "after_receipt", "after_commit"), help="explicit disposable crash qualification only")
    status = sub.add_parser("status")
    status.add_argument("run_id", nargs="?")
    sub.add_parser("stop", help="request a specific current instance to stop and clean up")
    sub.add_parser("events")
    sub.add_parser("check")
    cancel = sub.add_parser("cancel")
    cancel.add_argument("run_id")
    ack = sub.add_parser("ack-cleanup", help="record explicit cleanup evidence; launch no work")
    ack.add_argument("attempt_id")
    ack.add_argument("--evidence", required=True)
    resolve = sub.add_parser("resolve", help="reconcile an uncertain effect; launch no work")
    resolve.add_argument("run_id")
    resolve.add_argument("key")
    resolve.add_argument("decision", choices=("retry", "fail", "succeed"))
    resolve.add_argument("--evidence", required=True)
    resolve.add_argument("--result", default="null")
    capture = sub.add_parser("capture", help="retain the exact guardian-accepted coding proposal")
    capture.add_argument("run_id")
    verify = sub.add_parser("verify", help="submit independent exact-candidate checks")
    verify.add_argument("run_id")
    owner = sub.add_parser("owner-init", help="register a private owner capability outside all worker projections")
    owner.add_argument("--capability", type=Path, required=True)
    approve = sub.add_parser("approve", help="record an explicit human decision for one exact verified candidate")
    approve.add_argument("run_id")
    approve.add_argument("--verification", required=True)
    approve.add_argument("--destination", type=Path, required=True)
    approve.add_argument("--capability", type=Path, required=True)
    land = sub.add_parser("land", help="physically land an approved exact candidate, reconciling a lost acknowledgment")
    land.add_argument("approval_id")
    land.add_argument("--capability", type=Path, required=True)
    land.add_argument("--test-crash-at", choices=("after_intent", "after_git"),
                      help="explicit disposable landing crash qualification only")
    estate = sub.add_parser("estate", help="inspect, retain and reclaim registered workspaces")
    commands = estate.add_subparsers(dest="estate_command", required=True)
    commands.add_parser("list")
    commands.add_parser("reconcile")
    for name in ("inspect", "retain", "reclaim"):
        command = commands.add_parser(name)
        command.add_argument("workspace_id", help="registered workspace ID, never an arbitrary path")
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
    from durable.jobs.registry import REGISTRY
    if contract == DEMO_CONTRACT:
        from durable.fixtures.workflows import REGISTRY
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
    def failpoint(point, detail):
        selected = args.test_crash_at
        if selected == "after_release_started" and point == "after_release":
            deadline = time.monotonic() + 10
            marker = store.root / "attempts" / detail["attempt_id"] / "process.json"
            while not marker.exists():
                if time.monotonic() >= deadline:
                    raise TimeoutError("test activity did not start")
                time.sleep(0.01)
        elif point != selected:
            return
        atomic_json(store.root / "test-crash.json", {"point": selected, **detail})
        os._exit(86)
    try:
        with Runtime(store, REGISTRY, contract, max_active=args.capacity,
                     cpu_slots=args.capacity, lock_timeout=0, execution=execution, failpoint=failpoint) as runtime:
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
    if data["status"] == "BLOCKED":
        data["reconciliation"] = {
            "cleanup": [dict(row) for row in store.db.execute(
                "SELECT c.attempt_id,c.reason FROM cleanup_blocks c JOIN attempts a ON a.id=c.attempt_id "
                "JOIN operations o ON o.id=a.operation_id WHERE o.run_id=? AND c.acknowledged IS NULL", (run_id,))],
            "effects": [{"key": op["key"], "reason": op["error"]} for op in data["operations"] if op["status"] == "UNCERTAIN"],
            "required_action": "Prove and acknowledge unproven cleanup, then explicitly resolve each uncertain effect with evidence. Neither decision launches work."}
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
    if args.command == "prepare":
        if args.execution is None:
            raise ValueError("prepare requires --execution for its immutable admission record")
        from durable.runtime.setup import prepare
        credentials = {name: path for name, path in (("codex", args.codex_auth), ("trae", args.trae_auth)) if path}
        return prepare(args.state, args.rootfs, args.cgroup_root, args.execution,
                       network=args.network, providers_bin=args.providers_bin, credentials=credentials)
    execution = None
    if args.execution:
        config = read_json(args.execution)
        os.environ["INSULA_CAPSULES"] = config["capsule_store"]
        execution = InsulaExecution(config)
        execution.verify(require_backend=args.command not in {"status", "events", "check", "stop", "cancel", "ack-cleanup", "resolve", "capture", "owner-init", "approve", "land", "estate"})
        if execution.config["state"] != str(args.state.resolve()):
            raise ValueError("state disagrees with admitted execution")
        code = Path(config["code"])
        if Path(__file__).resolve().parent != (code / "durable").resolve():
            # Execute the admitted orchestrator and registry, rather than checking
            # a frozen snapshot while importing mutable checkout code.
            argv = list(sys.argv[1:])
            for option in ("--execution", "--repository", "--destination", "--capability"):
                for index, value in enumerate(argv):
                    normalize = Path.absolute if option in {"--destination", "--capability"} else Path.resolve
                    if value == option and index + 1 < len(argv):
                        argv[index + 1] = str(normalize(Path(argv[index + 1])))
                    elif value.startswith(option + "="):
                        argv[index] = option + "=" + str(normalize(Path(value.split("=", 1)[1])))
            env = {**os.environ, "PYTHONPATH": str(code), "PYTHONNOUSERSITE": "1",
                   "PYTHONDONTWRITEBYTECODE": "1"}
            os.chdir(code)
            os.execve(sys.executable, [sys.executable, "-B", "-m", "durable", *argv], env)
    contract = execution.contract if execution else DEMO_CONTRACT
    store = Store(args.state)
    try:
        if execution is not None and args.command in {"start", "admit", "verify", "workload"}:
            from durable.authority import execution_roots, protect_owner
            protect_owner(store.root, execution_roots(execution), db=store.db)
        if args.command in {"owner-init", "approve", "land"}:
            if execution is None or execution.backend != "cgroup":
                raise ValueError("owner decisions require an admitted project execution")
            from durable.jobs.coding.landing import owner_init, approve, land
            if args.command == "owner-init":
                return owner_init(store, args.capability, execution=execution)
            if args.command == "approve":
                return approve(store, args.run_id, args.verification, args.destination, args.capability)
            def landing_failpoint(point, detail=None):
                if point == args.test_crash_at:
                    atomic_json(store.root / "test-landing-crash.json", {"point": point, "detail": detail})
                    os._exit(86)
            return land(store, args.approval_id, args.capability, failpoint=landing_failpoint)
        if args.command == "estate":
            from durable.estate.workspaces import Estate, EstateRefused
            estate = Estate(store)
            if args.estate_command == "list":
                return [estate.inspect(handle) for handle in estate.handles()]
            if args.estate_command == "reconcile":
                return estate.reconcile()
            handles = {h["id"]: h for h in estate.handles()}
            if args.workspace_id not in handles:
                raise EstateRefused("unknown workspace ID; paths cannot authorize reclamation")
            return getattr(estate, args.estate_command)(handles[args.workspace_id])
        if args.command == "capture":
            from durable.jobs.coding.candidates import capture
            return capture(store, args.run_id)
        if args.command == "verify":
            if execution is None or execution.backend != "cgroup":
                raise ValueError("verification requires an admitted Insula cgroup execution")
            from durable.jobs.coding.candidates import request_verification
            return request_verification(store, execution, args.run_id)
        if args.command == "workload":
            from durable.jobs.command.admission import submit
            return submit(store, execution, run_id=args.run_id, repository=args.repository,
                          manifest_path=args.manifest, base=args.base, task_revision=args.task_revision)
        if args.command == "admit":
            from durable.jobs.coding.admission import submit
            fixture = ({"path": args.fixture_path, "content": args.fixture_content,
                        "sleep": args.fixture_sleep} if args.fixture_path is not None else None)
            return submit(store, execution, run_id=args.run_id, repository=args.repository,
                          task=args.task, acceptance_path=args.acceptance, base=args.base,
                          task_revision=args.task_revision, provider=args.provider,
                          model=args.model, fixture=fixture)
        if args.command == "submit":
            from durable.fixtures.workflows import REGISTRY
            if execution is not None:
                raise ValueError("demonstrations require disposable state without an execution profile")
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
        if args.command == "ack-cleanup":
            store.acknowledge_cleanup(args.attempt_id, args.evidence)
            return {"cleanup_acknowledged": True, "work_launched": False}
        if args.command == "resolve":
            store.resolve(args.run_id, args.key, args.decision, args.evidence, json.loads(args.result))
            return {"effect_resolved": True, "work_launched": False}
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

"""Local command interface. JSON/argv only; no listening service."""
from __future__ import annotations

import argparse
from collections.abc import Mapping
import importlib
import json
import os
from pathlib import Path
import sys
from typing import Any

from durable.common import atomic_json, read_json
from durable.runtime.engine import Runtime, Store
from durable.execution import InsulaExecution
from durable.runtime.demo_workflows import DEMO_CONTRACT


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--state", type=Path, required=True)
    p.add_argument("--contract", default=DEMO_CONTRACT)
    p.add_argument("--execution", type=Path, help="admitted Insula execution file from insula.durable")
    p.add_argument("--registry", default="durable.runtime.demo_workflows",
                   help="importable module exposing REGISTRY: workflow -> (version, function)")
    p.add_argument("--allow-unpatched-sqlite", action="store_true", help="DISPOSABLE TESTS ONLY: bypass SQLite WAL-reset-fix version gate")
    sub = p.add_subparsers(dest="command", required=True)
    submit = sub.add_parser("submit")
    submit.add_argument("run_id")
    submit.add_argument("--workflow", default="pipeline")
    submit.add_argument("--args", default="{}", help="JSON object, not shell commands")
    run = sub.add_parser("run")
    run.add_argument("--max-active", type=int, default=4)
    run.add_argument("--cpu-slots", type=int, default=4)
    run.add_argument("--gpu-devices", default="", help="comma-separated IDs allocated by Insula")
    run.add_argument("--timeout", type=float, default=3600)
    run.add_argument("--serve", action="store_true")
    # Fault injection is deliberately explicit and must not be enabled in production.
    run.add_argument("--test-crash-at", choices=["after_claim", "after_guard_spawn", "before_release", "after_release", "after_receipt", "after_commit"])
    status = sub.add_parser("status")
    status.add_argument("run_id")
    sig = sub.add_parser("signal")
    sig.add_argument("run_id")
    sig.add_argument("key")
    sig.add_argument("payload")
    cancel = sub.add_parser("cancel")
    cancel.add_argument("run_id")
    ack = sub.add_parser("ack-cleanup")
    ack.add_argument("attempt_id")
    ack.add_argument("--evidence", required=True)
    res = sub.add_parser("resolve")
    res.add_argument("run_id")
    res.add_argument("key")
    res.add_argument("decision", choices=["retry", "fail", "succeed"])
    res.add_argument("--evidence", required=True)
    res.add_argument("--result", default="null")
    sub.add_parser("events")
    sub.add_parser("check")
    args = p.parse_args()
    try:
        execution = InsulaExecution(read_json(args.execution)) if args.execution else None
        if execution is not None:
            execution.verify()
            if execution.contract != args.contract or execution.config["state"] != str(args.state.resolve()):
                raise ValueError("CLI contract/state disagree with admitted execution")
        store = Store(args.state, allow_unpatched_sqlite=args.allow_unpatched_sqlite)
    except Exception as exc:
        print(f"{type(exc).__name__}: {exc}", file=sys.stderr)
        return 1
    try:
        if args.command in {"submit", "run"}:
            registry = importlib.import_module(args.registry).REGISTRY
            if not isinstance(registry, Mapping):
                raise ValueError("REGISTRY must be a mapping")
            for name, definition in registry.items():
                if (not isinstance(name, str) or not name or
                    not isinstance(definition, (tuple, list)) or len(definition) != 2 or
                    not isinstance(definition[0], str) or not definition[0] or
                    not callable(definition[1])):
                    raise ValueError("REGISTRY entries must bind a workflow name to (version, function)")
        if args.command == "submit":
            if args.workflow not in registry:
                raise ValueError(f"workflow unavailable in registry: {args.workflow}")
            result: Any = store.submit(args.run_id, args.workflow, registry[args.workflow][0], json.loads(args.args), args.contract)
        elif args.command == "run":
            def failpoint(point: str, detail: dict[str, Any]) -> None:
                if point == args.test_crash_at:
                    atomic_json(store.root / "test-crash.json", {"point": point, **detail})
                    os._exit(86)
            with Runtime(store, registry, args.contract, max_active=args.max_active,
                         cpu_slots=args.cpu_slots, gpu_devices=tuple(filter(None, args.gpu_devices.split(","))),
                         failpoint=failpoint, execution=execution) as rt:
                result = rt.drive(timeout=args.timeout, serve=args.serve)
        elif args.command == "status":
            result = store.status(args.run_id)
        elif args.command == "signal":
            store.signal(args.run_id, args.key, json.loads(args.payload))
            result = {"recorded": True}
        elif args.command == "cancel":
            store.cancel(args.run_id)
            result = {"cancel_requested": True}
        elif args.command == "ack-cleanup":
            store.acknowledge_cleanup(args.attempt_id, args.evidence)
            result = {"cleanup_acknowledged": True, "work_launched": False}
        elif args.command == "resolve":
            store.resolve(args.run_id, args.key, args.decision, args.evidence, json.loads(args.result))
            result = {"resolved": True, "work_launched": False}
        elif args.command == "events":
            result = [dict(r) for r in store.db.execute("SELECT * FROM events ORDER BY id")]
        else:
            result = {"integrity_check": store.db.execute("PRAGMA integrity_check").fetchone()[0],
                      "sqlite_version": __import__("sqlite3").sqlite_version,
                      "wal_reset_fix_version_verified": store.sqlite_fix_verified,
                      "synchronous": store.db.execute("PRAGMA synchronous").fetchone()[0],
                      "journal_mode": store.db.execute("PRAGMA journal_mode").fetchone()[0]}
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except (Exception, KeyboardInterrupt) as exc:
        print(f"{type(exc).__name__}: {exc}", file=sys.stderr)
        return 1
    finally:
        store.close()


if __name__ == "__main__":
    raise SystemExit(main())

"""Versioned workflow examples: replay-safe orchestration, not work in workflows."""
from __future__ import annotations

from pathlib import Path
import sys
from typing import Any
from .engine import Activity, Context

DEMO_VERSION = "demo-v1"
DEMO_CONTRACT = "local-demo-only-v1"  # NOT an Insula admission/ABI attestation.
ROOT = Path(__file__).resolve().parents[2]


def command(data: Any, **kw: Any) -> Activity:
    return Activity(argv=(sys.executable, "-m", "controller.durable.demo_activity"),
                    input=data, cwd=str(ROOT), **kw)


def pipeline(ctx: Context, args: dict[str, Any]) -> dict[str, Any]:
    prepared = ctx.activity("prepare", command({"value": args.get("value", 3)}, recovery="retry_safe"))
    shards = ctx.parallel([
        ("shard/0", command({"value": prepared["square"], "sleep": 0.05}, recovery="retry_safe")),
        ("shard/1", command({"value": prepared["square"] + 1, "sleep": 0.05}, recovery="retry_safe")),
    ])
    ctx.sleep("cooldown", float(args.get("pause", 0.1)))
    if args.get("require_approval", False):
        approval = ctx.wait_signal("approval")
        if approval.get("approved") is not True:
            return {"approved": False}
    return ctx.activity("aggregate", command({"values": [x["square"] for x in shards]}, recovery="retry_safe"))


def one_activity(ctx: Context, args: dict[str, Any]) -> Any:
    return ctx.activity("work", command(args.get("input", {}),
                        timeout=args.get("timeout", 10),
                        recovery=args.get("recovery", "retry_safe"),
                        max_attempts=args.get("max_attempts", 3),
                        backoff=args.get("backoff", 0.02),
                        cpus=args.get("cpus", 1), gpus=args.get("gpus", 0)))


def batch(ctx: Context, args: dict[str, Any]) -> Any:
    return ctx.parallel([
        (f"job/{i}", command(data, recovery="retry_safe", gpus=args.get("gpus", 0)))
        for i, data in enumerate(args["items"])
    ])


REGISTRY = {
    "pipeline": (DEMO_VERSION, pipeline),
    "one": (DEMO_VERSION, one_activity),
    "batch": (DEMO_VERSION, batch),
}

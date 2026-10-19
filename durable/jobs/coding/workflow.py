"""Replay-only orchestration: coding effects remain managed activities."""
from durable.runtime.engine import Activity
from durable.jobs.coding.admission import WORKFLOW_VERSION


def coding(context, args):
    return context.activity("coding", Activity(
        argv=("/usr/bin/python3", "-m", "durable.jobs.coding.worker"),
        inputs=args["inputs"], cwd="/workspace", workspace=args["workspace"],
        input={"identity": args["identity"], "provider": args["provider"]},
        env={"HOME": "/home/bench/run/home"},
        timeout=1800, recovery="manual", max_attempts=3,
        offline=args["provider"] == "deterministic"))

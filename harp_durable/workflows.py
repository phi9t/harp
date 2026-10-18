"""Replay-only orchestration: coding effects remain managed activities."""
from controller.durable.engine import Activity
from controller.durable.workflows import REGISTRY as DEMONSTRATIONS
from .admission import WORKFLOW_VERSION


def coding(context, args):
    return context.activity("coding", Activity(
        argv=("/usr/bin/python3", "-m", "harp_durable.worker"),
        inputs=args["inputs"], cwd="/workspace",
        input={"identity": args["identity"], "provider": args["provider"]},
        env={"HOME": "/home/bench/run/home"},
        timeout=1800, recovery="manual", max_attempts=1))


REGISTRY = {**DEMONSTRATIONS, "coding": (WORKFLOW_VERSION, coding)}

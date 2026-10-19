"""Replay-only orchestration: coding effects remain managed activities."""
from durable.runtime.engine import Activity
from durable.runtime.demo_workflows import REGISTRY as DEMONSTRATIONS
from durable.agents.admission import WORKFLOW_VERSION
from durable.git.candidates import verification, VERSION as VERIFICATION_VERSION


def coding(context, args):
    return context.activity("coding", Activity(
        argv=("/usr/bin/python3", "-m", "durable.agents.worker"),
        inputs=args["inputs"], cwd="/workspace",
        input={"identity": args["identity"], "provider": args["provider"]},
        env={"HOME": "/home/bench/run/home"},
        timeout=1800, recovery="manual", max_attempts=3,
        offline=args["provider"] == "deterministic"))


REGISTRY = {**DEMONSTRATIONS, "coding": (WORKFLOW_VERSION, coding),
            "verification": (VERIFICATION_VERSION, verification)}

from durable.workflows.workload import workload, WORKFLOW_VERSION as WORKLOAD_VERSION
REGISTRY["workload"] = (WORKLOAD_VERSION, workload)

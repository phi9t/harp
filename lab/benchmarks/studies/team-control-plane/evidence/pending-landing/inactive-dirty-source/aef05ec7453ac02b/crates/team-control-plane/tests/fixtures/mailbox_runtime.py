#!/usr/bin/env python3
"""Owned deterministic physical fixture: every invocation/effect is independent."""
import fcntl
import json
import os
from pathlib import Path
import sys
import time
import uuid

root = Path(sys.argv[1])
mode = sys.argv[2]
request = json.loads(sys.argv[3]) if len(sys.argv) > 3 else {}
root.mkdir(exist_ok=True)

def durable(path, value):
    with path.open("x") as stream:
        json.dump(value, stream)
        stream.flush()
        os.fsync(stream.fileno())
    fd = os.open(root, os.O_RDONLY)
    os.fsync(fd)
    os.close(fd)

if mode == "steer_gated":
    durable(root / "entered", {"pid": os.getpid()})
    deadline = time.monotonic() + 10
    while not (root / "continue").exists():
        if time.monotonic() >= deadline:
            raise RuntimeError("owned fixture gate timed out")
        time.sleep(0.005)

with (root / "runtime.lock").open("a") as gate:
    fcntl.flock(gate.fileno(), fcntl.LOCK_EX)
    state_path = root / "turn.json"
    state = json.loads(state_path.read_text()) if state_path.exists() else {"thread": "owned-thread", "turn": None}
    if mode == "complete":
        state["turn"] = None
    else:
        invocation = str(uuid.uuid4())
        durable(root / (invocation + ".invocation"), {"mode": mode, "request": request})
        if mode.startswith("steer") and state["turn"] != request["expected_turn"]:
            print(json.dumps({"known_absence": "expected turn has completed or changed", "actual_turn": state["turn"]}))
            sys.exit(0)
        effect = str(uuid.uuid4())
        if mode == "start":
            state["turn"] = "turn-" + effect
            effect_id = state["turn"]
        else:
            effect_id = "effect-" + effect
        durable(root / (effect + ".effect"), {"id": effect_id, "operation": request["intent"]["id"], "turn": state["turn"]})
        if mode != "steer_drop_reply":
            print(json.dumps({"effect_id": effect_id, "thread_id": state["thread"]}))
    temporary = root / ("state-" + str(uuid.uuid4()))
    durable(temporary, state)
    os.replace(temporary, state_path)
    fd = os.open(root, os.O_RDONLY)
    os.fsync(fd)
    os.close(fd)

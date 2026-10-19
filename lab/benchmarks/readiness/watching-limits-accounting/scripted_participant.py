#!/usr/bin/env python3
import json
import os
import sys
import time

print(json.dumps({"type": "thread.started"}), flush=True)
print(json.dumps({
    "type": "item.completed",
    "item": {
        "id": "scripted-command",
        "type": "command_execution",
        "command": "printf watching-limits-accounting",
        "status": "completed",
    },
}), flush=True)
mode = os.environ.get("CORENIUS_READINESS_PARTICIPANT", "sleep")
if mode == "stall":
    time.sleep(30)
elif mode == "finish":
    root = os.getcwd()
    with open(os.path.join(root, "work.txt"), "w") as handle:
        handle.write("finished\n")
    print(json.dumps({"type": "turn.completed"}), flush=True)
sys.exit(0)

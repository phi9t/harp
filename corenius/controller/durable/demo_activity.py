"""Executable activity protocol example and fault-injection workload."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import time

from controller.durable.common import atomic_json, publish_result


def main() -> None:
    request = json.load(sys.stdin)
    data = request["input"] or {}
    out = Path(request["attempt_dir"])
    atomic_json(out / "started.json", {"pid": os.getpid(), "attempt_id": request["attempt_id"]})
    # Optional test evidence. It deliberately is NOT deduplicated; it exposes
    # repeated execution when a failure happens before the receipt commit.
    if path := data.get("effect_file"):
        with open(path, "a") as f:
            f.write(json.dumps({"attempt": request["attempt_number"],
                                "key": request["idempotency_key"], "input": data}) + "\n")
            f.flush()
            os.fsync(f.fileno())
    if data.get("child_pid_file"):
        child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(120)"])
        atomic_json(Path(data["child_pid_file"]), {"pid": child.pid})
    time.sleep(float(data.get("sleep", 0)))
    if request["attempt_number"] <= int(data.get("fail_attempts", 0)):
        os._exit(17)
    if data.get("invalid_result"):
        (out / "result.json").write_text("not json")
        return
    if "values" in data:
        result = {"sum": sum(data["values"])}
    else:
        value = data.get("value", 1)
        result = {"square": value * value}
    result["gpu_devices"] = os.environ.get("CUDA_VISIBLE_DEVICES")
    publish_result(request, result)


if __name__ == "__main__":
    main()

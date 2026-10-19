#!/usr/bin/env python3
import argparse
import json
import math
from pathlib import Path

REQUIRED = {
    "schema_version": str,
    "run_id": str,
    "task_id": str,
    "condition": str,
    "capsule": dict,
    "outcome": dict,
    "timing": dict,
    "coordination": dict,
    "cost": dict,
    "checks": list,
}


def require_type(obj, key, typ):
    if key not in obj or not isinstance(obj[key], typ):
        raise ValueError(f"{key} must be {typ.__name__}")


def finite_number(value, name):
    if not isinstance(value, (int, float)) or not math.isfinite(value):
        raise ValueError(f"{name} must be finite number")


def validate(path: Path) -> None:
    data = json.loads(path.read_text())
    for key, typ in REQUIRED.items():
        require_type(data, key, typ)
    if data["schema_version"] != "corenius.bench.result.v0":
        raise ValueError("unsupported schema_version")
    for key in ("digest", "net", "toolchain"):
        require_type(data["capsule"], key, str)
    require_type(data["outcome"], "passed", bool)
    finite_number(data["timing"]["wall_time_sec"], "timing.wall_time_sec")
    for key in ("conflicts", "redo_work_items", "contamination_events"):
        require_type(data["coordination"], key, int)
    for key in ("usd", "input_tokens", "output_tokens"):
        value = data["cost"].get(key)
        if value is not None:
            finite_number(value, f"cost.{key}")
    for i, check in enumerate(data["checks"]):
        require_type(check, "name", str)
        require_type(check, "passed", bool)
    print(f"valid {path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("json_file", type=Path)
    validate(parser.parse_args().json_file)

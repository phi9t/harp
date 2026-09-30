"""Result records: schema, validation, aggregation.

`provenance` says which groups were measured by the harness and which are unmeasured; nothing an agent
reports about itself is stored as a measurement.
"""

from __future__ import annotations

import json
import math
import statistics
from pathlib import Path

SCHEMA = "corenius.bench.result.v1"


def _num(value, name, allow_none=False):
    if value is None and allow_none:
        return
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise ValueError(f"{name} must be a finite number")


def validate(record: dict) -> dict:
    if record.get("schema_version") != SCHEMA:
        raise ValueError("unsupported schema_version")
    for key, typ in (("run_id", str), ("task_id", str), ("condition", str), ("agent", dict), ("capsule", dict),
                     ("outcome", dict), ("timing", dict), ("coordination", dict), ("cost", dict), ("provenance", dict)):
        if not isinstance(record.get(key), typ):
            raise ValueError(f"{key} must be {typ.__name__}")
    if not (record["condition"] == "solo" or record["condition"].startswith("fleet:")):
        raise ValueError("condition must be solo or fleet:N")
    for key in ("digest", "net"):
        if not isinstance(record["capsule"].get(key), str):
            raise ValueError(f"capsule.{key} must be a string")
    out = record["outcome"]
    if not isinstance(out.get("passed"), bool):
        raise ValueError("outcome.passed must be a bool")
    _num(out.get("reward"), "outcome.reward")
    for key in ("wall_seconds", "agent_seconds", "verify_seconds"):
        _num(record["timing"].get(key), f"timing.{key}")
    for key in ("conflicts", "redos", "contamination", "tickets", "tickets_landed"):
        _num(record["coordination"].get(key), f"coordination.{key}")
    for key in ("usd", "input_tokens", "output_tokens"):
        _num(record["cost"].get(key), f"cost.{key}", allow_none=True)
    return record


def load_all(root: Path) -> list[dict]:
    records = []
    for path in sorted(Path(root).rglob("result.json")):
        records.append(validate(json.loads(path.read_text())))
    return records


def wilson(k: int, n: int, z: float = 1.96) -> tuple[float, float]:
    if n == 0:
        return (0.0, 1.0)
    p = k / n
    denom = 1 + z * z / n
    centre = (p + z * z / (2 * n)) / denom
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denom
    return (max(0.0, centre - half), min(1.0, centre + half))


def summarize(records: list[dict]) -> list[dict]:
    groups: dict[tuple, list[dict]] = {}
    for r in records:
        groups.setdefault((r["task_id"], r["condition"], r["agent"]["spec"]), []).append(r)
    rows = []
    for (task, condition, agent), rs in sorted(groups.items()):
        passed = sum(r["outcome"]["passed"] for r in rs)
        lo, hi = wilson(passed, len(rs))
        walls = [r["timing"]["wall_seconds"] for r in rs]
        rows.append({
            "task": task, "condition": condition, "agent": agent, "n": len(rs), "passed": passed,
            "pass_rate": round(passed / len(rs), 3), "ci95": [round(lo, 3), round(hi, 3)],
            "wall_median": round(statistics.median(walls), 2),
            "wall_stdev": round(statistics.pstdev(walls), 2) if len(walls) > 1 else 0.0,
            "tokens_out_median": (statistics.median([r["cost"]["output_tokens"] for r in rs if r["cost"]["output_tokens"] is not None])
                                  if any(r["cost"]["output_tokens"] is not None for r in rs) else None),
            "redos": sum(r["coordination"]["redos"] for r in rs),
            "conflicts": sum(r["coordination"]["conflicts"] for r in rs),
            "contamination": sum(r["coordination"]["contamination"] for r in rs),
            "capsules": sorted({r["capsule"]["digest"][:12] for r in rs}),
        })
    return rows


def table(rows: list[dict]) -> str:
    head = (f"{'task':30s} {'condition':8s} {'agent':16s} {'n':>3s} {'pass':>6s} {'ci95':>13s} {'wall':>7s} {'confl':>5s} "
            f"{'redo':>4s} {'contam':>6s} {'tok_out':>8s}")
    lines = [head, "-" * len(head)]
    for r in rows:
        lines.append(f"{r['task']:30s} {r['condition']:8s} {r['agent'][:16]:16s} {r['n']:3d} {r['pass_rate']:6.2f} "
                     f"[{r['ci95'][0]:.2f},{r['ci95'][1]:.2f}] {r['wall_median']:7.1f} {r['conflicts']:5d} {r['redos']:4d} "
                     f"{r['contamination']:6d} {'-' if r['tokens_out_median'] is None else int(r['tokens_out_median']):>8}")
    return "\n".join(lines)

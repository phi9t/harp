#!/usr/bin/env python3
"""Runner + CLI.  python3 experiments/collab/sim.py --scenario all --seeds 20"""
import argparse
import json
import multiprocessing as mp
import os
import statistics
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import scenarios  # noqa: E402
from strategies import STRATEGIES  # noqa: E402
from world import World  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

MAX_MINUTES = 200_000.0


def run_one(scn, strategy_name, seed, tmpdir=None):
    """One deterministic simulation on a fresh throwaway git repo."""
    w = World(scn, seed, tmpdir)
    try:
        STRATEGIES[strategy_name](w).start()
        w.sim.run(until=MAX_MINUTES)
        return w.metrics(strategy_name)
    finally:
        w.close()


def _job(args):
    scn, name, seed, tmp = args
    return run_one(scn, name, seed, tmp)


def pct(xs, q):
    if not xs:
        return float("nan")
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(round(q * (len(xs) - 1))))]


def aggregate(results):
    """results: list of per-seed metric dicts for one (scenario, strategy)."""
    lat = [x for r in results for x in r["lat"]]
    cyc = [x for r in results for x in r["cycle"]]
    mean = lambda k: statistics.fmean(r[k] for r in results)
    bad = sum(1 for r in results if (not r["complete"]) or r["sem_reached"] or r["non_ff"]
              or r["lost_work"] or r["ungated_landings"])
    return {
        "strategy": results[0]["strategy"], "scenario": results[0]["scenario"], "n": len(results),
        "throughput": mean("throughput"),
        "throughput_sd": statistics.pstdev(r["throughput"] for r in results),
        "lat_p50": pct(lat, 0.5), "lat_p95": pct(lat, 0.95),
        "cycle_p50": pct(cyc, 0.5),
        "wasted_gate_min": mean("wasted_gate_min"), "gate_min": mean("gate_min"),
        "text_conflicts": mean("text_conflicts"), "auto_resolved": mean("auto_resolved"),
        "sem_reached_total": sum(r["sem_reached"] for r in results),
        "sem_reached_runs": sum(1 for r in results if r["sem_reached"]),
        "red_min": mean("red_min"), "blocked_min": mean("blocked_min"),
        "contamination": mean("contamination"), "swept": mean("swept"),
        "ungated_landings": sum(r["ungated_landings"] for r in results),
        "non_ff": sum(r["non_ff"] for r in results),
        "lost_work": sum(r["lost_work"] for r in results),
        "incomplete": sum(1 for r in results if not r["complete"]),
        "invariant_fail_runs": bad, "makespan": mean("makespan"),
    }


def fmt_table(rows):
    cols = [
        ("strategy", "strategy", "%s"), ("thr/h", "throughput", "%.1f"),
        ("lat p50", "lat_p50", "%.0f"), ("lat p95", "lat_p95", "%.0f"),
        ("cycle p50", "cycle_p50", "%.0f"),
        ("wasted gate-min", "wasted_gate_min", "%.0f"), ("text confl", "text_conflicts", "%.1f"),
        ("sem->main (runs)", None, None), ("red-main min", "red_min", "%.0f"),
        ("blocked min/agent", "blocked_min", "%.0f"), ("contam", "contamination", "%.1f"),
        ("bad runs", "invariant_fail_runs", "%d"),
    ]
    out = ["| " + " | ".join(c[0] for c in cols) + " |", "|" + "|".join("---" for _ in cols) + "|"]
    for r in rows:
        cells = []
        for title, key, f in cols:
            if key is None:
                cells.append("%d (%d)" % (r["sem_reached_total"], r["sem_reached_runs"]))
            else:
                cells.append(f % r[key])
        out.append("| " + " | ".join(cells) + " |")
    return "\n".join(out)


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--scenario", default="all", help="'all' or comma list: " + ",".join(scenarios.SCENARIOS))
    ap.add_argument("--strategies", default="all")
    ap.add_argument("--seeds", type=int, default=20)
    ap.add_argument("--jobs", type=int, default=min(32, os.cpu_count() or 1))
    ap.add_argument("--tmpdir", default=None, help="parent dir for throwaway repos (default: system temp)")
    ap.add_argument("--json", default=None, help="write per-run + aggregate results here")
    args = ap.parse_args(argv)

    scns = scenarios.get(args.scenario)
    names = list(STRATEGIES) if args.strategies == "all" else args.strategies.split(",")
    if args.tmpdir:
        os.environ["TMPDIR"] = args.tmpdir
        import tempfile
        tempfile.tempdir = args.tmpdir
    jobs = [(s, n, seed, None) for s in scns for n in names for seed in range(args.seeds)]
    t0 = time.time()
    with mp.get_context("fork").Pool(args.jobs) as pool:
        res = pool.map(_job, jobs, chunksize=1)
    by = {}
    for r in res:
        by.setdefault((r["scenario"], r["strategy"]), []).append(r)
    agg = {}
    for s in scns:
        rows = [aggregate(by[(s.name, n)]) for n in names]
        agg[s.name] = rows
        print("\n### %s  (%d agents x %d tasks, files=%d zipf=%.1f p_sem=%.2f flake=%.2f, seeds=%d)" % (
            s.name, s.n_agents, s.tasks_per_agent, s.n_files, s.zipf_s, s.p_semantic, s.flake_rate, args.seeds))
        print(fmt_table(rows))
    print("\n[%d runs in %.0fs]" % (len(jobs), time.time() - t0))
    if args.json:
        with open(args.json, "w") as fh:
            json.dump({"aggregate": agg, "runs": [{k: v for k, v in r.items() if k not in ("lat", "cycle")} for r in res]}, fh, indent=1)
    return agg


if __name__ == "__main__":
    main()

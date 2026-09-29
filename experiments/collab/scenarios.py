"""Scenario definitions.  All durations are virtual minutes."""
from dataclasses import dataclass, replace


@dataclass(frozen=True)
class Scenario:
    name: str
    n_agents: int = 8
    tasks_per_agent: int = 5
    n_files: int = 30
    hunks: int = 6              # hunk slots per file (distinct hunks merge cleanly)
    zipf_s: float = 0.8         # file popularity skew (0 = uniform)
    edit_mean: float = 25.0     # mean edit duration
    gate_mean: float = 8.0      # mean gate duration (one run, one slot)
    gate_slots: int = 2         # parallel gate runners (e.g. 2 GPUs)
    p_semantic: float = 0.10    # per-task chance of a hidden semantic partner
    flake_rate: float = 0.03    # per gate-run transient failure probability
    rework_frac: float = 0.30   # rework cost as a fraction of edit duration
    batch_growth: float = 0.03  # gate cost growth per extra change in a batch
    wip_break: float = 0.50     # P(gate fails) when a half-edited file is in the tree
    coord_cost: float = 0.5     # coordinator handling cost per item (C, D, F)
    lease_granularity: str = "file"  # "file" or "hunk"

    @property
    def n_tasks(self):
        return self.n_agents * self.tasks_per_agent


BASE = Scenario("baseline_8")

SCENARIOS = {s.name: s for s in [
    replace(BASE, name="low_contention", n_files=60, zipf_s=0.0, p_semantic=0.0, flake_rate=0.0),
    replace(BASE, name="hotspot", n_files=12, zipf_s=1.3, p_semantic=0.05, flake_rate=0.0),
    replace(BASE, name="high_semantic", n_files=30, zipf_s=0.5, p_semantic=0.35, flake_rate=0.0),
    replace(BASE, name="flaky_gate", flake_rate=0.25, p_semantic=0.05),
    replace(BASE, name="agents_2", n_agents=2),
    BASE,
    replace(BASE, name="agents_32", n_agents=32),
]}


def get(names):
    if names in ("all", None):
        return list(SCENARIOS.values())
    return [SCENARIOS[n] for n in names.split(",")]

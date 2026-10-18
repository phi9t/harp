"""Deterministic tests for the collaboration simulator.
Run:  python3 -m pytest experiments/collab/test_sim.py -q
"""
import os
import sys
import tempfile
from dataclasses import replace

import pytest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import scenarios  # noqa: E402
from engine import Resource, Sim  # noqa: E402
from gitrepo import GitRepo  # noqa: E402
from sim import run_one  # noqa: E402
from strategies import STRATEGIES  # noqa: E402
from strategies.common import Strategy  # noqa: E402
from world import World  # noqa: E402

SMALL = replace(scenarios.BASE, name="small", n_agents=6, tasks_per_agent=4, n_files=10, zipf_s=1.0,
                p_semantic=0.2, flake_rate=0.05)
CORRECT = [n for n in STRATEGIES if n != "A_shared_tree"]


def run_world(scn, name, seed):
    w = World(scn, seed)
    strat = STRATEGIES[name](w)
    strat.start()
    w.sim.run(until=200_000)
    return w, strat, w.metrics(name)


# ---------------------------------------------------------------- engine
def test_engine_time_order_and_resource_fifo():
    sim, log = Sim(), []
    res = Resource(sim, 1)

    def proc(name, start, hold):
        yield sim.timeout(start)
        yield res.request()
        log.append((name, sim.now))
        yield sim.timeout(hold)
        res.release()

    for n, s, h in [("a", 0, 5), ("b", 1, 1), ("c", 2, 1)]:
        sim.spawn(proc(n, s, h))
    sim.run()
    assert log == [("a", 0), ("b", 5), ("c", 6)]


# ------------------------------------------------------------ real git
@pytest.fixture()
def repo(tmp_path):
    r = GitRepo(str(tmp_path / "r"))
    base = r.init_files(3, 6)
    yield r, base
    r.close()


def test_real_git_conflict_detection(repo):
    r, base = repo
    a = r.edit(base, [(0, 1)], 1, (), False, 1.0)
    same = r.edit(base, [(0, 1)], 2, (), False, 1.0)
    other = r.edit(base, [(0, 4)], 3, (), False, 1.0)
    assert r.cherry_pick(other, a, 2.0) is not None      # different hunk, same file: clean
    assert r.cherry_pick(same, a, 2.0) is None           # same hunk line: real text conflict


def test_cherry_pick_agrees_with_real_git_rebase(repo):
    r, base = repo
    onto = r.edit(base, [(0, 1)], 1, (), False, 1.0)
    clean = r.edit(base, [(0, 4), (2, 2)], 2, (), False, 1.0)
    clash = r.edit(base, [(0, 1)], 3, (), False, 1.0)
    for name, c in (("clean", clean), ("clash", clash)):
        mine = r.cherry_pick(c, onto, 5.0)
        r.run("checkout", "-q", "--detach", c)
        real = r.run("rebase", onto, check=False)
        if mine is None:
            assert real.returncode != 0, name
            r.run("rebase", "--abort", check=False)
        else:
            assert real.returncode == 0, name
            tree = r.run("rev-parse", "HEAD^{tree}").stdout.strip()
            assert tree == r.commit_info(mine)[0], name


def test_world_counts_text_conflicts_from_git():
    w = World(SMALL, 0)
    try:
        t0, t1 = w.tasks[0], w.tasks[1]
        t1.edits = list(t0.edits)                    # force overlap
        a = w.edit_commit(w.main, t0)
        b = w.edit_commit(w.main, t1)
        assert w.cherry_pick(b, a) is None and w.text_conflicts == 1
    finally:
        w.close()


# ---------------------------------------------------------- determinism
@pytest.mark.parametrize("name", list(STRATEGIES))
def test_deterministic(name):
    assert run_one(SMALL, name, 3) == run_one(SMALL, name, 3)


def test_generated_tasks_are_seeded():
    from world import gen_tasks
    assert gen_tasks(SMALL, 1)[0] == gen_tasks(SMALL, 1)[0]
    assert gen_tasks(SMALL, 1)[0] != gen_tasks(SMALL, 2)[0]


# ------------------------------------------- one key invariant per strategy
def test_A_shared_tree_cross_contaminates_and_lands_ungated_work():
    w, _, m = run_world(SMALL, "A_shared_tree", 0)
    assert m["complete"]
    assert m["contamination"] > 0            # gates ran with others' uncommitted edits
    assert m["ungated_landings"] > 0         # commits swept in unreviewed work
    assert m["red_min"] > 0
    assert m["text_conflicts"] == 0          # no merge machinery at all


def test_A_can_land_semantic_conflicts_on_mainline():
    hot = replace(SMALL, n_agents=8, p_semantic=0.4)
    assert sum(run_world(hot, "A_shared_tree", s)[2]["sem_reached"] for s in range(8)) > 0


@pytest.mark.parametrize("seed", range(4))
def test_B_lock_serializes_integration_and_lands_only_gated_ff(seed):
    w, _, m = run_world(SMALL, "B_worktree_ff", seed)
    assert m["complete"] and m["sem_reached"] == 0 and m["non_ff"] == 0
    assert m["ungated_landings"] == 0 and m["lost_work"] == 0
    useful = sorted((r.start, r.end) for r in w.runs if r.useful)
    for (s1, e1), (s2, e2) in zip(useful, useful[1:]):
        assert s2 >= e1 - 1e-9               # landing gates never overlap under the lock
    w.close()


@pytest.mark.parametrize("seed", range(4))
def test_C_follower_only_advances_mainline_on_turn_boundaries(seed):
    from strategies.landing_lead import FOLLOW_PERIOD
    w, s, m = run_world(SMALL, "C_lead_follower", seed)
    assert m["complete"] and m["sem_reached"] == 0 and m["ungated_landings"] == 0 and m["non_ff"] == 0
    assert len(w.history) > 1
    for t, _ in w.history[1:]:
        assert abs(t / FOLLOW_PERIOD - round(t / FOLLOW_PERIOD)) < 1e-9
    assert m["max_gate_concurrency"] == 1    # lead verifies serially
    w.close()


@pytest.mark.parametrize("seed", range(4))
def test_D_queue_speculates_in_parallel_and_never_lands_a_red_tip(seed):
    hot = replace(SMALL, n_agents=8, p_semantic=0.35, flake_rate=0.0)
    w, _, m = run_world(hot, "D_merge_queue", seed)
    assert m["complete"] and m["sem_reached"] == 0 and m["ungated_landings"] == 0
    assert m["non_ff"] == 0 and m["lost_work"] == 0 and m["red_min"] == 0
    assert m["max_gate_concurrency"] == hot.gate_slots    # speculative parallel testing happened
    w.close()


def test_D_bisects_failures_out_of_the_queue():
    hot = replace(SMALL, n_agents=8, p_semantic=0.5, flake_rate=0.0)
    ej = sum(run_world(hot, "D_merge_queue", s)[2]["ejections"] for s in range(4))
    assert ej > 0


@pytest.mark.parametrize("name", ["E_leases", "E2_leases_locked"])
@pytest.mark.parametrize("seed", range(4))
def test_E_leases_never_overlap_and_prevent_text_conflicts(name, seed):
    w, s, m = run_world(SMALL, name, seed)
    assert m["complete"] and m["sem_reached"] == 0 and m["ungated_landings"] == 0 and m["non_ff"] == 0
    assert m["text_conflicts"] == 0
    by_key = {}
    for key, agent, t0, t1 in s.intervals:
        by_key.setdefault(key, []).append((t0, t1))
    for spans in by_key.values():
        spans.sort()
        for (a0, a1), (b0, b1) in zip(spans, spans[1:]):
            assert b0 >= a1 - 1e-9           # a leased region has one holder at a time
    w.close()


@pytest.mark.parametrize("seed", range(4))
def test_F_patch_stream_gates_once_per_batch_and_stays_correct(seed):
    calm = replace(SMALL, n_agents=8, p_semantic=0.0, flake_rate=0.0, n_files=40, zipf_s=0.0)
    w, _, m = run_world(calm, "F_patch_stream", seed)
    assert m["complete"] and m["sem_reached"] == 0 and m["ungated_landings"] == 0
    assert m["gate_runs"] < m["n_tasks"]      # batching: fewer gate runs than tasks
    w.close()
    hot = replace(SMALL, n_agents=8, p_semantic=0.4)
    m2 = run_world(hot, "F_patch_stream", seed)[2]
    assert m2["complete"] and m2["sem_reached"] == 0 and m2["ungated_landings"] == 0


# ----------------------------------------------- the detector detects
class NoGate(Strategy):
    """Deliberately broken: rebases and fast-forwards without ever gating."""
    name = "broken_nogate"

    def start(self):
        for a in range(self.w.scn.n_agents):
            self.w.spawn(self.agent(a))

    def agent(self, a):
        w = self.w
        yield w.sim.timeout(w.offsets[a])
        for t in w.agent_tasks[a]:
            base = w.main
            yield from w.work(a, t.edit_dur)
            c = w.edit_commit(base, t)
            w.mark_ready(t)
            tip = w.cherry_pick(c, w.main)
            if tip is None:
                tip = w.edit_commit(w.main, t)
            w.land(tip)


def test_harness_flags_a_strategy_that_lets_semantic_conflicts_through():
    scn = replace(SMALL, n_agents=8, p_semantic=0.6)
    w = World(scn, 1)
    NoGate(w).start()
    w.sim.run()
    m = w.metrics("broken_nogate")
    w.close()
    assert m["sem_reached"] > 0 and m["ungated_landings"] > 0 and m["red_min"] > 0


# -------------------------------------------- smoke: every strategy x scenario
@pytest.mark.parametrize("scn", scenarios.get("all"), ids=lambda s: s.name)
@pytest.mark.parametrize("name", list(STRATEGIES))
def test_all_strategies_complete_everywhere(scn, name):
    tiny = replace(scn, tasks_per_agent=2, n_agents=min(scn.n_agents, 12))
    m = run_one(tiny, name, 0)
    assert m["complete"]
    if name != "A_shared_tree":
        assert m["sem_reached"] == 0 and m["ungated_landings"] == 0 and m["non_ff"] == 0 and m["lost_work"] == 0

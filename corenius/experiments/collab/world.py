"""Simulation world: tasks, git repo, gate oracle, metrics.

Ground truth lives in the git tree.  Every task appends a token `t<id>` to each
(file, hunk) line it edits; a half-finished edit carries `t<id>~`.  A hidden
semantic pair (a, b) is a pair of textually independent tasks that break the
gate when both are present unless one of them carries the token `r<a>-<b>`
(produced by rework).  The gate oracle reads the tree it is given, so every
strategy is judged on what it actually put in git.
"""
import math
import os
import random
import shutil
import tempfile
from dataclasses import dataclass, field

from engine import Resource, Sim
from gitrepo import EPOCH, GitRepo, fname, parse_tokens

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


@dataclass
class Task:
    id: int
    agent: int
    edits: list
    edit_dur: float
    gate_factor: float


@dataclass
class Run:
    start: float
    end: float = 0.0
    cost: float = 0.0
    useful: bool = False
    outcome: str = ""


@dataclass
class GateResult:
    ok: object            # True / False / None (cancelled)
    reason: str           # ok | semantic | wip | cancelled
    pairs: list
    runs: list = field(default_factory=list)

    def mark_useful(self):
        if self.runs:
            self.runs[-1].useful = True


def gen_tasks(scn, seed):
    rng = random.Random("tasks|%s|%d" % (scn.name, seed))
    weights = [1.0 / (i + 1) ** scn.zipf_s for i in range(scn.n_files)]
    tasks, tid = [], 0
    for a in range(scn.n_agents):
        for _ in range(scn.tasks_per_agent):
            k = rng.choice([1, 1, 2, 2, 3])
            edits = set()
            while len(edits) < k:
                f = rng.choices(range(scn.n_files), weights)[0]
                edits.add((f, rng.randrange(scn.hunks)))
            dur = min(max(rng.lognormvariate(math.log(scn.edit_mean) - 0.125, 0.5), 3.0), 4 * scn.edit_mean)
            tasks.append(Task(tid, a, sorted(edits), dur, 0.7 + 0.6 * rng.random()))
            tid += 1
    pairs = set()
    for t in tasks:
        if rng.random() < scn.p_semantic:
            o = rng.choice(tasks)
            if o.id != t.id:
                pairs.add((min(t.id, o.id), max(t.id, o.id)))
    offsets = [rng.uniform(0, scn.edit_mean) for _ in range(scn.n_agents)]
    return tasks, sorted(pairs), offsets


class World:
    def __init__(self, scn, seed, tmpdir=None):
        self.scn, self.seed = scn, seed
        self.sim = Sim()
        self.tasks, self.pairs, self.offsets = gen_tasks(scn, seed)
        self.by_id = {t.id: t for t in self.tasks}
        self.agent_tasks = {a: [t for t in self.tasks if t.agent == a] for a in range(scn.n_agents)}
        self._own_tmp = tmpdir is None
        parent = os.environ.get("TMPDIR")
        if tmpdir is not None:
            validator = _temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]
            tmpdir = str(validator(tmpdir))
        if not tmpdir and (not parent or not os.path.isabs(parent)):
            raise ValueError("Run with tools/temp_root.py or set an absolute TMPDIR")
        forbidden = os.path.join(os.sep, "tmp")
        if not tmpdir and (os.path.realpath(parent) == forbidden or os.path.realpath(parent).startswith(forbidden + os.sep)):
            raise ValueError("TMPDIR is forbidden")
        self.tmp = tmpdir or tempfile.mkdtemp(prefix="collab-", dir=parent)
        self.repo = GitRepo(os.path.join(self.tmp, "repo"))
        self.main = self.repo.init_files(scn.n_files, scn.hunks)
        self.slots = Resource(self.sim, scn.gate_slots)
        self.gated_ok = {self.main}
        self.history = [(0.0, self.main)]
        # bookkeeping
        self.ready, self.landed, self.task_start = {}, {}, {}
        self.busy = {a: 0.0 for a in range(scn.n_agents)}
        self.runs = []
        self.text_conflicts = 0
        self.auto_resolved = 0
        self.ejections = 0
        self.contamination = 0
        self.swept = 0
        self.ungated_landings = 0
        self.non_ff = 0
        self.lost_work = 0
        self.sem_reached = set()
        self.red_since = None
        self.red_total = 0.0
        self.landings = 0
        self._attempts = {}
        self.known_recon = {}

    # -- lifecycle -----------------------------------------------------------
    def close(self):
        self.repo.close()
        if self._own_tmp:
            shutil.rmtree(self.tmp, ignore_errors=True)

    @property
    def now(self):
        return self.sim.now

    def spawn(self, gen):
        return self.sim.spawn(gen)

    def work(self, agent, dur):
        yield self.sim.timeout(dur)
        self.busy[agent] += dur

    def rework_dur(self, task):
        return self.scn.rework_frac * task.edit_dur

    def all_landed(self):
        return len(self.landed) == len(self.tasks)

    def mark_start(self, task):
        self.task_start.setdefault(task.id, self.now)

    def mark_ready(self, task):
        self.ready.setdefault(task.id, self.now)

    def draw(self, kind, key):
        n = self._attempts.get((kind, key), 0)
        self._attempts[(kind, key)] = n + 1
        return random.Random("%d|%s|%s|%d" % (self.seed, kind, key, n)).random()

    # -- git helpers -----------------------------------------------------------
    def edit_commit(self, parent, task, recon=(), wip=False):
        # an author remembers every reconciliation it has already had to make
        known = self.known_recon.setdefault(task.id, set())
        known.update(recon or ())
        return self.repo.edit(parent, task.edits, task.id, tuple(sorted(known)), wip, self.now)

    def cherry_pick(self, commit, onto):
        c = self.repo.cherry_pick(commit, onto, self.now)
        if c is None:
            self.text_conflicts += 1
        return c

    def tokens(self, commit):
        return self.repo.tokens(commit)

    def violations(self, tok):
        fin, wip, rec = tok
        present = fin | wip
        return [p for p in self.pairs if p[0] in present and p[1] in present and p not in rec]

    def is_red(self, tok):
        return bool(self.violations(tok)) or bool(tok[1])

    # -- the gate ---------------------------------------------------------------
    def gate(self, target, ids, primary, cancel=None):
        """Generator: run the gate on `target` (a commit id, or a callable that
        returns the current token triple of a mutable working tree) until a
        definitive answer.  Transient flakes are re-run.  Each run occupies one
        gate slot for its duration.  Returns a GateResult."""
        ids = list(ids)
        task = self.by_id[primary]
        runs = []
        while True:
            req = self.slots.request()
            if cancel is not None:
                yield self.sim.any_of(req, cancel)
                if cancel.done:
                    if req.done:
                        self.slots.release()
                    else:
                        self.slots.withdraw(req)
                    return GateResult(None, "cancelled", [], runs)
            else:
                yield req
            dur = self.scn.gate_mean * task.gate_factor * (1 + self.scn.batch_growth * (len(ids) - 1))
            run = Run(self.now)
            live = callable(target)
            tok0 = target() if live else self.tokens(target)
            if live:
                main_tok = self.tokens(self.main)
            to = self.sim.timeout(dur)
            yield (self.sim.any_of(to, cancel) if cancel is not None else to)
            self.slots.release()
            run.end, run.cost = self.now, self.now - run.start
            runs.append(run)
            self.runs.append(run)
            if cancel is not None and cancel.done and not to.done:
                run.outcome = "cancelled"
                return GateResult(None, "cancelled", [], runs)
            if live:
                # contamination: someone else's uncommitted change was in the tree
                tok1 = target()
                theirs = ((tok0[0] | tok0[1] | tok1[0] | tok1[1]) - (main_tok[0] | main_tok[1]) - set(ids))
                if theirs:
                    self.contamination += 1
            pairs = self.violations(tok0)
            if pairs:
                run.outcome = "semantic"
                return GateResult(False, "semantic", pairs, runs)
            if self.draw("flake", primary) < self.scn.flake_rate:
                run.outcome = "flake"
                continue
            if tok0[1] and self.draw("wip", primary) < self.scn.wip_break:
                run.outcome = "wip"
                return GateResult(False, "wip", [], runs)
            run.outcome = "pass"
            if not live:
                self.gated_ok.add(target)
            return GateResult(True, "ok", [], runs)

    # -- landing ------------------------------------------------------------------
    def land(self, new, expect=None, moved=False):
        """Advance mainline to `new`.  `moved`: git already moved the ref (shared tree)."""
        old = self.main
        if expect is not None and expect != old:
            raise AssertionError("mainline moved under a landing that assumed %s" % expect[:8])
        if not self.repo.is_ancestor(old, new):
            self.non_ff += 1
        if new not in self.gated_ok:
            self.ungated_landings += 1
        if not moved:
            self.repo.update_ref(new, old)
        prev = self.tokens(old)
        tok = self.tokens(new)
        self.lost_work += len(prev[0] - tok[0] - tok[1])
        for p in self.violations(tok):
            self.sem_reached.add(p)
        red = self.is_red(tok)
        if red and self.red_since is None:
            self.red_since = self.now
        elif not red and self.red_since is not None:
            self.red_total += self.now - self.red_since
            self.red_since = None
        for tid in tok[0]:
            if tid not in self.landed:
                self.landed[tid] = self.now
        self.main = new
        self.landings += 1
        self.history.append((self.now, new))

    # -- results --------------------------------------------------------------------
    def metrics(self, strategy):
        scn = self.scn
        complete = self.all_landed()
        makespan = max(self.landed.values()) if self.landed else 0.0
        if self.red_since is not None:
            self.red_total += max(makespan - self.red_since, 0.0)
        lat = [self.landed[t] - self.ready[t] for t in self.landed if t in self.ready]
        cyc = [self.landed[t] - self.task_start[t] for t in self.landed if t in self.task_start]
        blocked = []
        for a in range(scn.n_agents):
            done = [self.landed[t.id] for t in self.agent_tasks[a] if t.id in self.landed]
            fin = max(done) if done else makespan
            blocked.append(max(fin - self.offsets[a] - self.busy[a], 0.0))
        total = sum(r.cost for r in self.runs)
        wasted = sum(r.cost for r in self.runs if not r.useful)
        return {
            "strategy": strategy, "scenario": scn.name, "seed": self.seed,
            "complete": complete, "n_tasks": len(self.tasks), "landed": len(self.landed),
            "makespan": makespan,
            "throughput": (len(self.landed) / (makespan / 60.0)) if makespan > 0 else 0.0,
            "lat": lat, "cycle": cyc,
            "gate_runs": len(self.runs), "gate_min": total, "wasted_gate_min": wasted,
            "text_conflicts": self.text_conflicts, "auto_resolved": self.auto_resolved,
            "ejections": self.ejections,
            "sem_reached": len(self.sem_reached), "red_min": self.red_total,
            "blocked_min": sum(blocked) / len(blocked),
            "contamination": self.contamination, "swept": self.swept,
            "ungated_landings": self.ungated_landings, "non_ff": self.non_ff,
            "lost_work": self.lost_work, "n_pairs": len(self.pairs),
            "max_gate_concurrency": self.slots.max_users,
        }

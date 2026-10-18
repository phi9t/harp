"""Shared strategy plumbing: the Strategy interface, async agents (submit and keep
working, rework arrives in a mailbox) and the batch/bisect lander used by the
merge queue (D) and the patch stream (F)."""
from dataclasses import dataclass


class Strategy:
    """Interface: __init__(world); start() spawns agent/daemon processes."""
    name = "base"

    def __init__(self, world):
        self.w = world

    def start(self):
        raise NotImplementedError


@dataclass
class Item:
    task: object
    commit: str
    recon: tuple
    t_submit: float


class AsyncAgents:
    """Agents that submit a finished change and immediately start the next task.
    Bounced work (text conflict / semantic failure) waits in a mailbox and is
    picked up at the agent's next task boundary."""

    def init_agents(self):
        self.mail = {a: [] for a in range(self.w.scn.n_agents)}
        self.mail_ev = {}

    def base(self):
        return self.w.main

    def submit(self, task, commit, recon):
        raise NotImplementedError

    def bounce(self, task, pairs):
        self.w.ejections += 1
        self.mail[task.agent].append((task, tuple(pairs or ())))
        ev = self.mail_ev.pop(task.agent, None)
        if ev is not None:
            ev.succeed()

    def agent(self, a):
        w = self.w
        yield w.sim.timeout(w.offsets[a])
        tasks = list(w.agent_tasks[a])
        while True:
            if self.mail[a]:
                t, pairs = self.mail[a].pop(0)
                yield from w.work(a, w.rework_dur(t))
                self.submit(t, w.edit_commit(self.base(), t, recon=pairs), pairs)
            elif tasks:
                t = tasks.pop(0)
                w.mark_start(t)
                base = self.base()
                yield from w.work(a, t.edit_dur)
                c = w.edit_commit(base, t)
                w.mark_ready(t)
                self.submit(t, c, ())
            elif all(t.id in w.landed for t in w.agent_tasks[a]):
                return
            else:
                ev = w.sim.event()
                self.mail_ev[a] = ev
                yield ev


class BatchLander(AsyncAgents):
    """Queue of ready changes.  Each round: stack up to K*S queued changes onto
    mainline with real git cherry-picks (text conflicts are ejected, or
    auto-resolved by re-applying the structured edit when `auto_p` > 0), cut the
    stack into S cumulative batches, gate all batches in parallel (batch j is
    tested on top of batches < j), then land in order.  A failing batch cancels
    the later speculative runs and is bisected to its culprit; the culprit is
    bounced to its author, everything before it lands, everything after it is
    restacked next round.  Only gated tips are ever fast-forwarded."""
    S = 1
    K = 4
    window = 0.0
    auto_p = 0.0

    def start(self):
        w = self.w
        self.init_agents()
        self.queue = []
        self.wake = None
        for a in range(w.scn.n_agents):
            w.spawn(self.agent(a))
        w.spawn(self.daemon())

    def submit(self, task, commit, recon):
        self.queue.append(Item(task, commit, tuple(recon), self.w.now))
        if self.wake is not None:
            ev, self.wake = self.wake, None
            ev.succeed()

    def _auto_resolves(self, task):
        return self.auto_p > 0 and self.w.draw("auto", task.id) < self.auto_p

    def daemon(self):
        w = self.w
        while True:
            if not self.queue:
                self.wake = w.sim.event()
                yield self.wake
                continue
            wait = min(i.t_submit for i in self.queue) + self.window - w.now
            if wait > 0:
                yield w.sim.timeout(wait)
            items = list(self.queue[: self.K * self.S])
            yield w.sim.timeout(w.scn.coord_cost * len(items))
            # ---- stack onto mainline
            tip, stacked, P = w.main, [], [w.main]
            for it in items:
                c = w.cherry_pick(it.commit, tip)
                if c is None:
                    if self._auto_resolves(it.task):
                        w.auto_resolved += 1
                        c = w.edit_commit(tip, it.task, recon=it.recon)
                    else:
                        self.queue.remove(it)
                        self.bounce(it.task, ())
                        continue
                stacked.append(it)
                tip = c
                P.append(c)
            n = len(stacked)
            ends = list(range(self.K, n, self.K)) + ([n] if n else [])
            ids = [it.task.id for it in stacked]
            cancel = w.sim.event()
            procs = [
                w.spawn(w.gate(P[e], ids[:e], ids[e - 1], cancel)) for e in ends
            ]
            landed_upto = 0
            for e, proc in zip(ends, procs):
                res = yield proc
                if res.ok:
                    w.land(P[e], expect=w.main)
                    res.mark_useful()
                    for it in stacked[landed_upto:e]:
                        self.queue.remove(it)
                    landed_upto = e
                    continue
                cancel.succeed()
                lo, hi, lo_res, pairs = landed_upto, e, None, res.pairs
                while hi - lo > 1:
                    mid = (lo + hi) // 2
                    r = yield from w.gate(P[mid], ids[:mid], ids[mid - 1])
                    if r.ok:
                        lo, lo_res = mid, r
                    else:
                        hi, pairs = mid, r.pairs
                if lo > landed_upto:
                    w.land(P[lo], expect=w.main)
                    lo_res.mark_useful()
                    for it in stacked[landed_upto:lo]:
                        self.queue.remove(it)
                culprit = stacked[hi - 1]
                self.queue.remove(culprit)
                self.bounce(culprit.task, pairs)
                break

from strategies.common import Strategy
from engine import Resource


class WorktreeRebaseFF(Strategy):
    """B: per-agent worktree; integration under one global lock: rebase onto
    mainline, gate the exact tip, fast-forward, retry if anything fails."""
    name = "B_worktree_ff"

    def start(self):
        self.lock = Resource(self.w.sim, 1)
        for a in range(self.w.scn.n_agents):
            self.w.spawn(self.agent(a))

    def agent(self, a):
        w = self.w
        yield w.sim.timeout(w.offsets[a])
        for t in w.agent_tasks[a]:
            w.mark_start(t)
            base = w.main
            yield from w.work(a, t.edit_dur)
            c = w.edit_commit(base, t)
            w.mark_ready(t)
            while True:
                yield self.lock.request()
                onto = w.main
                tip = w.cherry_pick(c, onto)
                if tip is None:
                    self.lock.release()
                    yield from w.work(a, w.rework_dur(t))
                    c = w.edit_commit(w.main, t)
                    continue
                res = yield from w.gate(tip, [t.id], t.id)
                if res.ok:
                    w.land(tip, expect=onto)  # lock guarantees mainline did not move
                    res.mark_useful()
                    self.lock.release()
                    break
                self.lock.release()
                yield from w.work(a, w.rework_dur(t))
                c = w.edit_commit(w.main, t, recon=res.pairs)

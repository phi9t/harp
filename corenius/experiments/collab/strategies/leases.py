from strategies.common import Strategy
from engine import Resource


class Leases(Strategy):
    """E: agents lease the files (or hunks) they will touch before editing, in a
    global sorted order (deadlock-free); conflicts become waits.  Integration is
    optimistic and fast-forward-only: rebase, gate, then compare-and-swap main;
    if main moved during the gate, rebase and gate again.  Leases are held until
    the change has landed."""
    name = "E_leases"

    serialize = False   # E2 variant: also hold a global integration lock

    def _unlock(self):
        if self.lock is not None:
            self.lock.release()

    def start(self):
        self.lock = Resource(self.w.sim, 1) if self.serialize else None
        self.res = {}
        self.intervals = []   # (key, agent, t_acquired, t_released) for the tests
        for a in range(self.w.scn.n_agents):
            self.w.spawn(self.agent(a))

    def keys(self, t):
        if self.w.scn.lease_granularity == "hunk":
            return sorted(t.edits)
        return sorted({f for f, _ in t.edits})

    def agent(self, a):
        w = self.w
        yield w.sim.timeout(w.offsets[a])
        for t in w.agent_tasks[a]:
            w.mark_start(t)
            held = []
            for k in self.keys(t):
                r = self.res.setdefault(k, Resource(w.sim, 1))
                yield r.request()
                held.append((k, w.now))
            base = w.main
            yield from w.work(a, t.edit_dur)
            c = w.edit_commit(base, t)
            w.mark_ready(t)
            recon = ()
            while True:
                if self.lock is not None:
                    yield self.lock.request()
                onto = w.main
                tip = w.cherry_pick(c, onto)
                if tip is None:  # cannot happen while leases are honoured
                    self._unlock()
                    yield from w.work(a, w.rework_dur(t))
                    c = w.edit_commit(w.main, t, recon=recon)
                    continue
                res = yield from w.gate(tip, [t.id], t.id)
                if res.ok:
                    if w.main == onto:
                        w.land(tip, expect=onto)
                        res.mark_useful()
                        self._unlock()
                        break
                    self._unlock()
                    c = tip          # lost the race: rebase again, re-gate
                    continue
                self._unlock()
                recon = res.pairs
                yield from w.work(a, w.rework_dur(t))
                c = w.edit_commit(w.main, t, recon=recon)
            for k, t0 in held:
                self.intervals.append((k, a, t0, w.now))
                self.res[k].release()


class LeasesLocked(Leases):
    """E2 (extra variant): leases for region ownership PLUS the global
    integration lock of B, so rebase+gate+ff is never raced."""
    name = "E2_leases_locked"
    serialize = True

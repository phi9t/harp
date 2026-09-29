from collections import deque

from strategies.common import AsyncAgents, Item, Strategy

FOLLOW_PERIOD = 15.0  # minutes between follower turn boundaries


class LeadFollower(AsyncAgents, Strategy):
    """C: agents push to a landing branch owned by one lead, who rebases and
    verifies submissions serially (one gate at a time).  A single follower
    fast-forwards mainline to the verified landing tip only at turn boundaries
    (every FOLLOW_PERIOD minutes).  Agents base new work on the landing tip and
    do not block on the lead; bounced work returns via their mailbox."""
    name = "C_lead_follower"

    def start(self):
        w = self.w
        self.init_agents()
        self.landing = w.main
        self.inbox = deque()
        self.wake = None
        self.follow_times = []
        for a in range(w.scn.n_agents):
            w.spawn(self.agent(a))
        w.spawn(self.lead())
        w.spawn(self.follower())

    def base(self):
        return self.landing

    def submit(self, task, commit, recon):
        self.inbox.append(Item(task, commit, tuple(recon), self.w.now))
        if self.wake is not None:
            ev, self.wake = self.wake, None
            ev.succeed()

    def lead(self):
        w = self.w
        while True:
            if not self.inbox:
                self.wake = w.sim.event()
                yield self.wake
                continue
            it = self.inbox.popleft()
            yield w.sim.timeout(w.scn.coord_cost)
            tip = w.cherry_pick(it.commit, self.landing)
            if tip is None:
                self.bounce(it.task, ())
                continue
            res = yield from w.gate(tip, [it.task.id], it.task.id)
            if res.ok:
                self.landing = tip
                res.mark_useful()
            else:
                self.bounce(it.task, res.pairs)

    def follower(self):
        w = self.w
        while not w.all_landed():
            yield w.sim.timeout(FOLLOW_PERIOD)
            if self.landing != w.main:
                w.land(self.landing, expect=w.main)
                self.follow_times.append(w.now)

"""Tiny deterministic discrete-event engine (simpy-like, stdlib only).

Processes are generators that yield Events.  Time is virtual (minutes); nothing
ever sleeps.  Ties are broken by insertion order, so runs are reproducible.
"""
import heapq
import itertools
from collections import deque


class Event:
    def __init__(self, sim):
        self.sim = sim
        self.done = False
        self.value = None
        self.cbs = []

    def succeed(self, value=None):
        if self.done:
            return
        self.done = True
        self.value = value
        cbs, self.cbs = self.cbs, []
        for cb in cbs:
            self.sim.schedule(0, cb, value)

    def wait(self, cb):
        if self.done:
            self.sim.schedule(0, cb, self.value)
        else:
            self.cbs.append(cb)


class Process(Event):
    def __init__(self, sim, gen):
        super().__init__(sim)
        self.gen = gen
        sim.schedule(0, self._step, None)

    def _step(self, val):
        try:
            ev = self.gen.send(val)
        except StopIteration as stop:
            self.succeed(stop.value)
            return
        ev.wait(self._step)


class Resource:
    """FIFO counting resource (gate-runner slots, locks, leases)."""

    def __init__(self, sim, capacity):
        self.sim = sim
        self.capacity = capacity
        self.users = 0
        self.queue = deque()
        self.max_users = 0

    def request(self):
        ev = Event(self.sim)
        if self.users < self.capacity:
            self.users += 1
            self.max_users = max(self.max_users, self.users)
            ev.succeed()
        else:
            self.queue.append(ev)
        return ev

    def release(self):
        if self.queue:
            self.queue.popleft().succeed()  # slot transfers to the next waiter
        else:
            self.users -= 1

    def withdraw(self, ev):
        try:
            self.queue.remove(ev)
            return True
        except ValueError:
            return False


class Sim:
    def __init__(self):
        self.now = 0.0
        self._q = []
        self._seq = itertools.count()

    def schedule(self, delay, fn, arg=None):
        heapq.heappush(self._q, (self.now + delay, next(self._seq), fn, arg))

    def event(self):
        return Event(self)

    def timeout(self, delay):
        ev = Event(self)
        self.schedule(delay, ev.succeed, None)
        return ev

    def spawn(self, gen):
        return Process(self, gen)

    def any_of(self, *events):
        out = Event(self)
        for ev in events:
            ev.wait(lambda v, ev=ev: out.succeed(ev))
        return out

    def run(self, until=float("inf")):
        while self._q:
            t, _, fn, arg = heapq.heappop(self._q)
            if t > until:
                heapq.heappush(self._q, (t, _, fn, arg))
                break
            self.now = t
            fn(arg)

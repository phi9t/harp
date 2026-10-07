# Lead protocol for this run (confirmed by the owner 2026-10-05)

The lead follows these for the whole effort. A replacement lead reads this first.

## Questions this run must answer
1. Does the done signal (marker commit on a clean tree) hold up: produced, forgotten, or produced and then written past?
2. What does the lead actually do per ticket, and how does the time split between reading the diff, re-running tests, checking claims and typing into a pane?
3. Which interventions recur? One-offs are noted; recurring ones become tickets.
4. Is rework through the same session reliable, including when the worker's context is nearly full?
5. Where do parallel workers actually collide, and are declared paths a useful control?
6. What do the monitors need to show: which columns are read, which ignored, what had to be looked up by hand?
7. How well does a worker follow a long prose prompt? Record each boundary ignored.

## Standing rules
1. Log every lead action in `lead-log.md`, not only problems: each review, each message to a worker, each manual lookup, with its trigger, what was done and rough duration.
2. After ticket 01 lands, and again after the first parallel round, compare what happened with what the remaining tickets assume and propose ticket changes to the owner before launching more.
3. Keep `ws.py` small. Every change to it goes in `notes.md` as a coordinator change with its reason; each is a requirement on the real tooling.
4. Let later tickets build on what works. If the marker commit and report file hold up, they become the toolbox design. Ticket 01 was written with a process-exit hand-off, which does not fit interactive sessions that never exit; check this at its review and correct it by a rework round, not by interrupting the worker.
5. Read the observer's `ATTENTION.md` at every wake-up, before anything else.
6. Do not commit on the integration branch while any worker is running (see notes: it makes every running Candidate stale).

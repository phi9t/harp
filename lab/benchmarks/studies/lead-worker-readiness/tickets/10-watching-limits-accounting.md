# 10 watching-limits-accounting: Workers can be watched, are bounded, and are accounted for

Type: feature
Status: resolved
Owner: worker-10
Blocked by: 04
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 37 (live part noted), 38, 39, 40, 41. Give each launched worker a window in a named tmux session showing its activity readably, backed by a durable action record that survives the window closing. Raise a notice when a worker passes its declared wall-clock limit or quiet time. When a ticket lands or is abandoned, every process of its worker is shown as stopped, or as explicitly retained for rework, and a retained worker counts against the parallel cap. Report the disk used by each worker's workspace and caches at the end of its run. tmux is a convenience: a launch still succeeds when tmux is missing.

Amended by the lead 2026-10-05 from what the run has shown. This ticket is where the interim coordinator the lead has been running by hand gets replaced. Read it first: `evidence/lead/ws.py.txt` (about 500 lines, no tests), with `evidence/lead/notes.md` and `evidence/lead/lead-log.md` for why each part exists.
- The owner wants workers as interactive sessions he can watch and type into: one tmux window per worker in a named session, started with the runtime's interactive command in the worker's worktree, with the marker hand-off (ticket 01), not a view of a non-interactive run. Add that as a launch mode of the real launch command (`lead_launch.py`), beside the existing non-interactive mode. The durable action record for an interactive session is the runtime's own session transcript: record its path in the run record once the session id is known.
- Two monitor commands, each usable in a tmux window on an interval and as one-shot JSON: a ticket monitor (one row per ticket: tracker status, triage, worker state, commits ahead of the integration tip, whether the branch contains the tip, dirty count, age of the last commit, what it waits for) and a worker monitor (per running worker: process, uptime, current activity, context remaining where the runtime shows it, disk used, last hand-off or delivery). The lead never used the tracker-status and triage columns to decide anything and looked up three things by hand at every wake-up: whether the tip was rewritten since it was last seen, the diff since the last reviewed Candidate, and the changed paths outside the ticket's declared paths. Show those three.
- Worker states the run needed, all derived from git, the process table and the pane, never from a worker's claim: running; finishing (marker commit made, session still busy); ready (marker, clean tree, session idle, tip not already reviewed); rework (a review was delivered and no new marker yet); waiting for re-integration (ticket 04's state); needs-info; stopped (no process, no marker); landed.
- A blocking wait command for the lead: return when any named ticket leaves the running states, and print why. It must not fire on `finishing` or `rework`.
- Probes of a worker's worktree take no git locks. `status` either changes nothing or its help says what it writes (finding addressed to this ticket).
- tmux remains a convenience for the non-interactive mode and a requirement only for the interactive one; say so in the help.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/lead_launch.py`, a new monitor module if you want one, their tests
- `bench/tmuxh/` only by import of its event formatter

## Acceptance
- [x] A launch into a named tmux session creates the session if needed and one window per worker; with tmux missing or failing the launch succeeds and records that there is no window.
- [x] The durable action record contains each command the scripted participant ran, after the window is closed.
- [x] Passing the wall-clock limit and passing the quiet time each append a notice to the run's status record while the worker is still running.
- [x] After a ticket lands, an accounting command lists each process of its worker as stopped; a worker deliberately retained for rework is listed as retained and reduces the available parallel slots by one.
- [x] The end-of-run record includes the disk used by the worker's worktree and its caches.
- [x] The scenarios that are deterministic run in the lead-driven mode and match their expectation.

## Comments
- 2026-10-05T13:00:02Z worker-10: claimed until 2026-10-07T13:00:02Z
- 2026-10-05T14:50:31Z worker-10: resolved

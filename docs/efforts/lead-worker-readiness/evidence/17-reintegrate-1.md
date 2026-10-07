# Ticket 17: content accepted; re-integrate, then hand over

I re-ran the real probe myself against 6087d2d for both runtimes. Codex: verdict CLEAN, ready, Candidate
imported as `worker/probe-codex`. traecli: verdict CLEAN, ready, Candidate imported as
`worker/probe-traecli`. In the shared repository `main` and `side` were unchanged and the status clean. The
git directory outside the workspace, under the run directory, with the workspace's `.git` a pointer file,
does what was needed. This message is not a review round.

`effort/lead-worker-readiness` has moved: tickets 06 and 10 landed since your base, and a ticket 18 now
exists. Ticket 06 changed `tools/agents/worker.py` (file-change events, unaccounted changes),
`tools/agents/lead_launch.py` (an `overlap` command) and the harness module, and added the
`parallel-workers-and-overlap` scenario, which launches two workers and lands them. Ticket 10 changed
`tools/agents/lead_launch.py` (watch windows, `monitor-tickets`, `wait`, `account`, `retain-worker`),
`tools/agents/worker.py` (limit notices, disk) and the harness module, and added the
`watching-limits-accounting` scenario.

Do: rebase your own branch onto the current tip of `effort/lead-worker-readiness`; resolve conflicts by
intent; make ticket 06's overlap report and ticket 10's monitor, wait, accounting and disk figures read the
worker's private repository (commits ahead, dirty state, tip, size including its git directory) now that a
worker is no longer a worktree of the shared repository; move your finding for ticket 18 into
`## For later tickets` as a `- to 18:` line now that the ticket exists; rerun the whole gate (every
`tools/agents/test_*.py` suite; every bench harness suite; every scenario under `bench/readiness/`;
`tracker.py lint`); and finish with a new final commit `ticket 17: ready for review` on a clean tree.
Nothing else is running; nothing will land before you.

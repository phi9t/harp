# Ticket 10: content accepted as built; re-integrate now

Your Candidate meets the ticket's acceptance items (watch windows, limits notices, accounting, disk, wait).
The paragraph I added to the ticket's description after it was written (an interactive launch mode, a worker
monitor, more monitor columns and states) was not in the acceptance items and you did not build it; that
was my omission, and that work is now a separate ticket 18. Do not start it. This message is not a review
round.

`effort/lead-worker-readiness` has moved: tickets 16 and 06 landed since your base. Ticket 16 changed
`tools/agents/lead_launch.py`, `tools/agents/land.py`, `tools/agents/tracker.py`, `tools/agents/effort.py`
and `tools/agents/acceptance.py` (an effort's records may live on a record branch; one lookup for an effort's
record) and the harness module. Ticket 06 changed `tools/agents/lead_launch.py` (an `overlap` command,
launch-time overlap notes), `tools/agents/worker.py` (file-change events, unaccounted changes) and the
harness module.

Do: rebase your own branch onto the current tip of `effort/lead-worker-readiness`, resolve conflicts by
intent (your `monitor-tickets`, `wait`, `account` and `retain-worker` commands must find the effort through
ticket 16's lookup and work on an effort whose records are on a record branch, run from the main checkout),
rerun the whole gate (every `tools/agents/test_*.py` suite; every bench harness suite; every scenario under
`bench/readiness/`; `tracker.py lint`), and finish with a new final commit `ticket 10: ready for review` on a
clean tree. Ticket 17 is running and will not land before you. Your context is low: if you cannot finish,
commit what you have with `ticket 10: needs-info` and say exactly what remains.

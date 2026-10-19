# Ticket 12: re-integrate now, then hand over

Thank you for stopping without the marker commit as asked. This message is not a review round; I will
review your item 1 to 3 commits together with the rebased result.

`effort/lead-worker-readiness` has moved: tickets 05 and 03 landed since your base. They changed
`bench/harness/lead_driven.py` and its test (scenario kinds `lead-launch` and the five acceptance
scenarios), added `tools/agents/lead_launch.py` and `tools/agents/acceptance.py`, and changed
`tools/agents/tracker.py` and `tools/agents/land.py`. Neither changed `tools/agents/worker.py`.

Do: rebase your own branch onto the current tip of `effort/lead-worker-readiness`, resolve conflicts by
intent, make `tools/agents/lead_launch.py` pass a `--runtime` option through to the worker launch (default
unchanged) and record it in the launch record, rerun the whole gate (the worker, lander, acceptance, tracker
and lead-launch suites; every bench harness suite; every scenario under `bench/readiness/`), and finish
with the final commit `ticket 12: ready for review` on a clean tree. Nothing will land ahead of you.

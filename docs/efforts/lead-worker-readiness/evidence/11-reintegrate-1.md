# Ticket 11: re-integrate now, then hand over

Thank you for holding the marker commit. This message is not a review round; I will review your
three-source collection together with the rebased result.

`effort/lead-worker-readiness` has moved: tickets 03 and 12 landed since your base. Ticket 12 changed
`tools/agents/lead_launch.py` (a `--runtime` option passed through and recorded) and
`tools/agents/worker.py`; ticket 03 added `tools/agents/acceptance.py`, changed `tools/agents/land.py`, and
added five acceptance scenarios to `bench/harness/lead_driven.py`. A ticket 16 and an amended ticket 04
were also added under the effort's tickets; your lint changes must accept them.

Do: rebase your own branch onto the current tip of `effort/lead-worker-readiness`, resolve conflicts by
intent, rerun the whole gate (the tracker, lead-launch, worker, lander and acceptance suites; every bench
harness suite; every scenario under `bench/readiness/`; `tracker.py lint` with no arguments so every effort
in the repository is linted), then run `python3 tools/agents/tracker.py findings lead-worker-readiness 04`
and paste its output into your report as evidence that this effort's real findings are collected. Finish
with the final commit `ticket 11: ready for review` on a clean tree. Nothing will land ahead of you.

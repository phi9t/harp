# Ticket 21: content accepted; re-integrate, then hand over

Reviewed Candidate: e3ad04a. I ran my own mutation check on a separate checkout of it: with
`tools/agents/land.py` replaced by a no-op, `land-exact-candidate` reports `fails` with the differing fields
named, and the acceptance scenarios stop with an error instead of passing; with `tools/agents/acceptance.py`
replaced, the acceptance scenarios stop with an error. A scripted participant run with `--proof live` is
recorded as `deterministic`, and the row lists the participant and the stories that ran. This message is
not a review round.

One small thing to fix while you are in there: in that last run the row's runtime column still said `codex`
although the binary was the scripted participant. Record the runtime as `scripted` whenever the harness
decided the proof is not live, and keep the requested runtime in a separate field.

`effort/lead-worker-readiness` has moved in two ways since your base:
- Ticket 19 landed. It changed `tools/agents/worker.py` (other agents' branch movements are a NOTE, the run
  record is updated under a lock, `status` writes nothing without a flag, launch validates before creating
  anything), `tools/agents/land.py` (acceptance read only from a committed tip, a record branch required for
  ticket landings, owner override and a gate required for `main`, landing records commit only themselves),
  `tools/agents/acceptance.py`, `tools/agents/lead_launch.py` (claims after workspaces), the harness module
  (git environment variables stripped) and `AGENTS.md`, and added the `shared-repository-safe` scenario.
- This effort's records were moved off the code branch (ticket 16's `move-records`). On the integration
  branch `docs/efforts/lead-worker-readiness/` no longer exists; the tickets, the spec and the evidence are
  on the branch `effort/lead-worker-readiness-records`, checked out at
  `/data02/home/philip.yang/workspace/corenius-lwr-records` (read-only for you). Your own report stays where
  it is in your tree, `docs/efforts/lead-worker-readiness/evidence/21-report.md`; after the rebase it will be
  the only file of that directory on your branch, and that is expected.

Do: rebase your own branch onto the current tip of `effort/lead-worker-readiness`; resolve conflicts by
intent (your scenario fixes must hold for ticket 19's new scenario too: the mutation check covers it; your
tracker change that ties "landed" to a landing record or integration history must work for an effort whose
records are on a record branch, run from the main checkout); rerun the whole gate (every
`tools/agents/test_*.py` suite except `test_sqlite_baseline.py`; every bench harness suite; every scenario
under `bench/readiness/`; `tracker.py lint`); update the report with the final core-set line and the list of
core stories that pass and fail; keep each `- to NN:` finding on one line; and finish with a new final commit
`ticket 21: ready for review` on a clean tree. Ticket 18 will not land before you.

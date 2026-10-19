# Lead review 1 of ticket 19: one consequence to close, and re-integrate in the same pass

Reviewed Candidate: 357a8c7. I re-ran three of the independent reviewer's reproductions against it in a
throwaway repository with a mainline checkout, an integration worktree and other agents' branches:
- another agent's commit and a new unrelated branch while a worker runs: verdict CLEAN, ready, the two
  movements reported as NOTE `shared-branch-moved`;
- a landing on `main` with no ticket and no gate: refused, exit 8, `main` unchanged;
- a hand-written untracked acceptance record in a third worktree: `acceptance refused: acceptance record
  missing`, the integration branch unchanged.
All three hold. The rest reads right.

## 1. An effort without a record branch can no longer land a ticket at all; say so and refuse cleanly (blocking)

The lander now reads the acceptance record from the committed tip of the record branch or, when the effort
names none, of the integration branch. For an effort without a record branch the second case cannot work:
the record names the Candidate, committing it on the integration branch moves the tip, and the Candidate is
then stale by the lander's own rule. That is the right outcome (it is why ticket 16 exists), but it must be
stated and not discovered:
- A ticket landing for an effort that names no record branch is refused before any gate runs, with its own
  message saying that ticket landings need the effort's records on a record branch and naming the command
  that moves them (`effort.py move-records <effort>`), and with its own exit code.
- The help of `land.py --ticket` and of `acceptance.py record` says the same, and no longer says anything
  about writing the record uncommitted.
- A test for the refusal, and the acceptance and landing scenarios run on an effort with a record branch.
If you see a sound way to support landing without a record branch, describe it in the report instead of
building it.

## 2. Re-integrate in the same pass

`effort/lead-worker-readiness` has moved: ticket 20 landed since your base. It changed
`tools/agents/worker.py` (resume keeps the launch boundary, session and process-group accounting, delivery
acknowledgement only on a worker-authored observation), `tools/agents/lead_launch.py` (one shared rework
counter, fresh-worker rework carries the launch profile) and `tools/agents/acceptance.py` (the counter file
`evidence/rework-NN.jsonl`). Rebase your own branch onto the current tip first, resolve conflicts by intent
(both of you changed how run records are written: keep your lock around every update, including ticket 20's
new writers), then do item 1 as new commits.

## Protocol

Rebase, then item 1, then the whole gate (every `tools/agents/test_*.py` suite except
`test_sqlite_baseline.py`; every bench harness suite; every scenario under `bench/readiness/`;
`tracker.py lint`). Update the report; keep each `- to NN:` finding on one line. Finish with a new final
commit `ticket 19: ready for review` on a clean tree. Tickets 18 and 21 will not land before you.

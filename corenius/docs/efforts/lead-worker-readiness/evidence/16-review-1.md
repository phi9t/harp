# Lead review 1 of ticket 16: one gap, and re-integrate in the same pass

Reviewed Candidate: 54e10bc. I ran your code read-only on the real repository: `effort.py status` from the
main checkout now lists every effort, with a per-effort problem where a record is unreadable, and no
failure of the whole command; the lookup fix from my note holds. I also ran the record branch end to end
in a temporary repository with a main checkout, an integration worktree, the record worktree and a worker
worktree: `move-records` moved every tracked record, a claim and an acceptance record were written from the
main checkout into the record worktree while a Candidate was outstanding, and that Candidate then landed
with no rebase. That is the ticket's central requirement and it works. The policy version bump is paired
with the manifest and the sync test passes.

## 1. Worker reports are stranded on the code branch after the move (blocking)

A worker writes its report at `docs/efforts/<effort>/evidence/NN-report.md` in its own worktree, so the
report is part of the Candidate and lands on the integration branch. In my run, after `move-records` and one
landing, the code branch again held `docs/efforts/demo/evidence/01-report.md`, the record branch did not,
and `tracker.py findings demo 02` listed the finding from the lead file but not the `- to 02:` line in that
report. So after the move: findings in reports no longer reach later briefs (ticket 11), claim sampling and
acceptance have to be pointed at a file on another branch, and the effort's directory exists on two branches
again, half and half.

Required: say where a worker's report lives once an effort has a record branch, and make every reader agree.
The arrangement I would accept without further discussion: the report stays in the Candidate (a worker can
only write in its own worktree), and at acceptance the tool that records the acceptance copies the report
at that exact Candidate (`git show <candidate>:<path>`) into the record branch's evidence directory, named
by ticket and round so earlier rounds are kept; findings, claim sampling and the acceptance record's claim
link then read the record-branch copy. A report present only on the code branch is still found by
`tracker.py findings` as a fallback and marked as not yet accepted. If you see a better arrangement, take
it and justify it in the report. Tests: the three-source findings test from ticket 11 run on a moved effort;
a landing after which the record branch holds the accepted report; the scenario case extended with it.

## 2. Re-integrate in the same pass

`effort/lead-worker-readiness` has moved: ticket 04 landed since your base. It changed
`tools/agents/acceptance.py` (reviews, rework requests, the shared effort lookup in every command),
`tools/agents/lead_launch.py` (a `rework` command), `tools/agents/worker.py` and the harness module, and
added the `rework-round` scenario. Rebase your own branch onto the current tip first, resolve conflicts by
intent (both of you changed how `acceptance.py` and `lead_launch.py` find the effort: keep one lookup, yours,
with ticket 04's commands using it), then do item 1 as new commits. Make the `rework` command and the
review and rework records work on a moved effort, with a test.

## 3. Small things

- Under `## For later tickets` add: `- to 14: before the live trial, this effort itself is to be moved with
  move-records at a moment when no worker is running; the lead does it.` and anything a lead must know
  about committing in the record worktree.
- State in the report what `move-records` does to worktrees of running workers that still contain the old
  effort directory (nothing, I expect), and what a worker launched after the move sees in its tree.

## Protocol

Rebase, then the fixes, then the whole gate (every `tools/agents/test_*.py` suite you touched or that
imports what you touched; every bench harness suite; every scenario under `bench/readiness/`;
`tracker.py lint`; `tools/test_sync_agentic_workflow.py`). Update the report. Finish with a new final commit
`ticket 16: ready for review` on a clean tree.

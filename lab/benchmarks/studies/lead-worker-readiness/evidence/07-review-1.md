# Lead review 1 of ticket 07: one defect, and re-integrate in the same pass

Reviewed Candidate: the tip of your branch at 11:44Z. The integration branch named in the effort record,
ticket landings that advance it by default, the refusal to land a ticket on the base branch without a
recorded owner override, and the behind-the-mainline status are right and stay.

## 1. The effort record is looked for only under `--repo`, and in the real layout it is not there (blocking)

An effort's directory is created on the effort's own branch and lives in that branch's worktree; the main
checkout, on the mainline, does not contain it until the owner moves the mainline. This effort is laid out
exactly so. I reproduced it: `main` in the main checkout; branch `effort/demo` in a second worktree holding
`docs/efforts/demo/EFFORT.md` with `Integration branch: effort/demo`; an accepted Candidate.
- `land.py <candidate> --repo <main checkout> --effort demo --ticket 01` answers
  `error: effort record .../r/docs/efforts/demo/EFFORT.md not found`, exit 64.
- `effort.py --repo <main checkout> status` prints nothing: the effort is invisible.
- The same two commands run from the integration worktree work.
The lander already reads the acceptance record from the target branch's checkout (ticket 03) and the launch
command already finds the integration branch's worktree (ticket 05); the effort record needs the same
treatment, with one twist: you need the record to learn the branch.

Required:
- Resolve an effort's record by looking in the given `--repo` first and then in every other worktree of
  the repository for `docs/efforts/<effort>/EFFORT.md`. If exactly one worktree has it, use it. If several
  have it, prefer the one whose checked-out branch equals the `Integration branch` the record names, and
  refuse with a clear message if that does not single one out. One helper, used by the lander, the launch
  command and `effort.py`.
- `effort.py status` lists efforts found in any worktree, each once, saying which worktree it was read from.
- Tests for the lander, the launch command and status in this layout, and add the layout to your scenario
  (expected: landing from the main checkout advances the integration branch, the main checkout and the
  mainline are untouched, status run from the main checkout shows the effort and its landed Candidate).

## 2. Re-integrate in the same pass

`effort/lead-worker-readiness` has moved: tickets 12 and 11 landed since your base. Both changed
`tools/agents/lead_launch.py` (a `--runtime` pass-through; findings from three sources in the brief) and
the harness module; 11 also changed `tools/agents/tracker.py`. Rebase your own branch onto the current tip
first, resolve conflicts by intent, then do item 1 as new commits.

## 3. Small things

- Add `stories = [29]` if your scenario definition lacks it after the rebase, and under
  `## For later tickets` add: `- to 16: the effort record, the tickets and the acceptance records are found by
  searching worktrees for the effort directory; keep that single lookup if bookkeeping moves to its own branch.`

## Protocol

Rebase, then the fixes, then the whole gate (the lander, acceptance, worker, tracker, lead-launch and effort
suites; every bench harness suite; every scenario under `bench/readiness/`; `tracker.py lint`). Update the
report. Finish with a new final commit `ticket 07: ready for review` on a clean tree. Ticket 04 is running
and will not land before you.

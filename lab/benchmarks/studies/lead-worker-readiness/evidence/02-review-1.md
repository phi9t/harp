# Lead review 1 of ticket 02: changes required before it can land

Reviewed Candidate: 955059a. The exact-Candidate resolution, the gate on a detached checkout, the stale
refusal with its own exit code and the removal of every rebase are right and stay. Three things must change.

## 1. Landing onto a branch that is checked out corrupts that checkout (blocking)

The lander now always moves the target with `git update-ref`. When the target branch is checked out in a
worktree (the normal case: the mainline in the main checkout, an integration branch in the lead's
worktree), the ref moves and the working files and index do not. I reproduced it with your lander: a
repository with `main` checked out, a candidate branch one commit ahead adding file `f`;
`land.py cand --no-push` exits 0, `main` points at the new commit, and `git status` in that checkout shows
`D  f` staged. The next commit made there would silently revert the landed work. The old lander avoided
this with `git merge --ff-only` in the worktree that has the target checked out; that helper was deleted.

This comes from an acceptance item of mine that was worded too broadly ("writes in a worktree it did not
create"). The rule that matters: the lander never writes in a worker's worktree and never rebases. The
target's own checkout must move together with its ref. Required:
- If the target branch is checked out in a worktree, advance it there with a fast-forward-only merge to the
  exact Candidate, so ref, index and files move together. If that checkout has tracked changes that
  overlap the landed range, refuse with the existing overlap code and change nothing (the existing overlap
  check looks only at the main checkout; make it look at whichever worktree has the target checked out).
- If the target is not checked out anywhere, keep the compare-and-swap `update-ref`.
- Tests for both: after landing onto a checked-out target, `git status` there is clean and the landed file
  exists; with an overlapping tracked change, landing is refused and nothing moves. Add the checked-out
  case to the `land-exact-candidate` scenario's verification.

## 2. Your Candidate is behind the integration branch; re-integrate (required to land)

`effort/lead-worker-readiness` has moved since you started: ticket 13 landed and changed
`bench/harness/lead_driven.py`, `bench/harness/test_lead_driven.py`, `bench/harness/results.py` and the
hand-off scenario definition. Under the rule you just implemented, your Candidate is stale. Rebase your own
branch onto the current tip of `effort/lead-worker-readiness` (this is the one rebase your brief allows),
resolve the conflicts by intent (keep ticket 13's result rows, report and notes commands; keep your landing
scenarios), and rerun the whole gate afterwards.

## 3. Declare the stories of your two scenarios (small)

Since ticket 13, each scenario definition carries `stories = [...]` and the report flags a scenario that
has none. Add them: `land-exact-candidate` covers 5 and 27; `land-stale-candidate` covers 28 and 30. Check
that `python3 bench/harness/lead_driven.py --report <dir>` lists both scenarios after a run.

## Protocol

Do item 2 first, then 1 and 3 as new commits. Update the report, and add to `## For later tickets` what
ticket 07 (integration branch) should know about landing onto a checked-out target. Finish with a new final
commit `ticket 02: ready for review` on a clean tree, then make no further commits. Your context is at
about 31%: if you cannot finish, commit what you have with `ticket 02: needs-info` and say exactly what
remains, so a fresh worker can take over.

# Lead review 1 of ticket 05: changes required before it can land

Reviewed Candidate: be64d24. The triage field in the tracker, the launchable set, the batch claim before any
branch is cut, the pinned sources in the launch record and the question command are right and stay. Three
things must change.

## 1. The launch command cannot be used when the integration branch has its own worktree (blocking)

`cmd_launch` resolves `--repo` with `worker.main_worktree()`, which always returns the main checkout, and
then requires that checkout to be on the integration branch. In the layout this effort and the case study
both use (mainline in the main checkout, the integration branch in a separate worktree) every launch is
refused. I reproduced it with your scenario's environment: a repository with `main` checked out, branch
`integ` in a second worktree, run from that worktree with `--repo . launch demo 02 --integration integ`:
`refused: repository worktree is on main, not integration branch integ`, exit 1. Your scenario passes only
because its disposable repository has a single checkout that is already on the integration branch.

Required: find the worktree that has the integration branch checked out (ticket 02, now landed, added a
helper for exactly this in the lander) and do the cleanliness check, the ticket reads and the claim commit
there. If the integration branch is checked out nowhere, refuse and say so. Never move or commit on the
mainline. Add the two-worktree layout as a participant of your scenario (expected: launched from the
integration tip, claim commit on the integration branch, main checkout and mainline untouched), and a unit
test for the "checked out nowhere" refusal.

## 2. Re-integrate (required to land)

`effort/lead-worker-readiness` has moved since you started: tickets 02 and 08 landed. Both changed
`bench/harness/lead_driven.py` and its test; 02 rewrote `tools/agents/land.py`; 08 changed
`tools/agents/worker.py` (access profile, new launch options, `GIT_OPTIONAL_LOCKS=0` for git probes). Rebase
your own branch onto the current tip (the one rebase your brief allows), resolve conflicts by intent (keep
their scenarios and scenario kinds; keep your `lead-launch` kind), and rerun the whole gate including every
scenario command (`handoff`, `land-exact-candidate`, `land-stale-candidate`, `access-profile`,
`launch-by-edges-and-triage`). The intermittent worker-suite failure you reported is very likely the
index-lock race ticket 08 fixed; after the rebase run the worker suite three times and report the result.

## 3. Small things

- `## For later tickets` must be one line per target ticket in the form `- to NN: text`. Split
  `- to 03/04: ...` into two lines; the launcher that feeds findings into briefs matches that form exactly.
- The launch command passes none of ticket 08's access-profile options through. After the rebase, pass
  `--network`, `--command-timeout` and `--external-add-path` through to the worker launch (defaults
  unchanged) and record them in the launch record.

## Protocol

Rebase first, then the fixes as new commits. Update the report. Finish with a new final commit
`ticket 05: ready for review` on a clean tree, then make no further commits.

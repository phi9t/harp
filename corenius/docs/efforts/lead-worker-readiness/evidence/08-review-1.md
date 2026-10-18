# Lead review 1 of ticket 08: re-integrate, then two small things

Reviewed Candidate: cfda301. On a separate checkout of that commit the worker suite (37 tests), every bench
harness suite and all four access-profile participants pass, and the sampled claims held. The access
profile, the generated instructions and the profile-driven findings are accepted as built.

## 1. Re-integrate (required to land)

`effort/lead-worker-readiness` has moved since you started: tickets 13 and 02 landed. Both changed
`bench/harness/lead_driven.py` and `bench/harness/test_lead_driven.py`; 13 added result rows, the report
and notes commands and `bench/harness/results.py`; 02 added the landing scenarios and changed
`tools/agents/land.py`. Rebase your own branch onto the current tip of `effort/lead-worker-readiness` (the
one rebase your brief allows), resolve conflicts by intent (keep their result rows, report, notes and
landing scenarios; keep your access-profile scenario and launcher changes), and rerun the whole gate,
including every scenario command (`handoff`, `land-exact-candidate`, `land-stale-candidate`,
`access-profile`).

## 2. Declare your scenario's stories

Since ticket 13 each scenario definition carries `stories = [...]`. Add `stories = [31, 32, 33, 36]` to the
access-profile scenario and check that `python3 bench/harness/lead_driven.py --report <dir>` lists it.

## 3. Explain one change in the report

You set `GIT_OPTIONAL_LOCKS=0` for every git call the toolbox makes. Say in the report why (what failed
without it, and how you know), and add one claim line citing it. If it was to stop the supervisor's
`git status` from taking the index lock in a worker's worktree, say so: that is a finding other tickets need,
so also add it under `## For later tickets` as `- to 06: ...`.

## Protocol

Rebase first, then 2 and 3 as new commits. Finish with a new final commit `ticket 08: ready for review` on
a clean tree, then make no further commits.

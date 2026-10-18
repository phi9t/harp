# 02 exact-candidate-lands: The reviewed commit is the commit that lands

Type: feature
Status: resolved
Owner: worker-02
Blocked by: 01
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 5, 27, 28, 30. Today the toolbox lander rebases the branch inside the worker's worktree and gates the rebased tip, so the landed commit is not the one that was reviewed, and the lead writes in the worker's workspace.

Change landing so that it takes an exact Candidate commit, runs the gate command on a separate checkout of exactly that commit, and fast-forwards the target branch to it. If the target has moved so that a fast-forward is impossible, landing is refused with a distinct exit code and message that a new Candidate on the new base is required; nothing is rebased and no worktree of the worker is touched. Producing that new Candidate is the worker's job (ticket 04's rework path), not the lander's.

Add the readiness scenarios: land an exact Candidate (expected: target tip equals the Candidate); stale base (expected: refused, target unchanged, worker worktree untouched).

## Declared paths
- `tools/agents/land.py`, `tools/agents/test_land.py`, `tools/agents/wt.py` if its `land` passthrough needs it
- `bench/readiness/` (two scenarios)

## Acceptance
- [x] Landing a Candidate leaves the target branch tip equal to the Candidate commit byte for byte; the gate ran on that commit (the gate sees it as `LAND_TIP`).
- [x] When the target has moved, landing exits with its own code, changes no ref, and leaves the worker's worktree and branch exactly as they were (checked by comparing `git status` and the branch tip before and after).
- [x] No code path in the lander runs a rebase or writes in a worktree it did not create.
- [x] A failing gate changes no ref.
- [x] The race test (two landings at once) still passes: exactly one lands and the other is refused as stale.
- [x] Both scenarios run in the lead-driven mode from ticket 01 and match their expectation.
- [x] The lander's and the worktree tool's existing tests pass, rewritten where they asserted rebase-on-land.

## Comments
- 2026-10-05T09:06:01Z worker-02: claimed until 2026-10-07T09:06:01Z
- 2026-10-05T09:51:11Z worker-02: resolved

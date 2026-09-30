# Lead verification notes: Milestone B Tier 1 fleet (2026-09-30)

What the lead checked, with citations, so `tools/agents/claims.py sample` can pick claims to re-verify. Tags: `[R]` read, `[X]` executed, `[I]` inferred.

1. [R] `land.py` pushes the exact tip to the remote before it moves local mainline: the push is at `tools/agents/land.py:235`.
2. [R] A rejected push is remembered and leads to a retry from the rebase: `tools/agents/land.py:237`.
3. [R] `wt.py finish` changes into the main worktree before removing a worktree, so it works when run from inside the one being removed: `tools/agents/wt.py:259`.
4. [R] `msg.py` refuses path components containing `..`, slashes or backslashes: `tools/agents/msg.py:24`.
5. [R] `claims.py` derives each claim's sampling rank from the seed, the report digest and the claim id: `tools/agents/claims.py:154`.
6. [R] `tracker.py` refuses to claim a ticket whose blockers are not resolved: `tools/agents/tracker.py:436`.
6b. [R] `tracker.py lint` reports a self blocker and an unknown blocker id: `tools/agents/tracker.py:282`.
7. [X] On the real repository `wt.py status` printed 36 rows and `git worktree list` printed 36 lines.
8. [X] `tools/agents/check.sh` exited 0 with the wt, tracker, msg, claims and land suites and was used as the gate of `land.py` itself.
9. [X] Eight independent `land.py` scenarios passed, including a pre-receive hook proving local mainline had not moved when the first push was rejected.
10. [I] Landing through the new tool did not touch the main checkout's seven uncommitted AIME files (status count stayed 7).
11. [R] The evidence worker's round 1 digest of the worktree diff ignored untracked files; this is why ticket 05 was sent back: `git -C ../corenius-worker-mb-evidence show 4ed6397:tools/agents/evidence.py`.

## Corrections

The first sampling run with `claims.py` (seed `fleet-1`) recorded two failures in my own notes: claim 6 cited `tracker.py:282`, which is the lint path, not the claim path (fixed above as 6 and 6b); claim 11 cited a file that does not exist in this tree (now cited as a `git show` of the worker's commit). The failed outcomes stay recorded; the corrected claims have new ids.

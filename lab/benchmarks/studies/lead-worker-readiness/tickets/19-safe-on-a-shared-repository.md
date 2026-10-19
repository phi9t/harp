# 19 safe-on-a-shared-repository: The tools are safe and correct in a repository other agents are using

Type: bug
Status: resolved
Owner: worker-19
Blocked by:
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md`. Source: `evidence/independent-review-1.md`, findings 1, 3, 5, 6, 7, 8, 14, 18, 22 (read each in full; most carry a reproduction). The independent reviewer ran every suite and every scenario green and still reproduced these in throwaway repositories. This repository has over a hundred worktrees and several agents committing at once; every item below is about that reality.

You may not run real model workers. Reproduce each finding first as a failing test in a throwaway repository that has: a mainline checkout, an integration worktree, a record worktree, at least two worker workspaces, and at least three unrelated branches that other "agents" commit to during the test.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/land.py`, `tools/agents/acceptance.py`, `tools/agents/lead_launch.py`, `tools/agents/tracker.py`, `tools/agents/effort.py` and their tests
- `bench/harness/lead_driven.py` (only for finding 18 and for scenarios), `bench/readiness/`
- `AGENTS.md` (the tools paragraph only)

## Acceptance
- [x] Finding 1: while a worker runs, a commit on an unrelated branch of the shared repository, a new unrelated branch, and the lead landing another ticket on the integration branch each leave the worker's verdict unaffected: reported at most as a NOTE that does not name the worker, never `VIOLATED`, never in `not_ready`. A worker moving a branch other than its own in its own repository is still a violation. A scenario shows a worker reaching ready while three other branches move.
- [x] Finding 3: where an effort's record is read from is decided by the record itself, not by the checkout a command runs in: the record branch if the effort names one, else the integration branch; a copy in another checkout is used only if that checkout has that branch checked out. Landing reads the acceptance record from the committed tip of that branch (an uncommitted or untracked record never authorizes a landing; say so in the help and remove the write-uncommitted-then-land text). A hand-written untracked `acceptance-NN.jsonl` in a third worktree does not authorize a landing. `tracker.py list`, `acceptance.py summary` and `effort.py status` give the same answer from every checkout.
- [x] Finding 5: recording a landing on a record branch commits only the landing record; an untracked draft and an uncommitted ticket edit in the record worktree are left as they were.
- [x] Finding 6: before pushing, the lander checks that the target checkout can fast-forward, including untracked files the Candidate would overwrite; if it cannot, nothing is pushed. If a push succeeded and the local fast-forward then fails, that is reported with its own exit code and message, not as a stale Candidate.
- [x] Finding 7: a landing on the effort's base branch, or on `main` when no effort is given, requires the owner override flag and records it; an effort record with no `Base:` line is treated as based on `main`; a landing with no gate is refused unless a flag says so explicitly.
- [x] Finding 8: a batch launch that fails after claiming leaves no ticket claimed without a running worker: workspaces are validated and created before the claim, or the claims of tickets that did not launch are released in the same command. Works with the default temp directory unusable (`TMPDIR` pointing at a read-only directory).
- [x] Finding 14: `worker.py status` writes nothing unless given a flag whose help says what it writes; `supervise`, `status` and `deliver-findings` update the run record under a lock so a concurrent pair cannot lose a write (test with two processes).
- [x] Finding 18: every git command the harness and the toolbox run has `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and `GIT_COMMON_DIR` removed from its environment; a scenario run with `GIT_DIR` pointing at a decoy repository leaves the decoy untouched.
- [x] Finding 22: `worker.py launch` validates every option before it creates anything; `--base` accepts a branch name and resolves it in the repository given by `--repo`; a refused launch leaves no clone and no run directory.
- [x] `AGENTS.md` describes the lander as it is (no rebase) and names `lead_launch.py` and `acceptance.py`.
- [x] Every existing scenario still matches its expectation; `tracker.py lint` is clean on every effort.

## Comments
- 2026-10-05T16:11:15Z worker-19: claimed until 2026-10-07T16:11:15Z
- 2026-10-05T18:47:27Z worker-19: resolved

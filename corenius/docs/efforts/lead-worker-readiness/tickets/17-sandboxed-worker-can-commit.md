# 17 sandboxed-worker-can-commit: A sandboxed non-interactive worker can commit its work

Type: bug
Status: resolved
Owner: worker-17
Blocked by: 12
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (stories 1, 2, 4). Decisions: `decisions/` beside it. Found by the lead on 2026-10-05 with the first real non-interactive worker run of the effort (`evidence/lead/notes.md`; the event stream is `evidence/17-real-codex-worker-probe-events.jsonl`).

The decided hand-off is that a worker commits in its own workspace and hands over an exact Candidate. `worker.py launch` puts the worker in a git worktree and starts the runtime with the `workspace-write` sandbox. A worktree's index and branch data live in the main repository's `.git/worktrees/<name>` and its objects in the main `.git/objects`, all outside the sandbox's writable root. The real run:

    worker.py launch --runtime codex --allow result.txt --task <create result.txt containing "ok", git add it, git commit>
    ...
    git add result.txt && git commit -m "add result"
    fatal: Unable to create '<main repo>/.git/worktrees/<name>/index.lock': Read-only file system

The supervisor judged it correctly (exit 0, not ready: dirty tree, no commit after base). Every test until now used a fake binary that is not sandboxed, so eleven tickets landed without this being seen.

Make a sandboxed worker able to commit, without giving it write access to the main repository's git data or to any shared branch. Choose the mechanism and justify it in the report. One arrangement that fits the spec and the local controller's design ("private Git metadata per attempt"): the worker's workspace is a private clone whose `.git` is inside the workspace, created from the base commit without copying the object store where that can be avoided; the worker commits there; at hand-off the lead side fetches the worker's branch into the main repository under the worker's branch name, and from then on the Candidate, the lander and the overlap report work as today. Whatever you choose, the lead must still never write in the worker's workspace, and a worker must still be unable to move any branch but its own.

The interactive mode the owner uses (an unsandboxed session in a tmux window) must keep working in whatever workspace kind you choose; say in the report what changes for it.

You may make real model runs for this ticket, and only these: the probe above with `--runtime codex --model gpt-5.5 --effort low`, and the same with the default runtime, each at most three times, each in a throwaway repository under a fresh `mktemp -d`. Record each run's verdict and keep one event stream per runtime as evidence. This is the one exception to the rule in your brief about not launching model workers.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/lead_launch.py`, `tools/agents/land.py`, `tools/agents/wt.py` and their tests
- `bench/harness/lead_driven.py`, `bench/readiness/` (scenarios)
- `docs/practice/lead-supervisor-worker.md`, `.claude/skills/run-worker/SKILL.md`

## Acceptance
- [x] The real probe (create one file with the edit tool, `git add`, `git commit`) run through `worker.py launch` and `supervise` under the default sandbox ends ready for review with a pinned Candidate, for the Codex runtime and for the traecli runtime; both event streams are in `evidence/`.
- [x] In that run the main repository's refs other than the worker's own branch, its index and its checked-out files are unchanged; a test with a scripted participant that tries to move another branch or write the main repository's git data shows it cannot or that it is reported.
- [x] The Candidate handed over can be gated and landed by the existing lander with an acceptance record, from the main checkout, in the layout with a mainline checkout, an integration worktree and several worker workspaces.
- [x] The marker hand-off, Candidate pinning, `candidate-moved`, the rework successor, resume and the overlap report still work with the new workspace kind; every existing scenario matches its expectation.
- [x] A worker launched from an integration branch tip starts from exactly that commit, and a worker asked to re-integrate onto a moved tip can do so from inside its sandbox.
- [x] Disk: the report states the size of a worker workspace before and after this change for this repository.
- [x] The practice note and the run-worker skill describe the workspace kind and why.

## Comments
- 2026-10-05T13:48:25Z worker-17: claimed until 2026-10-07T13:48:25Z
- 2026-10-05T15:58:14Z worker-17: resolved

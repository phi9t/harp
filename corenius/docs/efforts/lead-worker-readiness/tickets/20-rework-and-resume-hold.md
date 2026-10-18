# 20 rework-and-resume-hold: Rework rounds are capped everywhere and a resumed worker keeps its boundary

Type: bug
Status: resolved
Owner: worker-20
Blocked by:
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (stories 12, 13, 16, 40). Source: `evidence/independent-review-1.md`, findings 2, 4, 10, 11 (read each in full).

This ticket may make real model runs, and only these: one launch followed by one resume per runtime (`--runtime codex` and `--runtime traecli`, `--model gpt-5.5 --effort low`), at most three attempts each, each in a throwaway repository under a fresh directory on the same filesystem as your worktree. The task for the launch: create `a.txt` containing `one` and commit it. The resume message: create `b.txt` containing `two` and commit it. This permission is part of the ticket and overrides any general line in your brief about not running model workers.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/lead_launch.py`, `tools/agents/acceptance.py` and their tests
- `bench/harness/lead_driven.py`, `bench/readiness/` (scenarios)

## Acceptance
- [x] Finding 2: there is one review-round counter per ticket, kept with the effort's records, and every path that starts a review-driven round consults and advances it: `acceptance.py request-rework`, `worker.py deliver-findings`, `lead_launch.py rework` (same session and fresh worker). A fourth review-driven round is refused on each of the three commands and sets the ticket's triage to `ready-for-human`. A stale-base redo does not advance it. A test makes four consecutive `lead_launch.py rework` calls and four mixed calls.
- [x] Finding 2 and 11: `acknowledged` is true only on an observation that could not have been produced by the sender: a worker event or commit that follows the delivery and references the findings file's digest or path in a worker-authored event, or a new marker commit. The lead's own typed message appearing in a pane, an unrelated event from a busy worker, a resume process that merely started, and a runtime that exits non-zero are each `acknowledged: false` with the reason. Tests for each of those four.
- [x] Finding 4: a resumed turn runs under the same boundary as the launch: the same sandbox mode, the same working directory, the worker's private git directory writable, and inside the capsule when the launch was in one. Where a runtime's `resume` command has no flag for this, it is passed through configuration overrides; where it cannot be done at all, resume for that runtime and mode is refused with a message, not run unbounded. A fresh-worker rework carries the whole launch profile (sandbox, hand-off mode, protected paths, external protected paths, capsule, access profile).
- [x] Finding 4, real proof: for each runtime, the real launch-then-resume above ends with both commits on the worker's branch, verdict ready, and the Candidate imported; the two event streams per runtime are in `evidence/` and the verdict lines are in the report. If a runtime cannot do it under `workspace-write`, stop with `needs-info` and the exact error.
- [x] Finding 10: a worker is reported stopped only when no process of its session is alive: the run records the session or process group, the wall-clock wrapper does not detach from it, and `account` after a scenario shows no surviving process (the test checks the process table, not the run record).
- [x] Every existing scenario still matches its expectation.

## Comments
- 2026-10-05T16:11:15Z worker-20: claimed until 2026-10-07T16:11:15Z
- 2026-10-05T17:12:31Z worker-20: resolved

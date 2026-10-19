# 09 external-state-report: Protected state outside the repository is recorded before launch and reported after

Type: feature
Status: resolved
Owner: worker-09
Blocked by: 08
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 34, 35. The launcher takes declared protected paths outside the repository (files or directories) and records a baseline before the worker process starts: every entry beneath each path with its kind, size, modification time, symlink target (never followed) and, for regular files up to a stated size, a content digest. While the worker runs the supervisor compares cheaply (no hashing); after it exits it compares fully. The report lists entries modified, removed, and added outside the add-paths declared in ticket 08. It says the entry changed during the run and does not say who changed it.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/test_worker.py`
- `bench/readiness/` (scenarios using a run-owned directory as the stand-in for external state)

## Acceptance
- [x] A missing, relative or in-repository protected path is refused before any worktree or branch is created.
- [x] The baseline file's timestamp precedes the worker process's start time in the run record.
- [x] Modifying, removing and adding an entry each produce a report line with correct counts; an addition under a declared add-path is not reported.
- [x] A content change that keeps size and restores the modification time is caught by the final comparison.
- [x] A symlink beneath a protected path is recorded by target and not followed.
- [x] The report wording states a change during the run and never names the worker as the cause; a test asserts the wording.
- [x] All scenarios run in the lead-driven mode and match their expectation.

## Comments
- 2026-10-05T10:04:15Z worker-09: claimed until 2026-10-07T10:04:15Z
- 2026-10-05T10:25:21Z worker-09: resolved

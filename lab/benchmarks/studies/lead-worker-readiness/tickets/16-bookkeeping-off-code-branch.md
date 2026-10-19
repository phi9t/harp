# 16 bookkeeping-off-code-branch: Effort bookkeeping does not make a running Candidate stale

Type: feature
Status: resolved
Owner: worker-16
Blocked by: 03, 05, 07
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md`. Decisions: `decisions/` beside it. Added during the run from the lead's notes (`evidence/lead/notes.md`): the same root cause was worked around four times in three hours.

Today the tracker's ticket files, the acceptance records and the lead's notes are tracked files on the integration branch, and a Candidate lands only by fast-forward. So every piece of bookkeeping the lead commits while a worker runs (a claim, a resolve, an acceptance record, a note) moves the tip and makes every running Candidate stale, although no code changed. The lead currently avoids this by batching claims, deferring claims, writing acceptance records uncommitted before landing, and holding all other commits for moments when no worker runs. With three parallel workers those moments barely exist.

Make bookkeeping independent of the branch that code lands on. The observable requirement is below; choose the mechanism (for example a separate record branch with its own worktree that the tracker, the acceptance tool and the launch command read and write, with the brief carrying the pinned ticket and spec text so a worker needs neither in its tree), say in the report why you chose it and what you rejected, and keep every existing command working for an effort that has not been moved.

## Declared paths
- `tools/agents/tracker.py`, `tools/agents/acceptance.py`, `tools/agents/lead_launch.py`, `tools/agents/land.py`, `tools/agents/effort.py` and their tests
- `bench/harness/lead_driven.py`, `bench/readiness/` (scenarios)
- `docs/agents/issue-tracker.md` only if the documented layout changes (bump its version header and the manifest together, and `tools/test_sync_agentic_workflow.py` must pass)

## Acceptance
- [x] Scenario: a worker is launched; while it runs the lead claims a second ticket, resolves a third and records an acceptance for a fourth, all committed; the first worker's Candidate then lands without any rebase.
- [x] Scenario: an acceptance record is committed before its Candidate lands, and the landing succeeds; the write-uncommitted-then-land order is no longer required, and the help texts that describe it are corrected.
- [x] Launching a batch of three tickets leaves all three workers on the same base, equal to the code tip at launch.
- [x] The effort's history still shows, for each ticket, who claimed it, each review round, the accepted Candidate and when it landed, readable from a fresh clone.
- [x] A ticket's brief contains the exact ticket and spec text it was launched with, by content digest, whether or not those files exist in the worker's tree.
- [x] An effort that has not been moved to the new arrangement works exactly as before; a command moves an existing effort and reports what it did.
- [x] Every existing scenario still matches its expectation; tracker lint is clean on every effort in the repository.

## Comments
- 2026-10-05T12:15:03Z worker-16: claimed until 2026-10-07T12:15:03Z
- 2026-10-05T13:48:21Z worker-16: resolved

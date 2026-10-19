# Repair round 3: atomic refresh reservation

Base read: `795b5714dd752d37d26814730db03ce272619eab`.
Branch: `codex/collab-ops-atomic-refresh`.
Worktree: `/data02/home/philip.yang/workspace/corenius-codex-collab-ops-atomic-refresh`.
Scope: the independent review's Medium finding at `refresh.py:38`.
This is the final approved repair iteration, round 3 of 3.
The approved test seam is the public collab CLI in real disposable Git repositories.

## Failure and repair

Refresh committed abandonment before it reserved the replacement Attempt.
A crash in that gap left a Corenius tracker claim without an active Attempt.
The old Attempt could not refresh, and ordinary admission rejected the existing claim.

`StateStore.reserve_refresh` now writes both Attempt transitions and their journals in one transaction.
It checks that the original admission still matches the supplied snapshot.
The transaction marks the old Attempt abandoned, reserves the new allocating Attempt, and records the contribution replay list.
Core validates the current trunk and owned claim before that transaction.
Worktree allocation and cherry-picks run after commit; no database transaction covers Git execution.
All SQLite access remains in the state store.

If the process dies before the replacement reservation commits, SQLite rolls back abandonment and the refresh journal.
The old Attempt remains active with its tracker claim, branch, and worktree intact.
If the reservation commits before the process dies, the replacement is already active and carries its allocation and replay journals.
Existing Corenius allocation recovery adopts that exact transferred claim.
The old contribution remains on its original branch and worktree.

## Red and green evidence

The following literal command first exited 1: both adapter cases failed in 1.141 seconds.

```text
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_atomic_refresh.py -v
```

The test-only `after_refresh_abandon` failpoint first reproduced the committed gap.
After process death, both tests observed `abandoned` instead of the expected active `working` Attempt.
The fix moved that failpoint inside the atomic transaction, immediately after abandonment and before replacement insertion.
After the fix, the same command exited 0: both tests passed in 2.465 seconds.

Both tests prove that ticket bytes and original contribution refs remain unchanged.
They run `recover` on the original Attempt, retry refresh, and verify the contribution on current trunk.
They also verify that exactly one active Attempt remains after the successful retry.
The tests use public `show`, `status`, `recover`, and `refresh` commands; they never query SQLite directly.

A third case covers process death after the replacement reservation commits, using the existing `after_claim_record` failpoint.
It verifies one active allocating replacement, the old abandoned Attempt, unchanged tracker bytes, and a replay journal naming the original contribution.
Recovery creates the replacement worktree while retaining the original contribution.
It does not silently replay or resolve commits during allocation recovery.

The final focused command exited 0: three tests passed in 3.602 seconds.

```text
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_atomic_refresh.py -v
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_refresh_source.py -v
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_lifecycle_ops.py -k transferred -v
python3 tools/temp_root.py -- python3 -m py_compile tools/collab/refresh.py tools/collab/core.py tools/collab/state_store.py tools/collab/tests/test_atomic_refresh.py
python3 tools/temp_root.py -- git diff --check
```

The existing refresh/source suite passed: ten tests in 6.938 seconds.
The existing transferred-claim recovery case passed: one test in 1.066 seconds.
Compilation and diff checks passed.
The changed Python files have no lines longer than 120 columns.

## Authority and remaining verification

Only refresh admission, its narrow state-store transaction, the new CLI regression file, and this report changed.
No state-store lock or latest-event implementation changed.
No old owned worktree, shared branch, or salvage archive was changed.
The lead owns integration and the final affected-source checks and independent re-review.
No repository-wide gate or aggregate acceptance is claimed by this report.

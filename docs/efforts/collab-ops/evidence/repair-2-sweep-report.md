# Repair round 2: sweep ownership after archival

Base read: `09ced6dfc0cebbeb2edac3e36b3ce9bc295f8d65`.
Branch: `codex/collab-ops-sweep-repair`.
Worktree: `/data02/home/philip.yang/workspace/corenius-codex-collab-ops-sweep-repair`.
Scope: the independent review's Medium finding at `sweep.py:136`, plus the lead's check-lock requirement.
This is repair round 2 of the approved maximum of 3.
The approved seam is the public collab CLI in real disposable Git repositories.

## Failure and repair

A terminal dirty worktree was classified as unlocked before archival.
An owner could lock it while archival ran.
The sweep then restored tracked files and deleted untracked and ignored files before Git removal noticed the lock.
The removal failed, but the owner had already lost its workspace bytes.

Sweep now reads the current Attempt owner, state, branch, path, worker liveness, Git registration lock, and exact HEAD before destructive cleanup.
It repeats these checks after archival and immediately before clearing dirty content and removing the worktree.
The archive records its manifest hash when created.
Before clearing content, sweep verifies the owned archive location, manifest hash, subject, artifact hashes, complete source inventory, and original index bytes.
A new owner, lock, live or unknown worker, moved workspace, or changed recovery copy causes refusal before clearing content.

Destructive sweep acquires the global check execution lock before the lane and metadata locks.
An active check process therefore prevents archive or removal work.
Dry-run sweep retains its existing lock behavior.

## Red and green evidence

The first literal command exited 1 with one failing test:

```text
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_sweep_repair.py -v
```

The test paused the real Git subprocess during bundle verification, acquired an external owner lock, and resumed sweep.
Sweep returned exit code 2 because removal saw the lock, but `code.txt` contained `base\n` instead of the owner's unsaved bytes.
After ownership and archive revalidation, the same command exited 0: one test passed in 1.441 seconds.
It also checks the staged version, untracked and ignored binary files, branch HEAD, worktree registration, and new lock reason.

After adding the check-lock regression, the same command exited 1 with one failure and one pass.
The failing test kept a real gate process running while a second terminal dirty Attempt was swept.
Sweep returned exit code 0 and removed that second workspace.
After the execution-lock change, the same command exited 0: two tests passed in 2.198 seconds.

The final regression suite passed: three tests in 3.732 seconds.
The third test pauses after archive restoration verification, corrupts the archive tar, and confirms refusal with the original workspace bytes retained.
The race tests use subprocess barriers around real Git commands; they do not replace collab modules or query private storage.

```text
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_sweep_repair.py -v
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_cleanup.py -v
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_collab.py -k sweep -v
python3 tools/temp_root.py -- python3 -m py_compile tools/collab/sweep.py tools/collab/archives.py tools/collab/tests/test_sweep_repair.py
python3 tools/temp_root.py -- git diff --check
```

The existing cleanup suite passed: nine tests in 4.731 seconds.
The existing sweep cases passed: six tests in 2.038 seconds.
Compilation and diff checks passed.
The changed Python files have no lines longer than 120 columns.

## Authority and remaining verification

Only `sweep.py`, `archives.py`, the new public-CLI test file, and this report changed.
No shared checkout, salvage archive, or retained run artifact was cleaned or changed.
The lead owns integration, the repository gate, and independent re-review.
This focused evidence does not claim aggregate acceptance or a completed review round.

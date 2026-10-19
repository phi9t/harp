# Ticket 09 Liveness Inspect Report

## Built

- Added `--worker-pid` and `--worker-run-dir` to `open-attempt`; Attempt rows
  now retain optional worker process metadata and old databases migrate the new
  nullable columns on load.
- Added worker liveness checks for recorded PIDs and `worker.py` run
  directories. Run-directory checks prefer live process-session membership,
  then `exit_code`, then the recorded PID.
- `recover` and `abandon` now refuse even with `--confirm-stopped` while the
  recorded worker is still alive, and recheck after taking lifecycle locks.
- Added `inspect <attempt>` to return structured blockers for live recorded
  workers, busy collab locks, missing or substituted worktrees, hidden index
  flags, tracked/untracked files and ignored files.
- Updated `tools/collab/README.md` for launcher liveness metadata and the new
  inspect command.

## Acceptance

- [x] recover and abandon refuse while the recorded worker runs. Covered by
  `test_recover_and_abandon_refuse_live_recorded_worker_then_proceed_after_exit`
  using real child processes.
- [x] They proceed once it has exited and `--confirm-stopped` is given. The same
  test terminates the recorded PID/run-directory workers, writes the launcher
  `exit_code` for the run-directory case, then verifies `recover` reaches
  `working` and `abandon` reaches `abandoned`.
- [x] inspect names hidden index flags, ignored files and locks that block a
  change. Covered by
  `test_inspect_names_hidden_index_ignored_files_and_busy_locks`, which asserts
  `hidden_index`, `ignored_file` and `lock` blockers from the public CLI.

## Claims

- Fresh red evidence before implementation:
  `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_recover_and_abandon_refuse_live_recorded_worker_then_proceed_after_exit tools.collab.tests.test_collab.CollabTests.test_inspect_names_hidden_index_ignored_files_and_busy_locks -v`
  failed because `--worker-pid` was unrecognized and `inspect` was an invalid
  command.
- Focused post-change slice passed:
  `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_recover_and_abandon_refuse_live_recorded_worker_then_proceed_after_exit tools.collab.tests.test_collab.CollabTests.test_inspect_names_hidden_index_ignored_files_and_busy_locks tools.collab.tests.test_collab.CollabStructureTests.test_collab_toolset_has_named_modules_and_single_sqlite_owner -v`
  passed: 3 tests in 2.110s.
- Full public collab CLI suite passed:
  `python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v`
  passed: 66 tests in 93.244s.
- Registered collab gate passed:
  `python3 tools/temp_root.py -- timeout 120s sh tools/agents/checks.d/65-collab.sh`
  passed: 66 tests in 89.700s.
- Diff hygiene passed:
  `python3 tools/temp_root.py -- git diff --check`.
- Broader repository gate was attempted under the per-command cap with
  `CHECK_CARGO=0 python3 tools/temp_root.py -- timeout 120s sh tools/agents/check.sh`;
  it exited 124 in `tools/agents/checks.d/15-worker.sh` before reaching the
  collab gate.

## Outside declared paths

- None. Changes are under `tools/collab/` plus this report.

## For later tickets
none

## Not checked

- Full `tools/agents/check.sh` completion was not checked because the
  per-command 120-second cap expired during `15-worker.sh`.
- Cargo tests were not run; the effort spec notes the native controller tests
  currently have an unrelated environment blocker, and this worker ran the
  broader gate with `CHECK_CARGO=0`.
- No network, push, merge, rebase, container build or global package install was
  performed.

# Ticket 11 Report

## Built
- Added `collab sweep`, dry-run by default, with `--apply` for cleanup.
- Added `tools/collab/sweep.py` to classify clean terminal Attempt worktrees, skip fresh/in-progress/locked worktrees, remove merged terminal worktrees and branches, and archive clean unmerged terminal branch heads under `refs/collab/archive/` before removal.
- Added foreign unmerged branch reporting with owner prefix and no mutation.
- Updated the collab README cleanup section for the new sweep behavior and current tier limits.
- Added public CLI tests using disposable real Git repositories for the ticket acceptance cases.

## Acceptance
- [x] Dry run removes nothing and lists every candidate with its reason.
  - `test_sweep_dry_run_lists_candidates_without_removing`
- [x] A freshly opened Attempt survives `--apply`.
  - `test_sweep_apply_never_removes_fresh_attempt`
- [x] A locked worktree is reported as skipped, not failed.
  - `test_sweep_reports_locked_terminal_worktree_as_skipped`
- [x] An unmerged clean branch is archived before removal and its commits stay reachable.
  - `test_sweep_archives_unmerged_clean_terminal_branch_before_removal`
- [x] A foreign unmerged branch is listed with its owner and left alone.
  - `test_sweep_lists_foreign_unmerged_branch_with_owner_and_leaves_it`

## Claims
- Red-green evidence:
  - First red run: `python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_sweep_dry_run_lists_candidates_without_removing -v` failed because `sweep` was not a valid command.
  - Focused sweep run: `python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_sweep_dry_run_lists_candidates_without_removing tools.collab.tests.test_collab.CollabTests.test_sweep_apply_removes_merged_terminal_attempt tools.collab.tests.test_collab.CollabTests.test_sweep_apply_never_removes_fresh_attempt tools.collab.tests.test_collab.CollabTests.test_sweep_reports_locked_terminal_worktree_as_skipped tools.collab.tests.test_collab.CollabTests.test_sweep_archives_unmerged_clean_terminal_branch_before_removal tools.collab.tests.test_collab.CollabTests.test_sweep_lists_foreign_unmerged_branch_with_owner_and_leaves_it -v` passed, 6 tests in 5.095s.
- Narrow suite:
  - `python3 -m unittest discover -s tools/collab/tests -v` passed, 70 tests in 91.927s.
- Registered collab gate:
  - `sh tools/agents/checks.d/65-collab.sh` passed, 70 tests in 96.545s.
- Diff hygiene:
  - `git diff --check -- tools/collab` passed.
- Broad gate attempt:
  - `timeout 115s sh tools/agents/check.sh` exited 124 at the enforced command cap while still in `tools/agents/checks.d/15-worker.sh`.
  - Before timeout, the gate had passed `tools/test_sync_agentic_workflow.py`, `00-temp-paths.sh`, `05-gate-dispatch.sh`, and `10-wt.sh`; no failure from this ticket's collab checks was observed because the run did not reach `65-collab.sh` before timeout.

## Outside declared paths
- None beyond this allowed ticket report path.

## Integration notes
- Dirty-worktree archival, stale collab ref pruning, old check log pruning, orphaned check-worktree pruning, and explicit foreign-branch cleanup flags remain for later sweep tiers.
- The full repository gate is longer than this worker's 120-second command limit on this host; use a supervisor-side or longer-running gate when that evidence is required.

## Not checked
- Did not push, merge, rebase, or run container builds.
- Did not run the full `tools/agents/check.sh` to completion because of the per-command limit; the ticket-relevant registered collab gate passed separately.

## For later tickets
none

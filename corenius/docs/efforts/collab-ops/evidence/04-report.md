# Ticket 04 Check Runner Report

## Built

- Added `--check-timeout-seconds` to `collab init`; the value is optional, finite,
  positive, and stored in collab configuration.
- Split check execution into prepare, run, and receipt-publish phases. Prepare
  and receipt publication take the lane lock briefly; the configured command
  runs under a separate live-check lock so landings are not blocked for the
  length of a check.
- The check command writes directly to the durable log while running, terminates
  the process group on timeout, and records timeout as a distinct Check receipt
  outcome.
- Check receipts are v2 and record `argv`, `outcome`, `timeout_seconds`,
  `log_sha256`, subject, base, Candidate, check path, log path, trunk, and
  inspection conditions.
- Landing now requires a matching v2 passed receipt, including the configured
  timeout, before it can fast-forward.
- Updated `tools/collab/README.md` for the timeout option, v2 receipt behavior,
  streamed logs, and the live-check lock.

## Acceptance

- A hung check ends as a distinct timeout outcome:
  `test_hung_check_times_out_with_distinct_receipt_and_live_log` verifies
  `state=check_failed`, `outcome=timeout`, `exit_code=null`,
  `timeout_seconds=0.2`, and a matching log hash.
- The log is readable while the check runs:
  `test_live_check_log_is_readable_before_command_exits` starts a live check,
  reads the stored log before release, and observes streamed output.
- The receipt records argv, outcome and log hash:
  `test_check_receipt_is_durable_with_verified_state`,
  `test_hung_check_times_out_with_distinct_receipt_and_live_log`, and existing
  receipt tamper tests verify v2 receipt fields and log hash validation.
- Another landing is not blocked for the length of a check:
  `test_running_check_does_not_hold_landing_lane_lock` lands one verified
  Attempt while another check command is still running.
- A dirty workspace, changed HEAD or moved trunk still fails the check:
  `test_failing_mutating_and_head_moving_checks_never_verify`,
  `test_zero_exit_with_trunk_movement_is_check_failed`, and hidden-index check
  tests pass with explicit non-passing outcomes.

## Claims

- Fresh red evidence before implementation:
  `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_check_receipt_is_durable_with_verified_state -v`
  failed with `KeyError: 'version'`.
- Fresh red evidence before implementation:
  `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_hung_check_times_out_with_distinct_receipt_and_live_log -v`
  failed because `--check-timeout-seconds` was unrecognized.
- Focused post-change slice passed:
  `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_check_receipt_is_durable_with_verified_state tools.collab.tests.test_collab.CollabTests.test_hung_check_times_out_with_distinct_receipt_and_live_log tools.collab.tests.test_collab.CollabTests.test_live_check_log_is_readable_before_command_exits tools.collab.tests.test_collab.CollabTests.test_running_check_does_not_hold_landing_lane_lock tools.collab.tests.test_collab.CollabTests.test_live_check_child_retains_check_lock_after_cli_parent_death tools.collab.tests.test_collab.CollabTests.test_failing_mutating_and_head_moving_checks_never_verify tools.collab.tests.test_collab.CollabTests.test_zero_exit_with_trunk_movement_is_check_failed -v`
  passed: 7 tests in 11.933s.
- Structure checks passed:
  `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabStructureTests -v`
  passed: 2 tests in 0.002s.
- Registered collab gate passed on the final tree:
  `python3 tools/temp_root.py -- sh tools/agents/checks.d/65-collab.sh`
  passed: 54 tests in 76.104s.
- Broader repository gate was attempted with the worker's command limit guarded:
  `python3 tools/temp_root.py -- timeout 115s sh tools/agents/check.sh`
  exited 124 after the timeout guard while running the worker suite; no final
  repository-gate verdict was produced.

## Outside declared paths

- None. Changes are under `tools/collab/` plus this report.

## For later tickets
none

## Not checked

- Full `tools/agents/check.sh` completion was not checked because it exceeded
  the 115-second guard used to satisfy the worker command limit.
- Cargo tests did not run to completion separately.
- No network, push, merge, rebase, container build, or global package install was
  performed.

### Worker notes for later work (not addressed to a ticket)
- Check receipts now include an explicit `outcome` string that landing can treat
  as the stable v2 receipt contract.
- The new live-check lock is global for checks, matching the existing single
  lane model. If later work introduces parallel check lanes, that lock can be
  narrowed without changing the receipt contract.

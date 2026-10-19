# Ticket 05 Landing V2 Report

## Built

- Added `land --expected-tip` so the operator can pin the reviewed Candidate and refuse a moved Candidate branch before publication.
- Changed default `land` to require a configured trunk upstream, verify the local fast-forward and final operator-file overlap first, record push intent, push the exact Integration commit, and then fast-forward the primary checkout.
- Added `land --no-push` for explicit local-only/offline landing.
- Added durable landing phases for `push_intended`, `push_completed`, and `landed_records_incomplete`; `recover --confirm-stopped` can finish a pushed landing after a crash before the local fast-forward.
- Classified push failures as `authentication`, `network`, `non-fast-forward`, or `unknown` and restored the Attempt to `verified` when the push did not land.
- Reported post-landing record failures as a successful local landing with incomplete records and a recovery path.
- Updated the collab README for the new landing and recovery contract.

## Acceptance

- A crash between push and local fast-forward is recovered by recover and never leaves the remote ahead without a record: `test_default_landing_pushes_and_recover_completes_after_push_crash` verifies `push_completed`, remote Integration, local base, and recovery to `landed`; `test_recover_completes_after_pushed_landing_started_crash` covers the pushed `landing_started` crash point.
- A moved Candidate is refused against an expected tip: `test_land_refuses_moved_candidate_branch_against_expected_tip` moves the worker branch after check and verifies local and remote remain at base.
- Authentication, network and non-fast-forward push failures are reported distinctly: `test_push_failure_reports_authentication_and_network_distinctly` drives fake SSH authentication and network failures; `test_push_failure_reports_non_fast_forward_without_local_landing` drives a real non-fast-forward file remote.
- An ignored or untracked collision that appears during a long check is caught at landing: `test_ignored_collision_created_during_check_is_caught_before_push` creates ignored operator content after the check starts and verifies no push or local merge happens.
- `--no-push` lands locally only: `test_no_push_lands_locally_only_when_upstream_exists` verifies local HEAD advances while the upstream remains at base.
- A failed post-landing record write is reported as landed, records incomplete: `test_post_landing_record_failure_reports_landed_records_incomplete` blocks the landed ref, verifies `landed_records_incomplete`, then removes the blocker and recovers to `landed`.

## Claims

- Baseline before changes passed: `python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v` ran 64 tests in 87.451s.
- Fresh red evidence before implementation: `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_default_landing_pushes_and_recover_completes_after_push_crash tools.collab.tests.test_collab.CollabTests.test_land_refuses_moved_candidate_branch_against_expected_tip tools.collab.tests.test_collab.CollabTests.test_push_failure_reports_non_fast_forward_without_local_landing tools.collab.tests.test_collab.CollabTests.test_push_failure_reports_authentication_and_network_distinctly tools.collab.tests.test_collab.CollabTests.test_ignored_collision_created_during_check_is_caught_before_push tools.collab.tests.test_collab.CollabTests.test_no_push_lands_locally_only_when_upstream_exists tools.collab.tests.test_collab.CollabTests.test_post_landing_record_failure_reports_landed_records_incomplete -v` failed with unrecognized `--expected-tip` and `--no-push`.
- Focused post-change slice passed: `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_default_landing_pushes_and_recover_completes_after_push_crash tools.collab.tests.test_collab.CollabTests.test_land_refuses_moved_candidate_branch_against_expected_tip tools.collab.tests.test_collab.CollabTests.test_push_failure_reports_non_fast_forward_without_local_landing tools.collab.tests.test_collab.CollabTests.test_push_failure_reports_authentication_and_network_distinctly tools.collab.tests.test_collab.CollabTests.test_ignored_collision_created_during_check_is_caught_before_push tools.collab.tests.test_collab.CollabTests.test_no_push_lands_locally_only_when_upstream_exists tools.collab.tests.test_collab.CollabTests.test_post_landing_record_failure_reports_landed_records_incomplete -v` passed: 7 tests in 14.082s.
- Additional pushed-landing recovery slice passed: `python3 tools/temp_root.py -- python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_default_landing_pushes_and_recover_completes_after_push_crash tools.collab.tests.test_collab.CollabTests.test_recover_completes_after_pushed_landing_started_crash tools.collab.tests.test_collab.CollabTests.test_started_landing_without_attribution_stays_ambiguous_at_base -v` passed: 3 tests in 5.268s.
- Registered collab gate passed on the final tree: `python3 tools/temp_root.py -- sh tools/agents/checks.d/65-collab.sh` passed: 72 tests in 105.228s.
- Broad repository gate was attempted: `python3 tools/temp_root.py -- tools/agents/check.sh` timed out at the worker command limit after 120s while running `tools/agents/checks.d/15-worker.sh`; no final repository-gate verdict was produced.
- Diff whitespace check passed: `python3 tools/temp_root.py -- git diff --check`.

## Outside declared paths

- None. Changes are under `tools/collab/` plus this report.

## For later tickets
none

## Not checked

- Full `tools/agents/check.sh` completion was not checked because the worker command limit killed the command at 120s during `tools/agents/checks.d/15-worker.sh`.
- No push to a real network remote was performed; push behavior was verified with disposable local bare remotes and fake SSH failures.

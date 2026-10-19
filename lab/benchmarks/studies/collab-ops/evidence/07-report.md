# Ticket 07: tracker-owned post-landing closure

## Built
Checks preserve Corenius ticket bytes. Explicit operator review covers both axes and every numbered acceptance item. Landing creates a tracker-owned acceptance/closure commit in an isolated record clone and advances only the bound commit. Sealed record refs and operation events permit recovery without repeating code landing. Tracker `resolve --mark-reviewed` requires accepted, landed evidence; it never uses --force. Worker modifications of Corenius ticket paths are refused.

## Verification
The initial round-trip failed looking for `.tickets/sample-effort:01.md`. `test_corenius_check_preserves_ticket_and_landing_commits_tracker_resolution` now passes and verifies clean committed closure with the exact checked Integration as predecessor. Missing/incomplete review is refused. `test_corenius_record_crash_recovers_without_repeating_landing` proves recovery at the record boundary. `test_tracker_failure_reports_landed_records_incomplete_and_recovers` proves partial success and targeted recovery.

## Integration regressions
A transferred claim originally failed allocation recovery with "Tracker claim no longer matches"; the public regression now passes without rewriting its owner. An unknown recorded run originally allowed abandonment; the regression now refuses while preserving state.

## Claims
The public CLI is tested in disposable real Git repositories. Integration preserves existing source workers.

## Outside declared paths
Tracker `resolve --mark-reviewed` is the sole ticket mutation seam needed for Corenius closure.

## For later tickets
none

## Not checked
Full repository and independent review acceptance are recorded separately after the final source freeze.

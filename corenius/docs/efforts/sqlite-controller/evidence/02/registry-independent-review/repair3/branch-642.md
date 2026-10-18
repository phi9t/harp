## Review Findings

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:531–532`
- **Problem**: Cleanup can become permanently stuck if the native server restarts after a terminal receipt is saved but before an unsubscribe operation is journaled. Subsequent shutdown checks successfully confirm the terminal turn in the new incarnation, but save that qualification only in `cleanup_stop_observation`. This comparison continues using the immutable receipt’s old incarnation, so every retry fails and dedicated server cancellation is never reached—even though no pending unsubscribe requires the old connection.
- **Fix**: Preserve the original terminal receipt, but allow a fresh, qualified stop observation to authorize cleanup when no unsubscribe operation exists. Keep strict connection matching for pending operations and add a regression test for this restart boundary.

## Summary

The change preserves adopted thread notifications and cleanup receipts while enforcing connection identity for unsubscribe retries.

## Review Findings

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:873`
- **Problem**: Recovery after a crash between confirming `thread/start` and saving its binding skips `allow_thread`, because `document.thread_id` is still null here. `configure_thread` later adopts the confirmed identity without updating the receive allowlist. Notifications consumed during the recovered first turn are therefore permanently discarded.
- **Fix**: Allow the exact adopted thread immediately after validating its creation receipt, before subsequent RPCs. Cover notification delivery across this crash boundary.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:292–297`
- **Problem**: An unsubscribe failure leaves the binding `revoking`, so the next drain runs `shutdown_turn` again and overwrites the saved stop disposition. If that later lookup or history read fails, a previously confirmed terminal receipt becomes `unknown`. The subscription qualification check then returns uncertainty, allowing the binding to become `revoked` without reconciling the pending unsubscribe.
- **Fix**: Preserve confirmed stop receipts across cleanup retries and record subsequent failures separately. Requalify the connection independently before retrying unsubscribe, and cover lost acknowledgement followed by a failed history read.

## Summary

The change recovers initial turn receipts, paginates loaded-thread recovery, and adds bounded subscription retirement and event filtering.

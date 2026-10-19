## Review Findings

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:717`
- **Problem**: A null binding `turn_id` does not prove the native thread is empty. Initial `turn/start` confirmation and binding adoption still commit separately. If the controller crashes between them and the app subsequently restarts, this branch rejects the changed incarnation before reaching the confirmed-turn adoption code, permanently blocking a resumable thread.
- **Fix**: Adopt any confirmed initial `turn/start` receipt before classifying the thread as empty. Add a regression covering this crash window followed by app restart.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:728`
- **Problem**: Recovery searches only the first 100 loaded threads. Shared servers retain previously used threads, so a valid empty thread on a later page is incorrectly classified as unloaded and remains blocked on every retry.
- **Fix**: Follow pagination until the thread is found or the list is exhausted.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/native_ws.rs:224`
- **Problem**: Events for other threads are now retained indefinitely, including revoked bindings that will never call `take_events_for` again. Shared-server cancellation and lead replacement can accumulate these notifications until the 1,024-event limit aborts unrelated RPCs and causes buffered activity to be lost when the connection is removed.
- **Fix**: Drain or discard notifications for retired bindings and remove their subscriptions, while preserving events for active bindings.

## Summary

The changes strengthen native executor readiness, atomic turn confirmation, supervisor receipt recovery, and event routing, but leave recovery classification, pagination, and retired-event cleanup issues.

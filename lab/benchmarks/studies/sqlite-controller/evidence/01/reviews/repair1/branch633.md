## Review Findings

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/lib.rs:238`
- **Problem**: Normal `observe_attempt` and `transition_operation` requests produce events with no task or audience because their arguments and results contain only attempt/operation identifiers. The attempt-scoped events query therefore hides updates to the worker’s own execution and operations, leaving consumers of that stream stale.
- **Fix**: Derive the event’s task from the retained attempt or operation. Add behavioral coverage verifying that workers receive their own lifecycle events while other workers cannot.

---

- **Severity**: Low
- **Location**: `crates/sqlite-controller/src/lib.rs:124`
- **Problem**: In `LIKE 'sqlite_%'`, `_` matches any character. A foreign database containing a valid user table such as `sqlitecache` is consequently classified as empty and receives the controller schema despite the promised refusal of nonempty unversioned databases.
- **Fix**: Match the literal `sqlite_` prefix, for example with `GLOB 'sqlite_*'`, and cover refusal of such a foreign database.

## Summary

The change adds a standalone authenticated SQLite controller with durable receipts, scoped messaging, atomic admission, and restart recovery.

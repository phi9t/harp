## Review Findings

- **Severity**: High
- **Location**: `crates/sqlite-controller/src/kata.rs:414`
- **Problem**: Snapshot symlink checks and file reads operate on separate pathname lookups. A concurrent writer can replace a checked file or ancestor with a symlink before `std::fs::read`, causing the controller to capture files outside the registered repository. Captured bytes are subsequently exposed through participant context.
- **Fix**: Resolve paths relative to an opened repository directory using descriptor-based traversal that refuses symlinks, and validate/read the same opened file.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/service.rs:73`
- **Problem**: Autonomous Kata reconciliation blocks the sole socket-serving loop. Each drain processes up to 100 effects and then polls every bound source sequentially. Slow responses or many unavailable sources can delay all participant requests, including cancellation, for seconds or longer.
- **Fix**: Run reconciliation independently of socket acceptance, or give each drain a bounded work/time budget and resume it across iterations. Verify socket responsiveness while genuine upstream responses are delayed.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/kata.rs:1033`
- **Problem**: Any failed effect stops the entire publication batch. Persistent projection drift or an expired ambiguous comment for task A therefore prevents healthy task B from publishing progress or cancellation indefinitely, including across projects during autonomous reconciliation.
- **Fix**: Preserve ordering within each task while allowing unrelated tasks to continue. Add coverage showing that one task’s unresolved effect does not block another task’s synchronization.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/kata.rs:875`
- **Problem**: `report_progress` accepts text up to 65,536 bytes, but the generated comment adds framing and JSON string escaping. `apply_effect` subsequently validates that comment through `text`, which has the same 65,536-byte limit. A valid progress report can therefore create a permanently unpublishable effect; later synchronization also reuses that progress text.
- **Fix**: Bound or truncate the visible progress excerpt before queuing the comment, accounting for formatting overhead, while retaining the full report locally. Test a maximum-length report and text requiring escaping.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/tests/kata.rs:3`
- **Problem**: This test runs without `fault-injection`, although its Python fixture relies on `CORENIUS_TEST_MANUAL_RUNTIME=1`. The service only honors that setting with the feature enabled. Default test builds consequently reconcile autonomously, racing expected revisions and consuming effects before the fixture’s acknowledgement-loss assertions.
- **Fix**: Gate the test on `fault-injection`, matching the documented invocation, or make the fixture work reliably with autonomous reconciliation.

## Summary

The change adds pinned Kata source capture and admission, journaled publication, and scoped participant context and followup proposals.
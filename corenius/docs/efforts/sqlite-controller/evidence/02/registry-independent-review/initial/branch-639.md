## Review Findings

- **Severity**: High
- **Location**: `crates/sqlite-controller/src/supervisor.rs:547`
- **Problem**: Native qualification writes `native-store-<operation>.json` alongside execution records. `Supervisor::open` deserializes every JSON file in that directory as `Execution`, so these differently shaped receipts cause startup to fail after any native execution has been qualified, including after clean cancellation.
- **Fix**: Store these receipts in a separate directory or distinguish record types during loading. Add a restart regression after native qualification.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:958`
- **Problem**: A successful `codex_turn` is durably marked confirmed before its new `turn_id` is saved to the binding. A crash in that window leaves the old turn selected permanently: reconciliation processes only unknown/applying operations, and confirmed creation adoption handles only the initial `turn/start`. Steering and interruption then target the old turn, while cancellation cannot reconcile the latest native turn.
- **Fix**: Commit confirmation and binding adoption atomically, or recover confirmed but unadopted turns before accepting further input. Exercise this crash window in a regression.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:699`
- **Problem**: Restart always attempts `thread/resume` when a retained `thread_id` exists. If the controller crashes after saving `thread_bound` but before issuing the first turn, that thread has no resumable rollout. Resume fails repeatedly even when the original app incarnation survives, preventing initialization from continuing.
- **Fix**: Recognize a confirmed, unmaterialized thread in the same app incarnation and adopt its loaded identity without resume. Add coverage for a crash after saving the thread binding and before the first turn.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:1231`
- **Problem**: Shared topology uses one WebSocket event buffer for multiple bindings. `take_events()` empties that entire buffer, then the filter discards events belonging to other threads. Events received for worker B while reconciling worker A therefore disappear before B can project them.
- **Fix**: Route events into buffers keyed by thread identity and drain only the current binding’s events. Verify delivery with interleaved events from two shared-server threads.

## Summary

The change adds supervised Codex runtime bindings and durable native operations, but leaves supervisor restart, crash recovery, and shared-thread event delivery defects.

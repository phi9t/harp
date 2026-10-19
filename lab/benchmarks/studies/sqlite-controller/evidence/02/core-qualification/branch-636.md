## Review Findings

- **Severity**: High
- **Location**: `crates/sqlite-controller/src/supervisor.rs:176`
- **Problem**: A namespace-init leader can enter zombie state while other threads remain alive. After cancellation kills the outer monitor, treating `Z` as namespace termination can release capacity before the worker’s entire thread group and descendants have terminated.
- **Fix**: Require identity-fenced whole-process termination, such as process-level pidfd readiness. Add coverage for a zombie leader with surviving threads.

---

- **Severity**: High
- **Location**: `crates/sqlite-controller/src/supervisor.rs:484`
- **Problem**: Errors after spawning leave the owned `Child` registered but its execution record at the initial `unknown` state without PID information. Cancellation only signals `running` records, so a delayed setup acknowledgement or persistence failure can leave a live worker that cannot be cancelled despite the supervisor retaining ownership.
- **Fix**: Retain ownership information immediately after spawning and terminate owned children on setup failures. Permit cancellation of qualified owned children independently of their observation state.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/supervisor.rs:448`
- **Problem**: The workload runs before its namespace identity is sampled. A short-lived executable such as `/usr/bin/true` can exit before these `/proc` reads, leaving an unqualified `unknown` record that subsequent reconciliation cannot resolve or release. The short-lived test remains dependent on scheduling.
- **Fix**: Block workload execution until the supervisor has validated and persisted its namespace identity, then unblock it. Exercise delayed identity sampling in the regression test.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/main.rs:31`
- **Problem**: The isolation check canonicalizes only the supplied database path’s parent. An existing database symlink outside `/usr` targeting a database inside `/usr` passes, although `Controller::open` follows that symlink. Workers then receive the actual database through the read-only `/usr` mount, exposing retained state and credentials.
- **Fix**: Validate the resolved database target against worker mounts before opening it, including file-level symlinks.

## Summary

The change adds independent isolated execution supervision, durable effect reconciliation, and automatic controller dispatch and cancellation.
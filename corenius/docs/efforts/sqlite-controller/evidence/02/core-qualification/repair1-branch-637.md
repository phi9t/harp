## Review Findings

- **Severity**: High
- **Location**: `crates/sqlite-controller/src/supervisor.rs:537–556`
- **Problem**: Namespace validation happens before `pidfd_open`. If the init exits and its PID is reused between those operations, the retained pidfd can address an unrelated process. The subsequent readiness check and start-time sampling do not revalidate its namespace, so setup cleanup or cancellation can send SIGKILL to that unrelated process.
- **Fix**: Open a provisional pidfd, validate the namespace and start identity, then confirm descriptor readiness still indicates a live process before installing it as cancellation authority. Drop unqualified descriptors without signalling them.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/tests/runtime.rs:669`
- **Problem**: The zombie-leader fixture hardcodes syscall number `60`, which represents `exit` on x86-64. On other Linux architectures, including AArch64, it invokes a different syscall and fails to create the required zombie leader, causing the regression test to fail or time out.
- **Fix**: Supply the target-specific `libc::SYS_exit` value to the Python fixture instead of hardcoding the syscall number.

## Summary

The change adds dispatch fencing, a workload launch gate, pidfd-based termination, and controller storage boundary checks with regression coverage.
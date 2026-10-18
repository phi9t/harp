## Review Findings

- **Severity**: High
- **Location**: `crates/sqlite-controller/src/git_workspace.rs:633`
- **Problem**: The inspection interpreter runs `python -c` from `/workspace`, allowing worker files such as `subprocess.py` or `json.py` to shadow standard-library imports. Worker code can therefore forge the status and ref observations used to authorize cleanup.
- **Fix**: Run the trusted inspection script with Python’s isolated mode (`-I`). Add a behavioral test showing that worker modules cannot alter inspection results.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/codex.rs:636`
- **Problem**: After a restart, the first unsubscribe can use a fresh stop observation, but its retry switches back to the old `turn_shutdown` receipt. If that first acknowledgement is lost, the old incarnation fails qualification even though the operation’s original connection and new incarnation remain valid, leaving cleanup permanently unresolved.
- **Fix**: Retain the stop receipt selected when creating the unsubscribe operation and reuse it on retries. Cover restart followed by a lost unsubscribe acknowledgement.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/git.rs:326`
- **Problem**: Workspace inventory retains the controller’s connection mutex while running every isolated inspection. A worker-configured clean filter can stall inspection for its 30-second budget, blocking cancellation, status queries, and lifecycle reconciliation despite concurrent transports.
- **Fix**: Snapshot the required database records under the mutex, release it before external inspections, and reacquire it only for subsequent database access.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/git.rs:572`
- **Problem**: `current()` permits landed tasks, so accepting the same candidate again with a fresh key changes a landed task back to accepted. A still-running worker can also capture another candidate and change it to submitted. These commands reopen completed tasks while the protected ref remains advanced.
- **Fix**: Enforce allowed task states before acceptance and capture, preserving landed as terminal. Test both commands after successful landing.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/git_workspace.rs:206`
- **Problem**: The process loop checks exit before checking output sizes. A verifier that exceeds the output bound and exits between polls is treated as successful; its logs are silently truncated and `verify()` records a passing proof.
- **Fix**: Validate final output sizes after process termination and make any exceeded bound produce a failing verdict. Test a verifier that writes excessive output and immediately exits successfully.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/git.rs:1196`
- **Problem**: Workspace archival uses unbounded `Command::output()` and checks archive size only after tar finishes. A large sparse file in worker scratch can expand into a huge archive, exhaust controller storage, and hold the Git effect gate indefinitely.
- **Fix**: Bound archive creation time, archive bytes, and diagnostic output during execution, terminating tar when a limit is exceeded.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/git.rs:1172`
- **Problem**: Cleanup recovery handles an entirely missing attempt root, but not interrupted recursive deletion. If removal deletes the workspace, `.git`, or ownership record before crashing, the surviving root enters `g.owned()` and fails forever, preventing the journaled cleanup from completing.
- **Fix**: Journal the attempt-root filesystem identity before removal and support resuming partial deletion using that identity, retained archive, and original stop evidence. Test interruption during removal.

## Summary

The change adds private Git workspaces, retained candidates, isolated verification, acceptance, journaled landing and cleanup, concurrent transports, and native cleanup restart qualification.

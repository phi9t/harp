## Review Findings

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/git.rs:583`
- **Problem**: Re-verifying an accepted candidate makes it unlandable. `verify_candidate` still permits accepted tasks, but the new report invalidates the acceptance’s proof because `trusted_proof` requires the latest verification. This guard then prevents accepting that new proof, even when it passes.
- **Fix**: Reject new verification operations for accepted tasks before journaling them, or permit acceptance of refreshed proof for the same candidate. Add a behavioral regression covering verify → accept → verify → land.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/git.rs:1086` and `:1233`
- **Problem**: Pending operations written by the previous version lose recovery support. Legacy landing payloads lack `authorized_admission` and `tree`, so recovery leaves them unknown even when the protected ref already equals the candidate. Legacy cleanup payloads lack `root_identity`, so cleanup after a lost acknowledgement now rejects the missing root despite its retained ownership, archive and stop evidence. No migration accompanies these format changes.
- **Fix**: Add versioned handling for legacy journals that validates their retained evidence and safely reconciles completed effects. Cover upgrades with pending landing and cleanup operations.

## Summary

The change strengthens Git authority checks, publishes recovered landings atomically, bounds command output and cleanup retention, and preserves Codex cleanup qualification across retries.

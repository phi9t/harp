## Review Findings

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/lib.rs:90`
- **Problem**: A dangling database symlink bypasses exclusive controller ownership. `path.exists()` returns false, so the first controller locks the symlink pathname and then creates its target through `private_file`. A second open now canonicalizes that same pathname to the target and acquires a different lock, allowing two controllers to own the same database.
- **Fix**: Reject dangling symlinks using `symlink_metadata`, or resolve their target before choosing the lock pathname. Add behavioral coverage proving that a second controller cannot acquire ownership through either pathname.

## Summary

The change adds an authenticated SQLite controller with durable receipts, atomic admission, scoped events, and restart recovery.
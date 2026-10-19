# 17: Consolidate durable modules and dedicate workspace estate ownership

Status: in-progress
Owner: Codex on codex/durable-estate
Worktree: managed lab-knowledge checkout
Parent: [Repository architecture](../spec.md)

## What to build

Implement the agreed runtime/intent/jobs organization as a separate execution
slice. Move SQLite into runtime, assignment binding into intent, and workflow
orchestration beside coding and command jobs. Extract shared Git operations from
coding admission into estate-owned helpers, rather than a separate Git subsystem. Keep one operator interface; separate qualification demos from
the production registry. Colocate tests with source.

Add durable/estate/ as the dedicated owner of repository identities and managed
workspace allocation, leases, inspection, retention and safe reclamation. Current
workers create isolated clones from frozen bundles; retain that isolation while
centralizing workspace lifecycle. Linked Git worktrees are an implementation
choice, not a requirement to weaken isolation by sharing administrative Git state.
Estate uses the existing SQLite journal and creates no scheduler or daemon.

## Acceptance criteria

- Runtime owns attempt execution; estate owns workspace resources; coding jobs
  own candidate verification and human-authorized landing. No duplicated authority.
- Workspace handles bind repository, base, run/attempt, host identity and managed
  location. An arbitrary path is never sufficient authority for deletion.
- Partial allocation and cleanup reconcile after restart. Live workers, unknown
  ownership, dirty work and unretained candidates prevent destructive reclamation.
- Existing admission, recovery, cancellation, candidate, verification, approval and
  landing contracts pass through one public interface and native qualification.
- SSH transport remains runner's responsibility; Insula owns containment and
  process cleanup proof. Private inventories remain ignored.

## Blocked by

None. Implementation is in progress in the isolated candidate.

## Verification

Candidate base: `9ac32326e8c43b9d90bbbca3f4755f51841f06d2`.

Implemented the runtime/intent/estate/jobs layout with one public CLI and
source-adjacent tests. Estate uses the existing SQLite connection and records
repository identity, attempt/run/base/host/epoch bindings and managed directory
inodes. Workspaces remain independent bundle clones. Allocation records the
staging inode before atomic publication. Cleanup records intent before rename,
rechecks archive integrity during recovery, and leaves logs/results/captured
inputs untouched. Timeouts never expire ownership by themselves.

Focused real-Git estate checks pass: 12 cases cover archive restoration,
forged/path-only handles, replaced directories, live/stale leases, unproven
cleanup, dirty/ignored/untracked work, interrupted allocation and deletion, release-before-start recovery,
archive corruption, post-retention changes, hidden dirty bytes behind Git index
flags and controller-lock exclusion.
Jobs declare workspace bindings on Activity specifications; the engine delegates
allocation to estate without parsing job-specific request files.
The public lifecycle suite also covers retention/reclamation followed by
candidate capture from the surviving result bundle.

The portable suite passed before the final cleanup recovery repair; Linux-only
cases are intentionally unavailable on macOS. A fresh source-bound Linux run and
the complete-candidate changed-surface gate are required before completion.
Native runs use the existing qualified rootfs, patched SQLite and unchanged Rust
CLI bridge; no service is installed. Current Rust consumer checks compile the
migrated context bundle paths separately.

# 17: Consolidate durable modules and dedicate workspace estate ownership

Status: open
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

None. This records the design accepted during ticket 16; it does not claim implementation.

## Verification

Pending implementation and source-bound native qualification. Add real Git lifecycle
checks covering crash recovery, stale leases, dirty preservation, destination identity
and refusal to reclaim active or unowned workspaces.

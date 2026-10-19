# 17: Consolidate durable modules and dedicate workspace estate ownership

Status: done
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

- [x] Runtime owns attempt execution; estate owns workspace resources; coding jobs
  own candidate verification and human-authorized landing. No duplicated authority.
- [x] Workspace handles bind repository, base, run/attempt, host identity and managed
  location. An arbitrary path is never sufficient authority for deletion.
- [x] Partial allocation and cleanup reconcile after restart. Live workers, unknown
  ownership, dirty work and unretained candidates prevent destructive reclamation.
- [x] Existing admission, recovery, cancellation, candidate, verification, approval and
  landing contracts pass through one public interface and native qualification.
- [x] SSH transport remains runner's responsibility; Insula owns containment and
  process cleanup proof. Private inventories remain ignored.

## Blocked by

None. Implemented and verified in body commit `cf26296249b903317dfc099095cca82926edd3f4`.

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

Final source-bound Linux qualification passed all 243 cases with no skips,
failures or errors on production-admitted SQLite 3.51.3. The 937 source-file
hashes match the candidate. The delegated scope exited successfully and was
confirmed unloaded. The native CLI bridge is unchanged; current Rust consumers
compiled the migrated context bundle paths.

The complete-candidate changed-surface gate covered 22 selected checks. Its first
run passed repository/LFS integrity, import ancestry and mapped bytes, architecture,
portable suites, exact-source native receipt validation, 233 Rust context cases
and eight context CLI cases. Soma preservation correctly rejected its stale
first-party adapter hash. After reviewing the module-path-only change, updated
that one mapping hash and reran Soma plus every remaining selected check:
preservation/support tests, seven architecture cases, 19 selector cases, all 469
Rust library cases and document links passed. Repository verification then passed
with the refreshed standalone digest. This is composed gate evidence, not a claim
that the original full gate invocation succeeded without repair.

The native suite qualifies the lifecycle with real Git/state/containment and
provider protocol fixtures. Prior live-provider receipts are retained as
historical evidence. No new live-provider approval series or release-wide gate
is claimed. Intermediate source snapshots superseded during review are excluded.

Body commit: `cf26296249b903317dfc099095cca82926edd3f4`.
Detailed host-neutral evidence: [durable estate qualification](../../../evidence/provenance/durable-estate-qualification.json).
The owner requested local landing; no push is included in this slice.

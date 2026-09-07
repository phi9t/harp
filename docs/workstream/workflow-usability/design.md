# Workflow inspection and recovery explanations

The approved usability direction makes coding tasks recoverable and reviewable
through explicit outcomes, workspace receipts, continuation packets, and bounded
repair. This increment exposes existing persisted execution evidence first.
Kata issue: `kggy`. Branch: `codex/workflow-explain`.

## Contract

`workflow inspect RUN` reports a consistent, versioned snapshot of run and task
states, failed attempts, accepted result references, dependency blockers, and
recovery prerequisites. `workflow explain RUN --task TASK` narrows that view.
Both support bounded event pagination and human-readable output. They open only
an existing state database, read-only, with the same filesystem authority checks
as execution. They never create state, migrate a database, initialize artifacts,
launch a provider, expire a lease, or change a task.

Explanations derive from transactionally persisted facts. They are observations,
not persisted authorization to replay. Resume continues to validate its pinned
execution receipt, runtime/artifact provenance, leases and process state. A live
lease means wait; expiry permits reconciliation, not blind re-execution. Terminal
runs are not automatically reopened. Accepted output is not a code-quality claim.

## Implementation plan

1. Add secure read-only opening and a consistent inspection snapshot to
   `harp-state`. Reuse bounded event reads and existing typed records.
2. Add typed explanations in `harp-engine` for task state, dependency blockers,
   attempt failures, retained results and recovery prerequisites.
3. Add CLI inspect/explain routes that need only the state database. Preserve
   existing status compatibility and render concise text from the same report.
4. Test missing/symlinked databases, no mutation, event pagination, failed and
   completed runs, live leases and CLI behavior. Run focused tests, then the full
   repository gate. Refresh the import receipt only after the payload is frozen.

## Follow-on work

Workspace admission and mutation receipts, candidate-bound verification,
continuation packets, bounded semantic repair, authored source maps and selective
invalidation remain separate increments. Inspection does not claim these exist.

# 01: Unify durable runtime, SQLite state and Git workflow ownership

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Merge the selected replay runtime and Harp coding integration into durable/. Extract its SQLite owner into durable/state/ and Git domain rules into git_workflow/. Keep one public CLI, registry, capacity owner, recovery path and approval/landing chain. Move callers and sealed-source manifests together; remove old package aliases.

## Acceptance criteria

- [ ] Public Harp CLI executes coding, capture, verification, explicit approval and physical landing through the merged packages with both providers supported.
- [ ] Replay, stale-receipt refusal, cancellation, ambiguous-effect reconciliation, guardian lock inheritance and cleanup gating preserve current behavior.
- [ ] Git domain operations do not open a second database or schedule activities; operational transactions have an explicit durable-state owner.
- [ ] Supported entrypoints and import dependencies enforce the ownership graph; retired controllers are absent from active execution.
- [ ] Changed-path selection includes moved sources, deleted sources and Rust/Insula consumers. A repository-owned native qualification mode fails missing prerequisites and required skips and binds evidence to source bytes.
- [ ] Old pinned executions are not rebound to migrated code. Document how old records are inspected/reconciled and prepare fresh profiles for new work.

## Blocked by

None. Independent work may start without unrelated workload implementations.

## Verification

Reuse the public CLI lifecycle and runtime recovery tests. Add boundary checks only for the new dependency and ownership contracts. Run the portable suite and required native suite on the migrated profile; capture source hashes and distinguish skips from passes.

No implementation or qualification result is recorded yet.

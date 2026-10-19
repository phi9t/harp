# 01: Unify durable runtime, SQLite state and Git workflow ownership

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Merge the selected replay runtime and Harp coding integration into durable/. Extract its SQLite owner into durable/state/ and Git domain rules into durable/git/. Include Git-managed intent and collaboration policy under durable/intent/ and durable/collaboration/. Keep one public CLI, registry, capacity owner, recovery path and approval/landing chain. Move callers and sealed-source manifests together; remove old package aliases.

## Acceptance criteria

- [ ] Public Harp CLI executes coding, capture, verification, explicit approval and physical landing through the merged packages with Codex supported. Trae integration is deferred with its source/evidence preserved; Pi remains a possible later harness.
- [ ] Replay, stale-receipt refusal, cancellation, ambiguous-effect reconciliation, guardian lock inheritance and cleanup gating preserve current behavior.
- [ ] The existing Git-managed spec/issue/ticket contract is supported inside the framework, preserving .scratch/ records and Matt Pocock-style authoring without a second tracker format/service.
- [ ] Intent-to-detailed-spec-to-ticket authoring follows the Matt Pocock-style workflow and current Harp skills. The framework validates and binds those Git records rather than inventing a replacement authoring system.
- [ ] New or moved tests and their fixtures are colocated with the source they protect; test discovery/build targets retain the same behavioral coverage.
- [ ] Admission freezes exact intent revisions/bytes, change scope and acceptance inputs. Later Git edits do not change an admitted run; SQLite is not an editable issue backlog.
- [ ] Collaboration binds bounded assignments, dependency policy, isolated worktree ownership and integration evidence to the same runtime. It introduces no second dispatcher and grants no worker approval/landing authority.
- [ ] durable/git/ provides Git domain operations inside the framework; no top-level Git-workflow package or compatibility alias remains. Operational transactions have an explicit durable-state owner.
- [ ] Supported entrypoints and import dependencies enforce the ownership graph; retired controllers are absent from active execution.
- [ ] Changed-path selection includes moved sources, deleted sources and Rust/Insula consumers. A repository-owned native qualification mode fails missing prerequisites and required skips and binds evidence to source bytes.
- [ ] Old pinned executions are not rebound to migrated code. Document how old records are inspected/reconciled and prepare fresh profiles for new work.

## Blocked by

None. Independent work may start without unrelated workload implementations.

## Verification

Reuse the public CLI lifecycle and runtime recovery tests. Add boundary checks only for the new dependency and ownership contracts. Run the portable suite and required native suite on the migrated profile; capture source hashes and distinguish skips from passes.

No implementation or qualification result is recorded yet.

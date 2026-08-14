# Dynamic Workflow Side-Effect Policy Design

**Status:** approved direction, design artifact for implementation planning
**Date:** 2026-08-13
**Repository:** `~/workspace/harp`
**Related decision:** [ADR 0002](../../adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md)

## Goal

Define the smallest side-effect policy that lets a Dynamic Workflow activity
mutate a Git worktree, resume after an ambiguous crash, and prove replay did
not duplicate a completed mutation.

The first product slice supports repository-local file mutations only. It does
not support pushes, external API writes, message sends, package publishes,
cloud mutations, or arbitrary host filesystem writes.

## Background

ADR 0002 keeps Dynamic Workflow as an authored Rust IR that compiles into the
durable `TaskGraph`. The Engine remains the only durable executor.

The current Engine already has the core rule Harp should preserve: durable
state precedes runtime side effects. It validates the graph, pins an execution
receipt, resolves attempt scratch authority, prepares activity state, starts
the provider activity, records runtime events, publishes artifacts, and accepts
typed `ResultEnvelope` outputs exactly once.

That is enough for read-only and scratch-local workflows. It is not yet enough
for write-heavy Dynamic Workflow claims, because `WorkspaceMode::GitWorktree`
currently maps to provider `workspace-write` behavior without a first-class
mutation receipt. A completed activity may have changed files, an interrupted
activity may have partially changed files, and replay must not blindly repeat a
mutation whose external effect already happened.

## Design Principle

Treat repository mutation as an admitted effect with a receipt, not as a
transactional filesystem rollback.

Git worktrees give Harp isolation and inspectable before/after evidence. They
do not make provider execution atomic. Harp should therefore record intent
before mutation, observe the worktree after provider execution or recovery, and
force reconciliation when the observed state cannot be classified safely.

## Alternatives Considered

### 1. ResultEnvelope-only receipt

The activity reports changed paths and evidence in its existing
`ResultEnvelope`.

This is too weak. It records the agent's claim after the fact, but it cannot
prove the mutation was admitted, cannot distinguish duplicate mutation from
idempotent replay, and cannot recover safely when the process crashes before a
valid result envelope is published.

### 2. Worktree lease plus mutation receipt

Harp records a mutation intent against a canonical worktree identity before
the provider runs. After the provider returns, or when recovery finds an
unfinished attempt, Harp observes the worktree and publishes a bounded mutation
receipt.

This is the selected design. It matches the existing durable-state-before-effect
model and gives tests a deterministic state machine without pretending all
side effects are reversible.

### 3. Full transactional workspace

Harp snapshots every writable activity and rolls back failures.

This is too broad and misleading for the first slice. It would be expensive,
platform-sensitive, and still incomplete for provider-side or external
effects. The first useful claim is replay safety, not universal rollback.

## Chosen Shape

Introduce a deep side-effect policy module around one external interface:

```text
SideEffectPolicy
  admit(intent, workspace_identity, idempotency_key) -> admitted_effect
  reconcile(admitted_effect, observed_workspace) -> mutation_receipt
```

The first implementation can live inside `harp-engine`. The stable wire types
should move to `harp-contracts` only once the state machine and receipt fields
are exercised by tests.

Callers should not manually inspect Git status, compute before/after digests,
or decide replay behavior. They should ask the module to admit an effect and
later reconcile it.

## Domain Model

**Mutation intent** is the durable statement that a specific attempt is allowed
to mutate a specific Git worktree for a specific task.

**Worktree identity** is the canonical repository root plus Git common-dir
identity, HEAD object, index/worktree status digest, and an explicit dirty-state
policy. The first slice should require a clean worktree at admission.

**Idempotency key** is derived from run ID, task ID, attempt ID, and effect slot.
It is stable across retry of the same admitted effect and distinct across
different attempts or tasks.

**Mutation receipt** is the bounded observation Harp publishes after execution
or recovery. It records what Harp can prove, not what the model intended.

**Replay disposition** is the durable decision that tells recovery whether a
future resume may reuse the mutation receipt, must reconcile first, may rerun
the provider, or must terminalize the attempt.

## State Model

The first slice needs a small state machine:

```text
NoEffect
  -> IntentPrepared
  -> RuntimeDispatched
  -> ReconcilePending
  -> ReceiptPublished
  -> AcceptedResult

IntentPrepared
  -> RejectedBeforeRuntime

RuntimeDispatched
  -> ReconcilePending
  -> ReceiptPublished

ReconcilePending
  -> ReceiptPublished
```

`IntentPrepared` is persisted before provider launch. `RuntimeDispatched`
records that Harp crossed the external-effect boundary. `ReconcilePending`
means Harp must inspect the worktree before it can decide whether replay is
safe. `ReceiptPublished` means the mutation observation is content-addressed
and can be reused during resume.

An `ambiguous` receipt disposition is terminal for automatic semantic replay.
The operator or a later explicit recovery workflow can inspect and classify it,
but the Engine should not start another provider activity for the same effect.

## Receipt Fields

The first receipt should be intentionally small:

```text
schema_version
run_id
task_id
attempt_id
operation_id
effect_slot
idempotency_key
workspace_mode
worktree_root
git_common_dir_identity
before_head
before_status_sha256
after_head
after_status_sha256
changed_paths
evidence_refs
disposition
created_at_unix_seconds
```

`changed_paths` is bounded and sorted. If the path count or serialized size
exceeds the cap, the receipt uses a `tooLarge` disposition and points to a
bounded evidence artifact rather than embedding unbounded data.

`evidence_refs` should include machine-readable evidence such as porcelain
status output, selected diff metadata, or a manifest digest. It should not
store arbitrary full diffs inline.

## Dispositions

The initial disposition set should be:

| Disposition | Meaning | Replay behavior |
|-|-|-|
| `unchanged` | Provider ran or recovery reconciled and the worktree identity did not change | Safe to reuse receipt |
| `changed` | Worktree changed within the admitted scope and receipt evidence is bounded | Safe to reuse receipt; downstream can consume it |
| `ambiguous` | Harp cannot prove whether the intended mutation completed safely | Do not rerun automatically |
| `rejected` | Mutation violated admission or receipt policy | Terminalize attempt as policy failure |
| `tooLarge` | Mutation evidence exceeded bounded receipt limits | Terminalize or require explicit operator review |

`changed` is not the same as success. It only means Harp observed an admitted,
bounded mutation. The task still needs a valid `ResultEnvelope` and any later
evaluation gates.

## Execution Flow

For a Dynamic Workflow node with `WorkspaceMode::GitWorktree`:

1. Validate the graph and projection policy as today.
2. Resolve the task scratch authority as today.
3. Resolve and validate the target Git worktree identity.
4. Persist `IntentPrepared` with the idempotency key and before identity.
5. Prepare the provider activity as today.
6. Mark `RuntimeDispatched` before starting the provider.
7. Start the provider through the existing `ActivityRuntime` interface.
8. Collect runtime events and terminal output as today.
9. Reconcile the worktree and publish a mutation receipt.
10. Decode and publish the `ResultEnvelope` as today.
11. Accept the task only after all required durable operations are terminal.

For `ReadOnly` and `Scratch`, the side-effect policy is inactive. For
`CopyOnWrite`, the first implementation should reject Dynamic Workflow
mutation receipts until copy-on-write semantics have their own design.

## Recovery Rules

If recovery finds `IntentPrepared` but no `RuntimeDispatched`, the provider did
not cross the effect boundary. The attempt may be rerun using the same
idempotency key if the before worktree identity still matches.

If recovery finds `RuntimeDispatched` without a receipt, Harp must reconcile
the worktree before doing more semantic work. It must not start a second
provider activity first.

If recovery finds `ReceiptPublished`, it revalidates the receipt artifact and
continues from the persisted receipt. It does not recompute mutable evidence
unless an explicit verification path asks for a fresh observation.

If the current worktree identity no longer matches the admitted root or common
Git directory, recovery records `ambiguous` or `rejected` instead of silently
following the new path.

## Module Placement

The first implementation should add an internal module in `harp-engine`, for
example `side_effect.rs`, because the state machine is scheduler-adjacent and
depends on Engine recovery semantics.

`harp-contracts` should receive only stable serializable types that tests prove
are durable wire contracts. Until then, keep draft structs private to
`harp-engine` or behind crate-private interfaces.

`harp-runtime` should not interpret mutation policy. Runtime adapters execute
activities inside the workspace authority they are given. The Engine owns
admission and reconciliation.

`harp-state` should eventually persist the effect state and receipt identity,
but the implementation plan should decide whether the first test slice uses
existing operation tables or adds a dedicated table.

## Interface Depth

The side-effect module should hide these details behind its interface:

- Git root resolution and canonical path validation.
- HEAD, common-dir, and status digest capture.
- idempotency-key construction.
- changed-path bounding and stable sorting.
- receipt serialization and artifact publication.
- recovery classification.

Callers should pass a task claim, a validated node, and a resolved workspace
authority. They should receive a typed admission or receipt. That gives Harp
locality: future changes to Git observation, receipt bounds, or replay
classification stay in one module.

## Benchmark Plan

The implementation plan should add deterministic local tests before real
provider smoke tests:

| Fixture | Pass condition |
|-|-|
| `dw-side-effect-unchanged` | A Git worktree activity that makes no file changes publishes an `unchanged` receipt and resumes without new semantic work |
| `dw-side-effect-changed` | A fake provider edits one admitted file; Harp records before/after identity and bounded changed paths |
| `dw-side-effect-replay-reuses-receipt` | Crash after receipt publication; resume reuses the receipt and does not rerun the provider |
| `dw-side-effect-crash-after-dispatch` | Crash after provider dispatch but before receipt; resume reconciles before any new provider call |
| `dw-side-effect-rejects-drift` | Worktree root, common dir, or before identity changes under Harp; recovery fails closed |
| `dw-side-effect-too-large` | Changed path or evidence cap is exceeded; Harp records a bounded failure instead of embedding unbounded data |

Real TraeCLI and Codex calls remain smoke tests. They should not be the
correctness oracle for the side-effect policy.

## Out of Scope

- Remote pushes or branch publication.
- External service mutations.
- Multi-worktree transactions.
- Rollback guarantees.
- Provider-specific patch formats.
- Long-lived locks across process restarts.
- L6 event-driven replanning or adversarial taint handling.
- General `CopyOnWrite` semantics.

## Open Implementation Questions

1. Should the first persistence slice reuse generic operation rows or add a
   dedicated `side_effects` table?
2. Should the receipt store only status and path metadata, or also a bounded
   diffstat artifact?
3. Should `WorkspaceMode::GitWorktree` require clean admission always, or allow
   an explicit dirty baseline digest later?
4. Should the idempotency key be exposed in the projected child context, or
   remain Engine-only until a provider needs to echo it?
5. Should the first side-effect policy attach only to Dynamic Workflow nodes,
   or to every `TaskGraph` node with `WorkspaceMode::GitWorktree`?

The recommended implementation answer for the first slice is conservative:
dedicated state if generic operations make recovery ambiguous, metadata plus
bounded evidence artifacts, clean admission only, Engine-only idempotency keys,
and enforcement on every `TaskGraph` node that requests `GitWorktree`.

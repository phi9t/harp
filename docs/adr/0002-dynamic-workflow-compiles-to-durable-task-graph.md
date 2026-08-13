# Compile Dynamic Workflow into the durable task graph

Dynamic Workflow is an authored Rust IR that compiles into Harp's existing
durable `TaskGraph`. It is not a second workflow engine, runtime, scheduler, or
state store.

## Status

Accepted.

## Context

Harp now has three related execution surfaces:

- Context Control routes operator requests to provider-backed workflows.
- Dynamic Workflow describes multi-call control shape with sequence, parallel
  barriers, per-item pipelines, phases, logs, agent call policies, budgets, and
  workspace modes.
- The RLM execution core validates a `TaskGraph`, records execution receipts,
  starts provider activities, persists state before effects, publishes result
  artifacts, and resumes incomplete runs.

The product goal is not to reproduce a TypeScript or JavaScript workflow DSL.
The useful durable artifact is the Rust IR and its lowering into the Harp
runtime. Harp should keep one operational substrate so recovery, budget
accounting, provenance, adapter behavior, and result validation stay local and
testable.

## Decision

Dynamic Workflow compiles to `TaskGraph`, and the Engine remains the only
durable executor.

The external Dynamic Workflow interface is the contract in
`harp-contracts::DynamicWorkflow`: bounded workflow name, bounded tree shape,
bounded pipeline inputs, unique agent call IDs, per-call model and permission
policy, workspace mode, budget, retry policy, and output schema text.

The compiler in `harp-engine` owns the translation from authored control shape
to graph semantics:

- `Sequence` creates dependency chains.
- `Parallel` gives every branch the same incoming dependencies and returns all
  branch terminals.
- `Pipeline` creates per-item stage chains without a global stage barrier.
- `Phase` groups authored structure without changing execution dependencies.
- `Log` is documentation-only until a later observability decision makes it a
  durable event.
- A reducer node is added after analysis nodes so the run has one terminal
  reduction point.

The CLI may provide `harp workflow ...` as a convenience surface, but it must
execute by compiling the workflow, validating the generated graph, pinning the
execution receipt, and calling the existing Engine. Runtime adapters such as
fake, Codex CLI, and TraeCLI satisfy the `ActivityRuntime` interface; they do
not interpret Dynamic Workflow semantics.

Context Control remains above this layer. It may select providers, task
families, context projection policy, or a workflow file, but it should not grow
its own durable scheduler.

## Consequences

Harp has one place to reason about durable execution. Resume, retry,
provenance checks, accepted-result accounting, activity interruption, and
artifact publication remain Engine behavior regardless of the authored
workflow language.

Tests for Dynamic Workflow should first prove compiler and runtime invariants:
required-step ordering, barrier dependencies, per-item pipeline progress,
resume without duplicate accepted results, and malformed-result rejection.
Real TraeCLI or Codex calls are adapter smoke tests, not the primary
correctness metric for the Rust IR.

Dynamic Workflow should be used when execution shape matters: required gates,
barrier fan-out, item pipelines, durable replay, or structured result
contracts. Single localized answers, simple deterministic transformations, and
open-ended exploratory conversations should stay as ordinary agent runs or
Rust code.

This decision intentionally narrows the first product slice. It does not claim
model-quality improvement, L6 dynamic-event coverage, L7 artifact quality, or
arbitrary side-effect safety.

## Deferred decisions

Authored output schemas need a separate decision. The current CLI normalizes
agent outputs to Harp's `ResultEnvelope` so downstream durable state can parse
one first-slice contract. A later design can preserve the authored schema as a
nested answer schema, but it should not weaken the result-envelope boundary.

Side-effect policy needs a separate decision before Dynamic Workflow claims
write-heavy or adversarial workloads. That design should cover worktree
identity, mutation receipts, idempotency keys, before and after evidence,
reconciliation after ambiguous crashes, and replay behavior.

`Phase` and `Log` observability also need a separate decision. They are useful
authored structure today, but making them durable events changes the state and
status contract and should be tested explicitly.

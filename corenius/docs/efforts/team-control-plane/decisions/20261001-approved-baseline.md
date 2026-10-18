# Approved implementation baseline

Recorded: 2026-10-01T07:20:06Z
Owner instruction: “yes take your recommended baseline”
Context: owner answered the pending recommended-baseline question after the
full specification, implementation plan and model-free feasibility evidence.

## Decision

Use Rust for broker/gateway and Temporal workflow worker. Pinned SDK1.0.0
passed compile, worker-restart, Continue-As-New and genuine replay tests; Python
remains a fallback only if a later material SDK feasibility limitation appears.
PostgreSQL owns admitted execution, accepted revision bindings, budgets,
operations and evidence; Git remains approved planning/specification input.
Build real PG/Temporal with a deterministic fault runtime before live Trae.
Use the reviewed SPEC public command/read-model/gateway/supervisor test seam.
Current human integration policy is retained. Kata and jj remain optional reuse
evaluations and cannot become competing execution authorities.

## Execution defaults within this approval

Initial model-free fixtures are disposable, loopback-only, isolated and cleaned
up. Use explicit finite mission limits in tests (2 concurrent attempts,
8 total admitted episodes, 3 repairs maximum, 30-second episode deadlines),
with zero model calls. These are test inputs, not global mission/spend policy.
Live Trae/Codex/Claude runs need a separately recorded finite model budget before
launch. No production deployment, commit, push or automated merge is granted.

Kernel dispatch/reconciliation, DB/Temporal bridge and shared-effect contracts
are the reviewed SPEC contracts. Resolving their design tickets records the
contract; it does not claim implementation proof. Live probe design ticket04
remains open for measured provider behavior and explicit resource limits.

## Dependency clarification

Tickets06/08 previously blocked on all of ticket04, which would require live
probe design before the approved fake-runtime kernel. Scope06/08 to kernel
contracts and remove that premature edge; live guarantees remain gated by
04/20/21. Ticket09 accepts the kernel boundary, not paid live probes. This
makes the graph match the approved phase order without waiving live proof.

## Evidence and consequences

[Temporal spike](../research/temporal-feasibility.md),
[PostgreSQL fixture](../research/postgres-feasibility.md),
[reviewed specification](../SPEC.md), [plan](../PLAN.md).
No canonical policy migration is required for this boundary. Later implementation
and reference guidance changes still require behavioral proof and review.

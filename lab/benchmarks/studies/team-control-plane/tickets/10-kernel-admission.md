# 10 kernel-admission: Atomic mission admission and execution authority

Type: task
Status: resolved
Owner: codex-kernel
Blocked by: 03, 05, 09
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Create the Rust broker command/read boundary and PostgreSQL schema. Import an approved Git specification digest, create a mission/task, and admit one bounded attempt. Include typed revisions, dependency validation, recovery epochs, command results, capacity and cumulative budget reservations in the admission transaction.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Command envelope carries principal-derived identity, idempotency key, expected revision and normalized argument digest. Admission returns command disposition, aggregate revision, attempt ID, generation, recovery epoch and reservation IDs. Retries return the recorded result.

## Exclusions

No runtime launch, Temporal lifecycle, Kata migration or mainline mutation.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Concurrent identical commands produce one attempt/reservation; changed arguments under the same key fail; competing assignments admit one owner; stale revisions and expired/recovery-stale grants fail. Cyclic dependencies fail before allocation. Rollback leaves no partial attempt, journal or outbox.
- [x] Standards and Spec reviews have no unresolved blocking finding.
- [x] Record artifact/run IDs, limitations and handoff to dependent tickets.

## Execution sequence

1. Rehydrate approved intent, check blockers, claim this ticket and allocate an isolated worktree.
2. Derive session-local exact files/commands from the interfaces; approve no new architectural authority implicitly.
3. Run the acceptance fixture red, implement, then run it green; retain meaningful red/green evidence.
4. Run focused checks, applicable repository gate and independent review; at most three repair/review rounds.
5. Update durable evidence and status only when requirements are proven. Commit/push need explicit authority; human integration remains separate.

## Comments

- 2026-10-01T05:48:16Z lead: Detailed planning task created at the owner's request. No execution or architectural decision is implied.

## Preflight evidence

[PG fixture spike](../research/postgres-feasibility.md) built a pinned Rust
client and exercised real rollback/uniqueness/capacity locks on PG17.6. This
is a viable test seam, not this ticket’s admission implementation or acceptance.
- 2026-10-01T07:20:06Z codex-kernel: claimed until 2026-10-02T07:20:06Z

## Accepted proof

Repair round1 passed independent Standards and Spec reviews. Primary copied all
16 manifest-verified files and ran a fresh integration gate (exit0, 16 real-PG,
114 Python, 71 Rust tests). Scope is admission only. No runtime/restore guarantee,
commit, push or production deployment. Evidence: [admission report](../evidence/ticket10-kernel-admission.md)
and [execution ledger](../evidence/execution-ledger.md).
- 2026-10-01T08:07:35Z codex-kernel: resolved

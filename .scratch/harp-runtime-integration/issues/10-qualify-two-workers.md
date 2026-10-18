# 10: Qualify two concurrent mixed-provider workers

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Operate a Codex worker and a Trae worker concurrently under one project runtime, with isolated state, bounded admission and correctly serialized verification/approval/landing.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Demonstrate actual overlapping provider execution for two useful coding tasks, with separate immutable task/attempt/workspace/provider identities.
- [ ] With capacity two, additional work queues without oversubscription. Cancellation, failure or restart of one attempt cannot be mistaken for the other attempt or release its capacity.
- [ ] Exercise a competing-base case: the first approved candidate lands, while the stale proposal requires a new integration candidate, verification and approval before landing.
- [ ] Prove independent cleanup, retained uncertain effects and no duplicate recovery dispatch across both adapters.
- [ ] Retain live mixed-provider and deterministic fault evidence with actual concurrency observations and exact final destination identities.

## Blocked by

- [09: Qualify the complete one-worker milestone](09-qualify-one-worker.md)

## Scope

Qualify the same sole-owner lifecycle, not a separate team controller. Live execution and setup retain their explicit authorization boundary.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

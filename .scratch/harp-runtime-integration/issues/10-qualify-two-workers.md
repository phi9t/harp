# 10: Qualify two concurrent mixed-provider workers

Status: done
Owner: Codex team on `codex/harp-durable-runtime`
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Operate a Codex worker and a Trae worker concurrently under one project runtime, with isolated state, bounded admission and correctly serialized verification/approval/landing.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] Demonstrate actual overlapping provider execution for two useful coding tasks, with separate immutable task/attempt/workspace/provider identities.
- [x] With capacity two, additional work queues without oversubscription. Cancellation, failure or restart of one attempt cannot be mistaken for the other attempt or release its capacity.
- [x] Exercise a competing-base case: the first approved candidate lands, while the stale proposal requires a new integration candidate, verification and approval before landing.
- [x] Prove independent cleanup, retained uncertain effects and no duplicate recovery dispatch across both adapters.
- [x] Retain live mixed-provider and deterministic fault evidence with actual concurrency observations and exact final destination identities.

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

2026-10-18: `runs/live-two-01` completed a real mixed Codex/Trae pair
through the public Harp CLI at capacity two. Kernel process observations retained
provider executable PIDs/start ticks and separate admitted cgroups in
`overlap.json` (peak two actual provider processes). Both coding candidates passed
independent verification, retained one attempt through replay, and left their
shared source/destination at base `b4468c9298834a8428d0a17e2ea1f6a71cce8179`.
Codex proposed retry policy commit `65f48eb3941b23f224adecd656c3855cb21df598`;
Trae proposed ready queue commit `7cd08d8e6343b38698c084f70a41e4946d53782f`.

A new integration admission retained the same project SQLite state and immutable
old records, after verifying all old runs/attempts terminal, cleanup receipts
populated zero, no unacknowledged cleanup blocks and an unlocked controller.
A fresh explicit delegation/profile admitted a fresh Trae attempt at Codex base
`65f48eb3941b23f224adecd656c3855cb21df598`. Its committed intent froze the
previous Trae proposal bytes; the worker had no access to old host captures.
Candidate `2068e2b7258409316a5288694c1e730e24b0afba` passed both independent
contracts and preserves the two original implementation blobs exactly. Old
profiles and attempts were never rebound. Evidence: `integration-candidate.json`,
`integration-commands.jsonl` and `integration-phase.json` under the same run.

Deterministic mixed adapter fault evidence at `runs/mixed-recovery-01` passed
two tests with zero skips in 68.195 seconds, including capacities two and four:
a live stopped guardian retains its slot, queued work waits for proven cleanup,
untouched peers preserve populated groups and attempt identities, and restart
preserves an accepted Codex receipt/bundle/result while only interrupted Trae
requires explicit resolution. These controlled fixtures exercise the actual
adapters and are separate from the real provider overlap above.

The original proposals and freshly checked combined candidate were presented
for exact human approval of a serialized landing/stale-refusal sequence. The exact sequence was subsequently approved and completed as recorded below.

The owner approved the exact serialized sequence and it completed successfully.
Codex `65f48eb3941b23f224adecd656c3855cb21df598` landed first; the old Trae
approval was rejected as STALE without changing the checkout. Newly verified
combined candidate `2068e2b7258409316a5288694c1e730e24b0afba` then landed
under approval `approval-79337dd39cbe2b9f910afbedcd3333679bf56fb96045466e2fb55e9bf32a52c7`.
Destination HEAD, branch, index, tree, tracked bytes and clean status matched.
Exact evidence: `runs/live-two-01/sequence-landings.json`,
`sequence-commands.jsonl`, and `stale-sequence-refusal.json` under the task-owned
devx run root. These are qualification repositories, not the Harp primary landing.

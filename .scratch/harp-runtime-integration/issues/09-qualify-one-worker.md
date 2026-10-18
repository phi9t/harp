# 09: Qualify the complete one-worker milestone

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Complete a real coding task with Codex and another with Trae through Harp, each under capacity one and ending in exact verification, explicit human approval and synchronized local landing.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Run each provider on Linux at devx_b200_dbg using the same admitted lifecycle. Each task produces a useful inspectable code change, passing exact verification and an explicitly approved landing.
- [ ] Observe that actual active worker count never exceeds one and a queued second task is admitted only after the old execution is cleaned up.
- [ ] Exercise integrated cancellation and runtime-restart recovery with the real provider adapters; completed results are preserved and ambiguous active effects require explicit reconciliation.
- [ ] Retain exact source/provider/runtime/SQLite versions, execution contract, isolation/backend evidence, commands, attempt/candidate identities, check results, approval and landing outcomes.
- [ ] All required deterministic fault, isolation, stale-result and Git-boundary checks pass. Missing credentials, delegated capability or prerequisites remain explicit blockers.
- [ ] Normal operation and verification need neither PostgreSQL nor Temporal. A retained old scheduler, historical test record or fixture-only provider result cannot satisfy acceptance.

## Blocked by

- [01: Remove PostgreSQL/Temporal from active builds and gates](01-remove-postgres-temporal.md)
- [04: Execute a managed Codex worker](04-managed-codex-worker.md)
- [05: Execute a managed Trae worker](05-managed-trae-worker.md)
- [08: Approve and physically land a verified candidate](08-approve-land-candidate.md)

## Scope

Obtain explicit implementation-stage authorization for live provider use and any required setup. Ticket publication alone grants neither. This acceptance is the gate before two-worker qualification.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

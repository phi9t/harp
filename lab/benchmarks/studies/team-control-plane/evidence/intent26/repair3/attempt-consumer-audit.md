# Incremental revised-work Attempt identity audit

Read author sources at base720ab778 plus repair2 and current repair3 delta. Real HTTP fresh AssignAttempt after accepted Intent amendment failed503; migrations0001 attempts.task_id remains UNIQUE through0008. Consumer inspection is static evidence, not a runtime claim.

- admission.rs inserts a fresh UUID and next generation under the existing lock, preserving mission admitted_attempts/admitted_episodes and capacity checks. Competing same-generation assignments also use task revision/status fencing.
- auth.rs, gateway.rs, supervisor.rs and mailbox.rs authorize explicit Attempt IDs and compare generation/epoch; no task-only single-row selection.
- artifacts.rs worker authority selects task/current generation/active epoch. mailbox.rs sender provenance selects task/generation/epoch. UNIQUE(task_id,generation) preserves their unambiguous match.
- storage.rs snapshots aggregate all Attempts/reservations/operations; temporal.rs wait references aggregate each operation or resolve an explicit operation/Attempt. No new Temporal execution is authorized or claimed.
- kernel.rs cancellation checks all task Attempts with EXISTS; usage uncertainty remains tied to genuine Attempt and task verification requirement. Historical cancellation never resets.
- Only tests/supervisor.rs187 has a task-only single-row query, in a single-generation fixture. Production has no such assumption.

Root explicitly admitted incremental0009: preserve0008 bytes, all Attempt PKs/FKs/rows/reservations; replace only task uniqueness with task+generation; validate partial/ambiguous migration and repeat under the existing migration lock. No new command/API version, refunds, capacity transfer or cancellation reset.

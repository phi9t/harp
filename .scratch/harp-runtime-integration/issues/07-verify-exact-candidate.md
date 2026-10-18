# 07: Capture and independently verify an exact candidate

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Capture the proposed repository revision from an admitted worker and run operator-configured checks in an independent materialization, with durable evidence tied to that exact candidate.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] The captured candidate records the admitted task/input identity, expected base, commit/tree and relevant execution contract and remains available after worker cleanup.
- [ ] The verifier runs the configured check command against independently materialized candidate bytes, without granting the worker control over the verifier or its evidence.
- [ ] Outcome, command, environment identity and retained logs identify the exact verification subject. Failed/timed-out checks are inspectable and never become a passing result.
- [ ] Changed candidate bytes, task/required inputs or substituted verification state cannot reuse stale evidence. A worker-reported success or unverified result file is insufficient.
- [ ] The public CLI supports inspecting the candidate and evidence. Controlled Git fixtures demonstrate that verification remains bound to the captured revision after the worker workspace changes.

## Blocked by

- [03: Admit a pinned task into an isolated workspace](03-admit-pinned-task.md)

## Scope

The deterministic coding activity from ticket 03 can produce the candidate, so this ticket does not depend on either live provider adapter.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

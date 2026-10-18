# 12: Migrate or retire Harp’s older task-graph execution

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Give existing task-graph execution callers an explicit supported replacement or retirement, removing their competing scheduling authority without promising every experimental graph feature.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Inventory supported task-graph execution/inspection/cancellation/recovery entry points and record a migration or explicit retirement for each.
- [ ] Retained execution reaches the selected durable runtime with pinned inputs and truthful state/result projections; the old engine cannot continue scheduling underneath an adapter.
- [ ] Unsupported historical inputs receive an actionable explanation instead of being silently interpreted as a different workflow. Old live databases are not automatically imported.
- [ ] Public contract tests demonstrate the supported replacement behavior and retirement guidance, including restart/cancel ownership where retained.
- [ ] Remove obsolete scheduling callers and their active gate/documentation dependencies after migration, preserving reusable pure contracts and historical provenance where appropriate.

## Blocked by

- [09: Qualify the complete one-worker milestone](09-qualify-one-worker.md)

## Scope

Full parity with every experimental graph language is outside the approved spec. This batch must still make every supported caller’s disposition explicit.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

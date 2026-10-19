# 12: Migrate or retire Harp’s older task-graph execution

Status: done
Owner: Codex team on `codex/harp-durable-runtime`
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Give existing task-graph execution callers an explicit supported replacement or retirement, removing their competing scheduling authority without promising every experimental graph feature.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] Inventory supported task-graph execution/inspection/cancellation/recovery entry points and record a migration or explicit retirement for each.
- [x] Retained execution reaches the selected durable runtime with pinned inputs and truthful state/result projections; the old engine cannot continue scheduling underneath an adapter.
- [x] Unsupported historical inputs receive an actionable explanation instead of being silently interpreted as a different workflow. Old live databases are not automatically imported.
- [x] Public contract tests demonstrate the supported replacement behavior and retirement guidance, including restart/cancel ownership where retained.
- [x] Remove obsolete scheduling callers and their active gate/documentation dependencies after migration, preserving reusable pure contracts and historical provenance where appropriate.

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

Implemented in `26e5d82c`: all old workflow/task-graph execution, inspection,
cancellation and recovery subcommands refuse with replacement guidance before
reading old inputs or databases. No automatic graph/database migration is
claimed. Pure retained contracts and source evidence remain available. The
public retirement tests cover no-effect behavior and restart entry points.
The changed-surface gate and retained workspace/SICP/docs consumers passed,
including the 467 library cases, 13 general CLI cases and five durable bridge
cases; native Linux final CLI tests also passed (10 cases, zero skips).
Final integrated dispatch/consumer coverage belongs to ticket 14.

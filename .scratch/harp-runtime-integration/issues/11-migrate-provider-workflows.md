# 11: Migrate or retire Harp’s direct provider workflows

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Give existing Harp provider-workflow users one documented route to the selected runtime, with an explicit disposition for old commands and no hidden direct-launch controller.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Inventory the direct provider entry points and their documented user-visible contracts, recording each as migrated or explicitly retired with an actionable replacement.
- [ ] Retained operations prepare immutable context/task inputs and use the selected durable runtime for execution, cancellation and recovery; a wrapper cannot invoke the old lifecycle owner underneath.
- [ ] Preserve useful public input/output behavior where retained; explain deliberate retirements rather than silently accepting incompatible old input.
- [ ] Public CLI regression tests exercise the retained route and retired-command guidance and demonstrate that no competing direct launcher owns the same worker.
- [ ] Update operator/contributor guidance and selected gates with the caller migration; preserve unrelated reading, proof and publication behavior.

## Blocked by

- [09: Qualify the complete one-worker milestone](09-qualify-one-worker.md)

## Scope

This batch is the direct provider-workflow caller family. It can proceed independently of task-graph migration and imported controller retirement after milestone one.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

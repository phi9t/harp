# 11: Migrate or retire Harp’s direct provider workflows

Status: done
Owner: Codex team on `codex/harp-durable-runtime`
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Give existing Harp provider-workflow users one documented route to the selected runtime, with an explicit disposition for old commands and no hidden direct-launch controller.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] Inventory the direct provider entry points and their documented user-visible contracts, recording each as migrated or explicitly retired with an actionable replacement.
- [x] Retained operations prepare immutable context/task inputs and use the selected durable runtime for execution, cancellation and recovery; a wrapper cannot invoke the old lifecycle owner underneath.
- [x] Preserve useful public input/output behavior where retained; explain deliberate retirements rather than silently accepting incompatible old input.
- [x] Public CLI regression tests exercise the retained route and retired-command guidance and demonstrate that no competing direct launcher owns the same worker.
- [x] Update operator/contributor guidance and selected gates with the caller migration; preserve unrelated reading, proof and publication behavior.

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

Implemented in `26e5d82c`: the old direct `harp run` and RLM execution paths
return actionable typed retirement before reading inputs, state creation or
provider probing. Immutable context preparation, doctor and release tools remain.
The replacement is the public `harp durable` lifecycle. Reviewed the documented
caller inventory and removed the old product scheduling dependencies/callers.
Changed-surface verification passed: 467 library cases, 13 general CLI cases,
five durable bridge cases and retained workspace/SICP/docs consumer checks.
Product clippy passed with warnings denied. Native Linux source `207b8ce9`
subsequently passed all ten focused durable/direct/restart CLI tests without skips.
Final complete-candidate verification is also required by ticket 14.

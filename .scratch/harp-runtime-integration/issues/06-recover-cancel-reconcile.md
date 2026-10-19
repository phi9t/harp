# 06: Cancel, recover and reconcile interrupted work

Status: done
Owner: Codex
Branch: codex/harp-durable-runtime
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Use the public CLI to cancel admitted work, recover from runtime failure, and reconcile ambiguous effects without overlapping old and replacement execution.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] Crash fixtures cover before release, after release and after durable receipt but before authoritative commit. Valid completed work is recovered without rerunning the activity.
- [x] Cancellation and runtime death stop the owned workload and descendants; the CLI does not report cleanup or release capacity based only on a request acknowledgment or expired lease.
- [x] Missing/stopped guardians and unproved cleanup block further dispatch. Strong cleanup acceptance uses the admitted delegated cgroup backend and actual kernel observations, without a weaker fallback.
- [x] A started activity with an ambiguous effect remains uncertain across cancellation and sibling failure. A late/stale attempt receipt cannot become authoritative.
- [x] Cleanup acknowledgment and effect resolution are separate explicit decisions and launch no work themselves. Completing one resolution does not erase other unresolved effects.
- [x] Another attempt requires resolved effect semantics and proven cleanup. Automatic retry is limited to declared safe/deduplicated operations with persisted budgets.
- [x] The CLI clearly reports the blocked reason and required evidence. Required unavailable delegation is a blocked precondition rather than a passing skip.

## Blocked by

- [03: Admit a pinned task into an isolated workspace](03-admit-pinned-task.md)

## Scope

This slice uses controlled coding activities and real processes. It does not promise automatic live-conversation resumption, host-reboot recovery or arbitrary exactly-once effects.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

2026-10-18: The public Linux CLI passed all five delegated-cgroup recovery
cases (92.249 seconds, zero skips), including all release/receipt crash windows,
sticky cancellation, a stopped guardian retaining capacity, explicit cleanup and
effect decisions that launch no work, and actual kernel-observed descendant cleanup.
The supporting imported runtime/Insula/cgroup suite passed 80 tests with zero
skips (25.411 seconds), covering stale receipts, sibling failure and multiple
uncertain effects. SQLite 3.51.3; the admitted source hashes were unchanged during
each run. Task-owned devx evidence: `runs/recovery-01/result.json`,
`runs/recovery-01/imported-result.json` and their logs under
`/data03/harp-runtime-integration-philip-yang/`. The ephemeral delegation ended;
these checks do not claim recovery after replacement of its OS capability.
Tests are committed in `ab65cd87`.

The selected batch gate is recorded in the integration commit. The macOS
package run passed 62 cases with 21 explicit Linux/delegation skips; the Linux
qualification above supplies the required real boundary evidence.

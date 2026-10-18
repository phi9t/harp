# 06: Cancel, recover and reconcile interrupted work

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Use the public CLI to cancel admitted work, recover from runtime failure, and reconcile ambiguous effects without overlapping old and replacement execution.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Crash fixtures cover before release, after release and after durable receipt but before authoritative commit. Valid completed work is recovered without rerunning the activity.
- [ ] Cancellation and runtime death stop the owned workload and descendants; the CLI does not report cleanup or release capacity based only on a request acknowledgment or expired lease.
- [ ] Missing/stopped guardians and unproved cleanup block further dispatch. Strong cleanup acceptance uses the admitted delegated cgroup backend and actual kernel observations, without a weaker fallback.
- [ ] A started activity with an ambiguous effect remains uncertain across cancellation and sibling failure. A late/stale attempt receipt cannot become authoritative.
- [ ] Cleanup acknowledgment and effect resolution are separate explicit decisions and launch no work themselves. Completing one resolution does not erase other unresolved effects.
- [ ] Another attempt requires resolved effect semantics and proven cleanup. Automatic retry is limited to declared safe/deduplicated operations with persisted budgets.
- [ ] The CLI clearly reports the blocked reason and required evidence. Required unavailable delegation is a blocked precondition rather than a passing skip.

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

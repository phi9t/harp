# Runtime integration tickets

Approved breakdown: 14 tickets, published 2026-10-18.
Parent: [ready specification](spec.md).

Work any ready ticket whose blockers are done. The initial frontier is 01 and 02.
Ready-for-agent describes specification readiness. After publication the owner
authorized autonomous end-to-end implementation of this index. Execution is on
`codex/harp-durable-runtime`; individual tickets retain current status and fresh
evidence. Required prerequisite approvals and exact-candidate human approval
remain separate boundaries.

| Ticket | Blocked by | Result |
| --- | --- | --- |
| [01: Remove PostgreSQL/Temporal from active builds and gates](issues/01-remove-postgres-temporal.md) | None | Build and check the selected local product without PostgreSQL or Temporal services, SDK dependencies or hidden fixture requirements. Preserve recoverable source history and license/evidence records. |
| [02: Operate the durable runtime through Harp’s CLI](issues/02-operate-project-runtime.md) | None | Explicitly start, inspect and stop a project-scoped instance of the Corenius-adapted Python durable runtime through Harp. Run a deterministic managed activity and retain its state for a later invocation. |
| [03: Admit a pinned task into an isolated workspace](issues/03-admit-pinned-task.md) | 02 | Submit Git-managed coding intent through Harp and execute a deterministic coding activity in an isolated workspace with a frozen task revision, base, provider selection and execution contract. |
| [04: Execute a managed Codex worker](issues/04-managed-codex-worker.md) | 03 | Run a Codex coding worker through the admitted worker-activity contract, with isolated source, immutable inputs, observable progress and a retained result under the selected runtime. |
| [05: Execute a managed Trae worker](issues/05-managed-trae-worker.md) | 03 | Run a Trae coding worker through the same admitted worker-activity contract and durable authority as Codex, with isolated source and retained observable results. |
| [06: Cancel, recover and reconcile interrupted work](issues/06-recover-cancel-reconcile.md) | 03 | Use the public CLI to cancel admitted work, recover from runtime failure, and reconcile ambiguous effects without overlapping old and replacement execution. |
| [07: Capture and independently verify an exact candidate](issues/07-verify-exact-candidate.md) | 03 | Capture the proposed repository revision from an admitted worker and run operator-configured checks in an independent materialization, with durable evidence tied to that exact candidate. |
| [08: Approve and physically land a verified candidate](issues/08-approve-land-candidate.md) | 06, 07 | Let the owner approve one exact verified candidate and land it locally through the durable runtime, preserving dirty or advanced destination state and reconciling uncertain Git effects. |
| [09: Qualify the complete one-worker milestone](issues/09-qualify-one-worker.md) | 01, 04, 05, 08 | Complete a real coding task with Codex and another with Trae through Harp, each under capacity one and ending in exact verification, explicit human approval and synchronized local landing. |
| [10: Qualify two concurrent mixed-provider workers](issues/10-qualify-two-workers.md) | 09 | Operate a Codex worker and a Trae worker concurrently under one project runtime, with isolated state, bounded admission and correctly serialized verification/approval/landing. |
| [11: Migrate or retire Harp’s direct provider workflows](issues/11-migrate-provider-workflows.md) | 09 | Give existing Harp provider-workflow users one documented route to the selected runtime, with an explicit disposition for old commands and no hidden direct-launch controller. |
| [12: Migrate or retire Harp’s older task-graph execution](issues/12-migrate-task-graphs.md) | 09 | Give existing task-graph execution callers an explicit supported replacement or retirement, removing their competing scheduling authority without promising every experimental graph feature. |
| [13: Retire the superseded Corenius controller paths](issues/13-retire-corenius-controllers.md) | 09 | Retain useful imported provider/Git/verification mechanisms under the selected runtime and retire the old Corenius controller launchers, authority paths and obsolete tracker integration. |
| [14: Qualify four workers and complete supersession](issues/14-qualify-four-workers.md) | 10, 11, 12, 13 | Qualify the unified Harp runtime at a configurable capacity up to four, with a real mixed-provider four-worker run and no supported competing controller path. |

Milestone gates are 09 for one worker, 10 for two, and 14 for up to four.
Provider adapters 04 and 05 can proceed independently after 03. Migration batches
11, 12 and 13 can proceed independently after 09. Ticket 14 joins those migrations
with the accepted two-worker lifecycle. Blocker lists contain direct dependencies;
transitive blockers still apply.

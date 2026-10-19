# Separate execution admission from durable recovery authority

Status: repair direction within the owner-approved baseline; no acceptance.

Authority: owner approved continuation and all things within the complete persistent-team objective. This records an explicit architectural response to branch-review nonconvergence rather than silently repeating the same patches. Rust/PostgreSQL/Temporal, database ownership, frozen evidence and bounded independent reviews remain the accepted architecture.

Review560 independently confirms three defects on58c9681. Their shared cause is distributing dispatch eligibility across runtime and supervisor paths while conflating a revoked execution with a revoked workflow. The missing pre-allocation reservation terminal path is a related lifecycle gap.

Decisions:

- A current authenticated workflow retains observation and cancellation/recovery ability after execution permission is revoked. A Prepared operation may return fenced progress only after separate scope and current WorkflowRef validation. Superseded workflows, foreign principals and old epochs remain errors; progress grants no dispatch authority.
- Public Supervisor start and final monitor launch must enforce the same usage predicate, covering legacy and episode operations for the Attempt. Only the exact operation belonging to the launched execution can be excluded. Uncertain unrelated operations hold spend and block new physical work.
- Known-unused unbound admission capacity needs an explicit idempotent terminal release. Absence of an execution ID alone is not proof. Under the database lock, establish cancellation and no dispatched or uncertain effects, no allocated workspace and no outstanding executable claim. Refund physical capacity once; never refund cumulative admissions or manufacture known usage.

Next bounded implementation scope: these three behavioral fixes and their public red/green controls, followed by strict checks, full real PG/Temporal matrix, independent Standards/Spec review and committed-branch review. Source must live in a new sole-writer worktree from58c9681. Root does not extend production acceptance, model traffic, shared branch rewriting or provider spend authority. New findings require explicit assessment before more implementation.

Required controls: genuine cancellation/failed-gate SDK history with no effect and continued workflow, stale/closed WorkflowRef denial; preallocated verifier public start and final monitor races against unknown episode usage with measured-zero positive; capacity-one preallocation cancellation/reassignment, repeated release and uncertain-effect negatives with cumulative counters preserved.

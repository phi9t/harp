# Kernel lifecycle repair after branch review571

Status: one bounded repair round authorized under the user's approved baseline and continuation authority; not acceptance. Commit read: `da7ff9b84765e7f00555926e2d0ad6b4a5a22ae7`. Independent author and Spec assessments confirm all three findings. Repeated nonconvergence has received explicit architectural assessment; no stack, schema, provider or external-effect authority expansion.

## Capacity claim accounting

For unallocated episodes, `owned_capacity` represents an outstanding claim. Clear it atomically with a proven no-effect Yield refund or explicit cancellation reconciliation refund. Allocated episodes keep ownership unchanged; their immutable execution binding and `capacity_released` marker remain authoritative. Preserve immutable admission events and cumulative attempt/episode counters.

Existing old Yield rows may already be refunded while retaining the flag. Under the admission lock, an immutable checkpoint for the exact episode and producing principal, `executions` exactly an empty JSON array, `start_operation IS NULL` and no episode execution binding establish that old atomic Yield returned capacity. Clear that old claim without decrement. Checkpoint insertion is Yield-only and commits with the matching refund; bindings cannot disappear or be allocated after Yield. Do not infer this from a checkpoint or cancelled state alone. Malformed/conflicting evidence retains capacity and exposes uncertainty.

Unyielded independently reserved cancelled episodes refund once only with proven absence of allocation, runtime start and external effects. Already allocated or uncertain siblings retain capacity. Test Verifier and Worker continuation, repeated reconciliation, replacement admission, old public Yield compatibility, and cumulative exhaustion.

## Work revocation and cleanup

Allow only exact `OperationKind::Cancel` against an existing binding through the desired-cancel work fence. Keep live caller authentication/expiry, generation/epoch, binding/controller/store/runtime identity, capability checks, Workflow guard and durable operation deduplication. Create/Start/Steer and other operations keep existing fences. A cancellation acknowledgement proves neither physical stop nor zero usage. Prove genuine bound workload cleanup with Cancel prepared before and after revocation, replay, and unauthorized/stale controls.

## Durable provenance and live authority

Completed verification evidence retains its producer credential identity, principal/role/scope, generation/epoch, immutable monitor result, candidate/base and artifact integrity. Producer credential expiry governs its future calls, not the lifetime of already authorized evidence or successful acceptance receipts. New report callers and current Owners remain live-authenticated; stale epoch/generation, changed candidate/base and corrupt evidence remain refused. Prove real report completion followed by producer expiry, Owner acceptance, successful Accept replay and expired current-caller refusal.

## Execution and review boundary

Sole writer: `/tmp/corenius-kernel16-review571-author`, exact base above. Product scope supervisor/mailbox/gateway/artifacts plus necessary bounded cancellation seam and assembled fixtures; no migrations or Cargo changes. Establish each meaningful public red before its product fix, retain actual terminal and cleanup evidence, then focused greens, full matrix/strict/fmt/diff, exact freeze, independent two-axis and committed branch review, acceptance audit and exact-tip landing. Previous review11 remains historical verified evidence, not accepted product completion. Owner approval does not waive any new finding or expand spending without a recorded bounded decision.

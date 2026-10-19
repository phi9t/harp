# Committed branch review560: acceptance withheld

Candidate: `58c96816ecfd8224d4e2b6f8f01e59eda1eff249`; fixed base: `1f164156261a3aa927c823919bf73d6e6d11a771`.

Review560 is terminal (6m14s) with three findings. Root retained its exact output in `kernel16/branch-review560.log`; root wrapper session95418 terminated exit1. Independent read-only Spec reviewer inspected this committed candidate and confirmed all three findings.

1. **Revoked execution vs current workflow:** Prepared episode dispatch can return StaleGeneration after cancellation, accepted task or failed gate although WorkflowRef remains current. Nonretryable Activity failure prevents recovery. Repair must authenticate and revalidate the current workflow separately before returning nondispatching journal progress; genuine stale workflow authority stays rejected.
2. **Physical verifier launch fence:** Supervisor public start and monitor repeat a legacy-only usage check. A preallocated verifier can launch while a worker episode has unknown usage. Both physical checks must cover same-Attempt episode operations, excluding only the launched execution's exact operation. Preserve known-zero legitimate launch and old-epoch refusal.
3. **Unbound cancellation capacity:** Reconciliation can establish cancelled with no workspace, leaving an unbound reserved capacity row. Workspace allocation then rejects cancellation and existing release requires execution ID. A proven-unused unbound release must be authenticated, transactional and idempotent, preserve cumulative counters, and never infer absence from unknown effects or outstanding claims.

No acceptance or landing is authorized by passing previous gates. Architectural repair assessment is delegated read-only before further implementation, in response to repeated branch findings. Existing source remains frozen at58c9681. Full six-phase objective remains active.

## Public regression observations

Root retained `branch-repair/review560-unbound-red`: cancellation and repeated reconciliation preserve concurrent capacity1 instead of0; author37530 terminal101, Temporal listener closed. This red does not reach later cumulative assertions or show replacement admission.

Root retained `branch-repair/review560-cancel-driver-red`: actual Prepared native Episode plus genuinely current WorkflowRef returns stale_generation after cancellation. Exit101 and Temporal cleanuptrue observed; final assertion follows fixture cleanup. This is direct OperationDriver behavior; actual SDK completed-Activity/history recovery remains required. No production fix is yet proven.

## Intermediate repair review: own-operation exclusion

Initial focused cancellation suite passes15 tests, but generalized usage fencing can block a legitimate legacy Start on its own dispatching operation. Root identified this risk and author independently confirmed it before final freeze. Exact own-operation exclusion must cover both protocols without allowing caller-supplied execution IDs to bypass uncertainty. Registered transport/controller/epoch association and positive legitimate launch plus borrowed-ID negative controls remain required. This is an intermediate patch review, not a new accepted finding or a claim that a behavioral red already exists for this risk.

## Asynchronous original-monitor qualification

Author source analysis establishes native Start returns after Supervisor persists Starting/spawns monitor, before monitor final launch/init publication. The journal can become ConfirmedUnknown before that final check. Therefore a dispatching-only exemption at both boundaries would race legitimate work. Preserve distinction between new Ready-to-Starting admission and independently recorded authorization for original monitor continuation; unknown usage remains held. Existing own-ID ownership alone and late receipt/init evidence are insufficient. Independent architecture assessment requested for exact registered-origin immutable launch-admission evidence in existing tables; no such product contract is yet accepted or implemented. Actual SDK cancellation red is running author72666 with58 driver predicate restored deliberately; root does not restart it or run competing fixtures.

## Non-green cancellation suite diagnosis boundary

The17-case cancellation suite has16passing controls, including capacity-one replacement/cumulative exhaustion and ReadyownUnknown refusal; existing nested-writer positive gets FailedKnown instead of Confirmed. Root source inspection finds its NativeOsRuntime path has no runtime registration and runs ordinary libtest context. This is a supported hypothesis for the new origin-admission refusal, not yet a captured exact cause. Required repair is genuine registered-origin owned subprocess setup preserving actual nested-writer Stop/unknown usage assertions. Changing expected terminal state or dropping the physical controls would not satisfy the existing contract.

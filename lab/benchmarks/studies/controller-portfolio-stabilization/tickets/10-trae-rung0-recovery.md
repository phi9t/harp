# 10 trae-rung0-recovery: Complete the Rung 0 recovery matrix

Type: feature
Status: open
Owner:
Blocked by: 09
Claimed:

## Parent

`docs/efforts/controller-portfolio-stabilization/SPEC.md`

## What to build

Repeat Rung 0 with deterministic loss, controller restart and Lead replacement at every authority and external-effect transition, proving idempotent recovery or retained Quarantine without duplicate workers or effects.

## Acceptance
- [ ] Faults cover assignment prepared, delivery acknowledgement lost, provider acknowledgement, first tool effect, Candidate submission, verification, acceptance, disposable ref advance and cleanup.
- [ ] Every replacement fences the prior RoleEpoch before issuing commands and preserves Attempt, RuntimeBinding, Candidate and operation identities.
- [ ] Unknown delivery never causes blind resubmission; unknown effects retain capacity and enter Quarantine.
- [ ] Replayed operations are idempotent and changed generations fail closed rather than adopting substitute state.
- [ ] Independent evidence shows no duplicate worker, tool effect, Candidate, acceptance, protected-ref update or cleanup.

## Comments

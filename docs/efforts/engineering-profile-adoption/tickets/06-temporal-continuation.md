# 06 temporal-continuation: Qualify Temporal continuation around canonical native operations

Type: task
Status: open
Owner:
Blocked by: 03, 04
Claimed:

Triage: needs-triage

## Acceptance

- [ ] Worker replacement, lost notifications and Continue-As-New retain waits/cursors while canonical operations reconcile once. Replay across the supported version matrix never grants authority or duplicates native effects.

## Requirements

If selected in 01, connect Temporal to the authoritative store and qualified gateway. Persist observation checkpoints; use bounded retries and explicit Activity budgets. Test lost reply after native start and after completion, controller restart, terminal failed turn and cancellation. Version workflow changes and retain replayable histories. Otherwise revise this contract under owner decision.

## Exclusions

No retry of an unknown external effect; no production Temporal service installation or blanket replay readiness claim.

## Evidence

Real owned server and canonical store, explicit SDK replay, crash injection and exact native identity evidence. Check live jobs survive replacement and stale epochs cannot commit; independently review all terminal/unknown states.

## Comments

# 30 protected-integration: physical exact-candidate CAS and recovery

Type: task
Status: open
Owner:
Blocked by: 11, 27, 31
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Build the actual protected integration gate beyond11's fixture handoff.
Journal intent before physical target mutation, serialize conflicting operations,
validate exact candidate/tree/evidence/approval and expected predecessor, and
enforce fast-forward-only ancestry. Follow
[v0.2 reconciliation](../v02-reconciliation.md).

Success-before-PG-receipt and lost acknowledgments require attributable recovery;
observing a target at candidate alone cannot identify the effect's operation.
Hold conflicts while unresolved. Qualify gate exclusivity or explicitly handle
out-of-band writers as unknown; no forced ref/reset/rebase to repair mismatch.

## Interface contract

Versioned integration grant/execute/inspect/reconcile binds scope/epochs,
IntentRevision, candidate/tree, verifier policy/environment, approval digest,
target identity and expected predecessor. One durable journal/receipt owns each
physical effect and its attribution. Preparation and execution are distinct.
Changed candidate/base/arguments require fresh proof/approval. Private fixture
targets exercise real CAS independently from production branch permissions.

## Exclusions

No main mutation in fixtures, shared branch rewrite/deletion, direct worker
landing, remote exactly-once claim or inference of effect attribution from
current ref equality. No conversion of the owner's checkout.

## Acceptance

- [ ] Actual private Git protected-target RED/GREEN: moved predecessor refuses CAS and preserves both histories (F19).
- [ ] Inject process death/lost acknowledgment after ref effect and before domain receipt; recover attributable operation or precise quarantine without another mutation (F20).
- [ ] Concurrent grant/cancel/revocation is serialized; stale scope/approval/evidence cannot mutate target; earlier granted receipt is reconciled under recorded policy.
- [ ] Candidate/tree/base or test/environment policy change refuses proof reuse; exact verified candidate and ff ancestry required.
- [ ] User-like untracked-file/path collision is preserved or explicitly refused; no checkout reset/discard (F21).
- [ ] GC during unknown integration keeps candidate, effect journal and proof references; capacity stays held until reconciliation.
- [ ] Independent conformance/branch review, frozen physical effect evidence and applicable checks accepted before authorizing any real landing service.

## Comments

- 2026-10-03T17:30:00Z lead: Current artifacts.rs prepare_integration creates a fixture candidate/handoff only; this ticket adds physical gate proof without claiming it already exists.

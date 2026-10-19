# 05 protected-promotion: Qualify protected exact-candidate promotion and receipts

Type: task
Status: open
Owner:
Blocked by: 03
Claimed:

Triage: needs-triage

## Acceptance

- [ ] Only the final integrated Candidate verified against current trunk and trusted contract advances trunk conditionally, with a unique create-only receipt. Lost acknowledgment, stale base and conflicting receipt cannot be accepted by guesswork.

## Requirements

Use a gate-owned bare repository, serial durable lane and private verification execution. Bind evidence to final Candidate/base/recipe; rebuild and reverify if trunk moves. Journal prepared/committing/effect-reconciled states. Define cancellation ordering and gate failover fencing. Synchronize operator checkout separately and retain dirty/untracked collisions.

## Exclusions

No production trunk promotion or push grant; no reuse of pre-reintegration green checks; no database/Git atomicity claim.

## Evidence

Disposable real Git crash/ack faults at each boundary; stale trunk, policy-tampering tests, conflicting marker and old-gate-resume scenarios. Prove protected refs/credentials absent from worker execution.

## Comments

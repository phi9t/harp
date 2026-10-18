# 07 concurrent-frontier: Qualify two-worker frontier and stale-base reintegration

Type: task
Status: open
Owner:
Blocked by: 05, 06
Claimed:

Triage: needs-triage

## Acceptance

- [ ] Two independent workers share conserved budgets and land serially; a moving trunk forces the second Candidate to rebuild/reverify. Dependents start only from accepted landed dependencies.

## Requirements

Reuse broker claims and gate reservations. Test same-ticket claims, nonoverlapping and semantically conflicting work, stale result after replacement, capacity exhaustion and cancellation. Track private workspace generations and build directories; live jobs prevent cleanup. Lead rehydration preserves healthy workers.

## Exclusions

No speculative dependency stacks, untracked recursive swarm or throughput superiority claim.

## Evidence

Deterministic two-worker Trial Repository first; separately authorized bounded native trial later. Evidence maps each acceptance outcome to exact candidates and receipts and shows rejection paths.

## Comments

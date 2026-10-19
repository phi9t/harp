# 09 operational-qualification: Qualify restore, fencing, retention and deployment readiness

Type: task
Status: open
Owner:
Blocked by: 08
Claimed:

Triage: needs-triage

## Acceptance

- [ ] An explicit readiness decision states which qualification rungs passed and which remain blocked. Database restore and gate/runtime failover reconcile external effects and fence old authority before dispatch resumes.

## Requirements

Define single-host footprint, remote-service security if needed, backup/restore, artifact retention, release/upgrade ownership and rollback. Qualify storage exhaustion, lost resources, authority-domain rotation, old gate resumption, active jobs during cleanup and lab runner termination deadlines. Measure useful accepted outcomes, rework and recovery, not agent count.

## Exclusions

No service deployment, broad provider spend, production workload or GPU campaign without its own assignment.

## Evidence

Real disposable restore/failover exercises and versioned qualification matrix; retain failed/unknown cases, cleanup-pending separately and exact input identities. Independent Standards+Spec plus operator readiness decision.

## Comments

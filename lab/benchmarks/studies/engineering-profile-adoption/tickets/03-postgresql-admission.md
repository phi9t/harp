# 03 postgresql-admission: Qualify PostgreSQL admission and migration boundary

Type: task
Status: open
Owner:
Blocked by: 02
Claimed:

Triage: needs-triage

## Acceptance

- [ ] If PostgreSQL is selected, concurrent duplicate commands create one Attempt/reservation/operation/outbox; changed arguments and stale revisions reject; faulted transactions leave no partial state or capacity leak.

## Requirements

Define schema ownership and versioned expand/migrate/contract before data movement. Reuse only reviewed kernel mechanisms. Fence expired/revoked epochs, enforce conserved reservations and record rejection receipts. Retaining I2 instead requires a reviewed rewritten contract, not pretending this PostgreSQL ticket passed.

## Exclusions

No production database migration, blind I3 branch merge or held transaction across provider/test work.

## Evidence

Real disposable PostgreSQL concurrency and write-stage faults plus compatibility/rollback proof. Retain exact image/source identities and cleanup. Independent review before any migration assignment.

## Comments

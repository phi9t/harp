# 07 workflow-bridge: define the PostgreSQL and Temporal handoff protocol

Type: task
Status: resolved
Owner:
Blocked by: 02, 05
Claimed:
Label: wayfinder:grilling
Parent: ../map.md

## Question

How do committed commands/outbox rows enter stable Workflow identities while
PostgreSQL remains authoritative for accepted domain transitions? Define
concurrent workflow start, acknowledgement levels, processed-command cursor,
terminal workflow delivery, missed wakeup recovery, workflow versioning and
Continue-As-New handoff. Decide what needs a approved runnable protocol spike.

## Acceptance

- [x] State which actor owns each cursor/acknowledgement and how duplicates are rejected.
- [x] Define terminal-workflow and rollover wakeup recovery without command loss.
- [x] Specify a real PostgreSQL/Temporal test seam and bounded retry/replay criteria.

## Comments

## Answer

The owner approved the recommended baseline on 2026-10-01T07:20:06Z. The reviewed
[SPEC.md](../SPEC.md) supplies this kernel design contract;
[decision](../decisions/20261001-approved-baseline.md) records scope, public
test seam, evidence and live-probe limitations. Resolution is design/admission
authority, not implementation or real provider behavioral acceptance.

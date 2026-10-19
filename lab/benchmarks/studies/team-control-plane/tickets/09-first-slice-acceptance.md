# 09 first-slice-acceptance: select the first buildable durability milestone

Type: task
Status: resolved
Owner:
Blocked by: 03, 06, 07, 08
Claimed:
Label: wayfinder:grilling
Parent: ../map.md

## Question

Should the first acceptance prove the durable PostgreSQL/Temporal kernel with
a fault-injected fake runtime before real Trae, or include live Trae from the
start? Define the minimal assign/start-observe/submit/verify loop and failure
matrix. Mark fake-kernel versus live-adapter proof explicitly; do not present
a PostgreSQL-only preparatory slice as the completed architecture.

## Acceptance

- [x] Owner selects milestone scope and existing/new public behavioral seams.
- [x] Every required failure has a measurable recover-or-quarantine verdict and proof class.
- [x] Produce a handoff to reviewed specification and blocker-linked implementation tickets.

## Comments

## Answer

The owner approved the recommended baseline on 2026-10-01T07:20:06Z. The reviewed
[SPEC.md](../SPEC.md) supplies this kernel design contract;
[decision](../decisions/20261001-approved-baseline.md) records scope, public
test seam, evidence and live-probe limitations. Resolution is design/admission
authority, not implementation or real provider behavioral acceptance.

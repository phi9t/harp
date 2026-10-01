# 05 authority-boundary: reconcile broker state with Git intent and human integration

Type: task
Status: resolved
Owner:
Blocked by:
Claimed:
Label: wayfinder:grilling
Parent: ../map.md

## Question

What does PostgreSQL own while current policy makes Markdown specs/tickets
canonical? Decide whether Git remains approved planning input with immutable
accepted revision bindings in the broker, or whether mission/task authority
migrates to the database with explicit policy changes and Git exports.
Clarify planning-ticket status versus runtime-task status without duplicate
authority. Current human integration remains in force until separately changed.

## Acceptance

- [x] Owner chooses the authority boundary and records which state each store owns.
- [x] Specify approval binding, stale-revision handling and allowed projection/import behavior.
- [x] Enumerate required versioned policy changes; grant no implicit automated merge authority.

## Comments

## Answer

The owner approved the recommended baseline on 2026-10-01T07:20:06Z. The reviewed
[SPEC.md](../SPEC.md) supplies this kernel design contract;
[decision](../decisions/20261001-approved-baseline.md) records scope, public
test seam, evidence and live-probe limitations. Resolution is design/admission
authority, not implementation or real provider behavioral acceptance.

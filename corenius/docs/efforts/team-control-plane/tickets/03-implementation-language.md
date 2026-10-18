# 03 implementation-language: choose the broker and workflow-worker stack

Type: task
Status: resolved
Owner:
Blocked by: 02
Claimed:
Label: wayfinder:grilling
Parent: ../map.md

## Question

Use Rust for both broker/gateway and Temporal worker, or a Rust broker with a
Python workflow worker? Prefer the existing Rust stack subject to a pinned
SDK replay/restart feasibility check. Current official sources support both;
an unsupported claim that Rust has no public SDK cannot decide this question.

## Acceptance

- [x] Owner chooses the preferred stack and permitted fallback.
- [x] Record exact spike question, scope and SDK/version compatibility checks before running it.
- [x] Record replay/versioning criteria and deployment consequences as the resolution.

## Comments

## Feasibility evidence

[Model-free Rust SDK spike](../research/temporal-feasibility.md) compiled SDK1.0.0
and exercised worker restart, Continue-As-New, successful actual offline replay
and rejected incompatible replay. Owner stack choice remains outstanding;
production compatibility and the DB bridge require later acceptance.

## Answer

The owner approved the recommended baseline on 2026-10-01T07:20:06Z. The reviewed
[SPEC.md](../SPEC.md) supplies this kernel design contract;
[decision](../decisions/20261001-approved-baseline.md) records scope, public
test seam, evidence and live-probe limitations. Resolution is design/admission
authority, not implementation or real provider behavioral acceptance.

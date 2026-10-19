# 01 runtime: Adopt durable replay and supervised activity runtime

Type: feature
Status: claimed
Owner: durable-runtime
Blocked by:
Claimed: 2026-10-13T03:41:40Z expires 2026-10-14T03:41:40Z

Triage: ready-for-agent

## Acceptance
- [x] Adopt reference public APIs and 35 fault tests under controller/durable without import path hacks.
- [x] Preserve temporary environment policy and fix manual-effect cancellation/sibling-failure uncertainty.
- [x] Record red/green evidence and reconcile independent design critique.

## Requirements
SPEC.md requirements 1-5; keep execution records separate from existing Git/ticket authority.

## Exclusions
DBOS, Rust port, publication, sandbox replacement.

## Comments
- 2026-10-13T03:41:40Z durable-runtime: claimed until 2026-10-14T03:41:40Z
- 2026-10-13T04:08:09Z durable-runtime: Original 35 port and all 43 adapted runtime tests passed. Independent critique reconciled; reports in ../evidence/runtime-implementation.md and design-critique.md.
- 2026-10-13T04:08:09Z codex: Tracker closure refused because no accepted, landed Candidate exists. Git publication and human merge are outside current authorization.

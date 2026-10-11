# 02 temporal-recovery: Temporal durable wait, worker restart and retained replay labs

Type: task
Status: open
Owner:
Blocked by: 01
Claimed:

Triage: needs-triage

## Acceptance
- [x] Detailed walkthrough covers durable waits, lost hints, SIGKILL/replacement,
  Activity reply loss, Continue-As-New, observed identity and replay.
- [x] Fresh real PostgreSQL/Temporal fixtures and actual SDK replay pass.
- [x] Exact owned PostgreSQL removal and Temporal listener closure are retained.
- [x] Evidence separates worker restart, server rollover and retained replay.
- [x] Parent completes independent Standards and Spec review.

## Requirements

Use the existing public Temporal fixtures. Keep PostgreSQL canonical.
Explain checkpoint history barriers, cursor continuity, generation fencing,
actual Activity retry and source-backed proof limitations.

## Exclusions

No fake server, native provider qualification claim, server restart/HA proof,
product semantics, deployment, Git publication or Trondheim integration.

## Evidence

See [Temporal walkthrough](../TEMPORAL.md) and
[evidence summary](../evidence/summary.md). The learning content is integrated
with passing native evidence. Independent review found no remaining blocking
finding. Ticket 01 awaits accepted landing, so this ticket stays open until
tracker closure. The bounded lab execution
was authorized by the user's explicit experiment request; it is not a product
implementation through unresolved ticket dependencies.

## Comments

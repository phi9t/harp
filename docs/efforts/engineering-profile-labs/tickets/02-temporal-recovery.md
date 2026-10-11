# 02 temporal-recovery: Temporal durable wait, worker restart and retained replay labs

Type: task
Status: resolved
Owner: codex-lab-closure
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
finding. The exact reviewed lab Candidate
`00f7f2ee94557d4f3cb9402d6d519d2a9798283c` landed on local main after the
full repository gate passed. After 01 closed with accepted main ancestry, this
ticket closed through normal tracker transitions. This was a bounded learning
experiment, not a production implementation through unresolved dependencies.

## Comments
- 2026-10-11T20:35:20Z codex-lab-closure: claimed until 2026-10-12T20:35:20Z
- 2026-10-11T20:35:20Z codex-lab-closure: resolved

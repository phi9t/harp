# 01 postgres-transactions: PostgreSQL transactional claims, receipts, revisions and fencing labs

Type: task
Status: resolved
Owner: codex-postgres-temporal-labs
Blocked by:
Claimed:

Triage: needs-triage

## Acceptance
- [x] Detailed walkthrough covers transactional claims, idempotent receipts,
  distinct-command revision conflicts, rollback and authority fencing.
- [x] Fresh real PostgreSQL kernel suite passes with private locked/offline Cargo.
- [x] Exact disposable container identity and removal are retained.
- [x] Proof limits distinguish database admission from external effects.
- [x] Parent completes independent Standards and Spec review.

## Requirements

Use the existing public kernel fixtures as the approved learning seam.
Explain each case's setup, expected result, source mechanism and failure boundary.
Record source revision, command, digest and cleanup evidence.

## Exclusions

No product semantics, real database migration, deployment, Git publication,
production-readiness claim or Trondheim integration.

## Evidence

See [PostgreSQL walkthrough](../POSTGRESQL.md) and
[evidence summary](../evidence/summary.md). The learning content is integrated
with passing native evidence. Independent review found no remaining blocking
finding. The exact reviewed lab Candidate `00f7f2ee94557d4f3cb9402d6d519d2a9798283c`
landed on local main after the full repository gate passed. Acceptance records
and ancestry now support tracker closure; adoption remains unqualified.

## Comments
- 2026-10-11T19:19:20Z codex-postgres-temporal-labs: claimed until 2026-10-12T19:19:20Z
- 2026-10-11T20:35:20Z codex-postgres-temporal-labs: resolved

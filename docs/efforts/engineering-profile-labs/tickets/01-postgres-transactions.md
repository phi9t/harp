# 01 postgres-transactions: PostgreSQL transactional claims, receipts, revisions and fencing labs

Type: task
Status: claimed
Owner: codex-postgres-temporal-labs
Blocked by: 
Claimed: 2026-10-11T19:19:20Z expires 2026-10-12T19:19:20Z

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
finding. The ticket remains claimed pending an accepted, landed Candidate;
local passing tests and review do not supply a promotion receipt.

## Comments
- 2026-10-11T19:19:20Z codex-postgres-temporal-labs: claimed until 2026-10-12T19:19:20Z

# 21 checkout-preflight: Share checkout preflight before publication

Type: task
Status: open
Owner:
Blocked by: 19
Claimed:

Triage: needs-triage

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [ ] Core and legacy reject real unfinished merge, cherry-pick, revert, rebase and sequencer state before any push.
- [ ] Empty cherry-pick with clean porcelain leaves remote/local refs and operator files unchanged.
- [ ] Share checkout safety without removing legacy support for disjoint unstaged operator changes; rerun preflight before recovered FF.

## Comments

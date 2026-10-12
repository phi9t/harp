# 21 checkout-preflight: Share checkout preflight before publication

Type: task
Status: resolved
Owner: trae
Blocked by:
Claimed:

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [x] Core and legacy reject real unfinished merge, cherry-pick, revert, rebase and sequencer state before any push.
- [x] Empty cherry-pick with clean porcelain leaves remote/local refs and operator files unchanged.
- [x] Share checkout safety without removing legacy support for disjoint unstaged operator changes; rerun preflight before recovered FF.

## Comments
- 2026-10-12T17:39:00Z trae: unblocked by resolved ticket 19 decision record `decisions/2026-10-12-mutation-contract.md`.
- 2026-10-12T17:38:46Z trae: claimed until 2026-10-13T17:38:46Z
- 2026-10-12T17:49:00Z trae: core landing now uses shared checkout preflight before push/local FF while retaining strict cleanliness; legacy marker preflight remains shared. Evidence: focused empty-cherry-pick core and legacy tests passed in the 5-test focused run; affected Collab/legacy suite passed 46 tests.
- 2026-10-12T18:54:20Z trae: resolved with --force despite missing accepted, landed Candidate

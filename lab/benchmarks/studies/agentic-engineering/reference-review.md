# Reference design and specification review

Date: 2026-10-01.
Scope: design/specification only, not implementation validation.
Read-only separate-context reviewers received artifact paths and repository
guidance without the design conversation. Neither reviewer changed files or
launched additional models.

## Design critique

Reviewer: `design_critique`.

- Bootstrap ambiguity: resolved in the spec by separately pinning the execution
  baseline and the common target base, requiring offline containment, timeout
  and emergency-shutdown probes before live runs. These do not count as proof
  of either candidate's implementation. Failed prerequisites block dispatch.
- No-push landing is not preparation: resolved by exact-tip gate results and
  a handoff without real mainline movement. Actual landing tests use temporary
  fixture repositories only.
- Dehydration requires commit authority: explicit prerequisite for baseline,
  implementation, evidence and dehydration commits before live demonstrations.
- Enforcement, cancellation scope, model-call accounting and measured capsule
  identity: specified with explicit limits and required proof.

## Adversarial specification review

Reviewer: `spec_review`.

Initial findings and resolutions:

1. Time cap only killed launcher: cap expiry must invoke owned shutdown across
   capsule sessions; required expiry fixture includes TERM-resistant descendants.
2. Spawn/registration crash could escape accounting: launch intent is persisted
   first; uncertain starts conservatively consume attempts and active capacity;
   fault-injection fixtures cover startup transitions.
3. GC race described unsupported launch-into-existing-tree behavior: corrected
   to the actual create-before-register interval, protected from reservation
   through active registration, with a public-CLI race fixture.
4. Recovery duplicated implementation-ticket responsibility: tickets 10/11 own
   committed handoffs; tickets 13/14 each own one independent fresh-session
   check, blocked only by their own candidate proof.

Additional safeguards: identity-change-between-check-and-signal fixture, stable
process handles or refusal, a real bubblewrap namespace fixture and explicit
authority to commit the common target base.

Re-review result: "No unresolved blockers found in the updated spec and ticket
graph." Reviewer explicitly limited this result to specification review;
ticket 09 must establish concrete interfaces and prove baseline feasibility.

## Coverage and remaining gates

- R1: 08, 12; R2: 09, 10, 11; R3/R4/R5: 09, 10, 11.
- R6: 09, 10, 11, 12; R7: 09, 10, 11, 13, 14, 15; R8: 08, 10, 11, 12, 15.
- Ticket graph and CLI test seam await owner review.
- Product implementation, live demonstrations and Git commits have not run.
- Owner approval of this spec is not evidence of passing implementation tests.
- Independent implementation review and fresh repository gates remain required.

Local planning checks: tracker lint for the existing effort passes. New records
remain uncommitted in `codex/reference-design` and are not in mainline or a
fresh clone until explicitly authorized commits/integration occur.

# 07 reference-quality-design: settle the design

Type: task
Status: open
Owner: lead
Blocked by:
Claimed:

## Intent

Make Corenius the sterling guidance and executable reference for agentic
engineering and multi-agent collaboration with Codex or Claude as lead and
traecli workers. Continue the existing effort rather than create a competing
roadmap.

## Route and authority

`ask-matt` routes this architectural work through `grill-with-docs`, supported
by brainstorming and domain modeling, then reviewed tracker specification and
blocker-linked implementation tickets. The installed Matt design instructions
are applied directly under the repository's invocation-availability adapter.
Design records are authorized. Implementation, commits, pushes and mainline
integration are not authorized by this ticket.

## Accepted decisions

See `../decisions/20261001-reference-quality.md`: workspace first with explicit
dependencies; guidance plus an executable workflow; independent demonstrations
by both lead harnesses, including recovery into a fresh session.

See `../decisions/20261001-reference-authority-and-proof.md`: bounded lead
autonomy, reproduced acceptance plus separate review, and a real improvement
demonstrated with controlled failures under both lead harnesses.

## Current design frontier

Design approved. The reviewed specification is `../reference-spec.md`; the
proposed graph is `../reference-ticket-breakdown.md`; review findings and their
resolutions are `../reference-review.md`. Owner review now covers the public-CLI
behavioral seam, specification and ticket granularity/blockers. Implementation
and live-run/Git authority are separate remaining gates. Concrete interfaces
and baseline probes are ticket 09's deliverable.

The owner subsequently supplied a larger external-broker RFC and requested
capture plus implementation investigation. That discovery lives at
`../../team-control-plane/map.md`; the preserved source is
`../../../rfcs/0001-durable-team-control-plane.md`. The reference specification
and graph remain prior draft inputs, not approved execution tickets. Resume or
revise them after the control-plane authority/first-slice decisions; do not
silently replace the approved design or implement both ledgers.

## Acceptance

- [x] Design frontier exhausted, with unresolved questions represented by
  bounded decision tickets rather than assumptions.
- [x] Owner confirms shared understanding of the design.
- [x] Specification records requirements, exclusions, authority and evidence
  criteria in the configured durable tracker.
- [x] Independent design critique and adversarial spec review reconciled.
- [ ] Blocker-linked implementation tickets prepared for isolated execution.

## Comments

- 2026-10-01T00:00:00Z lead: date-only record, time unspecified; owner accepted the first three design recommendations, recorded in the decision above. Worktree `codex/reference-design`, based on `963f58b`; records remain uncommitted pending explicit Git authority.
- 2026-10-01T03:38:51Z lead: owner accepted Q4–Q6; recorded authority, verification and demonstration decisions in 20261001-reference-authority-and-proof.md. Recomputed the frontier for evidence, documentation and budgets.
- 2026-10-01T03:45:28Z lead: owner accepted Q7–Q9. Recorded the decisions and drafted reference-design.md with explicit remaining proposals for owner confirmation. No product code or canonical policy changed.
- 2026-10-01T04:01:26Z lead: owner approved the complete design. Wrote the tracker spec and proposed ticket graph; separate-context critique and adversarial review were reconciled, with re-review reporting no unresolved blockers. Owner review of the spec, CLI seam and graph remains pending; no implementation or live model runs.

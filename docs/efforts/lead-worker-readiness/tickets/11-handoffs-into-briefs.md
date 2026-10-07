# 11 handoffs-into-briefs: A finding left for a later ticket reaches that ticket's worker

Type: feature
Status: resolved
Owner: worker-11
Blocked by: 05
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 42, 43. A worker can leave a finding addressed to a named later ticket as a structured line in its own ticket; the lead can add one too. When the addressed ticket is launched, its brief includes every finding addressed to it, with the source ticket. The tracker validates the section.

## Declared paths
- `tools/agents/tracker.py`, the lead launch command, their tests
- `docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md`

## Acceptance
- [x] An optional ticket section holds findings as `- to NN: text`; lint accepts it, and reports a malformed line, an unknown target and a self target.
- [x] A command appends a finding and a command lists the findings addressed to a ticket, with a JSON form.
- [x] Launching the addressed ticket puts those findings, with their source ticket, in the brief; the launch record shows them.
- [x] Claiming and resolving a ticket that has the section leave it intact; tickets without it stay valid.

## Comments
- 2026-10-05T11:15:52Z worker-11: claimed until 2026-10-07T11:15:52Z
- 2026-10-05T11:48:15Z worker-11: resolved

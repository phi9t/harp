# 07 integration-branch: An effort lands on its own integration branch

Type: feature
Status: resolved
Owner: worker-07
Blocked by: 02
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Story 29. An effort accumulates accepted Candidates on an integration branch that the lead may advance; only the owner moves the mainline. Launch (ticket 05) and landing (ticket 02) take the integration branch from the effort's record instead of a flag repeated on every command. When the mainline has moved, a command reports that the integration branch is behind and lists the Candidates that would need re-verification on a re-created integration branch; it does not rebase or re-create anything by itself.

## Declared paths
- `tools/agents/effort.py`, `tools/agents/land.py`, their tests
- `bench/readiness/` (scenario)

## Acceptance
- [x] An effort record names its integration branch; landing a ticket of that effort advances that branch and never the mainline.
- [x] An attempt by the lead tooling to land a ticket Candidate directly on the mainline is refused unless an explicit owner flag is given, and the flag's use is recorded.
- [x] When the mainline has advanced past the integration branch's fork point, a status command says so and lists the landed Candidates in order; no ref changes.
- [x] The scenario runs in the lead-driven mode and matches its expectation.

## Comments
- 2026-10-05T11:15:55Z worker-07: claimed until 2026-10-07T11:15:55Z
- 2026-10-05T12:15:00Z worker-07: resolved

# 15 traecli-worker-parity: The same scenarios and a second live run with traecli workers

Type: feature
Status: open
Owner:
Blocked by: 12, 14
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Second runtime in the owner's order. Run every deterministic scenario with the traecli runtime selected, and repeat the live trial shape of ticket 14 with traecli workers on a fresh small effort. Differences from the Codex results are recorded per scenario.

## Declared paths
- `bench/readiness/` only for runtime-specific fixtures
- `docs/efforts/lead-worker-readiness/evidence/`

## Acceptance
- [ ] Every deterministic scenario has a row for the traecli runtime.
- [ ] A live trial with traecli workers has run with the owner's recorded go-ahead, and its rows are in the results table.
- [ ] A short comparison lists each scenario whose outcome or interventions differ between the two runtimes, with the reason where known.

## Comments

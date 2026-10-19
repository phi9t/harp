# 14 live-trial-lead-driven: Live trial of the lead-driven instantiation on a real small effort

Type: feature
Status: open
Owner:
Blocked by: 04, 06, 07, 09, 10, 11, 12, 13, 16, 17, 18, 19, 20, 21, 22
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 8, 13, 37, 45. Run by the lead, not delegated to one worker. A real small Corenius effort of at least three tickets with one blocking edge, two workers in parallel and one review-driven rework, with Codex workers, landed on an integration branch. One ticket is seeded so that its obvious implementation breaks a stated spec requirement, to observe whether review rejects it. Every intervention by a supervising agent or the owner is recorded against the scenario it rescued. The owner's go-ahead for the run is recorded before launch.

## Declared paths
- `docs/efforts/lead-worker-readiness/evidence/` (trial record, running notes)
- the trial effort's own directory under `docs/efforts/`

## Acceptance
- [ ] The trial effort, its tickets and its integration branch exist, and the owner's go-ahead is recorded as a decision.
- [ ] Two workers' running intervals overlap, shown from the run records.
- [ ] The seeded ticket's first Candidate is rejected at review with the spec requirement cited, reworked by the same worker session, re-reviewed and landed; or the miss is recorded as a failure of story 8.
- [ ] Every landed commit equals an accepted Candidate, checked from the acceptance records and the integration branch history.
- [ ] Every worker process is accounted for at the end; the external-state report is attached for each run.
- [ ] The results table has a live row for every story the trial exercised, and the running notes list each intervention.

## Comments

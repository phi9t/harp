# 03 acceptance-record: Acceptance is a recorded decision that landing requires

Type: feature
Status: resolved
Owner: worker-03
Blocked by: 02
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 6, 7, 9, 10, 11, 17. Passing tests are not acceptance. Add an acceptance record for the lead-driven instantiation: a tracked file per ticket under the effort's `evidence/` naming the exact Candidate, the reviewer, the result on each review axis (repository standards; ticket and spec), each required check with its result, any owner waiver, and whether the reviewer was a context other than the lead.

Landing with a ticket reference is refused unless an acceptance record exists for exactly that Candidate with both axes passed and every required check passed or waived by the owner. What a worker writes in its ticket file (status, ticked items) has no effect on this. Claim sampling outcomes recorded with the claims tool are linked from the record.

Scenarios: no record (refused); record for a different commit (refused); failing required check without waiver (refused), with waiver (lands); worker ticks every item and sets its own status (no effect, still refused without a record).

## Declared paths
- `tools/agents/` (a new small acceptance module and its tests; `land.py` for the precondition; `claims.py` only to link outcomes)
- `tools/agents/checks.d/` (one script for the new suite)
- `bench/readiness/` (scenarios)

## Acceptance
- [x] A command writes an acceptance record for a ticket and Candidate, and refuses to mark it accepted unless both axes and every required check are recorded.
- [x] Landing with a ticket reference is refused when the record is missing, names another commit, lacks an axis, or has a failing check without an owner waiver; each refusal names the reason.
- [x] With a recorded owner waiver the same Candidate lands, and the waiver is in the record.
- [x] A worker's edits to its ticket's status line and acceptance items do not change the outcome of any scenario.
- [x] The record states whether the reviewer was the lead or another context, and a summary command counts tickets per effort reviewed by a non-lead.
- [x] Recorded claim-sampling outcomes for the worker's report are reachable from the acceptance record.
- [x] All scenarios run in the lead-driven mode and match their expectation; the new suite runs from the repository gate.

## Comments
- 2026-10-05T09:51:12Z worker-03: claimed until 2026-10-07T09:51:12Z
- 2026-10-05T11:15:52Z worker-03: resolved

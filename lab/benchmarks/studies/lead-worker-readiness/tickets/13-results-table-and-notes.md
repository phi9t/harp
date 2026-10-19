# 13 results-table-and-notes: Results and running notes are comparable across instantiations

Type: feature
Status: resolved
Owner: worker-13
Blocked by: 01
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 45 (recording part), 46, 47. The lead-driven mode writes one result row per scenario with: instantiation, runtime, outcome (passes, passes with intervention, not applicable by design, fails), each intervention with who made it and what it did, elapsed time, model usage where exposed and otherwise "unknown". A report command prints one table per scenario across instantiations, showing "not yet run" where there is no row, and a summary of the core set from decision item 3. A notes command appends a timestamped entry (obstacle, intervention or rule added) to the effort's running notes.

## Declared paths
- `bench/harness/results.py` and the lead-driven module, their tests
- `docs/efforts/lead-worker-readiness/evidence/` (the notes file format only)

## Acceptance
- [x] Each scenario run appends one row with the fields above; a run recorded with an intervention has outcome `passes with intervention` and lists it.
- [x] The report prints every scenario defined under the readiness directory against the three instantiations, with `not yet run` for missing rows and `not applicable by design` rows showing their reason.
- [x] The report states, for the core set, how many scenarios pass, pass with intervention, fail and are not yet run.
- [x] Deterministic and live rows are never merged into one figure.
- [x] The notes command appends an entry with time, kind and text, and refuses an unknown kind.

## Comments
- 2026-10-05T09:06:03Z worker-13: claimed until 2026-10-07T09:06:03Z
- 2026-10-05T09:26:12Z worker-13: resolved

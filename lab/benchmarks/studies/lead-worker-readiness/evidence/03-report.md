## Built

Implemented a lead-driven acceptance record path for ticket Candidates. The new `tools/agents/acceptance.py`
command writes per-ticket acceptance records, summarizes non-lead review coverage, and links existing claim
sampling outcomes.

`tools/agents/land.py` now refuses ticket-referenced landing unless the exact Candidate has an accepted
record with both review axes passed and every required check passed or owner-waived. Lead-driven readiness
scenarios cover missing, wrong-Candidate, failing-check, waived-check, and worker-ticket-tamper cases.

Review 1 repairs: the lander now reads acceptance from the target branch checkout when that checkout
exists; the record/landing order is documented as write-before-land, commit-after-land; claim sampling paths
inside the repository are stored relative to the repository root.

Reintegration repairs: rebased onto `effort/lead-worker-readiness` after tickets 08, 09 and 05 landed, and
merged the lead-driven scenario loader/test changes so `access-profile`, `external-state-report`,
`lead-launch`, and all five acceptance scenarios remain covered.

## Acceptance

- "A command writes an acceptance record for a ticket and Candidate, and refuses to mark it accepted unless both axes and every required check are recorded." Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_acceptance.py` -> OK, 7 tests. Red evidence before implementation: the suite first failed with `ModuleNotFoundError: No module named 'acceptance'`.
- "Landing with a ticket reference is refused when the record is missing, names another commit, lacks an axis, or has a failing check without an owner waiver; each refusal names the reason." Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py` -> OK, 20 tests, and by the readiness loop -> `acceptance-missing-record`, `acceptance-other-candidate`, and `acceptance-failing-check` all passed with `expected=acceptance-refused observed=acceptance-refused`. Red evidence before implementation: `test_land.py` failed with four `AttributeError: module 'land' has no attribute 'ACCEPTANCE_FAILED'` errors.
- "With a recorded owner waiver the same Candidate lands, and the waiver is in the record." Proved by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py` -> OK, 22 tests, including `test_acceptance_waiver_scenario_lands_and_records_waiver`; the readiness loop also printed `passes acceptance-waived-check commit-ready expected=landed observed=landed`.
- "A worker's edits to its ticket's status line and acceptance items do not change the outcome of any scenario." Proved by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py` -> OK, 22 tests, including the `acceptance-ticket-tamper` case; the readiness loop printed `passes acceptance-ticket-tamper commit-ready expected=acceptance-refused observed=acceptance-refused`.
- "The record states whether the reviewer was the lead or another context, and a summary command counts tickets per effort reviewed by a non-lead." Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_acceptance.py` -> OK, 7 tests.
- "Recorded claim-sampling outcomes for the worker's report are reachable from the acceptance record." Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_acceptance.py` -> OK, 7 tests.
- "All scenarios run in the lead-driven mode and match their expectation; the new suite runs from the repository gate." Proved by the full readiness loop over all 11 `bench/readiness/*` scenarios with `TMPDIR` under an absolute `.scratch` temp root -> all printed `passes`; `tools/agents/checks.d/25-acceptance.sh` -> OK, 7 tests. An initial loop using default `/tmp` reached `No space left on device` during the last scenario because `/tmp` was out of inodes; `df -i /tmp` showed 100% inode use.

Additional required checks run:

- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_claims.py` -> OK, 5 tests.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_worker.py` -> OK, 42 tests, with existing subprocess/socket `ResourceWarning` noise while passing.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_tracker.py` -> OK, 12 tests.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py` -> OK, 6 tests.
- `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py && PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_validate.py && PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_run.py && PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_tasks_tracked.py && PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_fleet.py` -> OK. `test_validate.py`, `test_run.py`, and `test_fleet.py` emitted existing `ResourceWarning: unclosed file ... bench/harness/tasklib.py:90` warnings while passing.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/tracker.py lint` -> OK.
- `git diff --check` -> OK.

## Claims

- [R] acceptance records are stored under `docs/efforts/<effort>/evidence/acceptance-<ticket>.jsonl` `tools/agents/acceptance.py:54`
- [R] `acceptance.py record` documents the write-before-land, commit-after-land order `tools/agents/acceptance.py:288`
- [R] acceptance validation rejects records for a different Candidate `tools/agents/acceptance.py:118`
- [R] acceptance validation requires the standards and spec review axes `tools/agents/acceptance.py:122`
- [R] acceptance validation rejects a failing required check without an owner waiver `tools/agents/acceptance.py:136`
- [R] the acceptance record includes the reviewer context field `tools/agents/acceptance.py:230`
- [R] the acceptance record includes claim-sampling linkage `tools/agents/acceptance.py:234`
- [R] claim-sampling report paths inside the repo are stored relative to the repo root `tools/agents/acceptance.py:219`
- [R] the summary command counts tickets reviewed by another context `tools/agents/acceptance.py:275`
- [R] `land.py` exposes a distinct acceptance-failed exit code `tools/agents/land.py:20`
- [R] ticket landing calls acceptance validation before gate execution `tools/agents/land.py:222`
- [R] ticket landing validates acceptance against the target worktree when that worktree exists `tools/agents/land.py:222`
- [R] the land command has `--effort` and `--ticket` options for ticket-referenced landing `tools/agents/land.py:251`
- [R] the land command documents the record as written before landing and committed after landing `tools/agents/land.py:254`
- [R] lead-driven scenarios can declare an acceptance case in `scenario.toml` `bench/harness/lead_driven.py:127`
- [R] lead-driven scenarios can request a separate target worktree `bench/harness/lead_driven.py:152`
- [R] lead-driven landing can invoke `land.py` from the source checkout instead of the target checkout `bench/harness/lead_driven.py:608`
- [R] the lead-driven harness still dispatches `lead-launch` scenarios to the lead-launch runner `bench/harness/lead_driven.py:1083`
- [R] the lead-driven harness preserves the access-profile scenario test `bench/harness/test_lead_driven.py:361`
- [R] the lead-driven harness preserves the external-state-report scenario test `bench/harness/test_lead_driven.py:394`
- [R] the lead-driven harness seeds a disposable effort ticket for acceptance scenarios `bench/harness/lead_driven.py:156`
- [R] the lead-driven harness writes acceptance records in the target checkout `bench/harness/lead_driven.py:545`
- [R] the lead-driven harness allows the acceptance record to be the only uncommitted target-worktree path after landing `bench/harness/lead_driven.py:641`
- [R] the lead-driven harness requires the acceptance record text to remain unchanged after landing `bench/harness/lead_driven.py:649`
- [R] the lead-driven harness passes `--effort` and `--ticket` to `land.py` for acceptance scenarios `bench/harness/lead_driven.py:622`
- [R] the lead-driven harness observes acceptance-refused landings as their own outcome `bench/harness/lead_driven.py:665`
- [R] the acceptance-missing-record scenario expects acceptance refusal `bench/readiness/acceptance-missing-record/scenario.toml:11`
- [R] the acceptance-waived-check scenario exercises an integration target worktree while invoking landing from the source checkout `bench/readiness/acceptance-waived-check/scenario.toml:12`
- [R] the acceptance-waived-check scenario uses mode `waived-check` `bench/readiness/acceptance-waived-check/scenario.toml:19`
- [R] the ticket-tamper fake worker changes the ticket status and acceptance item before committing `bench/readiness/acceptance-ticket-tamper/scripted_participant.py:28`
- [R] the new repository-gate script runs the acceptance suite `tools/agents/checks.d/25-acceptance.sh:3`
- [R] the lander test covers uncommitted acceptance in the target worktree `tools/agents/test_land.py:520`
- [R] the lander test covers committed acceptance making the Candidate stale before acceptance validation `tools/agents/test_land.py:552`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/03-report.md`: required hand-off report for this ticket.

## For later tickets

- to 14: required checks are named by the reviewer when recording; nothing yet ties them to the ticket. The live trial should name each ticket's required checks in the ticket.

## Not checked

- I did not run `tools/agents/check.sh` or cargo, per ticket instruction.
- I did not launch real model workers or spend model budget; all readiness runs used scripted participants and fake binaries.

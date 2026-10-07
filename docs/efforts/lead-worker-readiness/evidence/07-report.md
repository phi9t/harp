## Built

Rebased this ticket branch onto the moved `effort/lead-worker-readiness` tip before the review fixes.

Effort records now name an integration branch. Ticket landing resolves that branch from the effort record by
default, so accepted Candidates advance the effort integration branch while the base branch stays untouched.

Review 1 is fixed: the effort-record resolver now looks in `--repo` first and then every linked worktree.
If one worktree has `docs/efforts/<effort>/EFFORT.md`, it uses that record; if several have it, it prefers
the worktree checked out on the `Integration branch` named by the record and otherwise refuses as ambiguous.
`land.py`, `lead_launch.py`, and `effort.py status` all use that single resolver.

Direct landing to the effort base branch is refused for ticket landings unless `--owner-mainline-override`
names the owner; successful override use appends a JSONL record under the resolved effort evidence directory.

`effort.py status` now lists efforts found in any worktree once, includes the worktree the record was read
from, reports whether the integration branch is behind the base branch, and lists accepted landed Candidates
in integration-branch commit order. The lead-driven readiness harness includes the deterministic
`integration-branch` scenario for this behavior, including a main-checkout command path where the effort
record exists only in the integration worktree.

Red evidence before initial implementation:
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_effort.py EffortTest.test_new_creates_durable_and_scratch EffortTest.test_status_lists_effort EffortTest.test_status_reports_mainline_moved_and_landed_candidates` failed with missing `Integration branch`, missing `integration_behind_mainline`, and missing `landed_candidates`.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py LandTest.test_ticket_landing_uses_effort_integration_branch_by_default LandTest.test_ticket_landing_refuses_mainline_target_without_owner_override_and_records_override` failed because ticket landing still advanced `main` and direct mainline landing returned 0 instead of 8.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py LeadLaunchTest.test_launch_reads_integration_branch_from_effort_record_when_flag_is_omitted` failed because `lead_launch.py launch` still required `--integration`.
- `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_integration_branch_scenario_runs_through_lead_driven_mode` failed before the new scenario handler produced passing case records.

Red evidence before review-1 fixes:
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_effort.py EffortTest.test_status_finds_efforts_from_linked_worktrees` failed with `StopIteration` because status from the main checkout found no effort row.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py LandTest.test_ticket_landing_from_main_checkout_finds_effort_record_in_integration_worktree` failed with exit 64 and `effort record .../docs/efforts/sample/EFFORT.md not found`.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py LandTest.test_ambiguous_effort_records_without_integration_worktree_are_refused` failed with exit 7 instead of exit 64 because the lander did not detect ambiguous effort records.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py LeadLaunchTest.test_launch_from_main_checkout_finds_effort_record_in_integration_worktree` failed with exit 64 and `effort record .../docs/efforts/demo/EFFORT.md not found`.
- `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_integration_branch_scenario_runs_through_lead_driven_mode` failed with `unknown integration branch action 'main-checkout-effort-worktree'` before the scenario handler was added.

## Acceptance

- "An effort record names its integration branch; landing a ticket of that effort advances that branch and never the mainline."
  Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py` -> OK, 24 tests, including `test_ticket_landing_uses_effort_integration_branch_by_default` and `test_ticket_landing_from_main_checkout_finds_effort_record_in_integration_worktree`; and by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/lead_driven.py integration-branch --out "$tmp"` -> all six cases passed, including `land-default-integration` and `main-checkout-effort-worktree`.

- "An attempt by the lead tooling to land a ticket Candidate directly on the mainline is refused unless an explicit owner flag is given, and the flag's use is recorded."
  Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py` -> OK, 24 tests, including `test_ticket_landing_refuses_mainline_target_without_owner_override_and_records_override`; and by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/lead_driven.py integration-branch --out "$tmp"` -> `mainline-refused` and `mainline-owner-override` both passed.

- "When the mainline has advanced past the integration branch's fork point, a status command says so and lists the landed Candidates in order; no ref changes."
  Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_effort.py` -> OK, 13 tests, including `test_status_reports_mainline_moved_and_landed_candidates` and `test_status_finds_efforts_from_linked_worktrees`; and by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/lead_driven.py integration-branch --out "$tmp"` -> `moved-mainline-status` and `main-checkout-effort-worktree` both passed.

- "The scenario runs in the lead-driven mode and matches its expectation."
  Proven by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py` -> OK, 25 tests, including `test_integration_branch_scenario_runs_through_lead_driven_mode`; and by the required readiness scenario loop -> `integration-branch` printed six `passes` rows.

Review-1 requirements:
- "Resolve an effort's record by looking in the given `--repo` first and then in every other worktree of the repository for `docs/efforts/<effort>/EFFORT.md`."
  Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py` -> OK, 24 tests; `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py` -> OK, 10 tests; and `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_effort.py` -> OK, 13 tests.
- "If several have it, prefer the one whose checked-out branch equals the `Integration branch` the record names, and refuse with a clear message if that does not single one out."
  Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py` -> OK, 24 tests, including `test_ambiguous_effort_records_without_integration_worktree_are_refused`.
- "Tests for the lander, the launch command and status in this layout, and add the layout to your scenario."
  Proven by the new focused tests named above and the `main-checkout-effort-worktree` scenario case.

Additional gate evidence from the final review run:
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py` -> OK, 24 tests.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_acceptance.py` -> OK, 7 tests.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_worker.py` -> OK, 54 tests. This emitted existing `ResourceWarning` lines from Python subprocess/socket cleanup but exited 0.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_tracker.py` -> OK, 17 tests.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py` -> OK, 10 tests.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_effort.py` -> OK, 13 tests.
- Every `bench/harness/test_*.py` suite passed: `test_fleet.py` OK, 9 tests; `test_lead_driven.py` OK, 25 tests; `test_run.py` OK, 17 tests; `test_tasks_tracked.py` OK, 2 tests; `test_validate.py` OK, 9 tests. These emitted existing `ResourceWarning` lines from `bench/harness/tasklib.py:90` but exited 0.
- Every readiness scenario command passed with a fresh `mktemp -d` output directory and cleanup: `acceptance-failing-check`, `acceptance-missing-record`, `acceptance-other-candidate`, `acceptance-ticket-tamper`, `acceptance-waived-check`, `access-profile`, `external-state-report`, `handoff`, `integration-branch`, `land-exact-candidate`, `land-stale-candidate`, `launch-by-edges-and-triage`.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/tracker.py lint` -> exit 0.
- `git diff --check` -> exit 0.

## Claims

- [R] new efforts created by `effort.py new` include an `Integration branch: effort/<slug>` field `tools/agents/effort.py:46`
- [R] the shared effort-record lookup scans every linked worktree for matching `EFFORT.md` files `tools/agents/land.py:115`
- [R] ambiguous effort records prefer the worktree checked out on the record's named integration branch `tools/agents/land.py:152`
- [R] ambiguous effort records are refused when no single integration-branch worktree resolves them `tools/agents/land.py:158`
- [R] `land.py` exposes `resolve_effort_record` as the single resolver for one effort `tools/agents/land.py:164`
- [R] `effort.py status` lists records returned by the shared resolver `tools/agents/effort.py:450`
- [R] `effort.py status` includes the worktree the effort record was read from `tools/agents/effort.py:471`
- [R] accepted landed Candidates are ordered by their commit order on the integration branch `tools/agents/effort.py:137`
- [R] accepted landed Candidates are read from the resolved effort worktree's evidence directory `tools/agents/effort.py:140`
- [R] ticket landing defaults to the resolved effort record's integration branch when `--mainline` is omitted `tools/agents/land.py:181`
- [R] ticket landing to the effort base branch returns `MAINLINE_REFUSED` unless `--owner-mainline-override` is present `tools/agents/land.py:346`
- [R] ticket landing validates acceptance records from the resolved effort worktree `tools/agents/land.py:383`
- [R] successful owner override use is recorded in the resolved effort worktree `tools/agents/land.py:401`
- [R] `land.py` exposes `--owner-mainline-override` on the command line `tools/agents/land.py:413`
- [R] `lead_launch.py launch` resolves the integration branch from the shared effort-record resolver when the command omits `--integration` `tools/agents/lead_launch.py:66`
- [R] the lead-launch parser treats `--integration` as an optional override instead of a required argument `tools/agents/lead_launch.py:393`
- [R] the lead-driven landing verifier creates or opens a target branch checkout when the source checkout is not on the landing target `bench/harness/lead_driven.py:617`
- [R] the integration-branch readiness scenario declares stories 29 and 30 `bench/readiness/integration-branch/scenario.toml:2`
- [R] the integration-branch readiness scenario includes the main-checkout-effort-worktree case `bench/readiness/integration-branch/scenario.toml:33`
- [R] the lead-driven test asserts the integration-branch scenario passes each expected case `bench/harness/test_lead_driven.py:495`
- [R] the lead-driven test asserts main-checkout status reports the integration worktree and landed Candidate `bench/harness/test_lead_driven.py:519`

## Outside declared paths

- `tools/agents/lead_launch.py`: needed because the ticket explicitly says launch takes the integration branch from the effort record instead of repeating a flag on every command.
- `tools/agents/test_lead_launch.py`: test coverage for the launch command's new effort-record default and review-1 main-checkout/integration-worktree layout.
- `bench/harness/lead_driven.py`: needed so the lead-driven scenario can exercise real `land.py`, `effort.py status`, and `lead_launch.py` behavior, including review-1's cross-worktree layout.
- `bench/harness/test_lead_driven.py`: test coverage for the scenario's observed behavior.
- `docs/efforts/lead-worker-readiness/evidence/07-report.md`: required hand-off report for this ticket.

## For later tickets

- to 16: the effort record, the tickets and the acceptance records are found by searching worktrees for the effort directory; keep that single lookup if bookkeeping moves to its own branch.

## Not checked

- Did not run `tools/agents/check.sh` or cargo, per ticket instruction.
- Did not launch real model workers; all scenario coverage used scripted participants and fake workers.

# Ticket 01 report

## Built
- Split `tools/collab/collab.py` into a thin executable wrapper plus narrow modules:
  `cli.py`, `core.py`, `state_store.py`, `workspace.py`, `ticket_source.py`,
  `check_runner.py`, `landing.py`, `policy.py` and `common.py`.
- Kept the public CLI path `tools/collab/collab.py` and existing command behavior.
- Added table-driven CLI dispatch in `cli.py`.
- Added structural public-suite guards for module presence, table dispatch, SQLite
  ownership and line length.

## Acceptance
- Existing collab behavior suite baseline before refactor:
  `python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v`
  ran 48 tests in 73.452s, OK.
- Post-refactor collab suite:
  `python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v`
  ran 50 tests in 68.914s, OK. The total includes the 48 existing tests plus 2
  new structure guards.
- Compile check:
  `python3 tools/temp_root.py -- python3 -m py_compile tools/collab/*.py tools/collab/tests/test_collab.py`
  passed.
- SQLite ownership:
  `rg -n "import sqlite3|sqlite3\\." tools/collab` shows SQLite imports and calls
  only in `tools/collab/state_store.py`; the test file only contains guard strings.
- Line length:
  a Python scan over `tools/collab` `.py` and `.md` files printed `no lines over 120`.
- Repository gate:
  `tools/agents/check.sh` did not pass in this worker run. It timed out at the
  run's 120s per-command cap while executing `tools/agents/checks.d/15-worker.sh`.
  Split `tools/agents/test_worker.py` runs covered all 89 worker tests in five
  chunks, all OK. A later split of `bench/harness/test_lead_driven.py` found one
  repeatable failure in
  `LeadDrivenTest.test_interactive_launch_and_monitors_scenario_runs_through_lead_driven_mode`.

## Claims
- Collab behavior is unchanged at the public CLI seam covered by the existing
  48 tests.
- The CLI entry is table-driven and no longer uses packed one-line dispatch.
- Only `state_store.py` owns SQLite access.
- `tools/collab` source and docs are within the 120-column limit.

## Outside declared paths
- None changed outside `tools/collab/` and this report path.

## For later tickets
none

## Not checked
- The default repository gate was not completed successfully because of the
  command cap and the repeated lead-driven harness failure above.
- Remaining default-gate fragments after the confirmed `45-lead-driven-harness`
  failure, including the later agent checks and `cargo test --offline`, were not
  run in this worker attempt.

### Worker-reported harness observations (not addressed to a ticket)
- Investigate the repeated lead-driven harness failure:
  `LeadDrivenTest.test_interactive_launch_and_monitors_scenario_runs_through_lead_driven_mode`
  reports ticket `01` stuck at `tracker_status: claimed` and `worker_state: ready`
  instead of reaching finishing.
- `LeadDrivenTest.test_access_profile_scenario_runs_through_lead_driven_mode`
  timed out even when run alone under the 120s per-command cap.

## Built

Lead launches record optional tmux watch windows without making tmux a hard dependency for non-interactive workers.
Worker supervision records wall-clock and quiet-time notices while the process is still running, plus disk usage in RUNNING and final verdict/accounting records.
Lead accounting reports stopped, running and retained workers, counts retained workers against the parallel cap, and exposes monitor/wait commands that treat a marker commit from an active session as `finishing`.
After reintegration with tickets 06 and 16, `monitor-tickets`, `wait`, `account` and `retain-worker` resolve the effort through the record-branch-aware lookup and work from the main checkout when records live in a record worktree.
The lead-driven readiness harness has a deterministic `watching-limits-accounting` scenario for stories 38-41.

## Acceptance

- "A launch into a named tmux session creates the session if needed and one window per worker; with tmux missing or failing the launch succeeds and records that there is no window."
  - Proved after reintegration by `python3 tools/agents/test_lead_launch.py` -> `Ran 23 tests ... OK`; relevant tests include `test_launch_creates_tmux_session_and_one_window_per_worker`, `test_launch_records_missing_tmux_window_without_failing`, and `test_tmux_window_record_handles_missing_tmux_binary`.
  - Also proved by `python3 bench/harness/lead_driven.py watching-limits-accounting --out "$tmp"` -> `passes watching-limits-accounting tmux-missing-launch expected=checked observed=checked`.

- "The durable action record contains each command the scripted participant ran, after the window is closed."
  - Proved after reintegration by `python3 bench/harness/test_lead_driven.py` -> `Ran 29 tests ... OK`; the scenario records the scripted participant command from `events.jsonl` after stopping the launch.
  - Also proved by `python3 bench/harness/lead_driven.py watching-limits-accounting --out "$tmp"` -> `passes watching-limits-accounting tmux-missing-launch expected=checked observed=checked`.

- "Passing the wall-clock limit and passing the quiet time each append a notice to the run's status record while the worker is still running."
  - Proved after reintegration by `python3 tools/agents/test_worker.py` -> `Ran 66 tests ... OK`; relevant coverage includes `test_running_limit_notices_are_written_to_status_log`.
  - Also proved by `python3 bench/harness/lead_driven.py watching-limits-accounting --out "$tmp"` -> `passes watching-limits-accounting limit-notices-and-accounting expected=checked observed=checked`.

- "After a ticket lands, an accounting command lists each process of its worker as stopped; a worker deliberately retained for rework is listed as retained and reduces the available parallel slots by one."
  - Proved after reintegration by `python3 tools/agents/test_lead_launch.py` -> `Ran 23 tests ... OK`; relevant coverage includes `test_accounting_lists_stopped_and_retained_workers_and_slots`.
  - The new reintegration regression `test_monitor_wait_account_and_retain_use_record_branch_from_main_checkout` proves `account`, `retain-worker`, `monitor-tickets` and `wait` work from the main checkout after records move to a record branch.
  - The deterministic scenario also proves the stopped-accounting path: `passes watching-limits-accounting limit-notices-and-accounting expected=checked observed=checked`.

- "The end-of-run record includes the disk used by the worker's worktree and its caches."
  - Proved after reintegration by `python3 tools/agents/test_worker.py` -> `Ran 66 tests ... OK`.
  - Proved in accounting by `python3 tools/agents/test_lead_launch.py` -> `Ran 23 tests ... OK`.
  - Proved in the readiness scenario by `python3 bench/harness/lead_driven.py watching-limits-accounting --out "$tmp"` -> `passes watching-limits-accounting limit-notices-and-accounting expected=checked observed=checked`.

- "The scenarios that are deterministic run in the lead-driven mode and match their expectation."
  - Proved after reintegration by the required full scenario loop over every directory in `bench/readiness`: all 16 scenario commands exited 0 and printed `passes ... expected=... observed=...`, including `watching-limits-accounting`.
  - Also proved by all `bench/harness/test_*.py` suites: `test_fleet.py` ran 9 tests OK, `test_lead_driven.py` ran 29 tests OK, `test_run.py` ran 17 tests OK, `test_tasks_tracked.py` ran 2 tests OK, and `test_validate.py` ran 9 tests OK.
  - All `tools/agents/test_*.py` suites passed after reintegration, and `python3 tools/agents/tracker.py lint` exited 0.

## Claims

- [R] Non-interactive lead launches record an optional tmux watch window in `run.json` under `lead_launch.tmux_window` `tools/agents/lead_launch.py:558`
- [R] The tmux helper creates a session when `tmux has-session` fails and returns an unavailable record instead of raising when tmux cannot be executed `tools/agents/lead_launch.py:468`
- [R] The tmux success test checks one created session and two worker windows for a two-ticket launch `tools/agents/test_lead_launch.py:450`
- [R] The missing/failing tmux test checks launch success plus `available=false` and the captured tmux error `tools/agents/test_lead_launch.py:443`
- [R] The scripted readiness participant emits a durable command execution event for `printf watching-limits-accounting` `bench/readiness/watching-limits-accounting/scripted_participant.py:13`
- [R] The watching-limits scenario reads `events.jsonl` commands after stopping the launched worker and stores them as `action_record_commands` `bench/harness/lead_driven.py:1240`
- [R] The lead-driven test asserts the durable command list equals `["printf watching-limits-accounting"]` `bench/harness/test_lead_driven.py:460`
- [R] External protected-tree scans record entry count and scan seconds in the run record `tools/agents/worker.py:627`
- [R] Running supervision appends `wall-clock-limit` and `quiet-time` notices only while the worker is alive `tools/agents/worker.py:1497`
- [R] RUNNING and final verdict outputs include disk usage for the worktree, run directory and caches `tools/agents/worker.py:1562`
- [R] Accounting rows include worker process state, slot-held status and disk usage `tools/agents/lead_launch.py:998`
- [R] Retained workers reduce available parallel slots because account subtracts rows with `slot_held` from the cap `tools/agents/lead_launch.py:1018`
- [R] The wait command treats `running`, `finishing` and `rework` as running states and does not trigger on them `tools/agents/lead_launch.py:1044`
- [R] `worker.py status --help` discloses that status may update `run.json` when it observes Candidate/session/usage/acknowledgement state `tools/agents/worker.py:1815`
- [R] `lead_launch.py` has a record-branch-aware effort root helper backed by `land.resolve_effort_record` `tools/agents/lead_launch.py:82`
- [R] `monitor-tickets` reads tickets from the resolved effort record worktree instead of assuming the code checkout still has `docs/efforts` `tools/agents/lead_launch.py:986`
- [R] `account` and `retain-worker` validate the relevant effort through the resolved effort record worktree before reporting or retaining `tools/agents/lead_launch.py:1015`
- [R] The new reintegration regression moves records to `effort/demo-records`, removes effort records from main and code checkouts, and then exercises `retain-worker`, `monitor-tickets`, `wait` and `account` from the main checkout `tools/agents/test_lead_launch.py:535`
- [R] The deterministic readiness scenario is registered for stories 38, 39, 40 and 41 with `tmux-missing-launch` and `limit-notices-and-accounting` cases `bench/readiness/watching-limits-accounting/scenario.toml:1`

## Outside declared paths

- `bench/harness/lead_driven.py`: required to add lead-driven deterministic checks for ticket stories 38-41.
- `bench/harness/test_lead_driven.py`: required to test the new lead-driven scenario and report listing.
- `bench/readiness/watching-limits-accounting/`: required deterministic scenario fixture for the ticket's lead-driven acceptance item.
- `docs/efforts/lead-worker-readiness/evidence/10-report.md`: required hand-off report for this ticket.

## For later tickets

none

## Not checked

- Did not run `tools/agents/check.sh` or cargo, per ticket instruction.
- Did not start ticket 18's interactive launch mode or worker monitor work; the lead explicitly moved that omitted scope to ticket 18.
- The final full gate passed, but the suites still print existing `ResourceWarning` noise from long-lived fake subprocesses and `bench/harness/tasklib.py` unclosed file handles.

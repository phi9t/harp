## Built

`worker.py launch` accepts `--runtime traecli|codex`, keeps `traecli` as the default, records the selected runtime and per-runtime observability in `run.json`, and builds runtime-specific launch argv.

Codex `exec --json` events are covered with the lead-observed top-level stream shape from codex-cli 0.160.0: `thread.started`, `turn.started`, `item.started`, `item.completed`, agent message, and `turn.completed` with `usage`. The runtime adapter counts command events once by preferring `item.completed`, keeps started commands pending, and flushes pending started-only commands only when a stream ends without a completion. It also reports `usage` from `turn.completed`, and the supervisor records it as `model_usage` in `run.json`.

The archived-session rollout-shaped Codex fixture is kept only to show the older stored-session record shape the first implementation used. The fake Codex binary now emits the real `codex exec --json` stream shape for launch and resume tests.

`worker.py resume` sends a follow-up message to the observed session id for both runtime adapters and records a resume receipt. Missing session ids are reported as clean worker errors. Foreground resume timeouts are caught, partial stdout/stderr are kept in the receipt files, the receipt is marked `timed_out`, and the command exits `DEGRADED`. The default resume timeout is now the run's wall-clock cap.

After rebasing onto `effort/lead-worker-readiness`, the newly landed `tools/agents/lead_launch.py` path also accepts `--runtime`, passes it through to `worker.py launch`, and records it in `run["lead_launch"]["runtime"]`. The lead-driven `lead-launch` scenario path passes its runtime choice through to `lead_launch.py` and records the nested worker and lead-launch runtimes in the case observation.

The external-state detail change from review 1 is kept because reverting to full paths made the lead-driven external-state test fail its existing assertion that findings not name the worker; run-owned protected paths under this worktree contain `worker` in the absolute path. The detail now includes a root-qualified label such as `external[1]:root-name/path`, so two protected roots with the same child path are not ambiguous.

## Acceptance

> A launch option selects the runtime; the run record names it; the default stays as today.

Proved by `python3 tools/agents/test_worker.py`: `test_runtime_option_records_runtime_and_default_stays_traecli` passed; suite result `Ran 54 tests in 84.709s OK`. Also proved through the rebased lead-launch path by `python3 tools/agents/test_lead_launch.py`: `test_launch_passes_and_records_runtime_option` passed; suite result `Ran 7 tests in 11.122s OK`. The harness also proves both runtime entrypoints: `python3 -m unittest discover -s bench/harness -p 'test_*.py'` passed `test_codex_runtime_option_reaches_worker_and_results_row` and `test_lead_launch_runtime_option_reaches_worker_and_launch_record`; suite result `Ran 61 tests in 102.626s OK`.

> With a fake Codex binary emitting a recorded sample of the real event format, command events are extracted and the forbidden-command and quiet-time findings work as for traecli.

Proved by `python3 tools/agents/test_worker.py`: `test_codex_recorded_command_events_drive_forbidden_findings`, `test_codex_started_command_without_completion_is_checked_once_on_exit`, `test_codex_runtime_usage_is_saved_from_turn_completed`, and `test_codex_quiet_time_is_reported_from_its_event_stream` passed; suite result `Ran 54 tests in 84.709s OK`.

> The hand-off rule (exit 0, clean tree, at least one commit) is applied identically for both runtimes; one test runs the same scripted behavior under both and compares verdicts.

Proved by `python3 tools/agents/test_worker.py`: `test_handoff_rule_matches_for_traecli_and_codex` passed; suite result `Ran 54 tests in 84.709s OK`.

> Resume sends a follow-up message to the same session for both runtimes with the fake binaries; where a runtime cannot resume, the tool reports that.

Proved by `python3 tools/agents/test_worker.py`: `test_resume_sends_followup_to_same_session_for_each_runtime`, `test_resume_reports_missing_runtime_session_id`, `test_resume_default_timeout_uses_run_wall_clock_cap`, and `test_resume_timeout_records_partial_events_and_degrades` passed; suite result `Ran 54 tests in 84.709s OK`.

> The help text lists, per runtime, what is and is not observed.

Proved by `python3 tools/agents/test_worker.py`: `test_launch_help_lists_runtime_observation_limits` passed; suite result `Ran 54 tests in 84.709s OK`.

> The sample of the Codex event format is taken from a real local Codex session record with secrets and private reasoning removed, and its origin is stated in the test.

Proved by `python3 tools/agents/test_worker.py`: `test_codex_recorded_command_events_drive_forbidden_findings` passed and asserts the lead-captured real `codex exec --json` origin string from codex-cli 0.160.0 on 2026-10-05. The test also states the archived rollout sample is only archived-session coverage. Suite result `Ran 54 tests in 84.709s OK`.

Review-1 regression evidence:

- Initial red run of the six new review tests failed with missing `event_stream_commands`, missing `model_usage`, uncaught `TimeoutExpired`, default timeout `120 != 7`, and ambiguous external labels `['state.txt', 'state.txt']`.
- After the fix, `python3 tools/agents/test_worker.py WorkerTest.test_codex_recorded_command_events_drive_forbidden_findings WorkerTest.test_codex_started_command_without_completion_is_checked_once_on_exit WorkerTest.test_codex_runtime_usage_is_saved_from_turn_completed WorkerTest.test_resume_default_timeout_uses_run_wall_clock_cap WorkerTest.test_resume_timeout_records_partial_events_and_degrades WorkerTest.test_external_protected_change_labels_include_disambiguating_root`: `Ran 6 tests in 7.317s OK`.

Reintegration evidence:

- Rebased this branch onto `effort/lead-worker-readiness` at `8396875`, resolving the `bench/harness/lead_driven.py` import conflict by keeping both the new lead-launch/tracker constants and the ticket-12 worker adapter import.
- `python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_lead_launch_runtime_option_reaches_worker_and_launch_record` failed before the harness runtime pass-through with `KeyError: 'worker_runtime'`, then passed after the fix: `Ran 1 test in 1.559s OK`.
- `python3 tools/agents/test_lead_launch.py LeadLaunchTest.test_launch_passes_and_records_runtime_option` failed before the `lead_launch.py` runtime record with `KeyError: 'runtime'`, then passed after the fix: `Ran 1 test in 2.232s OK`.

Gate evidence after reintegration:

- `python3 tools/agents/test_worker.py`: `Ran 54 tests in 84.709s OK`.
- `python3 tools/agents/test_land.py`: `Ran 20 tests in 22.931s OK`.
- `python3 tools/agents/test_acceptance.py`: `Ran 7 tests in 1.800s OK`.
- `python3 tools/agents/test_tracker.py`: `Ran 12 tests in 2.905s OK`.
- `python3 tools/agents/test_lead_launch.py`: `Ran 7 tests in 11.122s OK`.
- `python3 -m unittest discover -s bench/harness -p 'test_*.py'`: `Ran 61 tests in 102.626s OK`.
- Every scenario under `bench/readiness/` passed using `TMPDIR="$PWD/.scratch"` and a fresh `mktemp -d -p "$TMPDIR"` output root: `acceptance-failing-check`, `acceptance-missing-record`, `acceptance-other-candidate`, `acceptance-ticket-tamper`, `acceptance-waived-check`, `access-profile`, `external-state-report`, `handoff`, `land-exact-candidate`, `land-stale-candidate`, and `launch-by-edges-and-triage`.
- `python3 tools/agents/tracker.py lint`: passed with exit code 0.
- `python3 -m py_compile tools/agents/worker.py tools/agents/lead_launch.py tools/agents/test_worker.py tools/agents/test_lead_launch.py bench/harness/lead_driven.py bench/harness/test_lead_driven.py`: passed.
- `git diff --check`: passed.

## Claims

- [R] the worker runtime set is exactly `traecli` and `codex` `tools/agents/worker.py:51`
- [R] the launch help states TraeCLI observed and not-observed surfaces `tools/agents/worker.py:72`
- [R] the launch help states Codex observed and not-observed surfaces `tools/agents/worker.py:74`
- [R] the Codex runtime binary comes from `CORENIUS_CODEX` when set `tools/agents/worker.py:89`
- [R] the default runtime binary path still comes from `CORENIUS_TRAECLI` when set `tools/agents/worker.py:90`
- [R] `event_usage` reports runtime usage only from completed turns with a usage object `tools/agents/worker.py:132`
- [R] the stream adapter emits completed command items once and records pending started items `tools/agents/worker.py:180`
- [R] the stream adapter flushes pending started-only commands when the stream ends `tools/agents/worker.py:192`
- [R] the helper `event_stream_commands` applies the runtime stream adapter to a list of events `tools/agents/worker.py:201`
- [R] external-state labels include the protected-root ordinal and root name `tools/agents/worker.py:600`
- [R] supervision saves `model_usage` from runtime events in the run record `tools/agents/worker.py:921`
- [R] resume catches `subprocess.TimeoutExpired` and marks the receipt timed out `tools/agents/worker.py:1135`
- [R] the resume parser leaves timeout unset so the run cap becomes the default `tools/agents/worker.py:1209`
- [R] `lead_launch.py` passes `--runtime` through to `worker.py launch` `tools/agents/lead_launch.py:248`
- [R] `lead_launch.py` records the selected runtime in the lead-launch run record `tools/agents/lead_launch.py:288`
- [R] `lead_launch.py` exposes `--runtime` with the worker runtime choices and default `traecli` `tools/agents/lead_launch.py:355`
- [R] lead-driven command checks reuse the worker stream adapter `bench/harness/lead_driven.py:222`
- [R] lead-driven maps the scenario runtime to a worker runtime before invoking lead-launch `bench/harness/lead_driven.py:899`
- [R] lead-driven passes the mapped runtime to `lead_launch.py launch` `bench/harness/lead_driven.py:923`
- [R] lead-driven records the nested worker runtime and lead-launch runtime for launched lead-launch cases `bench/harness/lead_driven.py:998`
- [R] the lead-captured Codex exec JSON sample origin is stated in the worker test `tools/agents/test_worker.py:145`
- [R] the archived-session sample origin is stated separately from the real exec JSON sample `tools/agents/test_worker.py:139`
- [R] the worker tests compare the handoff verdict fields for TraeCLI and Codex `tools/agents/test_worker.py:465`
- [R] the worker tests verify resume sends the same session id and message for both runtimes `tools/agents/test_worker.py:478`
- [R] the worker tests verify resume timeout receipts keep partial events and degrade `tools/agents/test_worker.py:514`
- [R] the worker tests verify external-state labels disambiguate two protected roots with the same child path `tools/agents/test_worker.py:661`
- [R] the lead-launch tests verify default TraeCLI and explicit Codex runtime records `tools/agents/test_lead_launch.py:279`
- [R] the harness test verifies `--runtime codex` reaches the lead-launch worker run and result row `bench/harness/test_lead_driven.py:121`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/12-report.md`: required hand-off evidence file for ticket 12 and review/reintegration evidence updates.
- `tools/agents/lead_launch.py`: reintegration instruction required passing the runtime option through newly landed lead-launch code and recording it.
- `tools/agents/test_lead_launch.py`: needed to prove lead-launch default and explicit runtime behavior after reintegration.

## For later tickets

- to 04: `worker.py resume` now depends on a session id observed in `events.jsonl`; fake-runtime tests cover both adapters, but real runtime resume should still be validated before spending model budget.
- to 04: resume runs in the foreground; a rework round needs it started detached and supervised like launch (cap, quiet time, hand-off detection).

## Not checked

- Real `traecli` and `codex` model worker launches were not run, per ticket instruction to avoid model budget.
- `tools/agents/check.sh` and cargo were not run, per ticket instruction.
- The first all-scenarios command using default `/tmp` failed in `launch-by-edges-and-triage` because `/tmp` had no free inodes; rerunning with `TMPDIR` under `.scratch` passed every scenario and the temporary tree was removed.
- The broad `bench/harness` discovery run emitted existing `ResourceWarning: unclosed file` messages from `bench/harness/tasklib.py:90`; the command exited 0 with `Ran 61 tests in 102.626s OK`.

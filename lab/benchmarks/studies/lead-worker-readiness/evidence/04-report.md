## Built

Review findings can now be recorded, delivered, acknowledged, and used to drive a rework round. Terminal delivery records transport send separately from worker-side acknowledgement; unacknowledged sends are non-counting until a later event, commit, or pane observation confirms receipt.

Rework commands now resolve effort records through the repository worktree registry. `acceptance.py` and `lead_launch.py rework` can be run from a main checkout while the effort directory and acceptance evidence live in the integration worktree.

The lead-driven `rework-round` scenario covers one accepted rework round from the main-checkout/integration-worktree layout, refusal to land a successor on the first Candidate's acceptance record, and the review cap with stale-base redo excluded.

## Acceptance

- "Delivering findings records the findings file's digest, the time and the worker run it was delivered to; the worker side acknowledges and the receipt is recorded." Proved by `python3 tools/agents/test_worker.py` (`Ran 60 tests ... OK`), especially `test_deliver_findings_records_digest_candidate_round_and_terminal_receipt`, `test_terminal_delivery_without_worker_observation_is_unconfirmed`, and `test_status_can_acknowledge_pending_terminal_delivery_from_pane_activity`.
- "A resumed scripted participant receives the findings and produces a second commit; the run record lists both Candidates in order with the review that rejected the first." Proved by `python3 bench/harness/test_lead_driven.py` (`Ran 26 tests ... OK`), especially `test_rework_round_scenario_runs_through_lead_driven_mode`, and by `python3 bench/harness/lead_driven.py rework-round --out "$tmp"` (`passes rework-round one-round-accepted expected=checked observed=checked`).
- "The second Candidate cannot land on the first one's acceptance record." Proved by `python3 bench/harness/test_lead_driven.py` (`Ran 26 tests ... OK`), especially `test_rework_round_scenario_runs_through_lead_driven_mode`, and by `python3 bench/harness/lead_driven.py rework-round --out "$tmp"` (`passes rework-round second-review-required expected=checked observed=checked`).
- "A fourth review-driven round is refused and the ticket's triage becomes `ready-for-human` with a comment; a stale-base redo leaves the round count unchanged." Proved by `python3 tools/agents/test_acceptance.py` (`Ran 11 tests ... OK`), especially `test_fourth_review_round_moves_ticket_to_ready_for_human_but_stale_redo_does_not_count`, and by `python3 bench/harness/test_lead_driven.py` (`test_rework_round_scenario_runs_through_lead_driven_mode` asserts rounds `[1, 1, 2, 3]`, count flags `[True, False, True, True]`, and triage `ready-for-human`).
- "When the worker's session can no longer be resumed, the tool says so and starts a fresh worker with the findings and the earlier Candidate in its brief; the record says which happened." Proved by `python3 tools/agents/test_lead_launch.py` (`Ran 12 tests ... OK`), especially `test_rework_starts_fresh_worker_when_original_session_cannot_resume` and `test_rework_from_main_checkout_finds_rejected_review_in_effort_worktree`.
- "All scenarios run in the lead-driven mode and match their expectation." Proved by running every directory under `bench/readiness/` with `python3 bench/harness/lead_driven.py <scenario> --out "$tmp"`; all 13 commands exited 0 and printed only `passes ... expected=... observed=...` rows.

Additional gate commands:
- `python3 tools/agents/test_worker.py`: `Ran 60 tests ... OK`.
- `python3 tools/agents/test_acceptance.py`: `Ran 11 tests ... OK`.
- `python3 tools/agents/test_lead_launch.py`: `Ran 12 tests ... OK`.
- `python3 tools/agents/test_land.py`: `Ran 24 tests ... OK`.
- `python3 tools/agents/test_tracker.py`: `Ran 17 tests ... OK`.
- `python3 tools/agents/test_effort.py`: `Ran 13 tests ... OK`.
- `python3 bench/harness/test_fleet.py && python3 bench/harness/test_lead_driven.py && python3 bench/harness/test_run.py && python3 bench/harness/test_tasks_tracked.py && python3 bench/harness/test_validate.py`: all exited 0.
- `python3 -m py_compile tools/agents/worker.py tools/agents/acceptance.py tools/agents/lead_launch.py bench/harness/lead_driven.py bench/readiness/rework-round/scripted_participant.py`: passed.
- `python3 tools/agents/tracker.py lint`: passed with no output.
- `git diff --check`: passed.

## Claims

- [R] pending review delivery selection ignores unacknowledged deliveries `tools/agents/worker.py:989`
- [R] terminal delivery acknowledgement can be observed from a new worker event `tools/agents/worker.py:1006`
- [R] terminal delivery acknowledgement can be observed from a new worker commit `tools/agents/worker.py:1031`
- [R] terminal delivery acknowledgement can be observed from pane activity containing the findings path `tools/agents/worker.py:1051`
- [R] terminal delivery receipts store the terminal target for later acknowledgement checks `tools/agents/worker.py:1122`
- [R] terminal delivery receipts store `sent: true` separately from acknowledgement state `tools/agents/worker.py:1124`
- [R] terminal delivery receipts start as `acknowledged: false` until an observation is found `tools/agents/worker.py:1131`
- [R] later acknowledgement marks a review delivery as counting toward the cap `tools/agents/worker.py:1173`
- [R] `supervise` checks pending rework deliveries for later acknowledgement during its loop `tools/agents/worker.py:1306`
- [R] `status` checks pending rework deliveries for later acknowledgement `tools/agents/worker.py:1373`
- [R] unacknowledged terminal findings delivery records no counted round `tools/agents/worker.py:1519`
- [R] unacknowledged terminal findings delivery exits degraded instead of clean `tools/agents/worker.py:1535`
- [R] acceptance commands resolve the effort worktree through `land.resolve_effort_record` `tools/agents/acceptance.py:38`
- [R] `acceptance.py record` writes records in the resolved effort worktree `tools/agents/acceptance.py:309`
- [R] `acceptance.py review` writes records in the resolved effort worktree `tools/agents/acceptance.py:319`
- [R] `acceptance.py request-rework` writes records in the resolved effort worktree `tools/agents/acceptance.py:349`
- [R] `acceptance.py summary` reads records from the resolved effort worktree `tools/agents/acceptance.py:403`
- [R] a fourth counted review request writes ticket triage `ready-for-human` `tools/agents/acceptance.py:362`
- [R] stale-base rework requests do not count toward the review cap `tools/agents/acceptance.py:375`
- [R] `lead_launch.py rework` loads rejected review records from the resolved effort worktree `tools/agents/lead_launch.py:414`
- [R] rework delivery records include the worker run directory `tools/agents/lead_launch.py:443`
- [R] rework delivery records include the findings digest `tools/agents/lead_launch.py:446`
- [R] rework delivery records include the delivery time `tools/agents/lead_launch.py:447`
- [R] same-session rework calls `worker.py resume` with `--detach` `tools/agents/lead_launch.py:550`
- [R] same-session rework reports mode `resumed` in its JSON output `tools/agents/lead_launch.py:561`
- [R] no-session rework records the fallback transport as `fresh-worker` `tools/agents/lead_launch.py:565`
- [R] the fresh-worker fallback records the fresh worktree in the receipt `tools/agents/lead_launch.py:568`
- [R] the rework harness creates a checked-out integration worktree for rework layouts `bench/harness/lead_driven.py:1571`
- [R] the main-checkout rework case removes the effort record from the command repo `bench/harness/lead_driven.py:1574`
- [R] the rework harness reports the command repo separately from the resolved effort worktree `bench/harness/lead_driven.py:1826`
- [R] the rework harness records two observed Candidates for normal rework cases `bench/harness/lead_driven.py:1830`
- [R] the rework cap case expects recorded rounds `[1, 1, 2, 3]` `bench/harness/lead_driven.py:1869`
- [R] the rework cap case expects stale-base not to count against the review cap `bench/harness/lead_driven.py:1870`
- [R] the `one-round-accepted` scenario uses the main-checkout/integration-worktree layout `bench/readiness/rework-round/scenario.toml:17`
- [R] the scripted participant refuses resume messages that omit the findings digest `bench/readiness/rework-round/scripted_participant.py:54`
- [R] the scripted participant commits a fix before the marker on resume `bench/readiness/rework-round/scripted_participant.py:60`
- [R] the harness test asserts the one-round rework case runs without the effort record in the command repo `bench/harness/test_lead_driven.py:535`
- [R] the harness test asserts one-round rework records are written under the effort worktree `bench/harness/test_lead_driven.py:537`
- [R] the harness test asserts the fourth review round triages the ticket to `ready-for-human` `bench/harness/test_lead_driven.py:548`
- [R] the acceptance test asserts all acceptance commands from main resolve to the integration worktree `tools/agents/test_acceptance.py:159`
- [R] the lead-launch test asserts rework from main finds a rejected review in the effort worktree `tools/agents/test_lead_launch.py:458`
- [R] the worker test asserts unobserved terminal delivery is sent but unacknowledged and non-counting `tools/agents/test_worker.py:607`
- [R] the worker test asserts delayed pane activity can acknowledge a pending terminal delivery `tools/agents/test_worker.py:672`

## Outside declared paths

- `bench/harness/lead_driven.py`: needed to make the readiness scenario execute the review-request and landing flow against the main-checkout/integration-worktree layout required by review-1.
- `bench/harness/test_lead_driven.py`: needed to lock the rework scenario's cross-worktree behavior and the fourth-review triage assertion in the harness suite.
- `docs/efforts/lead-worker-readiness/evidence/04-report.md`: required hand-off report for this ticket.

## For later tickets

- to 14: Real runtime same-session resume remains unspent by instruction; the implementation is covered by fake runtime events and should be live-validated before model budget is used.
- to 14: The first pending fix state is now `fixes-committed-waiting-for-re-integration`; later live orchestration can use that distinct state when a worker is told to hold the marker until the base moves.

## Not checked

- Real `traecli` or Codex model workers were not launched, by instruction.
- `tools/agents/check.sh` and cargo were not run, by instruction.
- Existing `ResourceWarning` noise from Python subprocess/capsule tests was observed during passing suites and was not fixed in this ticket.

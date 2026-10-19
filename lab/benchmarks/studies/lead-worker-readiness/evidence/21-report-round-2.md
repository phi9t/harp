# Ticket 21 report

## Built

The lead-driven readiness harness now fails scenarios when the tool they judge is replaced by a no-op stub, records worker execution failures as failed rows, records non-JSON worker event output as an unexpected-output failure instead of crashing, and demotes scripted/fake runtime proof from `live` to `deterministic`.

After reintegration review, result rows now also record `runtime: scripted` whenever the harness decides proof is not live, while preserving the CLI runtime request in `requested_runtime`.

The access-profile scenario performs the observed actions it claims: loopback fetch, external add-path write, and a command longer than 120 seconds. Worker run records and help text describe access-profile support by runtime.

Lead launch and tracker dependency checks require an accepted Candidate that has landed by landing record or integration history. The repository gate covers the worker, effort, lead-launch, lander, acceptance, tracker lint, and lead-driven harness suites.

## Acceptance

- "A mutation check exists and runs in the harness tests: for every scenario, replacing the tool it judges (the lander, the acceptance command, the launch command, the worker launcher, the tracker) with a stub that does nothing makes the scenario report `fails`. Findings 15 and 16 are the first two cases it catches."
  Proved by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py` in a clean detached checkout of `3df6674`: 36 tests OK. The mutation cases cover worker, acceptance, land, tracker, lead-launch, and bookkeeping-worker stubs.

- "Finding 16: a scenario whose worker process did not start, exited non-zero unexpectedly, or wrote to `stderr.log` an error the scenario did not expect reports `fails`; scripted participants are executable; no scenario commits or imports on the worker's behalf."
  Proved by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py` in a clean detached checkout of `3df6674`: 36 tests OK, including worker execution failure tests, the non-JSON worker event regression, and the scripted-participant executable test. The bookkeeping participant now commits its own work and report.

- "Finding 17: a result row's `proof` is `live` only when the run used a real runtime binary (decided by the harness, not by a flag); a run that errors writes a `fails` row; an empty record list is `fails`; a row lists only the stories whose participants actually ran, and the report shows which participants ran."
  Proved by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py` in a clean detached checkout of `3df6674`: 36 tests OK, including the review-round assertion that a requested `--runtime codex --proof live` scripted run records `runtime: scripted`, `requested_runtime: codex`, and `proof: deterministic`. Also proved in the same clean checkout by `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest $(find bench/harness -maxdepth 1 -name 'test_*.py' ! -name 'test_lead_driven.py' -printf 'bench.harness.%f\n' | sed 's/\.py$//' | sort | tr '\n' ' ')`: 37 tests OK.

- "Finding 13: the wall-clock notice is proven by a worker that really outlives a short cap, not by editing the run record after launch; if the notice cannot occur because the wrapper kills the process at the cap, change the product so that the notice is written when the cap is reached and say so."
  Proved in a clean detached checkout of `3df6674` by `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest $(find tools/agents -maxdepth 1 -name 'test_*.py' ! -name 'test_sqlite_baseline.py' -printf 'tools.agents.%f\n' | sed 's/\.py$//' | sort | tr '\n' ' ')`: 327 tests OK, and by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py`: 36 tests OK. Product behavior changed so the supervisor writes `wall-clock-limit` when timeout kills at the cap.

- "Finding 12: the access-profile scenario's participants perform the acts (a real loopback fetch, a real write under the declared add path, a real command longer than the old limit); the help and the launch record say for each runtime which parts of the profile are enforced, which are only declared, and which are only detected from command text."
  Proved in a clean detached checkout of `3df6674` by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py`: 36 tests OK, and by the all-tools suite above: 327 tests OK.

- "Finding 9: `monitor-tickets` computes commits ahead and whether the branch contains the integration tip from the integration commit recorded at launch and the worker's own repository; its worker state never says ready when the supervisor's verdict is not ready."
  Proved by the all-tools suite above: 327 tests OK. The regression tests use a private worker clone, a recorded launch base, and a persisted non-ready verdict.

- "Finding 19: a ticket counts as landed for launching its dependants only when a landing record or the integration branch's history shows its accepted Candidate; a ticket merely marked resolved does not unblock them. `tracker.py resolve` refuses when no accepted, landed Candidate exists unless forced, and a forced resolve is logged."
  Proved by the all-tools suite above: 327 tests OK, including the main-checkout plus integration/record-worktree blocker regression.

- "Finding 20: the repository gate's `checks.d` runs the worker, effort, lead-launch, lander and acceptance suites, lints every effort, and has an entry for the lead-driven harness tests."
  Proved by the check scripts under `tools/agents/checks.d/`, the all-tools suite above, and `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/tracker.py lint`: passed after reintegration.

- "Finding 21 and 23: `land.py --attempts` is removed or used; `lead_launch.py wait` exits non-zero on timeout; a failed check passed to `acceptance.py record` is stored and blocks acceptance unless waived, whether or not it was named required; `effort.py move-records` can be rerun after an interruption between its two commits."
  Proved by the all-tools suite above: 327 tests OK.

- "The readiness report, run after all of the above, prints the core-set line for the lead-driven instantiation; the report and your hand-off say plainly which core stories now pass, which fail, and why."
  Proved in a clean detached checkout of `3df6674` by running every readiness scenario command with fresh output directories and then in one shared output directory:
  `python3 bench/harness/lead_driven.py acceptance-failing-check --out "$(mktemp -d)"`,
  `acceptance-missing-record`,
  `acceptance-other-candidate`,
  `acceptance-ticket-tamper`,
  `acceptance-waived-check`,
  `access-profile`,
  `bookkeeping-off-code-branch`,
  `external-state-report`,
  `handoff`,
  `integration-branch`,
  `land-exact-candidate`,
  `land-stale-candidate`,
  `launch-by-edges-and-triage`,
  `parallel-workers-and-overlap`,
  `rework-round`,
  `shared-repository-safe`,
  and `watching-limits-accounting`: all returned 0. The shared-output report printed `Core set (deterministic, lead-driven): passes=11 passes with intervention=0 fails=0 not yet run=0 not applicable by design=0` and `Core stories (deterministic, lead-driven): passes=1,2,5,6,7,9,12,14,15,16,17,18,20,27,34,35 passes with intervention=- fails=- not yet run=- not applicable by design=-`. There are no failing deterministic lead-driven core stories; local-controller and workflow-controller rows are not run by this ticket's gate.

Review-round reproduction and verification before hand-off:

- `git worktree add --detach <dir under TMPDIR> b9cfd0b` followed by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_unexpected_worker_stderr_fails_participant`: passed locally, so I did not confirm a local file-mode or ignored-file delta for the lead's exact failure. `git status --ignored --short` in the working tree showed only ignored `__pycache__/` directories, and tracked modes for the touched files were `100644`.
- `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_non_json_worker_event_line_fails_participant`: failed before the fix with `json.decoder.JSONDecodeError`, then passed with `LeadDrivenTest.test_unexpected_worker_stderr_fails_participant`.
- Clean detached checkout of `3df6674` before the marker commit: `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py`: 36 tests OK.
- Clean detached checkout of `3df6674` before the marker commit: `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest $(find tools/agents -maxdepth 1 -name 'test_*.py' ! -name 'test_sqlite_baseline.py' -printf 'tools.agents.%f\n' | sed 's/\.py$//' | sort | tr '\n' ' ')`: 327 tests OK.
- Clean detached checkout of `3df6674` before the marker commit: `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest $(find bench/harness -maxdepth 1 -name 'test_*.py' ! -name 'test_lead_driven.py' -printf 'bench.harness.%f\n' | sed 's/\.py$//' | sort | tr '\n' ' ')`: 37 tests OK.
- Clean detached checkout of `3df6674` before the marker commit: all 17 `python3 bench/harness/lead_driven.py <scenario> --out "$(mktemp -d)"` readiness scenario commands passed.
- Clean detached checkout of `3df6674` before the marker commit: `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/tracker.py lint`: passed.
- Clean detached checkout of `3df6674` before the marker commit: `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile bench/harness/lead_driven.py bench/harness/results.py tools/agents/acceptance.py tools/agents/effort.py tools/agents/land.py tools/agents/lead_launch.py tools/agents/tracker.py tools/agents/worker.py && git diff --check`: passed.

## Claims

- [R] the lead-driven harness test stubs each judged tool and expects the scenario to fail `bench/harness/test_lead_driven.py:316`
- [R] the mutation cases include worker, acceptance, land, tracker, lead-launch, and bookkeeping worker substitutions `bench/harness/test_lead_driven.py:320`
- [R] the harness marks a run with no worker events as `worker-not-started` `bench/harness/lead_driven.py:595`
- [R] the harness records malformed worker event lines as unexpected output instead of raising a JSON decode error `bench/harness/lead_driven.py:593`
- [R] the harness appends malformed worker event lines as `worker-unexpected-output:<line>` failure details `bench/harness/lead_driven.py:597`
- [R] the harness marks unexpected stderr as `worker-stderr` `bench/harness/lead_driven.py:599`
- [R] the harness marks unexpected non-zero worker exits as `worker-exit-code` `bench/harness/lead_driven.py:608`
- [R] the harness has a test that all readiness scripted participants are executable `bench/harness/test_lead_driven.py:336`
- [R] the non-JSON worker event regression test expects a participant failure detail instead of a crash `bench/harness/test_lead_driven.py:390`
- [R] scripted proof is demoted from `live` unless all records say the runtime binary was real `bench/harness/lead_driven.py:1305`
- [R] non-live proof records the effective runtime as `scripted` `bench/harness/lead_driven.py:1318`
- [R] readiness rows store the effective runtime separately from the requested runtime `bench/harness/lead_driven.py:1368`
- [R] readiness rows store the requested runtime that was passed on the CLI `bench/harness/lead_driven.py:1377`
- [R] the runtime test proves a requested Codex scripted run stores runtime `scripted` and requested runtime `codex` `bench/harness/test_lead_driven.py:293`
- [R] result validation accepts only a non-empty string or null for `requested_runtime` `bench/harness/results.py:115`
- [R] readiness rows include `requested_runtime` in the stored schema row `bench/harness/results.py:181`
- [R] the readiness report includes a `requested runtime` column `bench/harness/results.py:304`
- [R] the readiness report prints a core story summary per proof and instantiation `bench/harness/results.py:330`
- [R] readiness rows store the participant names that actually ran `bench/harness/lead_driven.py:1375`
- [R] readiness rows store the stories observed in the run records `bench/harness/lead_driven.py:1376`
- [R] result validation requires participants to be a list of non-empty strings `bench/harness/results.py:142`
- [R] the access-profile participant performs a real loopback fetch `bench/readiness/access-profile/scripted_participant.py:49`
- [R] the access-profile participant emits `sleep 121` for the long command case `bench/readiness/access-profile/scripted_participant.py:82`
- [R] the access-profile participant runs a real `sleep 121` command `bench/readiness/access-profile/scripted_participant.py:86`
- [R] the access-profile harness test verifies the observed loopback fetch `bench/harness/test_lead_driven.py:522`
- [R] the access-profile harness test verifies the long command exceeded 120 seconds `bench/harness/test_lead_driven.py:532`
- [R] the access-profile harness test verifies the external add-path file exists `bench/harness/test_lead_driven.py:539`
- [R] worker run records include the access-profile support matrix `tools/agents/worker.py:1238`
- [R] worker help text documents TraeCLI access-profile support `tools/agents/worker.py:125`
- [R] worker help text documents Codex access-profile support `tools/agents/worker.py:130`
- [R] worker status notices include `wall-clock-limit` when the cap is reached `tools/agents/worker.py:1983`
- [R] the worker test proves a timeout-killed run writes a `wall-clock-limit` notice `tools/agents/test_worker.py:1507`
- [R] lead launch refuses blockers that lack an accepted and landed candidate `tools/agents/lead_launch.py:231`
- [R] monitor state returns `not-ready` when the persisted verdict is not ready `tools/agents/lead_launch.py:1066`
- [R] monitor-tickets uses the recorded integration base from the run before computing ahead and containment `tools/agents/lead_launch.py:1119`
- [R] lead-launch wait returns non-zero when no watched ticket leaves a running state before timeout `tools/agents/lead_launch.py:1213`
- [R] tracker treats a blocker as landed only when an accepted candidate is present in a landing record or integration history `tools/agents/tracker.py:691`
- [R] tracker blockers require both resolved status and an accepted landed candidate `tools/agents/tracker.py:699`
- [R] tracker resolve refuses missing accepted landed candidates unless forced `tools/agents/tracker.py:918`
- [R] tracker forced resolve logs the missing accepted landed candidate reason `tools/agents/tracker.py:926`
- [R] the tracker test verifies status-only resolved blockers do not unblock dependants `tools/agents/test_tracker.py:218`
- [R] the tracker test verifies an accepted landing record unblocks a dependant `tools/agents/test_tracker.py:236`
- [R] the tracker test verifies an accepted candidate in integration history unblocks from the main checkout `tools/agents/test_tracker.py:257`
- [R] acceptance records collect every failed check, not only required checks `tools/agents/acceptance.py:318`
- [R] acceptance records store all checks sorted by name `tools/agents/acceptance.py:354`
- [R] the acceptance test proves a failed non-required check blocks acceptance and is stored `tools/agents/test_acceptance.py:548`
- [R] effort move-records detects and reuses an existing record-branch worktree after interruption `tools/agents/effort.py:337`
- [R] the effort test proves move-records can rerun after the record-branch commit interruption `tools/agents/test_effort.py:128`
- [R] the lander test proves `--attempts` is not an exposed option `tools/agents/test_land.py:460`
- [R] the worker gate script runs the worker suite `tools/agents/checks.d/15-worker.sh:3`
- [R] the effort gate script runs the effort suite `tools/agents/checks.d/18-effort.sh:3`
- [R] the land gate script runs the land suite `tools/agents/checks.d/20-land.sh:3`
- [R] the acceptance gate script runs the acceptance suite `tools/agents/checks.d/25-acceptance.sh:3`
- [R] the tracker gate script runs the tracker suite `tools/agents/checks.d/30-tracker.sh:5`
- [R] the tracker gate script lints all discovered efforts `tools/agents/checks.d/30-tracker.sh:6`
- [R] the lead-launch gate script runs the lead-launch suite `tools/agents/checks.d/35-lead-launch.sh:3`
- [R] the lead-driven harness gate script runs the lead-driven harness suite `tools/agents/checks.d/45-lead-driven-harness.sh:3`

## Outside declared paths

- `/data02/home/philip.yang/workspace/corenius-lwr-21-scenarios-that-can-fail/tools/agents/tracker.py` - needed for finding 19 because blocker and resolve semantics live in the tracker.
- `/data02/home/philip.yang/workspace/corenius-lwr-21-scenarios-that-can-fail/tools/agents/test_tracker.py` - needed to cover finding 19, including the main-checkout plus integration-history regression.
- `/data02/home/philip.yang/workspace/corenius-lwr-21-scenarios-that-can-fail/tools/agents/worker.py` - needed for findings 12 and 13 because access-profile support records and wall-clock notices live in the worker supervisor.
- `/data02/home/philip.yang/workspace/corenius-lwr-21-scenarios-that-can-fail/tools/agents/test_worker.py` - needed to cover findings 12 and 13.

## For later tickets

- to 14: run each ticket's gate on a clean checkout of the exact commit before handing over; ticket 21's first hand-off passed in its worktree and failed in a clean checkout.

## Not checked

- `tools/agents/check.sh` was not run by instruction.
- cargo was not run by instruction.
- `tools/agents/test_sqlite_baseline.py` was not run because it hardcodes `/tmp`, and the worker brief explicitly says to leave it out of this gate.
- Real Codex or TraeCLI workers were not launched; only scripted and fake binaries were used by instruction.
- ResourceWarnings from long-lived fake subprocess cleanup in `tools/agents/test_worker.py` and unclosed file handles around `bench/harness/tasklib.py:90` were observed in passing suites and not fixed here.

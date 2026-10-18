## Built

Added the lead-driven readiness harness for the `handoff` scenario. It now
materializes the disposable repository through `bench/harness/tasklib.py` when a
scenario names a benchmark task, launches `tools/agents/worker.py` with a
scripted participant, and independently verifies the hand-off from git state
plus `run.json`.

Updated the toolbox hand-off rule. Exit mode remains the default. Marker mode is
available for interactive sessions that may stay alive: readiness is observed
when the branch tip subject equals the declared marker, the tree is clean, and
there is at least one commit after the launch base.

In both hand-off modes, the first ready observation pins the Candidate in the
run record. Later branch-tip movement is reported as `candidate-moved` by
`status` and by a later `supervise`, without replacing the pinned Candidate.

Updated the practice note and run-worker skill to state both modes and cite the
lead-worker-readiness decision file.

## Acceptance

- "A command runs the hand-off scenario offline with scripted participants and
  prints one result per participant; it exits 0 only when both results match
  their expectation." Proven by
  `tmp=$(mktemp -d); python3 bench/harness/lead_driven.py handoff --out "$tmp"; rc=$?; rm -rf "$tmp"; exit $rc`.
  Result: exit 0, printed:
  `passes handoff commit-ready expected=ready observed=ready` and
  `passes handoff dirty-final-message expected=not-ready observed=not-ready`.

- "The good participant's run is reported ready, and the recorded Candidate
  (commit, tree, base, brief digest) equals what `git` shows for the worker
  branch tip in the disposable repository." Proven by
  `python3 bench/harness/test_lead_driven.py`. Result: 9 tests OK. The specific
  test is `test_good_participant_records_candidate_equal_to_worker_branch_tip`,
  which checks the toolbox verdict's Candidate, not only the independent
  verifier's Candidate.

- "The participant that leaves a dirty tree, and a participant that exits
  without any commit, are each reported not ready; neither is ever reported
  ready." Proven by `python3 bench/harness/test_lead_driven.py` and by
  `tmp=$(mktemp -d); python3 bench/harness/lead_driven.py handoff --participant no-commit --participant move-foreign-branch --participant create-foreign-branch --participant rewrite-base --participant marker-ready --participant marker-dirty --participant marker-rewrite --out "$tmp"; rc=$?; rm -rf "$tmp"; exit $rc`.
  Result: test suite OK; optional command exited 0 and printed
  `passes handoff no-commit expected=not-ready observed=not-ready` and
  `passes handoff marker-dirty expected=not-ready observed=not-ready`.

- "A worker that moves a branch other than its own, or whose branch no longer
  contains its base, is reported as a violation." Proven by
  `python3 bench/harness/test_lead_driven.py` and the optional scenario command
  above. Result: test suite OK; optional command exited 0 and printed
  `passes handoff move-foreign-branch expected=not-ready observed=not-ready`,
  `passes handoff create-foreign-branch expected=not-ready observed=not-ready`,
  and `passes handoff rewrite-base expected=not-ready observed=not-ready`.

- "The verifier reads the disposable repository with its git metadata and the
  run record only; a test shows it does not read the participant's own final
  message to decide." Proven by `python3 bench/harness/test_lead_driven.py`.
  Result: 9 tests OK. The specific test is
  `test_dirty_final_message_is_not_used_as_readiness_evidence`.

- "A worker commit on its own branch is no longer a violation in the toolbox;
  every toolbox worker test still passes, with tests that asserted the old rule
  rewritten to assert the new one." Proven by
  `python3 tools/agents/test_worker.py`. Result: 30 tests OK, with existing
  `subprocess ... is still running` ResourceWarning noise.

- "The fleet runner's tests still pass unchanged." Proven by
  `python3 bench/harness/test_fleet.py`. Result: 9 tests OK, with existing
  `bench/harness/tasklib.py:90` ResourceWarning noise. Also ran
  `for t in bench/harness/test_*.py; do python3 "$t" || exit 1; done`; result:
  exit 0.

- "The practice note and the run-worker skill state the new hand-off rule and
  cite the decision file." Proven by source inspection:
  `docs/practice/lead-supervisor-worker.md:47` and
  `.claude/skills/run-worker/SKILL.md:27`.

Review 1 follow-up checks:

- Scenario outcome now judges the toolbox verdict against the independent
  verifier. Proven by `python3 bench/harness/test_lead_driven.py`. Result: 9
  tests OK. The specific doctored-verdict test is
  `test_doctored_toolbox_verdict_disagreements_fail_the_scenario`.
- Marker hand-off mode pins a Candidate before process exit, rejects marker
  hand-off with a dirty tree, and reports later branch-tip movement as
  `candidate-moved`. Proven by `python3 tools/agents/test_worker.py` and
  `python3 bench/harness/test_lead_driven.py`. Result: 30 worker tests OK and 9
  lead-driven tests OK.
- Scenario materialization uses the benchmark task library for
  `t01-log-window`. Proven by `python3 bench/harness/test_lead_driven.py`.
  Result: 9 tests OK. The specific test is
  `test_scenario_materializes_a_benchmark_task_with_tasklib`.
- `python3 tools/agents/tracker.py lint` exited 0.

## Claims

- [R] toolbox forbidden command patterns no longer include `git commit` `tools/agents/worker.py:34`
- [R] marker hand-off launch requires an explicit marker string `tools/agents/worker.py:258`
- [R] worker launch records a SHA-256 digest of the launched brief `tools/agents/worker.py:273`
- [R] worker launch records baseline heads for local branches other than the worker branch `tools/agents/worker.py:331`
- [R] worker launch records the selected hand-off mode `tools/agents/worker.py:334`
- [R] worker launch records the selected hand-off marker `tools/agents/worker.py:335`
- [R] ready hand-off Candidate records the worker branch commit from `HEAD` `tools/agents/worker.py:476`
- [R] ready hand-off Candidate records the worker branch tree from `HEAD^{tree}` `tools/agents/worker.py:477`
- [R] ready hand-off Candidate records the launch base commit `tools/agents/worker.py:478`
- [R] ready hand-off Candidate records the launched brief digest `tools/agents/worker.py:479`
- [R] candidate movement is reported as `candidate-moved` `tools/agents/worker.py:491`
- [R] marker hand-off readiness checks the branch tip subject against the marker `tools/agents/worker.py:508`
- [R] hand-off readiness requires a clean tree because dirty status adds `dirty-tree` `tools/agents/worker.py:512`
- [R] hand-off readiness requires at least one commit after base because count less than one adds `no-commit-after-base` `tools/agents/worker.py:521`
- [R] the first ready hand-off persists the Candidate into `run.json` `tools/agents/worker.py:529`
- [R] marker-mode supervise can stop once readiness is observed while the process is still alive `tools/agents/worker.py:572`
- [R] the lead-driven scenario materializes named benchmark tasks through `tasklib.materialize` `bench/harness/lead_driven.py:86`
- [R] the lead-driven verifier reads the worker run record from `run.json` `bench/harness/lead_driven.py:97`
- [R] the lead-driven verifier reads disposable repo cleanliness from `git status --porcelain` `bench/harness/lead_driven.py:102`
- [R] the lead-driven verifier detects a newly-created foreign branch `bench/harness/lead_driven.py:135`
- [R] the lead-driven verifier builds a ready Candidate commit from the disposable repo `HEAD` `bench/harness/lead_driven.py:142`
- [R] scenario judging fails when toolbox and verifier disagree on readiness `bench/harness/lead_driven.py:175`
- [R] scenario judging fails when the toolbox Candidate commit differs from the verifier Candidate commit `bench/harness/lead_driven.py:187`
- [R] candidate movement polling uses `worker.py status` after initial readiness `bench/harness/lead_driven.py:215`
- [R] the handoff scenario uses benchmark task `t01-log-window` for materialization `bench/readiness/handoff/scenario.toml:4`
- [R] the handoff scenario includes a `create-foreign-branch` participant expected not ready `bench/readiness/handoff/scenario.toml:32`
- [R] the handoff scenario includes a marker-ready participant with marker hand-off mode `bench/readiness/handoff/scenario.toml:47`
- [R] the handoff scenario includes a marker-rewrite participant that expects candidate movement `bench/readiness/handoff/scenario.toml:65`
- [R] the scripted participant can create a new foreign branch `bench/readiness/handoff/scripted_participant.py:61`
- [R] the scripted participant can make a marker commit and remain alive `bench/readiness/handoff/scripted_participant.py:82`
- [R] the scripted marker-dirty participant leaves `result.txt` dirty before the marker hand-off check `bench/readiness/handoff/scripted_participant.py:94`
- [R] the doctored-verdict test asserts readiness disagreement fails a scenario `bench/harness/test_lead_driven.py:101`
- [R] the doctored-verdict test asserts Candidate commit disagreement fails a scenario `bench/harness/test_lead_driven.py:116`
- [R] the materialization test checks that the scenario recorded `t01-log-window` `bench/harness/test_lead_driven.py:143`
- [R] the practice note cites the lead-worker-readiness decision file for the rule update `docs/practice/lead-supervisor-worker.md:47`
- [R] the practice note describes marker mode for sessions that may remain alive `docs/practice/lead-supervisor-worker.md:56`
- [R] the run-worker skill documents marker-mode launch flags `.claude/skills/run-worker/SKILL.md:31`
- [R] the run-worker skill says later branch-tip movement is reported as `candidate-moved` `.claude/skills/run-worker/SKILL.md:35`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/01-report.md`: required by this
  ticket's report-and-hand-off instructions.

## For later tickets

none

## Not checked

- I did not run cargo directly, per the ticket instructions.
- I did not run `CHECK_CARGO=0 sh tools/agents/check.sh` after review 1 because
  the lead review corrected that it is not the ticket gate.
- I did not run a benchmark task verifier for `t01-log-window`; review 1 asked
  only for task-library materialization, not capsule execution or verifier run.
- I did not run real `traecli` or `codex` model workers; all worker executions
  used fake/scripted binaries.

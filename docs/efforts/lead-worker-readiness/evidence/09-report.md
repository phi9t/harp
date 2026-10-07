## Built

`worker.py launch` now accepts `--external-protect-path` for absolute protected files or directories outside the repository. It records an external-state baseline before the worker process starts, including kind, size, modification time, symlink target, and small-file digests.

The worker supervisor compares that state cheaply while the worker runs and again with digests after exit. It reports modified, removed, and added protected entries as boundary violations, ignores additions below declared `--external-add-path` roots, and uses neutral wording.

The lead-driven benchmark has an `external-state-report` readiness scenario for stories 34 and 35, backed by a run-owned external-state fixture and an independent harness verifier.

## Acceptance

- "A missing, relative or in-repository protected path is refused before any worktree or branch is created."
  Proven by `python3 tools/agents/test_worker.py -k external_protected` -> OK, specifically `test_external_protected_path_is_validated_before_worktree_creation`.
- "The baseline file's timestamp precedes the worker process's start time in the run record."
  Proven by `python3 tools/agents/test_worker.py -k external_protected` -> OK, specifically `test_external_protected_baseline_precedes_worker_start`; also covered by `python3 bench/harness/test_lead_driven.py -k external_state_report` -> OK.
- "Modifying, removing and adding an entry each produce a report line with correct counts; an addition under a declared add-path is not reported."
  Proven by `python3 tools/agents/test_worker.py -k external_protected` -> OK, specifically `test_external_protected_changes_report_counts_and_ignore_declared_add_path`; also covered by `python3 bench/harness/lead_driven.py external-state-report --out <tmp>` -> both participants passed.
- "A content change that keeps size and restores the modification time is caught by the final comparison."
  Proven by `python3 tools/agents/test_worker.py -k external_protected` -> OK, specifically `test_external_protected_final_digest_catches_same_size_same_mtime_change`.
- "A symlink beneath a protected path is recorded by target and not followed."
  Proven by `python3 tools/agents/test_worker.py -k external_protected` -> OK, specifically `test_external_protected_symlink_records_target_and_is_not_followed`.
- "The report wording states a change during the run and never names the worker as the cause; a test asserts the wording."
  Proven by `python3 tools/agents/test_worker.py -k external_protected` -> OK, specifically `test_external_protected_changes_report_counts_and_ignore_declared_add_path`; also covered by `python3 bench/harness/test_lead_driven.py -k external_state_report` -> OK.
- "All scenarios run in the lead-driven mode and match their expectation."
  Proven by the required readiness scenario commands, all exit 0 and print `passes`: `access-profile`, `external-state-report`, `handoff`, `land-exact-candidate`, and `land-stale-candidate`.

Full gate run:

- `python3 tools/agents/test_worker.py` -> OK, 42 tests.
- `python3 bench/harness/test_fleet.py` -> OK, 9 tests.
- `python3 bench/harness/test_lead_driven.py` -> OK, 19 tests.
- `python3 bench/harness/test_run.py` -> OK, 17 tests.
- `python3 bench/harness/test_tasks_tracked.py` -> OK, 2 tests.
- `python3 bench/harness/test_validate.py` -> OK, 9 tests.
- `python3 bench/harness/lead_driven.py access-profile --out <tmp>` -> all participants passed.
- `python3 bench/harness/lead_driven.py external-state-report --out <tmp>` -> all participants passed.
- `python3 bench/harness/lead_driven.py handoff --out <tmp>` -> all participants passed.
- `python3 bench/harness/lead_driven.py land-exact-candidate --out <tmp>` -> all participants passed.
- `python3 bench/harness/lead_driven.py land-stale-candidate --out <tmp>` -> all participants passed.
- `python3 tools/agents/tracker.py lint` -> OK.
- `git diff --check` -> OK.

## Claims

- [R] the worker CLI exposes `--external-protect-path` for protected host files or directories `tools/agents/worker.py:884`
- [R] relative external protected paths are rejected `tools/agents/worker.py:273`
- [R] missing external protected paths are rejected `tools/agents/worker.py:276`
- [R] in-repository external protected paths are rejected `tools/agents/worker.py:278`
- [R] external protected paths are validated before the worker branch and worktree are created `tools/agents/worker.py:405`
- [R] the external-state baseline file is written before `subprocess.Popen` starts the worker `tools/agents/worker.py:516`
- [R] the run record stores the external-state baseline metadata `tools/agents/worker.py:536`
- [R] the run record's worker start timestamp is recorded after the process is spawned `tools/agents/worker.py:550`
- [R] external-state entries record the entry kind `tools/agents/worker.py:310`
- [R] external-state entries record size `tools/agents/worker.py:311`
- [R] external-state entries record modification time `tools/agents/worker.py:312`
- [R] symlink entries record their target with `os.readlink` `tools/agents/worker.py:315`
- [R] regular files up to the digest limit are eligible for a content digest `tools/agents/worker.py:316`
- [R] additions under declared external add paths are excluded from added-entry reports `tools/agents/worker.py:384`
- [R] each supervisor cycle runs a no-hash external-state comparison `tools/agents/worker.py:671`
- [R] the final supervisor pass runs a hash-enabled external-state comparison `tools/agents/worker.py:809`
- [R] external-state findings say the state "changed during the run" `tools/agents/worker.py:622`
- [R] external-state findings are emitted as boundary violations `tools/agents/worker.py:626`
- [R] the lead-driven harness parses `external_protect_paths` from scenario configuration `bench/harness/lead_driven.py:97`
- [R] the lead-driven harness passes each materialized external protected path to `worker.py` `bench/harness/lead_driven.py:536`
- [R] the lead-driven independent verifier adds external-state violations to readiness checking `bench/harness/lead_driven.py:308`
- [R] the new readiness scenario declares stories 34 and 35 `bench/readiness/external-state-report/scenario.toml:2`
- [R] the new readiness scenario declares `external-state` as a protected path `bench/readiness/external-state-report/scenario.toml:8`

## Outside declared paths

- `bench/harness/lead_driven.py`: needed so the lead-driven scenario runner can materialize run-owned external state, pass protected paths to `worker.py`, and independently verify stories 34 and 35.
- `bench/harness/test_lead_driven.py`: needed to prove `external-state-report` runs through lead-driven mode and matches expectations.
- `docs/efforts/lead-worker-readiness/evidence/09-report.md`: required hand-off report for this ticket.

## For later tickets

none

## Not checked

- `tools/agents/check.sh` and cargo were not run, per ticket instruction.
- No real model worker, traecli, or codex run was launched; tests use fake/scripted participants.
- Local-controller and workflow-controller instantiations were not run.

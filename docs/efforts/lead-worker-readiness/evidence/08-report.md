## Built

- Worker launches now declare and record an access profile: network, single-command timeout, external add paths, container-build permission, and shared-cache-write permission.
- Worker standing instructions are generated from that profile, and worker/capsule launch arguments receive the profile's timeout and network mode.
- Supervisor command findings now follow the profile, with legacy run records still readable.
- Lead-driven readiness includes an access-profile scenario covering default network, network off, a 125-second command budget, and an external add path.
- Supervisor git probes run with `GIT_OPTIONAL_LOCKS=0` because the worker-suite fake worker commit path and the supervisor's `git status` check raced on `.git/index.lock`; before this change the suite intermittently failed with `fatal: Unable to create ... index.lock`, and the final repeated worker-suite and tools-suite runs passed with the lock-free read probes.

## Acceptance

- "A default launch records network on, and the worker's instructions no longer say there is no network; a scripted participant performs a loopback fetch and no finding is raised."
  Evidence: `python3 tools/agents/test_worker.py` passed (37 tests OK), including `test_default_access_profile_records_network_on_and_allows_downloads`; `python3 bench/harness/lead_driven.py access-profile --out "$tmp"` passed `default-network expected=ready observed=ready`.

- "A launch with the network declared off records it, tells the worker, and raises a finding for a download command; inside a capsule the fetch fails."
  Evidence: `python3 tools/agents/test_worker.py` passed (37 tests OK), including `test_network_off_records_prompt_and_flags_download_command` and `test_network_off_is_enforced_inside_a_capsule`; `python3 bench/harness/lead_driven.py access-profile --out "$tmp"` passed `network-off-download expected=not-ready observed=not-ready`.

- "The single-command time limit is a launch option recorded in the run; a scripted command longer than the old two-minute limit completes when the launch allows it."
  Evidence: `python3 tools/agents/test_worker.py` passed (37 tests OK), including `test_command_timeout_is_a_launch_option_recorded_and_passed_to_worker`; `python3 bench/harness/lead_driven.py access-profile --out "$tmp"` passed `long-command expected=ready observed=ready`.

- "Declared external add-paths appear in the worker's instructions and the launch record; container builds and shared-cache writes are recorded as not allowed unless declared."
  Evidence: `python3 tools/agents/test_worker.py` passed (37 tests OK), including `test_external_add_paths_and_restricted_capabilities_are_recorded_in_prompt` and `test_container_builds_and_shared_cache_writes_follow_the_profile`; `python3 bench/harness/lead_driven.py access-profile --out "$tmp"` passed `external-add-path expected=ready observed=ready`.

- "Runs recorded before this change still supervise without error."
  Evidence: `python3 tools/agents/test_worker.py` passed (37 tests OK), including `test_run_recorded_before_access_profiles_still_supervises`.

- "All scenarios run in the lead-driven mode and match their expectation."
  Evidence: `python3 bench/harness/lead_driven.py handoff --out "$tmp"` passed `commit-ready expected=ready observed=ready` and `dirty-final-message expected=not-ready observed=not-ready`; `python3 bench/harness/lead_driven.py land-exact-candidate --out "$tmp"` passed `commit-ready expected=landed observed=landed`; `python3 bench/harness/lead_driven.py land-stale-candidate --out "$tmp"` passed `commit-ready expected=stale observed=stale`; `python3 bench/harness/lead_driven.py access-profile --out "$tmp"` passed `default-network`, `network-off-download`, `long-command`, and `external-add-path`.

Additional gate evidence: `python3 -m unittest discover -s tools/agents -p 'test_*.py'` passed (194 tests OK); `python3 -m unittest discover -s bench/harness -p 'test_*.py'` passed (55 tests OK); `python3 bench/harness/lead_driven.py --report "$tmp"` listed `Scenario: access-profile` and `Stories: 31, 32, 33, 36`; `python3 tools/agents/tracker.py lint` passed with exit 0.

## Claims

- [R] worker git read checks disable optional locks so supervisor status checks do not contend with worker commits `tools/agents/worker.py:103`
- [R] this report records why worker git read checks disable optional locks `docs/efforts/lead-worker-readiness/evidence/08-report.md:7`
- [R] the launch access profile records the network mode from CLI arguments `tools/agents/worker.py:213`
- [R] the launch access profile records the per-command timeout in seconds `tools/agents/worker.py:214`
- [R] the launch access profile records resolved external add paths `tools/agents/worker.py:215`
- [R] the launch access profile records whether container builds are allowed `tools/agents/worker.py:216`
- [R] the launch access profile records whether shared-cache writes are allowed `tools/agents/worker.py:217`
- [R] legacy run records without `access_profile` still receive a default command timeout `tools/agents/worker.py:233`
- [R] generated worker instructions say network access is available when the profile network is on `tools/agents/worker.py:242`
- [R] generated worker instructions say network access is off when the profile network is off `tools/agents/worker.py:243`
- [R] generated worker instructions list declared external add paths `tools/agents/worker.py:247`
- [R] non-capsule worker launches pass the profile timeout to `--shell-tool-timeout` `tools/agents/worker.py:359`
- [R] capsule worker launches set capsule networking from the access profile `tools/agents/worker.py:339`
- [R] new run records include the computed access profile `tools/agents/worker.py:389`
- [R] legacy runs ignore the old fixed network/container/cache command regexes as standing forbidden patterns `tools/agents/worker.py:452`
- [R] network commands are findings only when the profile declares network off `tools/agents/worker.py:523`
- [R] container build commands are findings when container builds are not allowed `tools/agents/worker.py:525`
- [R] shared-cache write commands are findings when shared-cache writes are not allowed `tools/agents/worker.py:527`
- [R] the worker CLI exposes `--network` with default `on` `tools/agents/worker.py:708`
- [R] the worker CLI exposes `--command-timeout` with default `120` `tools/agents/worker.py:710`
- [R] worker tests cover default network recording and absence of a network finding `tools/agents/test_worker.py:209`
- [R] worker tests cover network-off recording, prompt text, and download finding `tools/agents/test_worker.py:223`
- [R] worker tests cover command-timeout recording and worker handoff `tools/agents/test_worker.py:233`
- [R] worker tests cover external add paths plus container/cache prompt defaults and declared allows `tools/agents/test_worker.py:241`
- [R] worker tests cover container-build and shared-cache-write findings following the profile `tools/agents/test_worker.py:264`
- [R] worker tests cover records created before access profiles `tools/agents/test_worker.py:398`
- [R] worker tests cover network off inside a capsule `tools/agents/test_worker.py:496`
- [R] the lead-driven harness parses per-participant access-profile fields `bench/harness/lead_driven.py:83`
- [R] the lead-driven harness independently derives command-event access violations from the run profile `bench/harness/lead_driven.py:135`
- [R] the lead-driven verifier converts access violations into boundary violations `bench/harness/lead_driven.py:184`
- [R] the lead-driven harness forwards computed access arguments to worker launch `bench/harness/lead_driven.py:430`
- [R] the lead-driven result record includes `case_dir` for external path assertions `bench/harness/lead_driven.py:489`
- [R] harness tests assert the access-profile scenario emits four passing participant lines `bench/harness/test_lead_driven.py:321`
- [R] harness tests assert the default participant records network on and stays ready `bench/harness/test_lead_driven.py:325`
- [R] harness tests assert the offline participant records network off and receives a network-command finding `bench/harness/test_lead_driven.py:330`
- [R] harness tests assert the long-command participant records a 125-second command timeout `bench/harness/test_lead_driven.py:335`
- [R] harness tests assert the external-add-path participant records the resolved external path and default container/cache denials `bench/harness/test_lead_driven.py:339`
- [R] harness tests assert `--report` lists the access-profile scenario stories `bench/harness/test_lead_driven.py:348`
- [R] the access-profile readiness scenario declares stories 31, 32, 33, and 36 `bench/readiness/access-profile/scenario.toml:2`
- [R] the access-profile readiness scenario defines the default-network participant as expected-ready `bench/readiness/access-profile/scenario.toml:9`
- [R] the access-profile readiness scenario defines the network-off-download participant with network off and expected not ready `bench/readiness/access-profile/scenario.toml:14`
- [R] the access-profile readiness scenario defines the long-command participant with a 125-second command timeout `bench/readiness/access-profile/scenario.toml:20`
- [R] the access-profile readiness scenario defines the external-add-path participant with `external-adds` `bench/readiness/access-profile/scenario.toml:26`
- [R] the scripted default participant performs a loopback fetch before committing `bench/readiness/access-profile/scripted_participant.py:72`
- [R] the scripted network-off participant emits a curl command event `bench/readiness/access-profile/scripted_participant.py:74`
- [R] the scripted long-command participant rejects a timeout other than `125s` `bench/readiness/access-profile/scripted_participant.py:78`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/08-report.md`: required by the worker handoff instructions for ticket 08 evidence.

## For later tickets

- to 06: Supervisor git probes should use `GIT_OPTIONAL_LOCKS=0`; otherwise read-only checks like `git status` can race with worker commits on `.git/index.lock` and fail spuriously.

## Not checked

- `tools/agents/check.sh`, by instruction.
- Cargo tests/builds, by instruction.
- Real traecli or Codex/model worker launches, by instruction; verification used fake/scripted participants.

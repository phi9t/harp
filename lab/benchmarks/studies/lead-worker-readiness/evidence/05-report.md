## Built

- Added tracker support for the `Triage:` body field, `launchable`, and worker-question recording.
- Added `tools/agents/lead_launch.py`, which refuses non-launchable tickets, batch-claims before worker launch, launches from the checked-out integration worktree, uses marker handoff, and records pinned source digests.
- Passed ticket 08 and ticket 09 access-profile options through the lead launch path and into the launch record, including `--external-protect-path`.
- Added the lead-driven readiness scenario `launch-by-edges-and-triage`, including the two-worktree integration checkout case required by review 1.
- Reintegrated on current `effort/lead-worker-readiness` after ticket 09 and preserved `external-state-report` plus this ticket's `lead-launch` scenario kind.

Ticket 05 handles integration-branch tracker churn by claiming every requested ticket in one integration-branch commit before any worker branch is cut, then using that post-claim integration tip as every launched worker's base. A stale Candidate from before a lead claim or ticket resolution is therefore not reused by this launch path; later review/rework logic still owns already-reviewed Candidate reuse refusal.

## Acceptance

- "The tracker prints the launchable set: status open, every blocker resolved, triage `ready-for-agent`; ordered by id; with a JSON form."
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_tracker.py`: PASS.
  - Proven by `python3 bench/harness/lead_driven.py launch-by-edges-and-triage --out "$tmp"` in the required scenario loop: PASS, including `passes launch-by-edges-and-triage launchable-set expected=checked observed=checked`.

- "Launch is refused, with a distinct message each, for: an unresolved blocker, an unparseable blocker entry, triage other than `ready-for-agent` (including a missing `Triage:` line), and a launch that would exceed the parallel cap."
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py`: PASS, including unresolved blocker, unparseable blocker, human triage, missing triage, and cap refusal checks.
  - Proven by `python3 bench/harness/lead_driven.py launch-by-edges-and-triage --out "$tmp"` in the required scenario loop: PASS, including `unresolved-blocker-refused`, `bad-blocker-refused`, `human-triage-refused`, `missing-triage-refused`, and `parallel-cap-refused`.

- "A launched worker's branch starts at the current tip of the named integration branch, not at the mainline."
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py`: PASS, including single-checkout and linked-worktree integration launch checks.
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/test_lead_driven.py`: PASS, including assertions that the two-worktree case launches from integration, claims on integration, and leaves main checkout/mainline untouched.
  - Proven by `python3 bench/harness/lead_driven.py launch-by-edges-and-triage --out "$tmp"` in the required scenario loop: PASS, including `integration-base-and-pinned-brief` and `two-worktree-integration-checkout`.

- "The launch record holds the commit and content digest of the ticket, the spec and the worker instructions that formed the brief; a test changes the instructions after launch and shows the record still identifies the launched version."
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py`: PASS.
  - Proven by `python3 bench/harness/lead_driven.py launch-by-edges-and-triage --out "$tmp"` in the required scenario loop: PASS, including `passes launch-by-edges-and-triage integration-base-and-pinned-brief expected=checked observed=checked`.

- "A worker question moves the ticket's triage to `needs-info` and appends the question to its comments; lint accepts the result."
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_tracker.py`: PASS.
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py`: PASS.
  - Proven by `python3 bench/harness/lead_driven.py launch-by-edges-and-triage --out "$tmp"` in the required scenario loop: PASS, including `passes launch-by-edges-and-triage worker-question expected=checked observed=checked`.

- "Every existing ticket file in the repository still lints clean."
  - Proven by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/tracker.py lint`: PASS in the final required gate.

- "All scenarios run in the lead-driven mode and match their expectation."
  - Proven by final required scenario commands: `handoff`, `land-exact-candidate`, `land-stale-candidate`, `access-profile`, `external-state-report`, and `launch-by-edges-and-triage` all PASS.
  - Proven by every `bench/harness/test_*.py` suite in the final required gate: PASS.

Additional checks:
- Every `tools/agents/test_*.py` suite: PASS in the final required gate.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_worker.py` repeated three consecutive times after the ticket 09 rebase: PASS.
- The earlier isolated failure of `WorkerTest.test_external_add_paths_and_restricted_capabilities_are_recorded_in_prompt` did not reproduce: that test passed once directly, then five more direct repeats, then the full worker suite passed three consecutive repeats and again in the final full gate.
- The first post-ticket-09 full gate surfaced stale core-count expectations in `bench/harness/test_lead_driven.py`; those expectations now include both `external-state-report` and `launch-by-edges-and-triage`, and the full gate was rerun cleanly.
- `git diff --check`: PASS.
- `python3 -m py_compile tools/agents/lead_launch.py tools/agents/tracker.py bench/harness/lead_driven.py tools/agents/test_lead_launch.py tools/agents/test_tracker.py bench/harness/test_lead_driven.py`: PASS.

## Claims

- [R] the tracker accepts only the canonical triage values `tools/agents/tracker.py:39`
- [R] ticket parsing records a valid `Triage:` value on the ticket object `tools/agents/tracker.py:246`
- [R] new tracker tickets start with `Triage: needs-triage` `tools/agents/tracker.py:453`
- [R] `launchable` only includes open, blocker-resolved, `ready-for-agent` tickets `tools/agents/tracker.py:578`
- [R] `launchable --json` prints the machine-readable rows `tools/agents/tracker.py:587`
- [R] worker questions set triage to `needs-info` and append a `question:` comment `tools/agents/tracker.py:603`
- [R] the launch command finds the worktree where the integration branch is checked out by using the lander helper `tools/agents/lead_launch.py:57`
- [R] the launch command refuses when the integration branch is checked out nowhere `tools/agents/lead_launch.py:59`
- [R] the launch command refuses an unparseable blocker on the target ticket `tools/agents/lead_launch.py:131`
- [R] the launch command refuses unresolved blocker entries on the target ticket `tools/agents/lead_launch.py:134`
- [R] the launch command refuses a ticket whose triage is not `ready-for-agent` `tools/agents/lead_launch.py:150`
- [R] the launch command enforces the parallel cap before launching workers `tools/agents/lead_launch.py:305`
- [R] the launch command claims the batch before computing the worker base and launching workers `tools/agents/lead_launch.py:306`
- [R] the worker base is the current named integration branch commit after the claim commit `tools/agents/lead_launch.py:307`
- [R] pinned sources include commit, sha256, and size for each source path `tools/agents/lead_launch.py:80`
- [R] pinned launch sources include the ticket, spec, and standing worker instructions `tools/agents/lead_launch.py:195`
- [R] the brief includes the exact ready marker expected from marker handoff `tools/agents/lead_launch.py:250`
- [R] the launch command passes `--network` through to `worker.py` `tools/agents/lead_launch.py:257`
- [R] the launch command passes `--command-timeout` through to `worker.py` `tools/agents/lead_launch.py:259`
- [R] the launch command passes every `--external-add-path` through to `worker.py` `tools/agents/lead_launch.py:269`
- [R] the launch command passes every `--external-protect-path` through to `worker.py` `tools/agents/lead_launch.py:271`
- [R] the launch parser accepts `--external-protect-path` for lead launches `tools/agents/lead_launch.py:356`
- [R] the launch record stores lead-launch access profile metadata in `run.json` `tools/agents/lead_launch.py:287`
- [R] the lead-launch test verifies that external protected paths reach the worker external-state baseline `tools/agents/test_lead_launch.py:274`
- [R] the lead `question` wrapper delegates to tracker question handling `tools/agents/lead_launch.py:324`
- [R] the lead-driven harness keeps ticket 09's `external_protect_paths` participant option `bench/harness/lead_driven.py:106`
- [R] the lead-driven harness passes participant external protected paths to worker launches `bench/harness/lead_driven.py:585`
- [R] the lead-driven harness can run a launch command from `--repo .` in the chosen checkout `bench/harness/lead_driven.py:720`
- [R] the lead-driven harness records the two-worktree claim, main checkout, and mainline invariants `bench/harness/lead_driven.py:821`
- [R] the lead-driven harness fails a two-worktree launch case if those invariants do not hold `bench/harness/lead_driven.py:834`
- [R] the readiness scenario covers stories 18 through 24 `bench/readiness/launch-by-edges-and-triage/scenario.toml:2`
- [R] the readiness scenario defines the launchable-set expectation `bench/readiness/launch-by-edges-and-triage/scenario.toml:15`
- [R] the readiness scenario includes the two-worktree integration checkout case `bench/readiness/launch-by-edges-and-triage/scenario.toml:54`
- [R] the lead-driven unit test asserts that the two-worktree case leaves main checkout and mainline untouched `bench/harness/test_lead_driven.py:102`
- [R] the lead-driven unit test asserts the post-ticket-09 core summary counts with this ticket's scenario included `bench/harness/test_lead_driven.py:249`
- [R] the lead-driven unit test still covers ticket 09's external-state scenario after reintegration `bench/harness/test_lead_driven.py:383`
- [R] the ticket-format contract documents the `Triage:` body field values `docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md:14`
- [R] the ticket-format contract documents that legacy missing triage still lints but cannot launch `docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md:28`
- [R] the ticket-format contract documents the `launchable` command and JSON form `docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md:36`

## Outside declared paths

- `bench/harness/lead_driven.py`: required so the new readiness scenario can run as `kind = "lead-launch"` and exercise the product launch command; review 1 also required two-worktree scenario support in the lead-driven harness, and reintegration required preserving ticket 09's external-state support.
- `bench/harness/test_lead_driven.py`: required to lock the harness discovery, lead-launch scenario behavior, two-worktree invariants, and rebased core-count expectations beside the changed harness code.
- `docs/efforts/lead-worker-readiness/evidence/05-report.md`: required by this worker handoff instruction.

## For later tickets

- to 03: ticket 05 launches with marker handoff and records pinned launch sources; already-reviewed Candidate reuse refusal remains for review/rework acceptance logic.
- to 04: ticket 05 launches with marker handoff and records pinned launch sources; already-reviewed Candidate reuse refusal remains for review/rework acceptance logic.
- to 06: `lead_launch.py` records live launched runs with `lead_launch.effort` and `parallel_cap`, but overlap/path conflict detection between running tickets is not implemented here.
- to 11: `lead_launch.py` centralizes brief assembly in `build_brief()`; later-ticket findings should be inserted there once tracker/handoff parsing is ready.

## Not checked

- Did not run `tools/agents/check.sh` or cargo, per ticket instruction.
- Did not launch real model workers or real `traecli`/Codex sessions; tests and scenarios use fake binaries/scripts only.

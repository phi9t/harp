## Built

Rebased this ticket branch onto `effort/lead-worker-readiness` after ticket 13 landed, preserving ticket 13's readiness reporting and notes work plus this ticket's landing scenarios.

`tools/agents/land.py` still resolves the positional input once to an exact Candidate commit, gates a detached checkout of that commit with `LAND_TIP`, and refuses stale Candidates with exit code 6. Review fix 1 restores checked-out target handling: if the target branch is checked out in any worktree, the lander checks that target checkout for overlapping tracked changes and advances it with `git merge --ff-only` so the ref, index and files move together. If the target branch is not checked out, the lander keeps the compare-and-swap `git update-ref` path.

The two landing readiness scenarios now declare their stories. The exact landing scenario also verifies that the checked-out target is clean after landing and that the landed `result.txt` exists in that checkout. The report path is covered by a test and by a direct `lead_driven.py --report` run after both landing scenarios.

## Acceptance

- "Landing a Candidate leaves the target branch tip equal to the Candidate commit byte for byte; the gate ran on that commit (the gate sees it as `LAND_TIP`)." Proven by `python3 tools/agents/test_land.py`, result: 13 tests OK. The specific tests are `test_exact_candidate_commit_lands_without_moving_worker_branch_and_gate_sees_it` and `test_landing_checked_out_target_keeps_checkout_clean_and_populated`. Also proven by `tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT; python3 bench/harness/lead_driven.py land-exact-candidate --out "$tmp"`, result: exit 0 and printed `passes land-exact-candidate commit-ready expected=landed observed=landed`.

- "When the target has moved, landing exits with its own code, changes no ref, and leaves the worker's worktree and branch exactly as they were (checked by comparing `git status` and the branch tip before and after)." Proven by `python3 tools/agents/test_land.py`, result: 13 tests OK. The specific test is `test_stale_branch_candidate_refuses_without_touching_worker_worktree`. Also proven by `tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT; python3 bench/harness/lead_driven.py land-stale-candidate --out "$tmp"`, result: exit 0 and printed `passes land-stale-candidate commit-ready expected=stale observed=stale`.

- "No code path in the lander runs a rebase or writes in a worktree it did not create." Review 1 clarified this as: do not rebase and do not write a worker worktree; the target branch's own checkout must be advanced with the target ref. Proven by source scan `rg -n "rebase|reset|merge|worktree" tools/agents/land.py`, result: no `rebase` or `reset`; the only worktree creation is the detached gate checkout and the only merge command is `merge --ff-only` in the target checkout. Also proven by `python3 tools/agents/test_land.py`, result: 13 tests OK, including worker branch/status preservation and checked-out target cleanliness.

- "A failing gate changes no ref." Proven by `python3 tools/agents/test_land.py`, result: 13 tests OK. The specific test is `test_failing_gate_leaves_local_mainline_and_remote_unchanged`.

- "The race test (two landings at once) still passes: exactly one lands and the other is refused as stale." Proven by `python3 tools/agents/test_land.py`, result: 13 tests OK. The specific test is `test_concurrent_landings_leave_exactly_one_landed_and_one_stale`.

- "Both scenarios run in the lead-driven mode from ticket 01 and match their expectation." Proven by `tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT; python3 bench/harness/lead_driven.py land-exact-candidate --out "$tmp"`, result: exit 0 and printed `passes land-exact-candidate commit-ready expected=landed observed=landed`; and by `tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT; python3 bench/harness/lead_driven.py land-stale-candidate --out "$tmp"`, result: exit 0 and printed `passes land-stale-candidate commit-ready expected=stale observed=stale`. I also ran the existing readiness scenario command `tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT; python3 bench/harness/lead_driven.py handoff --out "$tmp"`, result: exit 0 and printed the two expected pass rows.

- "The lander's and the worktree tool's existing tests pass, rewritten where they asserted rebase-on-land." Proven by `python3 tools/agents/test_land.py`, result: 13 tests OK; and `python3 tools/agents/test_wt.py`, result: 12 tests OK.

Additional required gates:

- `python3 bench/harness/test_fleet.py`, result: 9 tests OK. Existing `ResourceWarning` noise from `bench/harness/tasklib.py:90` appeared.
- `python3 bench/harness/test_lead_driven.py`, result: 17 tests OK.
- `python3 bench/harness/test_run.py`, result: 17 tests OK. Existing `ResourceWarning` noise from `bench/harness/tasklib.py:90` appeared.
- `python3 bench/harness/test_tasks_tracked.py`, result: 2 tests OK.
- `python3 bench/harness/test_validate.py`, result: 9 tests OK. Existing `ResourceWarning` noise from `bench/harness/tasklib.py:90` appeared.
- `tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT; python3 bench/harness/lead_driven.py land-exact-candidate --out "$tmp" >"$tmp/exact.out" 2>"$tmp/exact.err"; python3 bench/harness/lead_driven.py land-stale-candidate --out "$tmp" >"$tmp/stale.out" 2>"$tmp/stale.err"; python3 bench/harness/lead_driven.py --report "$tmp"`, result: exit 0; report listed `Scenario: land-exact-candidate`, `Stories: 5, 27`, `Scenario: land-stale-candidate`, `Stories: 28, 30`, and lead-driven pass rows for both.
- `python3 tools/agents/tracker.py lint`, result: exit 0 with no output.

TDD red evidence:

- After adding the checked-out target lander tests but before changing `land.py`, `python3 tools/agents/test_land.py` failed as intended: two tests saw `D  topic.txt` in the checked-out target after landing, and the linked target worktree overlap test got exit 0 instead of `OVERLAP`.
- After adding the scenario checkout-state assertion but before extending `bench/harness/lead_driven.py`, `python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_exact_candidate_landing_scenario_lands_candidate_and_binds_gate` errored as intended with `KeyError: 'target_status_after'`.
- After adding the report test but before adding `stories` to the scenario TOMLs, `python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_report_lists_landing_scenarios_after_runs_with_stories` failed as intended because the report printed `Stories: not declared`.

## Claims

- [R] the lander defines `STALE_CANDIDATE` as exit code 6 `tools/agents/land.py:19`
- [R] the lander can find the worktree that has a target branch checked out `tools/agents/land.py:72`
- [R] the target-worktree lookup matches the full `refs/heads/...` branch name from `git worktree list --porcelain` `tools/agents/land.py:76`
- [R] overlap detection reads status from the target checkout when one exists `tools/agents/land.py:89`
- [R] overlap detection compares dirty tracked paths against the Candidate range `tools/agents/land.py:90`
- [R] the gate checkout is created as a detached worktree of the exact Candidate commit `tools/agents/land.py:114`
- [R] each gate receives `LAND_TIP` set to the exact Candidate commit `tools/agents/land.py:132`
- [R] the lander pushes the exact Candidate commit to the target branch ref `tools/agents/land.py:144`
- [R] stale refusal prints that a new Candidate on the new base is required `tools/agents/land.py:168`
- [R] checked-out target branches are advanced with `git merge --ff-only -q` `tools/agents/land.py:177`
- [R] non-checked-out target refs still use `git update-ref` with the expected old value `tools/agents/land.py:179`
- [R] landing resolves the requested Candidate once before any gate runs `tools/agents/land.py:191`
- [R] overlapping tracked target-checkout changes are refused before the remote push and local fast-forward steps `tools/agents/land.py:195`
- [R] stale Candidates are refused before gates run `tools/agents/land.py:205`
- [R] failing gates return `GATE_FAILED` before any target update `tools/agents/land.py:217`
- [R] the exact-candidate lander test asserts the gate-observed tip equals the Candidate `tools/agents/test_land.py:101`
- [R] the exact-candidate lander test asserts the worker branch tip stays at its later commit `tools/agents/test_land.py:103`
- [R] the checked-out target landing test asserts the target checkout is clean after landing `tools/agents/test_land.py:117`
- [R] the checked-out target landing test asserts the landed file exists in the target checkout `tools/agents/test_land.py:118`
- [R] the checked-out target overlap test asserts `OVERLAP` is returned `tools/agents/test_land.py:132`
- [R] the linked target worktree overlap test lands against `--mainline integration` `tools/agents/test_land.py:160`
- [R] the linked target worktree overlap test asserts the integration ref stays unchanged `tools/agents/test_land.py:165`
- [R] the stale-candidate test asserts the gate did not run `tools/agents/test_land.py:190`
- [R] the stale-candidate test asserts the target branch stays unchanged `tools/agents/test_land.py:191`
- [R] the concurrent landing test asserts one process exits OK and one exits stale `tools/agents/test_land.py:234`
- [R] landing verification records the target checkout status after the land command `bench/harness/lead_driven.py:267`
- [R] landing verification reads the landed `result.txt` from the target checkout when present `bench/harness/lead_driven.py:270`
- [R] landing verification classifies `landed` only when the target checkout status is clean `bench/harness/lead_driven.py:281`
- [R] landing verification stores `target_status_after` in the scenario result record `bench/harness/lead_driven.py:324`
- [R] the exact-candidate readiness test checks target tip equals the Candidate `bench/harness/test_lead_driven.py:284`
- [R] the exact-candidate readiness test checks gate observation equals the Candidate `bench/harness/test_lead_driven.py:285`
- [R] the exact-candidate readiness test checks the target checkout is clean after landing `bench/harness/test_lead_driven.py:286`
- [R] the exact-candidate readiness test checks the landed `result.txt` content `bench/harness/test_lead_driven.py:287`
- [R] the stale-candidate readiness test checks worker branch before and after are equal `bench/harness/test_lead_driven.py:297`
- [R] the report test asserts `land-exact-candidate` appears in `--report` output `bench/harness/test_lead_driven.py:311`
- [R] the report test asserts `land-stale-candidate` story numbers appear in `--report` output `bench/harness/test_lead_driven.py:314`
- [R] the exact-candidate readiness scenario declares stories 5 and 27 `bench/readiness/land-exact-candidate/scenario.toml:2`
- [R] the stale-candidate readiness scenario declares stories 28 and 30 `bench/readiness/land-stale-candidate/scenario.toml:2`

## Outside declared paths

- `bench/harness/lead_driven.py`: required so lead-driven scenarios can execute and independently judge the new landing behavior, including checked-out target checkout status.
- `bench/harness/test_lead_driven.py`: required to drive the readiness scenario behavior and report checks test-first.
- `docs/efforts/lead-worker-readiness/evidence/02-report.md`: required by this ticket's report-and-hand-off instructions.

## For later tickets

- to 07: landing onto a checked-out target branch must update that checkout with `git merge --ff-only` so the ref, index and files advance together; `git update-ref` alone moves the ref but can leave the checkout staged to delete landed files.

## Not checked

- I did not run `tools/agents/check.sh`, per the ticket instructions and ticket 01's handoff finding.
- I did not run cargo, per the ticket instructions.
- I did not run real `traecli` or `codex` model workers; all readiness commands used scripted participants.

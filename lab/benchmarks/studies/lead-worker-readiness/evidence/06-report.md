## Built

Added lead-side overlap reporting for active workers in one effort, including launch-time declared-path overlap notes, a read-only `lead_launch.py overlap` command, running changed-path conflicts, common changed paths, and behind-integration status.

Preserved ticket 16's split code/record checkout model after rebasing onto `effort/lead-worker-readiness` at `9ab6f26`: launch reads and claims tickets through the effort record worktree, while worker branches are based on the integration worktree. The overlap command also resolves the effort's integration branch from a main checkout where effort records are not present.

Added supervisor accounting for dirty allowed paths that are not explained by the worker event record, with help text stating the attribution limit. Codex `file_change` events now account for edited paths alongside shell command events; unexplained dirty allowed paths are a NOTE while the worker is still running and become a VIOLATION after a Candidate was pinned.

Added the `parallel-workers-and-overlap` readiness scenario. It launches two scripted workers at once, records overlapping intervals, lands ticket 01, refuses ticket 02's stale first candidate, resumes ticket 02 onto the moved integration branch, and lands its new candidate.

## Acceptance

- "A scenario runs two scripted workers whose running intervals overlap (shown from recorded start and end times) and both Candidates land; the second lands as a new Candidate on the moved base." Proved by `python3 bench/harness/test_lead_driven.py` in the full bench loop (`Ran 28 tests ... OK`) and by the readiness loop: `passes parallel-workers-and-overlap two-workers expected=checked observed=checked`. The scenario asserts `intervals_overlap`, landed tickets `["01", "02"]`, a changed second ticket 02 candidate, and ticket 02's new base matching the integration tip after ticket 01 landed.
- "A worker that writes outside its declared paths produces a finding naming the path while it is still running." Proved by `python3 tools/agents/test_lead_launch.py` in the full tools loop (`Ran 17 tests ... OK`) and the readiness loop. The `overlap` command reports `changed-outside-declared` for the running worker's `outside.txt`.
- "Two running tickets that declare a common path are reported at launch; a worker writing a path declared by the other running ticket produces a finding naming both tickets." Proved by `python3 tools/agents/test_lead_launch.py` in the full tools loop (`Ran 17 tests ... OK`) and the readiness loop. Batch launch reports `declared-path-overlap`, and the running overlap report emits `changed-path-declared-by-other` with both ticket ids.
- "A file changed in a worker's workspace by a process outside the worker's process tree is reported as a violation by the supervisor, or, where the process cannot be identified, as a change the worker's event record does not account for; the limits of this detection are stated in the tool's help." Proved by `python3 tools/agents/test_worker.py` in the full tools loop (`Ran 64 tests ... OK`). The supervisor help contains the attribution limit text, Codex `file_change` paths account for dirty allowed paths, an unexplained dirty allowed path is a NOTE while still running, and status on a post-candidate externally changed allowed path reports `unaccounted-worktree-change` as a VIOLATION.
- "All scenarios run in the lead-driven mode and match their expectation." Proved by the full readiness loop `for d in bench/readiness/*; do test -f "$d/scenario.toml" || continue; scenario=${d##*/}; tmp=$(mktemp -d); python3 bench/harness/lead_driven.py "$scenario" --out "$tmp"; code=$?; rm -rf "$tmp"; if test $code -ne 0; then exit $code; fi; done`, which exited 0 and printed `passes ... expected=... observed=...` for every scenario, including `passes parallel-workers-and-overlap two-workers expected=checked observed=checked` and all `rework-round` cases.

Additional gates run after rebasing onto `effort/lead-worker-readiness`:
- `for t in tools/agents/test_*.py; do printf 'RUN %s\n' "$t"; python3 "$t" || exit $?; done` exited 0. It ran `test_acceptance.py` (15), `test_claims.py` (5), `test_effort.py` (15), `test_evidence.py` (15), `test_land.py` (24), `test_lead_launch.py` (17), `test_mac_dashboard.py` (9), `test_mac_state_view.py` (20), `test_mac_tmux.py` (35), `test_monitor_wake.py` (15), `test_msg.py` (7), `test_sqlite_baseline.py` (4), `test_sqlite_responses_relay.py` (2), `test_tracker.py` (22), `test_worker.py` (64), and `test_wt.py` (12). Existing ResourceWarnings from fake subprocess tests appeared.
- `for t in bench/harness/test_*.py; do printf 'RUN %s\n' "$t"; python3 "$t" || exit $?; done` exited 0. It ran `test_fleet.py` (9), `test_lead_driven.py` (28), `test_run.py` (17), `test_tasks_tracked.py` (2), and `test_validate.py` (9). Existing `bench/harness/tasklib.py:90` ResourceWarnings appeared.
- full readiness loop `for d in bench/readiness/*; do test -f "$d/scenario.toml" || continue; scenario=${d##*/}; tmp=$(mktemp -d); python3 bench/harness/lead_driven.py "$scenario" --out "$tmp"; code=$?; rm -rf "$tmp"; if test $code -ne 0; then exit $code; fi; done` exited 0, ending with all `rework-round` cases and including `passes parallel-workers-and-overlap two-workers expected=checked observed=checked`.
- `python3 tools/agents/tracker.py lint` exited 0.
- `git diff --check` exited 0.

## Claims

- [R] lead-side git helpers set `GIT_OPTIONAL_LOCKS=0` for git probes `tools/agents/lead_launch.py:42`
- [R] worker git probes set `GIT_OPTIONAL_LOCKS=0` for git status and other checks `tools/agents/worker.py:362`
- [R] a worker's own report path is removed before declared-path and changed-path overlap analysis `tools/agents/lead_launch.py:136`
- [R] launch-time common declared paths produce `declared-path-overlap` findings `tools/agents/lead_launch.py:179`
- [R] running overlap reports emit `changed-outside-declared` findings for changed paths outside a ticket's declared paths `tools/agents/lead_launch.py:303`
- [R] running overlap reports emit `changed-path-declared-by-other` findings when a changed path is declared by a different active ticket `tools/agents/lead_launch.py:316`
- [R] running overlap reports include a `common_changed_paths` list `tools/agents/lead_launch.py:345`
- [R] running overlap reports emit `common-undeclared-changed-path` findings for paths changed by more than one active ticket and declared by none of them `tools/agents/lead_launch.py:335`
- [R] launch resolves the integration worktree, then resolves the effort record worktree through ticket 16's lookup `tools/agents/lead_launch.py:528`
- [R] launch reads ticket and spec sources from the record worktree and worker instructions from the code worktree `tools/agents/lead_launch.py:573`
- [R] `lead_launch.py overlap` resolves the effort integration before printing the read-only active overlap report `tools/agents/lead_launch.py:586`
- [R] batch-launched peer worker branches are recorded as peer branches, not foreign branch moves `tools/agents/lead_launch.py:521`
- [R] worker supervision skips peer worker branches when checking branch baselines `tools/agents/worker.py:1008`
- [R] Codex runtime observability includes `file_change` events `tools/agents/worker.py:62`
- [R] supervisor help states unaccounted workspace-change detection is based on recorded worker commands or file-change events `tools/agents/worker.py:74`
- [R] supervisor help states it cannot identify the writer process `tools/agents/worker.py:79`
- [R] runtime event parsing extracts paths from `file_change` item changes `tools/agents/worker.py:179`
- [R] the runtime adapter yields paths from `file_change` events `tools/agents/worker.py:243`
- [R] dirty-path accounting checks file-change paths as well as command text `tools/agents/worker.py:951`
- [R] running unaccounted dirty allowed paths are reported as NOTE findings `tools/agents/worker.py:958`
- [R] post-candidate allowed dirty paths emit `unaccounted-worktree-change` as a violation `tools/agents/worker.py:962`
- [R] the fake Codex runtime covers the real `file_change` item shape followed by `git add -A` `tools/agents/test_worker.py:279`
- [R] a dirty allowed path accounted for by a `file_change` event does not emit `unaccounted-worktree-change` `tools/agents/test_worker.py:494`
- [R] a dirty allowed path with no command or file-change event is a NOTE while the worker is still running `tools/agents/test_worker.py:1048`
- [R] external protected paths that contain the repository are rejected before launch `tools/agents/worker.py:561`
- [R] external protected paths that contain worker run storage are rejected before launch `tools/agents/worker.py:564`
- [R] the overlap regression moves effort records to a record branch before launching from the main checkout `tools/agents/test_lead_launch.py:682`
- [R] the overlap regression verifies ticket claims landed in the record branch, not the main or integration checkout `tools/agents/test_lead_launch.py:712`
- [R] the overlap regression verifies worker reports are excluded from overlap findings `tools/agents/test_lead_launch.py:748`
- [R] the parallel-overlap harness initializes the test repository in a two-worktree layout `bench/harness/lead_driven.py:1287`
- [R] the parallel-overlap launch command includes ticket `01` `bench/harness/lead_driven.py:1309`
- [R] the parallel-overlap launch command includes ticket `02` `bench/harness/lead_driven.py:1310`
- [R] the parallel-overlap harness collects the running overlap report while both workers are active `bench/harness/lead_driven.py:1344`
- [R] the parallel-overlap harness computes whether worker start and end intervals overlap `bench/harness/lead_driven.py:1364`
- [R] the parallel-overlap harness attempts to land ticket 02's first candidate as stale before resuming it `bench/harness/lead_driven.py:1373`
- [R] the parallel-overlap harness resumes ticket 02 through `worker.py resume` after the moved-base refusal `bench/harness/lead_driven.py:1388`
- [R] the parallel-overlap harness checks ticket 02's second candidate base against the integration tip after ticket 01 landed `bench/harness/lead_driven.py:1396`
- [R] the lead-driven dispatcher routes `kind = "parallel-overlap"` to the parallel-overlap runner `bench/harness/lead_driven.py:1939`
- [R] the new readiness scenario is declared as `kind = "parallel-overlap"` `bench/readiness/parallel-workers-and-overlap/scenario.toml:3`
- [R] the new readiness scenario sets `parallel_cap = 2` `bench/readiness/parallel-workers-and-overlap/scenario.toml:10`
- [R] ticket 01 in the fixture declares `shared.txt` `bench/readiness/parallel-workers-and-overlap/environment/docs/efforts/demo/tickets/01-alpha.md:14`
- [R] ticket 02 in the fixture declares `shared.txt` `bench/readiness/parallel-workers-and-overlap/environment/docs/efforts/demo/tickets/02-beta.md:14`
- [R] the lead-driven test asserts the two worker intervals overlap `bench/harness/test_lead_driven.py:582`
- [R] the lead-driven test asserts both tickets landed `bench/harness/test_lead_driven.py:583`
- [R] the lead-driven test asserts ticket 02's second candidate is based on the tip after ticket 01 landed `bench/harness/test_lead_driven.py:585`
- [R] the lead-driven test asserts all three overlap finding kinds appear in the scenario result `bench/harness/test_lead_driven.py:586`
- [R] the lead-driven test asserts unaccounted workspace-change reporting appears in the scenario result `bench/harness/test_lead_driven.py:590`

## Outside declared paths

- `bench/harness/lead_driven.py`: required to add the lead-driven scenario kind and runner that can launch two scripted workers, inspect overlap while they are alive, land one candidate, drive the second through resume, and record the readiness row.
- `bench/harness/test_lead_driven.py`: required to add the regression test for the new lead-driven readiness scenario and its observable results.
- `docs/efforts/lead-worker-readiness/evidence/06-report.md`: required by the ticket hand-off contract.

## For later tickets

- to 14: under the workspace-write sandbox a worker in a git worktree cannot commit (index.lock under the main repository's .git is read-only); found by a real Codex run on 2026-10-05. The live trial cannot use sandboxed worktree workers until a ticket fixes this.

## Not checked

- Did not run `tools/agents/check.sh` or cargo, per ticket instruction.
- Did not launch real `traecli` or `codex` model workers; all worker execution used fake/scripted binaries, per ticket instruction.

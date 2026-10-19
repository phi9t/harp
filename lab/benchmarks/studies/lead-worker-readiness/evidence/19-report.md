## Built

Ticket 19 now hardens the lead/worker toolbox for shared repositories:
worker launch preflights options before creating worker state, git subprocesses
scrub inherited repository environment, worker status is read-only by default,
run-record mutations are locked, ticket landings require committed acceptance
from an effort record branch, no-gate and owner-mainline landings are explicit,
and branch movement in the shared repository is downgraded to non-blocking
notes.

Review 1 is incorporated after rebasing onto `effort/lead-worker-readiness` at
`660b3b7`: ticket landings for efforts without a `Record branch` now refuse
before gates run, with exit code 11 and a message naming
`effort.py move-records <effort>`. The `land.py --ticket` and
`acceptance.py record` help now say the same and no longer imply an
uncommitted local record can authorize landing.

The readiness harness now has a `shared-repository-safe` scenario, uses
record-branch acceptance in landing scenarios, and has an
`integration-branch` case proving the no-record-branch refusal in the
three-checkout layout.

## Acceptance

> Finding 1: while a worker runs, a commit on an unrelated branch of the shared repository, a new unrelated branch, and the lead landing another ticket on the integration branch each leave the worker's verdict unaffected: reported at most as a NOTE that does not name the worker, never `VIOLATED`, never in `not_ready`. A worker moving a branch other than its own in its own repository is still a violation. A scenario shows a worker reaching ready while three other branches move.

Proved by `python3 tools/agents/test_worker.py`: 80 tests OK, including
`test_shared_repository_branch_movement_by_other_agents_is_non_blocking_note`
and `test_moving_a_foreign_branch_is_a_violation`. Also proved by the scenario
loop: `shared-repository-safe unrelated-branches-move expected=ready
observed=ready`.

> Finding 3: where an effort's record is read from is decided by the record itself, not by the checkout a command runs in: the record branch if the effort names one, else the integration branch; a copy in another checkout is used only if that checkout has that branch checked out. Landing reads the acceptance record from the committed tip of that branch (an uncommitted or untracked record never authorizes a landing; say so in the help and remove the write-uncommitted-then-land text). A hand-written untracked `acceptance-NN.jsonl` in a third worktree does not authorize a landing. `tracker.py list`, `acceptance.py summary` and `effort.py status` give the same answer from every checkout.

Proved by `python3 tools/agents/test_acceptance.py`: 16 tests OK,
`python3 tools/agents/test_tracker.py`: 23 tests OK,
`python3 tools/agents/test_effort.py`: 16 tests OK, and
`python3 tools/agents/test_land.py`: 29 tests OK. Scenario proof:
`acceptance-missing-record`, `acceptance-other-candidate`,
`acceptance-failing-check`, and `acceptance-ticket-tamper` all observed
`acceptance-refused`. Review-1 proof: `test_ticket_landing_without_record_branch_refuses_before_gate`
asserts exit code 11, unchanged integration tip, no gate execution, and stderr
that names `effort.py move-records sample`; the `integration-branch`
readiness scenario prints `ticket-without-record-branch-refused expected=checked
observed=checked`.

> Finding 5: recording a landing on a record branch commits only the landing record; an untracked draft and an uncommitted ticket edit in the record worktree are left as they were.

Proved by `python3 tools/agents/test_land.py`: 29 tests OK, including the
record-branch landing tests that assert only the landing record path is staged
and committed.

> Finding 6: before pushing, the lander checks that the target checkout can fast-forward, including untracked files the Candidate would overwrite; if it cannot, nothing is pushed. If a push succeeded and the local fast-forward then fails, that is reported with its own exit code and message, not as a stale Candidate.

Proved by `python3 tools/agents/test_land.py`: 29 tests OK. The suite covers
tracked overlap, untracked overlap, and the distinct local fast-forward failure
exit path.

> Finding 7: a landing on the effort's base branch, or on `main` when no effort is given, requires the owner override flag and records it; an effort record with no `Base:` line is treated as based on `main`; a landing with no gate is refused unless a flag says so explicitly.

Proved by `python3 tools/agents/test_land.py`: 29 tests OK. Scenario proof:
`integration-branch mainline-refused expected=checked observed=checked`,
`integration-branch mainline-owner-override expected=checked observed=checked`,
and all scenario landings that intentionally skip a gate pass `--allow-no-gate`.

> Finding 8: a batch launch that fails after claiming leaves no ticket claimed without a running worker: workspaces are validated and created before the claim, or the claims of tickets that did not launch are released in the same command. Works with the default temp directory unusable (`TMPDIR` pointing at a read-only directory).

Proved by `python3 tools/agents/test_lead_launch.py`: 32 tests OK, including
`test_failed_batch_launch_releases_tickets_that_did_not_launch_with_bad_tmpdir`.

> Finding 14: `worker.py status` writes nothing unless given a flag whose help says what it writes; `supervise`, `status` and `deliver-findings` update the run record under a lock so a concurrent pair cannot lose a write (test with two processes).

Proved by `python3 tools/agents/test_worker.py`: 80 tests OK, including the
read-only status, explicit `--update-run-record`, and concurrent locked update
tests. The rebased ticket-20 delivery acknowledgement test now calls
`status --update-run-record` before expecting any acknowledgement observation to
persist.

> Finding 18: every git command the harness and the toolbox run has `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and `GIT_COMMON_DIR` removed from its environment; a scenario run with `GIT_DIR` pointing at a decoy repository leaves the decoy untouched.

Proved by `python3 tools/agents/test_worker.py`: 80 tests OK,
`python3 bench/harness/test_lead_driven.py`: 30 tests OK, and the readiness
scenario loop, including `shared-repository-safe unrelated-branches-move
expected=ready observed=ready`.

> Finding 22: `worker.py launch` validates every option before it creates anything; `--base` accepts a branch name and resolves it in the repository given by `--repo`; a refused launch leaves no clone and no run directory.

Proved by `python3 tools/agents/test_worker.py`: 80 tests OK, including
`test_launch_validates_options_before_creating_run_or_clone`,
`test_launch_validates_base_before_creating_run_or_clone`,
`test_launch_validates_remaining_options_before_creating_run_or_clone`, and
`test_launch_base_branch_resolves_in_requested_repo`.

> `AGENTS.md` describes the lander as it is (no rebase) and names `lead_launch.py` and `acceptance.py`.

Proved by direct file inspection after the edit: `AGENTS.md` names
`lead_launch.py`, describes `land.py` as push then fast-forward with no rebase,
and names `acceptance.py`.

> Every existing scenario still matches its expectation; `tracker.py lint` is clean on every effort.

Proved by the current-tree scenario loop:
`python3 bench/harness/lead_driven.py <scenario> --out "$(mktemp -d)"` for all
17 readiness scenarios. Every scenario printed `OK <scenario>`. Also proved by
`python3 tools/agents/tracker.py lint`: exit 0.

Additional current-tree gate:
- `python3 tools/agents/test_acceptance.py`: exit 0, 16 tests OK.
- `python3 tools/agents/test_claims.py`: exit 0, 5 tests OK.
- `python3 tools/agents/test_effort.py`: exit 0, 16 tests OK.
- `python3 tools/agents/test_evidence.py`: exit 0, 15 tests OK.
- `python3 tools/agents/test_land.py`: exit 0, 29 tests OK.
- `python3 tools/agents/test_lead_launch.py`: exit 0, 32 tests OK.
- `python3 tools/agents/test_mac_dashboard.py`: exit 0, 9 tests OK.
- `python3 tools/agents/test_mac_state_view.py`: exit 0, 20 tests OK.
- `python3 tools/agents/test_mac_tmux.py`: exit 0, 35 tests OK.
- `python3 tools/agents/test_monitor_wake.py`: exit 0, 15 tests OK.
- `python3 tools/agents/test_msg.py`: exit 0, 7 tests OK.
- `python3 tools/agents/test_sqlite_responses_relay.py`: exit 0, 2 tests OK.
- `python3 tools/agents/test_tracker.py`: exit 0, 23 tests OK.
- `python3 tools/agents/test_worker.py`: exit 0, 80 tests OK.
- `python3 tools/agents/test_wt.py`: exit 0, 12 tests OK.
- `python3 bench/harness/test_fleet.py`: exit 0, 9 tests OK.
- `python3 bench/harness/test_lead_driven.py`: exit 0, 30 tests OK.
- `python3 bench/harness/test_run.py`: exit 0, 17 tests OK.
- `python3 bench/harness/test_tasks_tracked.py`: exit 0, 2 tests OK.
- `python3 bench/harness/test_validate.py`: exit 0, 9 tests OK.
- `python3 -m py_compile tools/agents/land.py tools/agents/worker.py tools/agents/lead_launch.py tools/agents/acceptance.py bench/harness/lead_driven.py bench/harness/test_lead_driven.py`: exit 0.
- `git diff --check`: exit 0.

The full tools run emitted expected negative-case stderr and ResourceWarnings
from fake worker tests. The full bench harness run emitted pre-existing
`bench/harness/tasklib.py:90 ResourceWarning: unclosed file
<_io.BufferedReader ...>` warnings.

## Claims

- [R] worker git subprocesses remove `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and `GIT_COMMON_DIR` before running git `tools/agents/worker.py:364`
- [R] the readiness harness git wrapper removes the same inherited git environment variables `bench/harness/lead_driven.py:102`
- [R] lander git subprocesses use the scrubbed git environment helper `tools/agents/land.py:86`
- [R] lead-launch git subprocesses use the worker scrubbed git environment helper `tools/agents/lead_launch.py:40`
- [R] branch movement inside the worker clone is still a `foreign-branch-moved` violation `tools/agents/worker.py:1385`
- [R] branch movement in the shared repository is recorded only as `shared-branch-moved` NOTE findings `tools/agents/worker.py:1400`
- [R] `worker.py status` loads the run with `persist=False` when `--update-run-record` is not passed `tools/agents/worker.py:2027`
- [R] the status help says the default writes nothing and names `--update-run-record` for writes `tools/agents/worker.py:2261`
- [R] run-record mutation paths share an exclusive `run.lock` flock `tools/agents/worker.py:1210`
- [R] `worker.py status --update-run-record` enters the locked run-record path `tools/agents/worker.py:2023`
- [R] launch rejects non-positive command timeouts before creating worker state `tools/agents/worker.py:1042`
- [R] launch validates allowed paths before creating worker state `tools/agents/worker.py:1044`
- [R] launch rejects marker handoff without a marker before creating worker state `tools/agents/worker.py:1047`
- [R] launch validates external add paths before creating worker state `tools/agents/worker.py:1053`
- [R] launch validates external protected paths before creating worker state `tools/agents/worker.py:1054`
- [R] launch rejects an existing worker branch before creating worker state `tools/agents/worker.py:1062`
- [R] `--base` is resolved against the requested repository before clone creation `tools/agents/worker.py:1064`
- [R] worker launch creates the run directory only after the preflight checks above `tools/agents/worker.py:1070`
- [R] the private clone accepts an already-resolved base commit `tools/agents/worker.py:559`
- [R] the private clone uses scrubbed git subprocesses `tools/agents/worker.py:565`
- [R] ticket landings without a record branch return the dedicated `RECORD_BRANCH_REQUIRED` exit code 11 `tools/agents/land.py:32`
- [R] ticket landings without a record branch refuse before gate handling `tools/agents/land.py:451`
- [R] the no-record-branch refusal message names `effort.py move-records` for the effort `tools/agents/land.py:453`
- [R] ticket landing acceptance is read from the committed record branch using `git show` `tools/agents/acceptance.py:284`
- [R] the lander reads ticket acceptance only from the effort record branch after the early refusal passes `tools/agents/land.py:489`
- [R] `land.py --ticket` help says ticket landings need a record branch and points to `effort.py move-records <effort>` `tools/agents/land.py:545`
- [R] `acceptance.py record` help says ticket landings need a record branch and points to `effort.py move-records <effort>` `tools/agents/acceptance.py:588`
- [R] the land regression test verifies no-record-branch refusal preserves the integration tip and does not run the gate `tools/agents/test_land.py:895`
- [R] the acceptance help regression test rejects the old write-uncommitted wording `tools/agents/test_acceptance.py:527`
- [R] harness acceptance preparation creates a dedicated record branch worktree before recording acceptance `bench/harness/lead_driven.py:695`
- [R] integration-branch landing scenarios record acceptance on a record worktree before landing `bench/harness/lead_driven.py:1774`
- [R] the integration-branch status scenario records acceptance on a record worktree instead of the integration worktree `bench/harness/lead_driven.py:1871`
- [R] the integration-branch no-record-branch scenario commits acceptance only on the integration branch before invoking the lander `bench/harness/lead_driven.py:1908`
- [R] the no-record-branch scenario asserts exit code 11, unchanged integration tip, no gate execution, and move-records stderr `bench/harness/lead_driven.py:1937`
- [R] the integration-branch scenario test expects `ticket-without-record-branch-refused` `bench/harness/test_lead_driven.py:517`
- [R] the integration-branch scenario test checks the refusal exit code, unchanged tip, skipped gate, and `effort.py move-records demo` stderr `bench/harness/test_lead_driven.py:538`
- [R] the readiness scenario file registers `ticket-without-record-branch-refused` `bench/readiness/integration-branch/scenario.toml:28`
- [R] landing without a gate is refused unless `--allow-no-gate` is set `tools/agents/land.py:458`
- [R] the lander checks untracked target-checkout paths that overlap candidate changes `tools/agents/land.py:320`
- [R] local fast-forward failure has its own `LOCAL_FF_FAILED` exit code `tools/agents/land.py:31`
- [R] a local fast-forward failure reports `local fast-forward failed` instead of stale-candidate text `tools/agents/land.py:509`
- [R] record-branch landing passes only the generated landing record path to the effort-record commit helper `tools/agents/land.py:511`
- [R] owner override is required for protected target branch landings `tools/agents/land.py:440`
- [R] batch launch releases tickets that did not get a worker after a launch failure `tools/agents/lead_launch.py:672`
- [R] lead launch creates brief temp directories under the explicit `TMPDIR` `tools/agents/lead_launch.py:521`
- [R] lead launch reports temp-directory `OSError` as `WorkerError` `tools/agents/lead_launch.py:522`
- [R] worker subprocesses launched by lead launch run with scrubbed git environment `tools/agents/lead_launch.py:571`
- [R] the rebased terminal-delivery test requires `status --update-run-record` before expecting delayed acknowledgement state to persist `tools/agents/test_worker.py:826`
- [R] the new shared-repository-safe scenario is registered as a readiness scenario `bench/readiness/shared-repository-safe/scenario.toml:1`
- [R] `AGENTS.md` names `lead_launch.py` in the tools paragraph `AGENTS.md:18`
- [R] `AGENTS.md` says `land.py` never rebases `AGENTS.md:20`
- [R] `AGENTS.md` names `acceptance.py` in the tools paragraph `AGENTS.md:21`

## Outside declared paths

- `bench/harness/test_lead_driven.py`: added/updated harness regression coverage for the declared `bench/harness/lead_driven.py` scenario behavior.
- `docs/efforts/lead-worker-readiness/evidence/19-report.md`: required hand-off report for this ticket and review round.

## For later tickets

none

## Not checked

- `tools/agents/test_sqlite_baseline.py` was not run, per instruction, because it hardcodes `/tmp` on a host where `/tmp` has no free inodes.
- `tools/agents/check.sh` was not run, per instruction, because it runs cargo and hangs in this worktree.
- Cargo was not run, per instruction.
- No real `traecli` or `codex` model workers were launched; tests used fake binaries.

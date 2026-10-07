# Ticket 22 report

## Built

Worker launches now create a run-owned temp directory at `RUN_DIR/tmp`, record it in `run.json`, pass `TMPDIR`, `TMUX_TMPDIR` and `TMPPREFIX` into detached, interactive and resumed runtime processes, and clean it up after stopped-run accounting unless `--keep-temp` is set. The rebase onto `effort/lead-worker-readiness` kept ticket 18's interactive tmux launch path and added the same run-owned temp environment to the tmux window and script.

Toolbox temporary directories now use `TMPDIR` when set, otherwise `CORENIUS_RUNS/tmp`; the lander gate worktree, lead-launch brief workspace, rework brief workspace, SQLite baseline run root and harness scenario repositories follow that rule. A new `checks.d` script rejects host `/tmp` literals under `tools/agents/`, `tools/insula/` and `bench/harness/` except documented in-capsule mount destinations.

## Acceptance

> Every worker launch, interactive or not, gives the worker a temp directory of its own inside the run directory and starts the runtime with `TMPDIR`, `TMUX_TMPDIR` and `TMPPREFIX` pointing into it; for a sandboxed runtime that directory is writable (a writable root for Codex); the run record names it; it is removed when the run is accounted as stopped unless a flag keeps it. A scripted participant that calls `tempfile.mkdtemp()`, starts a tmux server and runs a shell here-document creates nothing under the host's `/tmp` (the test compares a listing of a stand-in directory, not the real `/tmp`: make the temp root injectable).

Proved by the final clean detached checkout gate: every `tools/agents/test_*.py` suite passed, including `tools/agents/test_worker.py` (`Ran 88 tests ... OK`), `tools/agents/test_lead_launch.py` (`Ran 48 tests ... OK`) and `tools/agents/test_sqlite_baseline.py` (`Ran 5 tests ... OK`). The worker temp test verifies the runtime sees the three temp variables, `tempfile.mkdtemp()`, a tmux server and a shell here-document all use the run-owned temp root, the stand-in forbidden temp root stays empty, the run record names the temp object, and stopped-run cleanup removes it. The rebased interactive launch tests verify the script exports the run-owned variables and the tmux window receives them with `-e`.

> The standing brief generated for a worker states the rule in one line and names the worker's temp directory.

Proved by `tools/agents/test_worker.py` in the final detached gate; the temp probe reads `prompt.md` and checks that the generated brief contains the recorded temp directory. The generated brief line is in `tools/agents/worker.py`.

> No toolbox command creates anything under the host's `/tmp` when `TMPDIR` is unset: where a temp directory is needed and `TMPDIR` is unset, it uses a directory under the runs root (the same root `CORENIUS_RUNS` moves). The lander's and the harness's temporary worktrees and scenario repositories follow the same rule. A test runs the lander, a launch, an acceptance record and one scenario with `TMPDIR` unset and a stand-in "forbidden" directory and shows it stays empty.

Proved by the final clean detached checkout gate: every `tools/agents/test_*.py` suite passed, every `bench/harness/test_*.py` suite passed, and every scenario directory under `bench/readiness/` passed with `expected=... observed=...`. The no-host-temp tests cover worker launch, lead launch, lander, acceptance record, SQLite baseline and a lead-driven scenario with `TMPDIR` unset and a stand-in forbidden temp root.

> `tools/agents/sqlite_baseline.py` and `tools/agents/test_sqlite_baseline.py` no longer hardcode `/tmp`; they use the same rule. If a short path is required there (a socket path limit, for example) say so in the report and use the shortest directory the rule allows; if the suite cannot be made to pass, leave those two files unchanged and record it under `## For later tickets` as `- to 14: ...` with the exact error.

Proved by `tools/agents/test_sqlite_baseline.py` in the final detached gate: `Ran 5 tests ... OK`. No shorter special-case path was needed; `new_run_root()` uses `TMPDIR` or `CORENIUS_RUNS/tmp`.

> A check script under `tools/agents/checks.d/` fails when a file under `tools/agents/`, `tools/insula/` or `bench/harness/` contains a host `/tmp` path literal outside a short allowlist kept in the script (the in-capsule mount destinations, with a comment for each entry); it passes on your branch.

Proved before rebase continuation by `tools/agents/checks.d/90-no-host-tmp.sh`: exit 0. Proved again in the final detached gate by the same script's inclusion in the checked tree and by `python3 tools/agents/tracker.py lint`: exit 0.

> `AGENTS.md` gains one rule under "Rules for working here": agents and tools do not use the host's `/tmp`; set `TMPDIR`, `TMUX_TMPDIR` and `TMPPREFIX` to a directory you own, and say where the launcher puts a worker's. The run-worker skill and the practice note say the same.

Proved by direct file inspection: `AGENTS.md`, `.claude/skills/run-worker/SKILL.md` and `docs/practice/lead-supervisor-worker.md` all state the no-host-temp rule and name the worker launch temp locations.

> The report states what still touches the host's `/tmp` and cannot be changed from here (as observed on this host: the Codex sandbox creates mount points under `/tmp` when it starts a command), with the evidence.

Still outside this repository's control: Codex's own sandbox setup can attempt host `/tmp` mount-point creation before a shell command runs. The effort decision records this as outside the repo's authority, and ticket-20 real Codex evidence records `bwrap: Can't mkdir /tmp/.aws: No space left on device` in both launch and resume event streams. This ticket did not launch real Codex workers by instruction.

> Every existing scenario still matches its expectation; `tracker.py lint` is clean.

Proved by the final clean detached checkout gate. It ran every `bench/harness/test_*.py` suite, then every scenario under `bench/readiness/` with `python3 bench/harness/lead_driven.py <scenario> --out "$(mktemp -d)"`, then `python3 tools/agents/tracker.py lint`; the whole loop exited 0.

## Claims

- [R] worker temp parents use `TMPDIR` when set and fall back to `CORENIUS_RUNS/tmp` when unset `tools/agents/worker.py:669`
- [R] a worker run temp root is created at `RUN_DIR/tmp` `tools/agents/worker.py:680`
- [R] a worker run temp record stores `TMUX_TMPDIR` under `RUN_DIR/tmp/tmux` `tools/agents/worker.py:687`
- [R] a worker run temp record stores `TMPPREFIX` as `RUN_DIR/tmp/zsh` `tools/agents/worker.py:688`
- [R] launched worker process environments receive the run temp variables `tools/agents/worker.py:708`
- [R] temp cleanup only removes a temp path inside the run directory and not the run directory itself `tools/agents/worker.py:731`
- [R] temp cleanup records `removed_at` after removing the temp directory `tools/agents/worker.py:733`
- [R] the worker brief includes the one-line host `/tmp` ban and the run's temp directory `tools/agents/worker.py:811`
- [R] resume ensures a temp record before building the resume command `tools/agents/worker.py:1125`
- [R] capsule launches set capsule-internal temp variables below `/tmp/run/tmp` `tools/agents/worker.py:1495`
- [R] non-capsule launches append the run temp directory as an extra writable root `tools/agents/worker.py:1507`
- [R] interactive launch scripts export the run-owned `TMPDIR` `tools/agents/worker.py:1524`
- [R] interactive launch scripts export the run-owned `TMUX_TMPDIR` `tools/agents/worker.py:1525`
- [R] interactive launch scripts export the run-owned `TMPPREFIX` `tools/agents/worker.py:1526`
- [R] interactive tmux windows receive the explicit run temp environment `tools/agents/worker.py:1542`
- [R] worker launch records the temp object in `run.json` `tools/agents/worker.py:1589`
- [R] supervise removes run temp after the process exits `tools/agents/worker.py:2490`
- [R] status removes run temp for stopped runs when persistence is enabled `tools/agents/worker.py:2559`
- [R] detached resume runs with the run temp environment `tools/agents/worker.py:2623`
- [R] foreground resume runs with the run temp environment `tools/agents/worker.py:2665`
- [R] tmux launch environment forwarding merges explicit run temp values with ambient `CORENIUS_*` values `tools/agents/worker.py:2381`
- [R] lead launch brief scratch space uses the worker temp helper instead of ambient default temp `tools/agents/lead_launch.py:616`
- [R] lead launch keeps ticket 18's interactive worker arguments `tools/agents/lead_launch.py:658`
- [R] lead launch propagates `--keep-temp` to worker launches `tools/agents/lead_launch.py:660`
- [R] fresh rework brief scratch space uses the worker temp helper `tools/agents/lead_launch.py:973`
- [R] lead-launch accounting cleans temp for stopped runs `tools/agents/lead_launch.py:1504`
- [R] `lead_launch launch` exposes `--keep-temp` for retained temp directories `tools/agents/lead_launch.py:1625`
- [R] lander temporary worktrees are created under `temp_parent()` `tools/agents/land.py:344`
- [R] SQLite baseline temp roots use `TMPDIR` when set and `CORENIUS_RUNS/tmp` when unset `tools/agents/sqlite_baseline.py:38`
- [R] SQLite baseline run roots are created with `tempfile.mkdtemp(..., dir=str(temp_parent()))` `tools/agents/sqlite_baseline.py:48`
- [R] the guard script scans `tools/agents`, `tools/insula` and `bench/harness` `tools/agents/checks.d/90-no-host-tmp.sh:11`
- [R] the guard script allowlist is limited to commented in-capsule mount destinations `tools/agents/checks.d/90-no-host-tmp.sh:18`
- [R] the guard script exits nonzero when a disallowed host `/tmp` literal is found `tools/agents/checks.d/90-no-host-tmp.sh:75`
- [R] the worker temp probe exercises `tempfile.mkdtemp()` and a tmux server when available `tools/agents/test_worker.py:73`
- [R] the worker temp probe runs a shell here-document to a path under `TMPDIR` `tools/agents/test_worker.py:85`
- [R] the worker temp regression compares a stand-in forbidden temp root before and after launch/supervise `tools/agents/test_worker.py:1328`
- [R] the worker temp regression checks the runtime saw all three run temp variables `tools/agents/test_worker.py:1345`
- [R] lead-launch interactive tests check tmux receives `TMPDIR` through `-e` `tools/agents/test_lead_launch.py:781`
- [R] lead-launch interactive tests check the interactive script exports all three run temp variables `tools/agents/test_lead_launch.py:786`
- [R] lead launch with `TMPDIR` unset leaves a stand-in forbidden temp root unchanged `tools/agents/test_lead_launch.py:508`
- [R] lander gate checkout with `TMPDIR` unset leaves a stand-in forbidden temp root unchanged `tools/agents/test_land.py:230`
- [R] acceptance record with `TMPDIR` unset leaves a stand-in forbidden temp root unchanged `tools/agents/test_acceptance.py:196`
- [R] SQLite baseline tests verify `new_run_root()` avoids the stand-in forbidden root when `TMPDIR` is unset `tools/agents/test_sqlite_baseline.py:122`
- [R] the lead-driven scenario test with `TMPDIR` unset leaves a stand-in forbidden temp root unchanged `bench/harness/test_lead_driven.py:109`
- [R] `AGENTS.md` tells agents and tools not to use host `/tmp` and names the worker launch temp paths `AGENTS.md:42`
- [R] the run-worker skill says the launcher sets `TMPDIR`, `TMUX_TMPDIR` and `TMPPREFIX` under `run_dir/tmp` `.claude/skills/run-worker/SKILL.md:25`
- [R] the practice note says the launcher gives workers temp vars under `RUN_DIR/tmp` and bans host `/tmp` `docs/practice/lead-supervisor-worker.md:35`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/22-report.md`: required hand-off report for this ticket.

## For later tickets

none

## Not checked

- I did not run `tools/agents/check.sh`, by instruction.
- I did not run cargo, by instruction.
- I did not launch real Codex or real traecli model workers; all launch and scenario coverage used fake/scripted participants, by instruction.

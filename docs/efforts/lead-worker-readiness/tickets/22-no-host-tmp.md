# 22 no-host-tmp: No tool, test or worker uses the host's /tmp

Type: feature
Status: resolved
Owner: worker-22
Blocked by:
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` on the record branch (`/data02/home/philip.yang/workspace/corenius-lwr-records`, read-only for you). Decision: `decisions/20261005-no-tmp.md` beside it.

On 2026-10-05 the host's `/tmp` (a tmpfs with about a million inodes) filled with leftover test directories and agent worktrees. Every new tmux server, shell here-documents, tests on the default temp directory and every sandboxed Codex command failed for all agents at once. The owner then banned `/tmp`: "let's ban the use of /tmp altogether and get the agents to use other dirs as temp dirs".

Make that true for everything this repository launches and tests. The rule is about the host's `/tmp`. A path under `/tmp` inside a capsule (the capsule mounts its own private tmpfs there, for example `/tmp/ticket`, `/tmp/solution`, `/tmp/run`) is not the host's `/tmp` and stays.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/lead_launch.py`, `tools/agents/land.py`, `tools/agents/wt.py`, `tools/agents/effort.py`, `tools/agents/acceptance.py`, `tools/agents/tracker.py`, their tests
- `tools/agents/sqlite_baseline.py`, `tools/agents/test_sqlite_baseline.py` (only the two places that hardcode `/tmp`; change nothing else there)
- `tools/agents/checks.d/` (one new script), `bench/harness/` (temp handling only)
- `AGENTS.md` (one rule), `.claude/skills/run-worker/SKILL.md`, `docs/practice/lead-supervisor-worker.md`

## Acceptance
- [x] Every worker launch, interactive or not, gives the worker a temp directory of its own inside the run directory and starts the runtime with `TMPDIR`, `TMUX_TMPDIR` and `TMPPREFIX` pointing into it; for a sandboxed runtime that directory is writable (a writable root for Codex); the run record names it; it is removed when the run is accounted as stopped unless a flag keeps it. A scripted participant that calls `tempfile.mkdtemp()`, starts a tmux server and runs a shell here-document creates nothing under the host's `/tmp` (the test compares a listing of a stand-in directory, not the real `/tmp`: make the temp root injectable).
- [x] The standing brief generated for a worker states the rule in one line and names the worker's temp directory.
- [x] No toolbox command creates anything under the host's `/tmp` when `TMPDIR` is unset: where a temp directory is needed and `TMPDIR` is unset, it uses a directory under the runs root (the same root `CORENIUS_RUNS` moves). The lander's and the harness's temporary worktrees and scenario repositories follow the same rule. A test runs the lander, a launch, an acceptance record and one scenario with `TMPDIR` unset and a stand-in "forbidden" directory and shows it stays empty.
- [x] `tools/agents/sqlite_baseline.py` and `tools/agents/test_sqlite_baseline.py` no longer hardcode `/tmp`; they use the same rule. If a short path is required there (a socket path limit, for example) say so in the report and use the shortest directory the rule allows; if the suite cannot be made to pass, leave those two files unchanged and record it under `## For later tickets` as `- to 14: ...` with the exact error.
- [x] A check script under `tools/agents/checks.d/` fails when a file under `tools/agents/`, `tools/insula/` or `bench/harness/` contains a host `/tmp` path literal outside a short allowlist kept in the script (the in-capsule mount destinations, with a comment for each entry); it passes on your branch.
- [x] `AGENTS.md` gains one rule under "Rules for working here": agents and tools do not use the host's `/tmp`; set `TMPDIR`, `TMUX_TMPDIR` and `TMPPREFIX` to a directory you own, and say where the launcher puts a worker's. The run-worker skill and the practice note say the same.
- [x] The report states what still touches the host's `/tmp` and cannot be changed from here (as observed on this host: the Codex sandbox creates mount points under `/tmp` when it starts a command), with the evidence.
- [x] Every existing scenario still matches its expectation; `tracker.py lint` is clean.

## Comments
- 2026-10-05T19:29:21Z worker-22: claimed until 2026-10-07T19:29:21Z
- 2026-10-05T23:46:38Z worker-22: resolved

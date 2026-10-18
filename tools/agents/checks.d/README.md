One script per feature: `NN-name.sh`, run from the repository root by
`tools/agents/check.sh`. Exit non-zero on failure. Register a suite once.
`check.py` selects and supervises suites; `05-gate-dispatch.sh` exercises command
shims and disposable registrations, so its tests never recursively run the gate.

The default gate includes SQLite and excludes the parked PostgreSQL/Temporal
integration. `CHECK_TEAM_CONTROL_PLANE=1` enables that integration and requires
Temporal and Docker. `CHECK_CARGO=0` skips only the root Cargo suite; feature
Cargo suites still run. `CHECK_BENCH=1` adds benchmark/capsule checks and
`CHECK_SYNC=1` adds sibling policy drift checks. Existing switches still apply
inside an explicitly selected suite (including the parked services switch).

```sh
python3 tools/temp_root.py -- tools/agents/check.sh --list
python3 tools/temp_root.py -- tools/agents/check.sh --suite 00-temp-paths --suite 05-gate-dispatch --timeout 60 --json .scratch/gate.json
python3 tools/temp_root.py -- env CHECK_TEAM_CONTROL_PLANE=1 CHECK_BENCH=1 CHECK_SYNC=1 tools/agents/check.sh
```

Repeated `--suite` selects only the named suites, in registration order. Names
are script stems plus `policy`, `bench-<name>`, `sync`, and `cargo`; `--list`
shows the commands and current default selection without executing checks.
Unknown suites and non-positive/non-finite timeouts fail before execution.

Every suite has a wall-clock limit (`--timeout`, default 1800 seconds). The native
controller suite takes more than fifteen minutes with two test threads on the
development host. `CHECK_CONTROLLER_THREADS` changes only that suite's thread
count (default 2). Failure
and timeout are distinct JSON statuses and both allow later suites to run.
SIGINT/SIGTERM stop dispatch, clean the active suite, and write a partial summary
when `--json PATH` (alias `--json-summary`) is provided. The JSON includes suite
commands, return codes, durations, the limit, and any unrun suites. Exit codes
are 0 for success, 1 for check failures/timeouts, 2 for invalid arguments, and
128 plus the signal number for interruption.

Each suite starts a process group. Cleanup sends TERM, waits up to 0.5 seconds,
then sends KILL to the whole group even if its leader already exited. This also
cleans background children after normal suite completion. A child can escape by
creating a new session or process group; this gate is not a security boundary
or a replacement for cgroup containment. SIGKILL of the gate cannot produce a
partial report. Suite output streams directly to the invoking terminal/log.

The temporary-root behavior suite lives in `00-temp-paths.sh`; the shared source
scanner runs once in `90-no-host-tmp.sh`. The temporary-root wrapper preserves
safe inherited roots within the socket budget. It reserves 48 of Linux's 107
Unix socket pathname bytes for nested names. If the default worktree path is too
long, it uses an owned `~/.ctmp` directory created with mode 0700. Unsafe
or overlong inherited roots fail with an actionable error; they are not silently
replaced. The Python API accepts safe long data roots unless a socket suffix budget is
requested. Callers with longer suffixes must budget their complete socket paths.
Sandbox temporary roots and mounts are unchanged.

# State: multi-agent-bench

<!-- effort:machine:start -->
```json
{
  "base_branch": "main",
  "base_commit": "5bbcc850a39cdefb3b8891915b944adae5dea88c",
  "dehydrated": "2026-10-07T09:40:30Z",
  "effort": "multi-agent-bench",
  "runs": [
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/author-c-20260929T232224Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/author-mh1-py-inventory-20260930T002234Z",
      "verdict": "DEGRADED"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/author-mh2-rs-scheduler-20260930T002234Z",
      "verdict": "DEGRADED"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/author-mh3-cxx-engine-20260930T002234Z",
      "verdict": "DEGRADED"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/author-pr-20260929T232224Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/author-t-20260929T232224Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/blade-author-1-20260929T232855Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/blade-author-2-20260929T232855Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/blade-author-3-20260929T234740Z",
      "verdict": "DEGRADED"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/capsule-smoke-20260930T000005Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/research-blade-20260929T222010Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/research-blade-cuda-20260929T230245Z",
      "verdict": "DEGRADED"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/research-insula-20260929T222011Z",
      "verdict": "DEGRADED"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/research-tasks-20260929T222011Z",
      "verdict": "DEGRADED"
    }
  ],
  "worktrees": []
}
```
<!-- effort:machine:end -->

## Where we are

Landed: M1 capsule builder, M2 harness and single-ticket tracks, M3 blade ladder L0-L5, M4 multi-ticket tasks and fleet runner, M5 live pilot.

- `tools/insula/capsule.py` (20 tests): layered bwrap capsules; requests in `bench/insula/requests/` (`toolchains.json`, `agent.json`).
- 41 validated tasks under `bench/tasks/` (34 single, 7 multi: m01-m03, b-l5-fused-op, mh1-py-inventory, mh2-rs-scheduler, mh3-cxx-engine).
  `python3 bench/harness/validate.py`: 41 ok, 0 failing (301 s).
- Harness: `bench/harness/{tasklib,validate,fleet,run,results}.py` and `pilot.sh`, `fleet_exp.sh`; 29 tests.
- `worker.py launch --capsule` runs a real traecli inside a capsule (live: capsule-smoke CLEAN in 59 s).
- Live results are in `bench/RESULTS.md` and `bench/results-archive/`: pilot 102/102 passed (too easy to discriminate); solo vs fleet on the
  small multi tasks: equal correctness, fleet not faster (startup and redo cost), about 5x tokens on b-l5-fused-op.
- tmux harness (landed 2026-09-30): `bench/tmuxh/bench-tmux run ...` = a window per run, a pane per agent, a curses dashboard in window 0, on the dedicated
  tmux socket `corenius-bench`; tests via `bench/tmuxh/check.sh`; design in decisions/20260930-tmux-harness.md. Live-tested with real traecli.
- The hard-task experiment finished (18/18) and is in `bench/RESULTS.md` section 4 and `bench/results-archive/fleet-hard.json`: mh1 and mh2 pass in both conditions, mh3-cxx-engine solo 1/3 and fleet 0/3.

## Next actions

1. Find out why mh3-cxx-engine fails (fleet and mostly solo): rerun a few with the saved verifier tail (fleet runs now write `verify-tail.txt`).
2. If everything still passes, the benchmark needs harder or larger tasks to separate correctness: more tickets, hidden cross-cutting
   invariants that fail a ticket-local reading, longer tasks. Author with workers (`author-mh*` runs are the pattern), validate with validate.py.
3. Cost in dollars is unmeasured; only tokens are. Add it if a price source is given.

## Open questions

- Is a docker-exported hermetic base layer wanted, or is host `/usr` (non-hermetic, labelled) acceptable for the benchmark?
- Should the Rust `src/insula.rs` runner and the Python capsule builder be unified? (Recorded follow-up, not started.)
- How many repeats per task for the first pilot? Sizing gave about 93 observations per arm for 0.5 vs 0.7 (an assumption).

## Traps

Things a fresh agent would get wrong.

- `.gitignore` has `*.log`: a task file ending in .log is silently not committed (t01's events.log was missed once). `bench/harness/test_tasks_tracked.py` guards it.
- The worktree `corenius-multi-agent-bench` (branch effort/multi-agent-bench) lags main after the tmux harness landed; run `git merge --ff-only main` there before using it, and do not
  change harness code in it while `bench/harness/fleet_exp.sh` is running from it.
- Never point libtmux at the default tmux server; the harness uses socket `corenius-bench` only.

- zsh does not word-split unquoted variables: pass bwrap/blade flag lists as arrays. This bit us four times.
- The capsule clears the environment on purpose (spack `LIBRARY_PATH` breaks blade's bundled gcc); pass env with `--env`.
- `/usr/local/cuda` is a symlink into `/home/...`; a tmpfs `/home` hides it. The capsule follows host symlinks; do not rebind `/home`.
- GPU needs `--gpu` (device nodes plus a read-only `/sys`); without it CUDA programs fail or segfault.
- Blade needs the pip-check marker files and `--toolchain=x86_64-gcc830 --skip-bundle`; use `blade-offline` inside the capsule, never `blade` on the host.
- Workers cannot use the GPU inside their own sandbox; GPU tasks are validated by the lead.
- Supervisor rules were loosened during this effort (rm by target, protected-head advance); see decisions/. Old verdicts are in `verdict.prev.json`.
- `~/.cargo` holds other agents' config directories; the capsule binds only `~/.cargo/bin` and `~/.rustup`.

## Do not redo

- Blade and capsule probes are recorded in decisions/ and evidence/; the CUDA-in-blade question is solved.
- The six worker-authored terminal, twelve python/rust/c++ and six blade L0-L2 tasks are validated and committed.

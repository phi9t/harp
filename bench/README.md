# corenius-bench

A benchmark for multi-agent work on real tasks. Tasks are small repos with real verifiers: terminal work, Python (uv), Rust
(cargo), C++ (cmake + ninja) and blade (+ CUDA) builds. The same task runs solo or as a lead/worker fleet, and every step that
executes task code runs inside an insula capsule.

## Layout

```
bench/
  tasks/<id>/            task.toml, instruction.md, environment/ (agent's workspace, incl. visible tests),
                         tests/ (verifier-owned, restored before grading), solution/ (never mounted for the agent),
                         tickets/<id>/ (multi tasks only)
  insula/requests/       capsule requests: toolchains.json (offline task capsule), agent.json (adds traecli, network)
  insula/layers/         small tree layers (blade-offline wrapper)
  harness/               tasklib, validate, fleet, run, results, pilot.sh (+ tests)
  results/               run output (gitignored)
tools/insula/capsule.py  layered capsule builder and runner
```

## Commands

```sh
python3 bench/harness/validate.py [--task ID]...          # offline gate: is the benchmark itself sound?
python3 bench/harness/run.py ID... --agent reference      # or noop | cmd:SHELL | traecli[:MODEL]
python3 bench/harness/run.py ID --condition fleet:3 --agent reference     # multi tasks only
python3 bench/harness/run.py m01-disjoint-utils --condition controller:t1 --agent reference
python3 bench/harness/run.py m01-disjoint-utils --condition controller:t2 --agent reference
python3 bench/harness/run.py m01-disjoint-utils --condition controller:complete --agent reference
python3 bench/harness/run.py ID --agent traecli --authorize-model-spend   # a real model; spends budget
python3 bench/harness/run.py --report bench/results/<dir>                 # pass rate with 95% Wilson interval
bench/harness/pilot.sh OUT [REPEAT] [PARALLEL]                            # live pilot over all single tasks
```

`validate.py` runs, for every task and inside the capsule: the start state fails; the reference solution applies and passes; the
verdict is deterministic; deleted or stubbed tests do not help; an empty patch fails; tampering with protected source files is
undone before grading. Multi tasks add: every ticket is necessary, and every landing order ends in a pass with the declared
number of conflicts.

`controller:<ticket>` is the deterministic controller-backed mode for the first controller dogfood slice. It accepts
`m01-disjoint-utils` tickets `t1` and `t2`, plus `controller:complete` for the complete two-ticket m01 path. It runs no model
calls and rejects unsupported tasks or dependency graphs before creating the requested output directory. Single-ticket runs
create fresh disposable Git/Kata/controller state, record a noop negative partial-verifier control and a fixed-ticket positive
control, then accept and land the exact candidate through authenticated SQLite-controller commands. Complete mode admits both
initial workers from the same canonical base, records useful-work intervals bounded by task file writes, contract checks and
commits, lands the first candidate, retains the stale-base refusal for the second candidate at conditional landing, releases
the initial attempts, and admits at most one fresh integration attempt from the current canonical base. The fresh integration
candidate receives fresh complete-suite verification and acceptance; the original stale proof is recorded as rejected for the
new candidate/base. The final committed canonical tree and omission/tamper controls are independently graded from exported
exact commits with safe archive-member validation, restored unchanged m01 tests, and the private pinned verifier runtime
snapshot.

The benchmark `result.json` is written under `--out`; the retained live controller artifacts are in `result.controller.run_root`
under a short `cb-*` directory below the repository temporary root selected by `tools/temp_root.py`, because Unix-domain
supervisor sockets must stay below `SUN_LEN`. Host and controller runs must use an owned temporary root outside `/tmp`; this
worktree uses `/data02/home/philip.yang/.cbt`.

Controller mode requires the pinned Kata binary
`~/.local/share/corenius/pinned-tools/kata/8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6/kata-pinned` with SHA-256
`8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6`, plus the Rust controller binaries buildable offline with
`cargo build --manifest-path crates/sqlite-controller/Cargo.toml --target-dir .scratch/controller-backed-bench/target --locked
--offline --bins`. The controller record pins the verifier adapter script, protected test file, configured executables and
policy/config digests. Git-backed verification uses a private verifier-only runtime snapshot mounted read-only at `/usr`, and
the controller validates that snapshot's manifest before dispatch and before trusted-proof acceptance. The benchmark-built
snapshot includes the pinned Python executable at the exact resolved policy path, its stdlib, extension modules, dynamic
loader and shared-library dependencies; the snapshot manifest digest is bound into the verifier environment/proof and retained
in controller metadata. Independent verifier logs record that same configured Python executable in `argv` instead of a
synthetic host path.

The pinned artifact stays outside disposable source worktrees. Set `CORENIUS_KATA_PINNED_BINARY` to select another copy for
controller mode or the retained SQLite entry point; the native Kata fixture uses `KATA_PINNED_BINARY`. Each supplied copy
must retain the approved SHA-256. These overrides do not authorize model calls or live qualification.

## Watching runs in tmux

```sh
bench/tmuxh/bench-tmux run m03-same-file-append --condition fleet:2 --agent reference --attach
bench/tmuxh/bench-tmux run p01-slugger t01-log-window --agent traecli --authorize-model-spend --slots 4
bench/tmuxh/bench-tmux ls | attach | down          # the dedicated tmux server (socket `corenius-bench`); your own tmux is untouched
python3 bench/tmuxh/dashboard.py --root bench/results/tmux-<stamp> --once      # plain-text dashboard
```

One window per run and one pane per agent (a fleet run also gets a lead pane that shows landing, conflicts and redos); window 0 is a
curses dashboard (phase, time, tokens, commands, last event, verdict; `j/k` select, `Enter` jump, `l` log tail, `x` kill pane, `q` quit).
The scheduler is the lead and runs the same `run_solo` / `run_fleet_condition` code as `run.py`, so results are identical records;
panes and the dashboard talk to it only through state files (`runs/*/state.json`, `units/*/state.json`). `bench/tmuxh/check.sh` runs the
tmux tests (libtmux is installed once through the uv project in `bench/tmuxh`).

## Isolation

`tools/insula/capsule.py` builds a tmpfs root with the host `/usr` read-only, tool layers at fixed paths, the workspace at
`/workspace`, a cleared environment (an allowlist), a private `/proc`, and `--unshare-net` unless the policy says otherwise.
Verifiers and reference solutions always run offline. Only the model agent gets the network, and its capsule carries traecli's
config as read-only files bound into a tmpfs home; the host home is not visible. `--gpu` binds `/dev/nvidia*` and `/sys`.
Layers are content addressed; host-provided layers (`/usr`, rust, blade, cuda, traecli) are labelled non-hermetic in every
result. Workers run with `tools/agents/worker.py launch --capsule ...`.

## Tasks (see `python3 bench/harness/validate.py` for the live list)

Tracks: T terminal, P python, R rust, C c++, M multi-ticket, B blade (levels L0-L5 modelled on how lgtorch and erdos use blade:
flat targets, pinned versioned third-party labels, macros and multi-package graphs, env-driven configuration and settlers,
codegen and wrapper macros, composite). Blade tasks run real blade 1.2.2 offline. `b-l4-proto-codegen` uses a `gen_rule`
fallback because real `proto_library` does not link against the host protobuf here (see its `fidelity` metadata).

## Known limits

- Build files (`CMakeLists.txt`, `Cargo.toml`, blade BUILD files) are editable by the agent; an agent could weaken a sanitizer flag
  or a test target. Test sources and, for blade tasks, the source files the fix does not touch are restored before grading.
- The capsule base is the host `/usr` (Debian 12), not a hermetic image.
- Enforcement of "the worker changed only allowed paths" is a post-hoc check (git diff), not prevention.
- Results depend on one model configuration and a small number of repeats; `RESULTS.md` states what was measured and what was not.

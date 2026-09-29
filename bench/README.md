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
python3 bench/harness/run.py ID --agent traecli --authorize-model-spend   # a real model; spends budget
python3 bench/harness/run.py --report bench/results/<dir>                 # pass rate with 95% Wilson interval
bench/harness/pilot.sh OUT [REPEAT] [PARALLEL]                            # live pilot over all single tasks
```

`validate.py` runs, for every task and inside the capsule: the start state fails; the reference solution applies and passes; the
verdict is deterministic; deleted or stubbed tests do not help; an empty patch fails; tampering with protected source files is
undone before grading. Multi tasks add: every ticket is necessary, and every landing order ends in a pass with the declared
number of conflicts.

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

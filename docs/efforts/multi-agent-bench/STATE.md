# State: multi-agent-bench

<!-- effort:machine:start -->
<!-- effort:machine:end -->

## Where we are

Milestones (see EFFORT.md): M1 capsule builder done; M2 harness plus T/P/R/C tracks done; M3 blade ladder half done; M4 partly done.

- Capsule builder `tools/insula/capsule.py` (20 tests): layered bwrap capsules from request files in `bench/insula/requests/`
  (`toolchains.json`, `agent.json`). Real uv, cargo, cmake, ninja, gcc, real blade and CUDA all run inside it, offline; a blade-built
  sm_100 kernel runs on the B200s with `--gpu`.
- Task format and validator: `bench/harness/{tasklib,validate}.py`. 33 tasks under `bench/tasks/` (T 8, P 6, R 5, C 5, M 3 multi,
  B 6 blade L0-L2), each passing 7 checks (8 with protected sources, 9 for multi tasks) inside the capsule.
- Fleet runner `bench/harness/fleet.py` and experiment runner `bench/harness/run.py` with agents reference, noop, cmd, traecli
  (traecli refuses to run without `--authorize-model-spend`). Result records `corenius.bench.result.v1` and aggregation in `results.py`.
- `tools/agents/worker.py launch --capsule REF` runs a real traecli worker inside a capsule (tested with a fake traecli only).
- Workers used so far: research-blade, research-insula, research-tasks, research-blade-cuda, author-t, author-pr, author-c,
  blade-author-1 (all landed after lead verification); blade-author-2 (blade L3-L4 incl. proto) still running when this was written.

Nothing here has been run against a real model as an agent under test. No pass-rate or solo-versus-fleet claim exists.

## Next actions

1. Collect blade-author-2 (tasks b-l3-settler-conflict, b-l3-env-matrix, b-l4-wrapper-macro, b-l4-proto-codegen): check its verdict, run
   `python3 bench/harness/validate.py --task ...`, review the verifiers, then commit.
2. Author the L5 composite multi-ticket blade task (proto field + CUDA kernel + python wrapper), using the vendored protobuf package
   from b-l4-proto-codegen, with tickets/ and the multi checks.
3. Only on the owner's request: a live pilot `python3 bench/harness/run.py --all --agent traecli --authorize-model-spend`, starting with
   one or two tasks. Land by fast-forward; push nothing.

## Open questions

- Is a docker-exported hermetic base layer wanted, or is host `/usr` (non-hermetic, labelled) acceptable for the benchmark?
- Should the Rust `src/insula.rs` runner and the Python capsule builder be unified? (Recorded follow-up, not started.)
- How many repeats per task for the first pilot? Sizing gave about 93 observations per arm for 0.5 vs 0.7 (an assumption).

## Traps

Things a fresh agent would get wrong.

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

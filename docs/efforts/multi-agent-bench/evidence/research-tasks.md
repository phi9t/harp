- [R] Harbor's durable task shape is `instruction.md`, `task.toml`, `environment/`, `solution/`, and `tests/`; keep that authoring shape, but map Docker runtime behavior to bwrap capsule mounts (`/workspace`, `/tests`, `/solution`, `/logs/verifier`) instead of Docker images. Evidence: Harbor `docs/content/docs/tasks/index.mdx:24-43`, `:443-484`.
- [R] Harbor already separates agent timeout, verifier timeout, network policy, resources, and reward parsing; keep these fields because they map cleanly to capsule policy, timeout wrappers, and result schema. Evidence: Harbor `src/harbor/models/task/config.py:37-43`, `:340-345`, `:422-461`, `:562-590`, `:801-827`.
- [R] Harbor and Terminal-Bench rely on verifier-owned tests copied at runtime, not tests baked into the agent image; this is the key anti-cheating rule to port. Evidence: Harbor `docs/content/docs/tasks/task-difference.mdx:6-7`, `src/harbor/verifier/verifier.py:124-230`.
- [R] Terminal-Bench 2.0 is present on disk only as Harbor registry metadata and conversion/adaptation code; the task repository contents were not locally available in this investigation. Evidence: Harbor `registry.json:94012-94140`, `src/harbor/mappers/terminal_bench.py:48-77`.
- [I] Our no-network bwrap benchmark should exclude online LLM-judge tasks from the core T/P/R/C tracks, or replace them with deterministic offline judges, because Harbor's judge examples depend on provider keys and network calls. Evidence: Harbor `examples/tasks/llm-judge-example/task.toml:30-32`, `examples/tasks/llm-judge-example/tests/llm_judge.py:30-58`, `docs/content/docs/rewardkit/judge-criteria.mdx:98-110`.
- [X] I built six small offline red/green task sketches under `docs/efforts/multi-agent-bench/evidence/research-tasks/verified-tasks/` and verified each start state fails and each fixed state passes under `bwrap --unshare-net`.
- [X] I prototyped a 40-line pristine-test restore verifier at `docs/efforts/multi-agent-bench/evidence/research-tasks/prototypes/pristine_verify.py` and demonstrated that deleting task-local tests does not evade verification.
- [I] The 24-task T/P/R/C catalog below should favor behavior checks over textual checks, and every uv/cargo task must either use only stdlib/no crates or carry vendored/cache inputs inside the capsule.
- [I] Multi-agent tasks should deliberately exercise Corenius scarcity axes S1 source text, S2 landing ref, S3 gate capacity, S4 attention, and S5 shared mutable state; otherwise solo-vs-fleet differences will mostly be noise.
- [X] A two-proportion sizing calculation for pass/fail with `p_solo=0.50`, `p_fleet=0.70`, `alpha=0.05`, and `power=0.80` gives about 93 observations per arm, so `24 tasks x 4 repeats = 96` is a plausible first pilot only under independence assumptions.

1. Source base and constraints

- [X] I read Harbor at commit `f761ebdc18c3fd71dc10b581c1408946946ca553` with commands of the form `git -C /data02/home/philip.yang/workspace/harbor show f761ebdc18c3fd71dc10b581c1408946946ca553:<path> | nl -ba | sed -n '<range>p'`; `git -C /data02/home/philip.yang/workspace/harbor rev-parse HEAD` printed `f761ebdc18c3fd71dc10b581c1408946946ca553`.
- [X] I read Corenius collaboration evidence at commit `b8a3742a100ed952161f7ee81b6fab3d59a856a3`; `git -C /data02/home/philip.yang/workspace/corenius rev-parse HEAD` printed `b8a3742a100ed952161f7ee81b6fab3d59a856a3`.
- [R] The approved plan says all task-code execution runs inside an insula bwrap boundary, tasks live in a Harbor-like layout under `bench/tasks/<id>/`, and verification uses pristine tests inside insula. Evidence: `/data02/home/philip.yang/.claude/plans/tidy-questing-alpaca.md:7-10`, `:121-129`, `:195-214`.
- [R] The effort record says the benchmark measures solo versus lead/worker fleet work, records pass rate, wall time, conflicts, and contamination, and includes uv, cargo, cmake+ninja, and blade+CUDA tracks. Evidence: `docs/efforts/multi-agent-bench/EFFORT.md:7-18`.
- [R] The earlier decision file still says `cpp3rdlib` deps were untested, but the later decision corrects it: versioned labels such as `cpp3rdlib/x:ver@//cpp3rdlib/x:x` should be kept because offline vendored packages under `BLADE_ROOT/cpp3rdlib/<x>/` resolve. Evidence: `docs/efforts/multi-agent-bench/decisions/20260929-probe-findings-blade-and-insula.md:34-36`, `docs/efforts/multi-agent-bench/decisions/20260929-blade-3rdlib-resolves-from-local-packages.md:6-19`.

2. Harbor task layout, schema, results, and what to port

- [R] Harbor presents itself as a framework from the Terminal-Bench creators for evaluating agents, building benchmarks, running environments in parallel, and generating RL rollouts. Evidence: Harbor `README.md:10-15`.
- [R] Harbor says it is the official harness for Terminal-Bench 2.0 and that the local example run uses Docker. Evidence: Harbor `README.md:30-41`, `docs/content/docs/tutorials/running-terminal-bench.mdx:7-13`.
- [R] The Harbor task tree is exactly `instruction.md`, `task.toml`, `environment/`, `solution/`, and `tests/`. Evidence: Harbor `docs/content/docs/tasks/index.mdx:24-43`.
- [R] `task.toml` carries `schema_version`, `[task]` metadata, arbitrary `[metadata]`, `[verifier]`, `[agent]`, `[solution]`, `[environment]`, optional MCP servers, and optional healthcheck fields. Evidence: Harbor `docs/content/docs/tasks/index.mdx:71-131`.
- [R] Harbor network policy has modes `no-network`, `public`, and `allowlist`, with baseline environment policy and optional phase overrides for agent and verifier. Evidence: Harbor `docs/content/docs/tasks/index.mdx:133-151`, `src/harbor/models/task/config.py:37-43`.
- [R] Harbor agent config includes `timeout_sec` and an optional user; verifier config includes `timeout_sec`, `env`, optional user, and `environment_mode` of shared or separate. Evidence: Harbor `src/harbor/models/task/config.py:340-345`, `:555-590`.
- [R] Harbor environment config includes build timeout, optional Docker image, OS, CPUs, memory, storage, GPUs, GPU types, TPU, MCP servers, environment variables, skills dir, healthcheck, and workdir. Evidence: Harbor `src/harbor/models/task/config.py:422-470`.
- [R] Harbor supports multi-step tasks with per-step `agent`, `verifier`, `min_reward`, healthcheck, and artifact collection, plus a task-level multi-step reward strategy of `mean` or `final`. Evidence: Harbor `src/harbor/models/task/config.py:760-827`.
- [R] Harbor environment special paths are `/logs/agent`, `/logs/verifier`, `/logs/artifacts`, `/tests`, and `/solution`; `/tests` is copied by the verifier after the agent runs, and `/solution` is copied only by the Oracle agent. Evidence: Harbor `src/harbor/models/trial/paths.py:12-49`, `docs/content/docs/tasks/index.mdx:449-470`.
- [R] Harbor tests must write `/logs/verifier/reward.json` or `/logs/verifier/reward.txt`, and Harbor reads JSON first then falls back to text. Evidence: Harbor `docs/content/docs/tasks/index.mdx:474-484`.
- [R] Harbor rejects empty, non-numeric, or non-finite reward outputs. Evidence: Harbor `src/harbor/verifier/verifier.py:67-122`.
- [R] Harbor trial outputs include `config.json`, `lock.json`, `agent/`, `artifacts/`, `verifier/`, `test-stdout.txt`, `test-stderr.txt`, `reward.txt`, `reward.json`, and `result.json`. Evidence: Harbor `src/harbor/models/trial/paths.py:190-291`.
- [R] Harbor `TrialResult` records task name, trial name, trial URI, task ID, source, task checksum, config, agent info, agent result, verifier result, verifier environment mode, exception info, timing fields, and optional step results. Evidence: Harbor `src/harbor/models/trial/result.py:70-97`.
- [R] Harbor `JobStats` records completed, errored, running, pending, cancelled, retries, per-eval stats, token totals, and cost; job aggregation records reward values per trial and exception stats. Evidence: Harbor `src/harbor/models/job/result.py:28-42`, `:129-169`, `:236-243`.
- [R] Harbor writes job config, job lock, job result, and then final job stats with metrics and pass-at-k after trials complete. Evidence: Harbor `src/harbor/job.py:953-1084`.
- [R] Harbor's `hello-world` example is deterministic in task config and tests file contents in `/app/hello.txt`, but its `tests/test.sh` installs curl and uv over the network. Evidence: Harbor `examples/tasks/hello-world/task.toml:16-27`, `examples/tasks/hello-world/tests/test.sh:1-18`, `examples/tasks/hello-world/tests/test_state.py:7-21`.
- [R] Harbor's `hello-cuda` example declares `gpus = 1` and acceptable GPU types `A100` and `H100`. Evidence: Harbor `examples/tasks/hello-cuda/task.toml:22-28`.
- [R] Harbor's simple multi-step example declares two steps, `create-file` and `append-content`, each with agent and verifier timeouts. Evidence: Harbor `examples/tasks/hello-multi-step-simple/task.toml:15-31`.
- [R] Harbor's LLM judge example passes `ANTHROPIC_API_KEY` and model name in verifier env and writes a JSON reward from an Anthropic response. Evidence: Harbor `examples/tasks/llm-judge-example/task.toml:30-32`, `examples/tasks/llm-judge-example/tests/llm_judge.py:30-58`.
- [R] Harbor's sidecar-artifacts example uses a separate verifier and collects sidecar evidence after the agent finishes; the test checks sidecar logs and writes `reward.txt`. Evidence: Harbor `examples/tasks/sidecar-artifacts/task.toml:21-31`, `examples/tasks/sidecar-artifacts/tests/test.sh:21-42`.
- [R] Harbor's network-policy matrix example can switch agent and verifier network modes per step. Evidence: Harbor `examples/tasks/network-policy-matrix/dynamic/steps-mixed/task.toml:29-51`.
- [R] Harbor's AIME adapter test is deterministic: read `/app/answer.txt`, compare it to an expected answer, and write `reward.txt`. Evidence: Harbor `adapters/aime/src/aime/task-template/tests/test_outputs.py:4-30`, `adapters/aime/src/aime/task-template/tests/test.sh:18-25`.
- [R] Harbor's AA-LCR adapter uses an OpenAI-compatible LLM judge and logs question, official answer, predicted answer, grade, and reward. Evidence: Harbor `adapters/aa-lcr/template/tests/llm_judge.py:1-74`, `:81-120`.
- [I] Keep compatible: file layout, TOML task metadata, agent/verifier timeouts, reward file contract, trial/job result split, artifact collection naming, and deterministic tests copied at verifier time.
- [I] Change for bwrap capsules: replace Dockerfile/build-image assumptions with `capsule` or `environment.capsule_request`, translate `network_mode = "no-network"` to `bwrap --unshare-net`, translate CPU/memory/storage to wrapper limits or recorded requested limits, mount `/tests` from a pristine verifier copy after agent completion, omit `.git` and `solution/` from agent mounts, and record capsule digest because there is no Docker image digest.

3. Terminal-Bench evidence from disk only

- [R] Harbor's migration docs say Terminal-Bench used `docker-compose.yaml`, `task.yaml`, `solution.sh`, `run-tests.sh`, and `tests/test_outputs.py`, while Harbor uses `instruction.md`, `task.toml`, `environment/Dockerfile`, `solution/solve.sh`, and `tests/test.sh`. Evidence: Harbor `docs/content/docs/migration.mdx:14-45`, `docs/content/docs/datasets/adapters.mdx:697-713`.
- [R] Harbor says Terminal-Bench had instruction text nested in `task.yaml`, root-level Docker context, and parser-driven stdout-to-reward logic; Harbor moved instruction to Markdown, environment files to `environment/`, and reward writing into the task tests. Evidence: Harbor `docs/content/docs/tasks/task-difference.mdx:12-18`, `:48-64`.
- [R] Harbor's Terminal-Bench mapper accepts `instruction`, `author_name`, `author_email`, `difficulty`, `category`, `tags`, duration estimates, `max_agent_timeout_sec`, and `max_test_timeout_sec`, then maps max timeouts into Harbor agent/verifier timeout fields. Evidence: Harbor `src/harbor/mappers/terminal_bench.py:48-77`, `tests/unit/mappers/test_terminal_bench.py:17-76`.
- [R] The Terminal-Bench mapper appends a reward logging suffix to converted tests that writes `1` or `0` to `/logs/verifier/reward.txt` based on exit code. Evidence: Harbor `src/harbor/mappers/terminal_bench.py:37-45`, `:357-361`.
- [R] The Terminal-Bench mapper has anti-cheating migration logic for Dockerfiles that reference tests or solution files, copying them into Harbor's environment/solution handling instead of leaving the old root-level shape untouched. Evidence: Harbor `src/harbor/mappers/terminal_bench.py:319-361`.
- [R] The mapper unit tests say simple single-service docker-compose files can collapse to Dockerfile, but multi-container services, custom networks, and real volumes cannot. Evidence: Harbor `tests/unit/mappers/test_terminal_bench.py:82-110`.
- [R] Harbor `registry.json` contains a `terminal-bench` dataset at version `2.0` pointing to `https://github.com/laude-institute/terminal-bench-2.git` at commit `69671fbaac6d67a7ef0dfec016cc38a64ef7a77c`, with task names including `adaptive-rejection-sampler`, `build-cython-ext`, `compile-compcert`, `db-wal-recovery`, and `dna-assembly`. Evidence: Harbor `registry.json:94012-94140`.
- [I] Task categories visible from the registry names include build/compile repair, data processing, scientific/statistical programming, web/server configuration, async/concurrency, legacy-language modernization, database/storage recovery, algorithms/puzzles, security/recovery, ML/data work, and image-to-code tasks; this is an inferred taxonomy from names, not official metadata, because the Terminal-Bench 2 task repository contents were not present locally.
- [R] Harbor's RewardKit docs include deterministic programmatic criteria for files, commands, JSON, CSV, HTTP, images, and trajectories, and also LLM/agent judges configured via TOML. Evidence: Harbor `docs/content/docs/rewardkit/index.mdx:87-104`, `:170-240`, `docs/content/docs/rewardkit/judge-criteria.mdx:9-56`.
- [R] Harbor CHANGELOG and docs mention future or later Terminal-Bench work only lightly: the roadmap says "Develop new agent benchmarks like Terminal-Bench 3.0", and the README/docs focus on Terminal-Bench 2.0. Evidence: Harbor `docs/content/docs/contributing/roadmap.mdx:8-12`, `README.md:30-41`.
- [X] Command used to check for later-version references: `git -C /data02/home/philip.yang/workspace/harbor grep -n -i "terminal-bench\\|t-bench\\|Terminal Bench\\|2.0\\|3.0\\|4.0" f761ebdc18c3fd71dc10b581c1408946946ca553 -- README.md CHANGELOG.md docs/content/docs src tests registry.json`; output included `docs/content/docs/contributing/roadmap.mdx:11:- [ ] **Build new benchmarks** -- Develop new agent benchmarks like Terminal-Bench 3.0.` and no local Terminal-Bench 3.0 or 4.0 task format.
- [I] For this benchmark, Terminal-Bench compatibility should mean compatible authoring conventions and reward semantics, not running through Terminal-Bench's Docker-compose path.

4. Verifier design and 24 task catalog

- [X] Local toolchain versions available for the prototype sweep were: `uv 0.12.17 (x86_64-unknown-linux-gnu)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`, `rustc 1.95.0 (59807616e 2026-04-14)`, `cmake version 3.25.1`, `ninja 1.11.1`, and `bubblewrap 0.8.0`.
- [I] Every task below should store a reference solution under `solution/solve.sh`, a verifier-owned `tests/test.sh`, and a `task.toml` with `single` or `multi` metadata; the harness should verify that the start tree fails and the fixed tree passes before accepting the task.
- [I] For uv tasks, use stdlib-only tasks or include `uv.lock` plus a pre-populated read-only wheel/cache layer; run `uv run --offline ...` with `UV_CACHE_DIR` bound to the capsule cache. For cargo tasks, use std-only crates or include `vendor/` plus `.cargo/config.toml` source replacement; run `cargo test --offline`. For cmake+ninja tasks, vendor any third-party headers/sources inside the task and run `cmake -S . -B build -G Ninja && cmake --build build && ctest --test-dir build`.

| id | level | starting broken state | reference fix | verifier command | offline dep plan | expected wall time | single/multi |
|---|---|---|---|---|---|---|---|
| t01-log-window | S | [X] `metrics.sh` counts the wrong inclusive time window. | [X] Fix awk predicate for events 3..4. | `bash tests/test.sh` | shell/coreutils only | [X] fixed measured 0.018 s | single |
| t02-quoting-archive | S | [X] `safe_pack.sh` fails on filenames with spaces by unquoted manifest expansion. | [X] Read manifest safely and pass `tar --files-from`. | `bash tests/test.sh` | shell/tar only | [X] fixed measured 0.021 s | single |
| t03-jsonl-rollup | S | [I] Pipeline sums all records instead of filtering by service and status. | [I] Parse JSONL with Python stdlib and assert exact counts. | `python3 tests/test_rollup.py` | Python stdlib only | [I] < 1 s | single |
| t04-permission-repair | S | [I] Startup script creates unreadable runtime dir and loses sticky mode. | [I] Set owner-independent mode and verify command behavior as unprivileged user. | `bash tests/test.sh` | shell/stat only | [I] < 1 s | single |
| t05-git-conflict-marker-cleanup | M | [I] A generated config contains conflict markers and incompatible choices. | [I] Resolve config so parser and behavior test pass. | `python3 tests/test_config.py` | Python stdlib only; no `.git` mounted to agent | [I] < 1 s | single |
| t06-process-lock | M | [I] CLI starts two workers that both write a lock file. | [I] Add atomic `mkdir`/`flock` style locking and cleanup. | `bash tests/test_race.sh` | shell/coreutils only | [I] 2-5 s | single |
| t07-csv-join-pipeline | M | [I] Join script drops duplicate keys and mishandles quoted commas. | [I] Use Python `csv` module and deterministic sort. | `python3 tests/test_join.py` | Python stdlib only | [I] < 1 s | single |
| t08-log-rotation-recovery | M | [I] Rotation script overwrites the current log and miscomputes retention. | [I] Preserve current file and enforce retention by mtime/name tests. | `bash tests/test_rotate.sh` | shell/coreutils only | [I] < 2 s | multi-ticket |
| p01-slugger | S | [X] Slugifier leaves repeated separators and returns empty slug for blank title. | [X] Collapse separators and default to `untitled`. | `uv run --offline python -m unittest discover -s tests` | stdlib only, checked-in `uv.lock`; writable `UV_CACHE_DIR` | [X] fixed measured 0.064 s | single |
| p02-invoice-cents | S | [X] Decimal cents calculation rounds `100.5` down to `100`. | [X] Use `Decimal` quantization and half-up behavior. | `uv run --offline python -m unittest discover -s tests` | stdlib only, checked-in `uv.lock`; writable `UV_CACHE_DIR` | [X] fixed measured 0.065 s | single |
| p03-cli-subcommand | M | [I] CLI ignores `--format json` and always prints text. | [I] Add subcommand parser branch and JSON output test. | `uv run --offline python -m unittest discover -s tests` | stdlib only; no package install | [I] < 1 s | single |
| p04-cache-key-normalization | M | [I] Cache key includes absolute temp path, making results non-reproducible. | [I] Normalize paths relative to workspace and hash content. | `uv run --offline python -m unittest discover -s tests` | stdlib only; fixture files vendored | [I] < 1 s | single |
| p05-offline-wheel-conflict | M | [I] Project depends on two local wheels with incompatible transitive pins. | [I] Point `pyproject.toml` to a compatible local wheel set and lock it. | `uv sync --offline && uv run --offline pytest` | `environment/wheels/` and `uv.lock` in capsule | [I] 5-15 s | single |
| p06-typing-contract | M | [I] Public API returns `dict[str, Any]` and breaks a typed caller. | [I] Add dataclass/protocol boundary and behavior tests. | `uv run --offline python -m unittest discover -s tests` | stdlib only or vendored mypy if type checker is used | [I] < 2 s | multi-ticket |
| r01-countdown-division | S | [X] Rust solver permits non-exact division or misses exact-division rejection. | [X] Return `None` unless division is exact and positive. | `cargo test --offline` | std-only crate, checked-in `Cargo.lock` | [X] fixed measured 0.148 s | single |
| r02-borrowed-index | S | [I] Parser returns borrowed slices tied to a temporary buffer. | [I] Store owned strings or restructure lifetimes. | `cargo test --offline` | std-only crate | [I] < 1 s | single |
| r03-trait-format | M | [I] Trait impl formats IDs differently in `Display` and serializer helper. | [I] Centralize canonical formatter and assert round trip. | `cargo test --offline` | std-only crate | [I] < 1 s | single |
| r04-workspace-feature | M | [I] Workspace feature is enabled in one crate but not propagated to a dependent crate. | [I] Fix `[features]` propagation and integration test. | `cargo test --workspace --offline` | local workspace crates only | [I] 1-3 s | multi-ticket |
| r05-channel-shutdown | L | [I] Worker pool can deadlock when sender drops after first error. | [I] Add explicit shutdown path and join semantics. | `cargo test --offline -- --test-threads=1` | std-only crate; deterministic timeouts | [I] 2-5 s | multi-ticket |
| c01-window-mean | S | [X] C++ moving average uses an off-by-one denominator. | [X] Track window count correctly and assert expected means. | `cmake -S . -B build -G Ninja && cmake --build build && ctest --test-dir build --output-on-failure` | no external deps | [X] fixed measured 0.392 s | single |
| c02-link-order | M | [I] Static libraries link in the wrong order, causing unresolved symbols. | [I] Fix `target_link_libraries` dependency direction. | same cmake+ninja+ctest command | no external deps | [I] < 2 s | single |
| c03-sanitizer-ub | M | [I] Vector code reads one element past end but ordinary tests may pass. | [I] Fix loop bound and run ASAN-enabled verifier build. | `cmake -S . -B build -G Ninja -DENABLE_ASAN=ON && cmake --build build && ctest --test-dir build` | compiler runtime from capsule | [I] 2-8 s | single |
| c04-header-refactor | M | [I] Header-only helper exposes inline global state and violates ODR. | [I] Move state to `.cc` and export a clean header. | cmake+ninja+ctest | no external deps | [I] < 2 s | multi-ticket |
| c05-generated-table | L | [I] Generated lookup table is stale after schema change. | [I] Regenerate from vendored generator input and test values. | `python3 tools/gen.py && cmake ... && ctest ...` | Python stdlib generator vendored | [I] 2-5 s | multi-ticket |

- [X] Verification sweep command shape for the six constructed tasks was `bwrap --ro-bind / / --dev-bind /dev /dev --bind <task-copy> <task-copy> --tmpfs /tmp --unshare-net --clearenv --setenv HOME <task-copy>/.home --setenv PATH /data02/home/philip.yang/.local/bin:/root/.cargo/bin:/usr/local/bin:/usr/bin:/bin --chdir <task-copy> bash -c '<verifier command>'`.
- [X] `t01-log-window start` failed and `fixed` passed: `elapsed=0.018 status=1` for start, `elapsed=0.018 status=0` for fixed.
- [X] `t02-quoting-archive start` failed because `tar` treated the unquoted spaced path as separate files: `tar: input/a: Cannot stat`, `tar: b.txt: Cannot stat`, `status=2`, `elapsed=0.019`; fixed passed with `status=0`, `elapsed=0.021`.
- [X] `p01-slugger start` failed under `uv run --offline` with two unittest failures: expected `q3-core-api-audit` but got `q3-core-api----audit`, and expected `untitled` but got empty; `elapsed=0.100 status=1`; fixed passed with `elapsed=0.064 status=0`.
- [X] `p02-invoice-cents start` failed with expected `101` but got `100`; `elapsed=0.064 status=1`; fixed passed with `elapsed=0.065 status=0`.
- [X] `r01-countdown-division start` failed under `cargo test --offline` with `left: Some(3) right: None`; `elapsed=0.145 status=101`; fixed passed with `elapsed=0.148 status=0`.
- [X] `c01-window-mean start` failed in `window_test.cpp:9`; `elapsed=0.433 status=8`; fixed passed with `elapsed=0.392 status=0`.

5. Anti-cheating and verifier hardening

| cheat | bwrap/capsule hardening mechanism |
|---|---|
| [I] Edit or delete tests before verifier runs. | [X] Do not mount verifier tests during agent phase; at verifier phase copy pristine tests into the workspace and then run the command. Prototype below demonstrates this. |
| [I] Replace `tests` with a symlink to benign files. | [R] `pristine_verify.py` unlinks a symlink or removes any existing tests path before copying and rejects symlinks under the copied tests tree. Evidence: `docs/efforts/multi-agent-bench/evidence/research-tasks/prototypes/pristine_verify.py:6-19`. |
| [I] Read `solution/` or hidden expected answers. | [I] Do not mount `solution/` or hidden verifier fixtures in agent capsules; mount them only for oracle/reference validation or verifier phase. |
| [I] Special-case visible tests. | [I] Split tests into public smoke tests and hidden pristine tests copied only at verification; generate deterministic hidden fixtures from harness-owned seed recorded in result metadata, not from a path the agent can read. |
| [I] Modify build files to skip tests. | [I] Verifier-owned tests should build/import the public artifact directly and fail if the build target disappears; for Rust copy hidden integration tests into `tests/`, for Python import the module under test, and for C++ compile a verifier-owned test binary against the candidate library. |
| [I] Fetch code or answers from the network. | [I] Run agent build/test/reference/verifier phases with `bwrap --unshare-net`; allow network only for model transport outside task-code execution if the harness needs it. |
| [I] Read host credentials or caches. | [I] Use `--clearenv`, tmpfs `/home`, a minimal env allowlist, and read-only toolchain/cache layers; do not mount host `$HOME`, `.ssh`, or broad cache trees. |
| [I] Mine git history for the solution. | [I] Materialize the candidate task as a working tree without `.git`, without branch history, and without solution files. |
| [I] Write outside the task or contaminate sibling tasks. | [I] Bind only the candidate workspace and per-run cache as writable; bind source/toolchain layers read-only and reject output paths outside an allowlist. |
| [I] Forge reward files directly. | [I] Put reward output in verifier-owned `/logs/verifier`, clear it before verify, run tests after agent phase, and parse only the reward produced by the verifier command. |
| [I] Make tests flaky to win retries. | [I] Record every verifier command, stdout/stderr, exit code, wall time, and retry count; aggregate nondeterminism as repeat variance and mark tasks with inconsistent reference passes as invalid. |

- [R] The pristine-restore prototype is exactly 40 lines and restores verifier-owned tests before running the task-local verifier command. Evidence: `docs/efforts/multi-agent-bench/evidence/research-tasks/prototypes/pristine_verify.py:1-40`.
- [X] Demonstration command after a simulated deleting agent removed `workspace/tests`: `bash tests/test.sh` produced `bash: tests/test.sh: No such file or directory` and `naive_status=127`.
- [X] The pristine verifier demonstration under bwrap used `bwrap --ro-bind / / --dev-bind /dev /dev --bind docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/pristine-demo docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/pristine-demo --tmpfs /tmp --unshare-net --clearenv --setenv HOME docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/pristine-demo/home --setenv PATH /usr/local/bin:/usr/bin:/bin --chdir /data02/home/philip.yang/workspace/corenius-worker-research-tasks python3 docs/efforts/multi-agent-bench/evidence/research-tasks/prototypes/pristine_verify.py --workspace docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/pristine-demo/workspace --pristine-tests docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/pristine-demo/pristine-tests -- bash tests/test.sh`.
- [X] The pristine verifier restored the missing tests and failed for the real reason, not because tests were absent: output summary was `elapsed=0.046`, `pristine_status=1`, because `app/value.txt` contained `13` rather than the expected `42`.

6. Multi-agent task designs

- [R] Corenius's collaboration design names five scarcity axes: source text S1, landing ref S2, gate capacity S3, attention S4, and shared mutable state S5. Evidence: Corenius `docs/spikes/collab/design-space.md:23-35`.
- [R] Corenius's safety/liveness invariants include gated landing refs I1, one writer per working tree I2, scarce device exclusivity I3, append-only replayable coordination facts I4, human veto I5, and request liveness L1. Evidence: Corenius `docs/spikes/collab/design-space.md:36-52`.
- [R] The simulator's existing result columns are throughput per hour, ready-to-land latency, cycle time, wasted gate minutes, text conflicts, semantic conflicts reaching mainline, red-main minutes, blocked minutes per agent, contamination, and bad runs. Evidence: Corenius `experiments/collab/RESULTS.md:3-10`.
- [R] The design-space event vocabulary includes task decomposition, claims, leases, landing, conflict prediction, messages, contracts, verification, human decisions, and runtime events. Evidence: Corenius `docs/spikes/collab/design-space.md:750-766`.
- [I] The real benchmark should turn those simulated quantities into per-run fields: landed tickets/hour, ticket latency p50/p95, cycle p50, wasted verifier seconds, text conflicts, semantic conflicts caught by verifier, red-tip duration if any, blocked wait time, contamination events, redo work, and bad-run invariant violations.

| id | dependency shape | ticket decomposition | correct landing order | expected conflict | coordination metrics |
|---|---|---|---|---|---|
| mt01-disjoint-cli-commands | [I] Disjoint source files, shared test suite. | T1 add `import` command, T2 add `export`, T3 docs/help text. | T1/T2 parallel, T3 last. | [I] none if files claimed correctly. | wall time vs sum solo, claims precision, gate queue time. |
| mt02-shared-parser-interface | [I] Shared `parser.py` interface with two downstream commands. | T1 extend AST, T2 update formatter, T3 update linter. | T1 first, T2/T3 after. | [I] stale-read semantic failure if T2/T3 start from old AST. | stale.read count, redo work, semantic conflict caught by verifier. |
| mt03-two-ticket-same-file | [I] Two tickets edit separate hunks in `router.py`. | T1 add route normalization, T2 add auth bypass fix, T3 integration tests. | T1/T2 any order, T3 last. | [I] real text conflicts in nearby table/list. | text conflicts, rebase time, bad-run rate. |
| mt04-generated-artifact | [I] Schema source generates Python and Rust constants. | T1 update schema, T2 regenerate Python use site, T3 regenerate Rust use site. | T1 before T2/T3. | [I] generated artifact drift if workers regenerate from different schema. | artifact digest mismatch, redo items, landed order violations. |
| mt05-lockfile-hotspot | [I] Two Python dependencies from local wheels update one `uv.lock`. | T1 add parser wheel, T2 add reporter wheel, T3 update import code. | T1/T2 under lockfile coordination, T3 after both. | [I] S5 hotspot conflict in `uv.lock`. | lockfile conflicts, blocked time, final dependency graph correctness. |
| mt06-rust-workspace-feature | [I] Workspace feature spans three crates. | T1 feature in core, T2 CLI uses feature, T3 integration test crate. | T1 first, T2/T3 after. | [I] semantic conflict if feature name diverges. | declared vs observed footprint, verifier failures by ticket. |
| mt07-cmake-target-split | [I] C++ library target split affects link graph and tests. | T1 split `core` library, T2 update binary link, T3 update test link. | T1 first, T2/T3 after. | [I] build-system conflict in `CMakeLists.txt`. | wasted build minutes, text conflicts, fast-forward retries. |
| mt08-csv-pipeline-stages | [I] Pipeline stages are disjoint but share data contract. | T1 parser, T2 transform, T3 aggregator, T4 end-to-end verifier. | T1/T2/T3 can parallel after contract registered; T4 last. | [I] semantic mismatch on field names. | contract events, stale.read, semantic verifier failures. |
| mt09-state-migration | [I] Data migration touches schema, loader, and compatibility tests. | T1 schema bump, T2 loader backward compatibility, T3 exporter forward compatibility. | T1 first, T2/T3 after. | [I] shared fixture edits and semantic conflict. | redo work, red-tip if gated wrong, contamination. |
| mt10-observability-reward | [I] Harness records extra coordination metrics while task behavior changes. | T1 emit event, T2 aggregate result, T3 verifier asserts metrics. | T1/T2 before T3. | [I] shared result schema edits. | result-schema validation failures, metric completeness, wall time. |

- [I] These tasks directly check simulator assumptions that were weak in the results note: conflict resolution cost, semantic conflict clustering by module, gate cost scaling with change size, stale-instruction cost, and whether declared footprints predict observed footprints. Evidence for simulator weak spots: Corenius `experiments/collab/RESULTS.md:100-107`.
- [I] The tasks also give live checks for design-space experiments E1, E2, E4, E5, E6, E8, E9, E15, and E16 because they log footprint precision/recall, landing throughput, soft/hard claim value, conflict prediction, message delivery, verifier-agent signal if added later, recovery invariants, decomposition quality, and enforcement reach. Evidence for proposed experiments: Corenius `docs/spikes/collab/design-space.md:997-1035`.

7. Result schema and aggregate table

- [I] A per-run `result.json` should keep Harbor-compatible concepts but add coordination fields: `schema_version`, `run_id`, `task_id`, `condition`, `task_digest`, `capsule.digest`, `capsule.layers`, `capsule.net`, `capsule.toolchain`, `agent`, `tickets`, `outcome`, `timing`, `coordination`, `cost`, `checks`, `exceptions`, and `artifacts`.
- [I] `outcome` should contain at least `passed`, `reward`, `non_determinism_class`, and `exception_type`; `timing` should contain wall, setup, agent, land, verify, and gate wait seconds; `coordination` should contain conflicts, redo work, contamination, stale reads, lease waits, landed ticket count, failed landing attempts, and bad-run invariant violations.
- [I] Costs should be nullable and separated by source: model input tokens, cache tokens, output tokens, tool runtime cost, GPU seconds, and USD when available.
- [I] Capsule identity should include the capsule digest, layer digests, no-network flag, writable mount list digest, readonly mount list digest, toolchain versions, host kernel/driver metadata when host-provided layers are used, and whether any layer is non-hermetic.
- [R] The prototype validator requires `schema_version`, `run_id`, `task_id`, `condition`, `capsule`, `outcome`, `timing`, `coordination`, `cost`, and `checks`; it rejects unsupported schema versions and non-finite numeric fields. Evidence: `docs/efforts/multi-agent-bench/evidence/research-tasks/schema/result_schema.py:7-50`.
- [R] The sample result records condition `solo`, capsule digest/net/toolchain, passed outcome, timing, coordination counters, null cost fields, and the verifier command. Evidence: `docs/efforts/multi-agent-bench/evidence/research-tasks/schema/sample-result.json:1-39`.
- [X] Validator command under bwrap: `bwrap --ro-bind / / --dev-bind /dev /dev --bind docs/efforts/multi-agent-bench/evidence/research-tasks docs/efforts/multi-agent-bench/evidence/research-tasks --tmpfs /tmp --unshare-net --clearenv --setenv HOME docs/efforts/multi-agent-bench/evidence/research-tasks/run-work/home --setenv PATH /usr/local/bin:/usr/bin:/bin --chdir /data02/home/philip.yang/workspace/corenius-worker-research-tasks python3 docs/efforts/multi-agent-bench/evidence/research-tasks/schema/result_schema.py docs/efforts/multi-agent-bench/evidence/research-tasks/schema/sample-result.json`; output was `valid docs/efforts/multi-agent-bench/evidence/research-tasks/schema/sample-result.json`, `elapsed=0.040`.

Aggregate table proposal:

| grouping columns | summary columns |
|---|---|
| [I] `task_id`, `track`, `difficulty`, `single_multi`, `condition`, `agent_id`, `capsule_digest`, `n_runs` | [I] `pass_rate`, `pass_rate_ci_low`, `pass_rate_ci_high`, `reward_mean`, `reward_sd`, `wall_time_mean`, `wall_time_sd`, `wall_time_p50`, `wall_time_p95`, `non_deterministic_runs`, `exception_rate`, `cost_usd_mean`, `input_tokens_mean`, `output_tokens_mean`, `conflicts_mean`, `redo_work_mean`, `contamination_rate`, `wasted_gate_sec_mean`, `speedup_vs_solo`, `solo_sum_sec`, `fleet_wall_sec`, `bad_run_count` |

- [I] Variance should be reported at two levels: within-task repeat variance and across-task aggregate variance; otherwise a 24-task suite with clustered easy/hard tasks can overstate confidence.
- [I] Non-determinism should be recorded as repeated reference-solution disagreement, repeated verifier disagreement on the same fixed tree, or agent-run stochasticity with identical capsule digest and prompt.

8. Sizing for binary pass/fail

- [X] Sizing command:

```bash
python3 - <<'PY'
import math
z_alpha=1.96
z_beta=0.84
p1=0.50
p2=0.70
pbar=(p1+p2)/2
n=((z_alpha*math.sqrt(2*pbar*(1-pbar)) + z_beta*math.sqrt(p1*(1-p1)+p2*(1-p2)))/(p2-p1))**2
print(f'n_per_arm={n:.2f}, ceil={math.ceil(n)}')
print(f'24 tasks x 4 repeats = {24*4} observations per condition')
PY
```

- [X] Output was `n_per_arm=92.90, ceil=93` and `24 tasks x 4 repeats = 96 observations per condition`.
- [I] Assumptions: two-sided alpha 0.05, 80% power, independent Bernoulli outcomes, solo pass probability 0.50, fleet pass probability 0.70, and no task clustering.
- [I] The independence assumption is likely false because runs cluster by task, track, and difficulty; use a mixed-effects logistic model or bootstrap by task, and treat `24 x 4` as a pilot rather than a final powered benchmark if task-level variance is high.
- [I] If the plausible effect is smaller than 20 percentage points, the run count must increase sharply; for example, detecting 0.50 to 0.60 would require roughly four times as many independent observations as 0.50 to 0.70 because the denominator is squared.

Open questions

- [R] Harbor docs and source establish Terminal-Bench 2.0 conversion behavior, but the Terminal-Bench 2 task repository contents were not present locally; I could not read real Terminal-Bench task `task.yaml`, `run-tests.sh`, or hidden test conventions beyond the Harbor mapper and registry. Evidence: Harbor `registry.json:94012-94140`, `src/harbor/mappers/terminal_bench.py:48-77`.
- [I] Decide whether the first benchmark suite should exclude LLM-judge tasks entirely or include a separate optional online track with explicit credentials, network policy, and cost accounting.
- [I] Decide whether to preserve Harbor's `reward.txt` fallback or require `reward.json` only; JSON is better for multi-metric coordination data, but text fallback improves compatibility.
- [I] Decide how strictly to enforce CPU/memory/storage in the bwrap capsule; Harbor records fields and providers enforce differently, while bare bwrap needs external `timeout`, cgroup, or runner-level enforcement.
- [I] Decide whether six verified sketches are enough for the current planning milestone or whether every one of the 24 catalog entries must be instantiated before implementation starts.
- [I] Decide whether task fixtures should be committed as hand-written directories only or generated from a compact task DSL that can create start/fixed/reference variants.
- [I] Decide how much of RewardKit to reuse; its deterministic criteria vocabulary is useful, but its install and judge paths may not fit no-network capsules without vendoring.

Things I got wrong or could not check

- [R] I initially inherited the older decision note that `cpp3rdlib` resolution was still open; the later decision corrects it and says versioned `cpp3rdlib/...@//...` labels resolve from vendored local packages. Evidence: `docs/efforts/multi-agent-bench/decisions/20260929-probe-findings-blade-and-insula.md:34-36`, `docs/efforts/multi-agent-bench/decisions/20260929-blade-3rdlib-resolves-from-local-packages.md:6-19`.
- [X] I did not run `blade`, `bpt`, or `bvc` in this worker session.
- [X] I did not run network commands or installs; no `curl`, `wget`, `git fetch`, `git clone`, `pip install`, or `uv` install command was used in this worker session.
- [X] A preliminary timing wrapper used `/usr/bin/time`, but `/usr/bin/time` was absent and printed `/bin/bash: line 206: /usr/bin/time: No such file or directory`; I switched to bash `TIMEFORMAT`.
- [X] A preliminary verification sweep used `bash -lc`, which reset `PATH` enough that `uv` and `cargo` were unavailable; I reran with `bash -c` and explicit `PATH`.
- [X] The first `t01-log-window` start state accidentally passed; I changed its verifier window to `3 4`, expecting `errors=1 warnings=1`, so the start state now fails and the fixed state passes.
- [X] I saw one preliminary bwrap run print `eth1: error fetching interface information: Device not found`; the successful rerun did not show it, so I treat it as host/interface noise rather than task behavior.
- [I] I did not inspect the full Harbor registry for all Terminal-Bench 2 task names beyond the sampled line range and grep output, so the task-category list is intentionally not an official exhaustive taxonomy.

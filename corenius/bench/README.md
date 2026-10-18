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
  insula/requests/       capsule requests: toolchains.json (offline task capsule), terminal_bench.json
                         (toolchains plus /app and /tests workspace aliases), deep_swe.json
                         (toolchains plus /app, /tests and /logs workspace aliases), swe_bench.json
                         (toolchains plus /testbed and /logs workspace aliases), agent.json (adds traecli, network)
  insula/layers/         small tree layers (blade-offline wrapper)
  harness/               tasklib, validate, fleet, run, results, pilot.sh (+ tests)
  inventories/           external public benchmark inventories (catalog snapshots, not runnable Corenius tasks)
  results/               run output (gitignored)
tools/insula/capsule.py  layered capsule builder and runner
```

`bench/inventories/` contains tracked metadata snapshots for external public
agent benchmarks. They are source inventories for comparison and planning; only
`bench/tasks/` contains runnable Corenius benchmark tasks.

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
python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --all --replace
python3 bench/harness/validate.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench --task terminal_bench_analyze_access_logs
python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task simple-web-scraper --replace
python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_simple_web_scraper --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task accelerate-maximal-square --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_accelerate_maximal_square --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task audio-synth-stft-peaks --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_audio_synth_stft_peaks --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task kv-store-grpc --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_kv_store_grpc --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task form-filling --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_form_filling --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task sqlite-db-truncate --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_sqlite_db_truncate --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task pandas-etl --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_pandas_etl --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task pandas-sql-query --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_pandas_sql_query --agent reference
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task count-call-stack --task gcode-to-text --task large-scale-text-editing --task log-summary-date-ranges --task recover-accuracy-log --replace
python3 tools/temp_root.py -- python3 bench/harness/validate.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench --task terminal_bench_count_call_stack --task terminal_bench_gcode_to_text --task terminal_bench_large_scale_text_editing --task terminal_bench_log_summary_date_ranges --task terminal_bench_recover_accuracy_log
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task deterministic-tarball --task jq-data-processing --task regex-log --task tree-directory-parser --replace
python3 tools/temp_root.py -- python3 bench/harness/validate.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench --task terminal_bench_deterministic_tarball --task terminal_bench_jq_data_processing --task terminal_bench_regex_log --task terminal_bench_tree_directory_parser
python3 tools/insula/docker_image_rootfs.py IMAGE --out-root .scratch/public_agent_benchmarks/rootfs --replace
python3 tools/temp_root.py -- docker pull debian:bookworm-slim
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py debian:bookworm-slim --out-root .scratch/public_agent_benchmarks/rootfs --name-suffix html_filter_browser_apt --apt-package chromium --apt-package chromium-driver --apt-package python3 --apt-package python-is-python3 --apt-package python3-pytest --apt-package python3-selenium --apt-package python3-bs4 --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task filter-js-from-html --task break-filter-js-from-html --replace
python3 tools/temp_root.py -- python3 bench/harness/validate.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench --task terminal_bench_filter_js_from_html --task terminal_bench_break_filter_js_from_html
python3 tools/temp_root.py -- docker pull postgres:16
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py postgres:16 --out-root .scratch/public_agent_benchmarks/rootfs --name-suffix python_test_apt --apt-package python3 --apt-package python3-pip --apt-package python3-pandas --apt-package python3-pytest --apt-package python3-psycopg2 --apt-package procps --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task postgres-csv-clean --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_postgres_csv_clean --agent reference
python3 tools/temp_root.py -- docker pull ubuntu:24.04
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py ubuntu:24.04 --out-root .scratch/public_agent_benchmarks/rootfs --name-suffix home_server_https_apt --apt-package nginx --apt-package curl --apt-package openssl --apt-package dnsutils --apt-package net-tools --apt-package procps --apt-package python3 --apt-package python3-pytest --apt-package python3-flask --apt-package python3-requests --apt-package python3-urllib3 --mkdir /root/.acme.sh --mkdir /var/www --touch /etc/ddclient.conf --touch /etc/nginx/nginx.conf --touch /etc/nginx/sites-available/default --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task home-server-https --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_home_server_https --agent reference
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py ubuntu:24.04 --out-root .scratch/public_agent_benchmarks/rootfs --name-suffix security_celery_redis_rce_apt --apt-package redis-server --apt-package python3 --apt-package python3-pytest --apt-package python3-redis --apt-package python3-requests --mkdir /opt/security_celery_redis_rce --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task security-celery-redis-rce --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_security_celery_redis_rce --agent reference
python3 tools/temp_root.py -- docker pull vulhub/minio:2023-02-27T18-10-45Z
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py vulhub/minio:2023-02-27T18-10-45Z --out-root .scratch/public_agent_benchmarks/rootfs --name-suffix python_test_apt --apt-debian-archive --apt-package python3 --apt-package python3-pytest --apt-package curl --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task security-vulhub-minio --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_security_vulhub_minio --agent reference
python3 tools/temp_root.py -- docker pull vulhub/spring-messaging:5.0.4
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py vulhub/spring-messaging:5.0.4 --out-root .scratch/public_agent_benchmarks/rootfs --name-suffix python_test_apt --apt-debian-archive --apt-package python3 --apt-package python3-pytest --apt-package python3-requests --apt-package curl --mkdir /server --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task spring-messaging-vul --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_spring_messaging_vul --agent reference
python3 bench/harness/materialize_public_tasks.py --suite deep_swe --task abs-module-cache-flags --replace
python3 bench/harness/validate.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/deep_swe --task deep_swe_abs_module_cache_flags
python3 bench/harness/materialize_public_tasks.py --suite swe_bench --swe-dataset SWE-bench/SWE-bench_Lite --swe-split dev --task marshmallow-code__marshmallow-1343 --replace
python3 bench/harness/validate.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/swe_bench --task swe_bench_marshmallow_code_marshmallow_1343
rootfs="$(python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py programbench/abishekvashok_1776_cmatrix.5c082c6:task_cleanroom_v6 --name-suffix pytest_timeout --pip-package pytest-timeout==2.4.0 --replace | tail -n 1)"
python3 bench/harness/materialize_public_tasks.py --suite program_bench --task abishekvashok__cmatrix.5c082c6 --program-branch 79b69dd3fd98 --rootfs "$rootfs" --replace
python3 bench/harness/validate.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/program_bench --task program_bench_abishekvashok_cmatrix_5c082c6
bench/harness/pilot.sh OUT [REPEAT] [PARALLEL]                            # live pilot over all single tasks
```

`validate.py` runs, for every task and inside the capsule: the start state fails; the reference solution applies and passes; the
verdict is deterministic; deleted or stubbed tests do not help; an empty patch fails; tampering with protected source files is
undone before grading. Multi tasks add: every ticket is necessary, and every landing order ends in a pass with the declared
number of conflicts.

`materialize_public_tasks.py` adapts metadata-only public benchmark inventories
into runnable Corenius task directories under `.scratch/`. The Terminal-Bench
materializer reads the locked public source cache, skips tasks that lack upstream
tests or reference solutions, and emits a `materialization_manifest.json` with
exact counts and skip reasons. Generated Terminal-Bench tasks use the
`terminal_bench` capsule, which aliases `/app` to the writable workspace and
`/tests` to `/workspace/tests` so upstream tests and solutions can run inside
insula without Docker containers. Some formerly multi-container tasks can be
adapted with task-local `[bench.services.*]` entries; `simple-web-scraper` runs
its upstream Python `server` service inside the same Insula invocation, with a
temporary `server` hostname mapped to loopback. `simple-sheets-put` runs the
upstream FastAPI service in Insula too, using a workspace SQLite database as the
persistent SQLAlchemy backend so state survives the separate agent/reference and
verifier invocations without a Dockerized Postgres service. When an upstream client image
expects Python packages that the host capsule lacks, the materializer can build a
scratch Python dependency layer and bind it into the task-local capsule request.
`accelerate-maximal-square` uses that path for NumPy and rewrites the stale
runtime `pip install numba==0.53.1` reference into an offline C-backed Python
module that preserves the expected `/app/maximal_square.py` interface while
meeting the public speed threshold inside Insula.
`audio-synth-stft-peaks` uses the same dependency-layer path for pinned
NumPy/SciPy, removes the upstream `apt-get`/`pip3 install` bootstrap from the
reference script, and keeps the public Bash/awk `PEAKS_ONLY` behavior intact.
`kv-store-grpc` uses a pinned gRPC dependency layer, asks the agent to produce a
runnable `/app/server.py` instead of relying on a background process surviving
between phases, and starts that server as a verifier-only service before pytest
connects to port 5328.
`form-filling` uses a pinned pypdf/scikit-learn/NumPy dependency layer and
replaces the upstream `wget` step with a deterministic local PDF placeholder so
the RAG-style extraction script and verifier stay fully offline under Insula.
`sqlite-db-truncate` validates through the generic Dockerfile `COPY` emulation:
the upstream corrupted `trunc.db` is projected into `/app`, and the stdlib-only
reference recovery script runs unchanged in the shared Terminal-Bench capsule.
`pandas-etl` combines that generic `COPY task-deps/data.csv ./` projection with
a pinned pandas dependency layer and a direct `python3` reference run, replacing
the upstream `curl`/`uv` package bootstrap.
`pandas-sql-query` translates the upstream Docker image-build data generation
into deterministic materialized `products.csv` and `orders.csv` inputs, then
runs the SQL verifier with pinned pandas and DuckDB in an offline dependency
layer.
`count-call-stack`, `gcode-to-text`, `large-scale-text-editing`,
`log-summary-date-ranges`, and `recover-accuracy-log` replay upstream
Dockerfile build-time data preparation into the materialized workspace:
archives are decompressed, deterministic input/log generators are run and then
removed when the upstream image removes them, `/app/raw_logs` and
`/app/recovered_logs` are created for recovered accuracy logs, and the
large-file Vim verifier snapshots `input.csv` around pytest so Corenius'
repeatability check is deterministic.
`deterministic-tarball`, `jq-data-processing`, and `tree-directory-parser`
also replay upstream image-build setup under `.scratch`: the deterministic
source tree, generated user JSON plus seeded hash, and generated `tree -F` map
are materialized before the agent starts. The jq task carries a task-local
minimal `yq` compatibility shim as a read-only dependency bundle on `PATH`, so
the public verifier command remains available inside Insula without a Docker
container. `regex-log` validates unchanged in the shared Terminal-Bench capsule.
`hello-world`, `assign-seats`, `bank-trans-filter`,
`aimo-airline-departures`, `countdown-game`, `constraints-scheduling`, and
`log-summary` validate through the generic Terminal-Bench adapter: Dockerfile
workspace `COPY` instructions are replayed into the materialized `/app`
workspace, including idempotent directory copies such as `COPY deps/ /app/deps/`
and workspace-root copies such as `COPY inputs/ /app/`.
`postgres-csv-clean` uses a prepared `postgres:16` rootfs customized with Python
verifier packages and starts PostgreSQL as a same-Insula service with `PGDATA`
under the workspace, so database state persists across the separate
agent/reference and verifier invocations while the task still avoids Docker at
run time. `filter-js-from-html` and `break-filter-js-from-html` use a prepared
`debian:bookworm-slim` rootfs customized with Chromium, Chromedriver, Selenium,
BeautifulSoup and pytest. The materializer vendors no upstream repository into
the tracked tree; it caches the public `html-sanitizer-testbed` archive under
`.scratch`, copies only the public test cases into the generated task workspace,
and rewrites the verifier to read those fixtures offline.
`home-server-https` uses a prepared `ubuntu:24.04` rootfs customized
with nginx, OpenSSL, DNS/network tools and Python verifier packages. Its task
capsule binds workspace-backed overlays over the absolute system paths the
upstream task checks (`/var/www`, `/root/.acme.sh`, `/etc/ddclient.conf`, and
nginx config files), starts the mock Google API during both phases, and starts
nginx only during verification from the config/certs written by the agent phase.
The task-local capsule request opts into root inside an unshared user namespace
with `CAP_NET_BIND_SERVICE` so nginx can bind port 443 inside Insula without
running benchmark code in Docker.
`acl-permissions-inheritance` uses a prepared `ubuntu:24.04` rootfs customized
with ACL, util-linux/passwd, Python and pytest packages. Its task-local capsule
projects `/srv` onto a workspace-backed overlay and declares the benchmark's
`research`, `alice`, `bob`, and outside-user `mallory` identities, so `runuser`,
setgid inheritance and POSIX ACL checks run against real mapped users inside
Insula:

```sh
python3 tools/temp_root.py -- docker pull ubuntu:24.04
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py ubuntu:24.04 --name-suffix acl_permissions_inheritance_apt --apt-package acl --apt-package util-linux --apt-package passwd --apt-package python3 --apt-package python3-pytest --mkdir /srv --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite terminal_bench --task acl-permissions-inheritance --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/terminal_bench terminal_bench_acl_permissions_inheritance --agent reference
```

`security-celery-redis-rce` uses a prepared `ubuntu:24.04` rootfs customized
with Redis and Python verifier/client packages. The generated task starts Redis,
the upstream hidden file-monitor service, and a vulnerable Redis-queue worker as
same-Insula services. Generated instructions, solution, monitor and verifier
use `/var/lib/corenius/security_celery_redis_rce` as a retained workspace overlay
so exploit effects survive into the verifier phase. Ephemeral service files use
the private `/run/corenius-tmp` tmpfs. The pinned upstream source is unchanged.
`security-vulhub-minio` uses the public `vulhub/minio:2023-02-27T18-10-45Z`
image as a prepared rootfs and starts the real MinIO binary as an agent-phase
service. Because the image is based on an archived Debian release, export it with
`--apt-debian-archive` before installing curl and Python verifier packages.
`spring-messaging-vul` uses two prepared rootfs layers: a modern `ubuntu:24.04`
rootfs for the agent/verifier Python stack, and the public
`vulhub/spring-messaging:5.0.4` rootfs for the Java service. The service runs in
its own Insula with host-loopback aliases so `/server/secret*.txt` is mounted
only into the service, while the agent can reach `http://spring:19080/...` and
write only the exfiltrated `/app/secret*.txt` outputs.
The Terminal-Bench manifest also classifies skipped Compose tasks by the missing
Insula capability: hidden sidecar filesystem isolation, persistent sidecar
lifecycle, multi-node service bundles, or a customized rootfs with system
services. Same-Insula helper services are only used for tasks where restarting
the service per command and exposing the copied workspace do not change the
benchmark's target behavior.
`fix-git` is intentionally skipped even though it has tests and a reference
solution: its upstream Docker build clones
`https://github.com/TheMikeMerrill/personal-site.git` at commit `d7d3e4b`, and
that repository currently returns 404 to unauthenticated public access.

`tools/insula/docker_image_rootfs.py` exports a locally available Docker image
into a prepared rootfs tree for the capsule `rootfs` layer. This is the intended
bridge for image-backed public suites such as DeepSWE, SWE-bench, and
ProgramBench: Docker may be used as an image store/exporter or rootfs source,
but benchmark code still runs under insula. Rootfs exports can install apt or
pip packages and can create explicit mountpoint directories/files with `--mkdir`
and `--touch` for later workspace-backed overlays. Use `--apt-debian-archive`
for old Debian-based images whose default apt repositories have moved to the
archive mirrors.

The DeepSWE materializer reads public `datacurve-ai/deep-swe` task definitions,
clones the target repository into `.scratch/public_agent_benchmarks/cache/`,
checks out the recorded base commit, and emits runnable Corenius task roots
under `.scratch/public_agent_benchmarks/materialized_tasks/deep_swe`. The
generated verifier keeps DeepSWE's upstream `/app`, `/tests`, `/logs/artifacts`
and `/logs/verifier` paths by aliasing them into the workspace. For full-fidelity
runs, first pull the task's recorded image and export it:

```sh
python3 tools/temp_root.py -- docker pull public.ecr.aws/d3j8x8q7/swe-bench-202605:kh75679ajj3b8dtd7se3h7z0a1833y6r-v1.1
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py public.ecr.aws/d3j8x8q7/swe-bench-202605:kh75679ajj3b8dtd7se3h7z0a1833y6r-v1.1 --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite deep_swe --task abs-module-cache-flags --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/deep_swe deep_swe_abs_module_cache_flags --agent reference
```

If the matching prepared rootfs is missing, generated DeepSWE tasks fall back to
the shared `deep_swe` capsule and record `rootfs = ""` in `task.toml`; that is
useful for adapter inspection but may fail verifier dependencies such as Go,
Node, Python packages, or pinned report tools.

The SWE-bench materializer reads full public HuggingFace rows, clones the target
repository at `base_commit`, writes the problem statement, reference patch,
test patch and F2P/P2P lists into a Corenius task root, and aliases `/testbed`
to the writable workspace. Full-fidelity runs need the row's recorded image
exported first:

```sh
python3 tools/temp_root.py -- docker pull swebench/sweb.eval.x86_64.marshmallow-code_1776_marshmallow-1343:latest
python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py swebench/sweb.eval.x86_64.marshmallow-code_1776_marshmallow-1343:latest --replace
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite swe_bench --swe-dataset SWE-bench/SWE-bench_Lite --swe-split dev --task marshmallow-code__marshmallow-1343 --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/swe_bench swe_bench_marshmallow_code_marshmallow_1343 --agent reference
```

The ProgramBench materializer reads public `ProgramBench/ProgramBench` task
metadata, copies the cleanroom image workspace from a prepared rootfs, downloads
the selected public branch tarball(s) from `programbench/ProgramBench-Tests`,
and emits task-local protected verifier data. The generated verifier translates
ProgramBench's container commit lifecycle into Insula-local copies: it snapshots
the submitted workspace, removes stale `./executable`, runs `./compile.sh`
offline, stashes the newly built executable, restores the compiled snapshot for
each selected branch, extracts that branch's public test tarball, runs
`eval/run.sh`, and parses `eval/results.xml` itself.

Real ProgramBench tasks do not publish oracle source patches, so the generated
reference solution is an adapter smoke oracle only. For the cmatrix exemplar it
uses the selected public branch source to build `./executable`; that proves the
rootfs/verifier path under Insula, but it is not a leaderboard-faithful
ProgramBench baseline. The public cmatrix test runner expects `pytest-timeout`;
prepare a customized rootfs once and then materialize against it:

```sh
python3 tools/temp_root.py -- docker pull programbench/abishekvashok_1776_cmatrix.5c082c6:task_cleanroom_v6
rootfs="$(python3 tools/temp_root.py -- python3 tools/insula/docker_image_rootfs.py programbench/abishekvashok_1776_cmatrix.5c082c6:task_cleanroom_v6 --name-suffix pytest_timeout --pip-package pytest-timeout==2.4.0 --replace | tail -n 1)"
python3 tools/temp_root.py -- python3 bench/harness/materialize_public_tasks.py --suite program_bench --task abishekvashok__cmatrix.5c082c6 --program-branch 79b69dd3fd98 --rootfs "$rootfs" --replace
python3 tools/temp_root.py -- python3 bench/harness/run.py --tasks-root .scratch/public_agent_benchmarks/materialized_tasks/program_bench program_bench_abishekvashok_cmatrix_5c082c6 --agent reference
```

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
`cargo build --manifest-path controller/Cargo.toml --target-dir .scratch/controller-backed-bench/target --locked
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

`tools/insula/capsule.py` builds a tmpfs root with either the host `/usr` read-only or an explicit prepared `rootfs` tree,
tool layers at fixed paths, the workspace at `/workspace`, a generated `/etc/hosts`
with `localhost` and service aliases, a cleared environment (an allowlist), a
private `/proc`, and `--unshare-net` unless the policy says otherwise.
Rootfs-backed task capsules can opt into declared benchmark identities; Insula
then starts bwrap inside a mapped user namespace using the caller's subuid/subgid
range so tests can switch to those users without granting host root.
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

[R] Corenius currently enforces a single selected rootfs, not a layered capsule builder; source: `README.md:59-66` and `src/insula.rs:174-219`.
[R] The approved benchmark plan requires every task-code execution to run inside an insula bwrap boundary; source: `/data02/home/philip.yang/.claude/plans/tidy-questing-alpaca.md:5-10` and `docs/efforts/multi-agent-bench/EFFORT.md:7-13`.
[R] RTO's capsule model is the closest reusable design: base rootfs, spack, uv, candidate, plan, state, driver interface; source: RTO `AGENTS.md:51-70`.
[X] bwrap on this host is `bubblewrap 0.8.0` and supports the options needed for a root-tmpfs assembly; source: `docs/efforts/multi-agent-bench/evidence/research-insula/bwrap-help.txt:1-79`.
[X] A root-tmpfs assembly can ro-bind host `/usr`, synthesize `/bin`, `/lib`, `/lib64`, `/sbin`, mount layers, and run `sh`, `python3`, and `jq`; source: `docs/efforts/multi-agent-bench/evidence/research-insula/root-tmpfs.argv.txt:1-72` and `docs/efforts/multi-agent-bench/evidence/research-insula/bwrap_probe.out:2-18`.
[X] In that assembly `/usr` is read-only, `/workspace` is writable, `/home` hides host home dirs, `/proc` is private, and `--clearenv` leaves only the allowlist; source: `docs/efforts/multi-agent-bench/evidence/research-insula/bwrap_probe.out:7-26`.
[X] `--unshare-net` plus loopback setup leaves no route and permits only loopback traffic in the probe; source: `docs/efforts/multi-agent-bench/evidence/research-insula/bwrap_probe.out:37-42`.
[X] GPU is not usable on this host during this run: driver libraries exist, but `/dev/nvidia*` nodes are missing and both host and inner `nvidia-smi -L` fail; source: `docs/efforts/multi-agent-bench/evidence/research-insula/gpu_probe.out:1-24` and `docs/efforts/multi-agent-bench/evidence/research-insula/gpu-device-proc.out:1-8`.
[X] cargo, uv, cmake+ninja, and Spack GCC hello-world builds work inside bwrap with `--unshare-net`; Blade does not work from fresh HOME without preseeded Python 2.7 deps; source: `docs/efforts/multi-agent-bench/evidence/research-insula/toolchain_probe.out:2-24` and `docs/efforts/multi-agent-bench/evidence/research-insula/blade-fresh-home.out:1-137`.
[I] The benchmark builder should be a Python capsule builder with explicit layer requests, content-addressed read-only mounts, host-ro non-hermetic labels, and probes that fail on stale identity, host leakage, and writable-scope violations; basis: findings 1-9 below.

# Insula capsule and bwrap layering evidence

## 1. Corenius Insula Contract

1. [R] The current user-visible contract is: selection file `.corenius/insula/selected.json`, rootfs marker `.corenius-insula-rootfs.json`, marker fields `schema_version: 1`, `owner: "corenius-insula"`, `digest`, `node_path`, optional `dependency_lock_digest`, and a selected rootfs that must live under `store_root`, must not be a symlink, and whose saved digest must match the marker digest when loaded; source: `README.md:59-66`.

2. [R] The Rust data model has `InsulaMarker { schema_version, owner, digest, node_path, dependency_lock_digest }`, `InsulaSelection { rootfs, store_root, workspace, marker }`, and `BwrapPlan { executable, args, env }`; source: `src/insula.rs:15-45`.

3. [R] `validate_selection` rejects a symlink rootfs, non-directory rootfs, a rootfs outside canonical `store_root`, and a marker digest mismatch; source: `src/insula.rs:64-98`.

4. [R] `load_selection` reads `.corenius/insula/selected.json` and revalidates with the saved digest, while `scan` only accepts children of the store root whose marker validates; source: `src/insula.rs:119-148`.

5. [R] `build_command` maps only one rootfs at `/`, binds the whole workspace read-write at `/workspace`, mounts tmpfs `/tmp` and `/run`, creates `/home/corenius`, mounts private `/proc` and `/dev`, translates the cwd to `/workspace/...`, and runs `/bin/sh -lc <command>`; source: `src/insula.rs:150-200`.

6. [R] The runtime env is `env_clear()` plus the filtered map: preserved ambient keys are `TERM`, `COLORTERM`, and proxy variables, and Corenius sets `CORENIUS_IN_INSULA=1`, `HOME=/home/corenius`, `TMPDIR=/tmp`, fixed `PATH`, and `PWD=/workspace`; source: `src/insula.rs:201-219` and `src/insula.rs:325-337`.

7. [R] `InsulaSandbox::run` enforces timeout by killing the child and collects stdout/stderr, and `doctor` verifies the marker's `node_path` exists before reporting the bwrap plan; source: `src/insula.rs:228-303`.

8. [R] Tests enforce the current contract: bad digest, outside-store rootfs, wrong owner, malformed marker, and symlink rootfs are rejected; selection round-trip catches digest drift; plan construction filters `SECRET` while preserving allowed `TERM`; source: `tests/insula.rs:19-114`.

9. [R] The CLI currently supports `insula select` and dry-run `insula doctor`, and the CLI test checks that selecting writes state and doctor prints `bwrap plan:`; source: `tests/cli_tui.rs:57-95`.

10. [I] A layered benchmark capsule builder must add what the Rust contract lacks: a serialized rootfs plan, multiple layer mounts, per-layer digests, plan digest, capsule digest, stale-layer checks, driver binds, net modes, per-step writable policy, toolchain cache policy, and probes.

## 2. RTO Capsule Anatomy and Reuse

1. [R] I read RTO at commit `584d9b8da62bc27e794076cff44d27b64f688ad9`, recorded in this worktree's reference file as the master commit read; source: `docs/reference/robust-training-observatory.md:1-9`; command form used for RTO source reads was `git -C /data02/home/philip.yang/workspace/robust-training-observatory show 584d9b8da62bc27e794076cff44d27b64f688ad9:<path>`.

2. [R] RTO terminology separates four properties: hermetic means dependency provenance, isolated means namespaces and binds, offline means no runtime network beyond loopback, and reproducible means the rootfs digest comes from a committed build; source: RTO `AGENTS.md:14-21`.

3. [R] RTO defines Insula as the host facility, capsule as rootfs plus candidate plus plan, rootfs as a content-addressed store tree, rootfs plan as serialized binds and namespaces, candidate as read-only code under test, driver interface as the only host contribution, and evidence as `capsule x command -> evidence`; source: RTO `AGENTS.md:23-39`.

4. [R] RTO's capsule anatomy orders layers by change velocity and requires one owner, one build step, one digest, and read-only runtime mounting per layer; source: RTO `AGENTS.md:51-59`.

5. [R] RTO's layer table is base rootfs at `/`, spack foundation at `/run/insula/layers/spack`, uv overlay at `/run/insula/layers/uv`, candidate at `/workspace/candidate/<name>`, writable state at `/run/insula/out` and `/tmp`, and driver interface under `/dev/nvidia*`, `/dev/infiniband`, `/run/insula/driver/*`, and selected host config files; source: RTO `AGENTS.md:61-70` and `lab/agent-harness/INSULA.md:44-53`.

6. [R] RTO's path contract says scripts and reports use only canonical in-capsule paths: `/`, `/run/insula/layers/<layer>`, `/workspace/candidate/...`, `/run/insula/out`, and `/run/insula/driver`; source: RTO `AGENTS.md:73-76`.

7. [R] RTO rules require no installs at runtime, read-only layers, layer builds as the only runs with a writable layer mount, and do-not-shadow across layers; source: RTO `AGENTS.md:84-91`.

8. [R] RTO computes `capsule_digest = sha256(rootfs || spack || uv || candidate || plan)` in stack order; source: RTO `AGENTS.md:92-94`, `lab/agent-harness/INSULA.md:23-38`, and `rootfs/insula.py:511-519`.

9. [R] RTO's committed `capsule.json` records concrete identity fields: `candidate_digest`, `capsule_digest`, host metadata, ordered layer records, plan digest/store, properties, `rootfs_digest`, `spack_digest`, and `uv_digest`; source: RTO `lab/agent-harness/capsule.json:1-89`.

10. [R] RTO's rootfs plan is host-independent until launch and resolves placeholders such as `@rootfs`, `@layer:uv`, `@candidate:*`, `@out`, `@capsule`, `@plan`, and `@driver:*`; source: RTO `lab/agent-harness/INSULA.md:60-64` and RTO `docs/design/2026-09-29-insula-capsule-spike.md:121-160`.

11. [R] RTO's allowlisted binds include rootfs, spack and uv layers, candidate trees, out state, capsule and plan files, private tmpfs/virtual mounts, NVIDIA devices, NVIDIA driver libraries discovered from `ldconfig -p`, host `nvidia-smi`, `/etc/resolv.conf`, and inline `/etc/hosts`; source: RTO `lab/agent-harness/INSULA.md:71-94` and `lab/agent-harness/rootfs_plan.json:46-80`.

12. [R] RTO's launcher refuses non-placeholder `ro-bind`/`bind` sources, refuses unsupported `ro-bind-try` sources except `@driver:nvidia-smi` and `/etc/resolv.conf`, and refuses unsupported device paths outside `/dev/nvidia*` and `/dev/infiniband`; source: RTO `rootfs/insula.py:909-970`.

13. [R] RTO's digest helper hashes sorted paths and encodes symlinks, directories, regular files, and other inode types with mode and either link target, file-content hash, or device id; source: RTO `rootfs/insula.py:167-189`.

14. [R] RTO's JSON digest uses sorted compact JSON; source: RTO `rootfs/insula.py:94-99`.

15. [R] RTO's uv digest includes `uv.lock` bytes, interpreter path, and the uv-layer tree digest; source: RTO `rootfs/insula.py:278-283`.

16. [R] RTO's candidate build requires a pinned Arnold checkout, checks dirty staged candidate files, archives Arnold with fixed mtime, copies staged harness files, writes a deterministic tar with uid/gid/name/mtime scrubbed, stores the tar, extracts it, and chmods it read-only; source: RTO `rootfs/insula.py:286-299` and `rootfs/insula.py:325-350` and `rootfs/insula.py:384-451`.

17. [R] RTO's committed candidate path list is only `lab/agent-harness/README.md`, `lab/agent-harness/harness.py`, and `lab/agent-harness/rootfs/probe_insula.sh`; source: RTO `rootfs/insula.py:34-38`.

18. [R] RTO's uv layer build ro-binds the rootfs, gives exactly the uv stage directory a writable mount at `/run/insula/layers/uv`, binds a uv cache, ro-binds the harness, sets build env including `PYTHONNOUSERSITE=1`, `UV_CACHE_DIR`, index variables, and `UV_PYTHON_DOWNLOADS=never`, runs `uv venv` and `uv sync --frozen --no-install-project`, replaces `pip`/`uv` with fail-fast shims, scrubs bytecode, chmods read-only, renames to `uv-<digest>`, and records inputs; source: RTO `rootfs/insula.py:582-744`.

19. [R] RTO runtime stale checks refuse when rootfs, uv digest, uv inputs, plan digest, dirty candidate sources, or rebuilt candidate digest differ from `capsule.json`; source: RTO `rootfs/insula.py:771-800`.

20. [R] RTO's probe maps paths from `sys.path`, loaded modules, `/proc/self/maps`, and tool lookup; classifies by path prefix; rejects unknown paths and host-looking `/data02`, `/home/philip.yang`, `/workspace/robust-training-observatory`, and `/workspace/arnold` paths; checks shadow ownership for `numpy`, `requests`, `psutil`, `ray`, and `json`; source: RTO `rootfs/probe_insula.sh:115-176`.

21. [R] RTO's design text says the probe should classify paths by mount from `/proc/self/mountinfo`, but the committed probe classifies by path prefix; source: RTO design `docs/design/2026-09-29-insula-capsule-spike.md:193-198` versus RTO implementation `rootfs/probe_insula.sh:137-155`.

22. [R] RTO's probe recomputes identity from rootfs, spack, uv, candidate, and plan digests, then writes verdicts for hermetic, shadowing, isolated, offline, gpu, identity, plus py-spy; source: RTO `rootfs/probe_insula.sh:179-235` and `rootfs/probe_insula.sh:250-276`.

23. [R] RTO's documented probe verdicts are hermetic, shadowing, isolated, offline, gpu, and identity; source: RTO design `docs/design/2026-09-29-insula-capsule-spike.md:193-213`.

24. [R] RTO adopted from draccus: canonical path contract, per-layer ownership with one mutable layer, driver discovery via `ldconfig`, do-not-shadow, offline as separate from hermeticity, and run records stamped with capsule digest; source: RTO `docs/research/2026-09-28-draccus-as-reference-for-capsule-layering.md:17-28`.

25. [R] RTO deliberately did not adopt from draccus: mutable venvs, writable bundle state at runtime, interactive shell as first-class mode, and bundle-as-unit; source: RTO `docs/research/2026-09-28-draccus-as-reference-for-capsule-layering.md:29-38`.

26. [R] Known RTO limits: the spike excludes multi-host runs, a layer registry, security boundaries against hostile candidates, Spack builds, and Ascend/MLU drivers; source: RTO design `docs/design/2026-09-29-insula-capsule-spike.md:15-16`.

27. [R] Known RTO defect: rootfs provenance is still open because the design asks to record the base image manifest digest in `selected.json`; source: RTO design `docs/design/2026-09-29-insula-capsule-spike.md:240-243`.

28. [R] Known RTO defect: the committed harness reports Ray actor collection as `ok` but publishes zero actor records, and exercising Ray on `::1` remains open; source: RTO `lab/agent-harness/INSULA.md:135-139`.

29. [R] Known RTO limitation: it still fakes unavailable host services such as local metrics agent, Merlin, Lark, Housekeeper, and selected cgroup/API responses; source: RTO `lab/agent-harness/INSULA.md:141-143`.

30. [I] Reusable as-is for Corenius: capsule metadata shape, stable JSON/tree digest functions, stale-refusal policy, placeholder rootfs plan idea, run evidence carrying capsule and plan digests.

31. [I] Reusable with adaptation: driver-library discovery, uv layer build, probe verdict taxonomy, candidate archiving, because RTO has RTO-specific env vars, host name, Arnold pin, and candidate path list.

32. [I] Do not copy directly: RTO `OWNER`, `EXPECTED_HOSTNAME`, `EXPECTED_ARNOLD_COMMIT`, `HARNESS_CANDIDATE_PATHS`, fake-service assumptions, and exact Python package shadowing list.

## 3. bwrap 0.8 Behavior

1. [X] Command: `bwrap --version && bwrap --help > docs/efforts/multi-agent-bench/evidence/research-insula/bwrap-help.txt 2>&1`; output excerpt: `bubblewrap 0.8.0`; source: `docs/efforts/multi-agent-bench/evidence/research-insula/bwrap-help.txt:1`.

2. [X] The exact options available include `--args`, `--unshare-all`, `--share-net`, `--unshare-user`, `--unshare-user-try`, `--unshare-ipc`, `--unshare-pid`, `--unshare-net`, `--unshare-uts`, `--unshare-cgroup`, `--unshare-cgroup-try`, `--userns`, `--userns2`, `--disable-userns`, `--assert-userns-disabled`, `--pidns`, `--uid`, `--gid`, `--hostname`, `--chdir`, `--clearenv`, `--setenv`, `--unsetenv`, `--lock-file`, `--sync-fd`, `--bind`, `--bind-try`, `--dev-bind`, `--dev-bind-try`, `--ro-bind`, `--ro-bind-try`, `--bind-fd`, `--ro-bind-fd`, `--remount-ro`, `--exec-label`, `--file-label`, `--proc`, `--dev`, `--tmpfs`, `--mqueue`, `--dir`, `--file`, `--bind-data`, `--ro-bind-data`, `--symlink`, `--seccomp`, `--add-seccomp-fd`, `--block-fd`, `--userns-block-fd`, `--info-fd`, `--json-status-fd`, `--new-session`, `--die-with-parent`, `--as-pid-1`, `--cap-add`, `--cap-drop`, `--perms`, `--size`, and `--chmod`; source: `bwrap-help.txt:2-79`.

3. [X] Command: `bash docs/efforts/multi-agent-bench/evidence/research-insula/bwrap_probe.sh > docs/efforts/multi-agent-bench/evidence/research-insula/bwrap_probe.out 2>&1`; the root-tmpfs command template is stored in `root-tmpfs.argv.txt`; source: `docs/efforts/multi-agent-bench/evidence/research-insula/root-tmpfs.argv.txt:1-72`.

4. [X] Root-tmpfs assembly works with `--tmpfs /`, `--ro-bind /usr /usr`, symlinks `/bin`, `/lib`, `/lib64`, `/sbin`, minimal `/etc` files via fd-backed `--file`, `/proc`, `/dev`, tmpfs `/tmp`, `/run`, `/home`, a ro layer at `/cap/data/demo`, a rw workspace at `/workspace`, `--clearenv`, and explicit env; source: `bwrap_probe.sh:17-51`.

5. [X] That assembly ran `sh`, `python3`, and `jq` successfully, read the mounted layer, wrote `/workspace/write-test`, failed to write `/usr/bin/insula-write-test` with `Read-only file system`, hid `/home/philip.yang`, had zero routes, and showed five processes; output excerpt: `Python 3.11.2`, `jq-ok`, `workspace_write=workspace-ok`, `usr_write=blocked:... Read-only file system`, `host_home_visible=no`, `route_lines=0`, `ps_count=5`; source: `bwrap_probe.out:2-18`.

6. [X] `--clearenv` with explicit `--setenv` produced exactly `BENCH_ENV`, `HOME`, `LC_ALL`, `PATH`, and `PWD`; source: `bwrap_probe.out:20-26`.

7. [X] A nested ro mount under a ro-bound `/usr` works: `--ro-bind "$LAYER" /usr/local/share` allowed reading `nested-layer-ok` and blocked writes; source: `bwrap_probe.sh:85-100` and `bwrap_probe.out:28-31`.

8. [X] A single file can be ro-bound onto `/ld-linux-x86-64.so.2`; source command uses `--ro-bind /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2 /ld-linux-x86-64.so.2`, output `loader_file_bind=ok`; source: `bwrap_probe.sh:102-116` and `bwrap_probe.out:33-35`.

9. [X] `--unshare-net` starts with no route, but after `ip link set lo up`, IPv4 and IPv6 loopback sockets work while non-loopback `198.51.100.1:9` fails; source: `bwrap_probe.sh:118-174` and `bwrap_probe.out:37-42`.

10. [X] `--dev-bind-try /dev/nvidia*` is accepted even when no matching device exists; source: `bwrap_probe.sh:176-194` and `bwrap_probe.out:44-45`.

11. [X] Strict `--dev-bind /dev/nvidiactl /dev/nvidiactl` fails when the device node is absent; command saved in `nvidia-devbind-missing.out`, output `Can't find source path /dev/nvidiactl`; source: `docs/efforts/multi-agent-bench/evidence/research-insula/nvidia-devbind-missing.out:1-2`.

12. [X] `--new-session` made the process session id equal its pid, output `pid=59 sid=59 same=True`; source: `bwrap_probe.out:47-49`.

13. [X] `--cap-drop CAP_NET_RAW` under `--unshare-user` left effective capabilities at zero in this probe; source: `bwrap_probe.out:51-53`.

14. [X] `--seccomp 9` and `--add-seccomp-fd 9` accepted a hand-written cBPF allow-all program from fd 9 and executed a shell command; source: `bwrap_probe.sh:218-248` and `bwrap_probe.out:55-61`.

15. [I] The builder should treat `--dev-bind` as a required-device assertion and `--dev-bind-try` as optional pass-through; this distinction matters because GPU task scheduling should fail early when the required nodes are absent.

## 4. Verified Root-tmpfs Assembly

1. [X] The verified base argv starts with `bwrap --tmpfs / --unshare-user --unshare-ipc --unshare-pid --unshare-net --unshare-uts --unshare-cgroup-try --hostname corenius-bench`; source: `root-tmpfs.argv.txt:1-15`.

2. [X] The verified host userland mapping is `--ro-bind /usr /usr` plus `--symlink usr/bin /bin`, `--symlink usr/lib /lib`, `--symlink usr/lib64 /lib64`, and `--symlink usr/sbin /sbin`; source: `root-tmpfs.argv.txt:16-28`.

3. [X] The verified minimal `/etc` includes `/etc/ld.so.cache`, `/etc/ssl/certs`, fd-provided `/etc/passwd`, `/etc/group`, and `/etc/nsswitch.conf`; source: `root-tmpfs.argv.txt:29-42` and `bwrap_probe.sh:55-58`.

4. [X] The verified runtime mounts private `/proc`, `/dev`, `/tmp`, `/run`, `/home`, creates `/cap` and `/cap/data`, ro-binds a layer at `/cap/data/demo`, bind-mounts the workspace read-write at `/workspace`, clears env, sets a five-key allowlist, and chdirs to `/workspace`; source: `root-tmpfs.argv.txt:43-72`.

5. [X] The proof outputs are: `sh=/usr/bin/sh`, `Python 3.11.2`, `jq-ok`, `cap_layer=nested-layer-ok`, `workspace_write=workspace-ok`, `usr_write=blocked:... Read-only file system`, `home_entries=bench,`, `host_home_visible=no`, `route_lines=0`, and `ps_count=5`; source: `bwrap_probe.out:2-18`.

6. [X] The environment proof output contains only `BENCH_ENV=allow`, `HOME=/home/bench`, `LC_ALL=C`, `PATH=...`, and `PWD=/workspace`; source: `bwrap_probe.out:20-26`.

7. [I] For the benchmark, this root-tmpfs pattern is better than ro-binding `/` because it lets the builder create only the synthetic paths it intends to expose and avoids accidental host home or cache visibility.

8. [I] A production probe should add explicit checks for missing host `/etc` essentials, expected tool paths, `/proc/1/comm`, `/proc/net/dev`, route table, and path provenance from `/proc/self/mountinfo`.

## 5. GPU Boundary

1. [X] Command: `bash docs/efforts/multi-agent-bench/evidence/research-insula/gpu_probe.sh > docs/efforts/multi-agent-bench/evidence/research-insula/gpu_probe.out 2>&1`; the script discovers driver libs with clean `env -i PATH=/usr/sbin:/usr/bin:/sbin:/bin /sbin/ldconfig -p`, binds host `nvidia-smi`, binds driver libs under `/run/driver/lib`, and uses `--dev-bind-try` for NVIDIA devices; source: `gpu_probe.sh:10-63`.

2. [X] Clean `ldconfig -p` listed `libnvidia-ptxjitcompiler.so.1`, `libnvidia-nvvm.so.4`, `libnvidia-nvvm.so`, `libnvidia-ml.so.1`, `libcudadebugger.so.1`, and `libcuda.so.1`; source: `gpu_probe.out:6-12`.

3. [X] Device path check found missing `/dev/nvidiactl`, `/dev/nvidia-uvm`, `/dev/nvidia-uvm-tools`, `/dev/nvidia-modeset`, `/dev/nvidia0`, `/dev/nvidia-caps`, and `/dev/infiniband`; command and output are saved in `docs/efforts/multi-agent-bench/evidence/research-insula/gpu-device-proc.out:1-8`.

4. [X] `/proc/driver/nvidia` exists on the host and remains visible under a bwrap `--proc /proc` mount; commands and output are saved in `docs/efforts/multi-agent-bench/evidence/research-insula/gpu-device-proc.out:10-54`.

5. [X] `nvidia-smi -L` failed on the host with exit 9 and the same message inside bwrap: `NVIDIA-SMI has failed because it couldn't communicate with the NVIDIA driver`; source: `gpu_probe.out:1-4` and `gpu_probe.out:14-18`.

6. [X] `nvcc` from host CUDA 13.2 compiled a tiny CUDA program inside bwrap, but runtime device discovery failed with `cudaGetDeviceCount=-1 err=100 no CUDA-capable device is detected`; source: `gpu_probe.sh:65-112` and `gpu_probe.out:20-24`.

7. [R] RTO's allowlist expects NVIDIA devices `/dev/nvidiactl`, `/dev/nvidia-uvm`, `/dev/nvidia-uvm-tools`, `/dev/nvidia-modeset`, `/dev/nvidia[0-9]*`, `/dev/nvidia-caps`, `/dev/infiniband`; driver libs `libcuda.so.1`, `libnvidia-ml.so.1`, `libnvidia-*.so.*`, `libcudadebugger.so.1`; and host `nvidia-smi`; source: RTO `lab/agent-harness/rootfs_plan.json:2-21` and `lab/agent-harness/rootfs_plan.json:70-80`.

8. [R] RTO's driver discovery uses `ldconfig -p`, records both matching sonames and resolved matching files, and scans common lib dirs as fallback; source: RTO `rootfs/insula.py:807-838`.

9. [I] For `nvidia-smi -L`, bind the device nodes, `libnvidia-ml.so.1` plus resolved driver library targets, host `nvidia-smi`, and keep `/proc/driver/nvidia` available through private proc; for CUDA programs, also bind `libcuda.so.1`, JIT/NVVM/debugger driver libs, CUDA toolkit if compiling, and set `LD_LIBRARY_PATH` to the driver/toolkit mounts.

10. [I] The current host cannot prove GPU-positive behavior, so GPU benchmark tasks need a preflight that requires `nvidia-smi -L` success both outside and inside the capsule before accepting GPU results.

## 6. Toolchain Layers

1. [X] Command: `bash docs/efforts/multi-agent-bench/evidence/research-insula/toolchain_probe.sh > docs/efforts/multi-agent-bench/evidence/research-insula/toolchain_probe.out 2>&1`; the base args are saved in `toolchain-base.argv.txt` and include root-tmpfs, ro `/usr`, `/workspace` rw, tmpfs `/home`, `--unshare-net`, `--clearenv`, fixed `HOME`, `PATH`, and `LC_ALL`; source: `toolchain_probe.sh:7-40` and `toolchain-base.argv.txt:1-40`.

2. [X] Cargo and rustc worked inside bwrap offline when `~/.cargo`, `~/.rustup`, `/nix`, and the Spack GCC opt root were ro-bound, with `CARGO_NET_OFFLINE=true`, `CARGO_TARGET_DIR=/workspace/cargo-target`, and `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` set to the Spack GCC path; output excerpt: `cargo 1.95.0`, `rustc 1.95.0`, `Finished`, `cargo hello`; source: `toolchain_probe.sh:54-80` and `toolchain_probe.out:2-8`.

3. [X] Host Rust layer sizes are `~/.cargo` 287M, `~/.cargo/bin` 56M, `~/.cargo/registry` 231M, and `~/.rustup` 18G; source: `toolchain-sizes.out:11-12` and `toolchain-sizes.out:31-33`.

4. [I] A content-addressed Rust layer should copy only the selected rustup toolchain, cargo binaries needed for the task, and the offline crate registry/cache required by locked tasks; binding the full 18G `~/.rustup` is acceptable for probe speed but should be labelled `host-ro non-hermetic` if used directly.

5. [X] uv worked inside bwrap with `--unshare-net` and `UV_OFFLINE=1` when `/nix` and `~/.local/bin/uv` were ro-bound and `UV_CACHE_DIR` pointed at a writable workspace cache; output excerpt: `uv 0.12.17` and `uv hello`; source: `toolchain_probe.sh:88-101` and `toolchain_probe.out:10-13`.

6. [X] Host uv sizes are `~/.local/bin/uv` 48M, `~/.cache/uv` 11G, and `~/.local/share/uv` 367M; source: `toolchain-sizes.out:13-15`.

7. [I] uv should be a sealed layer built from `uv.lock` plus interpreter identity and a curated wheel/cache subset; direct binding of `~/.cache/uv` is too large and cache-poisoning-prone for deterministic benchmark runs.

8. [X] cmake+ninja worked from the ro-bound host `/usr` without extra toolchain binds; output excerpt: `cmake hello`, `cmake version 3.25.1`, `1.11.1`; source: `toolchain_probe.sh:103-121` and `toolchain_probe.out:15-19`.

9. [I] cmake+ninja can live in the base rootfs or a small build-tools layer rather than a per-task host bind, provided the compiler/linker selection is explicit.

10. [X] Spack GCC 13.4 worked inside bwrap when `/nix` and the Spack opt tree `/data02/home/philip.yang/devx/spack-v1.2.2/opt/spack/linux-icelake` were ro-bound; output excerpt: `gcc (Spack GCC) 13.4.0` and `spack gcc hello`; source: `toolchain_probe.sh:129-148` and `toolchain_probe.out:21-24`.

11. [X] The active `gcc` resolves to `/data02/home/philip.yang/devx/spack-v1.2.2/opt/spack/linux-icelake/gcc-13.4.0-lxsj2ofeq3zfis76q3onmwinqh4kxwsq/bin/gcc`; source: `toolchain-sizes.out:23-25`.

12. [I] Spack GCC needs its package prefix plus dependent runtime libs, which in this environment required `/nix`; copying the prefix plus referenced runtime closure into a layer is more honest than binding `/nix` silently.

13. [X] CUDA toolkit `/usr/local/cuda` resolves to `/data00/home/zfc/cuda_13_2` and is 7.1G; source: `toolchain-sizes.out:19-21`.

14. [I] CUDA should initially be a `host-ro` layer labelled non-hermetic, because the toolkit and driver stack are host-coupled; a baked CUDA layer is a later milestone once a positive GPU machine can verify runtime.

15. [X] Blade tool locations are large: `/opt/tiger/typhoon-blade` 4.6G and `/opt/tiger/ss_bin` 614M; Python 2.7 pieces measured at `/usr/bin/python2.7` 3.7M, `/usr/lib/python2.7` 107M, and `/usr/local/lib/python2.7` 29M; source: `toolchain-sizes.out:16-17` and `toolchain-sizes.out:26-29`.

16. [X] Command: `timeout 90s bwrap --ro-bind / / --dev-bind /dev /dev --bind <evidence>/blade-scratch <evidence>/blade-scratch --tmpfs /home --tmpfs /tmp --unshare-net --clearenv --setenv HOME <scratch>/home --setenv PATH /opt/tiger/ss_bin:/usr/local/bin:/usr/bin:/bin --setenv USER bench --chdir <scratch>/project /opt/tiger/ss_bin/blade build //:hello`; output showed Python 2.7 dependency checks followed by pip attempts against `http://bytedpypi.byted.org/simple`, repeated `Network is unreachable`, and `outer_exit=124`; source: `blade-fresh-home.out:1-14`, `blade-fresh-home.out:21-31`, and `blade-fresh-home.out:117-137`.

17. [R] Existing benchmark decisions already established that real Blade 1.2.2 can build a minimal target offline with a cleared env in about 3 seconds, that ambient `LIBRARY_PATH` breaks bundled gcc, that `hdrs=` is rejected by `cc_library`, that linked binaries request `/ld-linux-x86-64.so.2`, and that network-enabled Blade leaked build metadata; source: `docs/efforts/multi-agent-bench/decisions/20260929-probe-findings-blade-and-insula.md:6-37`.

18. [R] Existing benchmark decisions also established that versioned `cpp3rdlib/x:ver@//cpp3rdlib/x:x` labels can resolve from local vendored package directories offline, but real protobuf/gtest are not reproduced; source: `docs/efforts/multi-agent-bench/decisions/20260929-blade-3rdlib-resolves-from-local-packages.md:6-28`.

19. [I] Blade needs a content-addressed toolchain layer that includes `/opt/tiger/typhoon-blade`, `/opt/tiger/ss_bin`, Python 2.7 stdlib/site deps or a sealed warmed HOME, and a sysroot/loader path compatible with its linked outputs; running it from fresh tmpfs HOME is not sufficient on this host.

## 7. Layer Digest Algorithm

1. [I] Proposed canonical tree digest: walk `root.rglob("*")`, exclude known volatile names/suffixes such as `__pycache__`, `.pyc`, `.pyo`, `.DS_Store`, sort relative POSIX paths byte-stably, and feed records to SHA-256.

2. [I] Proposed record rules: directory `D <mode-octal> <rel>\n`; symlink `L <mode-octal> <rel> -> <link-target>\n`; regular file `F <mode-octal> <rel> <sha256(file-bytes)>\n`; other inode `O <type-octal> <mode-octal> <rel> <st_rdev>\n`.

3. [R] The proposed record rules intentionally match RTO's committed `canonical_tree_digest` structure for dirs, symlinks, regular files, and other inode types; source: RTO `rootfs/insula.py:167-189`.

4. [X] The demo implementation is in `digest_demo.py`, and it excludes `.DS_Store`, `__pycache__`, `.pyc`, and `.pyo`; source: `docs/efforts/multi-agent-bench/evidence/research-insula/digest_demo.py:1-48`.

5. [X] Command: `python3 docs/efforts/multi-agent-bench/evidence/research-insula/digest_demo.py docs/efforts/multi-agent-bench/evidence/research-insula/run-digest/a`; output was `sha256:19c68eecd776828d0a3c9ff77ce3474f679b750ae72d68b43bca1b83a795844b` in two consecutive runs; source: `digest_demo.out:1-4`.

6. [X] Command: `python3 .../digest_demo.py .../run-digest/b`; an equivalent second tree produced the same digest, and after `printf Hello > .../run-digest/b/sub/file.txt` the digest changed to `sha256:70bc2625adef12cb136f25daa3e771a0645ae9fe7f3585100c045698110b4e51`; source: `digest_demo.out:5-9`.

7. [I] Proposed JSON digest: serialize with sorted keys and compact separators, UTF-8 encode, then SHA-256; this keeps plan digests stable across formatting changes.

8. [R] RTO's JSON digest uses exactly sorted compact JSON; source: RTO `rootfs/insula.py:94-99`.

9. [I] Proposed host-provided layer digest: record `kind=host-ro`, host path, resolved path, device/inode metadata when available, size, relevant version command output, allowlisted files' tree/content digests when readable, and a warning field `hermetic=false`.

10. [I] Host-ro digests should never be used to claim byte-for-byte reproducibility; they should provide drift detection and scheduling compatibility only.

## 8. Worker Agent TraeCLI Capsule

1. [R] TraeX docs say to check `TRAE_HOME` and `TRAECLI_HOME`; unset `TRAECLI_HOME` commonly means `$TRAE_HOME/cli`, which stores plugin cache, skill cache, logs, sessions, memories, and CLI state; source: `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/paths.md:6-16`.

2. [R] TraeX docs list user config at `~/.trae/traecli.toml`, profiles at `~/.trae/<profile>.traecli.toml`, workspace resources at `<workspace>/.trae/`, runtime overrides via `-c key=value`, and env overrides including `TRAE_HOME`, `TRAECLI_HOME`, API endpoint variables, and auth-related environment variables; source: `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/paths.md:30-45`.

3. [R] TraeX docs list common user resources under `~/.trae/` as `traecli.toml`, `agents/`, `skills/`, and `rules/`; source: `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/paths.md:47-55`.

4. [R] TraeX docs list plugin cache under `$TRAECLI_HOME/plugins/cache/<marketplace>/<plugin>/<version>/`, marketplace temp under `$TRAECLI_HOME/.tmp/marketplaces/<marketplace>/`, user skills at `$TRAE_HOME/skills/<skill>/SKILL.md`, project skills at `<workspace>/.trae/skills/<skill>/SKILL.md`, plugin skills at `$TRAECLI_HOME/plugins/cache/.../skills/<skill>/SKILL.md`, and user hooks at `$TRAECLI_HOME/hooks.json`; source: `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/paths.md:120-151`, `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/paths.md:219-244`, and `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/paths.md:275-302`.

5. [R] TraeX settings docs list common top-level config keys including `model`, `model_provider`, reasoning/verbosity controls, approval policy, sandbox mode, writable roots, MCP, memories, skills, hooks, TUI, history, and `cli_auth_credentials_store`; source: `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/settings.md:17-45` and `/data02/home/philip.yang/.trae/plugins/cache/traex-bd-plugins/traex-guide/0.1.44+reinstall.379772.1789500675168549071.2.0/skills/traex-guide/references/settings.md:126-142`.

6. [X] Local no-content inventory showed `~/.trae/traecli.toml` exists mode 600 size 3989, `~/.trae/cli/auth.json` exists mode 600 size 628, and directories `~/.trae/cli/sessions`, `~/.trae/cli/memories`, `~/.trae/log`, `~/.trae/skills`, `~/.agents/skills`, and `~/.trae/plugins/cache` exist; command and output are saved in `docs/efforts/multi-agent-bench/evidence/research-insula/traecli-path-inventory.out:1-9`; no file contents were printed.

7. [X] `traecli --version` and `traecli exec --help` worked inside bwrap with tmpfs HOME; output included `traecli 0.207.1(internal edition)`, `Usage: traecli exec [OPTIONS]`, and a created tmpfs home tree `/home/bench/.trae/cli/tmp`; source: `traecli_tmp_home_probe.out:1-66`.

8. [X] `traecli --help`, `traecli acp --help`, and `traecli login --help` were captured without a model call; output lists commands including `exec`, `review`, `login`, `mcp`, `plugin`, `acp`, `sandbox`, `models`, `resume`, and `fork`; source: `traecli-help.out:1-41`.

9. [X] The root help invocation printed `WARNING: proceeding, even though we could not update PATH: Read-only file system (os error 30)`, so a capsule with no writable user state may still run help but triggers write attempts; source: `traecli-help.out:1-2`.

10. [R] `traecli` exposes `--remote-auth-token-env <ENV_VAR>`, `--profile`, `--sandbox`, `--add-dir`, `--ask-for-approval`, `--search`, `--allowed-tool`, and `--disallowed-tool`; source: `traecli-help.out:63-70` and `traecli-help.out:120-179`.

11. [I] Worker-agent capsule policy should allow network only for the TraeCLI process, bind only the worker worktree writable, set `HOME` to a writable tmpfs or per-run state dir, bind `~/.trae/traecli.toml` read-only if config is needed, bind `~/.trae/cli/auth.json` read-only only if file credential store is used, bind selected plugin/skill/cache directories read-only only when the task needs them, and keep host `$HOME`, `~/.ssh`, shell history, unrelated sessions, and global caches hidden.

12. [I] If the benchmark wants TraeCLI to persist per-run sessions or logs, it should point `TRAE_HOME`/`TRAECLI_HOME` at a per-run writable state mount, then copy in only minimal config/auth as files or explicit read-only binds.

13. [X] I did not run a model call; the executed TraeCLI probes were help/version only, as shown by `traecli_tmp_home_probe.out:1-66` and `traecli-help.out:1-41`.

## 9. Risks, Failure Modes, and Probes

1. [I] Stale layer risk: a selected layer path can change after `capsule.json` is built; probe by recomputing rootfs/layer/candidate/plan digests before every run and failing like RTO's `ensure_capsule_fresh`.

2. [R] RTO implements stale checks for rootfs digest, uv digest, uv input hashes, plan digest, dirty candidate sources, and rebuilt candidate digest; source: RTO `rootfs/insula.py:771-800`.

3. [I] Symlink escape risk: a layer path or selection file could point outside the store; probe by using `lstat` to reject symlink metadata before resolving and then requiring resolved paths under the declared store root.

4. [R] Corenius already rejects symlink rootfs paths and outside-store rootfs paths; source: `src/insula.rs:73-85` and `tests/insula.rs:31-51`.

5. [I] Host-path leakage risk: a bwrap plan can accidentally bind `/home`, `/data02`, `/workspace/<other>`, `/nix`, or a compiler prefix; probe by classifying `sys.path`, loaded modules, executable paths, and `/proc/self/maps` against mountinfo and allowlists.

6. [R] RTO's design describes mountinfo-based classification, but implementation currently uses prefix classification; source: RTO design `docs/design/2026-09-29-insula-capsule-spike.md:195-198` and RTO implementation `rootfs/probe_insula.sh:137-155`.

7. [I] `/proc` leak risk: omitting `--unshare-pid` or `--proc /proc` exposes host processes to tools like `ps` and py-spy; probe by checking `ps -e` process count, `/proc/1/comm`, and trying py-spy against a known child only.

8. [R] RTO explicitly warns never to commit py-spy output because dumps on a shared host contain other people's processes; source: RTO `AGENTS.md:119-120`.

9. [I] Network leak risk: ordinary benchmark tasks can fetch dependencies or call services if net is shared; probe by verifying route count, non-loopback connection failure, and loopback success.

10. [X] The root-tmpfs probe with `--unshare-net` got `route_lines=0`, loopback IPv4/IPv6 true, and non-loopback blocked; source: `bwrap_probe.out:37-42`.

11. [I] Shared-cache poisoning risk: direct rw binds of cargo, uv, npm, Blade, or compiler caches let one task affect another; probe by forcing caches under per-run state, hashing sealed layers, and rejecting writes to layer mounts.

12. [X] The root-tmpfs probe verified write-blocked `/usr` and a writeable `/workspace`, and the uv/cargo probes used writable caches/targets under `/workspace`; source: `bwrap_probe.out:7-8`, `toolchain_probe.sh:64-69`, and `toolchain_probe.sh:92-99`.

13. [I] bwrap availability risk: user namespaces or bwrap itself can be absent or disabled; probe with `bwrap --version`, then a minimal `--tmpfs / --unshare-user --unshare-pid --proc /proc /bin/true` smoke test before scheduling tasks.

14. [R] Corenius doctor already checks that `/usr/bin/bwrap` or `/bin/bwrap` exists before reporting `bwrap ok`; source: `src/insula.rs:291-301`.

15. [I] Environment contamination risk: variables such as `LIBRARY_PATH`, `PYTHONPATH`, `LD_PRELOAD`, token vars, or package-manager vars alter behavior; probe by running `env | sort` inside and comparing to the plan allowlist.

16. [X] The exact-env probe showed only five expected variables survived; source: `bwrap_probe.out:20-26`.

17. [R] The benchmark decision file records that `LIBRARY_PATH` from a Spack environment breaks bundled Blade GCC and that the capsule env allowlist must exclude it; source: `docs/efforts/multi-agent-bench/decisions/20260929-probe-findings-blade-and-insula.md:8-13` and `:17-23`.

18. [I] GPU false-positive risk: driver libraries can exist while device nodes are missing or the kernel driver is unusable; probe by requiring both host and inner `nvidia-smi -L`, `/dev/nvidia*` presence, `libcuda.so.1` load, and a CUDA runtime smoke test when a GPU task requires CUDA.

19. [X] This host showed exactly that false-positive shape: driver libs listed, `/proc/driver/nvidia` present, but `/dev/nvidia*` absent, `nvidia-smi -L` failed, and CUDA runtime found no device; source: `gpu_probe.out:6-24` and `gpu-device-proc.out:1-54`.

20. [I] Plan-digest drift risk: if the builder records only layer digests but not mount/env/net policy, two runs can have the same bytes but different authority; probe by hashing sorted plan JSON and stamping `plan_digest` into every result.

21. [R] RTO includes `plan_digest` in `capsule.json`, resolves placeholders at launch, and writes resolved env/mounts/bwrap args into plan evidence; source: RTO `lab/agent-harness/capsule.json:59-64` and RTO `rootfs/insula.py:1023-1034`.

22. [I] Worker-agent credential risk: a broad home bind leaks credentials and mutable history; probe by denying host home visibility, checking actual bind list, and confirming TraeCLI can start with tmpfs HOME plus explicit read-only config/auth files.

23. [X] The bwrap root-tmpfs probe hid host home directories, and the TraeCLI tmpfs-HOME help probe exited 0 while creating only `/home/bench/.trae/cli/tmp`; source: `bwrap_probe.out:9-10` and `traecli_tmp_home_probe.out:62-66`.

## Open questions

1. [I] Should Corenius keep the current Rust single-rootfs runtime as-is and build a separate Python capsule runner, or should the Rust runtime grow a compatible plan/capsule surface after the benchmark proves the model?

2. [I] Should shared content-addressed layer stores live under the repo, under `~/.cache/corenius`, or under a host-wide store with ownership checks?

3. [I] What is the minimum honest Rust layer: one selected rustup toolchain plus cargo registry for locked tasks, or a host-ro snapshot of `~/.cargo` and `~/.rustup` for v0 speed?

4. [I] What is the first machine with usable `/dev/nvidia*` where the GPU-positive probe can be run before accepting CUDA benchmark results?

5. [I] Should TraeCLI worker capsules persist sessions and memories as benchmark evidence, or should they run with per-run ephemeral CLI state and only retain harness-level transcripts?

6. [I] Should Blade get a sealed Python 2.7 dependency layer, a warmed read-only HOME, or a wrapper that disables dependency bootstrap before offline builds?

## Things I got wrong or could not check

1. [X] I could not verify a positive GPU run on this host because `/dev/nvidia*` and `/dev/infiniband` were absent, host and inner `nvidia-smi -L` failed, and the CUDA runtime returned `cudaErrorNoDevice`; source: `gpu_probe.out:1-24`.

2. [X] I did not run any networked fetch, package install, model call, `git fetch`, `git clone`, `pip install`, or `uv` install; the uv probe used `uv run --offline --no-project --python /usr/bin/python3 python hello.py`; source: `toolchain_probe.sh:88-101` and `toolchain_probe.out:10-13`.

3. [X] I did not inspect or print secret file contents; the TraeCLI inventory used `stat` and recorded only modes/sizes/existence for config/auth/state paths.

4. [X] I did not re-run the earlier successful warmed-state Blade build; I ran the safe fresh-HOME offline recipe and observed dependency-bootstrap retries ending in timeout; source: `blade-fresh-home.out:1-14` and `blade-fresh-home.out:117-137`.

5. [R] I corrected one RTO reading from the handoff: there is no `lab/agent-harness/AGENTS.md` at commit `584d9b8da62bc27e794076cff44d27b64f688ad9`; capsule terminology is in top-level `AGENTS.md`, while `lab/agent-harness/INSULA.md` documents the concrete agent-harness capsule.

6. [R] I corrected the stronger design claim that the committed RTO probe classifies by mountinfo: the design says mountinfo, but the committed `probe_insula.sh` classifies by path prefix.

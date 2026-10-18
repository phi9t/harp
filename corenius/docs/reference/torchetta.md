# Torchetta reference

- Repo: `/data02/home/philip.yang/workspace/torchetta` (remote `code.byted` = `git@code.byted.org:philip.yang/torchetta.git`, `git remote -v`)
- Commit read: `9322aa1e1392bf01643ab22741d5273771bebf0a` ("14: address review R1")
- Branch read: `hardening/32-b200-gpu-count-runs`, 16 commits ahead of `diziet/mainline` (`82b9b5f`), 0 behind (`git log diziet/mainline..HEAD`)
- Date: 2026-09-29
- Working tree at read time was DIRTY and changing under me: modified `scripts/smoke_ndtimeline_streams.py`, `scripts/verify_libtorch_runtime_bazel.py`, `torchetta/verifier.py`, `src/release.rs`, `tests/rust_cli_contract.rs`, `tests/test_verifier.py`, `tests/test_verifier_cli.py`; untracked `CLAUDE.md`, `.claude/`, `tests/test_smoke_ndtimeline_streams.py`. This is ticket 32 (`.scratch/harness-hardening/issues/32-*.md`, `Status: claimed`, `Claimed by: traecli GPT-5.5`). Statements below cite HEAD unless tagged `[dirty]`. `src/main.rs` and `src/release.rs` line numbers are HEAD-blob numbers; in the dirty tree they are shifted by a few lines (`git show HEAD:src/main.rs | grep -n ...` reproduces them).
- Method: read-only. Nothing built, run, or committed. No pytest/cargo run (the tree is being edited by another agent); test counts are quoted, not reproduced.

Labels used: no label = read directly, with path:line. `[inferred]` = my reasoning from what I read. `[quoted]` = taken from a doc or skill I did not re-verify against a run. `[unverified]` = I could not check.

Prior material reused and checked: `robust-training-observatory/docs/research/2026-09-28-training-loop-integration-source-map.md:28-36,137-194` and `CONTEXT.md:28,75`. Its Torchetta claims match the code I read (README:3, README:70, targets.tsv:2, smoke script). Its pinned HEAD `4343d43` is 12 commits stale; the CUDA 13.2 claim holds.

---

## 1. Problem, audience, pain

### What it is

A build-and-evidence harness for two ByteDance PyTorch forks, not the forks themselves.

> "Torchetta owns the build system and evidence; it does not own the libtorch source code." (`README.md:3-7`)

The forks live at `git@code.byted.org:bpt/libtorch.git`, branches `v2.7.0-training-serving` and `v2.10.0-training-serving`, pinned to commits `05aff68e...` and `bdace017...` (`targets.tsv:2-3`).

It turns "a pinned fork commit" into three things:
1. a wheel `torch 2.x.0+training.serving` built from source with CUDA 13.2 for `sm_80/90/100` (A100/H100/B200);
2. a shippable sandbox ("insula", a bwrap rootfs, ~4.4 GB) in which that wheel is the only place it is declared valid;
3. a directory of hash-referenced JSON evidence that says which gates passed.

### For whom

- Build/release engineers producing the `torch` wheel and rootfs bundle for SCM package `aml/lagrange/torchetta` (`docs/build-matrix.md:122-125`; `ultron/docs/workspace-repository-story.md:140`).
- Downstream training and serving stacks that need a torch ABI-compatible with native CUDA extensions: SGLang `sgl-kernel`, `flash-attn`, `flashinfer`, and others (`README.md:72-77`).
- Observability tooling (NdTimeline/ndtl) that needs the fork's NCCL stream-id accessors (`docs/research/2026-09-16-libtorch-cuda-ndtimeline.md:17-34`).
- Agents: the repo is effectively agent-authored. `git log`: 172 commits, all by one human identity, and all 172 carry a `Co-authored-by`/`Co-Authored-By` trailer naming TRAE CLI or GPT-5.5 via traecli (checked per commit with `git log --format=%b`).

### The pain it removes (concrete, from the repo's own failure log)

The build is not `pip install`. `docs/spec-libtorch-spack-build.md` is a catalog of the ways the fork fails to build on a modern toolchain:
- `packaging.version.InvalidVersion: '2.10.0-training-serving'` because the branch name is not PEP 440. Fix: Spack version stays the branch identity, `PYTORCH_BUILD_VERSION` becomes `2.10.0+training.serving` (`:270-284`).
- PyTorch found `/usr/bin/cmake` 3.25 instead of Spack's 3.31 because PATH order was wrong (`:286-301`).
- `TORCH_CUDA_ARCH_LIST=100` rejected; needs `10.0`. `CUDA_ARCH_NAME` was silently appended as `All` (`:303-318`).
- FBGEMM promoted GCC intrinsic warnings to errors (`:320-343`).
- GCC 12 lacks `<format>`, which DeepGEMM needs, so the rootfs moved to Ubuntu 24.04 + GCC 13 + Clang 18 (`:359-380`).
- The 2.7 branch needs eight recipe-time patches for CUDA 13 (CUB aliases, cuFFT enums, cudnn-frontend edge-data arg, TensorPipe CUDA IPC, RPC partial import) (`:435-647`; list at `docs/spec-custom-libtorch-verification.md:31-40`).
- The fork's own `byted_tools/build_scm_target.sh` hardcodes internal CI paths (`/opt/tiger`, `/usr/local/nccl`, `apt install`), so it is deliberately not used (`:79-84`).

Without Torchetta, each of these is rediscovered per person per machine. A wheel built any other way has no recorded provenance, and native consumers built against a different libtorch break at import or run time. The repo's stated ABI rule: prebuilt downstream extension wheels are "diagnostic only"; consumers must be source-built against this exact torch inside the matching insula (`README.md:166-169`).

There is a second, smaller pain: NdTimeline needs `ProcessGroupNCCL.get_coll_cuda_stream_id` / `get_p2p_cuda_stream_id`, which exist only in the fork. Torchetta is the place that proves a given wheel still has them (`scripts/smoke_ndtimeline_streams.py`, section 5).

---

## 2. Non-goals, boundaries, authority, dependents

### Deliberately not done

- Does not own libtorch source (`README.md:6-7`). Fork fixes happen upstream in `bpt/libtorch`; only compatibility shims live in the Spack recipe.
- Does not use the fork's own CI build script (`docs/spec-libtorch-spack-build.md:79-84`).
- Does not replace Spack with Bazel. Bazel is orchestration and verifier execution; Spack is "the only component that builds the external libtorch source" (`docs/spec-custom-libtorch-verification.md:189-192`; `docs/bazel-build-graph-architecture.md:207-208`). A pure-Bazel PyTorch build is a stated future direction, not a goal now (`:354-398`).
- Does not put consumer builds inside the torch package. Consumers are separate Spack specs `^byted-libtorch@...` so consumer rebuilds do not invalidate the torch wheel (`docs/agents/agentic-engineering.md:107-110`; `torchetta/targets.py:51-79`).
- Does not seal with cryptographic attestation. "Evidence-backed, not sealed with an extra cryptographic attestation layer" (`docs/spec-custom-libtorch-verification.md:104-110`).
- Does not make host-side runs count. "Treat build and verifier execution outside that rootfs as diagnostic only, not production evidence" (`AGENTS.md:143-146`).
- Kairo (the experiments tree) "is not a Torchetta release gate yet" (`docs/research/2026-09-18-kairo-thesis.md:85-87`).
- MLU targets: added then removed. `165ed1b Add first-class MLU build target support` was reverted wholesale in `cb15837` (2026-09-24: deleted `byted-torch-mlu` recipe, `spack/envs/torch-mlu-v2.*`, a 761-line research doc; `git show --stat cb15837`). Current `targets.tsv` has CUDA rows only. `distributed_timeline/docs/superpowers/research/2026-09-23-byted-torch-mlu-libtorch-ecosystem.md` still describes MLU rows and recipes that no longer exist at HEAD (stale sibling doc).

### What it treats as authority

| Authority | Where | Note |
|---|---|---|
| `targets.tsv` | `targets.tsv:1-3`; Python `torchetta/targets.py:97-118`; Rust `include_str!` at `src/targets.rs:30` | Branch, commit pin, wheel version, CUDA, arch, stream API names, SCM destination |
| Concretized Spack DAG | `spack/envs/libtorch-v2.*/spack.lock`; `tools/spack_packages.bzl` | `spack graph -d` output is required for production graph checks; recipe fallback is "diagnostic only" (`AGENTS.md:102-106`) |
| The rootfs contract | `torchetta/rootfs.py:14` (`schema_version=13`), `scripts/rootfs/build_rootfs.sh:10`, `enter_rootfs.sh:21` | Version-gated: `enter_rootfs.sh:527` refuses a rootfs with a mismatched contract |
| NVIDIA image `nvidia/cuda:13.2.1-cudnn-devel-ubuntu24.04` | `build_rootfs.sh:11` | Referenced by TAG, not digest (see 4 and 8) |
| Immutable source commits in evidence | `docs/spec-custom-libtorch-verification.md:104-110` | A released artifact lacking a recorded hash or source commit is "not hash-backed" |
| `CONSTITUTION.md` | `AGENTS.md:3-8` | Pinned "Clanker Constitution v2026.08.11" baseline; "direct user instructions and the more specific Torchetta guidance override it" |
| Spack owns native Python packages; uv owns only pure-Python overlay | `AGENTS.md:82-91`; `scripts/validate_uv_overlay.sh`; `scripts/uv_overrides.txt` | `torch`, `numpy`, `triton`, `nvidia-*` must never be installed by uv |

### What depends on it

- `ndtl/scripts/verify_cuda_torchetta_rootfs.sh` discovers the newest torch wheel under `$TORCHETTA_CACHE_ROOT/spack/opt/<target>/**/artifacts/wheels/torch-*.whl` (hardcodes `/data02/home/philip.yang/workspace/torchetta` as default `TORCHETTA_ROOT`) and runs `ndtl.verify cuda-collective --require-torchetta-rootfs` inside `enter_rootfs.sh`. `ndtl/src/ndtl/verify.py:66-69,385` records `TORCHETTA_IN_ROOTFS` and friends in its certification report.
- `vaso/experiments/spack-bazel-graph/native/pytorch/plan.py:289` copies Torchetta's build environment "verbatim" from `torchetta/build_environment.py` (a hand-copied duplicate, not an import).
- `ultron` lists Torchetta as a "supporting substrate" and speculates its pinned toolchains "could supply the `code_hash` and `container_hash` identity fields" (`ultron/docs/ultron-thesis.md:44`, labelled "Inference" there).
- Kairo model ladder inside this repo uses the rootfs to run TorchTitan and MILES (section 6).
- Nothing in `milano` or `arnold` references Torchetta (grep of `milano` for "torchetta" returned no lines; `arnold` has no matches).

---

## 3. Architecture

### Layers (top = operator, bottom = evidence)

```
  operator / agent
     |  scripts/*.sh (thin)      .claude/skills/run-torchetta/drive.sh
     v
  scripts/run_torchetta.sh ----> Rust CLI  src/main.rs (2950 lines), release.rs, commands.rs
     |                              |  subcommands: bootstrap concretize build inspect verify
     |                              |  verify-all prune  release{build,verify,package,validate,publish,rehearse}
     |                              v
     |                     Spack (unpinned develop) -- recipes in spack/repos/torchetta/packages/
     |                              |   byted-libtorch  ->  torch wheel  (python setup.py bdist_wheel)
     |                              |   10 native consumers, each ^byted-libtorch@... (pip wheel --no-build-isolation --no-deps)
     |                              v
     |                    per-package SEALS (torchetta/build_plan.py, tools/spack_build_plan.py)
     v
  scripts/rootfs/enter_rootfs.sh --> torchetta/rootfs.py --> bwrap (ro? no: rw repo bind, --unshare-all, --share-net)
     |
     v   runtime verifier (Python re-exec) -> generated Bazel workspace -> `bazel test //:runtime_probe_test`
  scripts/verify_libtorch_artifact.py -> scripts/verify_libtorch_runtime_bazel.py -> probes:
        import / CUDA / Dynamo-AOT + Inductor-Triton / C++ consumer / 2-rank NCCL stream ids / FSDP-DTensor / B200 live
     |
     v
  evidence dir: manifest.json (schema 4) + verification report (sha256 in manifest)
                + package-manifest.json + rootfs-bundle.tar.zst + native-consumer-source-builds/<target>/*.json
     |
     v   `torchetta release validate` (Rust, src/main.rs:1009)  ->  `release publish` (dry-run default) -> TOS / SCM
```

### Components and language split

- Rust CLI: `Cargo.toml:6-7` shows zero dependencies. It hand-rolls SHA-256 (`src/release.rs:67`, with known-answer tests `:877`) and scans JSON as strings (`json_top_level_string_field` `src/main.rs:1797`, `json_array_field` `:1581`). `docs/agents/agentic-engineering.md:61-67` names Rust the "deep module" for target metadata, Spack planning, artifact discovery, release manifests, and cleanup planning.
- Python `torchetta/`: verifier levels and runtime commands (`verifier.py`, 548 lines), rootfs plan (`rootfs.py`), Spack build planner (`build_plan.py`, 700), native-consumer install logic run inside Spack (`spack_native_consumer.py`, 1313), evidence checks (`kernel_torch_evidence.py`, `native_consumer_closure.py`), graph export (`spack_graph.py`), TOS publish (`tos_publish.py`).
- Bazel: `BUILD.bazel` + `tools/*.bzl`. `spack_pkg_*` filegroups mirror the Spack DAG (`tools/spack_packages.bzl`, 968 lines, generated); `real_spack_package_verifiers` verify per-package seals; `torchetta_spack_e2e` targets enter the rootfs and call the Rust CLI (`BUILD.bazel:98-130`).
- Spack: overlay repo with `byted-libtorch` plus native consumers, plus per-target envs (`spack/envs/libtorch-v2.{7,10}/spack.yaml`, `spack.lock`) and shared externals (`spack/common/packages.yaml`).
- Shell: thin wrappers; `scripts/common.sh` derives paths by calling `python3 -m torchetta.cache_layout --shell`.
- Kairo: `experiments/kairo/` (~7,000 lines Python; `wc` total 7003) is a research tree that ships in the same Python package (`CLAUDE.md:66-71`).

### Data flow for a release (CUDA target)

1. `release rehearse` (default: 2 CUDA targets x 2 iterations, each iteration in its own cache root and evidence dir; `README.md:61-69`; `src/main.rs:2361`).
2. Refuses a dirty tracked tree: `ensure_clean_source_tree` uses `git status --porcelain --untracked-files=no` (`src/main.rs:298-311`; called at `:846`, `:2364`). Untracked files do not block it `[inferred: they would not be recorded]`.
3. `concretize -f` then `spack install --fail-fast` of the generated env (`src/commands.rs:237-259`). The env is generated from the tracked template: placeholders replaced with cache paths, and the 10 consumer specs appended (`:151-222`).
4. `verify` runs `python3 scripts/verify_libtorch_artifact.py <branch> --json --level <L>` (`src/commands.rs:261-280`). The JSON stdout is written as `verification_report` and its SHA-256 goes into the manifest (`src/main.rs:855-860`).
5. Manifest entry per target with `release_scm_*`, `source_*` (Torchetta's own git repo/branch/commit), `upstream_source_*` (fork), `prefix`, `wheel`, `rootfs_bundle`, `verification_*`, and the ten `native-consumer-source-builds/...json` paths (`src/main.rs:867-905`).
6. `package` tars the rootfs with zstd into `rootfs-bundle.tar.zst` and writes a `package-manifest.json` (`src/main.rs:666-736`).
7. `validate` cross-checks (section 5), `publish` is a dry run unless `--publish-command <template> --execute` (`README.md:95-100`; `src/main.rs:952-1007`).

### Key abstractions and invariants

- Target (`torchetta/targets.py:18-49`): immutable row; `spack_root_spec` = `byted-libtorch@<ver> cuda_arch=80,90,100 target=x86_64`. Invariant: Spack `target=x86_64` (generic), not host microarch, so the artifact is portable with the rootfs (`docs/build-matrix.md:138-139`; `spack/envs/libtorch-v2.10/spack.yaml:19-21`).
- Verification level (`torchetta/verifier.py:26-105`, mirrored in `src/commands.rs:7-112`): a monotone ladder `artifact < runtime < cuda < compiler < distributed < cuda-b200-live`, with `cpp` a side branch = `runtime + cpp` only (`verifier.py:59`; `commands.rs:57-65`). Rust passes `--level <name>` to Python instead of expanding flags (ticket 03, commit `a5e1dcc`).
- Insula/rootfs contract: schema 13. The invariant is that evidence-producing runs observe `TORCHETTA_IN_ROOTFS=1` (`docs/spec-libtorch-spack-build.md:74-77`; env in `rootfs.py:75`).
- Seal (`build_plan.py:462-508`): one JSON per concrete Spack hash recording package spec, dependency hashes, prefix, exact `spack install` command, build log path, rootfs-contract digest, recipe digest, release source metadata, and native-consumer evidence classification. `seal_is_valid` (`:510-533`) rejects a seal whose concrete hash, dependency tuple, rootfs contract digest, package hash, or recipe digest changed. States: external / pending / blocked / sealed / failed / installed-unsealed (`.agents/skills/torchetta-spack-build/SKILL.md`; `.claude/skills/run-torchetta/SKILL.md:41`).
- Native-consumer evidence schema v2 (`docs/spec-libtorch-spack-build.md:798-812`): per consumer, source repo/ref/commit, Torch ABI (`_GLIBCXX_USE_CXX11_ABI`), compiler identity, CUDA library provenance, `ldd`/`readelf -d` per installed native object, import-smoke rows, and the policy string `source-built inside the matching insula Spack environment`. Two kinds: `linked-native` (compiled objects with passing linkage) and `source-jit-payload` (source shipped; not counted as ABI proof) (`torchetta/kernel_torch_evidence.py:10-11`; `docs/spec-libtorch-spack-build.md:837-865`).
- Payload identity vs adapter identity (`spec:814-835`): a torch-independent source/CUDA payload digest excludes the `byted-libtorch` hash; the torch-facing adapter digest includes it. Only `source-jit-payload` packages get cross-libtorch reuse today; linked-native are "torch-adapter-coupled" until recipes are physically split.
- Cache layout is computed, never hardcoded: `python3 -m torchetta.cache_layout --shell` (`scripts/common.sh:10-14`), mirrored in `src/cache_layout.rs`. Root: `${TORCHETTA_CACHE_ROOT:-${XDG_CACHE_HOME:-$HOME/.cache}/torchetta}` (`torchetta/cache_layout.py:12-17`).
- The rootfs bind layout (`torchetta/rootfs.py:bwrap_args`, `enter_rootfs.sh:256-330`): rootfs bind at `/`; the checkout bind-mounted at `/workspace/torchetta` and again at its host path; cache at `/cache/torchetta`; NVIDIA device nodes and driver libraries; `--unshare-all` then `--share-net` by default (`networked` default, `enter_rootfs.sh:8`); `--clearenv` then explicit `--setenv` list; optional ssh-agent socket passthrough for the private fork fetch.

---

## 4. Design decisions, trade-offs, lessons, known failures

### Decisions (with the recorded reason)

1. Spack builds native code; Bazel drives verification. Reason: "The Spack build works, so the next goal is not to replace it immediately"; Spack is first adapter, Bazel second (`docs/bazel-build-graph-architecture.md:206-208`). Trade-off: two graph representations that must stay aligned; drift is checked by `tools/export_spack_graph.py --check-bzl`, but see section 8.
2. Hermetic two-layer Python: pinned `python-build-standalone` CPython 3.11.16 (sha256 pinned, `build_rootfs.sh:16-18`) as base; uv only for Python-only overlay; Spack owns native packages. Reason: pip must never replace Spack-owned `torch`/`triton`/`nvidia-*` (`README.md:134-139`; `spack:220-237`).
3. Bazel owns the runtime probe workspace. `verify_libtorch_runtime_bazel.py` generates a temp Bazel workspace and runs `bazel test //:runtime_probe_test` so Bazel owns the venv, wheel install, import, CUDA, compiler, NCCL, and nested C++ consumer (`spec:697-706`). Bazel itself is shipped inside the rootfs (8.4.2, sha256 pinned, `build_rootfs.sh:19-21`).
4. Direct two-rank NCCL launcher, not `torchrun`. Reason: wheels are built without libuv; elastic rendezvous can request libuv even with `USE_LIBUV=0`, so the smoke spawns ranks itself with `tcp://...?use_libuv=0` (`AGENTS.md:148-151`; `docs/agentic-engineering-principles.md:65-69`).
5. `USE_TENSORPIPE=0` / `USE_RPC=0` for the 2.7 branch. Reason: TensorPipe CUDA IPC does not compile on CUDA 13 (`computeMode`), and env vars alone are insufficient because CMake defines `USE_RPC` unconditionally, leaving an `ImportError: _DEFAULT_NUM_WORKER_THREADS` at `import torch`. Recipe patches both CMake sites and the Python `is_available()` (`spec:563-609`). Same flags are in the shared base env for all targets (`torchetta/build_environment.py:14-15`).
6. CUDA arch `80,90,100` for everything, with FlashInfer needing `8.0 9.0a 10.0a` (`spec:196-203`); SGLang built with `SGL_KERNEL_ENABLE_SM100A=ON`, FA3 off (`torchetta/native_consumer_metadata.py:17-24`).
7. Consumers as separate Spack packages with `pip wheel --no-build-isolation --no-deps` (`spec:97-105`). Rejected: putting consumer builds inside `byted-libtorch` (`docs/agents/agentic-engineering.md:109-110`); prebuilt consumer wheels (diagnostic only, `README.md:166-169`).
8. Fail-closed for unmodeled kernels: MILES dependency scanning treats unknown CUDA kernel packages (`st-attn`, FA3/4, `flashinfer-cubin`, `quack-kernels`, ...) as required blockers until source-built or given separate runtime/JIT evidence (`docs/build-matrix.md:239-262`).
9. CCCL from the toolkit, never a separate checkout, for 2.7 CUB: a separately staged CCCL got past configure but failed with "version of CUB ... not compatible with this release of Thrust" (`spec:454-464`). A good rejected-alternative record.
10. Disabling FBGEMM to bypass the warning error was rejected: "would change the produced torch feature surface" (`spec:339-343`); instead `-Wno-error=maybe-uninitialized -Wno-error=uninitialized` (`torchetta/build_environment.py:38-41`).
11. `_GLIBCXX_USE_CXX11_ABI=1` fixed in the base env (`build_environment.py:22`) and recorded per consumer as `torch_abi`.
12. A GPU-count policy for the top level, set by the human on 2026-09-28: "cuda-b200-live (for 1GPU, 2GPU necessary), 4GPU and 8GPU are optional non-gating runs" (`.scratch/harness-hardening/map.md:76-77`). Implemented by ticket 32, in flight `[dirty]`.
13. Release validate requires level `cuda-b200-live` for CUDA targets, but `release build|verify` may run lower levels (for Bazel E2E) and print a note (ticket 12 decision, `.scratch/harness-hardening/map.md:73-75`). Not yet implemented in the validator (ticket 13 open); today `release_rehearsal_level()` returns the constant `"cuda-b200-live"` (`src/release.rs:187-190`).
14. Documentation split: durable intent in `.scratch/` tickets (gitignored, "Kata-shaped"), durable knowledge in `docs/` (`docs/agents/issue-tracker.md`; `docs/agents/agentic-engineering.md:183-192`).

### Lessons in the record (verified failures that shaped the harness)

- Version parsing, CMake discovery, arch format, warnings-as-errors, disk space (a native build failed unpacking cuDNN with ENOSPC; cleanup freed 117G to 407G on `/data02`, `spec:345-357`), compiler generation mismatch: all in section 1.
- Fork/NDTimeline mismatches recorded as verification obligations, not fixed here: the wrapper passes `DEVICE.index` to a binding that takes `at::Device` (`spec:413-419`; `distributed_timeline/bytedance/ndtimeline/stream.py:43-45` still does this, verified by `sed`); the FSDP2 NdTimeline patch drops the fork's 16-byte all-gather padding (`spec:421-427`); `GLOO_SOCKET_FAMILY` hard-fails on invalid values and should be in env snapshots (`:429-433`).
- The harness-hardening deep dive (2026-09-28, `.scratch/harness-hardening/spec.md:7-13`): "a dropped B200 flag, a probe that may import the wrong torch, a synthetic profiler trace accepted as real", template env used instead of the generated env, default rootfs used instead of the selected one, and duplicated logic drifting. Landed fixes (commits): generated env resolution `0ca12e4`; pre-commit desensitizer stages only the index blob `1b47d47`; exact level mapping `a5e1dcc`; runtime probe imports the wheel under test `f4d86b2`; Kairo rejects synthetic profiler traces `234172b`; verifier report archived + hashed in manifest `8152237`; Rust arg parsing `c0ee8fc`; publish placeholders shell-quoted with a single-pass renderer `90db2b1`/`fc0c8dc`; rehearsal packages rootfs once `3c114f1`/`9322aa1`.
- A separate test-env failure: PID-keyed `/tmp` paths collided because the executor sandbox runs in a PID namespace (ticket 33, `4343d43`; `protocol.md` "Environment facts").
- `/tmp` inode exhaustion (1M inodes at 100%) made bwrap fail with ENOSPC on `mkdir /tmp/.git`; launcher scripts now refuse above 90% (`.scratch/harness-hardening/protocol.md`, Environment facts).

### Open hardening tickets that describe known live defects (all `.scratch/harness-hardening/issues/`, `Status: open`)

- 13: `release validate` hardcodes `cuda=13.2` / `cuda_arch=80,90,100` (`src/main.rs:1081` `for required in ["cuda=13.2", "cuda_arch=80,90,100"]`), uses `.expect("CUDA targets require native consumer evidence")` (`:1090`, a panic for non-consumer targets), and checks the level and target strings by substring over the whole manifest (`:1073`).
- 15/16/17: bzl generator omits `NATIVE_CONSUMER_SPACK_TARGETS`; cutlass-dsl payload digest differs between lookup and store so it never cache-hits (and depends on a build-root path); uv overlay policy does not normalize PEP 503 names.
- 18: seals look for native-consumer evidence in the evidence dir but recipes write it in the install prefix, so `native_consumer_evidence_path` in a seal is always null at seal time (`build_plan.py:661-664` vs `spack_native_consumer.py:280`).
- 19: `installed_hashes()` swallows `CalledProcessError` (`build_plan.py:215-228`), so a broken Spack DB makes everything look pending; blocked plans print `built=none` and exit 0.
- 20-26: symlink-blind `rootfs_path_exists`; Rust target table hand-mirrored instead of generated; Kairo uses non-canonical target names; cache layout parity; core imports `experiments.kairo`; bwrap args duplicated in `enter_rootfs.sh` and `rootfs.py`.
- 29 (`ready-for-human`): floating inputs: Spack from `develop` (`scripts/bootstrap_spack.sh:8`; `src/commands.rs:231`), image by tag, nested triton by tag.

---

## 5. Build, test, release, verify: the trust model

### Build

- Prereqs: `uv`, `cargo`, `bwrap`, `docker` (to export the image), `nvidia-smi`, SSH access to `code.byted.org` (`.claude/skills/run-torchetta/SKILL.md:12-18` `[quoted]`).
- Entry: `scripts/bootstrap_spack.sh` (clones Spack `develop`), `concretize.sh`, `build_libtorch.sh`, `inspect_artifact.sh`, or `release build`. Production route is the per-package resumable planner: `python3 tools/spack_build_plan.py {status|preflight|run --resume}`. It does `spack install --only-concrete --only=package --keep-stage --dont-restage --keep-prefix /<hash>` per package, logs to `$CACHE/spack/seals/<target>/logs/<hash>.log`, and writes a seal on success (`build_plan.py:282-326`).
- Cost [quoted]: 3 min of the planner sealed 13/106 packages on a 216-core host; the full torch compile "takes hours" and was not run to completion by the skill's author (`SKILL.md:57-63`).
- Spack concretizer: `unify: true`, `reuse: false`, generic target, `build_jobs: 32`, `padded_length: 128` (`spack/envs/libtorch-v2.10/spack.yaml:9-25`). Native consumers are throttled to <=4 jobs, `NVCC_THREADS=1`, with SGLang's own `--threads=32` default overridden to 1 (`spec:867-877`).

### Test tiers

- Cheap gate `scripts/check_repo.sh` (re-execs under `uv run --frozen`): pytest over `tests/`, `cargo test`, `tools/desensitize_source.py --check --tracked`, `tools/check_agentic_workflow.py`, `check_libtorch_pin_drift.py --offline`, `check_source_patch_contract.py`, `py_compile` over all entrypoints and every Spack `package.py`, `bash -n` over every script, then `verify_all_artifacts.sh` (`check_repo.sh:21-101`). `AGENTS.md`'s gate list is a subset; the script is the source of truth (`CLAUDE.md:22-29`). `[quoted]` from the skill: 447 pytest passed, 2 skipped, 50 cargo tests, ~12 s (`SKILL.md:82-85`).
- Tests are "host-safe": no GPU, no Spack build, no rootfs entry, no network (`CLAUDE.md:86-88`; `.scratch/harness-hardening/spec.md:62-71`). Roughly half of `tests/test_*.py` are `test_kairo_*` `[inferred from filenames]`.
- A stated test discipline (hardening spec): behavior tests, not source greps. But `tests/rust_cli_contract.rs` is itself a source-text-grep test file the spec says not to extend (`spec.md:70-76`).
- Live verification ladder (through the rootfs): `artifact` (wheel structure, `METADATA` version, `libtorch*.so`, `ldd` closure), `runtime` (import wheel-under-test in a fresh venv, exact version), `cuda` (tensor op), `compiler` (Dynamo/AOT eager + Inductor/Triton CUDA inference), `cpp` (Bazel-built C++ transformer against the wheel, `ldd` proves `libtorch*`/`libc10*` come from the wheel), `distributed` (NCCL stream-id smoke + single-process FSDP/DTensor), `cuda-b200-live` (adds a live B200 `sm_100` capability check `probe_cuda_b200_live.py:25`) (`docs/spec-custom-libtorch-verification.md:42-83`).
- What the NCCL stream smoke actually checks: all-reduce, batched P2P, unbatched P2P, then `get_coll_cuda_stream_id(device)`, `get_p2p_cuda_stream_id(device, peer, 1|0)` all `>= 0`; `torch.device` first arg, not an int (`scripts/smoke_ndtimeline_streams.py` HEAD lines ~140-165). At HEAD it hardcodes 2 ranks (`git show HEAD:scripts/smoke_ndtimeline_streams.py`: `range(2)` at line 33, `WORLD_SIZE: "2"` at line 41). `[dirty]` version adds `--world-size N`, exit 77 skip when GPUs < N, and B200 capability assert `(10, 0)`.

### Release and evidence model

Artifacts of a release (directory `reports/releases/<name>/`, gitignored):
- `manifest.json` schema 4 (`src/main.rs:905`; doc `docs/build-matrix.md:315`): per-target entry, includes `verification_report` (relative path) and `verification_report_sha256`.
- `package-manifest.json` schema 1, `rootfs-bundle.tar.zst`.
- `native-consumer-source-builds/<target>/<consumer>.json` schema 2, x10.
- Verifier report JSON (per target).

`release validate` (`src/main.rs:1009-1135`) checks:
- files exist: manifest, package manifest, bundle, `etc/torchetta-rootfs-contract` in the rootfs;
- manifest `schema_version:4`, package `schema_version:1`, contract has a `schema_version=`;
- target strings appear somewhere in manifest or package text;
- verification report exists, is a relative path with no `..`, and its SHA-256 equals the manifest's (`:1137-1195`);
- target `components` contain `cuda=13.2` and `cuda_arch=80,90,100`;
- each native-consumer evidence path is referenced and the file exists, and its fields match the per-consumer contract: schema 2, `ok:true`, `spack_package`, `package`, `source_repo`, `source_ref`/`source_commit` (immutable), `build_command` matches the `pip wheel --no-build-isolation --no-deps` contract, policy string (`:1197-...`).

Separate Bazel-side check: `real_spack_kernel_torch_evidence` runs `torchetta/kernel_torch_evidence.py` over the release manifest, package manifest, native evidence, and per-package Bazel seal manifests. It requires `verification_level == cuda-b200-live` (`kernel_torch_evidence.py:11`), that the torch Bazel package is `sealed`, and that all ten managed consumers have package manifests, evidence, and a release-manifest reference (`:44-60`). `BUILD.bazel:61-92` binds this to a hardcoded release directory `reports/releases/staged-v210-20260922`.

Publication: `tools/publish_tos_release.py` uploads to a verified US TOS bucket only; a Singapore bucket with a similar id "must not be used" (`docs/tos-release-publishing.md:1-14`; `AGENTS.md:171-174`). The 4.1 GB rootfs is chunked into deterministic parts with a manifest recording each part's size and SHA-256 plus the original's; publish is complete only when no object is missing, no size mismatches, sizes sum, and the original SHA matches (`tos-release-publishing.md:39-43,90-98`). Auth: ByteCloud JWT from `bytedcli`, never TOS keys in files (`:15-28`).

### What this evidence does and does not prove (my assessment `[inferred]`)

Proves: a tracked-tree-clean checkout at a recorded commit built a wheel via a recorded Spack DAG inside a rootfs whose contract version/digest is recorded; the wheel imports and passes named probes; ten consumers were source-built with recorded source commits and linkage.

Does not prove:
- Signatures or non-repudiation: none (explicit non-goal). `release validate` checks hashes only for the verifier report. The wheel, the bundle, and native evidence files are checked for existence/fields, not content digests. `[inferred from src/main.rs:1009-1135; I found no wheel/bundle hash check there]`.
- Substring checks (`manifest_text.contains(required)`, `:1073`) can pass on the wrong field; ticket 13 exists for this.
- Reproducibility: Spack `develop`, image tag, and unpinned apt are floating (ticket 29). A rebuild can produce a different DAG. The committed graph already drifts from a fresh concretize: `drive.sh graph` reports about 18 packages with "stale Bazel Spack package edges" (e.g. committed graph builds `cmake` from Spack, current rootfs resolves it to `/usr`) `[quoted]` (`SKILL.md:95-102`).
- The staged v2.10 manifest that exists on disk (`reports/releases/staged-v210-20260922/manifest.json`) is `schema_version: 3`, has no `verification_report_sha256`, and has `"source_repo": "diziet/mainline"` (a branch name in a repo field). Current validator requires schema 4 (`src/main.rs:1046`), so `[inferred]` that release would not validate under HEAD without regeneration; I did not run validate.
- The NCCL stream-id smoke calls the backend directly, not the NdTimeline wrapper; the wrapper mismatch is a documented gap (`docs/research/2026-09-16-libtorch-c10d-cuda-accelerators.md:371-375`).

### Hooks and hygiene

- `.githooks/pre-commit` runs `tools/desensitize_source.py --fix --staged --stage`, which rewrites host home paths (`/data0N/home/<user>...`), non-documentation IPv4, MACs, GPU UUIDs, and hostnames matching `nNNN-NNN-NNN` to placeholders (`desensitize_source.py:40-60`). `check_repo.sh` runs the same in check mode over tracked files. Consequence: docs and tests must use placeholder paths. (This document uses real paths because it lives in a different repo; copying passages into Torchetta would trip the gate.)
- Hooks chain to any pre-existing `core.hooksPath` via `git config torchetta.chainedHooksPath` (`.githooks/run-chained-hook`, `scripts/install_git_hooks.sh:7-15`).

---

## 6. Interfaces with sibling repos

| Repo | Direction | Contract | Evidence |
|---|---|---|---|
| `distributed_timeline` (NdTimeline) | It consumes fork API | `ProcessGroupNCCL.get_coll_cuda_stream_id(device)`, `get_p2p_cuda_stream_id(device, peer, is_batched)`. Torchetta verifies them directly; NdTimeline's wrapper passes `DEVICE.index` (int) for P2P and is not exercised | `targets.tsv:1` `stream_api_names`; `distributed_timeline/bytedance/ndtimeline/stream.py:43-45` |
| `ndtl` | It calls Torchetta | `scripts/verify_cuda_torchetta_rootfs.sh` finds newest `torch-*.whl` (`-size +10M`) under `spack/opt/<target>`, installs it `--no-deps --force-reinstall` in a venv inside `enter_rootfs.sh`, runs `ndtl.verify cuda-collective --require-torchetta-rootfs`; report carries `torchetta_in_rootfs` etc. It does not go through a Torchetta release manifest | `ndtl/scripts/verify_cuda_torchetta_rootfs.sh:14-60`; `ndtl/src/ndtl/verify.py:66-69,381-400` |
| `torchtitan` | Torchetta's Kairo ladder drives it | `experiments/kairo/model_ladder/torchtitan_stage.py` runs `run_train.sh` with `MODULE`, `CONFIG`, `NGPU`, `CHECKPOINT_DIR`, `KAIRO_RAW_PROFILER_PATH`, `KAIRO_WORKLOAD_STAGE`; validates checkpoint; requires a real profiler trace (synthetic only with `--allow-synthetic-profiler` in dry-runs). Torchtitan source contains only staged copies of ndtl (`torchtitan/experiments/qwen3_4b_synthetic/synd-runs/*/staged_ndtl`), not Torchetta code | `torchtitan_stage.py:60-110`; grep over torchtitan |
| MILES / SGLang (not in the asked list, but the consumer workloads) | Consumers of the torch ABI | Ten managed CUDA consumers, `^byted-libtorch`; MILES scanner fails closed on unmodeled kernels | `docs/build-matrix.md:196-262` |
| `milano` | No direct interface | Sibling design of the same family. No file in Milano mentions Torchetta (grep empty). Overlap is conceptual: both use bwrap "insula", Spack-lock pinned targets, CUDA/NCCL, evidence gates. Differences below | see table below |
| `ultron` | It reads Torchetta as a substrate | Ultron labels the `code_hash`/`container_hash` mapping "Inference": no code contract | `ultron/docs/ultron-thesis.md:44`; `workspace-repository-story.md:102,140` |
| `vaso` | Copies Torchetta's build env; Vaso owns intended rootfs/execution policy | `native/pytorch/plan.py:289` re-encodes `BASE_ENVIRONMENT`. Ultron's audit calls Vaso the owner of sandbox-visible layout and Torchetta's external-asset move "a concrete adopter pattern" | `ultron/docs/research/workspace-focus-audit-2026-09-12.md:101-102,185-189` |
| `arnold` | None found | `[unverified]`: I searched for "torchetta"/"milano" in `arnold` markdown/all files and found nothing | - |
| `torchtitan` (again) | See above | No shared vocabulary beyond "checkpoint", "profiler trace" in Kairo | - |

### Shared vocabulary

- "insula": the bwrap rootfs sandbox that gates validity. Torchetta: each target "ships the matching bwrap rootfs sandbox as its `insula`" (`README.md:70-73`). Milano: "the local accelerator Insula" (`milano/AGENTS.md:13`). Same word, same idea, different implementation.
- "target": row in `targets.tsv` (`libtorch-v2.10`), aliases `v2.10`, `2.10`, full branch (`targets.tsv:3`; `.claude/skills/run-torchetta/drive.sh:17`).
- "release-valid" / "source-built inside the matching insula Spack environment": the exact policy string in evidence.
- "verifier level", "seal", "concrete hash", "native consumer", "linked-native" vs "source-jit-payload".
- "Kairo": Torchetta-side observability research vehicle ("start from a complete, queryable account of one training step", `kairo-thesis.md:9-16`); not a release gate.

### Torchetta vs Milano, side by side (same author, different generation of the idea)

| Aspect | Torchetta | Milano |
|---|---|---|
| Product | Custom-fork libtorch wheel + native consumers | In-tree `torch_accel` extension (`accel` PrivateUse1 device) |
| Language of orchestration | Rust CLI + Python + Bazel | `milano/` Python package + shell |
| NVIDIA stack | Spack `cuda@13.2` external from image; `nccl +cuda` constrained in Spack env (`spack/envs/libtorch-v2.10/spack.yaml:33-36`); `NCCL_HOME=/usr` in rootfs env (`rootfs.py:56`). Who provides NCCL is not stated unambiguously `[inferred; unresolved]` | Image ships everything; never built; "exactly one provider of each NVIDIA library" (`milano/AGENTS.md:98-106`) |
| Base image pin | Tag only (`build_rootfs.sh:11`) | Digest-pinned (`milano/AGENTS.md:98`) |
| Candidate under test | Working checkout bind-mounted read-write at `/workspace/torchetta` (`enter_rootfs.sh:263`) | Staged Git index archived and extracted read-only (`milano/AGENTS.md:27-30`) |
| Network in sandbox | Shared by default (`enter_rootfs.sh:8`, `rootfs.py:bwrap_args`) | Not documented in what I read |
| GPU concurrency | No lease mechanism found `[unverified]` | flock leases per GPU (`milano/AGENTS.md`, "GPUs are leased") |
| Multi-agent workflow | Ticket loop, `.scratch/`, branches `hardening/NN-slug`, stacked | `milano/scm.py` worktrees + `integrate` with serialized gates |
| CUDA | 13.2, `sm_80/90/100` | 13.0.2, `sm_100` |

The pattern to note: Torchetta's known defects (mutable checkout as candidate, floating base image, ambient network) are things Milano's launcher already fixed `[inferred: from the two AGENTS.md files]`.

---

## 7. Relevance to agentic engineering

### How agents work in it today

- Guidance files, layered: `AGENTS.md` (contract) -> imports `CONSTITUTION.md` (`AGENTS.md:3-8`); `CLAUDE.md` is `@AGENTS.md` plus extras (`CLAUDE.md:5-9`), and is untracked at HEAD (so a fresh clone lacks it). `docs/agents/*` (workflow, tracker, skill orchestration, triage labels, domain), `docs/agentic-engineering-principles.md`. Skills: `.claude/skills/run-torchetta/{SKILL.md,drive.sh}` (untracked) and `.agents/skills/torchetta-spack-build/SKILL.md` (tracked).
- Operating law: "Scale execution. Bound authority. Verify independently. Preserve intent." (`AGENTS.md:38`; `docs/agentic-engineering-principles.md:10`). Human owns merge, publish, destructive cleanup, high-cost GPU launch, release acceptance (`docs/agents/agentic-engineering.md:25-26,172-175`).
- Drift guard for the guidance itself: `tools/check_agentic_workflow.py` asserts required files contain required snippets (`:20-60`), and is part of `check_repo.sh:24`. Note: the required-snippet approach protects wording, not behavior `[inferred]`.
- Ticket ledger: `.scratch/<effort>/{map.md,spec.md,issues/NN-slug.md}`, lifecycle `open -> claimed -> resolved`, mandatory `## Answer` with commands, exit status, commit (`docs/agents/issue-tracker.md`). `.scratch/` is gitignored, so the ledger is not versioned with the code it governs.
- The Claude-reviewer / TRAE-executor loop is the most developed workflow (`.scratch/harness-hardening/protocol.md` v0.7):
  - reviewer (Claude Code) picks the lowest unblocked `ready-for-agent` ticket, writes the run prompt, reviews, and never writes product code;
  - executor (`traecli` GPT-5.5) does one ticket per run: failing test first, fix, `check_repo.sh`, commit `NN: title`, fill `## Answer`, fixed report headings;
  - supervisors: a model-free log watcher flags forbidden commands, protected-path edits, stalls; a small-model auditor scores each run against a fixed checklist (process only, "does not judge design");
  - rework is a new commit `NN: address review R1-Rn`, never amend; reviewer resumes the same executor session;
  - executor never blocks for an answer: it stops, leaves changes uncommitted, labels `ready-for-human`;
  - Retro after each five tickets; 8 tickets, 2 sent back, both for design issues an audit cannot see (unsafe default; untested hand-rolled hash).
- Authority boundary is written per effort: allowed edit surface; forbidden without a recorded human decision (pins, rootfs contract version, running Spack/GPU/release, `prune --execute`, publish, push/merge/history rewrite); always-allowed host-safe commands (`map.md:27-44`).
- Decision tickets: tickets of `Type: review` with options A/B are put to the human, who said "follow your recommendations"; the chosen option and its evidence are written into the ticket (tickets 05, 12, 25, 30).

### Mechanisms worth reusing in corenius

1. Evidence with a self-describing, hash-linked manifest plus a validator that fails closed (`src/main.rs:1009-1195`). Adopt the shape; fix the weaknesses in section 4 first (parse JSON properly; check every referenced file's digest).
2. A verification ladder with exact level semantics and an ordered gate list, and a rule that lower levels are allowed to run but cannot validate as release (ticket 12).
3. "Diagnostic only" flag for anything run outside the trust boundary, applied consistently in docs and code (`--no-rootfs`, host-side Spack, recipe fallback).
4. Per-unit seals keyed by content (concrete hash + recipe digest + rootfs contract digest) with an explicit `installed-unsealed` state that blocks dependents and needs an explicit `--adopt-installed` (`SKILL.md` for `torchetta-spack-build`).
5. Authority boundary written into each work item, plus "host-safe commands are always allowed" as the default agent lane.
6. Reviewer/executor split with process audit and independent design review; watcher for forbidden commands; report format with "Reviewer focus" and "Protocol feedback" headings; protocol as a living, versioned file with changelog and retros.
7. Regression-test-first rule: "a test that failed before your change passes after it" as the definition of done (`spec.md:15-16`), plus the rule to replace, not delete, source-grep tests.
8. Desensitizing pre-commit that rewrites only the staged blob (ticket 02), so agents cannot leak host paths, IPs, GPU UUIDs, hostnames into tracked text.
9. A drive script per repo as the agent front door with known-gotcha documentation (e.g. inherited `SPACK_USER_CACHE_PATH` beating the default; zsh not word-splitting) (`.claude/skills/run-torchetta/SKILL.md:87-123`).
10. Mirror-pair table (Python vs Rust, shell vs Python) that an agent must update together (`spec.md:49-60`; `CLAUDE.md:53-56`). Better: generate one side (ticket 21).
11. Computed environment: one Python module emits shell exports for every script (`common.sh`; `cache_layout --shell`), so paths are never hardcoded.

### Hazards for agents

- The tree is live-edited by another agent right now (dirty files above). Two agents in one checkout would test each other's changes; Torchetta has no worktree-per-task tooling like Milano's `milano.scm`, but the ticket loop uses stacked `hardening/NN-*` branches `[inferred]`.
- Costs: Spack builds take hours and 100s of GB; a full B200 rehearsal needs GPUs (8x B200 host cited in `SKILL.md:14`). No GPU lease mechanism found. An agent running `release rehearse` or `build_rootfs.sh` is a large, hard-to-reverse spend. The hardening spec forbids these explicitly (`spec.md:86-90`).
- `prune --execute`, deleting cache/rootfs/evidence state, and `--execute-publish` are human-only (`docs/agents/agentic-engineering.md:122-125`).
- A failed build's stage/seal/log state is evidence; "never delete ... to recover from a failed package" (`.agents/skills/torchetta-spack-build/SKILL.md`).
- Host vs rootfs cache-root confusion: the generated env has absolute paths from where it ran; switching sides breaks planner calls (`SKILL.md:110-115`). The rootfs also rewrites paths (`/cache/torchetta`, `/workspace/torchetta`).
- Inherited `SPACK_USER_CACHE_PATH` silently overrides the default and yields a stale spack-packages clone (`SKILL.md:89-93`).
- `TORCHETTA_RELEASE_SOURCE_*` env vars are trusted for provenance when `TORCHETTA_RELEASE_ROOTFS_REEXEC` is set (`src/main.rs:280-292`), so an agent or wrapper that sets them can label evidence with any repo/branch/commit `[inferred]`.
- Confusing scale of doc drift: read the code, not only the docs (section 8).
- Real-world credentials: TOS uploads use a JWT from the local `bytedcli` cache; agents must not paste tokens (`docs/tos-release-publishing.md:15-28`; `AGENTS.md:166-168`). Wrong region bucket is a documented trap.
- Attribution: nearly every commit has TRAE trailers; the human is sole author. Co-author policy for Claude-written commits would be a new decision `[inferred]`.

---

## 8. Open problems, debt, uncertainties

### Correctness/trust debt (known, tracked)

- Tickets 13, 15-24, 26-31 open (see section 4). Ticket 32 in flight `[dirty]`. `map.md` / ticket headers at read time: of 33 tickets, 16 `resolved` (including the 4 decision tickets 05, 12, 25, 30), 16 `open`, 1 `claimed` (32).
- Validator uses substring scans and hand-parsed JSON in a zero-dependency Rust binary; `release validate` never hashes the wheel or bundle `[inferred]`.
- Floating inputs (ticket 29): Spack `develop`, image tag, unpinned apt, nested triton by tag. `padded_length: 128` warning appears with long cache roots (`SKILL.md:103-106`).
- Committed `tools/spack_packages.bzl` drifts from the live DAG (~18 stale edges) `[quoted]`. Regenerating it is a production graph-contract change (`SKILL.md:99-102`).
- Seals do not cover the prefix contents or wheel hash `[inferred from build_plan.py:462-508: fields are spec, deps, prefix path, command, log path, digests of contract and recipe]`.
- No signing (explicit, `spec-custom-libtorch-verification.md:104-110`).

### Duplication (drift risk)

- Consumer versions live in three places: `torchetta/targets.py:51-79`, `src/targets.rs:169-193`, and doc text (`docs/build-matrix.md:200-208`). Not in `targets.tsv`. The doc pins `torch-memory-saver` by commit `0f3d6ede...` while code specs `torch-memory-saver@0.0.9` `[inferred: a version vs a commit; may be consistent via the recipe, I did not open the recipe]`.
- `torchetta/verifier.py` vs `src/commands.rs` levels; `rootfs.py` vs `enter_rootfs.sh` bwrap args (ticket 26); `build_rootfs.sh:10` and `enter_rootfs.sh:21` both hard-code contract version 13 next to `rootfs.py:14`.
- `docs/spack:184` says "cuDNN/NCCL: built by Spack unless suitable local externals are added" while the environment treats CUDA/cuDNN as image externals; `docs/spec-libtorch-spack-build.md:179` says CMake 3.28 while `docs/build-matrix.md:159` says 3.25 and `spack/common/packages.yaml` says `cmake@3.28.3`; release manifest "schema version 3" at `spec:774` vs "schema-v4" at `build-matrix.md:315` and code `src/main.rs:905`. Ticket 27 (docs drift) is open.
- `docs/agents/agentic-engineering.md:192` points to `lessons/*.org`, which does not exist in the repo (`ls lessons` fails). `docs/agents/skill-orchestration.md` says avoid `docs/superpowers/plans`, yet five exist there.
- `.claude/skills/run-torchetta/SKILL.md:55` points to `.agents/skills/torchetta-spack-build/SKILL.md` (exists), fine, but both skill trees mix two conventions (`.claude/` vs `.agents/`).

### Repo-shape hazards

- `BUILD.bazel:22-92` embeds one release's directory `reports/releases/staged-v210-20260922` and `/cache/torchetta/spack/checkout/bin/spack`; `reports/releases/` is gitignored, so `exports_files` names untracked paths. A fresh clone cannot build those targets. `[inferred]`
- `CLAUDE.md` and `.claude/` untracked; `.scratch/` (the whole ticket ledger and protocol) gitignored. Losing the working directory loses the intent history.
- Kairo (~7k lines) shares the package and gate with the release harness; ticket 24 (core imports Kairo) and 22 open. Its thesis is explicitly research ("may patch PyTorch, NCCL ... when public APIs hide the facts", `kairo-thesis.md:37-41`), which pulls against the harness's "bound authority" ethos `[inferred]`.
- Release-tree state is heavy: a staged v2.10 evidence dir on disk is ~4.3 GB (`rootfs-bundle.tar.zst`, `ls -la`); `release rehearse` writes 8 rehearsal dirs already under `reports/releases/rehearsals/`.
- The Rust CLI is one 2,950-line `main.rs` with the validator's string-scanning helpers inside it.

### Uncertainties I could not resolve

- Whether the CUDA image's NCCL or a Spack-built NCCL ends up loaded by the wheel in production; `NCCL_HOME=/usr` and Spack `nccl+cuda` both appear. Milano's rule ("exactly one provider") is not documented as a Torchetta invariant.
- Whether the staged v2.10 evidence (schema 3) was ever accepted by a validator of its time, or published as such. TOS doc names prefix `staged-v210-20260922`, matching the directory; I did not contact TOS.
- Whether any Torchetta wheel has actually passed `cuda-b200-live` at a recent commit. `docs/build-matrix.md:266-279` reports pass results as of 2026-09-09 for both targets (import, CUDA, C++ compile, 2-rank NCCL). No later dated pass is recorded in the docs I read. The hardening `map.md:61-63` says live evidence for gates 2-4 "is a follow-up that requires a human to schedule".
- Whether `distributed_timeline` was updated after `distributed_timeline/docs/.../2026-09-23-...` to stop assuming MLU rows.
- Test suite health at HEAD: not run (dirty, concurrently edited tree). `[quoted]` numbers are from a skill file that is untracked and of unknown date.
- Arnold: no interface found; may exist under other names.

---

## How to verify this document

1. Commit and dirtiness: `git -C /data02/home/philip.yang/workspace/torchetta rev-parse HEAD` should print `9322aa1e...`; `git status --short` shows the dirty/untracked files listed at the top (they may have moved on; ticket 32 is live).
2. Targets and pins: `cat targets.tsv` (2 rows, CUDA 13.2, `80,90,100`, stream API names, commits `05aff68e`, `bdace017`).
3. No Rust deps and hand-rolled hashing: `cat Cargo.toml`; `sed -n 67,80p <(git show HEAD:src/release.rs)`; `grep -n "fn json_top_level_string_field" src/main.rs`.
4. Validator behavior (HEAD blob): `git show HEAD:src/main.rs | sed -n 1009,1135p` (schema 4 at 1046, substring check 1073, hardcoded CUDA strings 1081, `.expect` 1090); `git show HEAD:src/main.rs | sed -n 1137,1195p` (report sha256 check).
5. Manifest fields: `sed -n 862,910p src/main.rs`.
6. Level ladder and the `cpp` side branch: `sed -n 53,75p torchetta/verifier.py`; `sed -n 57,75p src/commands.rs`.
7. Seals: `sed -n 462,535p torchetta/build_plan.py`; `sed -n 215,228p torchetta/build_plan.py` (swallowed error).
8. Rootfs contract and bwrap plan: `sed -n 12,20p torchetta/rootfs.py`; `sed -n 256,330p scripts/rootfs/enter_rootfs.sh`; `sed -n 8,12p scripts/rootfs/build_rootfs.sh` (tag not digest).
9. Floating pins: `grep -n "develop" scripts/bootstrap_spack.sh src/commands.rs`.
10. MLU removal: `git show --stat cb15837 | head -40`; `git log --oneline --all -i --grep=mlu`.
11. Agent-authorship: `git log --format=%b | grep -ci "co-authored"` (172) and `git log --format=%an | sort | uniq -c`.
12. Hardening ledger and protocol (untracked, may vanish): `ls .scratch/harness-hardening/issues`; `sed -n 1,60p .scratch/harness-hardening/map.md`; `sed -n 1,60p .scratch/harness-hardening/protocol.md`.
13. Smoke script HEAD vs dirty: `git show HEAD:scripts/smoke_ndtimeline_streams.py | grep -n "range(2)"` versus `git diff scripts/smoke_ndtimeline_streams.py`.
14. Sibling contracts: `sed -n 14,60p /data02/home/philip.yang/workspace/ndtl/scripts/verify_cuda_torchetta_rootfs.sh`; `sed -n 40,46p /data02/home/philip.yang/workspace/distributed_timeline/bytedance/ndtimeline/stream.py`; `grep -rIn -i torchetta /data02/home/philip.yang/workspace/milano --exclude-dir=.git --exclude-dir=target` (empty).
15. Staged evidence artifact: `head -12 reports/releases/staged-v210-20260922/manifest.json` (schema 3, `source_repo` is a branch name).
16. Doc drift: `grep -n "cmake@\|CMake 3" docs/*.md spack/common/packages.yaml`; `ls lessons`.
17. Run the gate yourself, on a clean worktree only: `./scripts/check_repo.sh` (cheap, host-safe). I did not.

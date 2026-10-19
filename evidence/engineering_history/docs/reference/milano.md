# Milano: reference for corenius

> **Verification (2026-09-29):** independently checked by a separate agent against `ed218ae`
> (`milano-verification.md`, 59 claims: 53 confirmed, 3 wrong, 1 stale, 2 not verifiable from source).
> Corrected here: the extracted candidate tree is a writable tmpfs, not read-only, so trust rests on the
> candidate digest, not immutability of `/workspace/milano`; the inside-rootfs `run-candidate` checks
> absolute paths, `..` and special files but not symlink/hardlink targets (host-side `candidate.py` does);
> the release test venv also installs unpinned `pytest`, `setuptools`, `ninja`. Stale: the rootfs still
> ships a Bazel binary on `PATH`. Unverified claims about receipts and GPU state need a run.


| | |
|---|---|
| Repo | `/data02/home/philip.yang/workspace/milano` (origin `git@code.byted.org:philip.yang/milano.git`) |
| Commit read | `ed218aeedd0c047a0434bfd263bb96f47187fff5` ("Let milano.scm finish run from the worktree it removes") |
| Branch read | `diziet/mainline` (36 commits, 2026-09-26 to 2026-09-28); working tree clean at read time |
| Date | 2026-09-29 |
| Mode | Read-only. No builds, GPU jobs, or git writes were run in the repo. One scratch experiment (section 8, item S1) imported `milano.scm` with bytecode writing disabled and operated only on throwaway repos under the session scratchpad. |

Citation form is `path:line` relative to the milano repo unless a full path is given. Labels:
**[READ]** stated in a file or commit message I opened; **[INFERRED]** my conclusion from what I read;
**[EXPERIMENT]** observed by running something. Unlabeled sentences that carry a citation are [READ].
Nothing here was verified on the accelerator host, so no claim about what runs on the B200s today is
first-hand; run-time results are quoted from commit messages and the roadmap.

Prior material reused and re-checked: `robust-training-observatory/docs/research/2026-09-28-training-loop-integration-source-map.md`
(Milano section, lines 91-135) and `2026-09-28-draccus-as-reference-for-capsule-layering.md`. Its Milano
citations (`torch_accel/__init__.py:6`, `docs/design/torch-accel-integration.md:3`) still resolve at this
commit. It was written at `bf6ab34`; two later commits (`f93083a`, `0876b47`) changed the NCCL story and added
`milano.scm`, so it does not describe the current NCCL provider.

---

## 1. What problem this repo exists to solve

**One-line purpose.** Milano builds, packages, and qualifies `torch_accel`, an in-tree PyTorch extension that
exposes a logical `accel` device (PyTorch `PrivateUse1`, renamed) backed by the CUDA driver API and NCCL, and it
decides whether one exact packaged artifact may be published (`README.md:3-5`, `CONTEXT.md:3-11`).

It is two things in one repo, and the second is the reason for most of the code:

1. **The extension** (`torch_accel/`, about 3,600 lines of C++/CUDA/Python) is a from-scratch out-of-tree
   accelerator backend: allocator, guard, streams, events, 33 ATen overloads, a NCCL `ProcessGroup`. It is
   deliberately small and is the object under test.
2. **The harness** (`milano/`, `scripts/`, about 4,900 lines) is a hermetic build/release/evidence system whose
   job is to make "this artifact works on 8 B200s" a claim a reader can re-check.

**For whom.** [INFERRED] The people and agents who own the `torch_accel` backend on host `n116-077-207`, and a
downstream SCM publisher: `scripts/publish_scm.sh:6` requires `MILANO_SCM_PUBLISHER` ("the reviewed SCM uploader
executable"), which Milano does not contain. Development is done largely by AI agents: commit trailers name
Claude Opus 5.5 (all commits from `b82b397` on) and TRAE CLI (`0c7fd08`, `07bb93b`); `AGENTS.md:179-235` is a
protocol for several concurrent agents. The observatory treats Milano as "the controlled model of CUDA-backed
PyTorch runtime mechanics" (`robust-training-observatory/.../2026-09-28-training-loop-integration-source-map.md:134`).

**The pain, concretely.** Every item below is a failure the repo records having actually had, not a
hypothetical.

- **Receipts that said something the run did not do (finding N2).** Every release qualification before
  `bf6ab34` loaded the CUDA image's system NCCL 2.28.3, while receipts recorded the pinned "2.29.7-1",
  copied from the target file. Cause: the Insula `LD_LIBRARY_PATH` listed `/usr/lib/x86_64-linux-gnu`, which
  outranks every RUNPATH (`docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:579-587`; commit
  `bf6ab34`). Two NCCLs then coexisted in one rootfs (`f93083a` message).
- **Coverage lost silently.** A multi-device test on a 1-GPU allocation skipped instead of failing
  (roadmap:152-165; `tests/conftest.py:16-37`, "Verified with 1 visible GPU: 3 skips without the variable, 3
  failures with it").
- **Wrong numbers with no error.** Baseline table (roadmap:13-25): `torch.ones(2**25).sum()` returned
  `16777216.0` (serial fp32 sum saturates and took 8.8 s, roadmap:328-329); `relu(NaN)` returned 0;
  `addmm(nan_bias, beta=0)` returned NaN; an op on `accel:1` moved the current device; int32 `fill_` raised.
- **Races only visible with a slow kernel.** Cross-device copy read stale data; `fill_` ignored the current
  stream (roadmap:144-147, review IDs C-H1, C-M3). These needed a controllable spin kernel to reproduce
  (`tests/support/accel_test_kernels.cu`).
- **Hangs.** A missing NCCL peer hung; a collective hang wedged a session. Now bounded by wall-clock budgets
  and non-blocking NCCL init (`milano/release.py:136-157`; roadmap:223-239, 378-388).
- **Flaky build path.** Bazel module resolution intermittently timed out, and the Bazel layer was a
  `no-sandbox`, `local` genrule that added no isolation (roadmap:23, 80; commit `1eaaaf0`).
- **Ambient host leakage.** The plan bound 34 host driver libraries including GTK, EGL, Wayland
  (`4a6d4b5` message); nothing audited what the runtime resolved.
- **Shared, fragile hardware.** GPU 0 primary-context faults (`retain=999`) blocked the eight-GPU gate
  (`07bb93b` message, roadmap:68, 920); concurrent agents fought over GPUs and, worse, over one worktree's
  staged index, which *is* the test candidate (`0876b47` message).
- **Overclaiming.** Topology names `fsdp-2`, `hsdp-4` etc. suggested real FSDP; they were hand-written
  all_reduce/all_gather loops, and their losses were printed, not checked (`987869f` message; roadmap:84).

**What goes wrong without it.** [INFERRED from the list above] An agent or human could change a kernel, run
tests on whatever GPU was free, on whatever libraries happened to load, from a working tree that was not what
they thought, and publish a receipt that names versions that were not exercised. Milano's structure (staged-index
candidates, content-addressed bundle store, mandatory bundle-candidate binding, loaded-library digest probe,
strict-xfail review findings) is a direct response to each of those.

---

## 2. Non-goals and boundaries

**Deliberate non-goals** (each with its source):

- **Not a general backend product.** One target, `torch-accel-v2.10`: Python 3.11.16, PyTorch 2.10.0 at commit
  `449b1768...`, CUDA 13.0.2, `sm_100`, NCCL 2.28.3-1, host `n116-077-207` (`targets/torch-accel-v2.10.toml`;
  `CONTEXT.md:3`). Adding a second target is not a supported path today.
- **No FSDP/DTensor claim.** `torch.distributed.fsdp` needs `reduce_scatter`, which is unimplemented; the five
  workloads are "FSDP/HSDP/TP-*style* probes" and the names are "evidence keys, not an FSDP claim"
  (`milano/release.py:91-96`; `docs/guides/build-and-qualify.md:83-89`; roadmap D4 at :84 defers a real claim to
  Task 5.4).
- **No fallback, ever.** No CPU/CUDA redispatch, no `PrivateUse1` backend fallback; missing operators must stay
  missing and fail (`AGENTS.md:160`, `:240`; `docs/design/torch-accel-integration.md:58-61`).
- **No PyTorch patches.** "Use stock pinned PyTorch extension interfaces" (`AGENTS.md:238`; roadmap:39).
  Contrast torchetta, which builds ByteDance libtorch forks (section 6).
- **Does not build the NVIDIA stack.** CUDA, cuDNN, NCCL are whatever the digest-pinned image ships
  (`AGENTS.md:97-107`; `milano/nvidia_externals.py:1-13`).
- **Not network-isolated.** "Hermetic means isolation at the rootfs level, not network isolation" (roadmap:44-50;
  `milano/rootfs.py:118-121` shares the network and binds host `resolv.conf`/`hosts`). Pinning downloads is a
  separate, lower-priority reproducibility concern (roadmap:545-563, Task 3.3).
- **Not performance-qualified yet.** The allocator is raw `cuMemAlloc`/`cuMemFree` with no stream-ordered
  caching (`torch_accel/csrc/runtime/allocator.cpp:27`, `:80`); small-op latency is about 2.3 ms against a
  target under 30 us (roadmap:21, 787-805). No benchmarks are recorded evidence yet (Task 5.0 open).
- **No AMP, RNG, compile, or serialization support** unless a claim needs it
  (`docs/design/pytorch-extension-mechanisms.md`, "Gaps and risks to track", from line 449; `resize_`/`set_`
  unregistered, line ~486).
- **No Bazel.** Dropped on 2026-09-28; Appendix A of the roadmap (:946-1028) records when to bring it back and
  how (Bazel would build artifacts, Milano would still qualify them).
- **No remote qualification host.** `scripts/remote/run_on_devx_b200_dbg.sh:4-9` is a retired stub that exits 2;
  `tests/test_remote_boundary.py` asserts it stays retired.
- **Local macOS checks never authorize a runtime or release claim** (`AGENTS.md:88`).
- **Not signed (yet).** "Receipts are not yet signed. Someone with write access to the cache could still fabricate
  a fully consistent run" (`docs/guides/build-and-qualify.md:135-136`; roadmap D3, Task 4.6).

**What Milano treats as authority** (things it will not second-guess):

| Authority | Where enforced |
|---|---|
| The **staged git index** (or an exact commit with `--rev`) is the candidate; the working tree is ignored | `scripts/run_local_insula.sh:21-40`; `AGENTS.md:26-30` |
| The **target TOML** for versions and qualification host | `milano/source_build.py:66-82`; `milano/verifier.py:303-309` |
| The **base image digest** (`nvidia/cuda:13.0.2-cudnn-devel-ubuntu24.04@sha256:ae7f65...`) | `scripts/rootfs/build_rootfs.sh:4-5, 61` |
| **dpkg database inside the image** for NVIDIA versions | `milano/nvidia_externals.py:71-100` |
| **`/proc/self/maps` and `mountinfo`** for what is really loaded and from which mount | `milano/isolation.py:147-166, 209-224` |
| **CPU reference / CUDA oracle** for numerics, **CPU replay** for distributed losses | `tests/python/test_op_matrix.py` (roadmap:195-202); `tests/distributed/parallel_training_smoke.py:207-329` |
| **The allowlist file** for the dispatcher surface | `tests/negative/accel_kernel_allowlist.txt`; `tests/negative/test_dispatch_surface.py:44-55` |
| **Hostname** `n116-077-207` for accelerator work | `scripts/run_local_insula.sh:8-14`; `milano/rootfs.py:71-75` |

**What depends on it.** [READ] `scripts/publish_scm.sh` (an externally supplied publisher receives only the
re-verified immutable bundle copy, `:13-21`). [INFERRED] Nothing else in the workspace depends on Milano in code:
a grep of the milano tree for `torchetta|ultron|ndtl|arnold|vaso|distributed_timeline|torchtitan|corenius|monarch`
returned no hits, and the observatory notes cite Milano only as a documented example (section 6).

---

## 3. Architecture

### 3.1 Execution model: the Insula

"Insula" is Milano's name for the bubblewrap sandbox in which everything runs. It has no glossary entry in the
repo; the definition below is assembled from `milano/rootfs.py` and `AGENTS.md:10-22, 108-136`.

```
 HOST n116-077-207 (only launches; contributes driver libs + /dev/nvidia*)
 ┌──────────────────────────────────────────────────────────────────────┐
 │ scripts/run_local_insula.sh [--rev <commit>] -- <cmd>                  │
 │   hostname check ─► git write-tree | rev-parse ─► git archive          │
 │   (mtime=1970)  ─► candidate.tar ─► sha256 = candidate_digest          │
 │   candidates/sha256-<d>.tar (atomic admit)  launchers/sha256-<d>/      │
 │   rootfs_key = sha256(build_rootfs.sh @ that tree)                     │
 │   ─► scripts/rootfs/enter_rootfs.sh ─► python -m milano.rootfs         │
 │        create_rootfs_plan(): bwrap args + plan_violations() allowlist  │
 └───────────────┬──────────────────────────────────────────────────────┘
                 │ bwrap --ro-bind rootfs / --unshare-all --share-net --clearenv
                 ▼
 ┌─ INSULA ─────────────────────────────────────────────────────────────┐
 │ /                     rootfs (ro): NVIDIA image + g++13/clang18/uv/py  │
 │ /run/milano/candidate.tar (ro) ─► run-candidate: re-hash, reject unsafe │
 │        members, extract to /workspace/milano (writable tmpfs; NOT enforced ro)  │
 │ /cache/milano (rw)    spack, venvs, builds, wheels, artifacts, evidence │
 │ /run/milano/driver    10 allowlisted host libcuda/NVML libs             │
 │ /dev/nvidia*, /usr/bin/nvidia-smi, /etc/{resolv.conf,hosts}             │
 │                                                                         │
 │ scripts/rootfs/run_pipeline.sh <mode>                                   │
 │   dev modes: build(cached by input digest) ─► py/torchrun/runtime-test  │
 │   release:   package ─► qualify_b200.sh ─► 6 gates ─► evidence dir      │
 └──────────────────────────────────────────────────────────────────────┘
```

Facts behind the diagram:

- **Candidate = staged index.** `git write-tree` then `git archive --mtime=1970-01-01`
  (`run_local_insula.sh:34, 39-40`); digest is the sha256 of the tar (`milano/candidate.py:11-16`,
  `milano/local_insula.py`). Unstaged edits are "silently absent" (`AGENTS.md:26-30`); the agent driver warns
  about them (`.claude/skills/run-milano/driver.sh:35-42`).
- **Candidate integrity inside.** `run-candidate` re-hashes the tar, rejects absolute/`..`/device/fifo members,
  extracts with `--no-same-owner` (`build_rootfs.sh:106-134`); the same checks exist host-side in
  `milano/candidate.py:20-39`.
- **Rootfs key is a recipe hash, not a measured tree.** `rootfs_key = sha256(build_rootfs.sh)` computed from the
  candidate's own copy of the script (`run_local_insula.sh:65-69`; `milano/local_insula.py:21`). That key is
  passed as the `rootfs_digest` recorded in build identities and receipts (`:82`). `milano/rootfs_identity.py`
  can hash a real tree, but is only a fallback when no digest is passed (`milano/rootfs.py:84`). See section 8.
- **Isolation.** `--ro-bind rootfs /`, `--unshare-all`, `--die-with-parent`, `--clearenv`
  (`milano/rootfs.py:88-117`); environment comes only from `milano/runtime_env.py:19-42`. Every bind source must be
  the rootfs, the cache, the candidate, or an allowlisted host file, else the plan is refused
  (`milano/isolation.py:87-109`, `milano/rootfs.py:156-166`). The driver bind was cut from 34 libraries to 10
  (`isolation.py:40-51`; `4a6d4b5`).
- **Hostname gate** in both shell (`run_local_insula.sh:8-14`) and Python (`rootfs.py:71-75`), with the default
  duplicated in `rootfs.py:22`, `run_local_insula.sh:8`, and `targets/*.toml` (roadmap Task 4.3 wants one source).
- **Inside-only.** Every pipeline/gate script starts with `test "${MILANO_IN_INSULA:-0}" = 1`
  (e.g. `run_pipeline.sh:4`, `qualify_b200.sh:4`); `tests/test_remote_boundary.py:35-37` asserts it.

### 3.2 Build and cache

- **Native build.** `milano/native_build.py` runs `setup.py build_ext` with the pinned venv Python and caches `_C`
  under `builds/<target>/by-input/<digest>/`. The digest covers native sources plus `setup.py`, rootfs digest,
  installed torch `RECORD`, NCCL prefix *path*, Spack lock digest, and arch list (`native_build.py:42-59, 196-206`).
  An `flock` plus a temp dir and `os.rename` make concurrent builders compile once and failures cache nothing
  (`:131-171`). Each candidate still gets its own build root and `BuildIdentity` (`:190-241`), recording the three
  tools actually used (`compiler_description`, `:92-108`). Measured: cold 37 s, cached 4 s (`1eaaaf0` message).
- **Development modes run the shipped extension.** `ensure_build_runtime` (`run_pipeline.sh:54-64`) builds via
  `build_torch_accel.sh`, copies `_C` next to `torch_accel/*.py` into `installed/source-<candidate>/`
  (`:27-49`), and runs Python with `-P` so the checkout's `torch_accel/` (which has no `_C`) cannot shadow it
  (`:51-53, 74-75`).
- **Layout authority.** `milano/target_layout.py` (`:20-66`) computes venv, build, artifact, wheel, and latest
  paths; shell scripts query it with `python -m milano.target_layout`. The PyTorch wheel dir is keyed by commit,
  CUDA, and NCCL (`:55-62`), added after "the first rebuild attempt silently relabeled the old 2.29.7-linked build:
  its CMakeCache still pointed at Spack" (`f93083a` message; `scripts/build_pytorch.sh:50-79` now starts clean and
  greps `CMakeCache.txt`).
- **Spack's role has shrunk.** The environment now concretizes to four specs, three of them non-buildable externals
  generated from the image's dpkg database (`spack/envs/torch-accel-v2.10/spack.yaml`, `nvidia-externals.yaml`;
  the old lock had 44 specs including a compiler toolchain, `f93083a` message). [INFERRED] Spack remains mostly as
  a name-to-prefix resolver and as the source of `spack_lock_digest` in identities and receipts.

### 3.3 Bundle

`milano/bundle.py` + `scripts/package_bundle.sh`:

- Payloads: repackaged `torch_accel` wheel (built deterministically, `bundle.py:84-132`), the PyTorch wheel, a
  `pip freeze` of the build venv as runtime constraints, the Spack lock, the target TOML, the build identity, the
  manifest schema (`package_bundle.sh:44-103`). The bundle contains no NVIDIA libraries.
- Manifest: `schema_version 1`, target, build identity, `capabilities` (`accel-runtime`, `accel-nccl`, `ddp`),
  `gate_order`, per-file sha256/size/role, and `runtime_libraries` (sha256 of the image's `libnccl.so.2.28.3`, the
  wheel's `libtorch_cuda.so`, and `torch_accel/_C...so`) (`bundle.py:202-216`; `package_bundle.sh:67-73`).
- Deterministic archive: sorted entries, mtime 0, uid/gid 0, gzip mtime 0 (`bundle.py:156-162, 228-239`).
- Path hygiene: destinations cannot be absolute, contain `..`, or start with `cache|dev|proc|rootfs|run|sys`
  (`bundle.py:37-47`).

### 3.4 The extension (`torch_accel/`)

Data flow for `left + right` on `accel:0` (from `docs/design/torch-accel-integration.md:42-61`, checked against
the source layout):

```
python import torch_accel
  __init__.py:6-8    refuse if PrivateUse1 already owned by anything but privateuseone/accel
  __init__.py:10-13  import _C, rename_privateuse1_backend("accel")
  __init__.py:34-38  register c10d backend "accel_nccl" for devices=["accel"]
  static init in _C: hooks, guard impl (C10_REGISTER_GUARD_IMPL), REGISTER_ALLOCATOR,
                     TORCH_LIBRARY_IMPL(aten, PrivateUse1, ...) per overload
tensor.add ──► dispatcher (PrivateUse1 key) ──► ops/pointwise.cpp
   validate_* (ops/validation.h) ──► at::empty keeps accel TensorOptions
   ──► kernels/pointwise.cu on current_stream(device)   (raw CUstream, streams.cpp)
   any CUDA/NCCL status ──► TORCH_CHECK exception
backward: PyTorch's generated formulas under AutogradPrivateUse1, redispatching to the same explicit kernels
```

Components: `runtime/` (allocator, `ContextScope`, streams with pools of 32 per device and priority, events,
hooks), `ops/` (creation, copy, pointwise, reduction, linear, metadata, `validation.h`), `kernels/*.cu`,
`distributed/` (`ProcessGroupAccelNCCL`, `WorkAccelNCCL`, `NcclComm`).

Key invariants and where they are enforced:

- **Logical identity is `PrivateUse1`, physical memory is `cuMemAlloc`.** `AccelAllocator::allocate` returns a
  `DataPtr` whose device is `PrivateUse1` (`allocator.cpp:23-37`). It is never a `cuda:N` facade
  (`torch-accel-integration.md:37-40`).
- **No fallback.** The set of `PrivateUse1` overloads equals `accel_kernel_allowlist.txt` exactly (33 `aten::`
  overloads plus 19 stock `c10d::` routing kernels, 63 lines with comments), and no backend fallback exists
  (`tests/negative/test_dispatch_surface.py:44-59`; roadmap:169-172).
- **Import never steals a slot** (`__init__.py:6-8`; `torch_accel/__init__.py:18-21` also refuses a foreign
  `torch.accel`).
- **The allocator deleter never throws.** `release` is `noexcept`, swallows failures, treats
  `CUDA_ERROR_DEINITIALIZED` and `CONTEXT_IS_DESTROYED` as success (`allocator.cpp:65-93`). Freeing temporaries is
  safe today only because `cuMemFree` waits for pending work (design doc :33-35); this is also why there is no
  async allocation and the small-op latency target is unmet.
- **Internal calls never move the user's current device.** `ContextScope` replaced bare `set_device` everywhere
  internal; `set_device` remains only for user intent (`a1c71c4` message; regression test
  `test_device.py::test_operations_on_another_device_leave_the_current_device_alone`).
- **Validation precedes any event or NCCL call.** `process_group_accel_nccl.cpp` validates rank, device, dtype,
  shape, layout, reduction, then all collectives go through one `enqueue_collective` (`:94-126`; design doc
  :107-110). Unsupported collectives use the stock `Backend` rejection.
- **NCCL lifecycle is locked and bounded.** `NcclComm` is non-blocking (`config.blocking = 0`, `nccl_utils.cpp:59`),
  enqueue and abort share one mutex, an aborted communicator raises on every later collective
  (`nccl_utils.h:20-24`; `nccl_utils.cpp:105-108`); init and issue are bounded by the process-group timeout.
- **Ordering.** The communication stream waits on a `ready` event recorded on the compute stream; a `completion`
  event is recorded on the comm stream and the *compute stream waits on it at enqueue*
  (`process_group_accel_nccl.cpp:107-116`). [INFERRED] That serializes compute behind every collective (no
  overlap); Task 5.3 lists "no forced compute-stream wait at enqueue" as a target (roadmap:813-828).
- **Each collective starts a detached watcher thread** (`work_accel_nccl.cpp:66-69`), also slated to become one
  watchdog per group (roadmap:817-819).
- **Test hooks live in the shipped binary**, gated by `MILANO_RUNTIME_TEST`
  (`work_accel_nccl.cpp:19-31`; `nccl_utils.cpp:138-143`, reachable through
  `TORCH_ACCEL_NCCL_TEST_ASYNC_ERROR` and `TORCH_ACCEL_NCCL_TEST_COMPLETION_DELAY_MS`). Roadmap Task 1.1 step 3
  (move them out of `_C`) is still unchecked (`:148-150`).

### 3.5 Key abstractions (harness)

| Abstraction | Invariant | Defined at |
|---|---|---|
| `Target` | Single source of pins; loaded from TOML | `milano/targets.py:14-63` |
| `RootfsPlan` / `create_rootfs_plan` | The only Insula execution seam; refuses non-allowlisted binds; host must match | `milano/rootfs.py:29-176` |
| `BuildIdentity` | Must match the target on version, commit, CUDA, arch, NCCL; digests well-formed | `milano/source_build.py:19-82` |
| `ReleasePlan`, `RELEASE_GATE_NAMES` | Ordered gates `repo, artifact, unpack, accel-runtime, accel-nccl, b200-release`; `b200-release` needs world size 8 and eight "NVIDIA B200" | `milano/release.py:11-18, 279-317` |
| `CudaHealthRequirement` | Exact and minimum device counts are mutually exclusive | `milano/release.py:25-40`, checked by `milano/cuda_health.py:78-110` |
| `GateReceipt` | Toolchain must record python, pytorch, cuda, nccl; `b200-release` requires 8x B200 names | `milano/verifier.py:32-67` |
| Bundle manifest | Every file's sha256 and size re-checked; no path escapes | `milano/verifier.py:94-117` |
| GPU lease | Exclusive `flock` per device under `$CACHE/locks/gpu/gpu-N`, shared across sandboxes; children inherit via `MILANO_GPU_LEASE` | `scripts/lib/gpu_lease.sh:13-58` |
| Timeout budget | pytest 600 s, torchrun 300 s, parallel-training 1200 s; `MILANO_TIMEOUT_<KIND>` override | `milano/release.py:136-157`; `scripts/lib/budget.sh:14-21` |

---

## 4. Key design decisions, trade-offs, lessons

Decisions D1-D7 are recorded at `roadmap:72-87` (accepted 2026-09-28; D5 still open).

| Decision | Chosen | Rejected / deferred and why | Evidence |
|---|---|---|---|
| Execution boundary | Local bubblewrap rootfs on one host; host is launch-only | Remote devx runner (retired in `07bb93b`); no reason recorded beyond "primary path" | `07bb93b`; `scripts/remote/run_on_devx_b200_dbg.sh` |
| Hermetic definition | Rootfs-level isolation; network allowed | Network isolation; offline mode exists as a plan option (`--offline`, `rootfs.py:76, 206`) but is not used by any gate | roadmap:44-50 |
| D1 build system | `setup.py`/ninja with input-digest cache (`milano.native_build`) | Keep Bazel: it was `no-sandbox`+`local`, added no isolation, "`//:source_tests` duplicated pytest", registry fetches were the flakiest step | roadmap:80; `1eaaaf0`; Appendix A gives return criteria (several native targets, remote cache, per-TU incremental builds) |
| D2 NCCL | **Single provider: the image's NCCL 2.28.3-1**, declared as a non-buildable Spack external from dpkg; bundle carries no NVIDIA libs; probe requires the image's `libnccl` to be the loaded one | Original: vendor a Spack-built 2.29.7 into `torch/lib` with `patchelf` (implemented in `bf6ab34`, then superseded by `f93083a` the same day), or a separate nccl wheel | roadmap:81-82, 577; `f93083a`, `bf6ab34` messages |
| D3 signing | `ssh-keygen -Y sign` key outside the cache, bind-mounted read-only for `b200-release` only; format kept signer-agnostic | Sigstore keyless (needs network/OIDC); KMS/HSM | roadmap:83; Task 4.6 (not started) |
| D4 FSDP | Rename to "style probes" now; implement `reduce_scatter` + FSDP2 helper ops later | Claim FSDP now | roadmap:84; `987869f`; Task 5.4 |
| D6 dtypes | Declared matrix: fp32/bf16/fp16/int32/int64/bool for creation/copy/fill; arithmetic fp32/bf16/fp16 with fp32 accumulation | Status quo (7 creatable, 2 arithmetic, 4 NCCL) | roadmap:86; today arithmetic accepts only fp32 and bf16 (`ops/validation.h:26-34`) |
| D7 qualification tests | (b) ship tests inside the bundle (Milestone 4) | (a) run candidate's tests, enforced with candidate == bundle build candidate (current) | roadmap:87, 719-728 |
| Native `PrivateUse1` approach | Explicit per-overload registration with negative tests | Blanket fallback; multiple private slots (`PrivateUse2/3` "are dispatcher backend components... not drop-in public device types") | `AGENTS.md:240`; `pytorch-extension-mechanisms.md:449-465` |
| Autograd | PyTorch's generated formulas under `AutogradPrivateUse1`; no custom engine | Custom autograd | `torch-accel-integration.md:80-87` |
| Test process | RED first, strict xfails tagged with review IDs, flipped to pass by the fixing task; one commit per task | Fix-then-test | roadmap:52-70, 218-219 |
| Numerics | Match PyTorch on NaN and `beta=0`; overlap checks via `at::assert_no_*overlap`; **deliberate deviation**: `mm(out=factor)` is rejected where CPU silently returns garbage | Copy CPU behavior verbatim | roadmap:349-354; `ops/validation.h:95-111` |
| Rootfs recipe as cache key | `sha256(build_rootfs.sh)`; measured tree digest deferred | Measure tree digest now (Task 3.5) | `run_local_insula.sh:65-69`; roadmap:610-628 |

### Lessons and known failures (all recorded)

1. **A claimed version is not a loaded version.** N2 is the central lesson. The fix is structural, not a
   version bump: one provider (`nvidia_externals.py` docstring: "which is how the Insula ended up with two NCCLs
   and loaded the wrong one"), plus a probe that maps `/proc/self/maps` and compares digests against the manifest
   (`isolation.py:237-283`).
2. **Directory order beats RUNPATH.** `LD_LIBRARY_PATH` naming `/usr/lib/x86_64-linux-gnu` shadowed every RUNPATH
   (`bf6ab34`); `runtime_env.py:30-33` keeps it out. (The comment there is now stale, section 8.)
3. **Stale CMake cache silently relabels a build** (`f93083a`); hence wheel dir keyed by inputs and a
   `CMakeCache.txt` assertion (`build_pytorch.sh:75-79`).
4. **Old mtimes defeat ninja.** Candidates extract with 1970 mtimes, so the native runtime test build is keyed by
   source hash (`run_pipeline.sh:89-93`); the roadmap also records a native test binary that had "never been
   rebuilt" (roadmap:283-285, commit `90d4bfc`).
5. **Dev modes that did not build.** `parallel-training` and `accel-runtime` called `prepare_build_runtime`
   without building, so any candidate not already built failed with no output (`987869f` message).
6. **Concurrent launches race.** `mv` of a launcher dir raced ("`$launcher.tmp`"); read-only git commands took
   `index.lock` (`0876b47` message; fixed by atomic rename, `GIT_OPTIONAL_LOCKS=0`).
7. **`cuMemFree` synchronization is load-bearing.** Correct temporaries lifetime today depends on it
   (design doc :33-35); replacing the allocator (Task 5.1) requires `recordStream` semantics first.
8. **C-M2 did not reproduce**: cross-GPU contexts were not created in a 4-rank torchrun; the probe is kept as a
   guard (roadmap:422-424). The repo does record non-reproducing findings.
9. **Roadmap prose drifts from code.** After `f93083a` the roadmap still describes 2.29.7 and `patchelf` vendoring
   in Task 3.4 (`:579-608`) and the risk register (`:923`); only an inline "Revised after review" note
   (`:577`) flags it. Treat the roadmap's Decisions table and commit messages as more current than task bodies.

---

## 5. Build, test, release, verification (the trust and evidence model)

### 5.1 Test tiers

- **Harness unit tests**: `tests/test_*.py` (26 files), no GPU. They cover the release plan, receipts, verifier,
  rootfs plan and isolation, native-build caching, cache GC, scm workflow (real throwaway git repos,
  `tests/test_scm.py`), forbidden-content audit, and doc-claim guards (`tests/test_integration_guide.py`).
- **Runtime tests**: `tests/python`, `tests/negative`, `tests/distributed`, plus `tests/native/runtime_test.cpp`
  (C++ contract test JIT-built by `run_pipeline.sh:78-104`). Collection is skipped unless `MILANO_RUNTIME_TEST=1`
  (`AGENTS.md:83-87`); release runs set it and `MILANO_REQUIRE_DEVICES=2` so skips become failures
  (`verify_release.sh:137`; `tests/conftest.py:31-35`).
- **Design-doc tests**: some tests grep docs and scripts (`tests/test_integration_guide.py`,
  `tests/test_remote_boundary.py`); roadmap Task 4.2 wants 23 text-grep tests replaced by argv tests
  (roadmap:680-681).
- **Negative surface**: `test_dispatch_surface.py` (allowlist equality), `test_unsupported_ops.py` (about 27 missing
  kernels plus 3 explicit rejections, roadmap:169-172), `test_distributed_failures.py` (timeouts, abort,
  injected async errors).
- **Reference oracles**: 158 CPU-parity cases across fp32/bf16 (`test_op_matrix.py`, roadmap:197-202); CPU replay of
  each of five distributed topologies, `rtol=1e-5, atol=1e-6` (`parallel_training_smoke.py:327-329`); a
  mutation that drops replica averaging fails (`987869f`).
- **Test-only kernels are outside the shipped binary**: `tests/support/accel_test_kernels.{cu,py}`, JIT-built and
  cached by source digest; `torch_accel.accelerator.stream_handle` is the public interop that lets them launch
  on accel's raw stream (`roadmap:140-143`; `torch_accel/csrc/registration.cpp:41-43`).

### 5.2 Pipeline modes

`scripts/rootfs/run_pipeline.sh` (317 lines) is the single dispatcher: `repo`, `build`, `package`, `artifact`,
`unpack`, `accel-runtime`, `accel-nccl`, `parallel-training`, `b200-release`, plus dev modes
(`runtime-test`, `operator-test`, `autograd-test`, `py`, `torchrun`, `feasibility-test`, `isolation-probe`,
`dependency-probe`, `spack-*`, `pytorch-build`, `cache-gc`). The `repo` gate is `check_repo.sh:10-21`: forbidden
content audit, `compileall`, full harness pytest, `bash -n` on every script, `git diff --check`.

Dev modes route around unhealthy GPUs (`milano.cuda_health --select-healthy`, `lease_devices`); release gates
never auto-select and need all eight (`AGENTS.md:58-66`; `qualify_b200.sh:43-46`).

### 5.3 Release flow

```
package         build (cached) ─► package_bundle.sh ─► bundle .tar.gz + latest.json (mutable pointer)
b200-release    qualify_b200.sh
  admit bundle: copy to artifacts/by-digest/<sha>.tar.gz, chmod 0444, digest of the COPY  (verifier.py:179-206)
  require bundle build candidate == running candidate                                      (verifier.py:164-176)
  mkdir evidence/releases/<bundle-digest>/<utc>-<rand>  (exclusive)                        (qualify_b200.sh:37-40)
  lease 8 GPUs; preflight cuda_health --require-b200-count 8 (primary-context probe)       (:43-46)
  for each of 6 gate invocations (release.b200_release_gate_invocations, release.py:236-269):
        run with stdin=/dev/null, tee to NN-<gate>.log; then write NN-<gate>.json receipt  (:94-102)
  verifier release-evidence: re-validate the whole run                                     (:111-113)
  overwrite evidence/releases/latest.json                                                  (:114-124)
publish_scm.sh: re-admit bundle, re-run release-evidence on the stored copy, exec publisher (publish_scm.sh:13-21)
```

Gates: 01 `repo`; 02 `artifact` (extract and verify manifest); 03 `unpack` (fresh venv from the bundle, import
check, isolation probe); 04 `accel-runtime` (packaged-runtime probe, isolation probe, full `tests/python` +
`tests/negative` + backend registration against the *installed bundle*, `MILANO_REQUIRE_DEVICES=2`); 05 `accel-nccl`
world size 2 (collectives + DDP); 06 `b200-release` world size 8 (health, probe, collectives, DDP, five topologies).
Source: `verify_release.sh:118-167`.

The tested runtime is a fresh venv built from the bundle wheels plus pinned pure-Python constraints, plus unpinned test tooling (`pytest setuptools ninja`, `verify_release.sh:75-78`), with
`PYTHONPATH` unset and `-P` so nothing imports from the build venv (`verify_release.sh:64-84, 137-139`).

### 5.4 What a receipt proves, and what it does not

`verify_release_evidence` (`verifier.py:225-275`) requires: exactly the six file names `NN-gate.json` in order; each
receipt rebuilt through `GateReceipt` (schema rules re-applied); bundle digest equals the stored copy's digest;
`passed is True`; the sibling `.log` exists and its sha256 equals `outputs.log_sha256`; all receipts agree on
candidate, rootfs, Spack lock, and host; host equals the target's qualification host; the bundle's build candidate
equals the receipts' candidate.

Strengths [INFERRED from the code above]: a hand-edited log, a reordered or missing gate, a mixed run, another host,
or a bundle swapped after the fact is rejected. Random run ids and exclusive `mkdir` avoid overwriting; the bundle
store is read-only content-addressed.

Limits, all real in the code:

- **Receipts are unsigned and self-consistent-forgeable** (`build-and-qualify.md:135-136`).
- **Failed gates leave no receipt.** `run_gate` writes the receipt only after the gate command succeeds
  (`qualify_b200.sh:94-102`, `set -e` with `pipefail`); failure evidence exists only as a partial log. Roadmap
  Task 4.4 wants failure receipts.
- **Some receipt fields are copied, not measured.** `toolchain.cuda` and `toolchain.nccl` come from the target
  TOML (`qualify_b200.sh:83-84`), `status` is the literal `"passed"` (`:87`), `passed=True` is hard-coded (`:88`).
  This is the same class of defect as N2. The compensating control is the isolation probe report in the hashed
  logs of gates 03 and 04, which checks loaded-library digests (`verify_release.sh:91-96, 128, 134`); gate 06
  does not run the probe.
- **GPU driver version and CUDA runtime actually loaded are not recorded in receipts**; only `nvidia-smi` names
  (`qualify_b200.sh:66-71`). The host driver libraries are bind-mounted from the host and are the one unpinned input.
  [INFERRED]
- **`rootfs_digest` is a recipe hash** (section 3.1), and the recipe runs unpinned `apt-get install`
  (`build_rootfs.sh:62-64`), so the same digest can denote different rootfs contents if rebuilt later. [INFERRED]
- **Candidate == bundle build candidate.** This forces every gate to run from the same commit that built the bundle
  and makes re-qualifying an old bundle from a newer checkout impossible until D7(b) lands.
- **`latest.json` is mutable.** `qualify_b200.sh:119-122` and `package_bundle.sh:104-112` overwrite pointer files
  that live under the artifact and evidence trees ("Evidence is append-only" is true of runs, not of pointers;
  roadmap Task 4.4 moves them out).
- **Content-addressing is partial.** Bundles and candidates are content-addressed; receipts are named by ordinal
  and directories by bundle digest + random run id. `milano/records.py`'s `EvidenceRecord` (which
  `AGENTS.md` describes as the evidence writer) is referenced only by `tests/test_records.py` (grep of the tree);
  real receipts go through `verifier.write_gate_receipt`, a near-copy of `write_record`
  (`records.py:44-66` vs `verifier.py:70-91`). [INFERRED: dead abstraction]
- **Insula is not tamper-proof against the cache.** The cache is mounted read-write and holds toolchains and
  evidence (`rootfs.py:108-110`); anyone with write access can alter it.

### 5.5 Evidence produced so far (as recorded, not re-verified)

Runs cited in messages: bundle `1b74312c` run `20260928T175526Z-9fa049544f75` (P0 gates); `8345398f` run
`20260928T182537Z-68aebd205820`; `7b6d7c62` run `20260928T194230Z-84792c2ae54a` (Milestone 1 exit, 240 passed, 9 strict
xfails, 0 skipped, roadmap:244-246); `14a4d366` run `20260928T204145Z-6039898084a9` (Milestone 2 exit, 291 passed,
0 xfailed, 0 skipped, roadmap:464-466); `9aa6de61` run `20260928T222132Z-f18f0be1b016` (current NCCL stack,
`f93083a`). Evidence lives in `$MILANO_CACHE_ROOT/evidence/`, not in git, so these ids cannot be checked from a clone.

### 5.6 Other verification machinery

- `milano/cache_gc.py` (dry run by default, never touches `evidence/`, keeps anything evidence or `latest` names;
  roadmap:100-109).
- `scripts/audit_forbidden_content.py`: rejects five legacy tokens stored hex-encoded, in tracked and untracked
  files (`:17-24`); run by the pre-commit hook and the `repo` gate. `AGENTS.md:176` forbids reintroducing names
  from the prior backend.
- `.githooks/pre-commit` (diff check + audit) and `.githooks/pre-push` (`milano.scm` refuses non-fast-forward,
  merge, or delete of `diziet/mainline`).

---

## 6. Interfaces with the other repos in the workspace

**Headline [READ+INFERRED]: Milano has no code-level interface with any sibling repo.** A tree grep for
`torchetta|ultron|ndtl|arnold|vaso|distributed_timeline|torchtitan|corenius|monarch` (excluding `.git`, `.venv`,
`uv.lock`) found no matches. Everything below is conceptual overlap, gathered from the siblings' own files and the
observatory notes, and is labeled accordingly. I read only the parts of siblings named in each row.

| Sibling | Relationship | Shared vocabulary / contract | Evidence |
|---|---|---|---|
| **torchetta** (`9322aa1`) | Peer build/evidence harness for a *different* torch: ByteDance libtorch training-serving forks (v2.7, v2.10), CUDA 13.2, sm 80/90/100. Milano targets stock PyTorch 2.10.0, CUDA 13.0.2, sm_100 | Same word "insula" for a bwrap rootfs; both use Spack, `check_repo.sh`, digest-pinned rootfs, "wheels are release-valid only inside their matching insula". Torchetta's first commit (2026-09-09) predates Milano's (2026-09-26), so [INFERRED] the concept flowed torchetta to Milano. Torchetta owns the NCCL stream-id accessor verifier (`get_coll_cuda_stream_id`, `get_p2p_cuda_stream_id`) that Milano's stock-PyTorch backend cannot provide | `torchetta/README.md:1-12, 59-72`; `torchetta/targets.tsv` header and rows; `robust-training-observatory/.../training-loop-integration-source-map.md:28-36` |
| **torchtitan** | Real training loop. Not a consumer of Milano. Milano's five "parallel-training" probes are hand-written stand-ins for its parallelisms | None in code. Milano explicitly does *not* qualify `torch.distributed.fsdp`/DTensor, which TorchTitan uses | `release.py:91-96`; source map :38-47, :122-135 |
| **ndtl** / **distributed_timeline** | NdTimeline/NDTL are trace producers/bridges. Milano has no NdTimeline/NDTL instrumentation | None. Observatory note: "Do not claim Milano has NdTimeline or NDTL instrumentation today" | source map :134-135, :348 |
| **arnold** | Product observability behavior; no link | None | `2026-09-29-controller-repos-source-map.md:24-27` (lists `milano` among "lower runtime repos") |
| **ultron**, **vaso** | Controller/governance and other repos; I found no Milano relationship and did not read their code | Unknown | (not read) |
| **monarch**, **nccl** (workspace dirs) | Named with Milano as "lower runtime repos" in the observatory notes | Unknown | controller-repos source map :24-27 |
| **robust-training-observatory** | Documents Milano; also the source of the "capsule" layering model whose relation to Milano's Insula is a design analogy | Its capsule layers (base rootfs, Spack foundation, uv overlay, candidate, state, driver interface) map onto Milano's rootfs + `/cache` + candidate + driver-allowlist, but Milano has *no sealed layers*: `/cache/milano` is one read-write mount holding Spack, venvs, builds, evidence | RTO `AGENTS.md` "Capsule anatomy" table (lines 51-70); `milano/rootfs.py:108-110` |
| **corenius** | This repo; Milano's `AGENTS.md`/`scm.py` protocol is a candidate pattern (section 7) | n/a | this document |

Shared-vocabulary crib for corenius readers:

- **Insula** (Milano, torchetta): bwrap-sandboxed rootfs plus mounts in which all builds/tests/evidence run.
- **Candidate**: the immutable archive of the *staged* tree under test; digest is its identity.
- **Bundle**: the artifact that is qualified and published (wheels + locks + manifest), addressed by sha256.
- **Gate**: an ordered release check that writes a receipt naming the bundle digest.
- **Receipt / evidence**: append-only JSON + hashed log under `evidence/releases/<bundle>/<run>/`.
- **Dev mode vs release gate**: dev modes run the cached shipped extension; release gates run only against the
  installed bundle.
- **Lease**: an `flock` on a GPU; the sole coordination primitive for shared hardware.
- **Task branch**: `<agent>/<topic>`; integration is by `milano.scm integrate`.

Who calls whom (within Milano): agents call `driver.sh` (`.claude/skills/run-milano/driver.sh`) or
`run_local_insula.sh`; `milano.scm integrate` calls `run_local_insula.sh --rev`; the pipeline calls the
`milano.*` modules via `python -m`; `publish_scm.sh` calls an external publisher.

---

## 7. Relevance to agentic engineering

### 7.1 How agents work in this repo today

- **Guidance files.** `AGENTS.md` (243 lines, checked in; the only agent instruction file), `CONTEXT.md`
  (12-line domain summary), and one project skill `.claude/skills/run-milano/{SKILL.md,driver.sh}`. There is no
  `CLAUDE.md`. Plans and specs live in `docs/superpowers/{plans,specs}` (the "superpowers" skill convention; the
  roadmap's header line 3 tells agentic workers to use `superpowers:executing-plans`).
- **A single command front door.** All execution goes through `run_local_insula.sh -- <cmd>`; the agent driver
  prints changes that are *not* in the candidate and a one-line `driver: <cmd> ok|FAILED (tree <hash>)`
  (`driver.sh:35-42, 65-73`).
- **A per-task protocol enforced by tooling** (`AGENTS.md:179-235`; `milano/scm.py`): setup once (hooks, SSH key,
  `pull.rebase`); `new --agent --topic` makes `../milano.worktrees/<agent>-<topic>` from `origin/diziet/mainline`;
  one task per commit with "Verified in the Insula: ..." and a Co-Authored-By trailer; roadmap boxes ticked in the
  same commit; `integrate` rebases, runs the chosen gates *on the exact tip* via `--rev`, fast-forwards, pushes
  (serialized by a lock in the git common dir, `scm.py:131-136`); `finish` removes the worktree and branch.
- **Gates chosen by what changed**: `torch_accel/` needs `runtime-test`; distributed also needs
  `accel-nccl --world-size 2 --collectives-only --ddp`; milestone exits need `package` then `b200-release`
  (`AGENTS.md:214-217`).
- **Working agreement inside the roadmap** (roadmap:52-70): RED test first, GREEN, regression gates, commit, close
  the milestone with a full release.
- **Observed practice.** Roughly the whole hardening programme (Milestones 0-3 partial) landed in one day, 27 commits
  on 2026-09-28 (36 in total: 6 on 09-26, 3 on 09-27), each citing a task number, review ID, and Insula results (`git log`). [INFERRED] The commit
  messages function as the durable ledger; the roadmap's status text is updated in the same commit.

### 7.2 Mechanisms worth reusing in corenius

1. **Candidate = staged index (or exact commit), archived deterministically and hashed.** Removes "my checkout vs
   what ran" ambiguity; `--rev` lets integration gate precisely what gets pushed
   (`run_local_insula.sh:21-40`).
2. **Bind receipts to artifact digests, and bind the tester to the artifact.** Copy into a read-only
   content-addressed store, hash the copy, refuse candidate != build candidate (`verifier.py:164-206`).
3. **Verify by observation, not declaration.** The isolation probe classifies what the process actually mapped and
   compares digests with the manifest (`isolation.py:237-283`); the N2 story is the argument for it.
4. **Strict-xfail as a review ledger.** Each review finding became an `xfail(strict=True)` test tagged with its ID;
   fixing it flips XPASS (failure) until the marker is removed (roadmap:218-219, 301-302).
5. **Surface allowlists with a two-way diff.** The dispatcher-surface test reports "registered but not listed" and
   "listed but not registered" (`test_dispatch_surface.py:44-55`); adding capability shows up in review.
6. **Fail-not-skip switches for release runs** (`MILANO_REQUIRE_DEVICES`, `tests/conftest.py:16-37`) and
   wall-clock budgets that convert a hang into a named failure (`budget.sh:14-21`).
7. **A tested multi-agent SCM helper.** `milano/scm.py` is stdlib-only and tested against real throwaway repos
   (`tests/test_scm.py`); it includes a pre-push hook that makes "linear mainline, no force, no delete" mechanical.
8. **Resource leases via `flock` on shared files** with holder metadata and waiting messages (`gpu_lease.sh:13-58`),
   simple enough to port to any scarce resource.
9. **Plan objects in Python, thin shell.** `create_rootfs_plan` returns data that can be inspected without
   launching (`--emit-plan`, `rootfs.py:224-226`), and `release_plan` is the single owner of gate order (`verifier`
   imports it, `verifier.py:16`).
10. **Honesty rules in the docs and tests.** Names that are evidence keys but not claims are annotated in code and
    guarded by tests (`release.py:91-96`; `tests/test_integration_guide.py:44-50` checks that the guide advertises only
    manifest capabilities).
11. **Evidence classes in the commit message** ("Verified in the Insula: rtest 291 passed; accel-nccl ...; evidence
    run ..."): a cheap convention that lets a reader tell which gate covered which claim.

### 7.3 Hazards for agents

- **Unstaged edits vanish from the candidate silently** (`AGENTS.md:26-30`). The driver warns; raw
  `run_local_insula.sh` does not.
- **Sharing a worktree means testing someone else's staged files.** Never stage or commit in the main checkout
  (`AGENTS.md:196`); run `milano.scm` from your own worktree because `python -m` imports the current directory's
  copy (`AGENTS.md:186`).
- **`milano.scm integrate` retry is broken after a rejected push [EXPERIMENT]** (item S1, section 8): a push
  rejected because another agent won the race leaves local `diziet/mainline` at the pre-rebase tip and the next
  attempt fails with "local diziet/mainline has commits not on the rebased branch; sync it first". The docs say it
  "retries from the rebase" (`AGENTS.md:212`; `0876b47` message). The integration lock is local to one clone's
  git dir (`scm.py:131-136`), so it does not order agents on different clones.
- **Documented gate names do not all exist.** `scm.py:14` (the module docstring) shows `--gate rtest`, but `rtest`
  is a `driver.sh` alias; `integrate` passes the string straight to `run_pipeline.sh`, which has no `rtest` mode
  (`scm.py:139-147`; `run_pipeline.sh:125-317`). `AGENTS.md:206, 214-217` uses the correct `runtime-test`.
- **GPU state is shared and can fail spuriously.** `retain=999` is a host fault, not a test failure; do not weaken a
  gate (roadmap:68-70). A qualification holds all eight GPUs for its whole run and other agents block on it
  (`AGENTS.md:224-229`), with a default wait of 900 s (`gpu_lease.sh:21`).
- **Shared cache.** Never delete another run's state by hand; `cache-gc` dry-run first (`AGENTS.md:230-232`).
  Each qualification admits a ~290 MB bundle (roadmap:106-107).
- **Hostname-gated.** Any other host exits with status 2 (`run_local_insula.sh:8-14`); agents on other machines can
  only edit and run host-safe checks, and those do not count (`AGENTS.md:88`).
- **Forbidden-content audit runs on tracked and untracked files** (`audit_forbidden_content.py:40-56`); the
  tokens are hex-encoded, so an agent cannot discover them by grep and should not try to decode or spell them.
- **User-specific config is committed.** `scm.LOCAL_CONFIG` sets `core.sshCommand ssh -i ~/.ssh/byted_code_devx`
  (`scm.py:252-257`) and `AGENTS.md:190` names the `code.byted.org` push key: `milano.scm setup` will fail or
  misconfigure on any other account or machine. [INFERRED]
- **A green source gate is not evidence for anything runtime.** `repo` runs harness tests and grep-style checks
  only; runtime claims need GPU gates (`AGENTS.md:88`).
- **Prose drift.** `runtime_env.py:30-32` comment and the roadmap's 3.4 body describe a superseded NCCL design;
  an agent reading them alone will build the wrong mental model.
- **Test hooks in the shipped `_C`** are controlled by environment variables that a runtime process can set
  (`MILANO_RUNTIME_TEST`, `TORCH_ACCEL_NCCL_TEST_*`); do not assume the shipped binary is hook-free.

---

## 8. Open problems, debt, uncertainties

**Roadmap status at this commit** (from task headings and boxes; `roadmap` = the hardening roadmap):

| Milestone | State | Notes |
|---|---|---|
| M0 P0 trust gates + cache GC | done | roadmap:91-111 |
| M1 test foundation | done except Task 1.1 step 3 (move NCCL fault hooks out of `_C`) | roadmap:148-150; exit met 2026-09-28 |
| M2 runtime correctness | all nine tasks done | exit met, bundle `14a4d366` (roadmap:464-466). Quantitative event-leak counter not done (:390-391) |
| M3 rootfs isolation, self-contained artifact | 3.1, 3.2, 3.4 done; **3.3 (locked PyTorch build env) and 3.5 (measured rootfs identity) open**; Bazel-binary removal follow-up open (:500-502) | |
| M4 Python orchestrator, signed evidence | **not started** (no task marked done) | shell is 317+168+124+84+... lines, not about 100 |
| M5 performance and capability | **not started** | small-op latency, caching allocator, real FSDP, TensorIterator, cuBLASLt, autocast |
| M6 continuous qualification | **not started**; D5 (CI system) still open | all four boxes unchecked, roadmap:883-897 |

Definition-of-done items still false: signed chained attestation; bundle runs without the Milano cache mounted;
dispatcher surface equals op table equals allowlist (no op table yet); all baseline rows met (latency row and
forged-evidence row open, roadmap:21, 25); 14 green nightlies; shell at most about 150 lines
(roadmap:928-941).

**Debt and uncertainties found in this read** (each with evidence; labels as in the header):

1. **`rootfs_digest` denotes the recipe, not the contents.** The build runs unpinned `apt-get update && install`
   on top of the pinned image (`build_rootfs.sh:62-64`); measured tree digest is Task 3.5. Consequence [INFERRED]:
   native-build cache hits (`native_build.py:42-59`, keyed by rootfs digest plus NCCL *path*) and receipts can
   agree while rootfs contents differ. Also the contract file omits any tree digest (`build_rootfs.sh:137-139`).
2. **Bazel still ships in the rootfs and on `PATH`** (`build_rootfs.sh:11-12, 88-91`; `runtime_env.py:38`), and
   `cache_gc.py` has a `_RETIRED = ("bazel",)` entry (`:30`). The `rootfs-contract.json` still records a
   Bazel version (`build_rootfs.sh:138`). Removing it changes the rootfs key, so it was batched with Task 3.5.
3. **Stale comment**: `runtime_env.py:30-32` says the system library directory "let the CUDA image's libnccl 2.28.3
   shadow the pinned NCCL", which was true when the pin was 2.29.7; the pin is now 2.28.3 from that same image
   (`targets/torch-accel-v2.10.toml`). Current behavior is right; the explanation is not.
4. **Receipt fields copied from the target file** (`qualify_b200.sh:80-88`), the same defect class as N2, and no
   driver version or loaded-library digests in the receipt itself (only in gate logs 03 and 04). `b200-release`
   (gate 06) runs no isolation probe (`verify_release.sh:151-165`). [INFERRED risk]
5. **Failed gates write no receipt**; no chain (`prev`), no argv/exit code/timing fields (Task 4.4 receipt v2).
6. **Unsigned receipts** (Task 4.6). The cache is a trusted mount.
7. **`records.py` (`EvidenceRecord`) and `insula.py` (`InsulaIdentity`) are unused by the pipeline** (grep;
   Task 4.7 mentions `InsulaIdentity`). `AGENTS.md` still lists `records.py` as the evidence writer.
   `bundle.py:23-30` duplicates `RELEASE_GATE_NAMES` (Task 4.7 wants an import); `cuda_health.py:12` imports
   `CudaHealthRequirement` from `release.py` (roadmap wants the reverse). The `sha256:` regex and validator are duplicated across `local_insula.py:13-18`,
   `target_layout.py:11-17`, `verifier.py:20-29`, `source_build.py:11`, `targets.py:9`, and `_candidate_key` exists
   twice (roadmap:651-652 counts seven validators).
8. **Inline Python inside shell** (`qualify_b200.sh:50-91, 114-124`; `package_bundle.sh:44-114`;
   `run_pipeline.sh:80-103, 132-138, 287-294`; `verify_release.sh:51-58`) and hard-coded `"gpu_count": 8` in
   `dependency-probe` (`run_pipeline.sh:138`), against the stated rule "Keep shell wrappers thin"
   (`AGENTS.md:241`). Task 4 addresses all of it.
9. **`milano.scm integrate` retry bug** (S1 below) and the `rtest` docstring error; `scm.py` machine-specific SSH
   config.
10. **Allocator and NCCL performance/behavior debts that are correctness-adjacent**: `cuMemFree`-synchronization
    dependence; compute stream forced to wait at enqueue; detached thread per collective; `reduce_scatter`,
    `alltoall` unimplemented so FSDP/DTensor and sharding claims are impossible; `resize_`/`set_` unregistered
    (`pytorch-extension-mechanisms.md` end; roadmap Milestone 5).
11. **Dtype policy is not yet the declared matrix (D6).** Arithmetic supports fp32 and bf16 only
    (`ops/validation.h:26-34`), so fp16 training is out of scope today.
12. **Reproducibility of build inputs**: `build_pytorch.sh:44-47` installs `setuptools>=75 wheel ninja cmake numpy
    pytest` unpinned from the network; the PyTorch source and Spack come from GitHub at pinned commits
    (`bootstrap_spack.sh:8`, `build_pytorch.sh:31-38`); Task 3.3 (hash-locked requirements) is open.
13. **Single point of failure host.** Everything, including the source gate, needs `n116-077-207` inside the Insula;
    no CI exists (D5 open). Hosted CI for host-safe checks is Milestone 6.1.
14. **Evidence is not in git and not inspectable from a clone.** Every run id cited above is
    unverifiable from the repo alone.
15. **Unknowns I could not resolve without running or asking:** whether `origin/diziet/mainline` equals the local
    HEAD; whether the cited evidence directories still exist; who consumes the publisher output downstream; whether
    any other agent or repo uses `milano.scm` or the `run-milano` skill; what "diziet" refers to (branch owner name,
    unexplained in the repo).

### Experiment record

- **S1 (retry after rejected push).** In a throwaway bare origin, a clone with `diziet/mainline` and a task worktree
  `t/x`, I called `milano.scm.integrate(..., run_gate=fn)` where `fn` pushed a competing commit from a second clone
  during the first gate. Result: `scm: push rejected (attempt 1)` then `ScmError: local diziet/mainline has commits
  not on the rebased branch; sync it first`; the local mainline was left at the pre-rebase tip (`9cc6662`) while the
  origin held the competitor's commit (`9c21d5e`). Cause read at `scm.py:205-215` (`_fast_forward_mainline` runs
  before the push and its ancestry check fails on the second attempt). `tests/test_scm.py` covers integration after
  another agent's push but not a push race during the gate (test list at `tests/test_scm.py:64-174`).
  Scratch files were under the session scratchpad only.

---

## How to verify this document

1. `git -C /data02/home/philip.yang/workspace/milano rev-parse HEAD` should print
   `ed218aeedd0c047a0434bfd263bb96f47187fff5`; `git branch --show-current` should print `diziet/mainline`. If HEAD
   moved, line numbers may drift.
2. Spot-check the trust model: read `milano/verifier.py:225-275` and confirm it matches section 5.4; read
   `scripts/qualify_b200.sh:80-102` for the copied `toolchain` fields and the receipt-only-on-success behavior.
3. Confirm the NCCL single-provider story: `git show f93083a --stat` and `git show bf6ab34` messages;
   `milano/nvidia_externals.py:1-13`; `spack/envs/torch-accel-v2.10/nvidia-externals.yaml`;
   `milano/runtime_env.py:28-33` (and the stale comment).
4. Confirm no sibling coupling:
   `grep -rIn -i "torchetta\|ultron\|ndtl\|arnold\|vaso\|distributed_timeline\|torchtitan\|corenius\|monarch" /data02/home/philip.yang/workspace/milano --exclude-dir=.git --exclude-dir=.venv --exclude=uv.lock`
   should print nothing.
5. Confirm dead abstractions:
   `grep -rn "EvidenceRecord\|InsulaIdentity" /data02/home/philip.yang/workspace/milano/milano /data02/home/philip.yang/workspace/milano/scripts`
   should show only their defining files and `tests/test_records.py` use.
6. Confirm the allowlist size and surface check: `wc -l tests/negative/accel_kernel_allowlist.txt` (63) and read
   `tests/negative/test_dispatch_surface.py:44-59`.
7. Reproduce S1 without touching the repo: create a bare origin plus two clones and a task worktree in a temp dir, set
   `PYTHONDONTWRITEBYTECODE=1`, put the milano checkout on `sys.path`, and call `milano.scm.integrate` with a
   `run_gate` that pushes a competing commit on its first call.
8. Roadmap status: `grep -n "— done\|^- \[ \]\|^- \[x\]" docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md`
   and compare with the table in section 8.
9. Runtime claims (test counts, evidence run ids, timings) are quoted from commit messages and the roadmap. To
   verify them you must be on `n116-077-207` and run
   `./scripts/run_local_insula.sh -- scripts/rootfs/run_pipeline.sh b200-release --bundle latest --world-size 8`
   then inspect `$MILANO_CACHE_ROOT/evidence/releases/<bundle-digest>/<run-id>/`; this document did not do so.

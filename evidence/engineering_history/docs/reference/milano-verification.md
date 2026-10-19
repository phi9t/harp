# Milano Reference Verification

Verified document: `docs/reference/milano.md`
Milano source boundary: `/data02/home/philip.yang/workspace/milano` at
`ed218aeedd0c047a0434bfd263bb96f47187fff5`.

Method: I inspected the Milano commit with read-only commands, using
`git show ed218ae:<path>`, `git grep` and `git log`/`git show --no-patch` for
commit messages. I did not run Milano scripts, builds, tests, the Insula, or GPU
jobs. Citations below are Milano paths and line numbers at `ed218ae` unless
explicitly marked as a commit message or command-derived count.

## Summary

| Verdict | Count |
|---|---:|
| CONFIRMED | 53 |
| WRONG | 3 |
| STALE | 1 |
| UNVERIFIABLE FROM SOURCE | 2 |
| Total | 59 |

## Claim Checks

1. Claim: "Commit read `ed218aeedd0c047a0434bfd263bb96f47187fff5`; branch read has 36 commits, 2026-09-26 to 2026-09-28."
   Evidence: `git rev-parse ed218ae^{commit}` returned that full SHA; `git rev-list --count ed218ae` returned 36; `git log --reverse ed218ae` showed first commit dated 2026-09-26 and `ed218ae` dated 2026-09-28.
   Verdict: CONFIRMED.

2. Claim: "One target, `torch-accel-v2.10`: Python 3.11.16, PyTorch 2.10.0 at `449b1768...`, CUDA 13.0.2, `sm_100`, NCCL 2.28.3-1, host `n116-077-207`."
   Evidence: `targets/torch-accel-v2.10.toml:1-11`; `README.md:6-15`; `AGENTS.md:91-95`.
   Verdict: CONFIRMED.

3. Claim: "Milano builds/packages/qualifies `torch_accel`, and the extension is about 3,600 lines while the harness is about 4,900 lines."
   Evidence: `README.md:3-4`; `CONTEXT.md:3-10`; command-derived counts at `ed218ae`: `torch_accel/` = 3,599 lines, `milano/` + `scripts/` = 4,944 lines.
   Verdict: CONFIRMED.

4. Claim: "`scripts/publish_scm.sh` requires `MILANO_SCM_PUBLISHER`, an external reviewed SCM uploader executable."
   Evidence: `scripts/publish_scm.sh:4-7`.
   Verdict: CONFIRMED.

5. Claim: "Finding N2: earlier release qualification loaded image NCCL 2.28.3 while receipts recorded 2.29.7-1; current design uses the image NCCL as the single provider."
   Evidence: `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:577-589`; `milano/nvidia_externals.py:1-13`; `spack/envs/torch-accel-v2.10/nvidia-externals.yaml:17-21`; `runtime_env.py:30-33`.
   Verdict: CONFIRMED.

6. Claim: "No FSDP/DTensor claim; the five workloads are FSDP/HSDP/TP-style probes and `reduce_scatter` is unimplemented."
   Evidence: `milano/release.py:91-96`; `docs/guides/build-and-qualify.md:83-89`; `docs/design/pytorch-extension-mechanisms.md:459-485`.
   Verdict: CONFIRMED.

7. Claim: "No CPU/CUDA redispatch or blanket `PrivateUse1` backend fallback; unsupported behavior must fail."
   Evidence: `docs/design/torch-accel-integration.md:58-61`; `tests/negative/test_dispatch_surface.py:44-59`; `AGENTS.md:238-240`.
   Verdict: CONFIRMED.

8. Claim: "Milano does not build the NVIDIA stack; CUDA, cuDNN, and NCCL come from the digest-pinned image and are non-buildable Spack externals."
   Evidence: `AGENTS.md:97-107`; `milano/nvidia_externals.py:1-13`; `spack/envs/torch-accel-v2.10/nvidia-externals.yaml:1-21`.
   Verdict: CONFIRMED.

9. Claim: "Hermetic does not mean network-isolated; the rootfs plan can share networking and bind host resolver files."
   Evidence: `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:44-50`; `milano/rootfs.py:118-122`.
   Verdict: CONFIRMED.

10. Claim: "`No Bazel. Dropped on 2026-09-28.`"
    Evidence: `scripts/check_repo.sh:10-20` no longer runs Bazel and roadmap task 3.1 deleted Bazel build files (`docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:493-495`), but the rootfs still defines and installs Bazel (`scripts/rootfs/build_rootfs.sh:11-12`, `scripts/rootfs/build_rootfs.sh:88-91`), `PATH` still contains `/opt/milano/bazel/bin` (`milano/runtime_env.py:38`), and the roadmap leaves Bazel removal as a follow-up (`docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:500-502`).
    Verdict: STALE. Correct current wording: Bazel is removed from Milano's active build/check path, but the rootfs still ships a Bazel binary and exposes it on `PATH`.

11. Claim: "No remote qualification host; `scripts/remote/run_on_devx_b200_dbg.sh` is a retired stub that exits 2 and tests assert that."
    Evidence: `scripts/remote/run_on_devx_b200_dbg.sh:4-9`; `tests/test_remote_boundary.py:4-17`.
    Verdict: CONFIRMED.

12. Claim: "Receipts are not signed yet and someone with cache write access could fabricate a consistent run."
    Evidence: `docs/guides/build-and-qualify.md:134-136`; signing is future work in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:730-739`.
    Verdict: CONFIRMED.

13. Claim: "Candidate equals the staged index, or an exact commit under `--rev`; it is archived deterministically with mtime 1970 and hashed."
    Evidence: `scripts/run_local_insula.sh:15-24`; `scripts/run_local_insula.sh:31-40`; `scripts/run_local_insula.sh:41-52`; `AGENTS.md:26-30`.
    Verdict: CONFIRMED.

14. Claim: "Candidate source is extracted to `/workspace/milano` read-only."
    Evidence: the candidate tar is mounted read-only (`milano/rootfs.py:111-113`), but `/workspace` is a writable tmpfs (`milano/rootfs.py:98-100`) and `run-candidate` creates `/workspace/milano` and extracts there without making it read-only (`scripts/rootfs/build_rootfs.sh:130-133`).
    Verdict: WRONG. Correction: the candidate archive is read-only; the extracted `/workspace/milano` tree is not enforced read-only by the bwrap plan or runner at this commit.

15. Claim: "The same candidate safety checks exist host-side and inside the rootfs runner."
    Evidence: inside `run-candidate` checks digest, absolute paths, `..`, char/block/fifo members (`scripts/rootfs/build_rootfs.sh:112-129`); host-side `candidate.py` also checks symlink and hardlink targets for absolute/`..` escapes (`milano/candidate.py:20-39`).
    Verdict: WRONG. Correction: the host-side validator is stricter than the inside-rootfs runner; the inside runner does not perform the symlink/hardlink target checks present in `candidate.py`.

16. Claim: "`rootfs_digest` is a recipe hash from `build_rootfs.sh`, not a measured tree digest, and measured tree identity is only a fallback/future task."
    Evidence: `scripts/run_local_insula.sh:65-69`; `scripts/run_local_insula.sh:78-83`; `milano/local_insula.py:22-23`; `milano/rootfs.py:83-85`; roadmap Task 3.5 in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:610-623`.
    Verdict: CONFIRMED.

17. Claim: "The rootfs plan uses `--ro-bind rootfs /`, tmpfs for `/tmp`, `/run`, `/workspace`, `--unshare-all`, `--die-with-parent`, `--clearenv`, and environment from `runtime_env.py`."
    Evidence: `milano/rootfs.py:88-117`; `milano/rootfs.py:141-149`; `milano/runtime_env.py:18-43`.
    Verdict: CONFIRMED.

18. Claim: "Every bind source must be the rootfs, cache, candidate, or allowlisted host file; driver bind was cut to 10 allowlisted library prefixes."
    Evidence: `milano/isolation.py:40-56`; `milano/isolation.py:87-109`; `milano/rootfs.py:156-166`; roadmap notes `34 libraries to 10` in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:504-514`.
    Verdict: CONFIRMED.

19. Claim: "Hostname gate exists both in shell and Python, defaulting to `n116-077-207`."
    Evidence: `scripts/run_local_insula.sh:8-14`; `milano/rootfs.py:22`; `milano/rootfs.py:71-75`.
    Verdict: CONFIRMED.

20. Claim: "Pipeline/gate scripts reject host execution with `test \"${MILANO_IN_INSULA:-0}\" = 1`."
    Evidence: `scripts/rootfs/run_pipeline.sh:1-5`; `scripts/package_bundle.sh:1-5`; `scripts/qualify_b200.sh:1-5`; `scripts/verify_release.sh:1-5`; `tests/test_remote_boundary.py:26-28`.
    Verdict: CONFIRMED.

21. Claim: "Native build cache digest covers native sources, `setup.py`, rootfs digest, torch RECORD, NCCL prefix path, Spack lock digest, and arch list; concurrent builders compile once and failures cache nothing."
    Evidence: `milano/native_build.py:42-59`; `milano/native_build.py:62-71`; `milano/native_build.py:122-171`; `milano/native_build.py:196-206`; roadmap mirrors this in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:476-486`.
    Verdict: CONFIRMED.

22. Claim: "The PyTorch wheel dir is keyed by commit, CUDA, and NCCL, and `build_pytorch.sh` starts clean and checks `CMakeCache.txt`."
    Evidence: `milano/target_layout.py:55-62`; `scripts/build_pytorch.sh:49-79`.
    Verdict: CONFIRMED.

23. Claim: "Spack now concretizes to four specs; NVIDIA CUDA/cuDNN/NCCL are non-buildable externals generated from dpkg, plus Python."
    Evidence: `spack/envs/torch-accel-v2.10/spack.yaml:22-32`; `spack/envs/torch-accel-v2.10/nvidia-externals.yaml:1-21`; `milano/nvidia_externals.py:71-100`.
    Verdict: CONFIRMED.

24. Claim: "The bundle payload includes wheels, runtime constraints, Spack lock, target TOML, build identity, and schema; it carries no NVIDIA libraries."
    Evidence: `scripts/package_bundle.sh:35-42`; `scripts/package_bundle.sh:67-73`; `scripts/package_bundle.sh:78-99`. The payload list contains no NVIDIA library payloads; NCCL is only recorded as a runtime library digest from the image.
    Verdict: CONFIRMED.

25. Claim: "The bundle manifest has schema version 1, target, build identity, capabilities `accel-runtime`, `accel-nccl`, `ddp`, gate order, file sha256/size/role, and optional runtime library digests."
    Evidence: `milano/bundle.py:202-216`; `packaging/bundle/manifest.schema.json:5-36`; `scripts/package_bundle.sh:69-73`; `scripts/package_bundle.sh:101-103`.
    Verdict: CONFIRMED.

26. Claim: "The bundle archive is deterministic and rejects bad destination paths."
    Evidence: destination validation in `milano/bundle.py:37-47`; normalized tar metadata in `milano/bundle.py:156-162`; sorted gzip/tar output with gzip mtime 0 in `milano/bundle.py:228-239`.
    Verdict: CONFIRMED.

27. Claim: "`torch_accel` import refuses a foreign `PrivateUse1` owner, renames to `accel`, refuses a conflicting `torch.accel`, and registers `accel_nccl` for `devices=[\"accel\"]`."
    Evidence: `torch_accel/__init__.py:6-21`; `torch_accel/__init__.py:29-39`.
    Verdict: CONFIRMED.

28. Claim: "Allocator returns a `PrivateUse1` `DataPtr` backed by `cuMemAlloc`; the deleter is `noexcept` and treats driver/context teardown errors as success."
    Evidence: `torch_accel/csrc/runtime/allocator.cpp:23-37`; `torch_accel/csrc/runtime/allocator.cpp:65-93`; `docs/design/torch-accel-integration.md:24-40`.
    Verdict: CONFIRMED.

29. Claim: "The dispatcher surface equals a 63-line allowlist with 33 `aten::` overloads plus 19 stock `c10d::` routing kernels, and no backend fallback."
    Evidence: `tests/negative/accel_kernel_allowlist.txt:1-63`; `tests/negative/test_dispatch_surface.py:44-59`; roadmap count in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:167-172`.
    Verdict: CONFIRMED.

30. Claim: "Arithmetic currently accepts only fp32 and bf16; D6's broader fp32/bf16/fp16/int/bool matrix is future policy."
    Evidence: current validation in `torch_accel/csrc/ops/validation.h:26-34`; roadmap D6 in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:86-87`.
    Verdict: CONFIRMED.

31. Claim: "Supported NCCL collectives validate rank/device/dtype/shape/layout/reduction before event creation or NCCL calls, then go through `enqueue_collective`."
    Evidence: e.g. constructor and allreduce validation in `torch_accel/csrc/distributed/process_group_accel_nccl.cpp:46-66` and `torch_accel/csrc/distributed/process_group_accel_nccl.cpp:159-177`; events are created only inside `enqueue_collective` at `torch_accel/csrc/distributed/process_group_accel_nccl.cpp:93-116`; design summary in `docs/design/torch-accel-integration.md:107-109`.
    Verdict: CONFIRMED.

32. Claim: "`NcclComm` is non-blocking, init/issue are bounded by timeout, enqueue and abort share a mutex, and later use after abort raises."
    Evidence: `torch_accel/csrc/distributed/nccl_utils.h:20-24`; `torch_accel/csrc/distributed/nccl_utils.h:40-54`; `torch_accel/csrc/distributed/nccl_utils.cpp:58-87`; `torch_accel/csrc/distributed/nccl_utils.cpp:105-110`; `tests/negative/test_distributed_failures.py:117-148`.
    Verdict: CONFIRMED.

33. Claim: "Collective ordering records a ready event on the compute stream, waits on the communication stream, then records completion and makes the compute stream wait at enqueue."
    Evidence: `torch_accel/csrc/distributed/process_group_accel_nccl.cpp:105-116`; `docs/design/torch-accel-integration.md:98-105`.
    Verdict: CONFIRMED.

34. Claim: "Each collective starts a detached watcher thread."
    Evidence: `torch_accel/csrc/distributed/process_group_accel_nccl.cpp:117-126`; `torch_accel/csrc/distributed/work_accel_nccl.cpp:65-69`.
    Verdict: CONFIRMED.

35. Claim: "NCCL test hooks live in the shipped `_C`, gated by `MILANO_RUNTIME_TEST` and `TORCH_ACCEL_NCCL_TEST_*` variables."
    Evidence: `torch_accel/csrc/distributed/work_accel_nccl.cpp:19-33`; `torch_accel/csrc/distributed/nccl_utils.cpp:138-143`; roadmap open step in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:120-150`.
    Verdict: CONFIRMED.

36. Claim: "Release gates are ordered `repo`, `artifact`, `unpack`, `accel-runtime`, `accel-nccl`, `b200-release`; `b200-release` requires world size 8 and eight `NVIDIA B200` names."
    Evidence: `milano/release.py:11-18`; `milano/release.py:236-269`; `milano/release.py:279-317`; `tests/test_release_plan.py:16-39`.
    Verdict: CONFIRMED.

37. Claim: "`GateReceipt` requires python, pytorch, cuda, and nccl in `toolchain`; `b200-release` requires 8x `NVIDIA B200` names."
    Evidence: `milano/verifier.py:32-67`.
    Verdict: CONFIRMED.

38. Claim: "Timeout budgets are pytest 600 s, torchrun 300 s, parallel-training 1200 s, with `MILANO_TIMEOUT_<KIND>` overrides."
    Evidence: `milano/release.py:136-157`; `tests/test_release_plan.py:156-164`; `scripts/lib/budget.sh:14-21`.
    Verdict: CONFIRMED.

39. Claim: "GPU lease is an exclusive `flock` per device under `$MILANO_CACHE_ROOT/locks/gpu/gpu-N`, inherited by children, with a default wait of 900 s."
    Evidence: `scripts/lib/gpu_lease.sh:1-13`; `scripts/lib/gpu_lease.sh:15-42`; `scripts/lib/gpu_lease.sh:49-57`; `AGENTS.md:224-229`.
    Verdict: CONFIRMED.

40. Claim: "The `repo` gate runs the forbidden-content audit, compileall, full harness pytest, `bash -n` on scripts, and `git diff --check`."
    Evidence: `scripts/check_repo.sh:10-20`; `AGENTS.md:73-80`.
    Verdict: CONFIRMED.

41. Claim: "Runtime test suites skip collection unless `MILANO_RUNTIME_TEST=1`; release runtime gate sets `MILANO_REQUIRE_DEVICES=2` so coverage loss fails instead of skipping."
    Evidence: `tests/python/conftest.py:7-10`; `tests/negative/conftest.py:7-10`; `tests/distributed/conftest.py:7-10`; `tests/conftest.py:16-35`; `scripts/verify_release.sh:137-139`.
    Verdict: CONFIRMED.

42. Claim: "Negative surface tests include 27 missing-kernel cases plus explicit validation rejections."
    Evidence: `tests/negative/test_unsupported_ops.py:37-68`; `tests/negative/test_unsupported_ops.py:82-105`; roadmap count in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:167-172`.
    Verdict: CONFIRMED.

43. Claim: "Parallel-training losses are checked against a CPU replay with `rtol=1e-5, atol=1e-6`."
    Evidence: `tests/distributed/parallel_training_smoke.py:207-260`; `tests/distributed/parallel_training_smoke.py:323-330`; guide summary in `docs/guides/build-and-qualify.md:83-89`.
    Verdict: CONFIRMED.

44. Claim: "Test-only spin kernels are outside the shipped binary and launch on accel's raw stream via `torch_accel.accelerator.stream_handle`."
    Evidence: `tests/support/accel_test_kernels.py:1-6`; `tests/support/accel_test_kernels.py:20-35`; `tests/support/accel_test_kernels.py:49-63`; `torch_accel/csrc/registration.cpp:41-43`.
    Verdict: CONFIRMED.

45. Claim: "`b200-release` admits an immutable copy into `artifacts/by-digest`, hashes the copy, chmods it read-only, and requires bundle build candidate == running candidate."
    Evidence: `scripts/qualify_b200.sh:27-40`; `milano/verifier.py:164-176`; `milano/verifier.py:179-206`; `tests/test_verifier.py:173-183`.
    Verdict: CONFIRMED.

46. Claim: "`verify_release_evidence` requires exactly the six receipt filenames in order, schema-valid receipts, matching bundle digest, passed=true, hashed logs, common candidate/rootfs/Spack/host, target host, and bundle candidate matching receipt candidate."
    Evidence: `milano/verifier.py:225-275`; `tests/test_verifier.py:101-170`.
    Verdict: CONFIRMED.

47. Claim: "Failed gates leave no receipt; receipts are written only after a gate command succeeds."
    Evidence: `scripts/qualify_b200.sh:94-102`; the script is `set -euo pipefail` at `scripts/qualify_b200.sh:1-2`; future failure receipts are planned in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:696-713`.
    Verdict: CONFIRMED.

48. Claim: "`toolchain.cuda` and `toolchain.nccl` are copied from target TOML, `status` is literal `passed`, `passed=True` is hard-coded, and gate 06 does not run the isolation probe."
    Evidence: `scripts/qualify_b200.sh:80-89`; `scripts/verify_release.sh:91-96`; `scripts/verify_release.sh:122-139`; `scripts/verify_release.sh:151-165`.
    Verdict: CONFIRMED.

49. Claim: "The tested runtime is a fresh venv built only from bundle wheels plus pinned pure-Python constraints."
    Evidence: `scripts/verify_release.sh:64-84` creates the venv and installs bundle wheels plus constraints, but also installs unpinned `pytest setuptools ninja` test tooling at `scripts/verify_release.sh:75-78`.
    Verdict: WRONG. Correction: the tested venv is based on the bundle wheels and runtime constraints, but it also installs unpinned test tooling (`pytest`, `setuptools`, `ninja`) during verification.

50. Claim: "`latest.json` pointers are mutable; evidence run directories are append-only but pointers are overwritten."
    Evidence: package pointer write in `scripts/package_bundle.sh:104-112`; release evidence pointer write in `scripts/qualify_b200.sh:114-124`; future pointer move in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:714-715`.
    Verdict: CONFIRMED.

51. Claim: "The cache is mounted read-write and is a trusted input; Insula is not tamper-proof against cache write access."
    Evidence: cache bind is `--bind` in `milano/rootfs.py:108-110`; cache state/evidence live under `/cache/milano` per `AGENTS.md:17-22`; unsigned-cache forgery warning in `docs/guides/build-and-qualify.md:134-136`.
    Verdict: CONFIRMED.

52. Claim: "`milano/cache_gc.py` is dry-run by default, never touches `evidence/`, and keeps evidence/latest/running-candidate/rootfs references."
    Evidence: `milano/cache_gc.py:1-10`; `milano/cache_gc.py:86-103`; `AGENTS.md:51-56`; roadmap task in `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:100-109`.
    Verdict: CONFIRMED.

53. Claim: "`scripts/audit_forbidden_content.py` rejects five legacy tokens, stored hex-encoded, in tracked and untracked files."
    Evidence: encoded needles in `scripts/audit_forbidden_content.py:17-24`; tracked/untracked scan in `scripts/audit_forbidden_content.py:40-56`; default selection in `scripts/audit_forbidden_content.py:129-135`; `AGENTS.md:175-177`.
    Verdict: CONFIRMED.

54. Claim: "Pre-commit runs `git diff --cached --check` and the forbidden-content audit; pre-push delegates to `milano.scm` to reject bad mainline pushes."
    Evidence: `.githooks/pre-commit:1-7`; `.githooks/pre-push:1-6`; `milano/scm.py:113-128`; `milano/scm.py:269-278`.
    Verdict: CONFIRMED.

55. Claim: "`milano.scm integrate` rebases, gates the exact tip using `run_local_insula.sh --rev`, fast-forwards mainline, and pushes under a local integration lock."
    Evidence: lock in `milano/scm.py:131-136`; gate command in `milano/scm.py:139-150`; integrate loop in `milano/scm.py:153-202`; `_fast_forward_mainline` in `milano/scm.py:205-215`; protocol in `AGENTS.md:205-212`.
    Verdict: CONFIRMED.

56. Claim: "`milano.scm integrate` retry is broken if a push is rejected after another agent wins during the gate; the next attempt fails with local mainline not on rebased branch."
    Evidence: first attempt fast-forwards local mainline before push (`milano/scm.py:188-200`); on push rejection it loops without resetting local mainline (`milano/scm.py:201-202`); the next `_fast_forward_mainline` rejects when current local mainline is not an ancestor of the rebased tip (`milano/scm.py:205-208`). Existing tests cover another push before integration, not a push race during the gate (`tests/test_scm.py:76-96`).
    Verdict: CONFIRMED from source control flow. I did not rerun the S1 throwaway experiment.

57. Claim: "`scm.py` documents `--gate rtest`, but `rtest` is only a driver alias and not a `run_pipeline.sh` mode."
    Evidence: docstring has `--gate rtest` in `milano/scm.py:10-15`; `integrate` passes gate strings through to `run_pipeline.sh` at `milano/scm.py:139-147`; `run_pipeline.sh` modes are enumerated at `scripts/rootfs/run_pipeline.sh:125-317` and contain no `rtest`; correct `runtime-test` guidance is in `AGENTS.md:214-217`.
    Verdict: CONFIRMED.

58. Claim: "Specific evidence run IDs and pass counts exist, including bundle `7b6d7c62` with 240 passed/9 strict xfails, bundle `14a4d366` with 291 passed/0 xfailed/0 skipped, and bundle `9aa6de61` current NCCL stack."
    Evidence: roadmap records the first two runs at `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:241-246` and `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:464-466`; `f93083a` commit message records bundle `9aa6de61` and run `20260928T222132Z-f18f0be1b016`.
    Verdict: UNVERIFIABLE FROM SOURCE. The repo source records these values, but the evidence directories live under `$MILANO_CACHE_ROOT/evidence/`, not git, and this task forbids running qualification or inspecting live accelerator state.

59. Claim: "GPU 0 primary-context `retain=999` blocked the eight-GPU gate."
    Evidence: roadmap records the health-failure policy and risk at `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:68-70` and `docs/superpowers/plans/2026-09-28-milano-hardening-roadmap.md:920-921`; `07bb93b` commit message records `b200-release` blocked by `device 0 retain=999`.
    Verdict: UNVERIFIABLE FROM SOURCE. The source records the incident, but current or historical GPU state requires live hardware/cache evidence, which I did not access.

## Wrong Claims And Corrections

- `docs/reference/milano.md` says the candidate is extracted read-only at `/workspace/milano`. At `ed218ae`, only `/run/milano/candidate.tar` is read-only. `/workspace` is a writable tmpfs and `run-candidate` extracts into it without changing permissions. Correction: say "the candidate archive is mounted read-only; extracted source is in `/workspace/milano` and is not made read-only by the runner."
- The document says the same candidate checks exist host-side and inside. Correction: host-side validation is stricter because `milano/candidate.py` rejects symlink and hardlink targets that escape via absolute paths or `..`; the inside runner does not.
- The document says the tested runtime venv is built only from bundle wheels plus pinned pure-Python constraints. Correction: `verify_release.sh` also installs unpinned `pytest`, `setuptools`, and `ninja` as test tooling.

## Important Omissions Noticed

- The stale NCCL prose is broader than the reference says. The document mentions the stale `runtime_env.py` comment and roadmap Task 3.4 body; `scripts/verify_release.sh:64-68` also still says the torch wheel has pinned NCCL vendored into `torch/lib`, which is superseded by the image-NCCL design.
- The extracted-candidate writability matters for threat modeling. The candidate tar is immutable, but the command runs from a writable extraction tree in tmpfs; evidence should rely on candidate digest and bundle binding, not on `/workspace/milano` being immutable.
- The S1 retry bug is not covered by `tests/test_scm.py`; the test suite covers an upstream push before integrate runs, not a competing push after gates and before push.
- Runtime outcome claims in commit messages are useful ledger entries but not independently auditable from a clone because evidence lives in the cache, not in git.

Unchecked claims: sibling-repo facts outside the Milano tree, live accelerator/cache evidence, exact historical runtime outcomes, and the section 8 S1 throwaway experiment were not re-run because the task forbids modifying or running Milano and requires read-only source inspection at ed218ae.

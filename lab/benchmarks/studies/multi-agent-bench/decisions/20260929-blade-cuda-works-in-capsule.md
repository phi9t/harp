# blade-cuda-works-in-capsule

Date: 20260929
Supersedes: 20260929-research-workers-verified-and-corrected.md (item 3 only)

## Decision

Real blade builds and tests CUDA (`cu_library`, sm_100) offline inside the `toolchains` capsule, and the built kernel runs on
the B200s with `--gpu`. Blade tasks at every ladder level, including CUDA, use real blade.

## Why

`research-blade-cuda` (worker, verdict DEGRADED: stderr from one sandbox-blocked delete only; run
`research-blade-cuda-20260929T230245Z`) found the root cause of "nvcc not found": `/usr/local/cuda` is a symlink to
`/home/zfc/cuda_13_2`, and the safe recipe's `--tmpfs /home` hides the link's target. My earlier "sailfish environment"
guess (item 3 of the previous decision) was wrong; the capsule's symlink-following `host-ro` layer already fixes it.
Lead re-ran in the capsule (offline, `--net none`), not on the worker's numbers:
- `blade-offline build //:cuda_app //:cuda_test //:host_test`: builds, cold 1.8 s, warm 1.4 s.
- `--gpu`: `BLADE_CUDA_RESULT answer=42 devices=8 device0=NVIDIA B200 sm=100`; `blade-offline test //:cuda_test` prints
  `[minitest] PASS ...` and `failures=0`.
- Without `--gpu` the CUDA runtime reports "no CUDA-capable device" cleanly (blade-built, cudart linked).
- `blade test` exit codes: pass 0, failing `cc_test` 250 (blade's internal 2042 truncated to a process status).
- `cc_test` needs `gtest_libs=[]`, `gtest_main_libs=[]` with a vendored `cpp3rdlib/minitest`.
- Blade's default gcc 4.9 toolchain lacks C++11 defaults (`nullptr` fails); `--toolchain=x86_64-gcc830 --skip-bundle` fixes it.
  `blade-offline` (capsule layer `blade-tools`) now adds both flags for build, test and run.
- The BUILD uses `-Wl,-dynamic-linker,/lib64/ld-linux-x86-64.so.2` and rpaths so the binary runs in the capsule.
- Probe on the capsule: all PASS; test suites: capsule 16, harness validate all tasks ok.

## Consequences

- `cuda` layer env sets `NVCC` and `CUDA_PATH` (not needed for the build to work, kept explicit).
- Blade-mini can be built from `evidence/research-blade-cuda/project` and `research-blade/projects`.
- Still unverified: blade `proto_library`/`thrift_library` builds (worker reported partial), commit-pinned deps, wheel packaging.

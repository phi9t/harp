# capsule-builder-m1

Date: 20260929
Supersedes:

## Decision

`tools/insula/capsule.py` (Python, standard library) is the capsule builder for the benchmark. Layers are `base`
(host `/usr`, labelled non-hermetic), `host-ro` (host paths bound read-only, fingerprint-digested, non-hermetic) and
`tree` (copied into a content-addressed store, hermetic). The root is a bwrap tmpfs, layers are bound at fixed paths,
env is an allowlist, net defaults to none. Requests live in `bench/insula/requests/`; `toolchains.json` is the first.
`terminal_agent/src/insula.rs` is untouched.

## Evidence (lead ran everything)

- `tools/insula/test_capsule.py`: 16 tests, real bwrap, pass. They cover deterministic digests, one layer moving only
  its own digest, read-only `/usr` and layers, hidden home, offline interfaces, env allowlist and forbidden vars,
  shadowing and reserved paths, stale host layer refused, tampered tree layer failing the identity probe, rw
  destinations, timeout and record, gc, symlinked host paths, GPU mode, CLI round trip.
- `toolchains.json` capsule (base + uv copy + rust + blade + blade-tools + cuda), `--net none`, no GPU: uv 0.12.17, cargo
  and rustc 1.95, cmake 3.25, ninja, g++ 12.2 (Debian, from host `/usr`), python 3.11, jq, git, nvcc 13.2 all run.
  Probe: usr_readonly, workspace_writable, host_home_hidden, pid_isolated, offline, env_clean, marker, identity,
  shadowing all PASS; hermetic INFO lists base, rust, blade, cuda as non-hermetic.
- Workloads inside the capsule, offline: blade `l0-flat` builds (1.3 s, using `blade-offline`, which pre-seeds the pip
  markers); the r01 cargo, p01 uv and c01 cmake tasks fail at start and pass fixed; an sm_100 CUDA kernel compiled with
  nvcc runs correctly on the B200s with `--gpu` (8 devices, `nvidia-smi -L` works).

## Findings that shaped the code

- `/usr/local/cuda` is a symlink to `/home/zfc/cuda_13_2` (another user's directory, `/home` is `/data00/home`). Inside the
  capsule `/home` is a tmpfs, so the link dangled and bwrap failed to mount. `host_binds()` now follows symlinks: the real
  directory is bound where the link points, and the link is recreated unless it is under `/usr`.
- GPU mode needs `/sys` read-only as well as `/dev/nvidia*`, and a fresh `/proc`. Without `/sys` the CUDA program segfaults
  silently. Without `--gpu` a CUDA program also segfaults (no devices), so GPU-less runs must not execute kernels.
- A layer mounted under the host home makes only its own top-level directory appear there; the home probe accepts exactly
  those names and fails on anything else.
- Host tools come from the host `/usr` (Debian 12: gcc 12, python 3.11, git 2.39), not the spack toolchain the shell sees.

## Not done / known limits

- No `uv` project environment layer (`uv sync` from a lock) and no docker-exported hermetic base yet; both are follow-ups.
- Blade `cu_library` still fails inside blade's own execution environment (nvcc "not found"); a research worker is on it.
- Host-ro layers are fingerprinted by top-level listing plus probe commands, not full content. A change deep inside `~/.rustup`
  would not be noticed. The layer is labelled non-hermetic for that reason.
- GPU mode exposes all of `/sys` read-only.
- traecli inside a capsule (the worker case) is not built or tested yet.

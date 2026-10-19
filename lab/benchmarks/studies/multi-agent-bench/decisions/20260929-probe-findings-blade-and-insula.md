# probe-findings-blade-and-insula

Date: 20260929
Supersedes:

## Decision

1. Blade tasks use real blade 1.2.2 (`/opt/tiger/ss_bin/blade`), run only inside a capsule with `--net none` and
   a cleared environment. Emulation is a fallback and is labelled `fidelity: emulated`.
2. The capsule env allowlist excludes `LIBRARY_PATH` (and all spack variables).
3. Capsules are layered and built from request files; layers are bound read-only at canonical paths because
   bwrap 0.8 has no overlayfs.
4. The capsule builder is Python under `tools/insula/`; `terminal_agent/src/insula.rs` is not edited in this effort.

## Why

Probe on 2026-09-29 (scratch project, hand-built bwrap, only the scratch dir writable):
- A minimal `cc_library` + `cc_binary` built with real blade, offline (`--unshare-net`), in about 3 s.
- Every bundled gcc (4.9.2, 4.9.4, 8.3) fails with "unknown spec function 'debug-level-gt'" when `LIBRARY_PATH`
  is set, as the spack environment sets it. Bisected by trying each variable alone; a clean env compiles.
- `hdrs=` is rejected by this blade ("unrecognized options").
- The built binary's interpreter is `/ld-linux-x86-64.so.2`, absent on the host, so it does not run outside the
  toolchain's sysroot.
- The first probe ran with network. Blade contacted internal services (remote cache/exec, build-event report)
  and sent build metadata, including the user name and the command line of the calling process. Offline runs
  only log the failed report and still build.
- `blade --help` earlier ran pip installs into the user's python2.7 environment.
- lgtorch `7ec5497` and erdos `9c075bd` were read from their git remotes over ssh into the scratchpad only.

Layered capsules follow robust-training-observatory's design, which the owner asked for by name.

## Consequences

- The blade-toolchain layer must expose the sysroot at `/` (or wrap `blade run`) so built binaries execute.
- Cpp3rdlib deps have not been tested; that probe is still open and decides how far up the ladder real blade goes.
- Telemetry exposure from probe 1 is a one-time event; from now on blade only runs with the network off.
- Two insula runners (Rust and Python) can drift; tracked as a follow-up.

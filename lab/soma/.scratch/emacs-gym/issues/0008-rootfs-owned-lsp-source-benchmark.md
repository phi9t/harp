# 0008: Implement The Rootfs-Owned LSP Source Benchmark

Status: Closed
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike; 0003 Rootfs-Local Doom Lifecycle Worker; 0004 Checked Elisp Bridge

## Intent

Create benchmark tasks that prove real Emacs+LSP operation against real source
fixtures inside the pinned Soma rootfs.

The benchmark must distinguish an Emacs client running in Insula from a fully
rootfs-owned LSP workflow. A task does not count as hermetic if `pyright`,
`clangd`, `rust-analyzer`, compilers, headers, package managers, build tools, or
language runtime state resolve from the host.

## Requirements

- Run Emacs, Doom package code, LSP client code, language servers, language
  runtimes, compilers, build/test tools, and source fixtures inside the selected
  Soma rootfs.
- Provide at least three source fixtures:
  - Python with the selected Python LSP, Python executable/prefix, package path,
    and test runner resolved inside the rootfs.
  - C/C++ or CUDA with `clangd`, compiler/linker, headers, libc/libstdc++
    identity, compile database, and build command resolved inside the rootfs.
  - Rust with `rust-analyzer`, `rustc`, `cargo`, crate/cache paths, and test
    command resolved inside the rootfs.
- Record selected rootfs manifest, dependency-closure id, ABI manifest id,
  emitted bwrap plan, server executable, server process id, cwd, environment,
  workspace root, document uri, document version, position encoding, diagnostics,
  and request ids.
- Verify source-level LSP behavior through Emacs, not only by invoking the
  language server directly.
- Treat server startup, indexing, diagnostics, rename/reference requests, and
  build/test commands as separate asynchronous states with explicit timeouts and
  `outcome_unknown` where completion cannot be proven.
- Make network policy explicit. Network may be allowed for materialization or a
  declared task topology, but scored episodes must not mutate the rootfs
  dependency closure invisibly.

## Verification

- A scripted correct policy opens each source fixture in Emacs, starts or
  attaches to the intended language server, and obtains diagnostics or symbol
  information from the rootfs-owned server.
- A Python fixture fails if the Python LSP, Python executable, package path, or
  test runner resolves from host state.
- A C/C++ fixture fails if `clangd`, compiler, headers, libc/libstdc++, compile
  database, or build command resolves from host state.
- A Rust fixture fails if `rust-analyzer`, `rustc`, `cargo`, crate/cache paths,
  or test command resolves from host state.
- A policy that edits text directly and reports it as an LSP refactor fails the
  grader.
- A stale document-version or stale workspace-root request fails without
  applying edits.

## Atlas Evidence

Probe `lsp-server-rootfs-resolution-contract` records the selected-rootfs
Insula's LSP server inventory and derives the production requirement: Emacs,
language servers, language runtimes/toolchains, and source fixtures must be
inside the selected rootfs closure before LSP benchmark evidence is accepted.

## Implementation Evidence

Implemented in the selected source-built Emacs rootfs:

- Runner: `scripts/insula/run-emacs-gym-lsp-benchmark-rootfs.sh`
- Public wrapper: `benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh`
- Benchmark implementation: `benchmarks/emacs-gym/lsp/elisp/run-benchmark.el`
- Latest standalone summary:
  `benchmarks/emacs-gym/lsp/traces/latest/summary.json`
- Latest aggregate copy:
  `benchmarks/emacs-gym/traces/latest/lsp/summary.json`

Selected rootfs:

- Store id:
  `soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Materialization kind: `source-built-emacs-v8`
- Recipe SHA256:
  `0bddf41e6539849c1d8e0a44954ebb3293f6eaa08081741c4adbf3c777fe808c`
- Base image:
  `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`
- Docker image:
  `soma-emacs-rootfs:0bddf41e6539849c`
- Image id:
  `sha256:9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d349d7bd6b6bb80`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Emacs: source-built stock GNU Emacs `30.2` at
  `/opt/soma/emacs/bin/emacs`
- Emacs executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Emacs source archive SHA256:
  `b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9`
- Doom commit:
  `8b475fcb2574286d7f4413c22cc7902d80d21ced`
- Rust toolchain: `1.81.0`, including the `rust-src` component required by
  `rust-analyzer`.

Validation:

- `scripts/insula/validate-emacs-rootfs-contract.py <selected-rootfs>` exits
  `0` with `"ok": true`, no errors, and no warnings.
- `scripts/insula/run-emacs-gym-lsp-benchmark-rootfs.sh` exits `0`.
- `benchmarks/emacs-gym/lsp/traces/latest/summary.json` reports
  `total_scenarios=6`, `scenarios_as_expected=6`, and
  `insula_checks_passed=true`.
- `scripts/insula/run-emacs-gym-benchmarks-rootfs.sh` exits `0`.
- `benchmarks/emacs-gym/traces/latest/summary.json` reports
  `benchmarks_passed=8`, `total_benchmarks=8`, `total_scenarios=76`,
  `scenarios_as_expected=76`, and `passed=true`.

Covered LSP scenarios:

- `correct-python-lsp-source`
- `correct-cpp-lsp-source`
- `correct-rust-lsp-source`
- `bad-direct-text-edit-as-refactor`
- `stale-document-version-rejected`
- `stale-workspace-root-rejected`

The Python fixture currently uses rootfs-owned `pylsp` because the selected
Doom profile contains `lsp-pylsp.el`. The rootfs contract also owns
`pyright`/`pyright-langserver`; switching the benchmark to Pyright is a separate
profile/client choice, not a hermeticity blocker.

## Remaining Follow-Up

This issue's rootfs-owned LSP source benchmark is complete for batch-safe
evidence. The live daemon bridge now adds one model-facing Python rename through
real `lsp-rename`, `textDocument/rename`, lsp-mode workspace edit application,
multi-file update checks, and rootfs Python tests after the rename. Remaining
daemon bridge evidence now also includes terminal/minibuffer
`command.dispatch -> lsp-rename`, prompt observation, tmux-supplied rename
input, original-request terminal status polling, passive post-command source
observation, and the same multi-file update/test checks. It also adds
diagnostics-after-edit evidence through a separate rootfs-owned
`pyright-langserver` fixture: the scenario records rootfs-owned
pyright/node/python executable paths, uses full-document LSP sync with debounce
disabled, guarded live-buffer replacement plus save creates pyright diagnostics
containing `Expected expression`, repair/save goes through the bridge, active
observation sees zero diagnostics afterward under explicit stale-diagnostic
cleanup policy, and the rootfs Python test passes.
Remaining LSP work is
additional
workspace-edit/code-action cases, multi-root workspace stress cases, and broader
async model-facing RPC bridge coverage.

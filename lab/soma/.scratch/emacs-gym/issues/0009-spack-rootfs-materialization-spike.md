# 0009: Implement The Rootfs Materialization Spike

Status: Source-built baseline captured; follow-up Spack/provenance work remains
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike

## Intent

Prototype a production-style materialization path for the Soma Insula rootfs.

The goal is not to launch the full gym yet. The goal is to prove that a complex
Emacs/Doom/LSP/Jupyter/Python/C++/Rust rootfs can be built with accounted
tool/package/dependency closure and ABI evidence rather than assembled from
ambient host installs.

The first executed path is not Spack-backed. It is a CUDA-image-derived
Docker/source-build materialization that builds GNU Emacs from source inside the
rootfs, syncs Doom inside the rootfs, and exports a selected rootfs consumed by
`bwrap`. Spack remains a follow-up option for richer dependency-DAG and ABI
provenance, but it must preserve the container-image glibc/dynamic-loader
boundary and must not introduce Nix or host ABI leakage.

## Requirements

- Define a Spack environment or equivalent manifest for the ABI-sensitive
  closure:
  - Emacs with required runtime features and native-comp/libgccjit support.
  - tree-sitter and grammars used by the selected Emacs profile.
  - Python and Jupyter native dependencies.
  - C/C++ toolchain, headers, libc/libstdc++/libgcc surfaces, and `clangd`.
  - Rust toolchain and `rust-analyzer`.
  - Git, shell/process tools, build/test tools, and any formatter/linter tools
    needed by initial benchmark fixtures.
- Emit materialization evidence: Spack concretization or equivalent lock,
  compiler/target metadata, source provenance, binary-cache provenance if used,
  rootfs manifest, tree hash, exported view identity, and ABI manifest.
- Keep rootfs construction separate from scored benchmark execution.
- Allow network during materialization only when the fetched inputs and produced
  manifest are recorded.
- Export a selected rootfs that `soma.insula` can validate without trusting
  environment variables alone.
- Document any Python-only overlay strategy. If `pip`, `uv`, or Jupyter kernel
  package installation is used, it must be captured as part of materialization
  evidence and not happen during scored episodes.

## Verification

- A clean materialization run produces a selected rootfs/view plus manifest
  artifacts.
- The rootfs contains or explicitly accounts for `emacs`, `python3`, `jupyter`,
  selected Python LSP, `clangd`, `rust-analyzer`, `clang` or GCC, `rustc`,
  `cargo`, Git, shell, and build/test tools.
- A clean verification run enters the materialized rootfs through `soma.insula`
  and proves tool resolution from inside the rootfs.
- The verification fails if any required tool resolves from undeclared host
  state.
- Re-running materialization with unchanged inputs is idempotent or produces an
  explained manifest change.

## Atlas Evidence

The current Atlas probes classify the development wrapper as
`not_rootfs_owned` with `abi_guarantee=none`. This issue provides the
materialization path needed to turn those contract probes into production
rootfs evidence.

## Artifact Storage

Rootfs payload storage is now codified as a Waystone handoff rather than git
content or Soma-owned TOS traffic:

- Operational doc: `docs/insula/rootfs-artifact-storage.md`
- Entrypoint: `scripts/insula/run-rootfs-artifact-storage.sh`
- Python authority: `scripts/insula/rootfs_artifact_storage.py`
- Current target: storage owner Waystone, provider `tos`, site `example-region`,
  profile hint `soma-insula-artifacts`, bucket name `example-artifacts`, object
  prefix `soma/insula-rootfs/`

Soma writes credential-free artifact manifests and
`*.waystone-storage-request.json` handoff files. Waystone owns the local vault,
bucket-id/profile resolution, and effectful upload/download traffic for TOS or
HDFS. Waystone owns HDFS today; until TOS transport is committed there, Soma
request files are handoff evidence rather than completed transfer evidence. A
downloaded rootfs must still pass
`validate-emacs-rootfs-contract.py --verify-rootfs-tree` before `selected.json`
can select it.

## Captured Evidence

Captured on 2026-09-28 and refreshed against the selected v8 rootfs:

- Builder: `scripts/insula/build-emacs-rootfs.sh`
- Rootfs contract validator:
  `scripts/insula/validate-emacs-rootfs-contract.py`
- Runtime runner: `scripts/insula/run-emacs-internals-probes-rootfs.sh`
- Public probe entrypoint: `probes/emacs-internals/scripts/run-probes.sh`
- Base image:
  `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`
- Materialization kind: `source-built-emacs-v8` / `docker-cuda-source-build`
- Recipe SHA:
  `0bddf41e6539849c1d8e0a44954ebb3293f6eaa08081741c4adbf3c777fe808c`
- Docker image:
  `soma-emacs-rootfs:0bddf41e6539849c`
- Image id:
  `sha256:9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d349d7bd6b6bb80`
- Selected rootfs store id:
  `soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Rootfs contract:
  `.scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34/etc/soma-insula-rootfs-contract.json`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Emacs executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Emacs source archive SHA256:
  `b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9`,
  checked before extraction during materialization and rechecked by the
  validator.
- Current trace summary:
  `probes/emacs-internals/traces/latest/summary.md`

The selected rootfs contains source-built GNU Emacs `30.2`, rootfs-owned Doom at
commit `8b475fcb2574286d7f4413c22cc7902d80d21ced`, Python/Jupyter/ipykernel,
Pyright, `pylsp`, `clangd`, C/C++ compilers, Rust `1.81.0`, `cargo`, and
`rust-analyzer`.

The selected-rootfs runners now fail closed through
`validate-emacs-rootfs-contract.py` before benchmark work. The validator checks
that the rootfs contract is CUDA-image-derived, rootfs-owned, non-Nix,
container-glibc-based, and backed by a stock GNU Emacs source build under
`/opt/soma/emacs` with retained source archive/tree, configure flags,
dynamic-loader/DSO evidence, rootfs-owned Doom, and required LSP/Jupyter and
language toolchains.

The validator rejects superseded v2/v3/v4/v5 rootfs materializations. v2
recorded Emacs's ignored `--with-json` configure option. v3 self-recorded the
downloaded Emacs source hash and lacked required-tool realpath/hash metadata.
v4 records JSON support as a runtime capability (`json-parse-string`), pins the
GNU Emacs source archive hash before extraction, and records executable
realpaths plus hashes for required tools. v5 adds ELF, dynamic-loader, and
`ldd` evidence for native tools in the production tool inventory. v6 keeps that
contract and adds the Rust `rust-src` component so `rust-analyzer` can resolve a
rootfs-owned sysroot for source benchmarks. v7 adds `emacsclient` to the
production-required tool inventory for daemon terminal-frame evidence. v8 adds
the rootfs tree manifest and hash to the selected-rootfs identity.

The latest selected-rootfs Atlas run reports 35 probes total and 35 passing
under `selected-container-rootfs`. New or strengthened evidence includes:

- executable realpath/version/hash/ldd provenance for production-required tools
- Python introspection plus tiny C and Rust compile/run smoke tests
- Jupyter kernelspec identity through `jupyter_client`
- a real Python Jupyter kernel request with matching shell reply, IOPub idle,
  kernel-side `sys.executable`, `sys.prefix`, `cwd`, plan path, and run id
- stdio initialize/shutdown for `clangd`, `pyright-langserver`, and
  `rust-analyzer`
- long-lived Comint shell cwd/tool drift proof
- timeout classified as `outcome_unknown` with a non-idempotent side effect
  verified exactly once

The latest selected-rootfs benchmark runs report:

- 11 Comint scenarios total and 11 as expected through
  `benchmarks/emacs-gym/comint/scripts/run-benchmark.sh`
- 7 Magit scenarios total and 7 as expected through
  `benchmarks/emacs-gym/magit/scripts/run-benchmark.sh`
- 8 existing-kernel Jupyter scenarios total and 8 as expected through
  `benchmarks/emacs-gym/jupyter/scripts/run-benchmark.sh`
- 6 LSP source scenarios total and 6 as expected through
  `benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh`
- 6 Org structure scenarios total and 6 as expected through
  `benchmarks/emacs-gym/org/scripts/run-benchmark.sh`
- 8 Dired filesystem scenarios total and 8 as expected through
  `benchmarks/emacs-gym/dired/scripts/run-benchmark.sh`, including stale
  same-path mark revalidation by filesystem signature
- 16 bridge transport scenarios total and 16 as expected through
  `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh`, including passive
  read-purity, checked command dispatch, selected-window command drift
  rejection, checked edit/save, stale-edit conflict rejection, and
  point/narrowed/mark-active/read-only checked-edit rejection, checked
  completion acceptance, stale completion rejection, and `episode.finish` final
  evidence validation
- 27 live daemon bridge scenarios total and 27 as expected through
  `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh`, including a
  rootfs-owned `emacsclient --tty` terminal client frame, real tmux-delivered
  terminal key dispatch into that frame, one waitable in-flight request through
  `request.status` and `request.await`, `outcome_unknown` timeout plus later
  reconciliation, an explicit `waiting_for_input` request/respond lifecycle,
  checked point navigation, a real terminal minibuffer prompt opened by queued
  daemon `command.dispatch` and completed by tmux input, real terminal-key
  Magit hunk staging from the daemon-owned terminal frame, queued Magit hunk
  staging through daemon `command.dispatch`, Magit Transient prompt-state
  observation/cancel from real terminal input, Magit Transient Stage suffix
  execution with observed target-file scope, Magit Diff Transient `--stat`
  infix state transition with diff-buffer evidence and unchanged Git state, a package-owned Magit
  `magit-stage-files` prompt completed by tmux input with preserved command
  identity and hook evidence, stale Magit view recovery through model-facing
  `magit.observe_sections`, stale repository-fingerprint rejection, refreshed
  section re-selection, and checked `magit-stage`, `episode.finish` rejection
  while a required request is still running, and one live Comint terminal-input
  path through `comint-send-input` with process identity, input-marker, run-id,
  and exactly-once side-effect evidence, plus one daemon existing-kernel
  Jupyter eval-region path with associated-source observation, request
  status/await completion, rootfs-owned Python/Jupyter runtime evidence,
  preserved kernel token, and exactly-once selected-region execution, plus
  daemon stale `completion.accept` rejection after terminal-frame human editing
  with conflict evidence and preserved final buffer text, plus daemon Python LSP
  rename through real `lsp-rename`, `textDocument/rename`, lsp-mode workspace
  edit application, rootfs-owned `pylsp`/Python identity, multi-file update
  checks, and rootfs Python tests after the rename, plus terminal/minibuffer
  `command.dispatch -> lsp-rename` evidence with prompt observation, tmux input,
  original-request terminal status polling, and passive post-command source
  observation, plus `daemon-multi-async-status-await` evidence for two
  daemon-owned async requests with independent status/results awaited out of
  submission order, plus `daemon-package-async-jupyter-magit` evidence for a
  delayed existing-kernel Jupyter eval-region request and queued Magit staging
  request completing without request/result swap or duplicate kernel execution,
  plus `daemon-comint-terminal-interrupt-reconcile` evidence for terminal-frame
  `C-c C-c` invoking `comint-interrupt-subjob`, preserving the rootfs-owned
  shell process, and preventing the interrupted command's side effect
- 8 benchmark families and 90 scenarios total as expected through
  `scripts/insula/run-emacs-gym-benchmarks-rootfs.sh`, with aggregate
  current-selection checks against each child benchmark's `insula-plan.json`

This establishes the selected Vaso-style materialization baseline for Soma:
Docker is a materializer/exporter for a CUDA-image-derived rootfs, GNU Emacs is
built from the verified GNU source tarball inside that rootfs, Doom and package
state are synced inside that rootfs, and benchmark execution consumes the
exported tree through `bwrap`. Spack is therefore not required to prove the
first source-built Emacs Insula; it remains the stronger follow-up path for
dependency-DAG, compiler, source, binary-cache, and ABI provenance.

## Remaining Work

- Add a richer materialization manifest or Spack-backed equivalent that records
  dependency DAGs, compiler target metadata, tree hashes, package source
  provenance, and ABI surfaces beyond the first rootfs contract.
- Add driver/device projection evidence for CUDA tasks that actually execute GPU
  code inside the Insula.
- Add richer terminal key-dispatch probes for Ghostty/SSH/tmux/Doom/Evil
  behavior beyond the first tmux-to-emacsclient buffer-switch and Magit staging
  proofs.
- Add remaining package workflow benchmarks and variants: interactive
  `lsp-mode` rename UX, additional LSP workspace-edit/code-action variants,
  additional `emacs-jupyter` display variants and recovery coverage, richer Org
  capture/agenda/external-Babel workflows, Dired partial-failure/prompt
  variants, richer PTY Comint prompt/process-control variants, and richer
  daemon-owned async bridge semantics beyond the first cross-package
  Jupyter/Magit package-operation case.

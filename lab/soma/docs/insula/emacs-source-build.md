# Soma Emacs Source-Build Rootfs

This note records the current Soma decision for Emacs materialization.

Soma uses a Vaso-style rootfs path for the first production benchmark baseline:
Docker materializes and exports a CUDA-image-derived filesystem, then benchmark
and probe execution consume that filesystem through `bwrap`. Docker is not part
of scored runtime execution.

Spack is not required for the first Emacs baseline. It remains a candidate
follow-up layer for richer native dependency DAG, compiler, source,
binary-cache, install-view, and ABI provenance. If used, it must run inside the
same container-rootfs ABI boundary and must not provide glibc, the dynamic
loader, or undeclared host/Nix ABI surfaces to production benchmark runs.

## Selected Rootfs

- Builder: `scripts/insula/build-emacs-rootfs.sh`
- Validator: `scripts/insula/validate-emacs-rootfs-contract.py`
- Materialization kind: `source-built-emacs-v8` / `docker-cuda-source-build`
- Base image:
  `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`
- Selected store id:
  `soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Recipe SHA256:
  `0bddf41e6539849c1d8e0a44954ebb3293f6eaa08081741c4adbf3c777fe808c`
- Docker image tag:
  `soma-emacs-rootfs:0bddf41e6539849c`
- Docker image id:
  `sha256:9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d349d7bd6b6bb80`
- Rootfs contract:
  `.scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34/etc/soma-insula-rootfs-contract.json`
- Rootfs tree manifest:
  `/etc/soma-insula-rootfs-tree-manifest.jsonl`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Rootfs tree entries: `152105`

## Rootfs Artifact Storage

The materialized rootfs payload is too large for git. Soma records artifact
identity and storage handoff metadata in
[Rootfs Artifact Storage Handoff](rootfs-artifact-storage.md) and routes local
planning through `scripts/insula/run-rootfs-artifact-storage.sh`.

Current artifact target:

- Storage owner: Waystone (`~/workspace/waystone`)
- Provider: `tos`
- Bucket name: `example-artifacts`
- Site: `example-region`
- Profile hint: `soma-insula-artifacts`
- Object prefix: `soma/insula-rootfs/`

Soma request files are credential-free. Any TOS vault, numeric bucket id,
bytedcli profile, or HDFS credential state belongs to Waystone.

Safe inspection:

```sh
scripts/insula/run-rootfs-artifact-storage.sh plan
```

Local packaging:

```sh
scripts/insula/run-rootfs-artifact-storage.sh pack
```

The generated `*.waystone-storage-request.json` is the handoff to Waystone for
effectful upload or download. Soma must not run `bytedcli tos` traffic from
repo-owned scripts. Until Waystone has a committed TOS interface, this handoff
is not completed transfer evidence.

Downloaded rootfs payloads must be unpacked under `rootfs-store/content/<store-id>`
and validated with `validate-emacs-rootfs-contract.py --verify-rootfs-tree`
before changing `selected.json`.

## Emacs Build Contract

- Source: GNU release tarball
  `https://ftp.gnu.org/gnu/emacs/emacs-30.2.tar.xz`
- Source archive in rootfs: `/opt/soma/src/emacs-30.2.tar.xz`
- Source tree in rootfs: `/opt/soma/build/emacs-30.2/`
- Source archive SHA256:
  `b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9`
- Source SHA policy: checked before extraction during materialization and
  rechecked by the rootfs validator.
- Install prefix: `/opt/soma/emacs`
- Executable: `/opt/soma/emacs/bin/emacs`
- Executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Patch variant: `stock`
- Patch set: empty
- Configure flags:
  `--prefix=/opt/soma/emacs --with-native-compilation --with-tree-sitter --with-sqlite3 --without-x --without-sound --without-dbus --without-gconf --without-gsettings --without-toolkit-scroll-bars --without-xwidgets --without-mailutils`
- Runtime capability checks:
  `json-parse-string=t`, `native-comp-available-p=t`,
  `treesit-available-p=t`

## ABI Boundary

- ABI base: `container-image-rootfs`
- Glibc owner: `container-image-rootfs`
- Glibc version: `glibc 2.39`
- Emacs dynamic loader: `/lib64/ld-linux-x86-64.so.2`
- Forbidden ABI prefixes: `/nix`, `/data02/home`
- Required native tools have rootfs-owned paths, realpaths, executable SHA256
  values, ELF/interpreter metadata where applicable, and `ldd` evidence in the
  rootfs contract.

The validator rejects missing source archives, missing retained source trees,
wrong Emacs source hashes, unrecognized configure options, missing capability
records, missing required tools, host/Nix ABI prefixes, missing dynamic loader
paths, and missing shared-library dependencies.

## Tool And Package Closure

The selected rootfs owns:

- Doom core and generated profile state under `/opt/soma/doom-home`
- Magit, Org+Jupyter, LSP, Python, Rust, C/C++, shell, and Dired support for
  the current minimal Doom profile
- Jupyter, `ipykernel`, and the Python kernel used by the existing-kernel
  benchmark
- `pyright`, `pyright-langserver`, `pylsp`, `clangd`, and `rust-analyzer`
- Python `3.12.3`, GCC/G++ `13.3.0`, Clang/clangd `18.1.3`, Rust `1.81.0`,
  and `rust-src`
- Git, shell, `emacsclient`, tmux, ripgrep, fd, jq, and shellcheck

## Evidence Commands

```sh
scripts/insula/validate-emacs-rootfs-contract.py \
  .scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34 \
  --verify-rootfs-tree

probes/emacs-internals/scripts/run-probes.sh

workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh

scripts/insula/run-emacs-gym-benchmarks-rootfs.sh
```

Current evidence from the selected rootfs:

- Rootfs contract validation exits successfully with `"ok": true`, including
  rootfs tree-manifest recomputation.
- Internals probes report 36 probes total and 36 passing.
- Doom lifecycle worker passes cold, warmed, cold-repeat-1, and
  warmed-repeat-1 child Emacs checks, with teardown probes showing no
  token-marked child processes left after exit.
- Focused bridge transport reports 16 scenarios total and 16 scenarios as
  expected, including passive buffer reads, checked command dispatch,
  selected-window command drift rejection, checked range edits, checked saves,
  stale edit rejection, point/narrowed/mark-active/read-only state drift
  rejection, checked `completion.accept`, stale completion rejection, and
  `episode.finish` final evidence validation.
- Focused daemon bridge transport reports 28 scenarios total and 28 scenarios as
  expected, including a rootfs-owned `emacsclient --tty` terminal frame,
  real tmux-delivered terminal key dispatch into that frame, in-flight request
  status, bounded await, `outcome_unknown` timeout plus later reconciliation,
  explicit `waiting_for_input` prompt response, checked point navigation, a real
  terminal minibuffer prompt opened by queued daemon `command.dispatch` and
  completed by tmux input, real terminal-key Magit hunk staging from the
  daemon-owned terminal frame, queued Magit hunk staging through daemon
  `command.dispatch`, Magit Transient prompt-state observation/cancel from real
  terminal input, Magit Transient Stage suffix execution with observed
  target-file scope, Magit Diff Transient `--stat` infix state transition with
  diff-buffer verification, a package-owned Magit `magit-stage-files` prompt completed
  by tmux input with `this-command` and hook evidence preserved, stale Magit
  view recovery through model-facing `magit.observe_sections`, stale
  repository-fingerprint rejection, refreshed section re-selection, and checked
  `magit-stage`, and `episode.finish` rejection while a required request is
  still running, plus a
  live Comint terminal-input scenario that submits through `comint-send-input`
  to a rootfs-owned shell and verifies process identity, Comint input markers,
  run-id output, and exactly-once side effects, plus a daemon existing-kernel
  Jupyter eval-region scenario with associated-source observation, daemon
  request status/await completion, rootfs-owned Python/Jupyter runtime evidence,
  preserved kernel token, and exactly-once selected-region execution, plus
  UI-rendered Jupyter display-update presentation through
  `jupyter.eval_region_via_repl`, REPL-owned display-id range observation,
  checked terminal navigation to that range, and tmux capture of
  `SOMA_DISPLAY_STAGE:updated`, plus
  daemon stale `completion.accept` rejection after human terminal editing with
  conflict evidence and preserved final buffer text, plus daemon Python LSP
  rename through both semantic `lsp.rename_symbol` and terminal/minibuffer
  `command.dispatch -> lsp-rename` paths, `textDocument/rename`, lsp-mode
  workspace edit application, rootfs-owned `pylsp`/Python identity, multi-file
  update checks, rootfs Python tests after the rename, and
  `daemon-multi-async-status-await` coverage for two independent daemon-owned
  async requests awaited out of submission order without result swapping, plus
  `daemon-package-async-jupyter-magit` coverage for a delayed existing-kernel
  Jupyter eval request that remains independently observable while a queued
  checked Magit package command completes first, plus
  `daemon-comint-terminal-interrupt-reconcile` coverage for terminal-frame
  `C-c C-c` invoking `comint-interrupt-subjob`, preserving rootfs-owned shell
  process identity, and preventing the interrupted command's side effect.
- Focused Org structure reports 6 scenarios total and 6 scenarios as expected,
  including source-edit writeback to the parent Org buffer, refile move
  metadata preservation, duplicate-ID rejection, visible-text-only move
  rejection, and Babel result insertion in the parent document.
- Focused Dired filesystem reports 8 scenarios total and 8 scenarios as
  expected, including marked-delete behavior, deletion-flag confusion rejection,
  rename collision rejection, overwrite rejection, and Dired-undo-as-filesystem
  recovery rejection, plus stale same-path mark revalidation by filesystem
  signature.
- Focused Jupyter existing-kernel reports 8 scenarios total and 8 scenarios as
  expected, including `display_data` and `update_display_data` parent-id,
  display-id, and payload ordering evidence.
- Focused Comint process benchmark reports 11 scenarios total and 11 scenarios
  as expected, including process-mark reobservation after asynchronous output,
  stale process-mark submission rejection, partial-output completion waiting,
  premature partial-output success rejection, and interrupt reconciliation.
- Aggregate selected-rootfs benchmark reports 8 benchmark families, 90
  scenarios total, and 90 scenarios as expected. The latest run id is
  `20260929T040533Z-3771476`.
- The aggregate runner validates every child benchmark's `insula-plan.json`
  against the currently selected rootfs store id, rootfs path, contract path,
  recipe SHA, base image, image tag, image id, and rootfs tree-manifest identity
  before declaring the aggregate passed. For every benchmark family, it
  also requires `soma_insula_authority=scripts/insula/soma_insula.py`.
- `scripts/insula/soma_insula.py` now emits the selected-rootfs plan, bwrap
  mount arguments, environment allowlist, and entrypoint argv for the generic
  command runner. Magit, Comint, Jupyter, LSP, Org, Dired, the batch bridge,
  live daemon bridge, Internals Atlas probes, and the Doom lifecycle worker
  route through this module.

## Remaining Provenance Work

The source-built baseline is sufficient for current production-style batch
evidence. It does not close the richer materialization backlog:

- package-level source and binary-cache provenance beyond the current tool
  inventory
- compiler target metadata and native dependency DAG
- promoted immutable Spack or equivalent install views, if needed
- CUDA driver/device projection evidence for GPU-consuming tasks
- broader Transient infix behavior, richer package-specific prompts, and richer
  daemon-owned async package-operation evidence

Those additions should augment or replace the materialization layer without
weakening the selected-rootfs runtime contract.

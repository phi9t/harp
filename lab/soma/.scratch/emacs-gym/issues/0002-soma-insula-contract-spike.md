# 0002: Implement The Soma Insula Contract Spike

Status: Evidence captured; package workflow gates remain
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas

## Intent

Replace the development minimal-bind Insula with a pinned `soma`-owned rootfs
contract for Emacs internals probes and future gym tasks.

Hermetic means the selected rootfs owns the tools, packages, dependencies, and
ABI surface used by the task. It is not enough to enter `bwrap`, clear the
environment, or disable network while still resolving Emacs, LSP servers,
Jupyter, Python, C/C++/Rust toolchains, build tools, or shared libraries from
the host.

## Requirements

- Define the rootfs metadata contract, selected-rootfs file, and validation
  rules.
- Define the dependency-closure and ABI manifest contract for the selected
  rootfs.
- Define the rootfs materialization strategy. The first executed path is a
  CUDA-image-derived source-build rootfs; Spack or another richer layer can
  follow for dependency DAG, compiler, shared-library, and source/binary
  provenance.
- Refuse symlink escapes and stale/missing rootfs selections.
- Clear the environment and set only an explicit allowlist.
- Provide private HOME, TMPDIR, XDG cache/data/runtime dirs, and run dirs.
- Ensure Emacs, Doom core/profile/package closure, Git/shell tooling, Jupyter
  server/kernels, Python packages, Python/C++/Rust compilers, LSP servers,
  build/test tools, and ABI-relevant shared libraries resolve from the selected
  rootfs unless a host projection is explicitly declared.
- Bind Doom core, user config, package sources, tool caches, and task fixtures
  according to Atlas-derived read/write ownership rules.
- Emit a bwrap plan and contract digest before execution.
- Provide rootfs sentinel checks stronger than environment variables alone:
  working directory, entrypoint identity, selected manifest, and mount plan.
- Provide an extension point for subprocess/kernel launchers, especially
  Jupyter kernels, so Emacs-side and kernel-side execution can prove the same
  rootfs contract.
- Treat network as an explicit phase/task policy, not as the definition of
  hermeticity. Networked materialization may be allowed if it produces a new
  recorded rootfs manifest; scored benchmark episodes must not mutate the
  dependency closure invisibly.
- Keep package installation and dependency resolution in materialization phases:
  `spack install`, `pip install`, `cargo install`, and similar resolution must
  not occur invisibly inside scored benchmark episodes.
- Preserve the current `probes/emacs-internals/scripts/run-probes.sh` public
  entrypoint.

## Captured Evidence

Captured on 2026-09-27 and refreshed on 2026-09-29:

- `scripts/insula/build-emacs-rootfs.sh` materializes a CUDA-image-derived
  rootfs with source-built GNU Emacs `30.2`, rootfs-owned Doom, Python/Jupyter,
  Pyright, `pylsp`, `clangd`, C/C++ compilers, Rust `1.81.0`, `cargo`, and
  `rust-analyzer`.
- `scripts/insula/run-emacs-internals-probes-rootfs.sh` validates
  `selected.json`, rejects `/nix/store` in the rootfs contract, mounts the
  selected rootfs read-only at `/`, mounts rootfs-owned Doom into
  `/project/home`, clears the environment, emits `insula-plan.json`, and runs
  batch probes inside `bwrap`.
- `probes/emacs-internals/scripts/run-probes.sh` now uses the selected-rootfs
  runner by default. `SOMA_USE_DEVELOPMENT_INSULA=1` is the explicit fallback
  for regenerating development-minimal traces.
- `probes/emacs-internals/traces/latest/summary.md` reports 35 probes total and
  35 passing under `selected-container-rootfs`.
- `workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh` emits cold, warmed,
  cold-repeat-1, and warmed-repeat-1 source-built Emacs/Doom lifecycle traces
  under the same selected rootfs, with per-child teardown probes showing no
  preexisting token process and no token-marked process remaining after exit.
- `benchmarks/emacs-gym/comint/scripts/run-benchmark.sh` reports 11 Comint
  benchmark scenarios total and 11 as expected under the same selected rootfs,
  including process-mark drift reobservation, stale process-mark rejection,
  partial-output completion waiting, premature partial-output success
  rejection, and interrupt reconciliation.
- `benchmarks/emacs-gym/magit/scripts/run-benchmark.sh` reports 7 Magit
  selective-staging benchmark scenarios total and 7 as expected under the same
  selected rootfs.
- `benchmarks/emacs-gym/jupyter/scripts/run-benchmark.sh` reports 8
  existing-kernel Jupyter benchmark scenarios total and 8 as expected under the
  same selected rootfs, including protocol-level display/update evidence.
- `benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh` reports 6 LSP source
  benchmark scenarios total and 6 as expected under the same selected rootfs.
- `benchmarks/emacs-gym/org/scripts/run-benchmark.sh` reports 6 Org structure
  benchmark scenarios total and 6 as expected under the same selected rootfs.
- `benchmarks/emacs-gym/dired/scripts/run-benchmark.sh` reports 8 Dired
  filesystem benchmark scenarios total and 8 as expected under the same
  selected rootfs, including stale same-path mark revalidation by filesystem
  signature.
- `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh` reports 16 bridge
  transport scenarios total and 16 as expected under the same selected rootfs,
  including passive read-purity, checked command dispatch, selected-window
  command drift rejection, checked edit/save, stale-edit conflict rejection,
  point/narrowed/mark-active/read-only checked-edit rejection, checked
  completion acceptance, stale completion rejection, and `episode.finish` final
  evidence validation.
- `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` reports 27 live
  daemon bridge scenarios total and 27 as expected under the same selected
  rootfs, including rootfs-owned `emacsclient --tty` terminal-frame attachment,
  real tmux-delivered terminal key dispatch into that frame, one waitable
  in-flight request through `request.status` and `request.await`,
  `outcome_unknown` timeout plus later reconciliation, an explicit
  `waiting_for_input` request/respond lifecycle, checked point navigation,
  a real terminal minibuffer prompt opened by queued daemon `command.dispatch`
  and completed by tmux input, real terminal-key Magit hunk staging from the
  daemon-owned terminal frame, queued Magit hunk staging through daemon
  `command.dispatch`, Magit Transient prompt-state observation/cancel from real
  terminal input, Magit Transient Stage suffix execution with observed
  target-file scope, Magit Diff Transient `--stat` infix state transition with
  diff-buffer evidence and unchanged Git state, a package-owned Magit
  `magit-stage-files` prompt completed by tmux input with preserved command
  identity and hook evidence, stale Magit
  view recovery through model-facing `magit.observe_sections`, stale
  repository-fingerprint rejection, refreshed section re-selection, and checked
  `magit-stage`, and `episode.finish` rejection while a required request is
  still running, plus one daemon existing-kernel Jupyter eval-region path with
  associated-source observation, request status/await completion,
  rootfs-owned Python/Jupyter runtime evidence, preserved kernel token, and
  exactly-once selected-region execution, plus daemon stale `completion.accept`
  rejection after terminal-frame human editing with conflict evidence and
  preserved final buffer text, plus daemon Python LSP rename through both
  semantic `lsp.rename_symbol` and terminal/minibuffer
  `command.dispatch -> lsp-rename`, `textDocument/rename`, lsp-mode workspace
  edit application, rootfs-owned `pylsp`/Python identity, multi-file update
  checks, rootfs Python tests, prompt observation, original-request terminal
  status polling, passive post-command source observation, and
  `daemon-multi-async-status-await` evidence for independent daemon-owned
  request status/results when two async requests are awaited out of submission
  order, plus `daemon-package-async-jupyter-magit` evidence for a delayed
  existing-kernel Jupyter eval-region request and queued Magit staging request
  completing with independent request/result identity and no duplicate kernel
  execution, plus `daemon-comint-terminal-interrupt-reconcile` evidence for
  terminal-frame `C-c C-c` invoking `comint-interrupt-subjob`, preserving the
  rootfs-owned shell process, and preventing the interrupted command's side
  effect.
- `scripts/insula/run-emacs-gym-benchmarks-rootfs.sh` reports 8 benchmark
  families, 90 scenarios total, and 90 scenarios as expected under the same
  selected rootfs in run `20260929T040533Z-3771476`, with aggregate
  current-selection checks against each child benchmark's `insula-plan.json`,
  including rootfs tree-manifest identity and required `soma_insula.py`
  authority for every benchmark family.
- `scripts/insula/soma_insula.py` is the first repo-owned Insula authority
  module. It is unit-tested for selected-rootfs escape rejection, network
  authorization, generic batch plan emission, bwrap mount evidence, environment
  allowlist evidence, output-root enforcement, and Nix contract rejection. The
  generic batch runner and the Magit, Comint, Jupyter, LSP, Org, Dired, and
  batch bridge wrappers now use it. The live daemon bridge wrapper now also
  uses its typed daemon/tmux/socket runtime fields and `run-command` path, with
  validated trace evidence under
  `benchmarks/emacs-gym/bridge/traces/soma-insula-module-daemon-green2/`. The
  Internals Atlas probe wrapper and Doom lifecycle worker wrapper also use its
  `run-command` path, with validated traces under
  `probes/emacs-internals/traces/soma-insula-module-command-green1/` and
  `workers/emacs-doom-lifecycle/traces/lifecycle-teardown-green2/`.

The evidence covers selected rootfs identity, rootfs-owned tool resolution,
executable provenance, source-built Emacs identity, cold/warmed plus repeated
Doom startup, selected-profile module activation, teardown probes, C/Rust smoke
builds, Jupyter kernelspec and real kernel execution, LSP stdio initialization for
Pyright/clangd/rust-analyzer, Comint rootfs identity, long-lived shell state,
timeout-as-unknown behavior, and the first selected-rootfs benchmark grader
loop for Comint process submission, plus the first live daemon terminal-frame
Comint submission through `comint-send-input` with rootfs-owned process,
input-marker, run-id, and exactly-once side-effect evidence. It also covers
batch Magit selective
staging through rootfs-owned Magit and the first synchronous checked bridge
command-dispatch slice, exact Git index/worktree checks, refresh and post-stage
hook evidence, existing-kernel `emacs-jupyter` association/execution with
duplicate-timeout rejection, real Emacs `lsp-mode` source fixture coverage for
Python/C++/Rust, a batch-safe model-facing request envelope with JSONL
Unix-socket round-trip smoke, Org parent-buffer/source-edit/refile/Babel
checks, Dired filesystem/mark/rename/undo checks, checked buffer read/edit/save
semantics with stale-edit rejection, a live daemon/external-client JSONL bridge
smoke, one timer-backed daemon async status/await proof, a package-owned Magit
prompt path through the daemon bridge, daemon stale completion conflict
rejection after terminal-frame human editing, and targeted bad-policy
rejection.

The lifecycle evidence also records a profile boundary: the selected rootfs
Doom profile enables Magit, LSP, Org+Jupyter, Python, Rust, C/C++, and shell
support, but not Evil, Corfu, Vertico, Popup, or Doom's TTY module. Those absent
capabilities need a different selected profile or rootfs materialization before
they can be treated as ready.

## Verification

- Running the probe suite records a pinned rootfs identity instead of
  `development-minimal-binds`.
- The emitted plan records dependency-closure ownership, ABI manifest identity,
  required tool inventory, network policy, and any allowed host projections.
- The materialization evidence records Spack concretization or equivalent build
  identity, compiler selection, source/build provenance, binary-cache
  provenance, and exported rootfs/view identity.
- A missing, stale, or escaping rootfs selection fails before starting Emacs.
- A synthetic subprocess/kernel preflight proves cwd, Python executable/prefix,
  package path, run id, and bwrap plan from inside the rootfs.
- `emacs --batch` reports rootfs-owned Emacs, Doom package paths, native-comp
  paths, and cache/runtime dirs.
- `pyright-langserver`, `clangd`, and `rust-analyzer` resolve inside the rootfs
  and complete stdio `initialize`/`shutdown` protocol probes against temporary
  source roots. The first Emacs `lsp-mode` source benchmark is captured in
  issue 0008; the live daemon bridge now adds one model-facing Python rename via
  real `lsp-rename`, `textDocument/rename`, lsp-mode workspace edit
  application, and terminal/minibuffer command-path rename evidence. Broader
  LSP stress cases remain follow-up work.
- Jupyter kernelspec discovery and one real rootfs Python kernel request report
  kernel-side rootfs identity: cwd, `sys.executable`, `sys.prefix`, import
  path, run id, request parent header, and selected manifest. The first
  existing-kernel `emacs-jupyter` association benchmark with duplicate-timeout
  rejection is captured in issue 0006; terminal-frame and richer display/recovery
  variants remain follow-up work.
- Comint subprocess fixtures report matching rootfs identity and tool
  resolution before their output is accepted by the grader.

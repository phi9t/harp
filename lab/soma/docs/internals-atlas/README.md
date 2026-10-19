# Emacs/Doom Internals Atlas

This Atlas is the pre-implementation gate for the Doom Emacs gym. Its job is to
turn source reading and executable mechanism probes into constraints for the
gym action/observation interface.

The central question for every supported operation is:

> Starting from this exact session state, what code runs, what implicit context
> does it consume, what state can it change, what asynchronous work does it
> initiate, and what evidence establishes completion?

## Current Authority

The current local target inspected for this pass is:

- Insula mode: `selected-container-rootfs`
- Rootfs: `.scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Materialization kind: `source-built-emacs-v8`
- Rootfs recipe SHA256:
  `0bddf41e6539849c1d8e0a44954ebb3293f6eaa08081741c4adbf3c777fe808c`
- Base image: `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Emacs: `/opt/soma/emacs/bin/emacs`
- Emacs version: `30.2`
- Emacs executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Emacsclient executable SHA256:
  `dbce94539b0ee6db6cb50078162896ea35ab8ef80e2c7d5c02dbf58f16aa315a`
- Emacs features: `ACL GMP GNUTLS LIBSELINUX LIBXML2 MODULES NATIVE_COMP NOTIFY INOTIFY PDUMPER SECCOMP SQLITE3 THREADS TREE_SITTER ZLIB`
- Emacs source directory: `/opt/soma/build/emacs-30.2/`
- Doom core: `/project/home/.emacs.d` at `8b475fcb2574286d7f4413c22cc7902d80d21ced`
- Doom user config: `/project/home/.doom.d`
- Generated Doom module state: `/project/home/.emacs.d/.local/etc/@/init.30.d/95-modules.auto.el`
- Probe traces: `probes/emacs-internals/traces/latest/`
- Comint benchmark traces: `benchmarks/emacs-gym/traces/latest/comint/`
- Magit benchmark traces: `benchmarks/emacs-gym/traces/latest/magit/`
- Jupyter benchmark traces: `benchmarks/emacs-gym/traces/latest/jupyter/`
- LSP benchmark traces: `benchmarks/emacs-gym/traces/latest/lsp/`
- Aggregate benchmark traces: `benchmarks/emacs-gym/traces/latest/`
- Raw research notes: `.scratch/emacs-gym/research/`

The selected rootfs source-builds GNU Emacs and retains the matching source
tree at the `source-directory` reported by Emacs. C-level Atlas claims can now
be grounded in that rootfs source tree, while package-specific claims should
continue to cite rootfs-owned Doom/Straight sources and executed runtime probes.
The aggregate benchmark runner also verifies each child benchmark
`insula-plan.json` against the current selected rootfs identity before
declaring the combined run passed. For every benchmark family, it also
requires `soma_insula_authority=scripts/insula/soma_insula.py`.
The Magit, Comint, Jupyter, LSP, Org, Dired, batch bridge, live daemon bridge,
Internals Atlas probe, and Doom lifecycle worker plans are now emitted by
`scripts/insula/soma_insula.py`, which records the bwrap mount args,
environment allowlist, entrypoint argv, and runtime-specific fields as
first-class plan evidence.

## Running Probes

Authoritative probe runs use the selected-rootfs Soma Insula wrapper:

```sh
probes/emacs-internals/scripts/run-probes.sh
```

That command enters `bwrap`, clears the environment, disables network, mounts
the selected CUDA-image-derived rootfs read-only at `/`, uses private
HOME/TMP/RUN/cache directories, mounts rootfs-owned Doom into `/project/home`,
and binds the `soma` checkout writable at `/workspace/soma` for trace output.
It writes:

- `insula-plan.json`
- one JSON trace per probe
- `summary.json`
- `summary.md`

The inner batch runner refuses host execution unless
`SOMA_ALLOW_NON_INSULA_PROBES=1` is set. Non-Insula runs are development-only and
are not benchmark evidence. To regenerate the old host-bind development
classification, run with `SOMA_USE_DEVELOPMENT_INSULA=1`; that path must remain
labeled `not_rootfs_owned` and `abi_guarantee=none`.

Current status: 36 batch-safe mechanism probes and the Insula runtime checks
pass under the selected container rootfs. The run proves rootfs-owned Emacs,
Doom package state, Python/Jupyter, Pyright, clangd, rust-analyzer, C/C++ and
Rust smoke builds, LSP stdio initialization, a real Jupyter kernel request, and
long-lived Comint timeout semantics, plus CAPF provider exit-function
acceptance semantics. It is production-style Atlas evidence for the rootfs
contract, not yet a full interactive terminal-frame benchmark.

## Running The Doom Lifecycle Worker

The rootfs-local Doom lifecycle worker is:

```sh
workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh
```

It uses the selected CUDA-image-derived rootfs, launches source-built Emacs
`/opt/soma/emacs/bin/emacs`, loads rootfs-owned Doom from
`/project/home/.emacs.d`, and writes:

- `workers/emacs-doom-lifecycle/traces/latest/insula-plan.json`
- `workers/emacs-doom-lifecycle/traces/latest/cold.json`
- `workers/emacs-doom-lifecycle/traces/latest/warmed.json`
- `workers/emacs-doom-lifecycle/traces/latest/cold-repeat-1.json`
- `workers/emacs-doom-lifecycle/traces/latest/warmed-repeat-1.json`
- `workers/emacs-doom-lifecycle/traces/latest/summary.json`
- `workers/emacs-doom-lifecycle/traces/latest/summary.md`

Current status: cold, warmed, cold-repeat-1, and warmed-repeat-1 child Emacs
processes exit successfully and their checks pass. Each child launch carries a
unique `SOMA_LIFECYCLE_CHILD_TOKEN`; the lifecycle summary records no
preexisting token process and no token-marked process remaining after exit for
all four phases. The warmed phases prove first-use loading for `magit`, `org`,
`jupyter`, `lsp-mode`, `python`, `rust-mode`, `cc-mode`, and `sh-script`, plus
major-mode activation for Elisp, Org, Python, Rust, C++, and shell buffers.
Capability declarations are based on Doom's loaded `doom-modules` state, not
raw generated-profile text; this matters because disabled-module documentation
can contain strings such as `(:ui . popup)` even when the module is not active.
The selected rootfs profile currently does not enable Corfu, Vertico, Popup, or
Doom's TTY module.

## Running The Comint Benchmark

The first selected-rootfs benchmark slice is the Comint process benchmark:

```sh
benchmarks/emacs-gym/comint/scripts/run-benchmark.sh
```

It uses the selected CUDA-image-derived rootfs. When run directly it writes:

- `benchmarks/emacs-gym/comint/traces/latest/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/comint/traces/latest/summary.json`
- `benchmarks/emacs-gym/comint/traces/latest/summary.md`

The current aggregate evidence is under
`benchmarks/emacs-gym/traces/latest/comint/`.

Current status: 11 scenarios total and 11 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark accepts
the scripted policy that submits live input through `comint-send-input` and
rejects raw process sends, old-history submission, wrong-buffer output
ownership, stale process-mark submission after asynchronous output, and
retrying a non-idempotent command after a timeout classified as
`outcome_unknown`. It also accepts reobserving the process mark after
asynchronous output before submission, waiting for final completion after
partial output, and interrupting after partial output while preserving the
Comint shell for reconciliation.

## Running The Magit Benchmark

The first package-specific selected-rootfs benchmark slice is:

```sh
benchmarks/emacs-gym/magit/scripts/run-benchmark.sh
```

It uses the selected CUDA-image-derived rootfs, loads rootfs-owned Magit from
the Doom/Straight build tree, creates disposable Git repositories inside
`/project/tmp`. When run directly it writes:

- `benchmarks/emacs-gym/magit/traces/latest/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/magit/traces/latest/summary.json`
- `benchmarks/emacs-gym/magit/traces/latest/summary.md`

The current aggregate evidence is under
`benchmarks/emacs-gym/traces/latest/magit/`.

Current status: 7 scenarios total and 7 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark accepts
scripted policies that stage selected Magit hunk, file, and active-region file
sections through the first synchronous checked bridge command-dispatch route,
while preserving unrelated staged and unstaged state. It rejects raw
`git add -u`, bridge-wrapper direct `magit-stage-1`, unstaged-list staging, and
stale-view staging after an external edit. This is batch-safe package and
bridge evidence; daemon-routed Magit hunk staging now has first queued-package
operation coverage in the live daemon benchmark.

## Running The Bridge Transport Benchmark

The model-facing request-envelope smoke is:

```sh
benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh
```

The aggregate selected-rootfs run writes the current evidence under:

- `benchmarks/emacs-gym/traces/latest/bridge/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/traces/latest/bridge/summary.json`
- `benchmarks/emacs-gym/traces/latest/bridge/summary.md`

Current status: 16 scenarios total and 16 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark accepts
request-envelope `emacs.observe_buffer` and checked `command.dispatch`, stores
terminal request status by request id, rejects duplicate request ids, rejects
unchecked direct-call operations, verifies checked `buffer.read`,
`buffer.replace_range`, `buffer.save`, stale-edit conflict rejection,
selected-window command drift rejection, point edit drift rejection, narrowed
buffer state drift rejection, active-region state drift rejection, read-only
state drift rejection, checked `completion.accept`, stale completion acceptance
rejection after intervening human editing, `episode.finish` final evidence
validation, and a JSONL Unix-domain socket round trip inside the selected
source-built rootfs.

The live daemon bridge smoke is:

```sh
benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh
```

The aggregate selected-rootfs run writes that evidence under:

- `benchmarks/emacs-gym/traces/latest/bridge-daemon/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/traces/latest/bridge-daemon/summary.json`
- `benchmarks/emacs-gym/traces/latest/bridge-daemon/summary.md`

Current status: 28 scenarios total and 28 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark starts
source-built Emacs as a named daemon inside the selected rootfs, starts the
JSONL bridge socket inside that daemon at
`/project/run/soma-gym-bridge-daemon.sock`, and verifies that a separate
rootfs-owned Python client can submit `emacs.observe_buffer`, retrieve stored
terminal status through `request.status`, and observe command-dispatch plus
direct-call rejection under the daemon transport profile. It also attaches
rootfs-owned `emacsclient --tty` through rootfs-owned `tmux`, observes the
resulting daemon-owned terminal client frame through `emacs.observe_frames`,
verifies real tmux-delivered terminal key dispatch by switching that client
frame to the daemon target buffer and observing the selected-frame buffer
through the bridge,
verifies one timer-backed daemon-owned in-flight request whose initial
submission returns `running`, whose in-flight state is visible through
`request.status`, and whose terminal success is observed through `request.await`
without resubmission, verifies two daemon-owned async requests with independent
status/result identity that can be awaited out of submission order without
result swapping, verifies `request.await` timeout as `outcome_unknown`
followed by reconciliation of the original request, verifies an explicit
`waiting_for_input` request/respond lifecycle, verifies checked point navigation
in the selected terminal frame, verifies a real terminal minibuffer prompt opened
by queued daemon `command.dispatch` and completed by tmux input, verifies queued
Magit hunk staging through daemon `command.dispatch` with terminal status
observed through `request.await`, verifies real terminal-key Magit hunk staging
from the daemon-owned terminal frame with post-stage hook and index/worktree
evidence, verifies Magit Transient prompt-state observation and cancellation
from real terminal input without changing fixture Git state, verifies Magit
Transient Stage suffix execution from terminal input with observed target-file
scope and preserved unrelated file state, verifies a Magit Diff Transient
`--stat` infix state transition from terminal input with diff-buffer evidence
and unchanged Git state, verifies a package-owned Magit
`magit-stage-files` minibuffer prompt opened by queued daemon
`command.dispatch`, completed by tmux input, and checked through preserved
`this-command`, post-stage hook identity, index/worktree contents, and unrelated
state preservation, verifies stale Magit view recovery through model-facing
`magit.observe_sections`, stale repository-fingerprint rejection, refreshed
section re-selection, and checked `magit-stage`, and verifies `episode.finish`
rejection before required
requests become terminal. It also verifies a live Comint terminal-input path in
which a rootfs-owned terminal frame submits shell input through
`comint-send-input`, preserves process identity, updates Comint input markers,
passes the Insula run id to subprocess output, and performs the expected side
effect exactly once. It also verifies a live terminal-frame Comint interrupt
path in which `C-c C-c` invokes `comint-interrupt-subjob`, Comint post-command
hook evidence records the interrupt command, the rootfs-owned shell process
survives, and the interrupted command's side effect does not occur. It also
verifies a daemon existing-kernel Jupyter
eval-region path: the terminal frame switches to an associated source buffer,
`jupyter.observe_source` records association and selected-region identity,
`jupyter.eval_region` enters the daemon request lifecycle, `request.status` and
`request.await` observe completion, `jupyter.query_state` records rootfs-owned
Python/Jupyter runtime state, and the grader checks preserved kernel token plus
exactly-once selected-region execution. It also verifies UI-rendered Jupyter
display-update presentation through `jupyter.eval_region_via_repl`: the source
range is submitted through the associated REPL client with
`jupyter-repl-execute-cell`, `jupyter.observe_display` finds the updated
display-id range in `*jupyter-repl[soma-daemon-display-update]*`, checked
terminal point navigation moves to that range, and tmux capture contains
`SOMA_DISPLAY_STAGE:updated`. It also verifies daemon stale
`completion.accept` rejection after human terminal editing: the stale request
fails with `conflict`, failed preconditions include character revision, point,
and suffix, and final buffer text preserves the human edit. It also verifies
daemon LSP diagnostics-after-edit through a dedicated rootfs-owned
`pyright-langserver` fixture: it records rootfs-owned pyright/node/python
executable paths, uses full-document LSP sync with debounce disabled, saves a
guarded live-buffer edit through the bridge before pyright diagnostics are
required, repairs and saves the source through the bridge, and a later
observation sees diagnostics clear under an explicit
`lsp-diagnostic-clean-after-change` fixture policy. It also verifies
daemon Python LSP rename through both semantic `lsp.rename_symbol` and
terminal/minibuffer `command.dispatch -> lsp-rename` paths, and records
`textDocument/rename`, lsp-mode workspace edit application, rootfs-owned
`pylsp`/Python identity, multi-file update checks, rootfs Python tests, terminal
prompt evidence, original-request terminal status polling, and passive
post-command source observation. It also verifies
	`daemon-package-async-jupyter-magit`: a delayed existing-kernel Jupyter eval
	request remains independently observable while a queued checked Magit package
	command completes first, then the original Jupyter request is awaited with
	preserved kernel token and exactly-once execution. Full FIM model serving and
	ranking, richer package-specific prompts, additional Transient infix classes,
	broader package-operation async cases, additional Jupyter display/recovery variants,
remaining checked edit variants, richer LSP rename variants, and
Doom/Evil-specific terminal bindings remain separate gates.

## Running The Jupyter Benchmark

The existing-kernel Jupyter benchmark is:

```sh
benchmarks/emacs-gym/jupyter/scripts/run-benchmark.sh
```

The aggregate selected-rootfs run writes the current evidence under:

- `benchmarks/emacs-gym/traces/latest/jupyter/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/traces/latest/jupyter/summary.json`
- `benchmarks/emacs-gym/traces/latest/jupyter/summary.md`

Current status: 8 scenarios total and 8 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark accepts
the scripted policy that associates an Emacs source buffer with an existing
rootfs-owned kernel and evaluates the selected region once. It rejects missing
association, wrong source buffer, whole-buffer execution, new-kernel continuity
loss, and duplicate retry after a timeout classified as `outcome_unknown`. It
also captures `display_data` and `update_display_data` messages for one request,
including parent ids, shared display id, ordered initial/update payloads, and
unchanged kernel identity. Restart recovery beyond rejection and
terminal-frame/richer async bridge behavior remain separate gates.

## Running The LSP Benchmark

The rootfs-owned LSP source benchmark is:

```sh
benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh
```

The aggregate selected-rootfs run writes the current evidence under:

- `benchmarks/emacs-gym/traces/latest/lsp/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/traces/latest/lsp/summary.json`
- `benchmarks/emacs-gym/traces/latest/lsp/summary.md`

Current status: 6 scenarios total and 6 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark starts
real Emacs `lsp-mode` clients against generated Python, C++, and Rust fixtures
inside the bwrap private project runtime. It uses rootfs-owned LSP servers and
language toolchains, rejects direct text replacement reported as an LSP
refactor, and rejects stale document-version or stale workspace-root actions.
The live daemon bridge now adds model-facing Python rename scenarios through
both semantic `lsp.rename_symbol` and terminal/minibuffer
`command.dispatch -> lsp-rename` paths. These record real `lsp-rename`,
`textDocument/rename`, lsp-mode workspace edit application, rootfs-owned
`pylsp`/Python identity, multi-file update checks, rootfs Python tests, prompt
observation, tmux-supplied rename input, original-request terminal status
polling, and passive post-command source observation. Additional workspace
polling, and passive post-command source observation. It also adds a dedicated
rootfs-owned `pyright-langserver` diagnostics fixture in the daemon benchmark:
guarded buffer replacement plus save creates diagnostics containing
`Expected expression`, a bridge-mediated repair/save restores the source, and
active LSP observation sees diagnostics clear under explicit stale-diagnostic
cleanup policy.
Additional workspace
edits/code actions, multi-root projects, and richer terminal-frame/async bridge
behavior remain separate gates.

## Running The Org Benchmark

The Org structure benchmark is:

```sh
benchmarks/emacs-gym/org/scripts/run-benchmark.sh
```

The aggregate selected-rootfs run writes the current evidence under:

- `benchmarks/emacs-gym/traces/latest/org/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/traces/latest/org/summary.json`
- `benchmarks/emacs-gym/traces/latest/org/summary.md`

Current status: 6 scenarios total and 6 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark accepts
source-edit writeback through `org-edit-src-exit`, `org-refile` subtree moves
that preserve IDs, custom properties, and children, and Babel execution that
inserts results into the parent Org buffer. It rejects source-edit-buffer-only
changes, copy-like refile behavior that duplicates IDs, and visible-text-only
moves that drop metadata.

## Running The Dired Benchmark

The Dired filesystem benchmark is:

```sh
benchmarks/emacs-gym/dired/scripts/run-benchmark.sh
```

The aggregate selected-rootfs run writes the current evidence under:

- `benchmarks/emacs-gym/traces/latest/dired/insula-plan.json`
- one JSON trace per scenario
- `benchmarks/emacs-gym/traces/latest/dired/summary.json`
- `benchmarks/emacs-gym/traces/latest/dired/summary.md`

Current status: 8 scenarios total and 8 scenarios as expected, with Insula
runtime checks passing under `selected-container-rootfs`. The benchmark grades
filesystem state independently from Dired buffer presentation. It accepts
marked-file deletion, rename without collision, and collision rejection without
overwrite; it rejects deletion-flag/ordinary-mark confusion, collision
overwrite, treating Dired undo as filesystem recovery, and trusting a stale
Dired mark after external same-path replacement. The positive stale-mark case
records pre/post filesystem signatures, observes that Dired can preserve a
same-name mark after refresh, clears the stale mark, aborts the destructive
operation, and preserves the replacement file.

Hermeticity for the production gym means the selected rootfs owns the tools,
packages, dependencies, and ABI surface used by the task. It is not sufficient
for Emacs to run in `bwrap` while language servers, Jupyter kernels, Python,
C/C++/Rust toolchains, build tools, or ABI-relevant shared libraries resolve
from the host. Network is a task or materialization policy, not the definition
of hermeticity.

Stock Emacs is the default runtime target for Atlas probes. A patched Emacs
build is a possible observability substrate only after a probe demonstrates that
stock Lisp/advice/module-level instrumentation cannot capture the required
evidence faithfully. If a patch is used, the Emacs source revision, configure
flags, patch set, native-comp/libgccjit closure, and ABI manifest become part of
the selected Insula identity, and the same fixture must be compared against the
stock build.

## How Findings Constrain The Gym

The gym interface is constrained by these rules:

1. Do not collapse buffer, text, file, and window state into one filename-keyed
   string.
2. Do not treat a character modification tick as a universal freshness guard.
3. Do not treat direct function calls, `call-interactively`, command dispatch,
   and PTY input as equivalent.
4. Do not validate Magit behavior only with Git index state; command identity
   and refresh/post-stage hooks are part of operational fidelity.
5. Do not treat timeout as "nothing happened" for LSP, Jupyter, or
   process-backed operations.
6. Do not treat Doom `doom!` declarations as package readiness. Declaration,
   installation, autoload availability, feature load, mode activation, and
   external service readiness are separate states.
7. Do not accept ghost text or delayed edits without validating buffer identity,
   revision, prefix/suffix, and intended replacement range.
8. Do not treat every text read as passive. Property-preserving reads can run
   access fontification hooks.
9. Do not merge completion providers conceptually unless the CAPF owner allows
   fallback through `:exclusive no`.
10. Do not submit Comint input without checking point relative to the process
    mark and the old-input policy.
11. Do not treat Dired undo as recovery for filesystem effects.
12. Do not rely on in-Emacs timeouts as the only guard for CPU-bound or wedged
    package code.
13. Do not treat a development minimal-bind namespace as a production rootfs
    closure. Benchmark evidence needs selected-rootfs, dependency-closure, and
    ABI-manifest evidence.
14. Do not accept Emacs+LSP evidence unless the language server initializes
    from the selected rootfs and the relevant Python/C++/Rust toolchain smokes
    inside that rootfs.
15. Do not accept Emacs+Jupyter evidence unless kernelspec identity and the real
    kernel process prove the same rootfs identity, Python executable/prefix,
    import path, run id, and manifest as the Emacs-side plan.
16. Do not accept Comint evidence unless long-lived process identity, cwd,
    environment, process mark/input boundary, and timeout outcome semantics are
    represented.
17. Do not accept patched-Emacs evidence unless the patch set is source-pinned,
    rootfs-owned, ABI-accounted, and compared against stock Emacs under the same
    fixture.

## Files

- `source-map.md`: inspected versions, source paths, observation classes, and
  RFC 0002 assumptions that are confirmed, revised, or rejected.
- `state-ownership.md`: the ownership graph for buffers, text, windows, files,
  overlays, markers, processes, and kernels.
- `command-dispatch.md`: direct calls, interactive calls, command-loop state,
  terminal input, minibuffers, transients, operation states, and timeouts.
- `doom-lifecycle.md`: Doom bootstrap/profile/module/package activation and
  environment identity.
- `package-contracts/*.md`: package-specific state machines and gym contracts.
- `probes/README.md`: current probe suite, trace files, and probe record
  contract.

## Remaining Gaps

- C-level source mapping still needs exact file:line citations against
  `/opt/soma/build/emacs-30.2/` for `eval.c`, `keyboard.c`, `lread.c`,
  `buffer.h`, `insdel.c`, and `xdisp.c`; the source tree is now present in the
  selected rootfs.
- The current run now includes first selected-rootfs terminal-frame Magit,
  generic minibuffer, Magit package minibuffer, Transient prompt-state, and
  Transient Stage suffix evidence, plus one Magit Diff Transient `--stat` infix
  state transition. PTY/terminal-frame probes still need to cover command-loop
  hooks, terminal byte decoding, broader minibuffer behavior, additional
  Transient infix/lifetime classes, and Doom popup rendering.
- Jupyter batch evidence now covers kernelspec identity, one real Python kernel
  request, existing-kernel association, selected-region execution, wrong-buffer
  rejection, restart/continuity rejection, timeout as `outcome_unknown`, and
  duplicate retry rejection. It also covers protocol-level
  `display_data`/`update_display_data` parent-id, display-id, and payload
  ordering. The live daemon bridge now adds model-facing existing-kernel
  eval-region coverage with associated-source observation, request-status/await
  completion, rootfs-owned runtime evidence, preserved kernel identity,
  exactly-once selected-region execution, and UI-rendered display-update
  presentation through the emacs-jupyter REPL path. Additional display variants
  and recovery beyond rejection remain benchmark work.
- LSP batch evidence now covers real `lsp-mode` source queries for Python,
  C++, and Rust fixtures, with lower-level stdio initialization still recorded
  for Pyright, clangd, and rust-analyzer. The daemon bridge now adds
  model-facing Python LSP rename through both semantic `lsp.rename_symbol` and
  terminal/minibuffer `command.dispatch -> lsp-rename`, request tracking,
  `textDocument/rename`, lsp-mode workspace edit application, multi-file update
  checks, rootfs Python tests, and passive post-command source evidence. It also
  adds daemon diagnostics-after-edit evidence through a separate rootfs-owned
  `pyright-langserver` fixture with full-document LSP sync and explicit
  stale-diagnostic cleanup policy.
  Additional workspace edits/code actions, multi-root projects, and richer
  terminal-frame/async bridge variants remain benchmark work.
- Comint batch evidence now covers point/process-mark input boundary,
  rootfs-owned subprocess identity, long-lived cwd/tool drift, timeout as
  `outcome_unknown`, and a selected-rootfs benchmark grader for live input,
  raw-send rejection, old-history rejection, wrong-buffer rejection, and
  non-idempotent retry rejection. It now also covers process-mark drift after
  asynchronous output through a positive reobserve-before-submit scenario and a
  negative stale-process-mark submission scenario. It also covers partial-output
  completion waiting, premature partial-output success rejection, and
  interrupt reconciliation after partial output. The daemon bridge now adds one
  terminal-frame `comint-send-input` path with process identity, input-marker,
  run-id, and exactly-once side-effect evidence, plus one terminal-frame
  `comint-interrupt-subjob` path with process identity preservation and no
  interrupted side effect. Richer PTY prompt and additional terminal-frame
  process-control variants remain benchmark work.
- Org batch evidence now covers source-edit writeback to the parent Org
  document, subtree refile with ID/property/child preservation, duplicate-ID
  copy rejection, visible-text-only move rejection, and Babel parent-buffer
  result insertion. Capture/refile prompts, agenda views, external Babel
  languages, and terminal-frame/richer async bridge behavior remain benchmark
  work.
- Dired batch evidence now covers marked delete, deletion-flag confusion,
  rename without collision, collision rejection, overwrite rejection, and
  Dired undo not being filesystem recovery. It also covers stale same-path mark
  handles after external replacement by requiring filesystem-signature
  revalidation before destructive operations. Partial filesystem failures,
  interactive prompts, and terminal-frame/richer async bridge behavior remain
  benchmark work.
- The selected rootfs is a CUDA-image-derived source-build path. A Spack-backed
  or otherwise richer materialization path may still be needed for deeper
  dependency DAG and ABI provenance, but it must preserve the container-image
  glibc/dynamic-loader boundary and avoid Nix/host ABI leakage.
- The Atlas needs a stock-vs-patched Emacs observability decision. Candidate
  patches must be justified by failing or insufficient stock probes and then
  rerun under the same fixtures to prove they expose evidence without changing
  the behavior being measured.

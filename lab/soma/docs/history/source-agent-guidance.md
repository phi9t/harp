# Soma Agent Guide

Soma is the workspace for Emacs-specialist model work: an operator model, a
completion model, and the executable Emacs/Doom gym used to train and evaluate
them.

## Insula Means Rootfs-Owned Execution

In this repo, **Insula** means a selected rootfs entered through `bwrap` whose
tool, package, dependency, and ABI closure is accounted for by repo-owned
contracts and evidence.

An Insula is not just a namespace, container, virtualenv, or `bwrap` invocation.
Production benchmark evidence requires the selected rootfs to own Emacs, Doom
package state, LSP servers, Jupyter server/kernels, Python/C++/Rust toolchains,
build/test tools, subprocess environments, cache/runtime paths, and
ABI-relevant shared libraries. Host projections are allowed only when explicitly
declared in the plan.

Network policy is explicit per phase or task. Disabling network can be useful,
but offline execution is not what makes a run hermetic; the rootfs dependency
closure and ABI manifest do. Selected-rootfs runners must fail closed for
host-network sharing: setting `SOMA_INSULA_NETWORK=share-net` also requires a
matching task declaration through `SOMA_INSULA_ALLOW_NETWORK_FOR=<task-id>`.

The `development-minimal-binds` wrapper is development evidence only. It must
remain labeled as `not_rootfs_owned` with `abi_guarantee=none`; use it only via
an explicit development fallback.

The first production-style rootfs path is CUDA-image-derived and source-builds
GNU Emacs inside that image. Its glibc and dynamic loader come from the base
container-image rootfs, not from Nix or the host. Spack remains a candidate
materialization/provenance layer for a larger compiled closure, but it must
respect the same container-rootfs ABI boundary. `soma.insula` owns selection,
validation, bwrap plan emission, execution, and runtime proof; any build system
is evidence-producing infrastructure, not the authority for episode success.

Rootfs payload storage is a separate artifact boundary. Use
`scripts/insula/run-rootfs-artifact-storage.sh` and
[Rootfs Artifact Storage Handoff](docs/insula/rootfs-artifact-storage.md) when
packing selected rootfs artifacts or writing storage request files. Soma owns
artifact identity, manifests, and restore validation. Waystone
(`~/workspace/waystone`) owns effectful storage traffic. It owns HDFS today,
and TOS traffic should be added there rather than in Soma. The current TOS
target is site `example-region`, profile hint
`soma-insula-artifacts`, bucket name `example-artifacts`, and object prefix
`soma/insula-rootfs/`. Soma request files must not contain AK/SK values, tokens,
numeric bucket ids, bytedcli credential material, or executable bytedcli
upload/download commands. Until Waystone has a committed TOS interface, these
request files are handoff evidence, not completed transfer evidence.

## Emacs Source And Patch Discipline

Stock Emacs is the default runtime target. Treat a patched Emacs build as an
explicit Atlas/Insula decision, not as an incidental local tool swap.

Patch Emacs only when executable probes show that Lisp/advice/module-level
instrumentation cannot provide required operability or observability without
changing the behavior being measured, missing important events, or imposing
unacceptable overhead. If a patch is used, the exact Emacs source revision,
configure flags, patch set, build provenance, native-comp/libgccjit closure, and
ABI manifest are part of the selected Insula identity.

Every patched-Emacs claim needs a stock-vs-patched comparison under the same
fixture. The patch should expose state and event evidence; it should not encode
task answers, policy shortcuts, or benchmark-only behavior.

## Work Routing

- Read [RFC 0002](docs/rfcs/0002-doom-emacs-gym-insula.md) before changing the
  Insula, gym, benchmark, or subprocess-launch design.
- Read [the Internals Atlas](docs/internals-atlas/README.md) before freezing or
  extending the Emacs action/observation interface.
- Keep issue-tracked follow-up work under `.scratch/emacs-gym/issues/`.
- Use `probes/emacs-internals/scripts/run-probes.sh` for authoritative
  selected-rootfs Atlas probe evidence. Set `SOMA_USE_DEVELOPMENT_INSULA=1` only
  when deliberately regenerating development-minimal traces.
- Use `scripts/insula/validate-emacs-rootfs-contract.py <rootfs>` before
  accepting or selecting a rootfs. It verifies the CUDA-image-derived ABI
  contract, `/opt/soma/emacs` source build, retained GNU Emacs source tree,
  rootfs-owned Doom, required LSP/Jupyter/Python/C++/Rust tools, dynamic loader,
  linked DSOs, and forbidden host/Nix ABI prefixes.
- Use `scripts/insula/run-rootfs-artifact-storage.sh plan` to inspect the
  Waystone storage request for selected-rootfs artifacts. Use `pack` to create
  the local archive, manifest, and upload request. Delegate upload/download
  traffic to Waystone, then validate any restored rootfs before changing
  `selected.json`.
- Use `.agents/skills/waystone-hdfs-rootfs-artifacts/SKILL.md` when persisting,
  retrieving, validating, or debugging selected-rootfs artifacts through
  Waystone HDFS. Start with
  `scripts/insula/run-rootfs-artifact-storage.sh hdfs-introspect`; if a
  Waystone-side check fails, let it write a ticket draft in
  `~/workspace/waystone/.scratch/soma-hdfs-artifact-tickets/`.
- Use `workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh` for
  selected-rootfs Doom cold/warmed lifecycle evidence before claiming profile,
  module, first-use, teardown, repeated-launch, or source-built Emacs
  readiness.
- Use `benchmarks/emacs-gym/comint/scripts/run-benchmark.sh` for the current
  selected-rootfs Comint process benchmark evidence.
- Use `benchmarks/emacs-gym/magit/scripts/run-benchmark.sh` for the current
  selected-rootfs Magit selective-staging benchmark evidence. It exercises the
  first synchronous checked-bridge command-dispatch slice; terminal-frame and
  async RPC bridge behavior are separate gates.
- Use `benchmarks/emacs-gym/jupyter/scripts/run-benchmark.sh` for the current
  selected-rootfs existing-kernel Jupyter benchmark evidence.
- Use `benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh` for the current
  selected-rootfs Emacs plus Python/C++/Rust LSP source benchmark evidence.
- Use `benchmarks/emacs-gym/org/scripts/run-benchmark.sh` for the current
  selected-rootfs Org structure benchmark evidence. It grades parent Org
  document state for source-edit writeback, subtree refile metadata, and Babel
  result insertion.
- Use `benchmarks/emacs-gym/dired/scripts/run-benchmark.sh` for the current
  selected-rootfs Dired filesystem benchmark evidence. It grades filesystem
  state independently from Dired buffer marks, flags, rename collision handling,
  and undo presentation.
- Use `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` for the
  selected-rootfs live daemon bridge evidence. It starts source-built Emacs as a
  daemon inside the Insula, starts the JSONL bridge socket in that daemon, and
  drives it from an external rootfs-owned Python client. It now includes a first
  daemon-owned in-flight request/status/await check, an explicit
  `waiting_for_input` request/respond lifecycle, `episode.finish` lifecycle
  validation, rootfs-owned `emacsclient --tty` terminal-frame evidence, and
  real tmux-delivered terminal key dispatch. It also covers a real terminal
  minibuffer prompt opened by queued `command.dispatch` and completed by tmux
  input, real terminal-key Magit hunk staging from the daemon-owned terminal
  frame, plus the first queued daemon package operation: checked Magit hunk
  staging through `command.dispatch`, with terminal status observed through
  `request.await`, first Magit Transient prompt-state observation/cancel
  evidence from real terminal input, one Magit Transient Stage suffix execution
  from terminal input, including the observed target-file scope and preserved
  unrelated file state, one Magit Diff Transient `--stat` infix state transition
  from terminal input with diff-buffer verification and unchanged Git state, and
  one package-owned Magit file-selection prompt via
  queued `magit-stage-files`, tmux input, preserved `this-command`, and exact
  Git/hook verification. It also covers stale Magit view recovery through
  model-facing `magit.observe_sections`, repository-fingerprint precondition
  rejection, refreshed section re-selection, and checked `magit-stage`. It also
  includes a live Comint terminal-input scenario
  that submits through `comint-send-input` to a rootfs-owned subprocess and
  checks process identity, input markers, run-id output, and exactly-once side
  effects. It also includes a daemon existing-kernel Jupyter eval-region
  scenario that switches the terminal frame to an associated source buffer,
  submits `jupyter.eval_region`, observes request status/await completion, and
  verifies rootfs-owned Python/kernel identity, `/project/run/jupyter` runtime
  ownership, preserved kernel token, and exactly-once selected-region execution.
  It also includes daemon stale `completion.accept` rejection after real
  terminal-frame human editing, with conflict evidence and preserved final
  buffer text. It also includes daemon Python LSP rename through both the
  semantic `lsp.rename_symbol` operation and terminal/minibuffer
  `command.dispatch -> lsp-rename`, with `textDocument/rename` request
  evidence, lsp-mode workspace edit application, rootfs-owned `pylsp`/Python
  identity, multi-file updates, and rootfs Python tests. It also includes
  daemon LSP diagnostics-after-edit evidence through a separate rootfs-owned
  `pyright-langserver` fixture, full-document LSP sync with debounce disabled,
  guarded live-buffer replacement plus save, repair/save through the bridge,
  explicit stale-diagnostic cleanup policy, and rootfs Python tests. It also includes
  daemon-owned multi-request async status/await evidence with two independent
  in-flight requests awaited out of submission order without result swapping,
  plus a cross-package async scenario where a delayed Jupyter eval request keeps
  its own lifecycle while a queued Magit package operation completes first.
  It also includes a live terminal-frame Comint interrupt scenario that sends
  `C-c C-c`, observes `comint-interrupt-subjob`, preserves shell process
  identity, and proves the interrupted command's side effect did not occur.
  Current selected-rootfs aggregate evidence reports this daemon family as
  28/28 scenarios under `benchmarks/emacs-gym/traces/latest/bridge-daemon`.
  The latest aggregate gym is 90/90 scenarios across 8 families in run
  `20260929T040533Z-3771476`, with required `soma_insula.py` authority evidence
  for every benchmark family. The focused Comint family includes process-mark
  reobservation after asynchronous output, stale process-mark submit rejection,
  partial-output completion waiting, premature partial-output success
  rejection, and interrupt reconciliation. The focused Jupyter family and
  daemon bridge include
  `display_data` and `update_display_data` message capture with parent-request
  ids, shared display id, ordered initial/update payloads, and preserved kernel
  identity.
  Richer package-specific prompts, additional Transient infix classes, and
  broader daemon-owned async semantics for package operations remain separate
  gates.
- The first `soma.insula` extraction is `scripts/insula/soma_insula.py`.
  The generic batch runner and the Magit, Comint, Jupyter, LSP, Org, Dired,
  batch bridge, live daemon bridge, Internals Atlas probe, and Doom lifecycle
  worker wrappers now use it for selected-rootfs resolution, network
  authorization, plan emission, bwrap mount construction, runtime-specific env
  fields, and environment allowlist evidence.
- Do not call a run hermetic when Emacs is inside Insula but Jupyter kernels,
  LSP servers, Comint subprocesses, Python, compilers, or build tools resolve
  from undeclared host state.

# RFC 0002: Doom Emacs Gym on an Insula-Style Rootfs

Status: Draft
Created: 2026-09-27
Quest: Build the executable Emacs training environment described in RFC 0001

## Summary

The next major slice should build an Emacs/Doom Internals Atlas before freezing the Doom Emacs gym interface. After that, the gym should be implemented as a repo-local Insula-style system: a pinned rootfs entered through `bwrap`, a controlled Emacs profile, disposable task fixtures, and an auditable trace/checker layer.

Hermeticity means the rootfs owns the tool, package, dependency, and ABI closure
that benchmark tasks rely on. It does not merely mean "entered a namespace" or
"disabled the network." Emacs, Doom package sources/builds, language servers,
Jupyter server/kernels, Python/C++/Rust toolchains, build/test tools, and
ABI-relevant shared libraries must resolve from the selected Soma rootfs unless
the task explicitly declares a host capability projection.

The recommended architecture is:

```text
host coordinator
  -> soma-local Insula gateway
    -> bwrap task sandbox
      -> rootfs mounted read-only
      -> task fixture mounted read-write
      -> soma harness/profile mounted read-only by default
      -> project-owned HOME/cache/tmp/run dirs
      -> Emacs daemon or batch Emacs with a repo-owned profile
      -> optional Jupyter server/kernel, LSPs, Magit, Dired, comint processes
      -> Unix-socket adapter for structured observations and semantic actions
```

Host code should orchestrate only: select a task, mint a run id, emit the bwrap
plan, start or stop tmux/session processes, and collect artifacts. Substantive
Emacs, Python, C/C++/Rust tooling, Jupyter, LSP, Comint, build/test, and checker
work should run inside the rootfs.

The design should not copy a sibling repo's wrapper wholesale. It should copy the pattern: thin shell gateway, testable Python policy, explicit invocation specs, rootfs contract validation, task-local state, and auditable plans.

## Gym Versus Benchmark

Build the gym as a controllable, resettable instance of real Doom Emacs. Build the benchmark as a separate collection of initial states, user goals, permitted operations, and independently checked outcomes.

The gym provides:

1. A real rootfs-local Emacs process.
2. A terminal frame for Doom/Evil/minibuffer/transient behavior.
3. A small checked Elisp bridge.
4. Disposable repositories, Org files, processes, LSP servers, and kernels.
5. Reset, trace, timeout, and action-validation machinery.

The benchmark provides:

1. Fixture generators and initial-state records.
2. Public user goals and allowed interface profiles.
3. Private graders and invariants.
4. Budgets and scoring rules.
5. Held-out splits for model comparison.

The learner-facing bridge and evaluator must stay separate. A policy should never receive the private grader, expected index contents, hidden fixture generator knobs, or a `check_hidden_tests` action.

For version zero, use one Linux worker, one Emacs instance per episode, a Python supervisor, a small Elisp bridge, and CPU-only repositories/kernels. Distributed RL infrastructure is not part of the first gym build.

The first externally meaningful milestone is:

> A scripted policy can repeatedly complete a selective Magit staging task, while a deliberately incorrect policy fails the grader, even when it produces a convincing success message.

Rootfs contract and Emacs lifecycle work precede that milestone as construction
gates, but they are not the benchmark claim. The current selected-rootfs
contract and batch Doom lifecycle worker now have evidence; terminal-frame
fidelity, request lifecycle semantics, and a first queued daemon Magit package
operation now have selected-rootfs evidence, and the daemon path now includes a
live Comint terminal-input scenario. The Internals Atlas precedes interface
freeze: it validates command context, buffer/window/text ownership, Doom
lifecycle, package state machines, and asynchronous semantics before the bridge
interface is frozen.

## Source Anchors

This RFC is grounded in the current local checkouts:

| Checkout | Relevant pattern |
| --- | --- |
| TorchTitan | Rootfs entrypoint, plan emission, GPU/Emacs binds, and an existing rootfs-hosted Jupyter/Emacs verifier. |
| Monarch | Insula Python module, declarative invocation specs, plan hashing, and contract validation. |
| NCCL | Rootfs store selection, symlink/escape guards, sandbox-owned NVIDIA driver mounts, and Bazel-owned runtime discipline. |
| Lagrange Torch MLU Insula | Pinned image inspection, evidence-producing phases, cache layout ownership, packaging, and clean-rootfs self-verification. |

Important files:

- `/private-source/home/operator/workspace/torchtitan/experiments/jupyter_rootfs/README.org:5` explains the existing rootfs-hosted Jupyter path for Doom `emacs-jupyter`.
- `/private-source/home/operator/workspace/torchtitan/experiments/jupyter_rootfs/run_common.sh:20` and `:34` define rootfs re-entry for server work and Emacs-backed verifiers.
- `/private-source/home/operator/workspace/torchtitan/scripts/rootfs/enter_rootfs.sh:88` documents the opt-in Emacs bind mode, and `:176` defines plan emission.
- `/private-source/home/operator/workspace/torchtitan/scripts/rootfs/enter_rootfs.sh:548` binds the repo-owned Emacs profile and host package sources read-only.
- `/private-source/home/operator/workspace/torchtitan/experiments/jupyter_rootfs/emacs_profile/init.el:73` defines the minimal Doom-compatible verifier profile.
- `/private-source/home/operator/workspace/torchtitan/experiments/jupyter_rootfs/verify_attach.el:46` shows direct `emacs-jupyter` execution and drift assertions.
- `/private-source/home/operator/workspace/monarch/scripts/rootfs/execution_contract.sh:15` states that an environment marker alone does not prove rootfs entry.
- `/private-source/home/operator/workspace/monarch/ginkgo/insula/schema.py:31` defines the invocation schema for binds, environment, command, network, GPU, and isolation flags.
- `/private-source/home/operator/workspace/monarch/ginkgo/insula/bwrap_plan.py:21` builds an auditable `bwrap` plan from a materialized invocation.
- `/private-source/home/operator/workspace/nccl/research/rootfs/tools/rootfs_entry.py:114` validates selected rootfs metadata and rejects unsafe selections.
- `/private-source/home/operator/workspace/nccl/research/rootfs/tools/rootfs_entry.py:244` shows an explicit environment allowlist.
- `/private-source/home/operator/workspace/nccl/research/rootfs/tools/rootfs_entry.py:344` binds both visible and resolved NVIDIA driver libraries under a sandbox-owned path.
- `/private-source/home/operator/workspace/lagrange-torch-mlu-insula/scripts/inspect_insula.sh:42` writes preflight evidence for the pinned base image.
- `/private-source/home/operator/workspace/lagrange-torch-mlu-insula/lagrange_torch_mlu/insula.py:15` defines required tools/packages checked by the Insula preflight.

## Current Local Pattern

### TorchTitan: rootfs-hosted Emacs/Jupyter

TorchTitan already has an experiment that attaches Doom `emacs-jupyter` to kernels running inside its bwrap rootfs. Its README says the server and kernels run under `scripts/rootfs/enter_rootfs.sh`, and batch Emacs verifiers re-enter that same rootfs with a repo-owned Emacs profile.

Transferable rules:

1. Keep host-side scripts limited to orchestration such as token minting, tmux sessions, port checks, and log paths.
2. Re-exec server, kernel, and Emacs verifier logic through the rootfs gateway.
3. Mount the repo-owned Emacs profile into rootfs `HOME`, and mount host package/runtime sources read-only only when explicitly requested.
4. Assert rootfs drift from inside Emacs/Jupyter, not only from shell.
5. Keep generated tokens, logs, venvs, and Jupyter state under git-ignored cache roots.
6. Preserve a stable run id across rootfs re-entry so preflight, server, kernel,
   Emacs verifier, and checker evidence land in one result directory.
7. Use cwd, entrypoint, manifest, and emitted-plan sentinels; environment
   variables alone are weak evidence.

For `soma`, this is the closest starting point, but it is a verifier, not a gym. The gym must add task fixtures, reset semantics, structured observations, semantic actions, and independent final-state checkers.

### Monarch: Insula as a typed execution protocol

Monarch has moved its shell entrypoints behind `python3 -m ginkgo.insula.cli`. The shell scripts are compatibility adapters; the actual policy is in Python modules with schema validation, materialization, plan emission, and command execution.

Transferable rules:

1. Put rootfs policy in testable Python, not a large shell script.
2. Use invocation specs rather than ad hoc bwrap construction at every call site.
3. Clear the environment, then set exactly the values the invocation declares.
4. Emit a plan with mounts, environment, command argv, network/GPU policy, and a bwrap argv digest.
5. Validate rootfs entry with more than a marker variable: fixed mount path, contract digest, uid namespace shape, and controlled tool paths.

For `soma`, the right shape is a repo-local `soma/insula` module inspired by Monarch, extended for Emacs-specific and GPU/Jupyter-specific binds.

### NCCL: fail-closed rootfs selection and sandbox-owned driver binds

NCCL's rootfs entrypoint validates `selected.json`, store ownership, rootfs ownership markers, rootfs manifest hashes, and symlink safety before constructing bwrap arguments. It also binds required NVIDIA driver libraries under `/run/nvidia-driver` rather than relying on mutable system library paths inside the rootfs.

Transferable rules:

1. Treat selected rootfs metadata as an input contract, not a convenience file.
2. Reject symlinks or selected paths that escape the store.
3. Mount driver libraries under a sandbox-owned `/run/host-*` path.
4. Force caches and output bases outside the repo mount.
5. Use an explicit allowlist for inherited environment variables.

For `soma`, this matters because editor workloads are especially prone to host
leakage through `HOME`, `XDG_*`, `PATH`, package caches, native compilation
caches, language-server caches, Jupyter runtime dirs, Doom/straight state,
compiler toolchains, Python environments, Rust cargo state, C/C++ build caches,
and dynamically loaded shared libraries.

### Lagrange: evidence-producing phases

Lagrange treats its Insula image as a pinned input and writes JSON evidence before building on top of it. It centralizes cache paths, phases the workflow, packages outputs with manifests and checksums, and clean-verifies release payloads in a separate verifier rootfs.

Transferable rules:

1. Inspect the base rootfs/image first and write evidence.
2. Keep a typed cache layout module as the single owner of cache, temp, source, evidence, and artifact paths.
3. Make every major phase produce evidence: inspect, materialize, verify, run tasks, package, clean-verify.
4. Separate build/rootfs materialization from clean verification.
5. Refuse dangerous output paths and stale/missing contracts.

For `soma`, this becomes a gym ledger: rootfs id, ABI manifest id, Emacs
version, Doom commit or profile id, package lock id, Python/Jupyter/kernel id,
LSP server/toolchain ids, task fixture id, adapter version, and checker version.

### Production Insula: rootfs-owned runtime closure

The production Insula is not the current development minimal-bind namespace.
It is a selected rootfs with an accounted runtime closure. The host may provide
the Linux kernel, `bwrap`, optional device nodes, optional driver projections,
and artifact collection, but the rootfs owns the tools used by the episode.

The selected rootfs must contain or deliberately project:

1. Emacs and its ABI-relevant runtime libraries, including native-comp support
   such as `libgccjit` if enabled.
2. Doom core, generated profile state, package sources/build artifacts, and
   tree-sitter grammars used by the selected profile.
3. Git, shell utilities, tmux/PTY helpers, and process tools needed by editor
   workflows.
4. Python, package manager state, Jupyter server, `ipykernel`, and any language
   kernels used by benchmark tasks.
5. Python LSP tooling such as `pyright` or an explicitly chosen alternative.
6. C/C++ toolchain and LSP tooling such as `clang`, `gcc`/`g++`, `clangd`,
   headers, libc, libstdc++, and build tools.
7. Rust toolchain and LSP tooling such as `rustc`, `cargo`, and
   `rust-analyzer`.
8. Formatter, linter, test-runner, and build-system tools that tasks are allowed
   to invoke.

The rootfs contract must record enough ABI surface to make a task reproducible:
glibc, dynamic loader, libstdc++, libgcc/libgccjit, compiler and linker
versions, Python shared-library identity, Emacs module/native-comp settings,
tree-sitter ABI, Jupyter/kernel package versions, and LSP server versions. A
successful run without this manifest is evidence of behavior, not an ABI
guarantee.

The current source-built Emacs rootfs is validated by
`scripts/insula/validate-emacs-rootfs-contract.py`. Selected-rootfs runners call
that preflight before entering `bwrap`; it rejects missing source archives,
missing retained Emacs C sources, wrong Emacs prefixes, missing Doom state,
missing required LSP/Jupyter/Python/C++/Rust tools, missing dynamic loader or
linked DSOs, and forbidden host/Nix ABI paths.

Network is a capability, not the definition of hermeticity. Build and
materialization phases may use network if the phase records the fetched inputs
and produces a new rootfs manifest. Scored benchmark episodes should normally
use an already materialized rootfs; if a task needs network, the plan must say
so and the network must not mutate the rootfs closure invisibly.

### Artifact storage boundary: Waystone-owned TOS

Materialized rootfs payloads are stored outside git. Soma uses TOS as the first
rootfs artifact store, but the effectful storage client belongs to Waystone
(`~/workspace/waystone`), which already owns developer-facing HDFS traffic.

The storage path is deliberately outside the runtime hermeticity claim. Soma
packages and names a selected rootfs artifact, writes a local artifact manifest,
and emits a credential-free Waystone storage request. Waystone owns the
effectful upload/download path under its own auth and safety surfaces. It owns
HDFS today; TOS transport should be added there rather than in Soma. Until that
Waystone TOS interface is committed, Soma request files are handoff evidence,
not completed transfer evidence. The selected rootfs contract and post-download
validation remain the authority for benchmark execution.

A rootfs restored from storage must pass
`scripts/insula/validate-emacs-rootfs-contract.py <rootfs> --verify-rootfs-tree`
before `selected.json` can make it active.

Current target policy:

- Storage owner: Waystone
- Provider: `tos`
- Bucket name: `example-artifacts`
- Site: `example-region`
- Profile hint: `soma-insula-artifacts`
- Object prefix: `soma/insula-rootfs/`
- Soma handoff entrypoint: `scripts/insula/run-rootfs-artifact-storage.sh`

Soma request files may record non-secret target defaults, local artifact paths,
object keys, archive hashes, and restore-validation commands. They must not
contain AK/SK values, tokens, numeric bucket ids, bytedcli credential material,
or executable bytedcli upload/download commands. Local vaults and profile setup
for TOS belong to Waystone.

### Materialization strategy: CUDA-image source build first, Spack later if needed

The first production-style rootfs is deliberately CUDA-image-derived. It uses
the PyTorch CUDA 13.2 image lineage already used by local TorchTitan/Monarch
rootfs work, keeps glibc and the dynamic loader from the container-image rootfs,
and builds GNU Emacs from source inside that rootfs. This matches the current
constraint: no Nix, no host glibc, and no host Doom/package/toolchain projection
for production evidence.

For a rootfs with this much compiled and ABI-sensitive state, Spack may still be
the right later materialization layer. Its role would be to build and record a
richer dependency DAG for Emacs, native-comp/libgccjit, tree-sitter, Python,
C/C++/Rust toolchains, language-server dependencies, Jupyter native
dependencies, libc/libstdc++/libgcc surfaces, and build tools. If Spack is used,
it must operate within the selected container-rootfs ABI boundary rather than
pulling glibc, the loader, or core ABI surfaces from Nix or ambient host state.

The split should stay explicit:

1. The materialization layer owns compiler selection, source/build provenance,
   binary caches or build caches, ABI-sensitive package closure, and exported
   views or rootfs staging. The current implementation is the Docker/source-build
   script in `scripts/insula/build-emacs-rootfs.sh`; Spack can replace or
   augment that layer later.
2. `soma.insula` owns selected-rootfs metadata, symlink/escape checks, mount
   layout, environment allowlist, bwrap plan emission, run-id continuity,
   subprocess launch policy, and runtime proof from inside Emacs, LSP servers,
   Jupyter kernels, Comint processes, and checkers.
3. `soma.gym` owns fixtures, observations, actions, traces, and private graders.

The production rootfs build should emit at least:

```text
concrete materialization identity
compiler and target metadata
rootfs manifest and tree hash
ABI manifest for dynamic loader, libc, libstdc++, libgcc/libgccjit, Python, Emacs modules, and tree-sitter
tool inventory for emacs, git, shell, python, jupyter, pyright or chosen Python LSP, clangd, rust-analyzer, compilers, cargo, build/test tools
source and binary-cache provenance
```

Do not let benchmark episodes run `spack install`, `pip install`, `cargo
install`, or package-manager resolution as an invisible side effect. Those
belong to materialization phases that produce a new manifest, then scored
episodes consume that manifest read-only.

Current evidence from the selected source-built materialization:

- Builder: `scripts/insula/build-emacs-rootfs.sh`
- Emacs source-build provenance note:
  `docs/insula/emacs-source-build.md`
- Runner: `probes/emacs-internals/scripts/run-probes.sh`
- Base image: `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`
- Materialization kind: `source-built-emacs-v8` / `docker-cuda-source-build`
- Selected rootfs store id: `soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Recipe SHA:
  `0bddf41e6539849c1d8e0a44954ebb3293f6eaa08081741c4adbf3c777fe808c`
- Docker image:
  `soma-emacs-rootfs:0bddf41e6539849c`
- Image id:
  `sha256:9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d349d7bd6b6bb80`
- Rootfs tree manifest: `/etc/soma-insula-rootfs-tree-manifest.jsonl`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Rootfs tree entries: `152105`
- Emacs: source-built GNU Emacs `30.2` at `/opt/soma/emacs/bin/emacs`
- Emacs executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Emacsclient executable SHA256:
  `dbce94539b0ee6db6cb50078162896ea35ab8ef80e2c7d5c02dbf58f16aa315a`
- Emacs source archive SHA256:
  `b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9`,
  checked before extraction during materialization and rechecked by the
  validator.
- Emacs configure flags:
  `--prefix=/opt/soma/emacs --with-native-compilation --with-tree-sitter --with-sqlite3 --without-x --without-sound --without-dbus --without-gconf --without-gsettings --without-toolkit-scroll-bars --without-xwidgets --without-mailutils`
- Emacs capability checks: `json-parse-string`, native compilation, and
  tree-sitter are recorded as runtime facts in the rootfs contract.
- Required native tool ABI checks: the rootfs contract records ELF status,
  dynamic loader, and `ldd` output for native tools in the production tool
  inventory.
- Doom: rootfs-owned checkout and sync at commit `8b475fcb2574286d7f4413c22cc7902d80d21ced`
- Rust: rootfs-owned `1.81.0` toolchain with `rust-src` installed for
  `rust-analyzer` sysroot discovery
- Rootfs contract: `/etc/soma-insula-rootfs-contract.json`
- Probe result: 36 batch-safe probes, 36 passing, under
  `selected-container-rootfs`
- Doom lifecycle worker:
  `workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh`
- Doom lifecycle result: cold, warmed, cold-repeat-1, and warmed-repeat-1
  source-built Emacs children pass under the selected rootfs, loading
  rootfs-owned Doom from `/project/home/.emacs.d` and `/project/home/.doom.d`;
  teardown probes show no preexisting token process and no token-marked process
  remaining after each child exit
- Comint process slice: `benchmarks/emacs-gym/comint/scripts/run-benchmark.sh`
  reports 11 scenarios total and 11 as expected under the same selected rootfs,
  including process-mark drift, partial-output waiting, premature-success
  rejection, timeout/retry rejection, and interrupt reconciliation
- First Magit vertical slice:
  `benchmarks/emacs-gym/magit/scripts/run-benchmark.sh` reports 7 scenarios
  total and 7 as expected under the same selected rootfs, with correct policies
  routed through the first synchronous checked-bridge command-dispatch slice
- Existing-kernel Jupyter slice:
  `benchmarks/emacs-gym/jupyter/scripts/run-benchmark.sh` reports 8 scenarios
  total and 8 as expected under the same selected rootfs, including existing
  kernel association, selected-region execution, wrong-buffer rejection,
  restart/continuity rejection, timeout/retry semantics, and
  `display_data`/`update_display_data` parent-id, display-id, and payload
  ordering evidence
- Rootfs-owned LSP source slice:
  `benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh` reports 6 scenarios total
  and 6 as expected under the same selected rootfs. It exercises real Emacs
  `lsp-mode` clients against Python, C++, and Rust fixtures, rejects direct text
  replacement as an LSP refactor, and rejects stale document/workspace roots
- Org structure slice:
  `benchmarks/emacs-gym/org/scripts/run-benchmark.sh` reports 6 scenarios total
  and 6 as expected under the same selected rootfs. It grades the parent Org
  document for source-edit writeback, verifies `org-refile` subtree moves
  preserve IDs/properties/children, rejects duplicate-ID copy behavior and
  visible-text-only moves that lose metadata, and verifies Babel result
  insertion in the parent buffer.
- Dired filesystem slice:
  `benchmarks/emacs-gym/dired/scripts/run-benchmark.sh` reports 8 scenarios
  total and 8 as expected under the same selected rootfs. It grades filesystem
  state independently from Dired buffer presentation, covering marked delete,
  deletion-flag confusion, rename without collision, collision rejection,
  overwrite rejection, Dired undo not being filesystem recovery, and stale
  same-path replacement revalidation by filesystem signature.
- Bridge transport slice:
  `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh` reports 16 scenarios
  total and 16 as expected under the same selected rootfs. It exercises
  request-envelope submission, terminal request status storage, duplicate
  request-id rejection, direct-call rejection, checked command dispatch,
  checked `buffer.read`, checked `buffer.replace_range`, checked `buffer.save`,
  stale-edit conflict rejection, selected-window command drift rejection, point
  edit drift rejection, narrowed/mark-active/read-only edit-state guards,
  checked completion acceptance, stale completion rejection, `episode.finish`
  final evidence validation, and a JSONL Unix-domain socket round trip
- Live daemon bridge slice:
  `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh` reports 28
  scenarios total and 28 as expected under the same selected rootfs in
  `benchmarks/emacs-gym/traces/latest/bridge-daemon/summary.json`.
  It starts
  source-built Emacs as a named daemon, starts the JSONL bridge socket inside
  that daemon at `/project/run/soma-gym-bridge-daemon.sock`, and drives
  `emacs.observe_buffer`, `request.status`, command-dispatch rejection, and
  direct-call rejection from an external rootfs-owned Python client process. It
  also attaches rootfs-owned `emacsclient --tty` through rootfs-owned `tmux`,
  observes the resulting daemon-owned terminal client frame through
  `emacs.observe_frames`, sends real terminal keys through tmux to switch that
  client frame to the daemon target buffer, submits a timer-backed daemon-owned
  request that initially returns `running`, observes that in-flight state
  through `request.status`, then waits for terminal success through
  `request.await` without resubmission, and verifies that a short await timeout
  returns `outcome_unknown` before a later await reconciles the same original
  request; verifies an explicit `waiting_for_input` request/respond lifecycle;
  verifies checked point navigation in the selected terminal frame; verifies a
  real terminal minibuffer prompt opened by queued daemon `command.dispatch` and
  completed by tmux input; verifies queued Magit hunk staging through
  `command.dispatch`, with terminal request success observed through
  `request.await`, preserved `this-command`, refresh and post-stage hook
  evidence, exact index/worktree checks, and unrelated staged/unstaged state
  preservation; verifies real terminal-key Magit hunk staging from the
  daemon-owned terminal frame with post-stage hook and index/worktree evidence;
  verifies Magit Transient prompt-state observation and cancellation from real
  terminal input without changing fixture Git state; verifies Magit Transient
  Stage suffix execution from terminal input with observed target-file scope and
  preserved unrelated file state; verifies a Magit Diff Transient `--stat`
  infix state transition from terminal input, then applies the toggled state to
  a diff buffer without mutating Git state; verifies a package-owned Magit
  `magit-stage-files` prompt opened by queued daemon `command.dispatch`, completed
  by tmux input, and checked through preserved `this-command`, post-stage hook
  identity, exact index/worktree contents, and unrelated state preservation;
  verifies stale Magit view recovery through the model-facing daemon path by
  reading refreshed `magit.observe_sections` evidence, rejecting old
  repository-fingerprint preconditions before command mutation, then reselecting
  and staging only the refreshed intended hunk while preserving the external hunk;
  and verifies that a premature `episode.finish` reports `incomplete_episode`,
  then succeeds after the required request IDs reach terminal success. It also
  verifies a live Comint terminal-input path: a rootfs-owned terminal frame
  submits shell input through `comint-send-input`, preserves the Comint process
  identity, updates `comint-last-input-*` markers, carries the Insula run id
  into subprocess output, and performs the intended side effect exactly once.
  It also verifies daemon stale completion acceptance rejection: terminal-frame
  human input invalidates an older proposal, `completion.accept` fails with
  `conflict`, and the final buffer text preserves the human edit. It also
  verifies daemon Python LSP rename through both semantic `lsp.rename_symbol`
  and terminal/minibuffer `command.dispatch -> lsp-rename` paths, with
  `textDocument/rename`, lsp-mode workspace edit application, rootfs-owned
  `pylsp`/Python identity, multi-file update checks, rootfs Python tests,
  prompt observation, original-request terminal status polling, and passive
  post-command source observation; verifies daemon diagnostics-after-edit through
  a separate rootfs-owned `pyright-langserver` fixture, with recorded
  rootfs-owned pyright/node/python paths, full-document LSP sync with debounce
  disabled, guarded live-buffer replacement plus save, diagnostics containing
  `Expected expression` after the broken save, bridge-mediated repair/save,
  explicit stale-diagnostic cleanup policy, zero diagnostics after repair, and
  rootfs Python tests; and verifies
  `daemon-multi-async-status-await`, where two daemon-owned async requests keep
  independent status/result identity and can be awaited out of submission order,
  plus `daemon-package-async-jupyter-magit`, where a delayed existing-kernel
  Jupyter eval request keeps independent status while a queued Magit package
  command completes first, and `daemon-comint-terminal-interrupt-reconcile`,
  where a real terminal-frame `C-c C-c` invokes `comint-interrupt-subjob`,
  keeps the rootfs-owned shell process alive, and prevents the interrupted
  command's side effect.
- Aggregate selected-rootfs benchmark:
  `scripts/insula/run-emacs-gym-benchmarks-rootfs.sh` reports 8 benchmark
  families, 90 scenarios total, and 90 scenarios as expected under
  `benchmarks/emacs-gym/traces/latest/summary.json` in run
  `20260929T040533Z-3771476`. The focused Comint family now covers
  process-mark reobservation after asynchronous output and stale process-mark
  submission rejection, partial-output completion waiting, premature
  partial-output success rejection, and interrupt reconciliation. The focused
  Jupyter family and daemon bridge now
  include `display_data` and `update_display_data` capture with parent-request
  ids, shared display id, ordered initial/update payloads, and preserved
  existing kernel identity. The aggregate runner
  rejects child evidence whose `insula-plan.json` does not match the currently
  selected rootfs store id, rootfs path, contract path, recipe SHA, base image,
  image tag, image id, and rootfs tree-manifest identity. It also rejects
  benchmark families whose child plans do not record
  `soma_insula_authority=scripts/insula/soma_insula.py`.
- First `soma.insula` authority extraction:
  `scripts/insula/soma_insula.py` now owns selected-rootfs resolution,
  symlink/escape checks, network authorization, generic bwrap mount
  construction, environment allowlist construction, and plan emission for the
  generic command runner. The Magit, Comint, Jupyter, LSP, Org, Dired, batch
  bridge, live daemon bridge, Internals Atlas probe, and Doom lifecycle worker
  wrappers now route through this module, including daemon/tmux/socket lifecycle
  plan fields for the live daemon bridge.

The selected-rootfs validator now rejects non-digest or non-CUDA base images,
Nix/host ABI prefixes, missing rootfs-owned tools, missing source archive/tree,
unrecognized Emacs configure options, missing rootfs tree-manifest evidence, and
unrecorded source-built Emacs capabilities before a runner enters benchmark work.

Those probes, the lifecycle worker, and the selected-rootfs benchmark slices
prove rootfs-owned executable provenance, source-built Emacs identity,
cold/warmed plus repeated Doom startup, selected-profile module state,
first-use feature loads, per-child teardown probes, Python/Jupyter kernelspec
and real kernel execution,
Pyright/clangd/rust-analyzer stdio initialization, C/Rust smoke builds, Comint
long-lived shell/timeout semantics, batch Magit selective-staging semantics
over real Magit status sections through a synchronous checked bridge
command-dispatch route, existing-kernel `emacs-jupyter` association/execution
semantics, real Emacs+LSP source behavior for Python, C++, and Rust fixtures,
and a batch-safe model-facing bridge request envelope with JSONL Unix-socket
transport smoke, plus a live daemon/external-client JSONL bridge path inside
the selected rootfs. The batch bridge now also proves read purity, checked
range edit/save, and stale edit conflict rejection. The daemon bridge now covers
one rootfs-owned `emacsclient --tty` terminal client frame and one in-flight
async request. The daemon benchmark now also sends real terminal keys through
tmux to switch the daemon-owned `emacsclient --tty` frame to the benchmark
target buffer, with the result verified through `emacs.observe_frames`, proves
`request.await` timeout reports `outcome_unknown` before later reconciliation of
the original request, and proves one queued real package operation by staging a
selected Magit hunk through daemon `command.dispatch` and awaiting the original
request's terminal status. It also proves one real terminal-key Magit hunk
staging operation from the daemon-owned terminal frame, one Magit Transient
prompt-state observation/cancel flow, one Magit Transient Stage suffix flow with
observed target-file scope, one Magit Diff Transient `--stat` infix state
transition with diff-buffer evidence and unchanged Git state, and one
package-owned Magit `magit-stage-files` prompt completed by tmux input with
preserved command identity and hook evidence. The daemon bridge now also proves
one model-facing existing-kernel Jupyter eval-region operation from an
associated source buffer, with `jupyter.eval_region` submitted through the
daemon request lifecycle, terminal status observed through
`request.status`/`request.await`, kernel token preserved, rootfs-owned
Python/Jupyter runtime paths recorded under `/project/run/jupyter`, and
exactly-once selected-region execution. It also proves UI-rendered Jupyter
display-update presentation through `jupyter.eval_region_via_repl`,
`jupyter-repl-execute-cell`, REPL-owned display-id range observation, checked
terminal navigation to that range, and tmux capture of
`SOMA_DISPLAY_STAGE:updated`. These slices
also prove daemon stale `completion.accept` rejection after terminal-frame
human editing, with conflict evidence and preserved final buffer text. The
daemon bridge now also proves model-facing Python LSP rename paths through both
semantic `lsp.rename_symbol` and terminal/minibuffer
`command.dispatch -> lsp-rename`, with `textDocument/rename` request evidence,
lsp-mode workspace edit application, rootfs-owned `pylsp`/Python identity,
definition, caller, and test-file updates, and rootfs Python tests after the
	rename. It also proves a first cross-package package-async queue case:
	`daemon-package-async-jupyter-magit` submits a delayed existing-kernel Jupyter
	eval request under durable buffer/kernel/text preconditions, switches to a
	fresh Magit status buffer, completes checked `magit-stage`, then awaits the
	original Jupyter request without result swapping or duplicate kernel execution.
	They do not yet prove Doom/Evil-specific bindings, richer package-specific
	prompts, additional Transient infix classes, full FIM model serving/ranking,
	richer interactive LSP rename variants,
multi-root LSP stress cases, additional Jupyter display/recovery variants, or
richer interactive Dired/Org workflow variants beyond the first batch-safe
slices.

The selected rootfs Doom profile is intentionally minimal:
`:emacs dired electric undo vc`, `:tools eval lookup lsp magit`,
`:lang emacs-lisp (org +jupyter) python rust cc sh`, and
`:config (default +bindings)`. It does not enable Evil, Corfu, Vertico, Popup,
or Doom's TTY module; do not infer those capabilities from host Doom state.

The Comint benchmark is the first executed benchmark-grader loop. It proves that
the selected rootfs can run a benchmark fixture, emit a plan, execute real
Comint subprocess scenarios, accept the correct scripted policy, and reject
targeted wrong policies. The Magit benchmark is the first package-specific
vertical slice: it creates disposable Git repositories, stages real Magit
sections through the first checked bridge route, verifies exact index/worktree
contents, captures refresh and post-stage hook evidence, and rejects raw Git,
wrapper-context, list-scope, and stale-view policies. The Org and Dired
benchmarks add first batch-safe structural/filesystem graders for parent Org
document state, refile metadata, Babel results, Dired mark/flag distinction,
rename collision handling, and Dired undo boundaries. These batch benchmarks do
not replace terminal-frame or full daemon async bridge work. The bridge
transport smoke adds request IDs, status storage, duplicate rejection,
direct-call rejection, and a JSONL Unix-socket round trip inside the selected
rootfs. The batch bridge also proves passive buffer read purity, checked range
edit/save behavior, and stale edit conflict rejection. The daemon bridge smoke
proves a separate rootfs-owned client process can talk to a live Emacs daemon
over the JSONL bridge socket and can observe one daemon-owned in-flight
operation through status and await, drive checked point navigation, and complete
one queued real package operation by staging a Magit hunk through
`command.dispatch`; terminal minibuffer prompt handling and real terminal-key
Magit hunk staging now have first selected-rootfs evidence, as does Magit
Transient prompt-state observation/cancel, Stage suffix execution, and a
Magit Diff Transient `--stat` infix state transition, plus a package-owned
    Magit `magit-stage-files` prompt; the daemon bridge also now has both a
    multi-request async status/await proof, a first cross-package real package
    async proof for delayed Jupyter eval plus queued Magit staging, and a live
    terminal-frame Comint interrupt reconciliation proof. Richer
    package-specific prompts, additional Transient infix classes, and broader
    package async variants remain open.

## Proposed Soma Architecture

### Pre-Implementation Gate: Internals Atlas

Do not freeze the gym action/observation interface until `.scratch/emacs-gym/issues/0001-emacs-doom-internals-atlas.md` is complete.

The Atlas must provide:

1. A versioned source map for Emacs, Doom, and target packages.
2. A state-ownership model for buffers, text, windows, files, processes, kernels, and package views.
3. A command-dispatch contract that distinguishes direct function calls, `call-interactively`, command-loop dispatch, and real terminal input.
4. A Doom bootstrap-to-first-use lifecycle trace.
5. Package-specific lifecycle contracts for Magit, Org, LSP, Jupyter, Comint, Dired, and completion.
6. Executable mechanism probes with traces, independent checks, counterexamples, and derived gym requirements.

The gym architecture below is therefore a hypothesis to validate. The Atlas may revise action preconditions, observation purity rules, dispatch semantics, package adapters, or benchmark invariants.

### Repo layout

Initial layout:

```text
CONTEXT.md
docs/rfcs/
  0001-emacs-specialist-models.md
  0002-doom-emacs-gym-insula.md
scripts/
  run
  rootfs/
    enter_rootfs.sh
    build_rootfs.sh
soma/
  insula/
    cli.py
    schema.py
    materialize.py
    bwrap_plan.py
    contracts.py
    local_environment.py
  gym/
    cli.py
    tasks.py
    runner.py
    trace.py
    checkers.py
  emacs/
    profile/
      early-init.el
      init.el
    adapter/
      soma-gym.el
tasks/
  buffer-edit-smoke/
    task.yaml
    fixture/
    checker.py
  magit-stage-selected-change/
    task.yaml
    fixture/
    checker.py
  jupyter-attach-smoke/
    task.yaml
    fixture/
    checker.py
benchmarks/
  development.yaml
  sealed.yaml
runs/
  README.md
```

The shell gateway should be thin:

```text
scripts/run -> python3 -m soma.gym.cli ...
scripts/rootfs/enter_rootfs.sh -> python3 -m soma.insula.cli enter-rootfs-compat ...
```

The `soma.insula` module should own rootfs selection, logical path refs, bwrap plan construction, environment clearing, and contract checks. The `soma.gym` module should own task fixtures, Emacs process lifecycle, action/observation traces, and independent checkers.

The benchmark specs should not be mounted wholesale into episodes. Only public fixture files and public goals should enter the episode filesystem. Private evaluation files stay outside the sandbox or are mounted only into a separate checker invocation unavailable to the policy.

### Invocation model

Each gym run should materialize from three declarative inputs:

1. Local environment: host paths for repo, rootfs store, cache, temp, run,
   results, optional GPU devices, and optional host capability projections.
2. Insula invocation: rootfs ref, rootfs manifest/digest, ABI manifest,
   declared toolchain profile, mounts, environment, cwd, argv, network mode,
   GPU mode, and artifact roots.
3. Gym task: fixture source, writable workspace policy, required capabilities, initial editor state, allowed operations, checker command, and expected invariants.

Example task shape:

```yaml
schema_version: 1
name: buffer-edit-smoke
capabilities:
  editor_edits: true
  shell_processes: false
  jupyter_execution: false
  git_write: false
fixture:
  source: tasks/buffer-edit-smoke/fixture
  mount: /workspace/task
emacs:
  profile: soma/emacs/profile
  mode: daemon
  adapter: soma/emacs/adapter/soma-gym.el
initial_state:
  open_files:
    - /workspace/task/src/example.py
checks:
  command:
    - /workspace/soma/.venv/bin/python
    - tasks/buffer-edit-smoke/checker.py
artifacts:
  trace: results://trace.jsonl
  final_observation: results://final-observation.json
```

### Sandbox layout

Use stable in-sandbox paths:

```text
/workspace/soma      repo harness, read-only by default for benchmark runs
/workspace/task      disposable task fixture, read-write
/project/home        rootfs-owned HOME
/project/run         sockets, pid files, Jupyter connection files
/project/tmp         TMPDIR
/project/cache       XDG, package, LSP, native-comp, Jupyter caches
/project/logs        Emacs, adapter, process, and checker logs
/project/results     run artifacts
/run/nvidia-driver   optional host driver projection
/run/host-*          optional host capability projections
```

Do not let benchmark runs write into host `$HOME`, host `.emacs.d`, host Jupyter runtime dirs, host LSP caches, or the source checkout except for explicitly declared task workspaces.

### Emacs process model

Start with two modes:

1. Headless batch mode for deterministic smoke checks.
2. Rootfs-local daemon mode for interactive gym tasks.

Batch mode proves the rootfs/profile/package path can load and execute checks. Daemon mode is the operator target: a normal Emacs event loop with buffers, windows, async processes, LSP, Magit, Org, and Jupyter clients.

The adapter should expose a Unix-domain socket inside `/project/run`. The host coordinator can bind the run directory and talk to the socket without exposing a network port. If a task needs Jupyter, start the Jupyter server in the same sandbox process tree so Emacs can use loopback inside the namespace. Use host `share-net` only for explicit human attach or package bootstrap modes.

### Emacs profile

The selected first profile is Doom-compatible but smaller than the user's live
Doom setup:

```elisp
(doom!
 :emacs dired electric undo vc
 :tools eval lookup lsp magit
 :lang emacs-lisp (org +jupyter) python rust cc sh
 :config (default +bindings))
```

The lifecycle worker proves this profile under the selected source-built Emacs
rootfs. It loads Doom through the generated profile, keeps Doom code
rootfs-owned and read-only, and provides writable per-run Doom cache/state
directories for startup side effects. Corfu, Vertico, Popup, Evil, and Doom's
TTY module are not part of this selected profile.

Later, add a "Doom fidelity" profile pinned to a Doom commit, package lock, and selected modules. Do not make full host Doom the default benchmark profile.

### Rootfs contract

`SOMA_IN_ROOTFS=1` should be a hint, not proof.

The contract checker should verify:

1. Cwd is the canonical sandbox mount, such as `/workspace/soma`.
2. The rootfs contract file exists inside `/etc`.
3. The selected rootfs digest, recipe hash, and ABI manifest match the local
   environment.
4. The uid map is controlled.
5. Emacs, Python, Jupyter, LSP servers, Git, shell, Python/C++/Rust compilers,
   linkers, package managers, formatters, linters, and test runners resolve to
   controlled rootfs paths.
6. `HOME`, `TMPDIR`, `XDG_CACHE_HOME`, `JUPYTER_RUNTIME_DIR`, `EMACSLOADPATH`, and native-comp cache paths are sandbox-owned.
7. Host leakage variables such as `PYTHONPATH`, `LD_LIBRARY_PATH`, `CONDA_PREFIX`, `MAMBA_ROOT_PREFIX`, and host Doom paths are absent unless explicitly projected.
8. Real Emacs+LSP+source fixtures can attach to rootfs-owned language servers
   for Python, C/C++, and Rust and return diagnostics or symbol information.
9. Real Emacs+Jupyter+kernel fixtures can attach to a rootfs-owned kernel and
   report kernel-side cwd, Python executable/prefix, import path, run id, and
   selected manifest.
10. Comint subprocesses used by tasks inherit the same rootfs identity and tool
    resolution as Emacs.

### Plan and trace artifacts

Every run should write:

```text
run.json
insula-plan.yaml
rootfs-contract.json
initial-observation.json
trace.jsonl
final-observation.json
checker-result.json
stdout.log
stderr.log
```

`insula-plan.yaml` should include rootfs identity, mount list, env, cwd, argv, network policy, GPU policy, Emacs profile identity, task fixture identity, and bwrap argv digest.

It should also include dependency-closure classification, ABI manifest id,
toolchain profile, required tool inventory, allowed host projections, and the
stable run id used across rootfs re-entry.

`trace.jsonl` should include:

1. User goal and constraints.
2. Observations.
3. Actions requested by the operator.
4. Adapter responses.
5. Buffer identity and `buffer-chars-modified-tick` when relevant.
6. Process/Jupyter request IDs and completion state.
7. Checker result.

This is the shared layer for operator training and for failure correction.

## Observation And Action Protocol

The observation/action interface is the model's training surface. Version it before generating demonstrations.

### Observations

Observations should be compact and identity-based:

```json
{
  "episode_id": "ep-0081",
  "observation_seq": 37,
  "editor_epoch": 1,
  "selected_frame": "frame-1",
  "selected_window": "window-2",
  "selected_buffer": "buffer-7",
  "interaction": {
    "evil_state": "normal",
    "minibuffer": null,
    "transient": null
  },
  "buffers": [
    {
      "id": "buffer-7",
      "name": "train.py",
      "path": "/workspace/task/train.py",
      "major_mode": "python-mode",
      "chars_tick": 104,
      "modified": true,
      "point": 457
    }
  ],
  "pending_operations": [],
  "recent_events": []
}
```

The initial observation should not dump every buffer's full contents. Use explicit reads for buffer ranges, Magit sections, Org subtrees, diagnostics, and process output. Buffer names are display information, not identity; use opaque handles for buffers, windows, processes, and kernels.

Observation functions must be pure with respect to meaningful editor state. Repeated observation must not move point, change selected windows, save buffers, clear marks, or otherwise help the policy.

Use one position convention and enforce it. Recommendation: buffer positions are one-based Emacs character positions and ranges are half-open; LSP positions are converted by the adapter.

### Actions

Start with this action surface:

| Action group | Examples |
| --- | --- |
| Inspection | `emacs.observe`, `buffer.read`, `command.describe`, `magit.read_sections` |
| Navigation | `buffer.select`, `window.select`, `point.set`, `region.set` |
| Editing | `buffer.replace_range`, `buffer.save`, `edit.undo` |
| Native operations | `command.invoke`, with an audited command catalog |
| Interaction | `minibuffer.respond`, `interaction.cancel`, `operation.await` |
| Lifecycle | `episode.finish` |

Package adapters should expose semantic handles, not task answers. For example, `magit.select_section` may select an observed section, but there should be no `magit.stage_the_bugfix` action.

Mutation requests must include preconditions:

```json
{
  "request_id": "req-19",
  "operation": "buffer.replace_range",
  "target": {
    "editor_epoch": 1,
    "buffer_id": "buffer-7"
  },
  "preconditions": {
    "chars_tick": 104
  },
  "arguments": {
    "start": 457,
    "end": 482,
    "text": "replacement"
  }
}
```

A stale request returns `conflict` and changes nothing.

Maintain two action profiles:

1. **Native-operation profile**: structured observations, checked navigation/editing, and actual package commands. This is the intended deployed operator surface.
2. **Keymap profile**: terminal input through the PTY on a smaller representative suite. This tests Doom/Evil bindings, prompts, and transient navigation.

Report the profiles separately.

### Asynchronous State

Treat asynchronous execution as a first-class state machine:

```text
accepted -> running -> succeeded
                    -> failed
                    -> waiting_for_input
                    -> outcome_unknown
```

Rejected requests are distinct from accepted requests that later fail or become unknown.

The Elisp RPC transport should validate and enqueue requests rather than directly running arbitrary long commands in the transport handler. Prompt responses and cancellation must remain available while a command is waiting for interaction.

Wait for specific completion conditions:

| Operation | Completion condition |
| --- | --- |
| Buffer edit | Edit committed and new buffer revision observed |
| LSP request | Matching response for the intended document state |
| Magit staging | Relevant subprocess completion, index inspection, status refresh |
| Jupyter execution | Matching request reply and matching execution-status events |
| Comint command | Task-specific output or exit condition |

A retry with the same request ID should not resubmit an operation. If submission may have occurred but completion cannot be established, return `outcome_unknown` and require reconciliation.

## Task Contracts

A benchmark task is a state-transition contract:

```text
Task = (
  initialization,
  public goal,
  allowed interface,
  success predicates,
  invariants,
  budgets
)
```

Example:

```yaml
id: magit.stage-selected-change.v1

public:
  goal: >
    In the current repository, stage only the divide-by-zero fix.
    Leave the debug-print change unstaged.
    Preserve existing staged changes and the unsaved notes buffer.
    Use Magit, and finish with its status buffer visible.

interface_profile: native-v1

fixture:
  generator: selective_staging
  parameters:
    unrelated_staged_change: true
    debug_change_same_file: true
    unsaved_notes_buffer: true
    randomized_initial_window: true

budgets:
  max_actions: 40
  wall_seconds: 180

private_evaluation:
  grader: selective_staging_v1
  checks:
    - expected_index_entries
    - unchanged_worktree
    - unchanged_unsaved_notes
    - unchanged_head
    - magit_operation_provenance
    - final_status_buffer_visible
```

Only the public goal, interface profile, and public environment state reach the policy.

## First Vertical Benchmark

The first package-specific vertical benchmark avoids GPU and Jupyter. It uses
selective Magit staging to prove the rootfs, lifecycle, fixture, trace, grader,
and a first checked bridge command-dispatch route, then extends the same
scenario through terminal-frame and async RPC paths.

Before that full vertical, the Comint process benchmark exercised the
rootfs-plus-fixture-plus-grader loop with a deterministic subprocess task:

```text
benchmarks/emacs-gym/comint/scripts/run-benchmark.sh
```

It is intentionally narrower than the Magit task. It has no model-facing bridge
yet, but it already rejects raw process I/O, wrong-buffer output ownership,
old-history submission, and duplicate non-idempotent execution after timeout.

The first checked-bridge-backed, batch-safe Magit vertical now runs here:

```text
benchmarks/emacs-gym/magit/scripts/run-benchmark.sh
```

It reports 7 scenarios total and 7 scenarios as expected under the selected
source-built Emacs rootfs. Correct policies stage selected Magit hunk, file, and
active-region file sections through `command.dispatch` in
`benchmarks/emacs-gym/elisp/soma-gym-bridge.el`. Bad policies fail for raw
`git add -u`, direct bridge-wrapper staging through `magit-stage-1`, selecting
the whole unstaged list, and acting on a stale Magit view after an external
edit.

Task: `magit.stage-selected-change.v1`

Goal:

```text
In the current repository, stage only the divide-by-zero fix.
Leave the debug-print change unstaged.
Preserve existing staged changes and the unsaved notes buffer.
Use Magit, and finish with its status buffer visible.
```

Fixture:

1. Build or select a rootfs with the required Emacs/Doom/Git/tool closure.
2. Emit and validate the Insula plan.
3. Create a disposable Git repository with an initial commit.
4. Add an unrelated staged change.
5. Add two unstaged changes in another file: the intended fix and a debug print.
6. Create an unsaved notes buffer.
7. Start from a non-Magit buffer with randomized windows.

For the first version, place the desired fix and debug print in separate hunks. Later add same-hunk and partial-hunk variants.

Grader:

1. Inspect the actual Git index with `git ls-files --stage -z`.
2. Parse mode, object ID, stage number, and path into a canonical map.
3. Compare the index to the initial index plus exactly the intended fix.
4. Compare the working-tree manifest, executable bits, symlink targets, and relevant untracked files.
5. Compare the notes buffer text and modified status independently of the filesystem.
6. Confirm HEAD is unchanged.
7. Confirm a real Magit staging operation occurred when the public goal required Magit.
8. Confirm the final Magit status view is visible.

Pass condition:

```text
index = initial index + intended fix only
AND working tree unchanged outside intended edits
AND existing staged changes preserved
AND unsaved notes preserved
AND HEAD unchanged
AND actual Magit staging operation observed
AND requested final view present
```

Targeted bad outcomes must fail: staging everything, staging nothing, staging the wrong hunk, changing the worktree, losing the notes buffer, committing/resetting, and emitting a fake success message.

This gives the first tight feedback loop for the Insula contract, Emacs lifecycle, adapter socket, Magit adapter, trace format, private grader, and false-completion resistance.

Next lightweight task: `buffer-edit-smoke`

Use it to test read purity, checked edits, stale-write rejection, save behavior, and final checker execution without package-specific complexity.

Second substantial task family: `jupyter-attach-smoke`

Copy the TorchTitan verifier shape, but make it a gym task:

1. Start a rootfs-local Jupyter server or kernel in the same sandbox.
2. Attach Emacs to the existing kernel.
3. Evaluate a selected region once.
4. Verify kernel identity, request/parent headers, rootfs Python prefix, and no restart.

The batch-safe implementation is now green under the selected source-built
Emacs rootfs. It includes protocol-level display-data/update evidence, and
should be treated as the baseline for richer terminal-frame presentation and
recovery testing.

Substantial LSP tasks have the same standard: a real Emacs client attaches to a
real rootfs-owned language server against rootfs-mounted source code, then the
checker verifies server identity, workspace root, document synchronization, and
the resulting diagnostics/refactor evidence. The first selected-rootfs LSP
source benchmark now satisfies this for Python, C++, and Rust symbol/source
queries plus stale-handle rejection. The live daemon bridge now adds one
model-facing Python rename through real `lsp-rename`, request tracking,
`textDocument/rename`, lsp-mode workspace edit application, and rootfs Python
tests after the rename, plus diagnostics-after-edit evidence through a dedicated
rootfs-owned `pyright-langserver` fixture. Future LSP tasks should broaden into
additional workspace-edit/code-action cases and multi-root stress.

## Network Policy

Default benchmark mode should be private/offline, but offline is not what makes
the run hermetic. The closure and ABI manifest do that. Network policy only
controls whether external communication is allowed during a phase.

Named opt-in modes:

| Mode | Use |
| --- | --- |
| `private` | Benchmark tasks where all dependencies are already materialized. |
| `loopback-in-sandbox` | Emacs plus Jupyter/server processes in one sandbox namespace. |
| `share-net` | Human attach, package bootstrap, or tasks that must contact a host-local service. |
| `networked-bootstrap` | Rootfs/package materialization only, never scored benchmark runs. |

Do not let a task silently opt into host networking because a Jupyter server is involved. Prefer one sandbox process tree with internal loopback.
The current selected-rootfs shell runners enforce this with a task-local token:
`SOMA_INSULA_NETWORK=share-net` is accepted only when
`SOMA_INSULA_ALLOW_NETWORK_FOR` exactly matches that runner's task id.

## Capability Policy

Model-visible operations should be capability-gated:

| Capability | Default | Notes |
| --- | --- | --- |
| Read editor state | Enabled | Safe for most tasks. |
| Reversible buffer edit | Enabled when task declares it | Requires expected buffer identity and revision. |
| Save file | Task-specific | Writes only under task fixture. |
| Shell/process input | Task-specific | Process handle must be task-owned. |
| Jupyter execution | Task-specific | Track request outcome; timeout means unknown. |
| Git index write | Task-specific | Only in disposable fixture repos. |
| Network | Disabled | Explicit mode only. |
| Arbitrary Elisp | Disabled | Debug/development only with explicit approval. |
| Destructive filesystem action | Disabled | Requires a checker-owned fixture and explicit policy. |

An Elisp denylist is not a security boundary. Treat arbitrary Elisp as full control of the sandbox and gate it accordingly.

## Decisions To Settle Next

1. **Insula ownership**: Copy/adapt Monarch's `ginkgo.insula` shape into `soma.insula`, or create a shared package. Recommendation: start repo-local in `soma`, because the Emacs/Jupyter/GPU binds are project-specific and should not be blocked on cross-repo generalization.
2. **First profile fidelity**: Minimal Doom-compatible profile or full pinned Doom profile. Recommendation: minimal profile for the first two smoke tasks; pinned Doom profile once rootfs and trace contracts are stable.
3. **Jupyter topology**: Server reachable over host loopback or same-sandbox loopback. Recommendation: same-sandbox loopback for scored gym tasks; host loopback only for human attach/debug.
4. **Task workspace write policy**: Bind `soma` writable or read-only. Recommendation: read-only harness plus read-write task fixture for scored tasks.
5. **Rootfs source**: Build from a general base image or reuse an existing TorchTitan rootfs for the first prototype. Recommendation: prototype may inspect TorchTitan's working path, but `soma` should get its own rootfs contract and build script before any benchmark claim.
6. **Dependency owner**: allow host-projected tools for development or require
   every tool inside the selected rootfs from day one. Recommendation: allow
   explicitly labeled development projections only for Atlas probes; production
   benchmark evidence requires rootfs-owned Emacs, LSP, Jupyter, Python/C++/Rust
   toolchains, and ABI manifests.
7. **Rootfs materialization tool**: use Spack or a lighter bespoke builder.
   Current decision: use a CUDA-image-derived source-build rootfs first, with
   GNU Emacs built inside the rootfs and glibc/dynamic-loader inherited from the
   base container image. Spack remains a candidate for richer compiled
   dependency-DAG and ABI provenance, but it must preserve the same
   container-rootfs ABI boundary and avoid Nix/host ABI leakage.
8. **Emacs adaptation surface**: stock Emacs plus Lisp/advice/module
   instrumentation, or a source-pinned patched Emacs build. Recommendation:
   start with stock Emacs and make the Atlas prove any patch requirement with a
   stock-vs-patched probe. A patch is justified only when stock instrumentation
   cannot expose required evidence faithfully, misses relevant state
   transitions, or adds overhead that would distort the workflow. If used, the
   patch set, configure flags, native-comp/libgccjit closure, and ABI manifest
   are part of the selected Insula rootfs identity.
9. **Trace format**: JSONL only or event schema plus binary artifacts. Recommendation: JSONL event stream with artifact pointers; keep screenshots/PTY logs as optional artifacts, not the primary observation.

## Failure Shields

The first implementation should fail closed on:

1. Missing rootfs contract.
2. Rootfs selected through symlink escape.
3. Missing dependency-closure or ABI manifest for a production benchmark.
4. Emacs, Doom package code, Python, Jupyter, LSP servers, compilers, build
   tools, or test tools resolving from host paths when the task requires a
   rootfs-owned closure.
5. Host `$HOME` visible as Emacs `HOME`.
6. Host Doom config loaded when the task asked for the repo profile.
7. Package/native-comp caches outside `/project/cache`.
8. Jupyter runtime files outside `/project/run` or `/project/cache`.
9. Host `PYTHONPATH`, `LD_LIBRARY_PATH`, conda, or mamba state in the sandbox.
10. Network mode mismatch between task spec and emitted plan.
11. Buffer edit attempted against a stale revision.
12. Checker running outside the rootfs.
13. Private benchmark files visible to the policy.
14. Grader accepting a success message without inspecting state.
15. Repeated request ID submitting a Jupyter cell or process input twice.
16. Patched Emacs binary, source tree, configure flags, patch set, native-comp
    closure, or ABI manifest missing from the selected rootfs evidence.
17. Patched instrumentation that changes the semantics of the command,
    package, process, display, or input path being measured.
18. Source-built stock Emacs claims without `/opt/soma/emacs/bin/emacs`, the
    matching GNU source archive/tree, configure flags, dynamic-loader evidence,
    linked-library evidence, and executable/source hashes in the selected
    rootfs contract.

## Milestones

1. **M-1: Emacs/Doom Internals Atlas**
   Build the source map, mechanism probes, Doom lifecycle trace, package contracts, and derived gym requirements. Exit when the first probe set runs, RFC 0002 assumptions are marked confirmed, revised, or rejected, and the stock-Emacs versus patched-Emacs observability question has an evidence-backed decision path.

2. **M0: Rootfs contract spike**
   Create `soma.insula` with local environment, invocation schema, rootfs
   selection, dependency-closure/ABI manifests, plan emission, and contract
   tests. Include a Spack-backed materialization sketch or spike for the
   ABI-sensitive closure. No Emacs episode lifecycle yet, but tool resolution
   must prove the planned rootfs owns the required commands. If the Atlas
   selects a patched Emacs experiment, this milestone must also prove that the
   patch set is source-pinned and included in the rootfs manifest. Current
   status: `scripts/insula/soma_insula.py` now owns the first generic batch
   runner slice and has unit coverage for rootfs selection, network policy, plan
   emission, mount evidence, environment allowlists, output-root enforcement,
   package-specific runtime fields, daemon bridge runtime fields, and Nix
   contract rejection. Internals probes, the Doom lifecycle worker, and the live
   daemon bridge now use the module-owned `run-command` path.

3. **M1: Lifecycle**
   Build the frozen Doom-compatible profile, isolated worker, terminal frame,
   cold reset, and teardown. Current status: the batch-safe rootfs-local worker
   passes cold, warmed, cold-repeat-1, and warmed-repeat-1 child-process
   checks, records source-built Emacs and rootfs-owned Doom identity, proves
   first-use package/mode loading for the selected minimal profile, and records
   per-child teardown probes with no token-marked process left after exit.
   Terminal-frame fidelity remains before the full milestone exits.

4. **M2: Editor bridge**
   Implement observe, read, select, checked edit, save, await, and finish.
   Current status: a batch-safe request-envelope transport smoke passes through
   direct Elisp submission and a JSONL Unix-domain socket, and a live daemon
   bridge smoke passes through a separate rootfs-owned Python client. It covers
   request IDs, stored terminal status, duplicate rejection, direct-call
   rejection, checked command dispatch, and daemon-backed external-client
   status lookup plus one timer-backed in-flight request observed through
   `request.status` and `request.await`. It also covers passive `buffer.read`,
   checked `buffer.replace_range`, checked `buffer.save`, and stale edit
   conflict rejection in the batch bridge. `episode.finish` now validates
   required request terminal states and final buffer evidence, and the daemon
   benchmark rejects premature finish while a required request is still
   running. The daemon benchmark also proves rootfs-owned `emacsclient --tty`
   can attach a terminal client frame to the same daemon, be observed through
   the bridge, and receive real tmux-delivered terminal key input that changes
   the selected client-frame buffer. It also proves one real terminal
   minibuffer prompt, one real terminal-key Magit staging operation, one Magit
   Transient prompt-state observation/cancel operation, and one Magit Transient
   Stage suffix operation, plus one Magit Diff Transient `--stat` infix state
   transition with diff-buffer evidence and unchanged Git state, plus one
   package-owned Magit `magit-stage-files` prompt completed through tmux input with command identity and hook evidence,
   one stale Magit view recovery path through model-facing
   `magit.observe_sections`, stale repository-fingerprint rejection, refreshed
   section re-selection, and checked `magit-stage`,
   plus one live Comint terminal-input scenario with rootfs-owned process,
   input-marker, run-id, and exactly-once side-effect evidence, and one
   existing-kernel Jupyter eval-region scenario with associated-source
   observation, daemon request status/await completion, preserved kernel token,
   rootfs-owned runtime paths, and exactly-once selected-region execution, plus
	   daemon stale `completion.accept` rejection after terminal-frame human editing,
	   and a cross-package async case where a delayed Jupyter eval request keeps
	   independent status while checked Magit staging completes first. Remaining exit
	   criteria are broader package-operation async variants, richer
	   package-specific prompts, and additional Transient infix classes.

5. **M3: First vertical benchmark**
   Implement selective Magit staging fixture and independent grader. The
   batch-safe selected-rootfs slice now passes with 7 scenarios as expected
   through the first synchronous checked bridge command-dispatch route.
   The daemon bridge now covers both terminal-frame Magit key staging and
   queued `command.dispatch` Magit staging, plus stale/refreshed Magit view
   recovery through the daemon model-facing path. Remaining exit criteria for
   the full vertical are richer terminal-frame package interactions and deeper
   package-owned async behavior.

6. **M4: Runtime task**
   Convert the TorchTitan rootfs Jupyter verifier shape into a fixture/checker
   task with existing-kernel association, request tracking, and
   duplicate-execution rejection. Current status: the batch-safe selected-rootfs
   Jupyter slice passes 8 scenarios as expected, including existing-kernel
   association, wrong-buffer rejection, restart/continuity rejection, timeout as
   `outcome_unknown`, and duplicate retry rejection. The daemon bridge now adds
   a first model-facing existing-kernel eval-region path through
   `jupyter.observe_source`, `jupyter.eval_region`, `request.status`,
   `request.await`, and `jupyter.query_state`. The daemon bridge now also proves
   UI-rendered display-update presentation through `jupyter.eval_region_via_repl`
   and the emacs-jupyter REPL display handler path. The focused Jupyter
   benchmark covers `display_data`/`update_display_data` parent-id, display-id,
   and initial/update payload ordering. Remaining exit criteria are additional
   display variants and recovery paths beyond rejection.

7. **M5: Breadth**
   Add Org, LSP, Dired, Comint, and recovery/coexistence templates with
   reliable scripted solutions. Current status: Comint, Jupyter, Magit, Org,
   Dired, and the first LSP source slice pass together under the selected
   source-built Emacs rootfs in the latest aggregate run,
   `20260929T040533Z-3771476`, with 90 scenarios as expected across 8 families,
   including the daemon pyright diagnostics-after-edit scenario. The generic
   batch runner is now backed by `scripts/insula/soma_insula.py` for
   selected-rootfs resolution, network policy, bwrap plan emission, mount
   construction, and environment allowlist evidence across Magit, Comint,
   Jupyter, LSP, Org, Dired, and the batch bridge. Remaining
   breadth work is additional LSP rename, code-action, multi-root, and
   diagnostics-after-edit variants beyond the landed pyright live-buffer case,
   richer Org and Dired workflows,
   Doom/Evil-specific terminal bindings, richer package-specific prompts,
   additional Transient infix classes, broader daemon async bridge coverage
   for package operations, and broader package/runtime variations under the
   same module-owned Insula authority path.

8. **M6: Benchmark**
   Define sealed splits, baselines, strict scoring, and trace reports. Exit when model comparisons are reproducible under identical interfaces.

9. **M7: Post-training**
   Promote verified SFT data, student recovery traces, and later RL only when held-out strict success improves.

## Recommendation

Start by building the resettable Doom gym and the private benchmark harness, not by training a model.

The first construction deliverable should be the `soma`-local Insula contract plus real Emacs lifecycle. The first benchmark deliverable should be `magit.stage-selected-change.v1`: a selective staging task where a reference scripted policy passes and deliberately wrong policies fail for the intended reason.

Once Magit works, add Jupyter to force persistent runtime identity, request tracking, and conservative retry semantics. That pair gives the core machinery needed for the rest of the gym.

Every later operator trajectory should point to a rootfs plan, initial state, public observations, validated actions, event trace, and private checker result. That is the evidence base needed before SFT or environment RL can say anything useful.

## Full Reference Document: Controllable Doom Gym And Benchmark

This section preserves the source reference draft that sharpened this RFC from an Insula/rootfs environment note into a gym-plus-benchmark design. The RFC sections above adapt it to `soma` and the local Insula pattern; this section keeps the full source text for future specs, tickets, and implementation checks.

**Build the gym as a controllable, resettable instance of real Doom Emacs. Build the benchmark as a separate collection of initial states, user goals, permitted operations, and independently checked outcomes.**

The distinction matters: the gym provides the environment; the benchmark determines whether a model actually accomplished useful work without damaging the session.

For your terminal-first Doom workflow, I would start with **one Linux worker, one Emacs instance per episode, a small Elisp bridge, a Python supervisor, and CPU-only repositories/kernels**. No distributed RL infrastructure yet.

The first milestone should be:

> A scripted policy can repeatedly complete a selective Magit staging task, while a deliberately incorrect policy fails the grader—even when it produces a convincing success message.

Below is the architecture and build order I would use. Names such as `emacs.observe` and `command.invoke` are proposed interfaces, not existing package APIs.

### 1. Separate the system into five components

```text
Model / scripted policy
          │
          │ public observations + permitted actions
          ▼
Python episode supervisor
          │
          ├── Action validation, deadlines, event log
          │
          ├── Isolated episode
          │     ├── Real Doom Emacs + terminal frame
          │     ├── emacs-gym.el bridge
          │     ├── Disposable repositories / Org files
          │     ├── Actual language servers
          │     └── Jupyter kernels / Comint processes
          │
          └── Private evaluation
                ├── Initial-state records
                ├── Expected outcomes and invariants
                └── Independent graders
```

I would organize the repository approximately like this:

```text
emacs-gym/
  profiles/
    doom/                  # Sanitized, pinned Doom configuration
    environment.lock.json  # Software/configuration identities
  elisp/
    egym-transport.el      # RPC framing and request queue
    egym-observe.el        # Editor observations
    egym-actions.el        # Checked action dispatch
    egym-events.el         # Commands, edits, prompts, process events
    adapters/
      magit.el
      org.el
      lsp.el
      jupyter.el
      comint.el
  python/emacs_gym/
    supervisor.py
    lifecycle.py
    protocol.py
    tasks.py
    graders/
  fixtures/                # Public episode materials
  task_specs/              # Not mounted wholesale into episodes
  tests/                   # Harness and grader tests
  runs/                    # Traces and evaluation reports
```

**Keep the learner-facing bridge and evaluator separate.** A model should never have an action like `check_hidden_tests` or access to a file containing the target Git index.

For your existing isolation direction, implement a backend interface around your Insula/bubblewrap environment rather than entangling task definitions with container commands. The task should request “a pinned Doom environment with a Python kernel,” not construct mount namespaces itself.

### 2. Make episode reset reliable before adding model intelligence

#### Build a frozen environment once

Capture the actual configuration you want the model to operate:

- Emacs binary and build features; Doom, package, and bridge revisions.
- Enabled Doom modules, keybindings, and relevant settings.
- Language-server binaries, Python environment, kernelspecs, Git, and terminal configuration.

Preserve your distinction between Evil-enabled programming/text buffers and non-Evil Magit, REPL, and other special buffers. That behavior belongs in the environment contract.

Install packages and perform configuration synchronization during environment construction—not during episodes. Pin the Doom revision you use rather than implicitly migrating to the latest one.

One current implementation detail is worth accounting for: Doom’s source distinguishes XDG data, cache, state, and profile locations, and marks `DOOMLOCALDIR` as legacy. Setting only `DOOMDIR` is therefore not a sufficient isolation strategy. :chatgpt-content-reference{index="0"}

Give every episode its own writable home, runtime directory, temporary directory, and configuration-state directories. Immutable package artifacts can be shared read-only where your pinned installation supports it.

#### Run a real interactive Emacs

A practical arrangement is a foreground daemon plus a terminal client:

```bash
# Inside the prepared episode environment:
emacs --fg-daemon=egym

# Launched by the supervisor with an allocated PTY:
emacsclient --socket-name=egym --tty
```

The supervisor owns these processes and their environment. Emacs supports named daemons, and `emacsclient --tty` creates a terminal frame. :chatgpt-content-reference{index="1"}

Use the terminal frame even when the model receives structured text observations. Window selection, minibuffers, transients, and terminal-specific behavior should exist in the environment.

Do not use `emacs --batch` as the whole gym. Use batch ERT tests for isolated bridge functions; run interaction tests in the live environment. ERT supports both modes, while batch mode has no user interface. :chatgpt-content-reference{index="2"}

#### Define reset as reconstruction

For version zero, use:

```text
create fresh episode filesystem
→ launch fresh Doom process
→ attach terminal frame
→ launch required services
→ construct task state
→ wait for explicit readiness conditions
→ capture initial facts
→ expose first observation
```

Construct editor state through a **privileged setup channel** unavailable to the policy. This channel may create unsaved buffers, position windows, associate kernels, and preload fixtures.

Start with cold process resets. Optimize startup later through a pool of unused, preinitialized workers. Do not initially try to clean a previously exercised Emacs session by killing a few buffers.

Likewise, recreate kernels from setup scripts between episodes. “Preserve the existing kernel” is an **intra-episode requirement**, not a reason to reuse contaminated kernels across independent rollouts.

For repositories, use independent copies or copy-on-write clones, including independent Git metadata. Avoid ordinary Git worktrees as your default isolation mechanism when shared repository metadata would undermine reset independence.

Bubblewrap can provide the filesystem and namespace machinery, but its documentation explicitly says that the security policy depends on how it is invoked. Withhold host credentials, SSH-agent sockets, unrelated home directories, and evaluator files. :chatgpt-content-reference{index="3"}

### 3. Design the observation and action protocol first

This interface is what the model will learn. Stabilize and version it before generating substantial training data.

#### Observations: compact state, expandable details

A minimal observation might look like:

```json
{
  "episode_id": "ep-0081",
  "observation_seq": 37,
  "editor_epoch": 1,
  "selected_frame": "frame-1",
  "selected_window": "window-2",
  "selected_buffer": "buffer-7",
  "interaction": {
    "evil_state": "normal",
    "minibuffer": null,
    "transient": null
  },
  "buffers": [
    {
      "id": "buffer-7",
      "name": "train.py",
      "path": "/workspace/project/train.py",
      "major_mode": "python-mode",
      "chars_tick": 104,
      "modified": true,
      "point": 457
    }
  ],
  "pending_operations": [],
  "recent_events": []
}
```

The initial observation should not contain every buffer’s complete contents. Provide explicit reads for buffer ranges, Magit sections, Org subtrees, diagnostics, and process output.

Use opaque handles for buffers, windows, processes, and kernels. Buffer names are display information, not identity.

For text revisions, `buffer-chars-modified-tick` is useful because it tracks text changes without treating text-property changes as character edits. Pair it with buffer identity and editor incarnation. :chatgpt-content-reference{index="4"}

For positions, choose one protocol convention and enforce it. For example:

> Buffer positions are one-based Emacs character positions; ranges are half-open. LSP positions are converted by the adapter.

Do not silently interchange bytes, Unicode characters, and LSP offsets. LSP positions use zero-based lines and character offsets interpreted according to a negotiated encoding. :chatgpt-content-reference{index="5"}

**Observation functions must not move point, change the selected window, save buffers, clear marks, or otherwise “help” the agent.** Test that property.

#### Actions: generic enough to require decisions

Start with roughly this surface:

| Action group | Examples |
|---|---|
| Inspection | `emacs.observe`, `buffer.read`, `command.describe`, `magit.read_sections` |
| Navigation | `buffer.select`, `window.select`, `point.set`, `region.set` |
| Editing | `buffer.replace_range`, `buffer.save`, `edit.undo` |
| Native operations | `command.invoke`, with an audited command catalog |
| Interaction | `minibuffer.respond`, `interaction.cancel`, `operation.await` |
| Lifecycle | `episode.finish` |

Then add package adapters only where necessary.

For example, `magit.select_section` can select an observed section. But do not expose `magit.stage_the_bugfix`: identifying the correct change is the policy’s job.

A mutation request should include relevant preconditions:

```json
{
  "request_id": "req-19",
  "operation": "buffer.replace_range",
  "target": {
    "editor_epoch": 1,
    "buffer_id": "buffer-7"
  },
  "preconditions": {
    "chars_tick": 104
  },
  "arguments": {
    "start": 457,
    "end": 482,
    "text": "replacement"
  }
}
```

A stale request returns `conflict`; it must not silently apply to a different revision.

Single-buffer modifications can use `atomic-change-group`. Multi-buffer changes need the lower-level change-group machinery; these mechanisms do not roll back kernel execution or filesystem side effects. :chatgpt-content-reference{index="6"}

#### Maintain two action profiles

**Native-operation profile:** structured observations, checked navigation/editing, and actual package commands. This should match your intended deployed operator.

**Keymap profile:** terminal input through the PTY, with a smaller representative task suite. This tests Doom/Evil bindings, prompts, and transient navigation.

Report them separately. A model using typed operations and one navigating keybindings are not solving exactly the same control problem.

Also, calling an Elisp function is not automatically equivalent to interactive invocation. Emacs distinguishes ordinary calls, `call-interactively`, and command-loop execution. Test your dispatcher against real interactive behavior rather than assuming equivalence. :chatgpt-content-reference{index="7"}

### 4. Treat asynchronous execution as a first-class state machine

This is one of the most important implementation details.

A command may return before an LSP request completes. A Jupyter execution may be submitted successfully even though the client times out. A command may enter a minibuffer instead of finishing.

Use explicit operation states:

```text
accepted → running → succeeded
                   → failed
                   → waiting_for_input
                   → outcome_unknown
```

Distinguish these from a request rejected before execution.

#### Do not block the transport inside a prompt

I would implement the Elisp RPC server using Emacs’s network-process support, but make its transport handler validate and enqueue requests rather than directly execute arbitrary long-running commands. Emacs provides network-server primitives through `make-network-process`. :chatgpt-content-reference{index="8"}

Dispatch through a controlled command mechanism. Keep prompt responses and cancellation available while a command is waiting for interaction.

Minibuffer input is a form of recursive editing; it is not equivalent to an ordinary function call that immediately returns a result. :chatgpt-content-reference{index="9"}

#### Wait for specific conditions, not global silence

Examples:

| Operation | Completion condition |
|---|---|
| Buffer edit | Edit committed and new buffer revision observed |
| LSP request | Matching response, associated with the intended document state |
| Magit staging | Relevant subprocess completion, index inspection, status refresh |
| Jupyter execution | Matching request reply and matching execution-status events |
| Comint command | Task-specific output/exit condition, not merely a prompt-shaped string |

Avoid `sleep(1)` as the universal synchronization mechanism.

Emacs subprocess output is processed at particular waiting points; `accept-process-output` can explicitly permit processing, but “some output arrived” does not mean the intended operation completed. :chatgpt-content-reference{index="10"}

#### Make retries conservative

Persist a request ledger in the supervisor:

```text
request ID
action arguments
submission status
associated external request/process ID
observed completion
```

A repeated RPC ID should not submit the operation again.

However, this does **not** magically provide exactly-once execution across every crash boundary. When submission may have occurred but its outcome cannot be established, return `outcome_unknown` and require reconciliation.

That case belongs in the benchmark.

### 5. Write each task as a state-transition contract

A task is more than a prompt. It needs:

\[
\text{Task} =
(\text{initialization},
\text{public goal},
\text{allowed interface},
\text{success predicates},
\text{invariants},
\text{budgets})
\]

Here is a concrete example specification:

```yaml
id: magit.stage-selected-change.v1

public:
  goal: >
    In the current repository, stage only the divide-by-zero fix.
    Leave the debug-print change unstaged.
    Preserve existing staged changes and the unsaved notes buffer.
    Use Magit, and finish with its status buffer visible.

interface_profile: native-v1

fixture:
  generator: selective_staging
  parameters:
    unrelated_staged_change: true
    debug_change_same_file: true
    unsaved_notes_buffer: true
    randomized_initial_window: true

budgets:
  max_actions: 40
  wall_seconds: 180

private_evaluation:
  grader: selective_staging_v1
  checks:
    - expected_index_entries
    - unchanged_worktree
    - unchanged_unsaved_notes
    - unchanged_head
    - magit_operation_provenance
    - final_status_buffer_visible
```

Only the public goal, interface, and appropriate environment state reach the model. The private section and fixture-construction machinery remain outside the episode.

#### Construct the fixture

Create a repository with an initial commit, an unrelated staged change, and two unstaged changes in another file: the desired fix and a debug print. Also create an unsaved notes buffer and start from a non-Magit buffer.

For the first version, put the two changes in separate hunks. Later introduce same-hunk and partial-hunk cases.

Magit supports staging files, hunks, and parts of hunks through section/region selection. That gives you a natural difficulty progression. :chatgpt-content-reference{index="11"}

#### Grade the actual index, not the status message

Have the supervisor inspect:

```bash
git ls-files --stage -z
```

Parse the mode, object ID, stage number, and path into a canonical map. Git documents `--stage` as exposing this index information. :chatgpt-content-reference{index="12"}

Construct the expected index offline from the initial fixture plus exactly the intended change. Compare index entries—not the raw bytes of `.git/index`.

Also compare the working-tree manifest, including relevant untracked files, executable bits, and symlink targets. Compare the notes buffer’s text and modified status independently of the filesystem.

The desired pass condition is:

```text
index = initial index + intended fix only
AND working tree unchanged
AND existing staged changes preserved
AND unsaved notes preserved
AND HEAD unchanged
AND actual Magit staging operation observed
AND requested final view present
```

Do not require one exact command sequence. Require the intended effect and, when explicitly requested, the relevant interaction route.

A trace containing `magit-stage` is supporting evidence—not sufficient evidence by itself.

### 6. Make Jupyter your second substantial task family

Your Jupyter benchmark should check **identity, association, execution scope, and side effects**, not just output text.

An example:

> Connect this source buffer to the already-running experiment kernel. Evaluate the selected diagnostic region once. Do not execute the training cell or restart the kernel.

Construct two kernels with deliberately different state. Associate the buffer incorrectly, or leave it unassociated. Put a harmless diagnostic next to a “training” cell that increments a counter in the fixture.

`emacs-jupyter` distinguishes launching a new kernel, connecting to an existing one, and associating a buffer with an existing client. Those should be separate capabilities in your adapter and separate failure modes in your tests. :chatgpt-content-reference{index="13"}

Record, through the supervisor and service instrumentation:

```text
kernel process identity and start/generation information
buffer → client → kernel association
execution request IDs and submitted code
matching replies and outputs
restart events
fixture side effects
```

Jupyter associates replies and related outputs with requests through `parent_header`. For normal execution, collect the matching reply and matching busy/idle events; do not treat another request’s idle event as completion. Background-thread output can arrive after idle, so tasks involving such output need an additional explicit completion condition. :chatgpt-content-reference{index="14"}

Include these perturbations early: a delayed reply, a busy kernel, a misleading REPL buffer name, a stale selected region, and a connection failure after submission.

**The crucial negative example is a model that retries a cell and executes it twice.** The grader must reject that even when the final printed value looks correct.

Do not rely only on a visible variable such as `kernel_was_not_restarted = True`. Combine supervisor-owned lifecycle evidence with fixture state.

### 7. Start with a small but adversarial benchmark

I would initially write **five development templates in each of eight domains: 40 templates total**.

These are harness-development tasks, not yet a sealed generalization benchmark.

| Domain | Representative cases | Important negative constraints |
|---|---|---|
| Buffers/windows | Open implementation beside tests; switch between same-named files | Preserve unsaved text, intended point/region, and requested layout |
| Magit | Selective staging; unstage one change; inspect a conflict | No unrelated index changes, accidental commit, or reset |
| Org | Refile a subtree; change TODO state; edit a source block | Preserve IDs, properties, children, and unrelated text |
| LSP | References; rename; apply a selected code action | Correct workspace and document revision; no unrelated symbol edits |
| Jupyter | Attach, associate, execute region, interrupt permitted work | Correct kernel; no restart, duplicate execution, or unintended cell |
| Dired | Rename marked files; handle a collision | No overwrite or modification outside the intended set |
| Comint | Submit to the correct process; collect output; interrupt | Correct process/cwd; submit once; preserve unrelated processes |
| Recovery/coexistence | Human edits, stale handles, active prompts, service reconnects | Re-observe rather than overwrite or blindly retry |

For Org, use structural checks as well as byte-preservation checks. Its element API can parse a buffer into an AST with headlines, properties, and parent relationships. Run grading against an immutable captured copy rather than reusing the action adapter’s own transformation code. :chatgpt-content-reference{index="15"}

#### Parameterize difficulty, not just names

Useful variation includes duplicate filenames, multiple repositories, distractor hunks, Unicode before an edit location, narrowed buffers, read-only regions, alternate initial windows, and unsaved changes both inside and outside the target scope.

Names and line-number shuffling alone do not create meaningful generalization.

#### Inject concurrency at semantic boundaries

Instead of:

```text
sleep 0.7 seconds, then modify the buffer
```

use:

```text
after the agent observes the edit target
but before its next attempted write,
insert a simulated human edit
```

The model should encounter a real revision conflict, inspect the changed state, and recover.

Use controlled boundaries for the core suite, then add timing-jitter stress tests separately. This makes failures much easier to reproduce.

### 8. Test the graders more aggressively than the policies

A permissive grader will generate bad training data and misleading rewards.

For every task, create a correct scripted solution and several deliberately incorrect outcomes.

For selective staging, the mutations should include staging everything, staging nothing, staging the wrong hunk, changing the worktree, losing the notes buffer, and merely displaying a fake success message.

For Jupyter, include the right output from the wrong kernel, a restarted kernel with reconstructed variables, duplicated execution, and execution of the neighboring training cell.

The acceptance condition is:

> The correct reference solution passes, and every targeted incorrect outcome fails for the intended reason.

Also test the harness itself:

**Read purity:** repeated observation does not change meaningful editor state.

**Reset independence:** task B behaves identically whether or not another worker previously ran task A.

**Action consistency:** a checked native action and its interactive counterpart produce equivalent relevant state on representative fixtures.

**Stale-write rejection:** an edit against an obsolete revision changes nothing.

**Prompt/cancellation behavior:** transport remains usable while a command awaits input.

#### State the trust boundary honestly

For the first controlled benchmark, do not expose unrestricted Elisp evaluation or a general host shell. Use an audited command catalog and constrain executable fixtures.

Jupyter and Comint still introduce code-execution capabilities. Keep their processes away from evaluator files and editor-control sockets; more permissive experiments may require separate service sandboxes.

An in-process Emacs instrumentation layer is **not tamper-proof against unrestricted arbitrary code in the same trust domain**. A later open-tools benchmark needs stronger external evidence and should not be reported as equivalent to the restricted operator benchmark.

### 9. Define the scoring and dataset split before training

#### Use strict success as the headline metric

For episode \(i\), define:

\[
S_i =
\mathbf{1}
[
G_i
\land I_i
\land P_i
\land B_i
]
\]

Here \(G_i\) is goal achievement, \(I_i\) is invariant preservation, \(P_i\) is compliance with required interaction routes, and \(B_i\) is completion within the published budgets.

Report a domain-macro average so that many easy buffer tasks cannot overwhelm fewer difficult Jupyter tasks.

Alongside it, report destructive effects, blocked prohibited attempts, recovery success, false completion claims, and latency/actions on successful runs.

Keep `goal_reached=true` visible even when strict success fails. That distinction helps diagnose a model that accomplishes the main change but damages surrounding state.

Start RL rewards with this binary terminal outcome. Add shaping only after the graders are trustworthy.

#### Separate infrastructure failures from policy failures

A fixture that never initialized is an infrastructure failure. A policy that exhausted its budget repeatedly sending bad commands is not.

Define retry rules in advance. Record infrastructure failures and reruns; do not silently remove inconvenient episodes.

#### Split by underlying structure

Keep three sets:

**Training:** templates and instances available for demonstration generation and RL.

**Development:** repeatedly inspected during iteration.

**Sealed evaluation:** held-out workflow families/compositions, repositories, and perturbation combinations.

Different seeds from the same template are useful robustness checks, but they do not establish unseen-workflow generalization.

For an initial scored evaluation, a reasonable target is **80 held-out base instances, balanced across domains, each with clean and perturbed variants**. That gives 160 episodes per model configuration. These are proposed counts, not an existing dataset.

Compare the same instances, interface version, observation budget, and runtime budget across models. For uncertainty estimates, resample at the task-family or fixture level rather than pretending every near-duplicate seed is independent.

### 10. Turn the harness into a training-data engine

Keep the core API small:

```python
# Interface sketch, not a complete implementation.
observation, info = env.reset(
    seed=42,
    options={"task_id": "magit.stage-selected-change.v1"},
)

while True:
    action = policy.act(observation)
    observation, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        break

env.close()
```

This follows Gymnasium’s reset/step shape, including separate termination and truncation. You can implement a lightweight wrapper first rather than introducing a full RL stack. :chatgpt-content-reference{index="16"}

The model adapter should whitelist the public observation and tool results. **Never forward the entire `info` dictionary**, which may contain evaluator diagnostics.

For every action, record:

```text
episode, task, environment and model identities
exact observation delivered to the policy
action and validated arguments
relevant precondition versions
submission/completion timestamps
native command and external request provenance
state changes and tool result
private final grading
```

Preserve the exact observation seen at decision time—not a richer snapshot reconstructed afterward. Otherwise, you can accidentally train on information unavailable when the action was chosen.

This is also the right backing format for your Emacs trace explorer: one turn, its relevant state, the action, its effects, and a marked correction.

Use reference scripts first, then strong-model demonstrations, then student-failure recovery traces. Promote data into SFT only after execution and verification.

### 11. The implementation sequence I recommend

| Milestone | Build | Exit condition |
|---|---|---|
| **1. Lifecycle** | Frozen Doom profile, isolated worker, terminal frame, cold reset | Repeated launches and teardown without leaked processes or state |
| **2. Editor bridge** | Observe, read, select, checked edit, save, await, finish | Read-purity, stale-write, prompt, and timeout tests pass |
| **3. First vertical task** | Selective Magit staging fixture and independent grader | Reference policy passes; all targeted bad outcomes fail |
| **4. Runtime task** | Existing-kernel association and single region execution | Wrong-kernel, restart, and duplicate-execution cases are rejected |
| **5. Breadth** | Org, LSP, Dired, Comint, and concurrency templates | Reliable scripted solutions across the development suite |
| **6. Benchmark** | Sealed split, baselines, scoring, trace reports | Reproducible model comparisons under identical interfaces |
| **7. Post-training** | Verified SFT data, student recovery data, later RL | Improvement on held-out strict success, not just training reward |

For the separate FIM/completion model, reuse the buffer snapshots, revisions, edit events, and latency instrumentation—but create a different benchmark. It should evaluate insertions at historical cursor states, stale-suggestion rejection, suffix handling, and functional correctness where applicable. Do not mix completion acceptance with operator task success into one score.

**The best first build is therefore not “all of Emacs.” It is a fresh Doom session, a checked state/action bridge, and one Magit task with a grader that is difficult to fool. Then add Jupyter to force the architecture to handle persistent runtime identity and uncertain asynchronous outcomes.** Once those two work, you have the core machinery needed for the rest of the gym.

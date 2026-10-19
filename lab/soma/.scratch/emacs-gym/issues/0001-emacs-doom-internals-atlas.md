# 0001: Build The Emacs/Doom Internals Atlas

Status: Done
Type: Major task
Quest: Emacs gym on Insula
Blocks: freezing the gym action/observation interface; rootfs-backed Magit and Jupyter benchmark tasks
Blocked by: none
Completed: 2026-09-27

## Intent

Build a source-grounded Internals Atlas for the Emacs/Doom mechanisms the gym will operate. The goal is to validate the gym architecture before freezing its interface, so the environment remains faithful to the user's real terminal-first Doom workflow instead of becoming an easy-to-train imitation.

The central question for every supported operation is:

> Starting from this exact session state, what code runs, what implicit context does it consume, what state can it change, what asynchronous work does it initiate, and what evidence establishes completion?

This task does not implement the full gym. It creates the source map, executable probes, traces, and derived contracts that the gym implementation must satisfy.

## Background

RFC 0001 defines the two-model direction: an Emacs operator and a completion/edit model sharing one Emacs state layer.

RFC 0002 defines the Insula-backed gym and benchmark hypothesis: a rootfs-local Doom-compatible Emacs process, Python supervisor, Elisp bridge, disposable fixtures, private graders, and trace artifacts.

This task revises the order of work. Before implementing a stable gym interface, inspect the actual execution semantics of Emacs, Doom, and the target packages. Correct final filesystem state is not enough; the bridge must preserve the editor/package operation path when the benchmark requires it.

Example risk:

```text
A bridge wrapper calls a Magit function directly.
The Git index changes correctly.
But command-loop context such as this-command differs from a genuine Magit staging command.
Package hooks or post-command behavior can therefore diverge from real operation.
```

## Scope

Study these foundational layers:

1. Emacs Lisp runtime and implicit dynamic context.
2. Buffers, text storage, markers, windows, files, undo, and change hooks.
3. Command dispatch, keymaps, minibuffers, transients, direct calls, `call-interactively`, and terminal input.
4. Reentrancy, subprocess filters, timers, asynchronous work, timeouts, and cancellation.
5. Observation effects, redisplay, terminal frame behavior, and Doom popup/window policies.
6. Doom bootstrap, profile generation, module/package activation, lazy loading, Evil integration, and first-use lifecycle.
7. Package state machines for Magit, Org, LSP, Jupyter, Comint, Dired, and completion.

Also inspect the installed/local target state when available:

1. Emacs binary and build features.
2. Doom core revision, module revision, generated profile, and package lock state.
3. Selected-rootfs Doom profile conventions that the gym must preserve; host
   Doom/Evil conventions only when deliberately comparing or selecting a fuller
   profile.
4. Package advice, hooks, keymaps, and buffer-local settings in a live session.

## Non-Goals

This task does not:

1. Train a model.
2. Implement the production gym interface.
3. Build a full benchmark suite.
4. Add unrestricted Elisp or host shell capabilities to the benchmark.
5. Treat upstream manuals as sufficient without local source/session validation.

## Deliverables

Create these repo-local artifacts:

```text
docs/internals-atlas/
  README.md
  source-map.md
  state-ownership.md
  command-dispatch.md
  doom-lifecycle.md
  package-contracts/
    magit.md
    org.md
    lsp.md
    jupyter.md
    comint.md
    dired.md
    completion.md
  probes/
    README.md
```

Create executable probes under:

```text
probes/emacs-internals/
  README.md
  elisp/
  fixtures/
  scripts/
```

Each probe must record:

```text
mechanism and hypothesis
exact source revision and entry points
minimal fixture
predicted behavior
observed trace
independent state checks
counterexample
derived gym requirement
```

The deliverables may be implemented in smaller follow-up tickets, but this major task is not complete until the Atlas exists and the first probe set runs.

## Required Study Areas

### 1. Lisp Runtime And Implicit Context

Map how functions consume implicit context:

```text
current-buffer
default-directory
current-prefix-arg
this-command
last-command
inhibit-read-only
inhibit-modification-hooks
process-environment
exec-path
buffer-local variables
dynamic bindings
advice
hooks
```

Study source areas:

```text
src/eval.c
src/lread.c
src/keyboard.c
```

Deliverable:

`docs/internals-atlas/command-dispatch.md` must distinguish upstream function definition, installed advice, effective hooks, buffer-local settings, and dynamic caller context.

### 2. Buffers, Text, Windows, And Files

Study distinctions among:

```text
buffer identity
underlying text identity
visited file identity
selected window
current buffer
point and mark
markers
narrowing
read-only state
overlays and text properties
undo state
```

Study source areas:

```text
src/buffer.h
src/insdel.c
src/xdisp.c
```

Required probes:

1. Current buffer versus selected window.
2. Direct buffer read versus showing/selecting a buffer.
3. Base and indirect buffers sharing text.
4. Marker movement under edits.
5. Narrowing or region changes without character changes.
6. Read-only or text-property changes that leave character text unchanged.

Deliverable:

`docs/internals-atlas/state-ownership.md` must define the ownership graph the gym will use instead of treating state as one string per filename.

### 3. Command Path And Interaction Context

Compare these execution routes:

1. Direct function call.
2. `call-interactively`.
3. Command-dispatch path.
4. Real terminal input through a PTY.

Study:

1. Input event decoding.
2. Active map lookup and command remapping.
3. Overriding maps, emulation maps, minor-mode maps, local maps, and text-property maps.
4. Evil state and Doom keybindings.
5. Minibuffer recursive editing.
6. Transient prefix/infix/suffix lifecycle.

Required probes:

1. Direct call versus interactive invocation versus terminal input.
2. Magit staging inside a bridge wrapper versus genuine interactive staging.
3. Minibuffer prompt interruption and recovery.
4. Transient argument lifetime around suffix execution.

Deliverable:

`docs/internals-atlas/command-dispatch.md` must state which bridge operations can call functions directly, which need `call-interactively`, and which require real command-loop/terminal-path tests.

### 4. Reentrancy, Async Work, And Cancellation

Study:

1. Emacs threads and cooperative switching points.
2. Process filters and recursive output processing.
3. Timers.
4. `accept-process-output`.
5. `with-timeout` limits.
6. Package request lifecycles.

Required probes:

1. Reentrant process output while instrumentation is active.
2. LSP-like delayed response after relevant state changes.
3. Jupyter delayed reply and retry.
4. Comint process output while point is not at process mark.
5. Out-of-process watchdog killing a stuck action.

Deliverable:

The Atlas must define operation states:

```text
accepted
running
succeeded
failed
waiting_for_input
outcome_unknown
```

It must also define when a timeout means failed versus unknown.

### 5. Observation Effects And Display

Study:

1. Passive reads versus derived inspections.
2. `buffer-substring` versus `buffer-substring-no-properties`.
3. Redisplay attempts versus user-visible completion.
4. Terminal frame behavior under Ghostty/SSH/tmux.
5. Doom popup/window behavior in the target profile.

Required probes:

1. Observation read-purity over point, selected window, mark, minibuffer, and modified status.
2. Derived inspection that may load packages or trigger fontification.
3. Redisplay or visible-window state after command return.

Deliverable:

`docs/internals-atlas/source-map.md` must classify observations as passive text/state read, derived inspection, or active diagnostic.

### 6. Doom Lifecycle

Pin and study the actual target topology:

1. Doom core revision.
2. Doom module revision or module source layout.
3. Generated profile/configuration state.
4. Package lock and package revisions.
5. Enabled modules and flags.
6. Evil integration and special-buffer behavior.

Trace:

```text
configuration declarations
-> package/module resolution and generated artifacts
-> early bootstrap
-> profile initialization
-> module initialization/configuration
-> user configuration
-> deferred first-use activation
```

Study representative expansions and execution effects of:

```text
doom!
package!
use-package!
after!
map!
```

Deliverable:

`docs/internals-atlas/doom-lifecycle.md` must distinguish:

```text
module declared
package installed
autoload available
feature loaded
mode active in this buffer
external service ready
```

It must also state which of these become part of the gym environment identity.

### 7. Package Contracts

Each package contract must answer:

```text
What state does the package own?
What session context does it require?
What command paths are faithful?
What asynchronous work can it start?
What handles become stale?
What evidence proves completion?
What negative cases must the benchmark reject?
```

#### Magit

Trace:

```text
visible selection
-> section identity and scope
-> chosen operation
-> Git invocation
-> index/worktree effect
-> refresh and reconstructed view
```

Required focus:

1. Sections, section values, and section maps.
2. Region-based staging.
3. `this-command` and post-stage hooks.
4. Stale section handles after refresh.
5. Exact index grading.

#### Org

Required focus:

1. Outline structure, properties, IDs, and parent relationships.
2. Capture/refile lifecycle.
3. Source-edit buffers versus parent Org buffers.
4. Babel execution state.
5. Structural grading from immutable captured copies.

#### LSP

Required focus:

1. Workspace and document synchronization.
2. Position encoding and conversion.
3. Document version checks.
4. Workspace edit application.
5. Diagnostics after edits.

#### Jupyter

Required focus:

1. Source buffer, REPL view, client object, connection, kernel incarnation, execution request, and displayed output.
2. Launching versus connecting versus associating an existing client.
3. Request IDs, replies, busy/idle events, streams, errors, display data, and `parent_header`.
4. Duplicate execution and outcome-unknown recovery.

#### Comint

Required focus:

1. Process mark versus point.
2. Submitting from old history versus current input.
3. Process bookkeeping beyond raw `process-send-string`.
4. Interrupt and output collection behavior.

#### Dired

Required focus:

1. Marks versus deletion flags.
2. Filesystem effects versus Dired buffer effects.
3. Why Dired undo does not generally undo filesystem operations.

#### Completion

Required focus:

1. Completion-at-point replacement ranges and collection properties.
2. Provider exclusivity and ordering.
3. Ghost-text presentation.
4. Cancellation/revision validation.
5. Partial/full acceptance and undo behavior.

## First Probe Set

Implement and run these probes first:

1. Current buffer versus selected window.
2. Direct call versus interactive invocation versus terminal input.
3. Magit staging inside a bridge wrapper.
4. Base and indirect buffers.
5. Marker movement under concurrent edits.
6. Narrowing or region changes without text changes.
7. Minibuffer and transient interruption.
8. Reentrant process output.
9. Cold versus warmed Doom session.
10. Org source-edit propagation.
11. Jupyter delayed response and duplicate retry.
12. FIM acceptance after human editing.

Each probe should have a deterministic reference script and independent state checks. A model is not involved.

## Execution Guidance

Use the Insula/rootfs pattern from RFC 0002 for executable probes when practical:

1. Host code orchestrates only.
2. Substantive Emacs/Doom/package probes run inside the rootfs.
3. Each run writes a plan, environment identity, trace, and outcome.
4. The probe must record whether it used batch Emacs, daemon plus terminal frame, or both.

Batch Emacs is acceptable for isolated Elisp/ERT probes. Interaction semantics require a live Emacs process and terminal frame.

Use upstream source/manuals to design probes, but local installed Emacs/Doom/package state is authoritative for this project once inspected.

## Completion Criteria

This task is complete when all of the following are true:

1. `docs/internals-atlas/README.md` explains the Atlas scope, how to run probes, and how findings constrain the gym.
2. `source-map.md`, `state-ownership.md`, `command-dispatch.md`, and `doom-lifecycle.md` exist and cite the exact source revisions or installed paths inspected.
3. Package contracts exist for Magit, Org, LSP, Jupyter, Comint, Dired, and completion.
4. The first probe set has executable scripts and recorded traces.
5. Every probe records predicted behavior, observed behavior, independent checks, a counterexample, and a derived gym requirement.
6. The Atlas explicitly identifies which RFC 0002 assumptions were confirmed, revised, or rejected.
7. Follow-up tickets exist for implementing the rootfs/lifecycle work only after the Atlas-derived interface constraints are known.

## Completion Evidence

Atlas artifacts:

```text
docs/internals-atlas/
  README.md
  source-map.md
  state-ownership.md
  command-dispatch.md
  doom-lifecycle.md
  package-contracts/
    magit.md
    org.md
    lsp.md
    jupyter.md
    comint.md
    dired.md
    completion.md
  probes/
    README.md
```

Executable probe artifacts:

```text
scripts/insula/run-emacs-internals-probes.sh
probes/emacs-internals/scripts/run-probes.sh
probes/emacs-internals/scripts/run-probes-batch.sh
probes/emacs-internals/elisp/run-probes.el
probes/emacs-internals/fixtures/README.md
probes/emacs-internals/traces/latest/
```

The public probe runner enters a selected Soma rootfs before invoking Emacs. It
clears the environment, disables network by default, uses private
HOME/TMP/RUN/XDG dirs, mounts the CUDA-image-derived rootfs read-only at `/`,
mounts rootfs-owned Doom into `/project/home`, and binds the `soma` checkout
writable at `/workspace/soma` for trace output. `run-probes-batch.sh` refuses
host execution unless `SOMA_ALLOW_NON_INSULA_PROBES=1` is explicitly set.

The old development minimal-bind path still exists for comparison, but only via
`SOMA_USE_DEVELOPMENT_INSULA=1`; it remains classified as `not_rootfs_owned`
with `abi_guarantee=none`.

Current verified command:

```sh
probes/emacs-internals/scripts/run-probes.sh
```

Current result:

```text
35 probes total
35 probes passing
Insula runtime checks passing
SOMA_INSULA_ACTIVE=true
SOMA_INSULA_MODE=selected-container-rootfs
trace plan: probes/emacs-internals/traces/latest/insula-plan.json
trace summary: probes/emacs-internals/traces/latest/summary.md
```

The latest pass adds production-style rootfs/toolchain contract probes. They
record selected rootfs identity, dependency closure, and container-rootfs ABI
manifest evidence for source-built Emacs, rootfs-owned Doom packages, LSP
servers, a real Jupyter kernel, Python/C++/Rust toolchains, build/test tools,
subprocess environments, mount plans, native-comp caches, and ABI-relevant
shared libraries.

First benchmark evidence now also exists for Comint:

```sh
benchmarks/emacs-gym/comint/scripts/run-benchmark.sh
```

The latest run reports 11 scenarios total, 11 scenarios as expected, and Insula
runtime checks passing under `selected-container-rootfs`. It proves the
selected-rootfs benchmark loop can accept a faithful Comint `comint-send-input`
policy and reject raw send, old-history submission, wrong-buffer output, and
duplicate retry after timeout. It now also covers process-mark reobservation
after asynchronous output, stale process-mark submit rejection, partial-output
completion waiting, premature partial-output success rejection, and interrupt
reconciliation.

First package-specific benchmark evidence now also exists for Magit:

```sh
benchmarks/emacs-gym/magit/scripts/run-benchmark.sh
```

The latest run reports 7 scenarios total, 7 scenarios as expected, and Insula
runtime checks passing under `selected-container-rootfs`. It uses real Magit
status sections over disposable Git fixtures, verifies exact index/worktree
state, captures refresh and post-stage hook evidence for faithful staging
paths, and rejects raw Git, direct-wrapper, list-scope, and stale-view bad
policies.

Rootfs-local Doom lifecycle evidence now exists:

```sh
workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh
```

The latest lifecycle summary reports cold, warmed, cold-repeat-1, and
warmed-repeat-1 child Emacs traces passing inside the selected
CUDA-image-derived rootfs. It proves source-built Emacs `30.2` at
`/opt/soma/emacs/bin/emacs`, rootfs-owned Doom core/user config at
`/project/home/.emacs.d` and `/project/home/.doom.d`, Doom startup through the
generated profile, repeated first-use feature loads for Magit, Org+Jupyter,
LSP, Python, Rust, C/C++, and shell support, Jupyter kernelspec discovery via
rootfs Python, and teardown probes with no preexisting
`SOMA_LIFECYCLE_CHILD_TOKEN` process and no token-marked process remaining
after each child exit.

Current aggregate selected-rootfs benchmark evidence:

```sh
scripts/insula/run-emacs-gym-benchmarks-rootfs.sh
```

The latest aggregate run reports 8 benchmark families, 90 scenarios total, 90
scenarios as expected, Insula runtime checks passing, and current selected-rootfs
identity checks passing for every child benchmark plan. The run id is
`20260929T040533Z-3771476`. Org now contributes source-edit, refile, and Babel
parent-buffer checks; Dired now contributes filesystem-owned mark/delete,
rename/collision, overwrite, undo-recovery, and stale same-path mark
revalidation checks; Jupyter now contributes
protocol-level `display_data`/`update_display_data` parent-id, display-id, and
payload ordering checks in the focused family and daemon bridge. The bridge slices now
include selected-window command drift rejection, point edit drift rejection,
narrowed/mark-active/read-only checked-edit rejection, checked completion
acceptance, stale completion rejection, and `episode.finish` final evidence
validation in batch mode, daemon rejection of premature finish while a required
request is still running, a rootfs-owned `emacsclient --tty` terminal client
frame that receives real tmux-delivered key input and is observed through the
daemon bridge, checked point navigation, a real terminal minibuffer prompt opened
by queued daemon `command.dispatch` and completed by tmux input, and queued
Magit hunk staging through daemon `command.dispatch` with terminal request
status observed through `request.await`. The daemon bridge also now proves real
terminal-key Magit hunk staging from the daemon-owned terminal frame with
post-stage hook and index/worktree evidence,
`request.await` timeout as `outcome_unknown` followed by later reconciliation of
the original request, an explicit `waiting_for_input` request/respond lifecycle,
and Magit Transient prompt-state observation/cancel from real terminal input
without changing fixture Git state. It also proves Magit Transient Stage suffix
execution from real terminal input with observed target-file scope and preserved
unrelated file state, Magit Diff Transient `--stat` infix state transition with
diff-buffer evidence and unchanged Git state, plus a package-owned Magit
`magit-stage-files` prompt
opened by queued daemon `command.dispatch`, completed by tmux input, and checked
through preserved `this-command`, post-stage hook identity, exact index/worktree
contents, and unrelated state preservation. It now also proves stale Magit view
recovery through model-facing `magit.observe_sections`, stale
repository-fingerprint rejection, refreshed section re-selection, and checked
`magit-stage`. It now also proves a live Comint
terminal-input path through `comint-send-input`, with rootfs-owned subprocess
identity, stable process id, updated Comint input markers, Insula run-id output,
and exactly-once side-effect evidence. It now also proves one model-facing
existing-kernel Jupyter eval-region path through the daemon bridge:
`jupyter.observe_source` records the associated source buffer and selected
region, `jupyter.eval_region` enters the request lifecycle, `request.status`
and `request.await` observe completion, `jupyter.query_state` records
rootfs-owned Python/Jupyter runtime evidence under `/project/run/jupyter`, and
the grader checks preserved kernel token plus exactly-once selected-region
execution. It now also proves UI-rendered Jupyter display-update presentation
through `jupyter.eval_region_via_repl`, `jupyter-repl-execute-cell`,
REPL-owned display-id range observation, checked terminal navigation to that
range, and tmux capture of `SOMA_DISPLAY_STAGE:updated`. It now also proves
daemon stale `completion.accept` rejection after
terminal-frame human editing, with failed character-revision, point, and suffix
preconditions, conflict evidence, and preserved final buffer text. It now also
proves daemon Python LSP rename through both semantic `lsp.rename_symbol` and
terminal/minibuffer `command.dispatch -> lsp-rename`, with `textDocument/rename`
request evidence, lsp-mode workspace edit application, rootfs-owned
`pylsp`/Python identity, definition/caller/test-file updates, rootfs Python
tests after the rename, prompt observation, tmux-supplied rename input,
original-request terminal status polling, and passive post-command source
observation. It now also proves `daemon-multi-async-status-await`, where two
daemon-owned async requests keep independent status/result identity and can be
awaited out of submission order, plus `daemon-package-async-jupyter-magit`,
where a delayed existing-kernel Jupyter eval-region request and queued Magit
staging request keep independent request/result identity across package state
machines and complete without duplicate kernel execution. It now also proves
`daemon-comint-terminal-interrupt-reconcile`, where terminal-frame `C-c C-c`
invokes `comint-interrupt-subjob`, preserves the rootfs-owned shell process,
and prevents the interrupted command's side effect, plus daemon pyright
diagnostics-after-edit evidence for full-document LSP sync, a coalesced
live-buffer replacement, save, diagnostic publication containing
`Expected expression`, repair/save, and diagnostic cleanup.

The first `soma.insula` authority slice is implemented in
`scripts/insula/soma_insula.py`. It owns selected-rootfs resolution,
symlink/escape checks, network authorization, generic bwrap mount construction,
environment allowlist construction, and plan emission for the generic command
runner. Current aggregate evidence shows Magit, Comint, Jupyter, LSP, Org,
Dired, the batch bridge, and the live daemon bridge plans emitted by this
module. Additional module-authority traces show the Internals Atlas probe
wrapper and Doom lifecycle worker wrapper now route through
`soma_insula.py run-command`:
`probes/emacs-internals/traces/soma-insula-module-command-green1/` and
`workers/emacs-doom-lifecycle/traces/lifecycle-teardown-green2/`.

The selected rootfs profile is intentionally minimal. It does not enable Evil,
Corfu, Vertico, Popup, or Doom's TTY module. Module activation evidence comes
from Doom's loaded `doom-modules` table and `doom-module-active-p`; generated
profile text can contain disabled-module documentation tokens and is not
authoritative on its own.

The Atlas should also answer whether stock Emacs supplies enough operability and
observability for the gym. A patched Emacs build is allowed only as an explicit
source-pinned Insula variant, with stock-vs-patched probes proving that the
patch exposes required evidence without changing the editor/package semantics
being measured.

The Atlas explicitly records RFC 0002 assumptions confirmed, revised, and
rejected in `docs/internals-atlas/source-map.md`.

## Follow-Up Tickets To Create After This Task

1. Implement the `soma.insula` contract spike using Atlas constraints.
2. Extend the rootfs-local Doom lifecycle worker with terminal-frame and
   repeated teardown/leak probes.
3. Implement the checked Elisp bridge with command-context tests.
4. Extend the selective Magit staging benchmark from batch-safe reference
   policies to the checked model-facing bridge and terminal-frame path.
5. Implement the existing-kernel Jupyter benchmark task.
6. Implement the Comint process benchmark task. Captured in
   `.scratch/emacs-gym/issues/0007-comint-process-benchmark.md`; the first live
   daemon terminal-input path and focused process-mark drift, partial-output,
   and interrupt reconciliation scenarios are now covered, while richer PTY
   prompt/process-control variants remain follow-up work.
7. Implement the rootfs-owned LSP source benchmark task.
8. Implement the Spack rootfs materialization spike.
9. Evaluate a source-pinned Emacs patch surface for operability and
   observability.

Created follow-up tickets:

1. `.scratch/emacs-gym/issues/0002-soma-insula-contract-spike.md`
2. `.scratch/emacs-gym/issues/0003-rootfs-local-doom-lifecycle-worker.md`
3. `.scratch/emacs-gym/issues/0004-checked-elisp-bridge-command-context.md`
4. `.scratch/emacs-gym/issues/0005-selective-magit-staging-benchmark.md`
5. `.scratch/emacs-gym/issues/0006-existing-kernel-jupyter-benchmark.md`
6. `.scratch/emacs-gym/issues/0007-comint-process-benchmark.md`
7. `.scratch/emacs-gym/issues/0008-rootfs-owned-lsp-source-benchmark.md`
8. `.scratch/emacs-gym/issues/0009-spack-rootfs-materialization-spike.md`
9. `.scratch/emacs-gym/issues/0010-emacs-patch-observability-spike.md`

## Notes For Implementers

Keep source reading and executable probes connected. Do not write a broad essay about Emacs internals without a runnable counterexample. Do not accept a probe as complete because it prints a plausible trace; preserve the independent state check that proves the behavior.

The final Atlas should make the gym harder to build but easier to trust.

## Original Request Text

This section preserves the source text that motivated this issue.

**Yes—and that should change the order of the project. Before we freeze the gym’s interface, we need a source-grounded understanding of Emacs’s execution semantics, Doom’s configuration lifecycle, and the state machines of the packages we want to operate.**

Otherwise, we risk building an environment that is easy to train against but subtly unlike your actual editor.

There is already a concrete example in Magit’s source: `magit-run-post-stage-hook` checks whether `this-command` is one of its recognized staging commands. A bridge that calls `magit-stage` from inside its own dispatch command can change the Git index correctly while leaving the surrounding command context different from a genuine staging operation. **Correct filesystem output does not necessarily establish faithful editor operation.** :chatgpt-content-reference{index="0"}

I would treat the previous gym architecture as a hypothesis to validate—not an interface to implement unchanged.

What follows is the foundation I would use. I checked upstream manuals and source, but have **not** inspected your installed Emacs binary, Doom revision, generated profile, or live package configuration. Those must eventually become the authoritative versions for this project.

### 1. What “thorough understanding” should mean

We need to be able to answer this question for every supported operation:

> Starting from this exact session state, what code runs, what implicit context does it consume, what state can it change, what asynchronous work does it initiate, and what evidence establishes completion?

That requires three complementary views.

| View | What we need to understand | Resulting artifact |
|---|---|---|
| **Implementation** | C primitives, Lisp functions, macros, hooks, advice, package code | Source map and annotated execution traces |
| **Operational semantics** | Preconditions, effects, interruption points, lifecycle ownership, recovery | Executable contracts and adversarial probes |
| **User interaction** | Key resolution, prompts, selections, window behavior, undo, coexistence with human edits | Interaction tests and benchmark tasks |

The objective is not to memorize every function. It is to understand the mechanisms well enough to predict unfamiliar behavior and investigate it correctly.

For the **gym builders**, that extends into C and package internals. For the **trained operator**, much of this should become compact operational knowledge plus effective introspection—not a requirement to reproduce Emacs source code from memory.

---

### 2. Start with the correct mental model

I would model your setup as:

```text
Ghostty → SSH → tmux → Emacs terminal input
                           │
                   Interactive execution
             command loop / keymaps / minibuffer
                           │
                     Live Lisp state
        buffers / windows / modes / hooks / advice
                           │
                  Package-specific state
          Magit / Org / LSP / Jupyter / Comint
                           │
                  External system state
          filesystem / Git / servers / kernels
```

Doom modifies the loading, configuration, keybinding, and presentation of these components. It is not a replacement for Emacs’s command loop or buffer machinery. Doom describes itself as a configuration framework, with modular configuration, lazy loading, package management, and optional Evil integration. :chatgpt-content-reference{index="1"}

For gym design, the useful abstraction is:

\[
S_{t+1}=F(S_t,a_t,e_t),\qquad O_t=G(S_t)
\]

Here, \(e_t\) represents events not initiated by the current model action: process output, human input, timer callbacks, server responses, and so on.

**The model sees an observation of the session, not the complete session.** A screenshot, a buffer dump, or a list of open files is insufficient by itself.

---

### 3. Understand the Lisp runtime and implicit execution context

This is the first foundational layer.

#### Elisp has more implicit context than a normal RPC API

An operation may depend on the current buffer, dynamically bound variables, buffer-local variables, active modes, advice, and hooks.

Even with `lexical-binding` enabled, variables declared special through forms such as `defvar` remain dynamically bound. Consequently, understanding a function requires examining not only its explicit arguments but also the bindings established by its callers. :chatgpt-content-reference{index="2"}

For example, we should investigate the effective values and ownership of:

```text
default-directory
current-prefix-arg
this-command / last-command
inhibit-read-only
inhibit-modification-hooks
process-environment
exec-path
```

The point is not to serialize all of these into every observation. It is to discover which ones are part of each action’s contract.

#### The function in the source file may not be the whole function you execute

Emacs advice can modify a function’s behavior without replacing its original definition. Doom and packages can therefore change how an otherwise familiar operation behaves. The advice system supports both named functions and function values stored elsewhere. :chatgpt-content-reference{index="3"}

Our investigation should distinguish:

```text
upstream function definition
          +
installed advice
          +
effective hooks
          +
buffer-local settings
          +
dynamic caller context
```

That is the actual operational unit.

#### The C source should be read selectively but seriously

These are the first implementation areas I would study:

| Source area | Questions to answer |
|---|---|
| `src/eval.c` | How are functions evaluated, dynamic bindings established, and nonlocal exits unwound? |
| `src/lread.c` | How do reading, loading, and autoload-related paths work? |
| `src/keyboard.c` | Where are input events decoded, commands selected, hooks run, and command-loop state maintained? |
| `src/buffer.h`, `src/insdel.c` | What owns text, positions, modification counters, and insertion/deletion effects? |
| `src/xdisp.c` | How does editor state become displayed output, and what work happens during redisplay? |

These are actual implementation boundaries, not merely conceptual categories. :chatgpt-content-reference{index="4"}

Allocation and garbage collection also belong in the study, particularly for instrumentation overhead and long-lived traces. I would prioritize their observable lifecycle and latency implications before attempting a comprehensive allocator audit.

**Gym consequence:** an action needs an explicit context contract, not just a function name and argument list.

---

### 4. Understand buffers, text, windows, and files as different objects

This is where a large number of subtle errors will originate.

#### A buffer is not a file, and current buffer is not selected window

Making a buffer current with `set-buffer` or `with-current-buffer` does not display or select it. Most text operations use the current buffer implicitly; the user interacts through the selected window. These normally correspond at command-loop boundaries but can differ during Lisp execution. :chatgpt-content-reference{index="5"}

Likewise, `switch-to-buffer` is not a harmless substitute for `with-current-buffer`: it participates in visible buffer switching and can change window history. :chatgpt-content-reference{index="6"}

Therefore, these are different tasks:

> Read a buffer.

> Show a buffer.

> Select a buffer for user interaction.

> Temporarily make a buffer current while operating on it.

A faithful bridge must preserve those distinctions.

#### Buffer identity and text identity can differ

At the C level, Emacs separates `struct buffer` from `struct buffer_text`. Indirect buffers can have different buffer objects referencing the same underlying text storage. That storage uses a gap and maintains character and byte positions separately. :chatgpt-content-reference{index="7"}

This matters for assertions such as:

> “Only buffer A changed.”

That statement is ambiguous when A and B share text.

Our state model should distinguish:

```text
buffer identity
underlying text identity
visited file identity
window identity
```

An action on an indirect buffer may legitimately affect its base buffer’s shared text. Killing a base buffer also kills its indirect buffers. :chatgpt-content-reference{index="8"}

#### Positions are not durable identities

Integer offsets identify positions in a particular text state. Markers can move with edits, with insertion behavior controlled by their insertion type. But movement with edits does not prove that a marker still identifies the intended semantic target. :chatgpt-content-reference{index="9"}

Our rule should be:

> Use positions or markers to locate candidates, but validate the relevant text and context before consequential mutation.

For a delayed FIM suggestion, for example, “the marker still exists” is weaker than “the prefix, suffix, buffer incarnation, and intended replacement range still match.”

#### A character revision is not a complete state revision

Emacs distinguishes character changes from other modifications, including text properties and overlays. Its internal modification counters reflect these distinctions. :chatgpt-content-reference{index="10"}

An unchanged character revision does not establish unchanged:

```text
point and active region
narrowing
read-only state
keymaps
selected window
package selection
completion overlay
```

Therefore, we need **operation-specific preconditions** rather than one universal `chars_tick` guard.

A buffer replacement may require a text revision and target-range check. A Magit action may additionally require the selected section and view generation. A Jupyter action requires the intended client association and execution target.

#### Undo and text hooks need their own study

Emacs change hooks are not a perfect, universally paired transaction log. The manual explicitly describes exceptions to one-before/one-after pairing, internal changes that bypass them, and suppression through `inhibit-modification-hooks`. :chatgpt-content-reference{index="11"}

That rules out the naïve implementation:

```text
record every before-change
pair with the next after-change
assume the result is a complete event history
```

We should combine bounded event capture with independently checked snapshots and detect instrumentation gaps.

**Gym consequence:** “buffer state” must become an ownership graph and a set of semantic invariants—not one string per filename.

---

### 5. Understand the entire command path—not just `call-interactively`

The conceptual path is:

```text
input events
  → key-sequence processing
  → active-map lookup and command remapping
  → command-loop context
  → interactive argument acquisition
  → command implementation
  → post-command behavior
  → subsequent display and asynchronous effects
```

The command loop reads key sequences and executes their definitions. `command-execute` invokes function commands through `call-interactively`; command-loop hooks and command-history context are additional mechanisms surrounding that call. :chatgpt-content-reference{index="12"}

These execution routes must be compared experimentally:

| Route | What it exercises |
|---|---|
| Direct function call | Explicit function arguments and implementation |
| `call-interactively` | Interactive argument acquisition and interactive-call semantics |
| Command-dispatch path | More of the surrounding command execution machinery |
| Real terminal input | Input translation, active bindings, command execution, and visible interaction |

None should be casually declared equivalent to another.

#### Keybindings are computed from state

Lookup is not simply “global map, then major-mode map.” Emacs considers overriding maps, text-property maps, emulation maps, minor-mode maps, local maps, and command remapping. The precise precedence and exceptions are documented in the keymap implementation model. :chatgpt-content-reference{index="13"}

For Doom/Evil, we need to inspect the **effective** state rather than infer bindings from a cheatsheet. Doom’s Evil module adds configuration and integrations beyond upstream Evil. :chatgpt-content-reference{index="14"}

The operator should be able to investigate:

> What does this key do **here, now**, and why?

Not merely:

> What does this key normally do in Doom?

#### Minibuffers and transients are not interchangeable dialogs

A minibuffer interaction participates in recursive editing and its own completion/input context. A Transient prefix installs a temporary keymap and exposes infix and suffix commands; infixes can configure arguments without exiting, while suffixes often execute an operation and leave the transient. :chatgpt-content-reference{index="15"}

Transient arguments also have a defined lifetime around suffix execution and return to the command loop. Calling a suffix outside that context may not supply the same arguments. :chatgpt-content-reference{index="16"}

**Gym consequence:** we need explicit interaction states and tested dispatch semantics. “Run command X” cannot be the entire control model.

---

### 6. Understand reentrancy, asynchronous work, and cancellation

It would be a mistake to reason:

> “Emacs usually executes Lisp cooperatively, so an action is atomic until it returns.”

Emacs threads are mostly cooperative, with switching at defined operations such as waiting for input or process output. Process filters can also be called recursively when they invoke functions that permit further output processing. :chatgpt-content-reference{index="17"}

That means we must identify **yield and reentry points**.

Consider this hypothetical sequence:

```text
operator observes buffer revision 12
  → starts an operation
  → operation waits for a server
  → another event changes relevant session state
  → response arrives
  → original operation resumes
```

The necessary consistency check may belong after the wait, not only before submission.

#### “Returned,” “finished,” and “displayed” are separate events

I would explicitly distinguish:

```text
command returned
request submitted
external operation completed
editor model updated
user-visible result presented
```

Each package needs evidence appropriate to its own lifecycle.

#### Timeouts are not universal preemption

Emacs’s `with-timeout` checks its timeout when Emacs waits for external events. Its implementation explicitly warns that a program looping without waiting will not detect the timeout. :chatgpt-content-reference{index="18"}

Therefore, the gym needs an out-of-process watchdog in addition to in-Emacs deadlines.

Likewise, cancellation must be classified by target: canceling a prompt, abandoning a request, interrupting a subprocess, and terminating a worker are different operations.

For non-idempotent external work, “the client stopped waiting” must not be interpreted as “the work did not happen.”

**Gym consequence:** completion and recovery must be defined per operation, with explicit unknown outcomes.

---

### 7. Understand display and observation effects

The observer is part of the system.

#### Reading can trigger work

`buffer-substring` can invoke `buffer-access-fontify-functions` before returning text with properties. `buffer-substring-no-properties` does not invoke those functions. Thus, even apparently read-only inspection can have different runtime effects depending on the API used. :chatgpt-content-reference{index="19"}

We should classify observations as:

| Observation class | Proposed rule |
|---|---|
| Passive text/state read | Avoid package loading, display changes, and expensive derived computations |
| Derived inspection | Permit bounded parsing or package queries; report their effects and cost |
| Active diagnostic | Explicitly acknowledged operation that may load code or change diagnostic state |

“Absolutely no state changes of any kind” is not a useful universal purity criterion. The contract should specify which semantic state must remain unchanged.

#### Display completion is not guaranteed by a return value

Emacs’s `redisplay` returning non-`nil` means it attempted redisplay, not necessarily that redisplay completed; newly arriving input can preempt it. :chatgpt-content-reference{index="20"}

For your Ghostty→SSH→tmux workflow, we should measure two separate things:

**Internal correctness:** the intended editor state exists.

**Interaction correctness:** the user sees the intended result without lost focus, corrupted layout, or excessive delay.

Doom’s popup configuration also introduces window parameters and display policies. Window behavior should therefore be tested with the actual popup module enabled, not reconstructed from generic split-window operations. :chatgpt-content-reference{index="21"}

---

### 8. Understand Doom as a configuration build system plus runtime

This is the Doom-specific investigation that we should not skip.

#### First, pin the actual source topology

Current upstream separates Doom core from the official module library; the module repository explains its relationship to core during the ongoing version transition. That makes old tutorials and assumed directory layouts unreliable foundations for an implementation. :chatgpt-content-reference{index="22"}

We should record the exact core revision, module revision, package revisions, and generated profile used by the target session.

#### Trace the configuration lifecycle

A useful conceptual decomposition is:

```text
configuration declarations
  → package/module resolution and generated artifacts
  → early bootstrap
  → profile initialization
  → module initialization/configuration
  → user configuration
  → deferred first-use activation
```

Doom’s bootstrap and `doom.el` describe early initialization, profile loading, module/configuration hooks, and deferred first-input, first-buffer, and first-file hooks. :chatgpt-content-reference{index="23"}

There is an especially important current-source detail: the `doom!` macro in the inspected upstream source documents and implements its module-processing work in noninteractive sessions rather than simply doing the same work during every interactive startup. This is precisely why we must inspect generated configuration, not only `$DOOMDIR/init.el`. :chatgpt-content-reference{index="24"}

#### Distinguish six different notions of “enabled”

Our inventory should separately capture:

```text
module declared
package installed
autoload available
feature loaded
mode active in this buffer
external service ready
```

These are not interchangeable conditions.

Doom’s `after!` is feature-load-sensitive configuration, with additional handling implemented by Doom; it is not a readiness barrier for an LSP server, Jupyter kernel, or completed UI operation. :chatgpt-content-reference{index="25"}

#### Study Doom’s macros by expansion

For the installed revision, I would trace representative uses of:

```text
doom!
package!
use-package!
after!
map!
```

The questions are: when does each form execute, what code does it generate, what is deferred, what is conditional on modules/flags, and what state survives into the session?

We should then compare a cold session, a session after opening its first source file, and a warmed session after using each target package.

**Gym consequence:** environment identity includes generated configuration and activation state—not just repository commits and a Docker image digest.

---

### 9. Study package internals as separate state machines

These packages share Emacs mechanisms but own different kinds of state. We should not force them into one generic “buffer plus command” abstraction.

#### Magit: structured views over Git state

Magit buffers contain nested sections with types and values. Commands can act on the current section or a region selecting compatible sections. Section-specific maps also affect interaction. :chatgpt-content-reference{index="26"}

The source of `magit-stage` branches on selection, diff type, and scope. A region, hunk, file, and list can therefore select materially different behavior. :chatgpt-content-reference{index="27"}

We need to trace:

```text
visible selection
  → section identity and scope
  → chosen operation
  → Git invocation
  → index/worktree effect
  → refresh and reconstructed view
```

Because sections can be recreated during refresh, our adapter should invalidate or re-resolve section handles rather than assume an old object remains authoritative. :chatgpt-content-reference{index="28"}

#### Org: document structure plus derived and editing views

We need separate investigations of outline structure, properties and IDs, capture/refile workflows, agenda views, source editing, and Babel execution.

A concrete distinction: editing a source block opens a separate editing buffer, with changes transferred back through Org’s source-edit workflow. That should not be confused with an ordinary indirect buffer sharing text immediately. :chatgpt-content-reference{index="29"}

A benchmark must distinguish “the source-edit buffer looks right” from “the parent Org block has been correctly updated.”

#### LSP: synchronized document state across a process boundary

The inspected `lsp-rename` implementation obtains interactive rename input, sends `textDocument/rename`, and applies a returned workspace edit. The client also implements position-encoding conversion and document-version checks. :chatgpt-content-reference{index="30"}

The investigation needs to trace:

```text
buffer and workspace selection
  → document synchronization
  → request and position conversion
  → response
  → affected-document resolution
  → edit application
  → subsequent diagnostics
```

For your stack, the fixtures should include Python and C++/CUDA/Rust projects with realistic build context. We should not equate successful text replacement with a valid language-server refactor.

#### Jupyter: clients, kernels, requests, and views

`emacs-jupyter` separates launching a new kernel, connecting to one, and associating a source buffer with an existing REPL client. `jupyter-repl-associate-buffer` sets the buffer’s `jupyter-current-client` and enables interaction with that client. :chatgpt-content-reference{index="31"}

Its client implementation separately handles request-related state, execution status, streams, results, errors, and display-data updates. :chatgpt-content-reference{index="32"}

Our ownership model should therefore distinguish:

```text
source buffer
REPL view
client object
connection
kernel incarnation
execution request
displayed output
```

Your benchmark should be able to reject the correct printed result from the wrong kernel.

#### Comint: input boundaries and process bookkeeping

Comint is not simply “append text and send newline.”

`comint-send-input` behaves differently depending on whether point is after the process mark or in earlier output/input history. Its send helpers also maintain Comint-specific bookkeeping beyond a raw `process-send-string`. :chatgpt-content-reference{index="33"}

The process mark itself is distinct from point and identifies where process output is inserted. :chatgpt-content-reference{index="34"}

A key probe is therefore:

> With point in old history and unsent input at the bottom, what exactly will this command submit?

#### Dired: selection state and filesystem state

Dired marks and deletion flags have different meanings. More importantly, `dired-undo` undoes changes in the Dired buffer; it does not generally undo the underlying filesystem operations or recover deleted files. :chatgpt-content-reference{index="35"}

This is a direct counterexample to any generic “undo the last action” recovery strategy.

#### Completion: providers, presentation, and edit acceptance

The Emacs completion-at-point protocol supplies a replacement range, a completion collection, and properties. Providers are not simply all queried and merged: the first applicable result normally takes responsibility, with defined exceptions such as `:exclusive`. :chatgpt-content-reference{index="36"}

For your separate FIM model, we need to study where generation integrates with that machinery—or deliberately remains separate—and then trace:

```text
trigger
  → context snapshot
  → request
  → cancellation/revision validation
  → ghost-text presentation
  → partial/full acceptance
  → actual edit and undo behavior
```

That final edit path deserves as much attention as model inference.

---

### 10. Build an executable internals laboratory

I would put a small **internals laboratory before the full training gym**.

Each study unit should contain:

```text
mechanism and hypothesis
exact source revision and entry points
minimal fixture
predicted behavior
observed trace
independent state checks
counterexample
derived gym requirement
```

These would be my first probes:

| Probe | What it should establish |
|---|---|
| Current buffer versus selected window | Which operations inspect, display, or select |
| Direct call versus interactive invocation versus terminal input | Which context and hooks differ |
| Magit staging inside a bridge wrapper | Whether command identity changes surrounding behavior |
| Base and indirect buffers | Shared text versus separate buffer state |
| Marker movement under concurrent edits | Why location tracking is not semantic validation |
| Narrowing or region changes without text changes | Why text revisions alone are insufficient |
| Minibuffer and transient interruption | Correct cancellation and continuation behavior |
| Reentrant process output | Whether instrumentation survives nested callbacks |
| Cold versus warmed Doom session | Which capabilities activate on first use |
| Org source-edit propagation | When edits become authoritative in the parent document |
| Jupyter delayed response and duplicate retry | Whether uncertainty is handled without repeated execution |
| FIM acceptance after human editing | Whether stale suggestions are safely rejected |

These are mechanism tests, not model benchmarks. A deterministic reference program should pass them before we ask a model to operate the environment.

#### Instrument at multiple levels

I would use source-level debugging for local control flow, narrow hook/advice instrumentation for selected boundaries, protocol logs for external requests, and supervisor-owned checks for filesystem/process outcomes.

Emacs provides debugging facilities and CPU/allocation profiling; the profiler exposes call relationships and incurs overhead, so measurement should be deliberately scoped. :chatgpt-content-reference{index="37"}

We should not interpret any single trace mechanism as complete. In particular, command hooks do not capture all asynchronous state changes, and change hooks are not an infallible audit trail.

For every important finding, preserve both **the trace** and **the independent outcome**.

---

### 11. How this changes the benchmark and post-training plan

The benchmark should be derived from mechanisms as well as workflows.

Instead of only:

```text
Magit task
Org task
Jupyter task
```

we should build a coverage matrix:

```text
workflow
× interaction state
× state-ownership boundary
× asynchronous condition
× recovery requirement
```

For example, “stage a hunk” should have distinct cases involving a fresh view, a stale view, an active region, unrelated staged changes, an interrupted prompt, and a human edit between observation and action.

I would also keep three claims separate in evaluation:

| Claim | Evidence required |
|---|---|
| **Outcome correctness** | The requested result exists |
| **Operational fidelity** | The intended editor/package path behaved correctly |
| **Coexistence and preservation** | Unrelated human state and external work were preserved |

A model should not receive a full pass merely because it changed the right file.

For training, the internals work should produce two kinds of material:

**Mechanism-grounded examples:** explain or predict behavior from actual state, inspect relevant context, recognize stale assumptions, and identify the correct owner of an operation.

**Executed trajectories:** perform real workflows, including recovery from states the student actually reaches.

I would use source reading to design these examples—not substitute thousands of source-code questions for operating experience.

---

### The revised starting point

**The next foundation should be an “Emacs/Doom Internals Atlas” backed by executable probes, not a larger list of tool endpoints.**

Its core contents should be a versioned source map, state-ownership model, command-dispatch contract, Doom loading/activation trace, and package-specific lifecycle contracts. The gym then becomes an implementation of those contracts, while the benchmark tests their externally meaningful consequences.

The first three investigations I would prioritize are **command execution and context**, **buffer/window/text ownership**, and **Doom’s bootstrap-to-first-use lifecycle**. Then use selective Magit staging and existing-kernel Jupyter execution as end-to-end studies.

That gives us a much stronger target than “a model that knows Emacs commands”: **a model that can reason about what the live editor is doing, choose an operation whose preconditions actually hold, and verify the result without disrupting your work.**

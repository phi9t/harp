# Jupyter Contract

## Owned State

- Source buffer.
- REPL buffer/view.
- `jupyter-current-client` buffer-local association.
- Client object and connection.
- Kernel id, session/incarnation, and execution state.
- Request id, parent header, replies, streams, errors, display data, and idle
  transition.

Primary local sources:

- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:148-154`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:542-575`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:760-783`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:1283-1308`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:1614-1638`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-repl.el:1935-2032`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-messages.el:127-140`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-messages.el:352-366`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-messages.el:621-669`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-server.el:224-233`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/jupyter/jupyter-server-kernel.el:335-370`

## Required Session Context

- Source buffer must be associated with the intended existing client.
- Kernel identity must include more than language/name.
- Kernel process and Python environment must be inside the same pinned Soma
  rootfs contract as the benchmark task unless the task explicitly studies a
  cross-environment failure.
- Rootfs evidence must include more than `SOMA_INSULA_ACTIVE`: verify working
  directory, entrypoint/rootfs sentinels, selected rootfs manifest, Python
  executable/prefix, package path, and emitted bwrap plan.
- Stable run id must survive any rootfs re-entry before kernel launch, so traces
  before and after re-exec land in the same result directory.
- Network or loopback topology is task policy. A network-enabled Jupyter task is
  still hermetic if the server/kernel/package closure and ABI evidence come from
  the selected rootfs; an offline task is not hermetic if the kernel resolves
  from host Python.
- Execution request must be tracked through reply and idle.
- Selected region/code context must be validated before sending.

## Faithful Command Paths

- Launching a kernel, connecting to an existing kernel, and associating a source
  buffer are separate actions.
- `jupyter-repl-associate-buffer` is the source-buffer association path.
- Synchronous helper evaluation differs from interactive REPL execution because
  it can inhibit handlers and history.

## Async Work

- Kernel request/reply messages.
- Busy/idle status messages.
- Streams, errors, display-data updates.
- Server/kernel restart and connection loss.

## Stale Handles

- Client after REPL buffer kill/disconnect.
- Kernel id/session after restart.
- Request id after timeout.
- Source buffer association after buffer mode/file change.

## Completion Evidence

- Request submitted to the intended client/kernel.
- Reply/output matched by parent header/request id.
- Idle status observed after the request.
- Kernel incarnation unchanged if the task required existing-kernel execution.
- Selected code, not adjacent training cell, was executed.
- Kernel process, `sys.executable`, `sys.prefix`, `cwd`, and import path match
  the pinned rootfs contract.
- The benchmark result directory contains the emitted bwrap plan and rootfs
  identity used for both Emacs-side control and kernel-side execution.

## Negative Cases

- Correct printed value from the wrong kernel.
- Timeout treated as "not executed" and blindly retried.
- Kernel restarted to satisfy a result.
- Region expanded and executed beyond requested bounds.
- Emacs is inside Insula but the kernel process uses host Python or ambient
  packages.
- A re-entry wrapper mints a new run id, splitting preflight, kernel launch, and
  execution evidence across unrelated result directories.

Probe `jupyter-delayed-reply-duplicate-retry` records the adapter-level rule:
after submission, timeout is `outcome_unknown`; blind retry of non-idempotent
code creates a second request and must be rejected by the gym.

Probe `jupyter-kernelspec-rootfs-contract` records kernelspec argv/resource
identity from inside the selected rootfs. Probe
`jupyter-real-kernel-rootfs-identity` launches one real Python kernel through
`jupyter_client`, executes one request, matches the shell reply and IOPub idle
state to that request, and records kernel-side `sys.executable`, `sys.prefix`,
`cwd`, import path, plan path, and run id.

Those probes prove batch-safe kernel launch/execution under the selected
rootfs.

## Current Benchmark Evidence

The selected-rootfs Jupyter benchmark is implemented under
`benchmarks/emacs-gym/jupyter/` and run through
`benchmarks/emacs-gym/jupyter/scripts/run-benchmark.sh`. The current
selected-rootfs aggregate trace lives under
`benchmarks/emacs-gym/traces/latest/jupyter/`.

The current summary reports 8 scenarios total and 8 as expected with Insula
checks passing:

- `correct-existing-kernel`
- `bad-missing-association`
- `bad-wrong-source-buffer`
- `bad-whole-buffer-region`
- `bad-new-kernel-continuity`
- `timeout-outcome-unknown`
- `bad-timeout-retry`
- `display-update-existing-kernel`

This proves batch-safe `emacs-jupyter` source-buffer association with an
existing rootfs-owned kernel, selected-region execution, kernel-continuity
checking, wrong-buffer rejection, conservative timeout/retry behavior, and
protocol-level `display_data`/`update_display_data` parent-id, display-id, and
payload ordering.

The live daemon bridge now covers two model-facing existing-kernel paths.
`jupyter.eval_region` remains the source-buffer evaluation path and proves
associated-source observation, request-status/await completion, rootfs-owned
runtime evidence, preserved kernel identity, and exactly-once selected-region
execution. `jupyter.eval_region_via_repl` submits the same source-buffer range
through the associated `jupyter-repl-client` with `jupyter-repl-execute-cell`,
so emacs-jupyter's native REPL display handlers own display insertion and
updates. Scenario `daemon-jupyter-display-update` in
`benchmarks/emacs-gym/traces/latest/bridge-daemon/` observes the updated
`jupyter-display` range in
`*jupyter-repl[soma-daemon-display-update]*`, navigates the terminal frame to
that range, and captures `SOMA_DISPLAY_STAGE:updated` through tmux.

The remaining Jupyter contract work is additional display variants, restart
recovery beyond rejection, and richer async model-facing bridge behavior.

The Jupyter benchmark should reuse the rootfs lessons from prior local
execution harnesses: host shell orchestration is allowed, but substantive
Python and kernel work must be launched through a repo-local rootfs gateway with
strict preflight sentinels and a recorded bwrap plan.

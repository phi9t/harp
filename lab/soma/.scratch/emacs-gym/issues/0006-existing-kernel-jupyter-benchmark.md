# 0006: Implement The Existing-Kernel Jupyter Benchmark

Status: Batch selected-rootfs benchmark captured; terminal/async variants remain
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike; 0004 Checked Elisp Bridge

## Intent

Create a Jupyter benchmark task that verifies attachment to an already-running
kernel and execution of only the requested code.

## Requirements

- Launch disposable kernels inside the pinned Soma Insula.
- Ensure Jupyter server, kernel launcher, Python executable, Python packages,
  and ABI-relevant Python/shared-library surface resolve from the selected Soma
  rootfs.
- Verify kernel-side rootfs identity, not only Emacs-side rootfs identity:
  `cwd`, rootfs sentinels, selected rootfs manifest, Python executable,
  `sys.prefix`, package path, ABI/dependency-closure manifest, and emitted
  bwrap plan.
- Preserve one stable run id across rootfs re-entry, preflight, kernel launch,
  source-buffer association, execution, and summarization.
- Keep host shell logic to orchestration. Substantive Python, kernel startup,
  package imports, benchmark grading, and trace summarization must execute
  through the repo-local rootfs gateway.
- Record kernel id, session/incarnation, client association, source buffer,
  REPL view, request id, parent header, reply, output, and idle transition.
- Distinguish launching, connecting, and associating a source buffer.
- Treat timeout after submission as `outcome_unknown`, not as not-run.
- Reject duplicate execution, wrong-kernel output, region expansion, and kernel
  restart when the task requires the existing kernel.

## Verification

- A scripted correct policy attaches to the existing kernel and runs only the
  selected diagnostic.
- A policy that restarts the kernel, runs the wrong cell, or retries blindly
  fails the grader.
- A policy that runs Emacs in Insula but launches the kernel on host Python
  fails the grader.
- A rootfs preflight emits the bwrap plan and then the kernel reports matching
  rootfs sentinels from inside the running kernel.
- A network-enabled topology is allowed only when declared in the task plan; it
  must not silently install, upgrade, or resolve packages outside the recorded
  rootfs closure during a scored episode.

## Implementation Notes

- Use Soma's selected-rootfs runner shape as the outer harness: emit an
  `insula-plan.json`, preserve one `SOMA_INSULA_RUN_ID`, mount the selected
  rootfs read-only, and run batch Emacs with rootfs-owned Doom under a private
  HOME.
- Reuse the topology learned from TorchTitan's `experiments/jupyter_rootfs`
  work: start a rootfs-local Jupyter server/kernel first, then have Emacs attach
  to that already-running kernel rather than launching or restarting it.
- The Emacs-side path should load `jupyter` and `jupyter-server`, connect to a
  known existing kernel id, associate the source buffer with
  `jupyter-repl-associate-buffer`, and execute exactly the selected region with
  `jupyter-eval-region` or the lower-level request-id-tracked
  `jupyter-execute-request` path.
- The grader must verify kernel-side identity from inside the running kernel:
  `sys.executable`, `sys.prefix`, cwd, `SOMA_INSULA_RUN_ID`,
  `SOMA_INSULA_PLAN`, kernelspec argv/resource paths, shell reply parent id,
  IOPub parent ids, and idle/completion state.
- Immediate negative scenarios: host-resolved kernelspec, Emacs-in-rootfs with
  host Python kernel, missing or wrong buffer association, executing the wrong
  region/cell, kernel restart when continuity is required, and duplicate
  non-idempotent execution after timeout.

## Captured Evidence

Captured on 2026-09-28 under the selected `source-built-emacs-v8` Insula:

- Runner: `benchmarks/emacs-gym/jupyter/scripts/run-benchmark.sh`
- Aggregate runner: `scripts/insula/run-emacs-gym-benchmarks-rootfs.sh`
- Selected rootfs store id:
  `soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Rootfs contract:
  `.scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34/etc/soma-insula-rootfs-contract.json`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Benchmark summary:
  `benchmarks/emacs-gym/traces/latest/jupyter/summary.json`
- Aggregate summary:
  `benchmarks/emacs-gym/traces/latest/summary.json`

The selected-rootfs Jupyter benchmark reports 8 scenarios total and 8 as
expected. It exercises rootfs-owned Emacs plus rootfs-owned Jupyter/Python
fixtures, associates an Emacs source buffer with an already-running kernel,
executes the selected region exactly once, verifies kernel-side rootfs identity
and request/parent-header evidence, and rejects wrong-buffer execution,
restart/continuity violations, duplicate retry after timeout, and host-kernel
drift. It also records `display_data` followed by `update_display_data` for one
request, verifies both messages share the request parent id and display id, and
checks ordered initial/update `application/json` payloads without changing kernel
identity.

The live daemon bridge now reports three model-facing existing-kernel Jupyter
paths. The aggregate run `20260929T040533Z-3771476` proves the source-buffer
`jupyter.eval_region` path: terminal frame switch to the associated source
buffer, `jupyter.observe_source` association and selected-region identity,
stored request completion through `request.status` and `request.await`,
kernel-side `jupyter.query_state`, preserved kernel token, exactly-once
selected-region execution, and request evidence for `display_data` followed by
`update_display_data`.

The focused selected-rootfs daemon trace
`benchmarks/emacs-gym/traces/latest/bridge-daemon/` reports 28 scenarios as
expected and adds UI-rendered display-update presentation. The new
`jupyter.eval_region_via_repl` path submits the source range through the
associated REPL client with `jupyter-repl-execute-cell`;
`jupyter.observe_display` then finds the updated display-id range in
`*jupyter-repl[soma-daemon-display-update]*`, checked terminal point navigation
moves to that range, and tmux capture contains `SOMA_DISPLAY_STAGE:updated`.
The `daemon-package-async-jupyter-magit` path submits a delayed
existing-kernel Jupyter eval-region request, runs queued Magit staging while it
is in flight, awaits Magit first, then awaits the original Jupyter request and
checks preserved request/result identity plus exactly-once kernel execution.

Remaining Jupyter work is not the first existing-kernel batch proof, the first
model-facing daemon eval-region/display-update path, or the first UI-rendered
display-update proof. It is additional display variants and recovery paths that
go beyond rejection.

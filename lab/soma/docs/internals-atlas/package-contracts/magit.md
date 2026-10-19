# Magit Contract

## Owned State

- Git repository identity, worktree, and index.
- Magit status/diff buffers.
- Section tree rooted in `magit-root-section`.
- Section type, value, start/end markers, visibility, and local keymaps.
- Selected section or compatible region selection.
- Refresh state and post-refresh hooks.

Primary local sources:

- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el:282-306`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el:340-345`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el:395-402`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/magit/lisp/magit-mode.el:94-115`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/magit/lisp/magit-section.el:445-491`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/magit/lisp/magit-section.el:552-610`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/magit/lisp/magit-section.el:1429-1494`

## Required Session Context

- Current buffer must derive from `magit-mode` for section-sensitive staging.
- Point/region must identify the intended section scope.
- `magit-diff-type` and `magit-diff-scope` must be observed.
- `this-command` matters for post-stage hook behavior.
- Magit refresh can reconstruct sections; stale section handles must be
  re-resolved.

## Faithful Command Paths

- Function-level staging is insufficient for benchmark fidelity.
- Probe `magit-wrapper-this-command` showed that `magit-run-post-stage-hook`
  only ran when `this-command` was a recognized Magit staging command.
- A faithful staging adapter must either invoke the real command path or
  explicitly model the skipped command-context consequences and grade them.

## Async Work

- Git subprocesses can run through Magit helpers.
- Refresh reconstructs Magit buffers after repository changes.
- Auto-revert may be activated for staged files.

## Stale Handles

- Section objects and markers after refresh.
- Buffer contents after Git state changes.
- Region selections after user movement or refresh.

## Completion Evidence

- Exact index contents.
- Worktree state unchanged except intended hunks.
- Magit view refreshed or intentionally marked stale.
- Post-stage hook behavior captured when the task requires operational
  fidelity.

## Negative Cases

- Correct index but skipped post-stage hook when hook behavior is in scope.
- Staging all changes instead of selected hunk.
- Using stale section handles after refresh.
- Losing unrelated staged/unstaged state.

## Current Benchmark Evidence

The selected-rootfs Magit benchmark is implemented under
`benchmarks/emacs-gym/magit/` and run through
`benchmarks/emacs-gym/magit/scripts/run-benchmark.sh`. The current
selected-rootfs aggregate trace lives under
`benchmarks/emacs-gym/traces/latest/magit/`.

The current summary reports 7 scenarios total and 7 as expected with Insula
checks passing:

- `correct-hunk-stage`
- `correct-file-stage`
- `correct-region-files-stage`
- `bad-git-add-all`
- `bad-wrapper-command-context`
- `bad-unstaged-list-stage`
- `bad-stale-view-stage`

This proves batch-safe selective staging through real Magit status sections and
the first synchronous checked bridge command-dispatch route. The grader checks
exact index/worktree preservation and rejects raw Git staging, wrapper-context
staging, list-scope staging, and stale-view staging. Terminal-frame key
dispatch and async model-facing bridge behavior remain separate gates.

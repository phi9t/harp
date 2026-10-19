# Org Contract

## Owned State

- Outline tree, heading levels, TODO state, properties, IDs, and parent/child
  relationships.
- Capture buffers and capture plist state.
- Refile target cache and target markers.
- Source-edit buffers and parent Org source regions.
- Babel source-block execution state and session buffers.

Primary local sources:

- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/org-src.el:528-546`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/org-src.el:1321-1371`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/org-src.el:1447-1518`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/org-refile.el:433-560`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/org-id.el:309-345`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/org-id.el:653-663`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/ob-core.el:673-678`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/org/lisp/ob-core.el:814-940`

## Required Session Context

- Current buffer, narrowed region, heading at point, and active region.
- File-visiting status for ID location persistence.
- Source-edit parent markers and edit-buffer identity.
- Babel session/default-directory state when source blocks execute.

## Faithful Command Paths

- Structural reads may use `org-element`/Org helpers with explicit buffer
  context.
- Capture/refile workflows need interactive or explicitly supplied target
  context.
- Source editing must distinguish the edit buffer from the parent Org document.

## Async Work

- Babel execution can run external processes or Jupyter/session-backed code.
- Capture finalization can run hooks and interact with window configuration.

## Stale Handles

- Heading positions after edits.
- Refile target cache after file changes.
- Source-edit parent markers after parent buffer mutation.
- Babel session association after process/kernel restart.

## Completion Evidence

- Structural parse of the parent Org document.
- Preserved ID/property drawers and subtree contents.
- Parent source block updated after `org-edit-src-exit`, not merely edit-buffer
  text.
- No duplicate subtree or duplicate ID unless copy semantics explicitly allow it.

## Negative Cases

- Source-edit buffer correct but parent Org block unchanged.
- Refile copies a subtree when move was requested.
- ID duplicated by copy-like behavior.
- Babel result produced in the wrong session or with wrong default directory.

## Current Benchmark Evidence

`benchmarks/emacs-gym/org/scripts/run-benchmark.sh` runs the selected-rootfs
Org structure benchmark. The current aggregate evidence under
`benchmarks/emacs-gym/traces/latest/org/summary.json` reports 6 scenarios total,
6 scenarios as expected, and Insula runtime checks passing. It covers
source-edit parent-buffer writeback, `org-refile` subtree movement with
ID/property/child preservation, duplicate-ID copy rejection, visible-text-only
move rejection, and Babel parent-buffer result insertion.

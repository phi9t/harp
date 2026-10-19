# Dired Contract

## Owned State

- Dired buffer lines.
- Mark characters and deletion flags.
- Current file at point and marked file set.
- Filesystem entries affected by Dired commands.

Primary local sources:

- `/opt/soma/emacs/share/emacs/30.2/lisp/dired.el.gz:2823-2832`
- `/opt/soma/emacs/share/emacs/30.2/lisp/dired.el.gz:3959-4059`
- `/opt/soma/emacs/share/emacs/30.2/lisp/dired.el.gz:4339-4422`

## Required Session Context

- Current Dired buffer and directory.
- Marked file set versus deletion-flagged set.
- Active region and `dired-mark-region`.
- Filesystem collision policy.
- File identity or signature for any stale-sensitive marked path.

## Faithful Command Paths

- Dired marking commands mutate Dired buffer mark columns.
- Deletion first gathers marked files, then performs filesystem deletion, then
  removes entries from Dired buffers.
- Dired undo does not generally undo filesystem operations.

## Async Work

- Some commands may prompt.
- File operations can fail after partial progress.
- Auto-refresh or revert can change buffer presentation.

## Stale Handles

- Marked lines after buffer revert.
- Filesystem state after external changes.
- Point/region after navigation.
- Same-path replacement: a Dired refresh can preserve a visible mark for the
  same basename even when the filesystem object has changed, so destructive
  operations need signature revalidation rather than trusting the mark column.

## Completion Evidence

- Exact filesystem changes.
- Dired buffer state matches filesystem after refresh.
- No unintended delete, overwrite, or rename collision.
- Stale marked entries are checked against filesystem identity/signature before
  destructive operations.
- If undo is invoked, evidence distinguishes buffer undo from filesystem
  recovery.

## Negative Cases

- Marked file set confused with deletion-flagged set.
- Dired buffer looks restored while files remain deleted.
- Collision silently overwrites a file.
- External filesystem state changed between observation and operation.
- A stale same-path Dired mark deletes an externally replaced file.

Probe `dired-undo-does-not-restore-files` attempted recovery after deleting a
file in a disposable directory. The important invariant is external: the file
remained deleted after the Dired undo attempt. Dired-buffer undo state is not a
filesystem transaction log and must not be used as the recovery boundary for
destructive actions.

## Current Benchmark Evidence

`benchmarks/emacs-gym/dired/scripts/run-benchmark.sh` runs the selected-rootfs
Dired filesystem benchmark. The current aggregate evidence under
`benchmarks/emacs-gym/traces/latest/dired/summary.json` reports 8 scenarios
total, 8 scenarios as expected, and Insula runtime checks passing. It grades
filesystem state independently from Dired buffer presentation and covers marked
delete, deletion-flag/ordinary-mark confusion, rename without collision,
collision rejection without overwrite, overwrite rejection, and Dired undo not
being filesystem recovery. It also covers stale same-path replacement after a
Dired mark: the accepted policy records a pre-change filesystem signature,
detects a post-refresh mismatch, clears/aborts the stale destructive action, and
preserves the replacement file; the rejected policy trusts the stale mark and
deletes the replacement.

---
name: sweep-collab
description: Preview and explicitly apply recoverable cleanup of terminal Collab Attempts and old artifacts.
disable-model-invocation: true
---

# Sweep Collab

1. Run from the intended repository:
   `python3 tools/temp_root.py -- python3 tools/collab/collab.py --repo . sweep`.
   Include `--archive-dirty`, `--include-foreign`, or `--retention-days DAYS` only
   when requested. The CLI owns eligibility, identity checks, archives, and retention.
2. Present the returned candidates, their reasons, and skipped items. Explain
   that `--apply` removes those eligible worktrees, branches, and retained artifacts;
   dirty work needs `--archive-dirty`, and unmerged foreign branches need
   `--include-foreign`. Archives preserve removed work before deletion.
3. After explicit user confirmation of the displayed scope, rerun the same
   command and options with `--apply`. Existing explicit authorization to apply
   that exact scope satisfies confirmation; preview still runs first.
4. Report the CLI's removed, archived, and skipped results, including archive
   paths and recoverable refs. Report a refusal as a refusal; preserve its reason.

Restoring an archive uses the CLI's `restore-archive ARCHIVE DESTINATION` command.

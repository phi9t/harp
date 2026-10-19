# Ticket 12: recoverable deep sweep

Implementation base: ff66dd24df0f90ae85c3f1dab286cd3b0c156c8c.
Read specification: docs/efforts/collab-ops/SPEC.md at the implementation base;
subsequent safety clarification read from c9d537b through the lead.
Status: implemented and locally verified; independent acceptance remains with the lead.

## Authority and seams

The user authorized `/implement`, isolated implementation, and commits. The lead
assigned this branch exclusively. The approved seams are the public collab CLI
and the legacy worktree CLI. Tests create real disposable Git repositories.
No real workspace cleanup was performed. Host commands used tools/temp_root.py.

Tracker claims for 12, 13, and 14 refused because their blockers remain open.
No blocker was resolved to bypass that refusal. The lead owns dependent ticket
claims and final acceptance. Skill owners unavailable in this harness were applied
through the repository workflow fallback; TDD was loaded and used at the approved seams.

## Red and green evidence

- Dirty archive test first failed because `sweep --archive-dirty` was unknown.
  The passing test restores distinct staged and unstaged tracked content,
  untracked content, binary ignored content, executable modes, and an external symlink.
- Foreign branch test first failed because `--include-foreign` archived nothing.
  The passing test reads the removed branch's file through its archive ref.
- Retention test first failed because the dry run contained no old terminal logs.
  The passing test keeps old artifacts while their Attempt is active, then removes
  the old log, check ref, and detached check workspace after abandonment.
  Younger evidence and a locked current check workspace survive.
- Additional passing cases retain externally locked dirty worktrees and unknown
  foreign checkouts, refuse tampered archive restoration, retain nested Git stores,
  and remove an individually identified old missing check registration.

Final focused verification:

```sh
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_cleanup.py -v
# 9 tests passed
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_collab.py -q
# 70 existing tests passed
python3 tools/temp_root.py -- python3 -m compileall -q tools/collab tools/agents/wt.py
python3 tools/temp_root.py -- git diff --check
```

## Interfaces and safeguards

`sweep` adds `--archive-dirty`, `--include-foreign`, and `--retention-days DAYS`
(default 30). Dry run remains the default; `--apply` acts.
`restore-archive ARCHIVE DESTINATION` reconstructs a new standalone Git checkout.
It requires an absent destination and validates archive artifact hashes.

`archives.py` preserves the complete worktree inventory, raw index, staged and
unstaged binary patches, and a Git bundle. Archive refs pin both HEAD and a commit
of the staging tree, so staged blobs remain reachable. A separate reconstruction
verifies file hashes, modes, symlink targets, and index bytes before deletion.
The source inventory is checked again after reconstruction. Normal Git worktree
removal follows explicit cleanup of the verified retired tree; no force removal,
reset, rebase, or git clean was introduced.

`retention.py` classifies only collab-owned artifacts of terminal Attempts.
Active or unresolved Attempt evidence stays. Archive refs and archive directories
stay indefinitely. Loose refs use their own modification time; packed refs have
unknown individual age and stay. Missing registrations require a matching
collab-check ref, an exact Git metadata pointer, and a retention timestamp.
Global worktree pruning is not used.

Liveness uses ticket 09's `require_worker_stopped` and `worker_liveness` API.
Unknown run-directory state also blocks removal. Ticket 09's liveness.py was
copied into this isolated tree only for testing and is excluded from this commit.

External private checkouts, foreign checked-out branches, nested Git stores,
special files, split indices, and identity mismatches are retained rather than
partially archived. Foreign branch-only cleanup permits fully merged branches;
unmerged branches require the flag and receive an archive ref before deletion.

Integration glue is limited to CLI sweep flags and restore command dispatch,
and Collab.sweep/Collab.restore_archive facade methods in core.py.
The lead runs the repository gate and performs independent review.

A follow-up regression showed that a preexisting archive-storage symlink wrote
into an unrelated tree. The archive root now requires canonical identity before
any write. The new public test passes and confirms that the unrelated directory
stays empty and the source worktree stays present.

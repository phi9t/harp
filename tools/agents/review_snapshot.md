# Review work in progress

`review_snapshot.py` captures committed, staged, unstaged and non-ignored
untracked files without changing the source index, objects, refs or worktree.
Use it when a branch review must include unfinished local changes.

Run from the source checkout. Choose a new output directory whose parent exists
outside both the source worktree and its Git directory:

```sh
python3 tools/temp_root.py -- python3 tools/agents/review_snapshot.py \
  --base main --output ../review-artifact
```

`--repo PATH` selects another source checkout. The base must resolve to a commit
with one merge-base against HEAD. The manifest records the requested base,
resolved base, merge-base, HEAD and source index digest.

Review all three incremental patches: `committed.patch`, `staged.patch` and
`unstaged.patch`. The last includes untracked additions. `combined.patch` shows
the net change since the merge-base; it can hide changes present only in the
index. All patches include binary payloads.

The artifact contains a self-contained bare Git repository. Its
`refs/snapshots/base`, `head`, `index` and `worktree` refs identify trees, not
commits. Read captured bytes without checking out files:

```sh
git --git-dir=../review-artifact/repository show refs/snapshots/index:path/to/file
git --git-dir=../review-artifact/repository ls-tree -rz refs/snapshots/worktree
```

`manifest.json` lists each file's Git mode, blob identity, SHA-256 and byte size.
Its `path_base64` field preserves arbitrary filename bytes; its `path` field is
an escaped display value. Deleted files remain available in earlier trees.
Symlinks retain their target bytes without opening the target. Raw worktree
bytes are retained without Git filters, text conversion or hooks.

Capture refuses conflicts, submodules, special files, unsafe Git paths, nested
repositories and symlink ancestors. Ignored untracked content is excluded. Git
modes retain executable and symlink identity; ownership, xattrs and other
filesystem metadata are outside this artifact. A tracked file missing from the
worktree is a deletion, including in sparse checkouts.

The tool compares source identities and reads the worktree again before
publishing. Detected changes fail the capture and remove its partial output.
Pause writers and retry. These checks are cooperative: they do not provide an
atomic filesystem snapshot or a boundary against hostile concurrent writers.

This artifact supports WIP review. It is not a Candidate, Integration commit,
check receipt or landing acceptance. Recreate it after changes; final acceptance
still requires checks against the admitted exact Integration commit.

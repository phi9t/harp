# 12 sweep-deep: Sweep, tiers 3-4, foreign branches

Type: feature
Status: open
Owner:
Blocked by: 11
Claimed:

Triage: ready-for-agent

## What to build
--archive-dirty saves the patch and untracked files before removing a dirty worktree. Stale collab refs, old check logs and orphaned check worktrees are pruned past a retention age. --include-foreign archives unmerged foreign branches before removal.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 40, 41, 44). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [ ] A dirty worktree is archived with its patch and untracked files before removal, and only with the flag.
- [ ] Stale refs, logs and orphan check worktrees past retention are pruned; younger ones stay.
- [ ] --include-foreign archives first, then removes.
- [ ] Every archive can restore the removed work.

## Comments

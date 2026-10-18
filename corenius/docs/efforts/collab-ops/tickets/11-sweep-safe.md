# 11 sweep-safe: Sweep, tiers 1-2, dry run by default

Type: feature
Status: claimed
Owner: collab-ops-11
Blocked by: 03
Claimed: 2026-10-12T01:49:53Z expires 2026-10-14T01:49:53Z

Triage: ready-for-agent

## What to build
A sweep command lists, and with --apply removes, merged clean worktrees and branches of terminal Attempts. Clean worktrees with unmerged commits are archived to a recoverable ref first. Fresh and in-progress worktrees are never swept; locked ones are skipped with a reason; branches collab did not create are touched only when fully merged.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 36-39, 42, 43). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [ ] Dry run removes nothing and lists every candidate with its reason.
- [ ] A freshly opened Attempt survives --apply (regression for the audited deletion).
- [ ] A locked worktree is reported as skipped, not failed.
- [ ] An unmerged clean branch is archived before removal and its commits stay reachable.
- [ ] A foreign unmerged branch is listed with its owner and left alone.

## Comments
- 2026-10-12T01:49:53Z collab-ops-11: claimed until 2026-10-14T01:49:53Z

# Effort: retro-fixes

Status: active
Base: main
Integration branch: codex/retro-fixes

## Intent

Implement the five environment repairs approved by the user after the Insula durable retrospective. Preserve the existing durable implementation by copying its uncommitted files into this isolated integration worktree; leave the original worktrees unchanged.

## Scope and non-goals

Gated path: process cleanup, review completeness and qualification evidence affect concurrency and acceptance boundaries. The user's “fix” approves the proposed improvements and behavioral test seams. Named Matt/Superpowers owners are unavailable; apply the documented crosswalk with independent critique and reviewers. No commit, push, PR, merge, host provisioning or branch-protection changes are authorized.

## Milestones and exit criteria

- Review snapshots cover committed, staged, unstaged and untracked changes without modifying the source Git index or executing hooks.
- Existing gate supports selected suites, bounded execution, cleanup and machine-readable timing; socket temporary paths remain private and outside /tmp.
- CI invokes existing checks; manual qualification requires a trusted runner, actual delegated cgroups and supported SQLite, and refuses skips.
- Effort snapshots require no commit and preserve dirty-worktree information; awaiting acceptance never grants landing or unblocks dependent work.
- Focused regressions, broader gate and independent Standards/Spec/defect reviews produce explicit evidence and limits.

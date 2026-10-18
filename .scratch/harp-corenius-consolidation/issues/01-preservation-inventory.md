# Preserve the full Corenius source inventory

Status: in-progress
Owner: Codex
Branch: codex/harp-corenius-consolidation
Parent: [consolidation intent](../spec.md)

## What to build

Record committed main, every registered worktree and dirty path, divergent
branches and durable efforts. Treat unfinished changes as preserved proposals,
not accepted source. Do not mutate the observed repositories.

## Acceptance criteria

- Exact main revision and complete registered worktree inventory are recorded.
- All dirty worktrees and unmerged branches have an explicit disposition before
  Corenius retirement can be proposed.
- Historical qualification limits remain visible.

## Blocked by

The owner explicitly requested all worktrees and branches. Inclusion is resolved;
no retirement is authorized. Preserve complete source branch history and exact
uncommitted source captures before combining them.

## Verification

Read git worktree list --porcelain, git status --porcelain for all 13 worktrees,
git branch --no-merged main, and the assembled controller implementation-status
record directly on devx_b200_dbg. Main is e045d4a6ae2a5efaa53fab93a129b3a57098f7ad.

Document verification: `mise run verify -- --plan` selected diff whitespace and
local Markdown structure/links for the four new intent/inventory records.
`mise run verify` passed on 2026-10-18. This verifies the records only; source
migration and runtime qualification remain open.

Full observation: [inventory](../inventory.md).

## Comments

Most worktrees have uncommitted work. The combined retro-fixes checkout contains
changes overlapping durable, gate, handoff and review checkouts; it cannot be
assumed to supersede them without content comparison.

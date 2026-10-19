# Landing is fast-forward only, after an overlap check

Date: 20260929
Supersedes: 

## Decision

Work lands by fast-forward from its own worktree after checking overlap with dirty files in the target checkout. No rebase, force-push or deletion of a shared branch.

## Why

Several agents share one checkout; the main checkout holds another agent's uncommitted work.

## Consequences

F6 must implement push-first, then local mainline move, as the fixed milano retry order.

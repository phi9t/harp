# Durable efforts are tracked; scratch is not

Date: 20260929
Supersedes: 

## Decision

Effort charters, decisions, tickets, evidence and STATE.md live in `docs/efforts/<slug>/` on main. Execution detail lives in gitignored `.scratch/<slug>/`. Sessions dehydrate and rehydrate with `tools/agents/effort.py`.

## Why

A gitignored `.scratch/` is lost on a fresh clone and invisible to other sessions. An orphan branch was rejected: awkward worktrees and no review visibility.

## Consequences

Policy 2026.09.29.3 changes cross-repo work to start in `docs/efforts/`. STATE.md is committed only at dehydrate to limit churn.

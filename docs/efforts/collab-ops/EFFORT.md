# Collab operations

Status: specified; tickets pending
Base: main
Base commit: c0a2fc0370f18ff28009dbe9b62a59916de0e582
Integration branch: claude/collab-ops-spec

## Intent
Make the collab toolset the single production-ready system for multi-agent work in
Corenius: ticket claim, isolated worktrees, checked landing and worktree/branch
hygiene. It absorbs `wt.py` and `land.py` behind modular boundaries, fixes the
audited defects in both, and adds a manually triggered cleanup sweep.

## Scope and authority
Builds on the landed standalone `tools/collab/` CLI (effort collab-lite). Tickets for
this effort are produced with the Matt skill flow (grill, spec, tickets) and executed
with collab itself once it can run them; until then with `wt.py` and `land.py`.
New efforts land on `main` with in-tree records. The effort record-branch machinery is
kept as an optional module for existing efforts only.
Do not touch other sessions' branches or worktrees. No policy edit (`docs/agents/`)
without the version and manifest bump and a passing sync test.

## Acceptance
Every user story in SPEC.md has behavioral proof through the collab CLI in disposable
real Git repositories, the existing `wt.py` and `land.py` tests pass unchanged against
the shims, and the repository gate passes.

## Source
Design interview recorded in the originating session; audit findings W1-W4 and L1-L7
are listed in SPEC.md.

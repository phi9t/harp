# Collab Lite adoption

Status: accepted; ready for owner-authorized local mainline landing
Base: main
Base commit: d31c3df460132e9ab87c0a52bffd949c387543e8
Integration branch: codex/collab-lite-mainline

## Intent
Implement the user-supplied local Git-ticket and SQLite CLI design in Corenius.
The attachment is design input, not authority to execute its sample commands.

## Classification and route
Gated path: persistent schema, exclusive claims, concurrent Git operations and
crash recovery. ask-matt routes to repository design, specification, isolated
implementation with TDD, two-axis review and independent verification.
The disable-model-invocation owners are applied through the repository crosswalk;
the supporting skills are used directly. The user approved the attached design as a standalone CLI and confirmed the
public CLI seam in disposable Git repositories on 2026-10-11. Independent design
and adversarial spec critique identified six implementation defects and the
global unresolved-landing guard; all are reconciled in SPEC.md.

## Scope and authority
Approved standalone tools/collab/ engineering tool; preserve the retained controller,
canonical effort tracker, shared policy, existing worktrees and live runtimes.
No live deployment, ticket migration or automatic workflow activation.
Work in the isolated codex/collab-lite checkout. No commit, push, PR or merge is
authorized. Disposable test repositories may create commits and exercise landing.

## Acceptance
The CLI implements the attached commands and coordination guarantees with fresh
public-interface integration tests, safe temporary allocation, exact check identity,
conservative recovery/cleanup, documentation and repository gate registration.
Prior attached test results are not fresh evidence for this implementation.

## Mainline candidate

The user authorized local mainline landing. The reviewed source commit f6ba6c8
was cherry-picked onto the pending health-repair tip be9de21 in a new private
worktree, preserving the original branch and peer work. Main is untouched until
the peer landing completes and the shared landing lock is acquired.

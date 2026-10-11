# 01 modular-core: Modular collab, same behavior

Type: task
Status: claimed
Owner: collab-ops-01
Blocked by: 
Claimed: 2026-10-11T23:37:11Z expires 2026-10-13T23:37:11Z

Triage: ready-for-agent

## What to build
Split the collab CLI into modules with narrow interfaces (ticket source, workspace, state store, check runner, landing, policy, CLI) with table-driven dispatch and 120-column lines. Behavior does not change.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 50, 54). Use the project glossary terms.

## Acceptance
- [ ] The 48 existing collab tests pass unchanged.
- [ ] Only the state store module touches SQLite.
- [ ] No line exceeds 120 columns and the CLI entry has no packed statements.
- [ ] The repository gate passes.

## Comments
- 2026-10-11T23:37:11Z collab-ops-01: claimed until 2026-10-13T23:37:11Z

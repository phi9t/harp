# 01 modular-core: Modular collab, same behavior

Type: task
Status: open
Owner:
Blocked by: 
Claimed:

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

# 01 modular-core: Modular collab, same behavior

Type: task
Status: resolved
Owner: collab-ops-01
Blocked by: 
Claimed:

Triage: ready-for-agent

## What to build
Split the collab CLI into modules with narrow interfaces (ticket source, workspace, state store, check runner, landing, policy, CLI) with table-driven dispatch and 120-column lines. Behavior does not change.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 50, 54). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [x] The 48 existing collab tests pass unchanged.
- [x] Only the state store module touches SQLite.
- [x] No line exceeds 120 columns and the CLI entry has no packed statements.
- [x] The repository gate passes.

## Comments
- 2026-10-11T23:37:11Z collab-ops-01: claimed until 2026-10-13T23:37:11Z
- 2026-10-11T23:59:00Z claude: released; first launch had no declared paths
- 2026-10-11T23:45:55Z collab-ops-01: claimed until 2026-10-13T23:45:55Z
- 2026-10-12T00:38:58Z collab-ops-01: resolved

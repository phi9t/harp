# 15 record-branch-module: Optional effort record-branch module

Type: task
Status: open
Owner:
Blocked by: 05
Claimed:

Triage: ready-for-agent

## What to build
Move the effort record-branch landing support into a separate optional module for existing efforts. New efforts land on main with in-tree records.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 51-52). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [ ] An effort that uses record branches lands exactly as before.
- [ ] A new effort lands on main with records in the tree and never loads the module.
- [ ] The module can be deleted without touching the core landing tests.

## Comments
- 2026-10-12T02:29:45Z codex-collab-ops-legacy: Provisional isolated dependency work implemented; see evidence/15-report.md. Blocker acceptance and root integration remain pending; no completion or claim is asserted.

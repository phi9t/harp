# 13 sweep-skill: Repo skill to trigger the sweep

Type: task
Status: open
Owner:
Blocked by: 12
Claimed:

Triage: ready-for-agent

## What to build
A repository skill runs the sweep on request: dry run first, apply only on explicit confirmation.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 45). Use the project glossary terms.

## Declared paths
- `.claude/skills/`

## Acceptance
- [ ] The skill is discoverable alongside the existing run-worker skill.
- [ ] It runs the dry run by default and states what --apply would do.
- [ ] It adds no logic beyond calling the CLI.

## Comments

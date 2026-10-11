# 10 refresh: refresh after trunk moves

Type: feature
Status: open
Owner:
Blocked by: 03
Claimed:

Triage: ready-for-agent

## What to build
refresh abandons an Attempt and opens a new one on current trunk, cherry-picking the worker's commits only when every pick applies cleanly and stopping at the first conflict. Landings by other tools count as ordinary trunk movement.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 24-26). Use the project glossary terms.

## Acceptance
- [ ] A clean refresh carries all commits forward onto the new Attempt.
- [ ] A conflict stops with the worktree left for the worker and nothing resolved silently.
- [ ] A direct landing by another tool followed by refresh works.

## Comments

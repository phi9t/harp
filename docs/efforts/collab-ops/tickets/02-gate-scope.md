# 02 gate-scope: Gate runs collab tests only when the toolset changes

Type: task
Status: open
Owner:
Blocked by: 
Claimed:

Triage: ready-for-agent

## What to build
The collab gate check runs its suite when the change set touches the toolset, and also runs it when the change set cannot be determined.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 53). Use the project glossary terms.

## Acceptance
- [ ] A change outside the toolset skips the suite and says so.
- [ ] A change inside the toolset runs it.
- [ ] An undeterminable change set runs it.
- [ ] Shellcheck is clean for the check script.

## Comments

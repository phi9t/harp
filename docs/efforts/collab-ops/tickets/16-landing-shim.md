# 16 landing-shim: Landing shim

Type: feature
Status: open
Owner:
Blocked by: 06, 07, 15
Claimed:

Triage: ready-for-agent

## What to build
The old landing tool delegates to collab with the same flags and exit codes and a deprecation notice.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 48-49). Use the project glossary terms.

## Acceptance
- [ ] The existing landing test suite passes unchanged against the shim.
- [ ] help carries a deprecation notice.
- [ ] Exit codes match the old mapping.

## Comments

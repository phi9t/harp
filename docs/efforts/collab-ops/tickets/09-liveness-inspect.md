# 09 liveness-inspect: Worker liveness and inspect

Type: feature
Status: open
Owner:
Blocked by: 03
Claimed:

Triage: ready-for-agent

## What to build
open-attempt accepts the launcher's process id or run directory. recover and abandon refuse while that worker is alive. inspect lists exactly what blocks a state change.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 29-31). Use the project glossary terms.

## Acceptance
- [ ] recover and abandon refuse while the recorded worker runs (tested with a real child process).
- [ ] They proceed once it has exited and --confirm-stopped is given.
- [ ] inspect names hidden index flags, ignored files and locks that block a change.

## Comments

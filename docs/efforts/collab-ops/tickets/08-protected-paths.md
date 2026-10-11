# 08 protected-paths: Protected paths from config

Type: feature
Status: open
Owner:
Blocked by: 01
Claimed:

Triage: ready-for-agent

## What to build
The protected-path list is set at init. Corenius defaults add the gate scripts, the gate entry point and the constitution to the existing coordination and policy paths.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 27-28). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [ ] A candidate touching a gate script, the gate entry point or the constitution is refused at submit.
- [ ] A repository can choose its own list at init.
- [ ] The default list matches the documented one.

## Comments

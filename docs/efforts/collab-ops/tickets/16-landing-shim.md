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

## Declared paths
- `tools/agents/land.py`
- `tools/collab/`

## Acceptance
- [ ] The existing landing test suite passes unchanged against the shim.
- [ ] help carries a deprecation notice.
- [ ] Exit codes match the old mapping.

## Comments
- 2026-10-12T02:29:45Z codex-collab-ops-legacy: Provisional isolated dependency work implemented; see evidence/16-report.md. Blocker acceptance and root integration remain pending; no completion or claim is asserted.

# 14 worktree-shim: Worktree shim

Type: feature
Status: open
Owner:
Blocked by: 11
Claimed:

Triage: ready-for-agent

## What to build
The old worktree tool delegates to collab with the same flags and exit codes, a configurable mainline, remote-aware status, locked creation with validated metadata, and a deprecation notice.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 46-49). Use the project glossary terms.

## Declared paths
- `tools/agents/wt.py`
- `tools/collab/`

## Acceptance
- [ ] The existing worktree test suite passes unchanged against the shim.
- [ ] status reports divergence against the remote when one exists.
- [ ] Concurrent creation of the same worktree yields one success and one refusal.
- [ ] Corrupt metadata is reported, not read as empty.
- [ ] help carries a deprecation notice.

## Comments

# 03 open-attempt: Open an Attempt from a Corenius ticket

Type: feature
Status: resolved
Owner: collab-ops-03
Blocked by: 01
Claimed:

Triage: ready-for-agent

## What to build
open-attempt reads a tracker-format ticket, claims it through the tracker, creates the worktree on an agent/topic branch and records the Attempt in the state store. The old claim command is renamed. A crash at any step is recoverable.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 1-4, 32-34 (claim side)). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [x] Opening an Attempt for a ready ticket claims it in the tracker, creates the worktree and records the Attempt together.
- [x] A blocked, claimed or resolved ticket is refused.
- [x] A crash after the claim or after the worktree exists leaves a state that recover resolves.
- [x] The tracker remains the only writer of ticket files.
- [x] Behavior is proven through the CLI in disposable Git repositories.

## Comments
- 2026-10-12T00:39:05Z collab-ops-03: claimed until 2026-10-14T00:39:05Z
- 2026-10-12T01:49:47Z collab-ops-03: resolved

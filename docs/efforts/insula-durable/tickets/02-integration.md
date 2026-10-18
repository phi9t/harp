# 02 integration: Integrate durable execution with existing Insula launch and checks

Type: feature
Status: open
Owner:
Blocked by: 01
Claimed:

Triage: ready-for-agent

## Acceptance
- [x] Launch through verified existing Insula with full contract binding, readonly code and stable state mount.
- [x] Compare process-group and delegated cgroup backends; refuse missing cgroup capabilities.
- [x] Add gate and usage documentation and record fresh full validation.

## Requirements
SPEC.md requirements 6-7 and clarified backend scope.

## Exclusions
New keeper, real GPU claims, DBOS integration, Git publication.

## Comments
- 2026-10-13T04:08:09Z codex: Integration implemented and verified in parent worktree. Claim/closure remains blocked by ticket 01 until human-controlled Candidate acceptance and landing. See ../VALIDATION.md for 69 passes, four native cgroup skips, clean independent reviews and full gate limits.

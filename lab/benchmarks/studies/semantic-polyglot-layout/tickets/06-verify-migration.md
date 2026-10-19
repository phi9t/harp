# 06 verify-migration: Verify the complete migration

Type: task
Status: resolved
Owner: TRAE
Blocked by: 05
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** The owner receives a reviewed Linux-only migration with working current navigation, preserved command contracts and fresh evidence covering the complete approved specification.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [x] Check every specification requirement against the integrated result and reconcile omissions; component checks from earlier tickets remain valid at the integrated tip.
- [x] Run the required repository engineering gate and relevant benchmark/Insula checks when affected; retain the current experimental gate coverage.
- [x] Rerun the real Linux controller client integration and root command/asset checks at the exact integrated source revision.
- [x] Audit active path references and ownership boundaries; current navigation and gates work, evidenced forwarding entry points preserve their contracts, and historical references remain intentionally preserved.
- [x] Retain independent Standards and Spec reviews for each implementation ticket and independent branch verification; resolve blocking findings before claiming completion.
- [x] Distinguish missing prerequisites and baseline failures from migration regressions; report unverified checks and remaining risks explicitly.
- [x] Preserve runtime/schema/protocol behavior, Linux-only scope, parked status and all qualification caveats.
- [x] Present the reviewed branch/worktree and fresh evidence without assuming commit, push, pull-request or merge authorization.

## Exclusions

No new product functionality, shared policy rollout, historical artifact rewriting, real-model qualification or Git publication without explicit authority.

## Verification evidence

Integrated repository gate, relevant optional suites, client evidence, root command and asset checks, path/consumer audit and independent fixed-base/spec review with exact source identity.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.
- 2026-10-13T01:02:45Z TRAE: verified the integrated semantic grouping migration on branch `worktree-semantic_grouping_refactor`. Evidence: `python3 tools/temp_root.py -- sh tools/agents/check.sh` exited 0 with `check: all green`; optional `CHECK_TEAM_CONTROL_PLANE=1 python3 tools/temp_root.py -- sh tools/agents/checks.d/70-team-control-plane.sh` exited 0; independent Standards and Spec reviews passed after tracking the root `Cargo.toml` and restoring historical source-read path references. Remaining authority boundary: no push, pull request or merge performed.

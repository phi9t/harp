# 08 landing-tracker-docs: Land pending tracker documentation from setup worktree

Type: task
Status: resolved
Owner: codex-landing-audit
Blocked by:
Claimed:

## Acceptance
- [x] Recover the pending issue-tracker and triage documentation from the setup worktree.
- [x] Use durable effort paths and the implemented tracker lifecycle and commands.
- [x] Version the edited policy documents and manifest together; sync tests pass.
- [x] Independent Standards/Spec and branch defect reviews pass.
- [x] All repository gate components pass, with environmental failures rerun.

## Intent and authority

The owner requested landing all pending worktrees/branches. Short path:
one repository, documentation-only recovery, reversible and no new API or
architectural decision. This request authorizes commit and local fast-forward
landing. No remote is configured. Preserve existing worktrees and shared refs.

## Source and disposition

Read against main48d71588fff00ac70fa4a29e066a47a6b05f2ebd and setup branch
codex/setup-matt-pocock-skills at the same commit. The two source documents
were uncommitted in the setup worktree; their bodies are copied exactly, with
new version headers. Their instructions agree with tracker.py's existing
header contract, lifecycle and durable effort policy.

An independent read-only audit accounted for all25 other nonancestor branches:
SQLite source equals483f1bc; MAC source equalsabe8715; reference and intent
variants are retained and parked; worker evidence branches are superseded.
All210 archived dirty-source entries passed SHA256 verification. Original
worktrees, private configuration, transcript, scratch artifacts and historical
variants remain preserved. No product source change or qualification is claimed.

## Verification

`python3 tools/test_sync_agentic_workflow.py`:24 tests passed.
`python3 tools/agents/test_tracker.py`:9 tests passed; tracker lint and
`git diff --check` passed. Independent Standards/branch review by
/root/landing_audit and Spec review by /root/landing_spec_review:PASS,
no blocking findings. The source documents were confirmed byte-identical
apart from added version headers. `sh tools/agents/check.sh` passed the tooling, SQLite fault-injection and
monitor suites, but exited1 when /tmp exhausted its inodes during legacy
control-plane compilation and root Cargo setup. No behavioral test failed.
Only this session's generated caches were moved to /data02; original
source trees and retained fixtures were preserved. With TMPDIR on /data02,
`sh tools/agents/checks.d/70-team-control-plane.sh` and `cargo test --offline`
both passed on retry (exit0). All required components therefore have fresh
passing evidence; the original full-command exit remains recorded as1.

The product tree stayed identical to main48d71588 throughout verification.
Final committed-tip policy tests and tracker lint are required before the
conditional fast-forward. Independent reviewers passed the documentation
bodies and version metadata before commit.
The retained native fault matrix does not establish complete live baseline
qualification or reset existing open findings.

## Comments
- 2026-10-05T06:09:09Z codex-landing-audit: claimed until 2026-10-06T06:09:09Z
- 2026-10-05T06:43:11Z codex-landing-audit: resolved

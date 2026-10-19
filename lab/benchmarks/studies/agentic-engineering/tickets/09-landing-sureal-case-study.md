# 09 landing-sureal-case-study: Land stable Sureal practice case study from concurrent worktree

Type: task
Status: resolved
Owner: codex-landing-audit
Blocked by:
Claimed:

## Acceptance
- [x] Preserve the complete case study byte-for-byte from the source worktree.
- [x] Preserve commit-read provenance, evidence labels and inference distinctions.
- [x] Independent Standards/Spec reviews find no unresolved blocking defects.
- [x] Focused policy and tracker checks pass with inherited product source unchanged.

## Intent and authority

The owner requested all pending worktrees/branches be landed. During the
initial queue's gate a new Claude worktree appeared with this uncommitted
case study. Short path: documentation-only, reversible, single repository;
no architecture or public API change. Copy the stable authored snapshot and
leave the original tree and branch untouched. Recommendations remain a case
study and do not alter canonical policy.

## Source

Source branch: worktree-agent-a74425ca9ef305e92 at48d71588fff00ac70fa4a29e066a47a6b05f2ebd.
Source path: .claude/worktrees/agent-a74425ca9ef305e92/docs/practice/imported/sureal-semantic-layout.md.
SHA256: ebb4fa909e7f8e1d696e49855cd076698896a7a941b44becb1af79633801291c.
The source hash matched before and after copying. The document declares
Sureal commit pins, Corenius commit read, observation time and READ,
OBSERVED, TRANSCRIPT, INFERRED and LEAD provenance labels.

## Verification

Product code and test trees remain unchanged from maina0b51e0 and previously
passed repository components, including the disk-inode failure retries
recorded in ticket08. Repeat focused documentation/tracker checks on the
committed candidate; no product qualification or policy adoption is claimed.

Independent Standards/defect review by /root/landing_audit and Spec review
by /root/landing_spec_review:PASS. Standards independently verified all six
Sureal commits, key coordinator behavior and the scheduling calculations.
Private transcript claims remain source-attributed; those records were not
read by the reviewers. Sync policy tests24/24 and tracker tests9/9 passed;
tracker lint passed. Conditional exact-tip checks precede local landing.

## Comments
- 2026-10-05T06:45:58Z codex-landing-audit: claimed until 2026-10-06T06:45:58Z
- 2026-10-05T06:47:19Z codex-landing-audit: resolved

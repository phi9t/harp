# Acceptance record: ticket 02

Candidate: cddfa2087 (rebased by its worker onto 4533ecb73). Landed on `effort/lead-worker-readiness` by fast-forward 2026-10-05T09:52Z.
Reviewer: the lead (not an independent context).
Review rounds: 1 of 3. Round 1 rejected Candidate 955059a (`02-review-1.md`); the same worker session re-integrated and reworked it in 21 minutes.

| Axis or check | Result | Evidence |
|---|---|---|
| Ticket and spec | pass | Seven acceptance items read against the diff. Item 3 ("writes in no worktree it did not create") is amended by the review: the target's own checkout is fast-forwarded; no worker worktree is touched and nothing is rebased |
| Repository standards | pass | Standard library only; two files outside declared paths (the lead-driven harness module and its test), reported by the worker |
| Gate on the exact Candidate | pass | lander, worktree-tool and worker suites; every bench harness suite; hand-off and both landing scenarios; tracker lint |
| Lead's reproduction | pass | Landing onto a checked-out target leaves that checkout clean with the landed file present; an overlapping tracked change is refused with code 3 and nothing moves. Round 1's Candidate failed the first of these |
| Re-integration | pass | The Candidate contains the integration tip that includes ticket 13 |

# Decision: reference workflow design approved for specification

Date: 2026-10-01.
Authority: owner replied "approved" to the complete reference design and its
remaining proposals.

The owner approved worker lifecycle hardening as the representative effort,
independent Codex and Claude implementations from the same base, optional LLM
relay with mandatory deterministic supervision, at least 30 days of local raw
evidence retention, and eight worker launches per lead including repairs.
Prior accepted decisions remain effective as recorded in the other decisions
from this date.

Approval permits specification and proposed ticket decomposition. The resulting
spec and graph require review before implementation. It does not authorize Git
commits, remote pushes, mainline integration or paid live demonstration runs.

Independent critique identified two current tool behaviors to account for:
no-push landing still advances mainline, and dehydration creates a commit.
The specification distinguishes preparation from real integration and makes
local commit authority a prerequisite for the live recovery demonstration.

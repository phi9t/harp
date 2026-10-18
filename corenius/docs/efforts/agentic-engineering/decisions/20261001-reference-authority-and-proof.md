# Decision: reference workflow authority and proof

Date: 2026-10-01.
Authority: the owner answered "accept" to recommendations Q4–Q6 in the
`ask-matt` design discussion. Builds on `20261001-reference-quality.md`.

## Accepted decisions

1. Leads may inspect, plan, launch workers within an agreed budget, cancel
   their own workers, verify results, and prepare reviewed changes.
   Commit/push actions follow explicit session authorization. Mainline
   integration follows the repository's declared authority; the current
   canonical policy requires human integration. This design decision does
   not itself grant commit, push, integration or live-run budget authority.
2. The lead reproduces acceptance checks for every ticket. A separate reviewer
   checks the assembled change. Security boundaries, concurrency and landing
   tools also require independent adversarial verification. Supervisors report
   execution compliance; their verdict does not establish artifact correctness.
3. Demonstrate a real improvement to Corenius independently under Codex and
   Claude leads, with the same contract and acceptance criteria. Include
   controlled failures: contradictory dispatch rejected before launch,
   defective output rejected during verification, full worker cancellation,
   and fresh-session recovery. Demonstration runs must remain isolated from
   one another and from unrelated work.

## Rationale

Bounded autonomy lets the lead complete authorized work without repeated
permission requests. Reproduced acceptance checks and separate review address
the session's observed false passes and incomplete evidence. Controlled
failures make recovery requirements observable rather than aspirational.

## Remaining design questions

Evidence retention and redaction, documentation structure, concrete launch
budgets, harness adapter boundaries and rollout sequence are not yet settled.
The specific representative improvement and detailed failure fixtures will be
selected in the specification, grounded in existing toolbox gaps.

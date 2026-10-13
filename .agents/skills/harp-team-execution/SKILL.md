---
name: harp-team-execution
description: Coordinate permitted agents on independently owned Harp implementation or review tasks; use when delegation improves delivery without constructing new coordination infrastructure.
---

# Team execution

Use existing agent tools when permitted delegation helps. Keep tightly coupled work inline with [harp-task-execution](../harp-task-execution/SKILL.md). Tool availability alone does not require delegation, a new service, or a user decision about execution mechanics.

## Bounded delegation

The coordinator retains the full goal and integration responsibility. Give each worker:

- The useful operation and bounded deliverable, including settled decisions.
- Necessary inputs, actual source identity, owned files/worktree, and dependencies.
- Permitted actions, resource limits, required checks, and stop condition.
- The next resolving probe for any uncertainty.

Use fresh, targeted context. Parallel writers need non-overlapping ownership and independent verification; serialize shared-state integration. Overlap independent review or implementation with external waits where safe. Answer routine worker questions from the existing brief instead of relaying every uncertainty to the user.

## Results and review

Treat a worker's report as a claim to inspect. Verify actual changes and evidence, including composed behavior across task boundaries. Use [harp-scope-review](../harp-scope-review/SKILL.md) for independent review of requirements and quality. Keep the reviewer independent from implementation; follow any stricter repository review requirements.

Workers report completed, completed with concerns, needs context, or blocked, with evidence and the exact next action. Reserve completed statuses for completed deliverables. Resolve context and routine mechanics within existing scope. Escalate only changes to intent, acceptance, authority, or a blocker that safe alternatives cannot resolve.

When two repair cycles or material scope growth stop advancing the useful operation:

1. Preserve the source identity, dirty state, accepted behavior, and open findings.
2. Classify findings against the current contract and operating conditions.
3. Re-slice oversized work into owned deliverables and necessary dependencies.
4. Restore progress using the smallest sufficient path; retain optional work separately.

This checkpoint does not waive blocking defects or require another user approval by itself. Keep meaningful progress updates while continuing authorized work.

## Integration

Review the composed candidate against the full request, [changed-surface verification](../../../docs/agents/verification.md), and current risks. Select from the integrated diff and include consumer checks across worker boundaries; do not promote the release gate into every local landing. Reverify affected behavior after repairs. Advisory findings alone do not block acceptance; unresolved required behavior and concrete current-operation risks do. Follow Harp's landing rules and existing authority. An initial useful result does not complete the remaining plan.

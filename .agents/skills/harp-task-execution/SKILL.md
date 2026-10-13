---
name: harp-task-execution
description: Execute an authorized Harp task or written plan through verified results; use for implementation and bounded repairs, including recovering from failures without adding unrelated prerequisites.
---

# Task execution

Read the current intent, plan, repository instructions, existing changes, and retained evidence. Identify the next useful operation and the full acceptance criteria. Inspect existing work before replaying effects.

## Execute and repair

1. Use an isolated worktree for feature work and preserve unrelated changes. Claim and update the relevant [local Markdown ticket](../../../docs/agents/issue-tracker.md), including verification evidence. Reuse existing tools; execution does not require a new coordinator.
2. Execute the next bounded operation under existing authorization. Adapt routine mechanics to evidence while preserving constraints and acceptance.
3. Select checks using [the test-selection guidance](../../../AGENTS.md#test-selection), prioritizing observable contracts over implementation details. Run the relevant checks and inspect their actual outputs. Capture source identity, commands, results, and material limits where needed to reproduce the claim.
4. Diagnose failed checks against the required behavior. Repair contract regressions; update or remove affected tests that only freeze incidental implementation choices, preserving meaningful coverage. Stay within scope and rerun affected verification. Retry with new evidence or a changed hypothesis, rather than repeating the same failing action.
5. Continue through the remaining requested work. Use [harp-scope-review](../harp-scope-review/SKILL.md) for independent review when required by the task or repository.

## When supporting work grows

If repeated repairs stop advancing the useful operation, identify that operation, its actual blocker, and the smallest sufficient repair. Preserve current work and separate optional infrastructure with a future activation condition. Resume a bounded alternative only when it satisfies the same requirements. This is an internal sequencing correction, not an automatic permission request or permission to drop required scope.

Escalate when a material intent decision or required authority is missing, or safe in-scope alternatives cannot resolve a blocker. State the exact condition, evidence, and decision or external change needed. Continue independent authorized work. Respect explicit design-only and approval restrictions.

## Completion and landing

Report delivered behavior, the current blocker, and the next resolving check in concise updates. Completion requires evidence for the whole request, not only the first happy path. Concrete data loss, authorization, isolation, and credential failures remain blocking in the current operation.

Follow `AGENTS.md` and [the verification contract](../../../docs/agents/verification.md) for changed-surface `mise run verify`, explicit staging, applicable import-receipt refresh, and local commits. Inspect the selected plan against the whole candidate, add relevant consumer checks, and use `--base` for multiple commits. Missing prerequisites block their selected checks; unrelated missing caches do not block landing. Do not download dependencies, hydrate caches, or silently skip selected checks. `verify-all` remains the release gate. Human merge/release authority and the explicit-push rule remain intact. Do not repeat a landing-choice question if the user already supplied that choice.

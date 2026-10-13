---
name: harp-outcome-design
description: Resolve open design decisions around the first useful result; use when a Harp feature or behavior change needs design, preserving settled intent and existing authorization.
---

# Outcome design

Resolve the decisions needed to deliver the user's result. Read the current request, earlier decisions, applicable repository instructions, and relevant code before introducing a new design process.

## Intent and authority

Identify the intended result, constraints, execution authority, and material decisions still open. A user-supplied or approved approach plus an implementation request is sufficient to continue; writing it down does not create another approval gate. For a genuinely unresolved product or architecture decision, present the tradeoff and recommendation and ask only for that decision. Continue independent authorized work.

Respect design-only requests and explicit approval restrictions. Silence, automatic continuation, or design approval alone does not authorize implementation. Existing authority does not expand to publication, deployment, extra spending, or a different outcome.

## First useful result

Choose a real end-to-end operation that produces evidence relevant to the goal, using existing tools and interfaces where sufficient. Keep the full requested outcome visible: an initial demonstration is a milestone, not completion.

For each proposed prerequisite, name:

- The concrete operation it enables.
- The observed failure or explicit requirement that makes it necessary now.
- The smallest sufficient implementation under the same constraints.

A local data-quality report can begin with an existing CLI on representative records. A durable coordinator becomes necessary only if the requested operation needs it. For a requested production coordinator, recovery and isolation are required deliverables. Preserve current controls for source identity, authorized resources, isolation, data integrity, and cleanup.

## Resolve and record

1. Inspect working examples and constraints. Compare alternatives where the decision remains open.
2. Prefer a bounded probe when it resolves uncertainty more cheaply than speculative design. Use a visual only when it clarifies the decision.
3. State the selected approach, first useful operation, risks, and verification. Separate current requirements from optional improvements and name when those become necessary.
4. For substantial work, record the design in the existing workstream's `design.md` under `docs/workstream/<workstream>/`. Follow Harp's canonical ownership rules for technical prose. Small changes need only the relevant intent record and a concise explanation.
5. Continue with [harp-delivery-plan](../harp-delivery-plan/SKILL.md) for multi-step implementation when authorized. Otherwise deliver the design and respect the pending decision.

Keep responsibilities and dependencies explicit. Follow existing patterns and avoid unrelated refactoring. Deferred requirements stay visible and are never reported as complete.

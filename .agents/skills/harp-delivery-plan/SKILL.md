---
name: harp-delivery-plan
description: Plan multi-step Harp work as independently verifiable useful slices; use before implementation when sequencing, dependencies, or acceptance need to be made explicit.
---

# Delivery plan

Turn the requested outcome into an executable sequence with an early useful result and complete final acceptance. Read the existing intent record and design; reuse settled decisions and execution authorization.

## Plan the next useful operation

Use the existing workstream tracker or plan under `docs/workstream/<workstream>/`. Scale detail to the task. Capture:

- **Outcome:** the full requested result and final acceptance.
- **First useful result:** the real operation, inputs, expected evidence, and stop condition.
- **Necessary prerequisites:** operation enabled → observed failure or explicit requirement → smallest sufficient implementation.
- **Remaining scope:** later required capabilities, optional improvements, and their activation conditions.
- **Authority:** existing authorization and explicit restrictions or unresolved decisions.

Order slices around observable results rather than subsystem completion. Prefer an existing CLI, serial execution, or local artifact when it meets the same constraints. General coordination, caching, and recovery infrastructure belongs on the critical path only when necessary for the requested operation. Explicit production requirements, current safety controls, and repository gates remain required.

For the next slice, name owned files, inputs, commands, expected observations, relevant failure cases, and completion criteria. Include code when a fragile interface needs precision; do not implement the entire solution in prose. Later uncertainties may remain explicit with a concrete probe that resolves each one.

## Verification and dependencies

Use behavior tests and real artifacts. Keep focused iteration checks distinct from Harp's full gate before commit or landing. Pair a contract test with the implementation it needs in the same coherent landing. A partial slice cannot satisfy full-task acceptance.

For permitted external executables, specify an authoritative pinned release, integrity check, confined location, and command proving it works. Honor Harp's cache rules; never hydrate Lean dependencies as an incidental prerequisite. Tests use isolated inputs and do not mutate canonical data.

Before execution, check scope coverage, dependency necessity, input/type consistency, and whether the next operation is actually runnable. Fix routine sequencing within the approved intent. Ask only when a material decision or missing authority prevents progress.

## Continue

If implementation is authorized, use [harp-task-execution](../harp-task-execution/SKILL.md). Use [harp-team-execution](../harp-team-execution/SKILL.md) when permitted delegation has independently owned work or review to perform. Select routine execution mechanics without another user approval stage. For a planning-only request, deliver the plan and stop.

# ACE versus MCE

## Observe

ACE updates structured context artifacts. MCE evolves a skill that constructs
and updates context, then lets an inner agent execute that skill.

## Predict

Classify the editable object in each system. Decide whether the system changes
`D_t`, the stored artifact, or `H_t` and `R_t`, the procedure that learns and
uses context.

## Compare

ACE is `persistent-adaptation` because the learned object is primarily the
context artifact. MCE is `harness-improvement` because the outer loop changes
the context-management skill itself.

## Explain

Two systems can produce similar final prompts while changing different causal
objects. The [context-engineering deep dive](../context_engineering_deep_dive.md)
traces artifact adaptation, procedure adaptation, and harness-code search.

## Missing fact

Neither source measures whether an accepted child context improver produces
better later context improvers under a matched protected envelope.

## Transfer

Fork [ACE](../diagnostics/cases/ace.json) or
[MCE](../diagnostics/cases/mce.json), then change `editable-components` and
observe the classification boundary.

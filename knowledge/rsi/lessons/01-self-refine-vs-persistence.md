# Self-Refine versus persistent adaptation

## Observe

Self-Refine repeatedly critiques and revises one task output. Its feedback and
revisions live inside the current episode. ACE writes selected context lessons
that later episodes can retrieve.

## Predict

Before revealing the comparison, identify which system changes durable state,
what later consumes that state, and the strongest honest claim for each.

## Compare

Self-Refine changes the current answer and remains `output-refinement`. ACE
changes cross-episode context artifacts and reaches `persistent-adaptation`.
Better current-task quality alone does not establish persistence.

## Explain

The decisive boundary is whether an artifact survives the episode and changes a
later trajectory. [Improvement types](../concepts/improvement-types.md)
separates current output, persistent adaptation, harness change, and successor
gain.

## Missing fact

ACE would need evidence that its learned context procedure, not only its stored
artifact, improves later improvement work to support a stronger recursive
claim.

## Transfer

Fork either [Self-Refine](../../../content/diagnostics/cases/self-refine.json) or
[ACE](../../../content/diagnostics/cases/ace.json) into the diagnostic workbench and change
the persistence scope.

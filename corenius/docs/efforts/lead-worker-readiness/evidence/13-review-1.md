# Lead review 1 of ticket 13: two changes before it can land

Reviewed Candidate: 37b0f5b. On a separate checkout of that commit every bench harness suite passes, the
report command prints what your report says, and the sampled claims held. The row schema, the validation
and the notes command are good and stay.

## 1. The core-set summary merges the three instantiations into one figure (should fix)

`_core_counts` adds up rows for lead-driven, local-controller and workflow-controller together. With one
scenario run for the lead-driven instantiation only, the summary reads `passes=1 ... not yet run=2`, and
nobody can tell which instantiation passed. The owner reads this summary to decide whether the lead-driven
instantiation is good enough, and the whole point of the table is to compare the three. Print one summary
line per instantiation and proof (for example `Core set (deterministic, lead-driven): ...`). The ticket's
rule that deterministic and live are never merged applies the same way to instantiations. Update the test.

## 2. A scenario's story numbers belong in the scenario, not in the report code (should fix)

`LEGACY_SCENARIO_STORIES = {"handoff": (1, 2, 4, 44)}` in the results module hardcodes one scenario's
stories; every later ticket adds scenarios and would have to edit that table. Put `stories = [1, 2, 4, 44]`
in the hand-off scenario's definition file and remove the table (you may edit that one file although it is
outside your declared paths; list it in the report). A scenario with no `stories` key should make the
report say so for that scenario instead of silently counting it as outside the core set.

## Protocol

Add commits on top of 37b0f5b; do not rewrite it. Extend the report. Finish with a new final commit
`ticket 13: ready for review` on a clean tree, then make no further commits.

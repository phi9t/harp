# AlphaEvolve versus DGM

## Observe

AlphaEvolve evolves solution programs against executable evaluators. DGM edits
an agent repository and preserves a branching lineage of evaluated descendants.

## Predict

Identify the editable object, archive semantics, generation edge, and whether
the accepted child is measured as a better producer of later children.

## Compare

AlphaEvolve is `persistent-adaptation`: selected solution programs persist, but
the agent harness is not the primary edited object. DGM is
`harness-improvement`: an editable agent repository and lineage persist. An
accepted lineage edge alone does not imply `successor-improvement`.

## Explain

[Harness search](../chapters/harness-search.md) separates solution-program
populations from editable agent lineages. [System state and notation](../concepts/system-state-and-notation.md)
places promotion authority and the protected archive outside candidate control.

## Missing fact

DGM needs a matched next-generation experiment showing that an accepted child
produces better later accepted children, not only better benchmark solutions.

## Transfer

Fork [AlphaEvolve](../../../content/diagnostics/cases/alphaevolve.json) or
[DGM](../../../content/diagnostics/cases/dgm.json), then toggle later-candidate production
and next-cycle measurement separately.

# Evolutionary search supplies an outer loop, not an aligned objective

## What Weng claims

**CLAIM — [WENG-HARNESS], “Evolutionary Search.”** Evolutionary methods suit
irregular, non-differentiable harness and program spaces when candidates can be
executed and scored. AlphaEvolve retains and mutates a program population; DGM
uses an archive of coding-agent variants rather than greedily replacing one
incumbent.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#evolutionary-search).

## Mechanism

**INFERENCE.** Evolution contributes population, parent selection, mutation,
evaluation, diversity, and archive policies. A branching archive can preserve
stepping stones that are weak under the current scalar score but structurally
useful later. It also makes lineage, rollback, and negative results explicit.

The evaluator remains outside evolution. Search pressure amplifies the supplied
fitness function; diversity can improve exploration but cannot repair a
misaligned or gameable objective.

## Hidden assumption

**INFERENCE.** Candidate fitness must be comparable across branches, with
matched resource vectors and stable task/evaluator identities. Archive
membership must not be confused with deployment authority.

## Demonstrated versus proposed

**EVIDENCE — [ALPHAEVOLVE], [DGM].** The inspected sources support program
evolution, candidate pools or branching archives, and author-reported results
in their studied domains. DGM explicitly keeps its foundation model frozen.

**MISSING.** Open-ended archives and repeated generations do not independently
prove open-ended recursive improvement, safe autonomous deployment, or a
better future improvement operator.

## What would weaken this interpretation

If archive diversity collapses to score-equivalent variants, or if a
resource-normalized greedy baseline performs equally well, the archive's
claimed exploration benefit would be weak. If changing the evaluator reverses
the lineage ranking, the fitness claim is evaluator-specific.

## Reader checkpoint

Why can a diverse archive be useful even when only one candidate is eventually
promoted?

<details>
<summary>Check your answer</summary>

The archive preserves alternative mechanisms, failed branches, and stepping
stones that may become useful after later tasks or mutations. Promotion still
requires a separate external decision; archive presence means “retained for
search,” not “authorized to replace the active system.”

</details>

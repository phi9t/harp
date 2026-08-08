---
id: rsi-system-alphaevolve
kind: concept
title: AlphaEvolve
summary: Evolutionary search over executable solution programs with automated evaluators, program databases, author-reported scientific applications, and externally fixed objectives.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-automated-research
related:
  - kind: compared-with
    target: rsi-system-dgm
attachments:
  - content/diagnostics/cases/alphaevolve.json
claims: []
human_review: null
---

# AlphaEvolve: evolutionary search over solution programs

## Search object

**EVIDENCE — [ALPHAEVOLVE], §§2–3.** AlphaEvolve uses language models to
generate and revise executable programs, automated evaluators to score them,
and a program database to retain useful candidates and evolutionary context.

The candidate is usually a solution program for a human-specified problem, not
the full agent that runs the search.

## Evolution loop

1. Sample promising and diverse programs from the database.
2. Give selected code and feedback to model-based proposers.
3. Generate candidate modifications.
4. Execute objective and validity evaluators.
5. Add valid candidates and measurements to the database.
6. Repeat under a fixed problem definition and resource budget.

Executable evaluation makes large search populations possible, but evaluator
quality and computational budget define the reachable discoveries.

<details>
<summary>Original sources for this mechanism</summary>

- Search system and program database: [AlphaEvolve, §§2–3](https://arxiv.org/abs/2506.13131).
- Applications and reported results: [AlphaEvolve, application sections](https://arxiv.org/abs/2506.13131).
- Checked-in text: `evidence/weng/text/alphaevolve.txt`.

</details>

## Evidence and limits

The white paper reports applications in data-centre scheduling, hardware
circuits, mathematical algorithms, and parts of model-training infrastructure,
including partial self-application to systems used by the model family.

These are author-reported applications. Problem specifications, evaluators,
models, code-execution environment, and promotion decisions remain
human-controlled. Partial self-application does not establish a generally
self-improving successor process.

## Claim ceiling

The Atlas classifies AlphaEvolve as `persistent-adaptation`: selected solution
programs persist and improve, while the harness that produces them is not the
primary accepted candidate. The system does not measure later
successor-production gain.

## Reading routes

- [Weng: evolutionary search](../weng/07-evolutionary-search.md)
- [AlphaEvolve versus DGM lesson](../lessons/05-alphaevolve-vs-dgm.md)
- [Searching for better harnesses](../chapters/harness-search.md)
- [Original white paper](https://arxiv.org/abs/2506.13131)

---
id: rsi-system-autoresearch
kind: concept
title: Karpathy Autoresearch
summary: A bounded edit-run-measure-keep research loop with one editable training file, fixed-duration experiments, scalar validation, and no autonomous evaluator or agenda change.
primary_parent: rsi-durable-improvement-workflows
additional_parents:
  - rsi-automated-research
related: []
attachments:
  - content/source_registry.md
claims: []
human_review: null
---

# Autoresearch: a bounded overnight research loop

## Editable object

**EVIDENCE — [KARPATHY-AUTORESEARCH], pinned README and `program.md` at
`228791fb499afffb54b46200aca536f79142f117`.** The agent edits one training
program while the surrounding evaluator, data, metric, and time budget remain
fixed. Repository instructions constrain the files it may change and tell it to
keep or discard experiments from the measured result.

## Loop

1. Read the current program and experiment record.
2. Propose one bounded training change.
3. Run training for the fixed time budget.
4. Read the validation metric and logs.
5. Keep improvements and revert regressions.
6. Record the result and repeat.

This loop is useful because the editable object and scalar objective are clear.
It becomes durable only when job identity, metric provenance, failures, and
keep/reject decisions survive process loss.

<details>
<summary>Original sources for this mechanism</summary>

- [Pinned upstream repository](https://github.com/karpathy/autoresearch/tree/228791fb499afffb54b46200aca536f79142f117).
- Checked-in source snapshot: `evidence/implementations/autoresearch/snapshot`.
- Registered claim ceiling: [source registry](../source_registry.md).

</details>

## Evidence and limits

The inspected source supports present-day workflow behavior at the pinned
commit. This packet does not execute the repository or reproduce a benchmark.
Humans still choose the research objective, training code boundary, dataset,
metric, and acceptance rule.

## Claim ceiling

Autoresearch is bounded workflow automation and automated experimentation. It
does not by itself demonstrate that the research agent improves its own
research procedure or builds a generally better successor.

## Reading routes

- [Workflows that persist across interruptions](../chapters/durable-improvement-workflows.md)
- [Automated research](../chapters/automated-research.md)
- [Original repository](https://github.com/karpathy/autoresearch)

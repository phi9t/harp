---
id: rsi-system-ai-scientist
kind: concept
title: AI Scientist
summary: Automated ideation, literature search, experiment-tree execution, debugging, ablation, manuscript generation, review evidence, and scientific-quality limitations.
primary_parent: rsi-automated-research
additional_parents:
  - rsi-durable-improvement-workflows
related:
  - kind: automates
    target: rsi-automated-research
attachments:
  - content/source_registry.md
claims: []
human_review: null
---

# AI Scientist: an ideation-to-paper research workflow

## Research workflow

**EVIDENCE — [AI-SCIENTIST], Methods.** The system automates idea generation,
literature novelty checks, code modification, experiment execution, debugging,
result logging, figure critique, manuscript generation, and automated review.
The published system includes template-based and more open-ended template-free
variants.

The template-free variant organizes experimentation into feasibility,
hyperparameter, main-agenda, and ablation stages. Each stage uses an agentic
tree with ordinary, debugging, replication, aggregation, and specialized
experiment nodes.

## Protected envelope

Humans still choose or constrain research domains, model access, compute
budgets, datasets, review protocol, safety policy, and whether a result is
submitted or withdrawn. Automated reviewers and VLM critics are signals, not
independent scientific authority.

<details>
<summary>Original sources for this mechanism</summary>

- End-to-end system and workflow: [AI Scientist, Methods](https://www.nature.com/articles/s41586-026-10265-5).
- Agentic experiment tree and node types: [AI Scientist, Methods and Figure 3](https://www.nature.com/articles/s41586-026-10265-5).
- Human evaluation: [AI Scientist, human evaluation results](https://www.nature.com/articles/s41586-026-10265-5).
- Limitations and ethics: [AI Scientist, Limitations](https://www.nature.com/articles/s41586-026-10265-5).
- Checked-in text: `evidence/weng/text/ai-scientist.txt`.

</details>

## Evaluation

**EVIDENCE — [AI-SCIENTIST], human evaluation.** Three generated manuscripts
were submitted to a workshop under a pre-approved protocol. One received an
average reviewer score of 6.33 and would likely have cleared the workshop
threshold; the other two did not. The authors' internal review concluded that
none met the main-conference bar.

Deeper experiment-tree budgets improve reported paper scores, which also means
resource accounting is part of any comparison.

## Failure modes and limits

The article names naive ideas, incorrect implementations, weak methodological
rigour, experimental errors, duplicated figures, and hallucinated citations.
Only computational experiments are covered. Automated paper generation also
creates review-load, disclosure, attribution, and unsafe-experiment risks.

## Claim ceiling

AI Scientist is broad `automated-ai-research`, not a demonstrated autonomous
successor-development loop. It produces research artifacts under human-set
goals and external review; it does not show that the research system improves
its own later research procedure.

## Reading routes

- [Weng: workflow design and search](../weng/05-workflow-design-and-search.md)
- [Automated research](../chapters/automated-research.md)
- [Original article](https://www.nature.com/articles/s41586-026-10265-5)

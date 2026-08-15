---
id: rsi-automated-research
kind: concept
title: Automated research as an RSI component
summary: Hypothesis generation, experiment design, implementation, measurement, negative results, research memory, and experiment selection.
primary_parent: modeling
additional_parents:
  - mlsys
related:
  - kind: executed-by
    target: rsi-durable-improvement-workflows
  - kind: evaluated-by
    target: rsi-evaluation-promotion-containment
attachments:
  - content/source_registry.md
  - content/missing_evidence.md
claims: []
human_review: null
---

# Automated research as an RSI component

## Research is a partially observable search problem

Automated AI research aims to choose a question, design an experiment, implement it, measure the result, update a technical belief, and select the next experiment. Coding and benchmark execution cover only part of that loop. The hard parts include choosing informative interventions, preserving negative evidence, identifying confounds, and deciding when a result changes the research direction.

Represent local research state as `Sᵣ,ₜ = (Kᵣ,ₜ, Yₜ, Zₜ, Mᵣ,ₜ)`. `Kᵣ,ₜ` is current technical knowledge, `Yₜ` is the hypothesis set, `Zₜ` is completed and pending experiment evidence, and `Mᵣ,ₜ` is research memory. These local symbols do not replace global `Rₜ`, which remains the candidate's retrospection policy, or protected `Aₜ`, which remains the accepted and rejected lineage archive. A research policy selects experiment `e` to maximize expected information and utility under cost:

`e* = argmax_e 𝔼[ΔU(Kᵣ,ₜ | result(e))] − λ Cost(e)`

`ΔU` can include uncertainty reduction, expected capability gain, falsification value, or downstream decision value. It must not collapse scientific validity into the same score as model performance.

## End-to-end control flow

1. Define the target question and current uncertainty.
2. Generate hypotheses that make different predictions.
3. Design the smallest experiment that distinguishes those predictions.
4. Precommit the metric, comparison, stopping rule, and invalidation conditions.
5. Implement the experiment in an isolated revision.
6. Validate data, code, environment, and measurement before the expensive run.
7. Execute under a recorded resource budget.
8. Store raw results, failures, and environment identity.
9. Analyze effect size, variance, confounds, and alternative explanations.
10. Update research memory with evidence and claim limits.
11. Select the next experiment based on the updated state.
12. Submit consequential claims or promotions to independent review.

Negative results are first-class. They shrink the hypothesis space, prevent repeated dead ends, and reveal evaluator or implementation weaknesses. A system that deletes failed experiments cannot estimate search cost or learn which research strategies fail.

<details>
<summary>Original sources for this mechanism</summary>

- AI Scientist, Methods and "Limitations," describes ideation, experiment execution, manuscript generation, automated review, and the reported quality limits: [Nature 651, 914–919](https://www.nature.com/articles/s41586-026-10265-5).
- "Why LLMs Aren't Scientists Yet," §§2–5, analyzes failure patterns across four autonomous research attempts: [arXiv:2601.03315v1](https://arxiv.org/abs/2601.03315).
- PaperBench, §§2–4 and Appendix B, defines paper-replication tasks and grading; RE-Bench, §§2–4, defines time-budgeted AI R&D comparisons: [PMLR 267, Starace et al.](https://proceedings.mlr.press/v267/starace25a.html), [PMLR 267, Wijk et al.](https://proceedings.mlr.press/v267/wijk25a.html).
- Karpathy Autoresearch, pinned `README.md` and `program.md` at commit `228791fb499afffb54b46200aca536f79142f117`, defines the narrow edit, train, measure, and keep-or-discard loop: [upstream repository](https://github.com/karpathy/autoresearch/tree/228791fb499afffb54b46200aca536f79142f117).

</details>

## Worked examples

### Autoresearch

Autoresearch narrows research to one editable training file, a fixed training time, and a scalar validation metric. This gives the loop a strong implementation and measurement boundary. It does not automate broad hypothesis selection, evaluator design, or multi-objective scientific judgment.

### AI Scientist

AI Scientist spans ideation, literature use, experiment execution, analysis, manuscript generation, and automated review. The published evaluation shows that broad workflow coverage does not imply main-conference-level scientific quality. Implementation errors, weak rigor, citation failures, and overclaiming remain part of the system behavior.

### ScientistOne

ScientistOne proposes chain-of-evidence research organization. In this packet it remains an identity-only source because the captured paper has not been inspected. It is a worked-example locator, not support for mechanism or result claims here.

### Autodata

Autodata applies an agentic research process to synthetic-data creation. It also remains identity-only in this packet. Its placement shows where data-generation research fits the loop; it does not promote uninspected claims.

### PaperBench and RE-Bench

PaperBench decomposes paper replication into many gradable tasks, which improves diagnosis but still covers replication rather than the whole research lifecycle. RE-Bench compares agents and humans under time budgets and shows that short-horizon performance does not determine long-horizon returns.

## Research memory

Research memory should store more than conclusions:

- question and hypothesis identifiers;
- expected discriminating observation;
- code, data, environment, and model revisions;
- metric definition and raw output locators;
- positive, negative, invalid, and interrupted outcomes;
- analysis and competing explanations;
- claim ceiling;
- next experiment and why it has information value.

Summaries may guide the model, but raw result references and immutable run metadata must remain available to reviewers. A result without its failed controls is not a sufficient research memory.

## Where the loop remains incomplete

Current systems often leave one or more decisions external:

- humans choose the research agenda;
- benchmarks define what matters;
- templates constrain experiment design;
- implementation correctness needs human repair;
- model judges assess novelty or writing;
- humans decide whether a result is credible or consequential;
- deployment and successor promotion stay outside the research agent.

These boundaries do not make automation unimportant. They determine the honest claim: research assistance, partial laboratory automation, or bounded AI-for-AI research rather than autonomous successor improvement.

## Failure modes and tradeoffs

- **Hypothesis laundering.** A vague idea becomes a post hoc story after seeing the result.
- **Implementation drift.** Code no longer tests the stated hypothesis.
- **Metric fixation.** The loop optimizes one benchmark rather than the research question.
- **Noise chasing.** It selects the best random seed without replication.
- **Negative-result loss.** Failed branches disappear, causing repeated work and selection bias.
- **Literature hallucination.** Nonexistent or misread sources enter the argument.
- **Research-memory contamination.** Conclusions lose their experimental conditions and limits.
- **Judge circularity.** The same model family proposes, writes, and validates the claim.
- **Long-horizon decay.** Context compaction drops the original question or control.

Narrow research loops are easier to automate and verify. Broad loops can discover more consequential changes but require stronger evidence, domain judgment, and independent review.

## What would weaken the mechanism

An automated-research claim weakens if the implementation does not test the proposed hypothesis, if independent reruns fail, if a simple baseline or random search matches experiment selection, or if hidden human repair supplies the decisive step. An RSI interpretation further weakens if the research system produces a better artifact but does not improve later research cycles.

## Open technical questions

- How should an agent estimate the information value of an experiment before running it?
- Can research memory preserve uncertainty rather than hardening every note into a fact?
- Which parts of scientific taste admit executable evaluation?
- How should independent reviewers be selected when model families share training data and biases?

<details>
<summary>Reference records and operational metadata</summary>

- Exact source status and inspected locators are in [[knowledge/rsi/source_registry|the source registry]].
- ScientistOne and Autodata are explicitly identity-only until their captures are inspected.
- The [[knowledge/rsi/missing_evidence|missing-evidence ledger]] tracks absent end-to-end and multi-generation reproductions.

</details>

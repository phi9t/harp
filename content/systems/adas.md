---
id: rsi-system-adas
kind: concept
title: Automated Design of Agentic Systems
summary: Meta-agent search over code-represented agent designs, an ever-growing archive, external task evaluation, transfer evidence, and bounded claim limits.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-evaluation-promotion-containment
related:
  - kind: compared-with
    target: rsi-system-aflow
attachments:
  - content/source_registry.md
  - content/diagnostics/cases/adas.json
claims: []
human_review: null
---

# ADAS: meta-agent search over agent programs

## Problem and RSI relevance

**EVIDENCE — [ADAS], §§2–3.** Automated Design of Agentic Systems treats the
complete agent design as a search object. The paper separates search space,
search algorithm, and evaluation function, then demonstrates one algorithm,
Meta Agent Search, over agent code.

**INFERENCE.** ADAS improves harness state `H_t`, not model weights `W_t`. It is
relevant to RSI because a discovered agent program may later propose another
program, but the paper evaluates task-solving agents rather than matched
parent-versus-child successor production.

## Editable object and protected envelope

The editable object is executable agent code assembled from foundation-model
calls, prompts, control flow, and aggregation logic. The protected envelope
contains the foundation models, task data, evaluator, iteration budget, and
archive-writing policy.

Code broadens the search space beyond prompt strings, but "all possible
designs" remains bounded by the supplied runtime, available model calls,
language, and evaluator.

## Algorithm

**EVIDENCE — [ADAS], §3 and Figure 1.**

1. Initialize an archive, optionally with hand-designed agent baselines.
2. Give the meta-agent the archive and task description.
3. Ask it to program one new agent while avoiding duplicate designs.
4. Execute the candidate on development tasks.
5. Store its code, design description, and metrics in the archive.
6. Repeat until the fixed iteration limit.

The archive is both memory and proposal context. It records stepping stones,
but it is not a quality-diversity proof that every useful behavioral region is
preserved.

<details>
<summary>Original sources for this mechanism</summary>

- Search-space formulation: [ADAS, §2](https://proceedings.iclr.cc/paper_files/paper/2025/file/36b7acf6f6010652b3f2a433774a66fe-Paper-Conference.pdf).
- Meta Agent Search: [ADAS, §3 and Figure 1](https://proceedings.iclr.cc/paper_files/paper/2025/file/36b7acf6f6010652b3f2a433774a66fe-Paper-Conference.pdf).
- Evaluation and transfer: [ADAS, §§4.1–4.3](https://proceedings.iclr.cc/paper_files/paper/2025/file/36b7acf6f6010652b3f2a433774a66fe-Paper-Conference.pdf).
- Checked-in text: `evidence/weng/text/adas.txt`.

</details>

## Evaluation

**EVIDENCE — [ADAS], §4.** The paper evaluates ARC, reading comprehension,
mathematics, science questions, and multi-task problem solving. It reports
progressive search improvements and gains over the compared hand-designed
baselines, including 25.9% and 13.2% accuracy improvements in two named
reasoning domains and transfer across domains and models.

These are author-reported comparisons under the paper's model, archive seeds,
prompts, task subsets, and budgets. They do not isolate code-space
representation from the meta-agent model or extra search compute.

## Failure modes and limits

- Archive context can bias the meta-agent toward earlier motifs.
- Finite task evaluation can select benchmark-specific control flow.
- Candidate code creates execution and containment risk.
- More candidates increase winner probability even if proposal quality is
  unchanged.
- External foundation models and task metrics remain fixed.

The paper's discussion also acknowledges that powerful automated design can
create destructive or misaligned agents. Runtime permissions and evaluator
ownership therefore belong outside candidate write control.

## Claim ceiling

**EVIDENCE.** The source supports Meta Agent Search as an automated
harness-program search method with author-reported task and transfer gains.

**MISSING.** It does not independently reproduce the result or measure whether
an accepted agent becomes better at producing later accepted agents under a
matched envelope. The Atlas therefore classifies ADAS as
`harness-improvement`, not successor or recursive improvement.

## Reading routes

- [Weng: workflow design and search](../weng/05-workflow-design-and-search.md)
- [ADAS versus AFlow lesson](../lessons/03-adas-vs-aflow.md)
- [Searching for better harnesses](../chapters/harness-search.md)
- [Original paper](https://proceedings.iclr.cc/paper_files/paper/2025/file/36b7acf6f6010652b3f2a433774a66fe-Paper-Conference.pdf)

---
id: dgm-hyperagents-successor
title: DGM-Hyperagents successor case study
type: deep-dive
status: active
created: 2026-08-09
updated: 2026-08-09
tags: [darwin-godel-machine, hyperagents, metacognitive-self-modification, successor-improvement]
confidence: high
canonical: ../rsi/systems/dgm.md
---

# DGM-Hyperagents: editable task and meta logic

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Why this is a direct DGM successor

Original DGM evolves a coding-agent program while its diagnostic,
instruction-generation mechanism remains external and fixed. The fixed
diagnostic layer creates an alignment assumption: a system that gets better at
coding must also happen to get better at proposing the next agent change.

**[EVIDENCE - DGM-077](claim_evidence_crosswalk.md#dgm-077-dgm-h-makes-task-and-meta-logic-one-editable-program).**
Hyperagents changes the mutable object. It places the task agent and the meta
agent—the logic that generates later modifications—inside one editable program.
The authors call edits to that meta logic *metacognitive self-modification*.

The resulting decomposition is:

$$
\text{DGM-H}
=
\underbrace{\text{protected exploration envelope}}_{\text{archive, selection, evaluation}}
+
\underbrace{\text{evolving hyperagent}}_{\text{task logic + meta logic}}
+
\underbrace{\text{frozen foundation model}}_{\text{inference engine}}.
$$

This is a material improvement over DGM's fixed instruction generator, but it
is not an edit to the whole running experiment.

## Outer loop and inner loop

The DGM-H outer loop retains DGM's archive-based exploration:

1. select a parent from the archive;
2. let that parent generate a modified hyperagent;
3. evaluate the child;
4. add the child to the archive; and
5. repeat under a fixed experiment budget.

Its inner loop is self-improving in two distinct ways:

| Inner-loop target | What the hyperagent can change | Why it matters |
|---|---|---|
| Task agent | Task-solving prompts, tools, memory, control flow, and other program logic | It can improve the current task behavior. |
| Meta agent | The procedures that inspect the repository, use past evaluations, choose edits, and construct later hyperagents | It can alter how future improvements are generated. |

**[EVIDENCE - DGM-078](claim_evidence_crosswalk.md#dgm-078-main-dgm-h-keeps-key-outer-loop-controls-fixed).**
The paper explicitly says the main-text experiments keep parent selection
handcrafted and non-modifiable to isolate hyperagent self-modification. It
reports only preliminary exploration of modifying parent selection in Appendix
E.5. The task distribution, evaluator, experiment budget, and outer execution
control therefore remain protected boundaries in the reported main setting.

## What the paper reports

**[SOURCE CLAIM - DGM-079](claim_evidence_crosswalk.md#dgm-079-dgm-h-reports-cross-domain-transfer-of-improvement-procedure).**
Across five-run experiments, the authors report that hyperagents evolved for
paper review or robotics reward design generate stronger math-grading agents
than an initial hyperagent under their transfer protocol. They attribute the
result to transferable mechanisms such as performance tracking and persistent
memory. They also report continued runs from transferred hyperagents, but say
the final DGM-H-plus-transfer versus fresh-DGM-H difference is not statistically
significant.

These are author-reported task and transfer outcomes from
arXiv `2603.19461v1`; Harp has not rerun the experiments or inspected a pinned
implementation snapshot.

## What this does not establish

DGM-H is a partial answer to the fixed-meta-agent limitation, not a discharge
of the packet's successor-improvement criterion.

The transfer protocol is stronger evidence than DGM's task-score proxy because
it freezes a transferred hyperagent and measures generated agents in a new
domain. But it is not yet the exact matched causal test requested by
[DGM-050](claim_evidence_crosswalk.md#dgm-050-matched-successor-improvement-evidence-is-missing):

- a named accepted parent and child must receive the same proposal protocol;
- the evaluator, selection rule, task distribution, permissions, model access,
  and root-tree budget must remain fixed;
- both must be evaluated as producers of later valid children; and
- the estimate must compare the child’s and parent’s next-cycle productivity,
  rather than only their selected descendants or transferred-run outcomes.

The source itself also names the residual boundary: its conclusion says parent
selection and evaluation protocols remain fixed, improving stability and safety
while limiting full self-modifiability.

## Reading and implementation next step

Read [the Hyperagents source record](source_registry.md#hyperagents-dgm-h-successor-paper)
with [the successor experiment design](10_successor_design.md). A stronger
audit would then pin the Hyperagents implementation and reconstruct its
transfer and parent-selection protocols before making any stronger causal
claim.

Back to the [DGM index](darwin_godel_machine_index.md).

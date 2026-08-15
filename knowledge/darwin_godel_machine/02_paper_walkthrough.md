---
id: dgm-paper-walkthrough
title: DGM paper walkthrough
type: deep-dive
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, paper-reading, research]
confidence: high
canonical: ../rsi/systems/dgm.md
---

# DGM paper walkthrough

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Learning outcomes

After this chapter, you should be able to:

- reconstruct the paper's argument without rereading every paragraph;
- find the appendix sections that carry algorithm and evaluation details;
- separate headline claims from experimental scope; and
- identify the paper's central assumption.

Primary source: [[evidence/weng/text/dgm.txt|captured DGM paper text]],
arXiv `2505.22954v3`.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-071: Captured paper identifies ICLR 2026 publication|EVIDENCE - DGM-071]].**
The captured v3 text identifies the paper as published at ICLR 2026.

## Argument map

The paper's causal argument is:

1. AI development still depends on human-designed agent systems.
2. Coding-agent designs can be represented as editable code.
3. A coding agent can edit that code, including code used in later edits.
4. Formal proof of useful rewrites is impractical.
5. Benchmarks can provide empirical selection pressure.
6. A branching archive can preserve stepping stones that a linear optimizer
   would discard.
7. Better benchmark-solving agents should be better self-editors because
   self-editing is a coding task.
8. Reported benchmark gains, ablations, and transfer support the value of
   self-modification and archive search.

Steps 1 through 6 define the system. Step 7 is the key proxy assumption. Step 8
tests downstream performance and components, not the proxy assumption
directly.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-050: Matched successor-improvement evidence is missing|MISSING - DGM-050]].**
No section reports the matched parent-versus-child next-cycle comparison that
would directly test step 7.

## Abstract

The abstract establishes four headline claims:

- DGM iteratively modifies its own code;
- it maintains an archive for open-ended exploration;
- reported performance rises from 20.0% to 50.0% on SWE-bench and from 14.2%
  to 30.7% on full Polyglot; and
- self-improvement and open-ended exploration outperform ablated systems.

Read the numbers carefully. The 30.7% Polyglot result is full-benchmark
evaluation. The search-time 50-task subset result is 38.0%.

Question to ask: Which part of the abstract is a measured result, and which
part is an interpretation about future self-improvement ability?

## Section 1: introduction

The introduction contrasts fixed human-designed AI systems with cumulative
scientific progress. It positions DGM as a practical relaxation of the Gödel
Machine idea.

The important scope limit appears early: the reported system improves coding
agent design around frozen foundation models. It does not retrain the
foundation model.

The introduction also states the central assumption:

> Better coding-benchmark performance indicates better coding ability and
> therefore better ability to perform self-modification.

Question to ask: What matched experiment would be needed to test "therefore"?

## Section 2: related work

The paper places DGM at the intersection of three areas.

### Open-endedness

The archive is meant to preserve interesting stepping stones and avoid local
optima. DGM differs from systems that generate diverse artifacts without
closing a self-referential agent-improvement loop.

### Meta-learning foundation-model agents

Methods such as ADAS use a fixed meta-agent to generate downstream agent
designs. DGM's distinction is that a selected agent implementation participates
in modifying the implementation used in later generations.

### Self-improving AI

The paper contrasts DGM with proof-driven Gödel Machines, weight-space
self-improvement, self-play, and related recursive code-generation systems.
Its claimed differentiator is open-ended archive search combined with
self-referential agent code.

Question to ask: Is the external diagnostic model functionally a fixed
meta-agent even if the coding agent implementation evolves?

## Section 3: Darwin Gödel Machine

This is the conceptual core.

### Self-referential agent code

The coding agent is a repository powered by frozen models. Prompts, workflows,
tools, and adapters are code and can be edited.

### Population-based search

Each iteration selects parents from the archive. Selection favors performance
and underexplored lineages. A parent receives an improvement task, edits itself,
and produces a child.

### Viability rather than strict improvement

The paper says children must compile and retain code-editing ability. It does
not require every admitted child to outperform its parent. That choice enables
stepping stones.

### Fixed open-ended controller

The archive policy and parent selection are fixed in the reported system. The
paper explicitly leaves self-modification of this outer process to future work.

Question to ask: Which state is recursive, and which state remains protected
and fixed?

## Section 4.1: experiment setup

The seed agent has two tools:

- Bash command execution;
- whole-file/directory editing and viewing.

The system runs for 80 iterations. The paper reports two parallel
self-modification attempts for SWE-bench and four for Polyglot. Claude 3.5
Sonnet performs self-modification in both experiments. Evaluation uses Claude
3.5 Sonnet on SWE-bench and `o3-mini` on Polyglot. Appendix D.1 reports
temperature 1.0.

The seed is intentionally weak. That makes improvement easy to observe, but it
also means some gains repair obvious missing agent features.

Question to ask: How much of the result is open-ended discovery versus
automatic completion of a deliberately minimal baseline?

## Section 4.2: benchmarks and staged evaluation

SWE-bench Verified contains Python repository issues that may require
multi-file changes. Polyglot covers several languages and generally smaller
single-file solution tasks.

The staged process is:

1. evaluate 10 tasks for basic editing;
2. evaluate an additional 50 tasks for agents that pass;
3. on SWE-bench, evaluate 140 more tasks when the 60-task score exceeds 40%
   and the agent ranks among the archive's top two at that point;
4. report selected agents on 200 SWE-bench tasks;
5. report the final Polyglot candidate on the full benchmark.

Polyglot uses pass@1, unlike the leaderboard's pass@2 setup.

Question to ask: How does adaptive allocation of evaluation tasks affect score
comparability and winner selection?

## Section 4.3: baselines

### Without self-improving agents

A fixed base agent modifies candidate agents. This tests whether updating the
modifier matters.

### Without open-ended exploration

Only the latest viable agent remains. This tests whether archive branching and
older stepping stones matter.

### Handcrafted systems

The paper compares against a checked open-source SWE-bench system and Aider on
Polyglot. These comparisons show practical competitiveness, but search compute,
agent workflow, and metric setup still need careful accounting.

Question to ask: Which baseline isolates recursive modifier improvement, and
which baseline isolates archive search?

## Section 4.4: results

The headline reported results are:

| Setting | Base | Best DGM | Scope |
|---|---:|---:|---|
| SWE-bench | 20.0% | 50.0% | 200-task paper subset |
| Polyglot search subset | 14.0% | 38.0% | 50 tasks |
| Polyglot full benchmark | 14.2% | 30.7% | Full benchmark |

## Discovered mechanisms

The reported changes are recognizable coding-agent design patterns. That is
scientifically useful: DGM searches over mechanisms that human harness
engineers also treat as first-class design choices.

### Fine-grained inspection and editing

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-072: Evolved agents add fine-grained file inspection and editing|SOURCE CLAIM - DGM-072]].**
The paper reports line-range viewing and exact string replacement instead of
whole-file-only inspection and overwrite. Appendix G compares two lineages
targeting fine-grained editing: one scores 23.3% and another 40.5%. These are
author-reported selected-run outcomes, not an independent ablation, but they
show that the implementation of a tool matters beyond its feature label.

### Context-length management

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-073: Evolved agents summarize history at the context limit|SOURCE CLAIM - DGM-073]].**
One evolved patch detects context-length errors, summarizes older user and
assistant messages, retains recent messages, and retries with the compressed
history. This makes context policy part of the evolved program rather than
external bookkeeping. The mechanism has not been independently reproduced
here.

Conceptually:

$$
\text{effective capability}
=
f(\text{base model},
\text{visible evidence},
\text{compression policy},
\text{tool trace},
\text{retry state}).
$$

### Multiple independent attempts

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-074: Evolved agents generate multiple candidate solutions|SOURCE CLAIM - DGM-074]].**
The paper reports an evolved workflow that resets to the base commit, generates
several candidate patches, runs regression tests, and stores the valid patches
and reports for later selection.

$$
x_1,\ldots,x_k \sim \pi_g(\cdot \mid \text{task})
$$

The added search breadth also adds model and test cost, which the paper does not
isolate from the mechanism's benefit.

### Model-based candidate judging

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-075: Evolved agents use model-based candidate judging|SOURCE CLAIM - DGM-075]].**
Another evolved workflow scores candidates with test reports and uses a further
foundation-model call to compare solutions or break ties. This is a small
proposer-and-judge architecture, not independent formal verification, and it
has not been reproduced here.

### Previous-attempt reuse

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-076: Evolved agents reuse previous-attempt evidence|SOURCE CLAIM - DGM-076]].**
The paper reports history-aware workflows that carry prior patches and test
outcomes into later attempts, asking the next attempt to repair observed
limitations or explore another approach. This is within-task episodic reuse,
not persistent learned memory.

Together these mechanisms span:

$$
\text{harness}
=
\text{observation policy}
+
\text{context policy}
+
\text{tool interface}
+
\text{search policy}
+
\text{verification policy}
+
\text{memory policy}.
$$

DGM's contribution is to make this program a search object rather than leaving
every dimension fixed by a human harness designer.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-026: Paper reports a higher functioning-child rate|SOURCE CLAIM - DGM-026]].**
Appendix A.4 reports a 51.3% functioning-child rate for DGM versus 32.5% for
each main baseline. These process measurements have not been independently
reproduced here.

Figure 3 shows two temporary score dips in the final best lineage. That is the
paper's strongest visual evidence for retaining non-monotone stepping stones.

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-034: Paper reports two score dips in the best lineage|SOURCE CLAIM - DGM-034]].**
The paper interprets those dips as evidence for stepping-stone search. The
lineage alone does not establish that either intermediate change causally
enabled the final descendant.

The transfer experiments test different models, benchmarks, and programming
languages. They reduce the likelihood of narrow overfitting, but they do not
measure future child-production ability.

Question to ask: Does transfer support general agent improvement, recursive
improvement, or both?

## Section 5: safety discussion

The paper identifies benchmark misspecification, vulnerability introduction,
misalignment amplification, and reduced interpretability as risks.

Reported controls include:

- isolated execution;
- time limits;
- restricted modification scope;
- monitoring;
- traceable lineage; and
- no deployment in real development environments.

The section also proposes optimizing for safety and interpretability, while
acknowledging that a system may learn to remove or bypass such constraints.

Question to ask: Which controls are outside candidate write authority?

## Section 6: conclusion and limitations

The paper states several limits:

- closed-source SWE-bench systems still perform better;
- one SWE-bench DGM run takes about two weeks;
- API cost is high;
- progress depends on foundation-model reasoning;
- the system does not update model weights;
- the domain is limited to coding; and
- coding-benchmark performance is only a proxy for self-improvement ability.

The proposed future directions include model training, broader domains,
co-evolved task distributions, and editable open-ended search policy.

Question to ask: Which future direction would most change the safety and
evaluation problem?

## Load-bearing appendices

### Appendix A

Contains baseline trees, model transfer, parent-selection ablation, valid-agent
rates, and three-run Polyglot stability.

Important reported values:

- DGM Greedy: 39.7% SWE-bench, 30.0% Polyglot subset;
- valid editing agents: DGM 51.3%, both main baselines 32.5%;
- three Polyglot runs: mean 40.7%, standard deviation 2.3% on the reported
  search setup.

### Appendix C

Defines the seed agent, parent-selection equations, diagnostic prompts, and
pseudocode. This appendix is essential for understanding what "self-modify"
means operationally.

### Appendix D

Records model assignments and temperature.

### Appendix E

Records task subsets, costs, and handcrafted comparison setup. The paper
estimates one SWE-bench DGM run at USD 22,000 and either main baseline at USD
10,000. It estimates one 60-task SWE-bench evaluation at USD 350 and one
60-task Polyglot evaluation at USD 5.

### Appendices F and G

Show the actual modification lineages and compare two implementations of
fine-grained editing. Nodes targeting similar functionality score 23.3% and
40.5%, supporting the claim that implementation details matter.

### Appendix H

**[[knowledge/darwin_godel_machine/claim_evidence_crosswalk#DGM-044: Paper reports objective hacking in Appendix H|SOURCE CLAIM - DGM-044]].**
Shows the tool-hallucination optimization and objective-hacking case. A node
gets a perfect detector score by removing the logging markers used by the
detector.

### Appendix J

Proposes editing the outer exploration process, changing human involvement,
and broadening the system. These are proposals, not demonstrated capabilities.

## Reading checklist

When reading any DGM claim, ask:

1. Is this a paper claim, code behavior, or packet inference?
2. Which agent component changes?
3. Which external component defines success?
4. Which task subset produced the score?
5. Was the result selected adaptively?
6. Does the evidence measure task performance or future improvement
   production?
7. Could the candidate influence the measurement channel?

Continue with [[knowledge/darwin_godel_machine/03_algorithm_derivation|algorithm derivation]].

Back to the [[knowledge/darwin_godel_machine/darwin_godel_machine_index|DGM index]].

---
id: verified-coevolution
title: Beyond self-training: recursive closure, model-harness coevolution, and assurance of self-improving AI.
type: research-agenda
mode: FALSIFIABLE AGENDA
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [recursive-self-improvement, coevolution, assurance, test-time-adaptation, verification]
confidence: medium
---

# Beyond self-training: recursive closure, model-harness coevolution, and assurance of self-improving AI.

Mode: `FALSIFIABLE AGENDA`.

This packet publishes a research agenda, not a claim that general
model-harness coevolution has already been demonstrated. Model-harness
coevolution is a research hypothesis in this packet. Recursive
self-improvement is treated as **recursive closure over an improvement
process**: a change counts only when an accepted update improves some part of
the later improvement process itself. See
[VCA-001](claim_evidence_ledger.md#vca-001-recursive-closure-definition).

The supplied planning agenda is preserved at
`evidence/verified_coevolution_agenda/artifacts/supplied_research_agenda.txt`.
It shaped this report, but it is not source evidence for paper claims. Source
ceilings and missing full-text boundaries are tracked in the
[claim evidence ledger](claim_evidence_ledger.md).

## Recursive closure

Let the system state be:

```text
X_t = (theta_t, H_t, M_t, D_t, V_t, G_t)
```

where `theta_t` is model parameters, `H_t` is the harness and tool-control
surface, `M_t` is persistent memory and learned skills, `D_t` is curriculum and
experience, `V_t` is evaluators or verifiers, and `G_t` is governance,
permissions, invariants, and promotion policy. A proposal operator `I_t`
generates a candidate update from `X_t`, trajectories, and an archive. A
promotion gate `G_t` accepts, rejects, or partially merges the candidate. See
[VCA-001](claim_evidence_ledger.md#vca-001-recursive-closure-definition).

An update is recursively self-improving only when the accepted change improves
part of a future improvement process: the proposer, evaluator, experience
representation, search policy, editable scope, or promotion rule. A higher task
score alone is not sufficient. A self-training round that improves the current
answer but leaves the later improvement operator unchanged is adaptation, not
recursive closure.

The levels below are a classification aid, not a maturity ladder. They are
useful because adjacent systems often differ along one axis rather than
belonging to clean architectural families. See
[VCA-002](claim_evidence_ledger.md#vca-002-classification-levels).

| Level | Persistent change | Later consumer | Honest claim |
|---:|---|---|---|
| 0 | More sampling or refinement inside one inference episode | Current task only | Output refinement. |
| 1 | Episodic memory, scratchpad, or local experience update | Later task episode | Persistent adaptation. |
| 2 | Prompt, skill, context policy, or retrieval rule | Later agent run | Procedure or scaffold tuning. |
| 3 | Harness control flow, tool code, agent source, or archive policy | Later harness search | Harness self-modification. |
| 4 | Model weights or adapters | Later model behavior | Weight-level self-training or test-time learning. |
| 5 | Coupled model and harness updates under a shared accounting boundary | Later model-harness search | Model-harness coevolution hypothesis. |
| 6 | Proposer, evaluator, verifier, search policy, editable scope, or governance rule | Later improvement process | Recursive closure over the improvement operator. |

## Three-axis field map

The agenda maps systems by editable substrate, update mechanism, and acceptance
evidence. The point is to separate what changes, how it changes, and what
authorizes the change. See
[VCA-003](claim_evidence_ledger.md#vca-003-three-axis-field-map).

| Work | Editable substrate | Update mechanism | Acceptance evidence | Claim ceiling |
|---|---|---|---|---|
| STOP | Improver program around a frozen language model | Recursive code-generation optimization | Held-out code-generation score and sandboxed execution | Source-backed protocol and limitation; not full RSI. |
| Meta-Harness | Harness code, filesystem state, prior traces | Search over harness programs | Scores, source, traces, and held-out evaluation | Existing Harp evidence supports fixed-model harness search, not coevolved weights. |
| AHE | Harness components and observability surfaces | Observability-driven edit proposal | Component, experience, and decision observability plus predicted edit effects | Harness-evolution evidence; not broad model adaptation. |
| ADAS | Code-defined agentic systems | Meta-agent search | Benchmark score and transfer tests | Agent-design search; not a verifier-governed coevolution program. |
| DGM | Self-modifying coding-agent implementation and archive | Archive-based open-ended evolution | SWE-bench and Polyglot author-reported scores | Self-modifying scaffold evolution with frozen foundation model. |
| Godel Agent | Agent logic and self-reference policy | Objective-directed self-modification | arXiv metadata and abstract-page identity only in this packet | Metadata-only local evidence; full text not vendored. |
| Mendel Godel Machine | Coding-agent lineages and recombination | Comparative evolution and cross-lineage hybridization | arXiv metadata and abstract-page identity only in this packet | Metadata-only local evidence; very recent preprint identity only. |
| LADDER | Curriculum plus model weights | Recursive problem decomposition and verifier-guided GRPO | ArXiv v3 full text, PDF, and extracted text | Author-reported mechanism and results; no reproduction. |
| LADDER-TTRL | Per-problem model weights | Problem-variant generation plus test-time RL | Numerical or rule-based verifier | Distinct test-time adaptation mechanism; rollback matters. |
| PRIME-RL TTRL | Test-distribution model weights | Repeated samples on unlabeled inputs plus RL | Majority-vote pseudo-label reward | Distinct TTRL mechanism; consensus can amplify systematic error. |
| NSRSA | Reasoning traces and recursive self-training data | Verified reasoning filters across iterations | Answer correctness, parseable arithmetic, variable consistency, domain constraints | Reasoning-data quality control, not broad alignment. |
| SAHOO | Acceptance policy and drift monitor | Learned Goal Drift Index plus constraints and regression risk | Empirical monitor and multi-objective acceptance control | Alarm and control layer, not proof of stable goals. |
| Scrivens verification | Parameter mutations under formal model | Statistical classification gate versus property verifier | Sequential-model assumptions and certified parameter regions | Conditional theory over encoded properties and explicit domains. |

LADDER-TTRL and PRIME-RL TTRL stay in separate rows. LADDER-TTRL generates a
tree of problem variants for a target problem, runs problem-specific GRPO, then
rolls back. PRIME-RL TTRL builds rewards from repeated samples and majority
vote on unlabeled test data. They fail differently: LADDER-TTRL depends on
valid difficulty gradients and reliable problem verification; PRIME-RL TTRL
depends on consensus being informative. See
[VCA-006](claim_evidence_ledger.md#vca-006-ttrl-distinction).

## Coevolution as hypothesis

The central hypothesis is not that history has already shown a law of inward
and outward migration. It is that the next useful RSI program should measure
whether capability moves between model weights and harness structure under
controlled, audited update loops. See
[VCA-007](claim_evidence_ledger.md#vca-007-coevolution-hypothesis).

Capability internalization is the hypothesis that successful search traces,
skills, or recovery patterns can be distilled into `theta_t` so the model needs
fewer later search nodes. Externalization is the hypothesis that state,
permissions, tools, episodic knowledge, auditing, and rollback mechanisms can
move into `H_t`, `M_t`, `V_t`, and `G_t` so the model can spend capacity on
harder reasoning rather than operational bookkeeping. See
[VCA-008](claim_evidence_ledger.md#vca-008-search-to-skill-hypothesis).

The falsifiable question is not only whether accuracy rises. It is whether the
next improvement round becomes cheaper, more transferable, more auditable, and
more productive after controlling for the full root-tree resources spent on
search, training, verification, rejected branches, and rollback.

## Recursive dynamics: epistemic drift, behavioral regression, and objective instability

Improvement does not automatically compound. Recursive loops can magnify
errors, narrow distributions, forget old capabilities, corrupt their own
evaluation signal, or collapse search diversity. The agenda separates six
failure modes. See
[VCA-011](claim_evidence_ledger.md#vca-011-drift-and-regression-boundaries).

| Failure mode | Meaning | Evidence role |
|---|---|---|
| Epistemic drift | Incorrect beliefs or reasoning traces propagate into later data. | NSRSA is a narrow verifier-gated intervention for arithmetic reasoning-data quality. |
| Distribution collapse | Generated data loses support from the original distribution. | The Nature model-collapse article anchors this risk for recursive generated-data training. |
| Capability regression | Previously reliable skills are forgotten or become inaccessible. | Delayed holdouts and rollback targets must be reserved. |
| Objective drift | The optimized proxy or behavioral target changes across generations. | SAHOO is relevant as monitoring and acceptance control, not proof of invariant goals. |
| Evaluator drift | The gate, verifier, or monitor becomes unreliable or manipulable. | Freeze evaluator identity and promotion authority during experiments. |
| Search collapse | Archive or lineage diversity disappears too early. | DGM and Mendel Godel Machine motivate archive and comparative-evolution experiments. |

NSRSA belongs under epistemic stability and training-data quality. It does not
establish corrigibility, honesty, non-deception, tool-use policy, or
long-horizon goal stability. SAHOO belongs under empirical monitoring and
acceptance control. It should raise an alarm or block a candidate; it does not
prove the goal stayed semantically invariant. See
[VCA-009](claim_evidence_ledger.md#vca-009-nsrsa-boundary) and
[VCA-010](claim_evidence_ledger.md#vca-010-sahoo-boundary).

Scrivens belongs under conditional theory for assurance. Its sequential-model
claims depend on assumptions about parameter mutations, distribution overlap,
regularity, cumulative risk, and the gate model. Certificates apply to encoded
properties over explicit domains. Zero-false-acceptance claims are not the same
as semantic alignment. See
[VCA-013](claim_evidence_ledger.md#vca-013-scrivens-boundary).

## Falsifiable research program

The companion [experiment protocol](experiment_protocol.md) pre-registers five
research thrusts. They share one rule: freeze evaluator identity and promotion
authority, count the complete root tree of resources, archive rejected branches,
reserve delayed holdouts, and record rollback targets before comparing
lineages.

1. Compare harness-only, weights-only, alternating, and joint adaptation under
   equal compute. The hypothesis is that alternating or joint updates can beat
   the better single-surface baseline on delayed holdouts.
2. Compare search-to-skill distillation from raw traces, stripped traces,
   reusable skills, and failure-recovery pairs.
3. Run 20 to 50 generations and report held-out gain, worst-case regression,
   calibration, evaluator disagreement, diversity, and cumulative risk.
4. Compare a single lineage, archive retention, and comparative or recombined
   evolution, including descendant productivity rather than endpoint score
   only.
5. Compare adaptive-proposer gates: classifier, ensemble, regression suite,
   executable or symbolic verifier, certified region, and hybrid gate.

This turns model-harness coevolution into a measured program rather than a
story about inevitable compounding.

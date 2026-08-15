---
id: rsi-recursive-improvement-loop
kind: concept
title: What makes an improvement loop recursive
summary: A state-transition test that separates task refinement, persistent adaptation, harness improvement, automated research, and successor improvement.
primary_parent: modeling
additional_parents:
  - mlsys
related:
  - kind: implemented-by
    target: rsi-harness-engineering
  - kind: governed-by
    target: rsi-evaluation-promotion-containment
attachments:
  - content/source_registry.md
  - content/bounded_transitive_closure.md
  - content/missing_evidence.md
claims: []
human_review: null
---

# What makes an improvement loop recursive

## The technical problem

An optimization loop is recursive only when an accepted change alters the process that produces later accepted changes. Repeating a prompt, sampling more answers, or running another search iteration may improve a task result without changing the improver. The distinction matters because task score, persistence, and recursive gain require different experiments.

Use five levels:

| Level | Persistent change | Next consumer | Honest claim |
|---|---|---|---|
| Task improvement | Current answer or action only | Current task | The system refined an output. |
| Persistent adaptation | Memory, skill, prompt, data, or policy | Later task episodes | The system retained a useful procedure. |
| Harness improvement | Agent code, tools, context policy, or workflow | Later agent runs | The deployed agent process changed. |
| Automated AI research | Hypotheses, experiment code, measurements, or training recipes | Later research cycles | Part of AI research became automated. |
| Successor improvement | The candidate and its ability to build the next candidate | Later accepted generations | The improvement process itself improved. |

The system boundary is the editable candidate `Cₜ = (Wₜ, Hₜ, Dₜ, Rₜ)`. `Wₜ` is model weights, `Hₜ` is the harness, `Dₜ` is persistent learned material, and `Rₜ` is the policy for inspecting traces and proposing changes. The evaluator, held-out signals, budget, permissions, protected archive, and promotion authority stay outside candidate write control.

## State transition and recursion test

Let `Π(Cₜ, B, P)` be a fixed proposal protocol that asks candidate `Cₜ` to produce possible changes under budget `B` and permissions `P`. Let `Valid_E(x)` mean that candidate `x` passes evaluator `E`, including outcome and integrity gates. Define:

`Q(Cₜ) = E[max Score_E(x) | x ∈ Π(Cₜ, B, P), Valid_E(x)]`

`Q(Cₜ)` measures the quality of later valid candidates produced by `Cₜ`, not the immediate task score of `Cₜ`. A bounded recursive gain is:

`RGₜ = Q(Cₜ₊₁) − Q(Cₜ)`

Every symbol depends on a fixed experiment contract. `Π`, `E`, `B`, `P`, task distribution, model access, and selection rule must match across parent and child. Otherwise a larger budget or weaker evaluator can masquerade as recursive gain.

The operational flow is:

1. `Cₜ` diagnoses traces and proposes a bounded delta.
2. An external runner applies the delta in an isolated candidate.
3. Protected tasks and runtime monitors produce observations.
4. The evaluator scores outcome, integrity, cost, and successor usability.
5. External promotion authority accepts or rejects the candidate.
6. The protected archive records every branch and creates a parent-child edge only for an accepted candidate.
7. The same proposal protocol runs from both parent and child to estimate `RGₜ`.

Search rounds inside step 1 are not generations. Retries inside step 3 are not generations. Generation `t+1` starts only after step 6 records an accepted parent-child transition.

<details>
<summary>Original sources for this mechanism</summary>

- STOP, Abstract, §2, and §6 "Limitations," defines recursive scaffolding optimization with fixed model weights and explicitly limits the full-RSI claim: [arXiv:2310.02304v3](https://arxiv.org/abs/2310.02304).
- Darwin Gödel Machine, §§1–3 and Figure 2, defines empirically selected coding-agent lineage and an archive of variants: [arXiv:2505.22954v3](https://arxiv.org/abs/2505.22954).
- Schmidhuber, "Gödel Machines," publisher abstract and chapter summary, supplies the contrasting proof-certified self-rewrite formulation: [DOI:10.1007/978-3-540-68677-4_7](https://doi.org/10.1007/978-3-540-68677-4_7). The packet contains only the publisher summary, so it does not support details beyond that summary.
- [[knowledge/rsi/concepts/system-state-and-notation|System state and notation]] defines the candidate and protected-envelope symbols used here.

</details>

## Worked examples and claim boundaries

### STOP

STOP improves a scaffolding program that asks a fixed language model to improve downstream programs. Its recursive object is the improver program. The model weights, task distribution, and utility function remain external. The result supports bounded recursive scaffolding improvement, not general successor construction.

### Darwin Godel Machine

DGM mutates coding-agent repositories, evaluates them on externally chosen benchmarks, and keeps a branching archive. A child can change code used to create later children, so the mechanism reaches the recursive boundary. Its claim still stops at fixed foundation-model weights, human-set tasks, sandboxing, and external selection.

### Recursive language models

An RLM recursively calls a language model over selected context. This is recursive inference. Training a root model inside that harness may improve long-context transfer, but the harness remains fixed. It does not create `Hₜ → Hₜ₊₁` or prove `RGₜ > 0`.

### Automated research systems

An automated researcher can propose experiments, implement them, and retain results. It becomes evidence for RSI only if an accepted research-system change measurably improves later research-system changes under the same research evaluator. Producing one stronger model or paper is AI-for-AI research, not yet recursive successor improvement.

## Failure modes

- **Retry inflation.** Counting retries or population members as generations overstates recursion.
- **Resource substitution.** More calls, larger models, or longer wall time increase `Q` without improving the process.
- **Evaluator capture.** Candidate access to tests, judge prompts, score aggregation, or promotion credentials invalidates attribution.
- **Survivorship bias.** Keeping only accepted branches hides the cost and risk of failed search.
- **One-step attribution.** A better child task score establishes immediate improvement, not a better producer of grandchildren.
- **Distribution drift.** Parent and child may face different task, tool, or context distributions.
- **Successor debt.** A candidate can raise task score while making later maintenance or diagnosis harder.

## What would weaken the mechanism

The recursive interpretation weakens if a matched experiment finds `RGₜ ≤ 0`, if gains disappear under a fresh evaluator or delayed task set, if resource normalization explains the effect, or if accepted children produce fewer valid grandchildren. A system that repeatedly improves immediate task score but does not improve later proposal quality is an adaptive optimizer, not evidence for recursive improvement.

## Open technical questions

- Which estimator of `Q(C)` is stable enough when candidate generation is stochastic and expensive?
- How many independent lineages are needed to separate a real recursive effect from winner selection?
- Should successor usability be a hard gate or one axis of a Pareto frontier?
- How can the experiment preserve open-ended novelty without granting the candidate control of its own evaluator?

<details>
<summary>Reference records and operational metadata</summary>

- Full retrieval dates, digests, access states, and claim ceilings are in [[knowledge/rsi/source_registry|the RSI source registry]].
- The bounded closure and stop rule are in [[knowledge/rsi/bounded_transitive_closure|the closure audit]].
- Exact experiment receipts remain missing; [[knowledge/rsi/missing_evidence|the missing-evidence ledger]] tracks that gap.

</details>

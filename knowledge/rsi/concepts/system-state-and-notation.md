---
id: rsi-system-state-and-notation
kind: concept
title: RSI system state and notation
summary: The editable candidate, protected evaluator envelope, generation boundary, and recursive-gain symbols used across the RSI technical spine.
primary_parent: rsi-recursive-improvement-loop
additional_parents:
  - rsi-evaluation-promotion-containment
related: []
attachments:
  - content/source_registry.md
  - content/system_state_and_notation.md
claims: []
human_review: null
---

# RSI system state and notation

## Candidate and protected envelope

Use `Cₜ = (Wₜ, Hₜ, Dₜ, Rₜ)` for the editable candidate at accepted generation `t`.

| Symbol | Meaning | Typical contents | Default write owner |
|---|---|---|---|
| `Wₜ` | Model weights | Frozen provider model or versioned checkpoint | Candidate only when training is in scope |
| `Hₜ` | Harness policy | Prompt, context, tools, workflow, delegation, recovery | Candidate within a declared manifest |
| `Dₜ` | Persistent learned material | Memory, skills, demonstrations, synthetic data | Candidate proposes; evaluator protects split boundaries |
| `Rₜ` | Retrospection policy | Trace inspection, diagnosis, and proposal logic | Candidate |

The protected experiment envelope contains:

| Symbol | Meaning | Why it stays external |
|---|---|---|
| `Xₜ` | Held-out outcomes and other external observations | Candidate-written signals cannot validate the candidate |
| `E` | Evaluator | Tests, judges, aggregation, and integrity checks must remain fixed during comparison |
| `B` | Root-tree budget | Prevents extra calls or children from masquerading as algorithmic gain |
| `P` | Permission policy | Candidate text must not grant runtime authority |
| `Aₜ` | Protected archive | Preserves accepted and rejected lineage, receipts, and rollback |
| `Π` | Proposal protocol | Parent and child need the same process for producing later candidates |

Reading selected records from the envelope does not grant permission to rewrite them.

## Generation and attempt

An attempt is one delivery or execution try. A search round is one optimizer iteration. A generation begins only when external promotion authority records an accepted edge `Cₜ → Cₜ₊₁` in `Aₜ`.

This definition prevents a retry loop, recursive model call, or population of candidates from being counted as successor generations.

## Immediate and recursive gain

Immediate gain compares task performance:

`IGₜ = Score_E(Cₜ₊₁) − Score_E(Cₜ)`

Recursive gain compares the later valid candidates that parent and child produce under the same protocol:

`RGₜ = Q(Cₜ₊₁; Π,E,B,P) − Q(Cₜ; Π,E,B,P)`

`Q` must be defined by the experiment. It may measure the best valid grandchild, the distribution of accepted gains, error rate, cost, useful diversity, or successor usability. A positive `IGₜ` with non-positive `RGₜ` is task improvement, not evidence that the improvement process improved.

<details>
<summary>Original sources for this mechanism</summary>

- [What makes an improvement loop recursive](../chapters/recursive-improvement-loop.md) defines the operational recursion test.
- [Evaluation, promotion, and containment](../chapters/evaluation-promotion-containment.md) defines protected gates and score vectors.
- STOP, §6 "Limitations," distinguishes fixed-model scaffolding optimization from full RSI: [arXiv:2310.02304v3](https://arxiv.org/abs/2310.02304).
- Darwin Gödel Machine, §§2–3 and Figure 2, defines empirically selected coding-agent lineage and its archive: [arXiv:2505.22954v3](https://arxiv.org/abs/2505.22954).

</details>

## Ownership rules

- The candidate may inspect allowed traces and propose edits.
- The runner applies edits and executes effects.
- The evaluator produces protected outcome and integrity judgments.
- The archive records every branch and official parent-child edge.
- Promotion authority decides which valid child becomes the next parent.
- Humans retain decisions whose values or consequences lack an adequate executable evaluator.

These are capability rules. Prompt wording alone does not enforce them.

## Failure modes

- Overloading `E` to mean both environment and evaluator.
- Calling every retry or recursive call a generation.
- Letting the candidate rewrite held-out signals or budget records.
- Comparing parent and child under different proposal protocols.
- Reporting an immediate score delta as recursive gain.
- Treating an internal transcript as the protected lineage archive.

## What would weaken this model

The decomposition is not useful if an experiment cannot identify which component changed, if evaluator and candidate authority cannot be separated, or if parent and child cannot run under a matched envelope. In that case the result may still show system improvement, but it cannot attribute recursive gain to an accepted candidate transition.

<details>
<summary>Reference records and operational metadata</summary>

- The prior [system-state page](../system_state_and_notation.md) now points readers here and remains available for old links.
- Exact source identities and access states remain in [the source registry](../source_registry.md).

</details>

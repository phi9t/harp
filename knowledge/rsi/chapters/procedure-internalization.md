---
id: rsi-procedure-internalization
kind: concept
title: From external procedures to learned behavior
summary: How prompts, playbooks, skills, scripts, workflows, and weights retain a useful practice.
primary_parent: modeling
additional_parents:
  - mlsys
related:
  - kind: changes
    target: rsi-foundation-model-inside-the-loop
  - kind: implemented-by
    target: rsi-harness-engineering
attachments:
  - content/source_registry.md
  - content/recursive_language_models_compositional_generalization.md
claims: []
human_review: null
---

# From external procedures to learned behavior

## One practice, several storage mechanisms

A useful practice can live in a prompt, playbook, skill package, script, workflow, memory record, training example, or model weights. These mechanisms differ in editability, execution guarantee, generalization, latency, and auditability. Calling all of them "learning" hides the engineering choice.

| Representation | How it affects behavior | Persistence | Main advantage | Main risk |
|---|---|---|---|---|
| Prompt | Adds instructions to current context | Per call or configured | Easy to edit and inspect | Model may ignore or misread it |
| Playbook or skill | Selects reusable instructions and resources | Cross-session | Modular and attributable | Retrieval and adherence can fail |
| Script | Executes fixed logic | Versioned code | Deterministic for covered inputs | Brittle outside encoded cases |
| Workflow | Enforces state transitions across time | Durable state | Recovery and mandatory steps | Orchestration complexity |
| Demonstration data | Trains or conditions behavior | Dataset or context | Shows concrete trajectories | Copies errors and narrow styles |
| Model weights | Changes policy across inputs | Checkpoint | Low per-call instruction cost | Harder to inspect, edit, or forget |

Internalization means that behavior previously dependent on an external procedure becomes reliably available from the model under a reduced procedure. It does not require deleting every instruction. It requires a controlled comparison.

## The four-way test

Let `M₀` be the original model, `M₁` the adapted model, `P₀` the minimal baseline harness, and `P₁` the procedure-bearing harness. Evaluate:

| Cell | System | Question |
|---|---|---|
| A | `M₀ + P₀` | What can the original model do unaided? |
| B | `M₀ + P₁` | What benefit does the external procedure provide? |
| C | `M₁ + P₀` | What behavior transferred into weights? |
| D | `M₁ + P₁` | Does the adapted model still use or overdepend on the procedure? |

Define procedure benefit before training as `PB₀ = Score(B) − Score(A)`. Define internalized gain as `IG = Score(C) − Score(A)`. Define residual procedure benefit as `RPB = Score(D) − Score(C)`.

Strong internalization has positive `IG` on held-out tasks and reduced but not necessarily zero `RPB`. If only `D` improves, training may have overfit to the procedure's state and action distribution. If `C` improves but `D` degrades, the external procedure may conflict with learned behavior.

<details>
<summary>Original sources for this mechanism</summary>

- "Harness Updating Is Not Harness Benefit," §§3.1–3.3 and §4, separates the ability to update a harness from a model's ability to activate and follow the update: [arXiv:2605.30621](https://arxiv.org/abs/2605.30621).
- Recursive Language Models, §§2–4, and "Language model harnesses are compositional generalizers," sections "Post-training setup" and "Results," provide a case where training inside a harness changes author-reported transfer behavior; the [[knowledge/rsi/recursive_language_models_compositional_generalization|mechanism anchor]] keeps paper, blog, and code evidence separate: [arXiv:2512.24601v3](https://arxiv.org/abs/2512.24601), [author technical blog](https://alexzhang13.github.io/blog/2026/rlm-harness-generalization/).
- ACE, §§2–4; MCE, §§2–3; and Meta-Harness, §§2–4, describe external context and skill evolution: [ICLR 2026 poster 10008343](https://iclr.cc/virtual/2026/poster/10008343), [arXiv:2601.21557](https://arxiv.org/abs/2601.21557), [arXiv:2603.28052v1](https://arxiv.org/abs/2603.28052).
- Continual Harness, §§3.2–3.4, describes DAgger-style relabeling and process-reward training under evolving harness state: [arXiv:2605.09998v1](https://arxiv.org/abs/2605.09998).

</details>

## Moving a practice through the stack

1. Observe repeated successful trajectories.
2. State the practice as a falsifiable rule with triggering conditions and expected effect.
3. Choose the least expensive representation that preserves correctness.
4. Test the external procedure on held-out tasks.
5. If invocation cost, adherence, or context limits dominate, generate training data from successful and corrected trajectories.
6. Train `M₁` while preserving a frozen evaluation harness.
7. Run the four-way test on familiar, shifted, and adversarial tasks.
8. Keep the external procedure if it still adds value or protects a hard invariant.
9. Remove or simplify it only after the reduced system passes the same gates.

Scripts and workflows should retain hard constraints even after training. A model may learn to request approval, but ordinary code should still enforce approval. Internalization can reduce instruction burden; it should not transfer evaluator or permission authority into mutable weights.

## RLM as a worked example

The RLM harness exposes long context as an external programmatic environment and permits recursive subcalls. Training the root model inside that harness changes the trajectories seen during learning. Reported transfer to longer and cross-domain tasks suggests that a harness can induce a reusable decomposition strategy.

The mechanism is not proof that the harness internalized completely. A direct model without the RLM environment lacks the external context operations. The useful learned behavior is better control of the fixed harness, not replacement of the harness.

## Failure modes and tradeoffs

- **Procedure dependence.** The adapted model works only when the exact training harness is present.
- **Adherence illusion.** The procedure file changes, but the model never retrieves or follows it.
- **Context imitation.** The model memorizes formatting rather than the underlying decision rule.
- **Authority internalization.** A learned policy is trusted to enforce a security or promotion invariant.
- **Catastrophic forgetting.** New procedure behavior damages prior capabilities.
- **Stale internalization.** Weights retain an old procedure after tools or policies change.
- **Hidden duplication.** The same rule persists in prompt, skill, code, and weights with inconsistent versions.
- **Evaluation leakage.** Training examples encode held-out procedures or answers.

External procedures are easier to patch and audit. Weight-level behavior is cheaper per call and may generalize better. The design should keep fast-changing rules external and move stable, high-frequency strategies into training only when the four-way test supports it.

## What would weaken the mechanism

The internalization claim weakens if `Score(C)` does not improve over `Score(A)`, if gains vanish under paraphrase or task shift, or if `D` works only because evaluation reproduces training scaffolding. Adherence logs showing that `P₁` was never invoked would invalidate a claim about procedure benefit.

## Open technical questions

- How can a model forget a superseded procedure without broad retraining?
- Which behavioral invariants should never move from code into weights?
- How should training represent tool failures and permission denials so the model learns recovery rather than shortcuts?
- Can skill-selection policies be trained while keeping skill contents separately versioned?

<details>
<summary>Reference records and operational metadata</summary>

- Exact paper identities, access states, and claim ceilings are in [[knowledge/rsi/source_registry|the source registry]].
- Harp inspected narrow pinned full and minimal code snapshots but did not run the
  RLM training or benchmark recipe.
- Internalization remains a mechanism claim unless a complete four-way receipt is available.

</details>

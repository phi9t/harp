---
id: rsi-system-rlm
kind: concept
title: Recursive Language Models
summary: Context offloading into a REPL, programmatic and recursive model calls, author-reported long-context and cross-domain transfer, runtime cost, and the boundary between recursive inference and RSI.
primary_parent: rsi-joint-harness-weight-adaptation
additional_parents:
  - rsi-harness-engineering
related:
  - kind: implemented-by
    target: rsi-harness-engineering
attachments:
  - content/recursive_language_models_compositional_generalization.md
claims: []
human_review: null
---

# Recursive Language Models: recursive inference through a harness

## Mechanism

**EVIDENCE — [RLM-PAPER].** An RLM externalizes long input into a REPL, lets a
root model inspect and transform selected portions programmatically, and permits
additional language-model calls, including recursive RLM calls. Intermediate
state can remain in variables rather than accumulating in the root transcript.

The Transformer does not gain recursive layers. Recursion belongs to the
system-level call and state structure.

## Learning interpretation

**CLAIM — [A1ZHANG-HARNESS-BLOG].** The harness may act as a higher-level
inductive bias by mapping globally unfamiliar tasks into locally familiar model
calls and reusable decomposition trajectories.

The blog describes this as harness-induced compositional generalization and
uses trajectory-similarity proxies. Terms such as locally in-distribution and
isomorphic are operational intuitions, not proven latent-space facts.

<details>
<summary>Original sources for this mechanism</summary>

- RLM architecture and results: [RLM paper, arXiv v3](https://arxiv.org/abs/2512.24601).
- Post-training and compositional-generalization experiments: [author technical blog](https://alexzhang13.github.io/blog/2026/rlm-harness/).
- Pinned implementation metadata: `evidence/rlm/artifacts/git/README.md`.
- Full local companion: [RLM compositional-generalization deep dive](../recursive_language_models_compositional_generalization.md).

</details>

## Reported evaluation and cost

The author blog reports RLM post-training on one Qwen3 30B-family model,
transfer to six task settings 8 to 32 times longer, and three paired
cross-domain settings. It reports stronger evaluation lift than a selected
direct-Transformer baseline and a 1.5 to 3 times runtime premium on similarly
sized tasks because RLM samples require multiple calls and waits.

The packet contains no independent reproduction or fully matched comparison at
equal tokens, FLOPs, calls, wall time, and dollars.

## Failure modes and limits

- Decomposition may be brittle or task-specific.
- Root traces can look similar while sub-call trees differ greatly in cost.
- External state and generated REPL code create security and recovery concerns.
- Better inference recursion does not imply a better harness optimizer.
- The reported RL experiment keeps the harness design fixed.

## Claim ceiling

RLM demonstrates recursive inference and author-reported harness-induced
learning transfer. It does not demonstrate recursive self-improvement:
`H_t -> H_(t+1)` is not learned and no accepted child is tested as a better
producer of later harness changes.

## Reading routes

- [Weng: harness versus core intelligence](../weng/03-harness-layer-vs-core-intelligence.md)
- [RLM mechanism deep dive](../recursive_language_models_compositional_generalization.md)
- [Joint harness and model-weight adaptation](../chapters/joint-harness-weight-adaptation.md)
- [Original paper](https://arxiv.org/abs/2512.24601)

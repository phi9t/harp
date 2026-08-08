---
id: recursive-self-improvement-missing-evidence
title: Recursive self-improvement - missing evidence
type: missing-evidence-ledger
mode: DOMAIN ORIENTATION
status: active
created: 2026-07-31
updated: 2026-08-01
tags: [recursive-self-improvement, missing-evidence, research-queue]
confidence: medium
---

# Missing evidence

Mode: `DOMAIN ORIENTATION`.

## Critical gaps

**MISSING.** A multi-generation experiment showing that accepted generation `t+1` is better than generation `t` at producing further accepted improvements, with evaluator and compute held fixed.

**MISSING.** A matched comparison separating gains from proposer quality, search budget, evaluator changes, harness changes, training-data changes, and weight updates.

**MISSING.** Independent reproduction of the RLM post-training results with total tokens, FLOPs, model calls, wall-clock time, and dollar cost matched against the direct-Transformer baseline.

**MISSING.** A causal ablation separating context offloading, programmatic sub-calls, recursion, prompt nudges, and additional inference compute in the RLM harness-generalization result.

**MISSING.** An independent reproduction of DGM, Meta-Harness, Self-Harness, AHE, or SIA across the same tasks, models, budgets, and acceptance rules.

**MISSING.** A generally accepted evaluation for research taste, importance, negative results, safety, and long-term scientific value that is robust to optimization pressure.

**MISSING.** Evidence that a public system autonomously chooses an important general AI research problem, builds a more capable successor, independently validates it, controls deployment, and repeats the whole cycle.

**MISSING.** A security evaluation where the editable system is adversarially incentivized to alter tests, graders, resource accounting, permissions, or lineage and the external controls are shown to hold.

**MISSING.** A matched architectural benchmark comparing the inspected Pi, Hermes, and Codex revisions under the same model, tasks, tool capabilities, context budget, concurrency budget, wall-clock limit, and scoring rule.

**MISSING.** A machine-readable candidate receipt schema shared across the harness examples that records editable-path manifest, parent revision, resource spend, evaluator revision, score vector, rejected regressions, and promotion authority.

**MISSING.** A test showing that branch summaries, memory writes, skill consolidation, or rollout compaction preserve the information needed for later improvement work rather than only preserving immediate task completion.

**MISSING — Codex/ARC contribution split.** The supplied OpenAI result reports
reasoning retention and compaction together. It does not provide
retention-only and compaction-only scores. The fastest resolution is the
matched 2×2 experiment in [[codex_state_continuity_and_compaction]].

**MISSING — compaction fidelity and service semantics.** The inspected Codex
and API sources establish typed checkpoints, replay, opaque reasoning and
compaction items, and `previous_response_id` transport. They do not establish
semantic fidelity across repeated compactions, cross-model checkpoint
portability, server-side KV-cache behavior, or durable external side-effect
semantics.

**MISSING.** An adversarial test showing that a candidate cannot gain evaluator capabilities by changing tool visibility, invoking a deferred registry path, editing a project-local extension or skill, or inheriting authority through a child agent.

## Vendored but not interpreted

**MISSING — `sources/source_registry.tsv`.** All 39 of Weng's numbered first-hop references now have local captures, but most remain `vendored-uninspected`. Their identity and relationship are preserved and their text is searchable; their mechanisms, results, and limitations must still not be reconstructed from titles, Weng's paraphrase, or an unreviewed local file.

**MISSING.** Full text for [GODEL-MACHINE] remains paywalled; the vendored representation is the publisher page and bibliography rather than the chapter text.

**MISSING.** Official venue status for many 2026 references was not verified. They remain in the `preprint-watchlist` even when [WENG-HARNESS] reports a venue.

**MISSING.** Citation counts are intentionally absent because no bibliometric provider, snapshot date, and exact matched records were acquired.

**MISSING.** The RLM paper's 51 parsed outgoing citations and the harness blog's 11-entry bibliography are captured but not yet resolved into an admitted bounded closure. Their presence is not evidence that those works support the maintained RSI synthesis.

## Research queue after the orientation pass

**INFERENCE — completed design step.** [[evaluator_integrity_and_promotion]] now formalizes the candidate/envelope split, promotion gates, resource accounting, reward-tampering threats, lineage receipts, adversarial boundary tests, and a Pi-first falsification protocol. None of those proposed controls has yet been exercised in a multi-generation run.

**INFERENCE — queue priority 1.** Reproduce one harness-evolution loop with an immutable evaluator, fixed token and wall-clock budgets, a lineage archive, and at least three accepted generations; measure both task score and quality of subsequent proposed edits.

**INFERENCE — queue priority 2.** Compare archive policies—greedy lineage, Pareto frontier, novelty search, and quality diversity—under identical proposer and evaluator conditions.

**INFERENCE — queue priority 3.** Study joint harness/weight updates for interference, attribution, rollback, and evaluator overfitting before promoting SIA-like systems into the core evidence tier.

**INFERENCE — queue priority 4.** Implement the smallest candidate/evaluator split on one harness: Pi for a compact mutable surface, Hermes for memory/skill adaptation, or Codex for lineage and containment. Reuse the same receipt schema so later cross-harness comparison remains possible.

**SPECULATION.** The first decisive negative result may be as valuable as a positive one: a carefully controlled study could show that performance saturates or regresses because later generations overfit the evaluator faster than they improve the improvement operator.

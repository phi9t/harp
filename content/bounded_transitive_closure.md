---
id: recursive-self-improvement-bounded-transitive-closure
title: Recursive self-improvement - bounded transitive closure
type: traversal-audit
mode: DOMAIN ORIENTATION
status: ClosurePartial
created: 2026-07-31
updated: 2026-08-01
tags: [recursive-self-improvement, provenance, evidence-graph, closure]
confidence: medium
---

# Bounded transitive closure

Mode: `DOMAIN ORIENTATION`.

## Closure definition

**INFERENCE — audit scope.** This document audits the Weng-rooted subclosure of a multi-anchor RSI topic. The RLM research lineage is a separate first-class mechanism anchor whose current capture includes [RLM-PAPER], [RLM-REPO], [A1ZHANG-HARNESS-BLOG], the blog bibliography, and the paper's parsed outgoing references. It is not retroactively counted as a Weng citation.

**INFERENCE — Weng seed set.** `C₀ = {SEED, WENG-HARNESS}`. `SEED` supplies the user's practical scope and reading intent. `WENG-HARNESS` is the anchor of this subgraph.

**INFERENCE — first expansion.** `C₁` registers all 39 numbered references in [WENG-HARNESS]. A body link is additionally retained only when it changes a decision about workflow realization, evaluator integrity, or the boundary between bounded automation and successor development.

**INFERENCE — second expansion.** `C₂` follows one citation hop from fetched `C₁` mechanisms only when the source supplies a missing foundation or direct predecessor. This admitted [GODEL-MACHINE] for proof-certified self-rewrite, [QD-2016] for archive diversity, [FUNSEARCH] for evaluator-grounded program evolution, and [CONCRETE-SAFETY] for reward-hacking lineage.

**INFERENCE — bounded closure.** The active map is `C* = retain(C₀ ∪ C₁ ∪ C₂)`. Depth three is forbidden in this orientation pass. The literal transitive closure would expand into all of language modeling, reinforcement learning, evolutionary computation, program synthesis, scientific method, software testing, and AI safety, so it would not reach a useful research fixpoint.

## Retention predicate

**INFERENCE — admission rule.** A source is admitted to synthesis only if it changes at least one of these decisions:

- **INFERENCE.** Whether the update is ephemeral or persistent.
- **INFERENCE.** Which part of candidate `Cₜ = (Wₜ, Hₜ, Dₜ, Rₜ)` is editable and which evaluator, budget, permission, archive, and promotion surfaces remain external, as defined in [[system_state_and_notation]].
- **INFERENCE.** How proposals are generated and the search space is represented.
- **INFERENCE.** How candidates are executed, evaluated, selected, and archived.
- **INFERENCE.** Whether improvement of later improvement ability is actually measured.
- **INFERENCE.** Which evaluator, security, budget, or deployment boundary prevents false improvement.

**INFERENCE — rejection rule.** Generic model architecture, prompting, RL, optimizer, benchmark, and company-positioning sources stop unless they contribute a distinct mechanism or a necessary evaluation boundary. A source that only repeats relevance stays registered but does not enter the synthesis.

## Traversal ledger

| Label | Branch | First-hop roots inspected | Retained second hop | Stop reason |
|---|---|---|---|---|
| EVIDENCE | Definition and formal self-reference | [WENG-HARNESS], [STOP], [DGM] | [GODEL-MACHINE] | Formal logic and universal search beyond the proof-versus-empirical selection contrast do not change the current practical taxonomy. |
| EVIDENCE | Harness and workflow optimization | [ADAS], [AFLOW], [META-HARNESS], [SELF-HARNESS], [AHE] | None | Agent-framework building blocks are already represented by executable-code, trace, search, and regression-gate axes. |
| EVIDENCE | Evolutionary program search | [ALPHAEVOLVE], [DGM] | [FUNSEARCH], [QD-2016] | General evolutionary-computation ancestry stops after evaluator grounding and archive diversity are explicit. |
| EVIDENCE | Autonomous research and evaluation | [AI-SCIENTIST], [NOT-SCIENTISTS], [PAPERBENCH], [REBENCH], [KERNELBENCH] | None | Individual benchmark task and paper lineages do not change the RSI decision boundary in this pass. |
| EVIDENCE | Evaluator integrity | [WENG-REWARD], [SELF-HARNESS], [AHE] | [CONCRETE-SAFETY] | Broader alignment and reward-model literatures are deferred to a dedicated technical deep dive. |
| EVIDENCE | Harness and weight co-adaptation | [SIA] | None | Continual learning and self-play are registered at first hop but not expanded because the anchor explicitly leaves them outside its focus. |

## Closure accounting

**EVIDENCE — `sources/source_registry.tsv`.** The Weng-rooted citation closure contains two seeds, all 39 numbered first-hop references, three retained body-linked sources, and four second-hop foundations: 48 nodes total. The registry contains seven additional non-closure nodes: the RLM paper and blog as research anchors, the announcing X clipping, and four implementation snapshots—Pi, Hermes Agent, OpenAI Codex, and pinned RLM repository metadata. These nodes do not retroactively enter `C*` or trigger another Weng-citation expansion. Every node has one stable ID, depth, access status, cohort, locator, and claim ceiling.

**EVIDENCE — `sources/evidence_graph.tsv`.** Every numbered reference has a `cites` edge from [WENG-HARNESS]. Retained conceptual and implementation relationships use separate typed edges such as `extends`, `implements`, `critiques`, `evaluates`, and `requires`.

**EVIDENCE — vendored closure.** `evidence/weng/` contains all 47 external nodes in `C*`: 47 primary captures, text for every node, 43 raw bibliography sections, 1,905 structured outgoing-citation records across 29 nodes, and 55 retained typed edges. The structured parser prefers arXiv HTML `ltx_bibitem` records and falls back only to unambiguous numbered bibliographies. Parsed depth-three citations are not admitted automatically.

**INFERENCE — saturation test.** The inspected subset supplies at least one primary source for each method family and each critical boundary in [[domain_orientation]]. New first-hop reading would refine comparisons and measurements, but no unresolved source identity prevents the current map from explaining what practical RSI is and is not.

**INFERENCE — status.** The traversal is `ClosurePartial`, not `ClosureSaturated`. Only a selected decision-relevant subset of first-hop sources was inspected, second-hop expansion was capped at depth two, and no independent multi-generation reproduction closes the core RSI claim.

**INFERENCE — implementation-overlay stop rule.** Source-code dependencies, design influences, issues, and release histories of [PI-MONO], [HERMES-AGENT], and [CODEX-REPO] are not traversed in this orientation. The overlay asks how current harnesses realize the map's control surfaces, not for a second unbounded source-code genealogy.

## Reproducibility receipt

**EVIDENCE — local acquisition on 2026-07-31.** The external closure capture completed at `2026-08-01T05:27:46Z`. `evidence/weng/manifest.tsv` records one result per node; `artifact_inventory.tsv` records the size and SHA-256 of all 231 captured or derived files, including five canonical license-reference captures. The user seed remains a separate 22,746-byte input with SHA-256 `fe06902aaf91996177b40ee47d6fb83f1a92f3503c144aeb1951c54b4a564e1a`.

**INFERENCE — capture policy.** Raw live pages are not canonical inputs to later synthesis. The vendored representations and their receipts are the organic reference corpus; `sources/source_registry.tsv`, `sources/evidence_graph.tsv`, and `claim_evidence_ledger.md` remain the interpreted claim and provenance layers. Refetching a mutable source requires a new retrieval time and digest.

**EVIDENCE — local search boundary.** Harp's SQLite FTS index includes the
maintained Markdown, all 47 Weng-closure text representations, and the three
RLM-lineage text representations. `harp search refresh` binds the index to a
digest of those canonical inputs, and `harp search status` rejects a missing
or stale receipt. Searchability does not change the citation graph or promote
an extracted text representation above its captured source.

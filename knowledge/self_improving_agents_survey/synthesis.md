---
id: self-improving-agents-survey-synthesis
title: Self-improving agents survey - Harp synthesis
type: survey-anchor
mode: DOMAIN ORIENTATION
status: active
created: 2026-08-09
updated: 2026-08-09
tags: [recursive-self-improvement, survey, foundation-model-adaptation, scaffold-improvement, evaluation, bibliography]
confidence: medium
---

# Self-improving agents survey: Harp synthesis

Mode: `DOMAIN ORIENTATION`.

This packet treats `Self-Improvements in Modern Agentic Systems: A
Survey` as a survey anchor, not as primary mechanism evidence for every linked
work. The survey and hub organize the landscape; individual papers and
repositories still own their mechanisms, results, limitations, and licenses.

## Evidence tiers and claim ceilings

| Source tier | Local artifact | What it supports | Claim ceiling |
|---|---|---|---|
| Survey paper | [SIMAS-PAPER], `evidence/self_improving_agents_survey/survey/artifacts/pdf/self-improvements-modern-agentic-systems-2607.13104v1.pdf`, extracted text in `survey/text/` | Formal vocabulary: agent configuration `A_t = (theta_t, Sigma_t)`, scaffold components, self-induced update operator, and the two top-level update pathways. | Author framing and taxonomy. No independent reproduction or verification of linked paper results. |
| Project hub | [SIMAS-SITE], `evidence/self_improving_agents_survey/site/index.html` | Dated hub taxonomy, representative paper path, category counts, one-hop bibliography topology, and same-site overview figure. | Dated project-page claim; mutable web page, not peer review or mechanism proof. |
| Pinned site source | [SIMAS-SITE-REPO], `evidence/self_improving_agents_survey/repos/site_repo/index-d8af6607ced118351108670f823cd106649cb757.html` | GitHub Pages source bytes at commit `d8af6607ced118351108670f823cd106649cb757`. | Pinned source identity for the rendered hub. |
| Update repository | [SIMAS-AWESOME-REPO], `evidence/self_improving_agents_survey/repos/awesome_repo/` | Linked bibliography/update repository identity, MIT license file, and default-branch commit `57a1d89e5bafcd65db7feb51e809422700aeb48a`. | Repository metadata and README framing only; no full source-tree audit. |
| One-hop inventory | `evidence/self_improving_agents_survey/link_inventory.tsv` | 467 unique hub links: 245 arXiv, 23 OpenReview, 183 GitHub, 10 publisher, one Hugging Face, one same-site, and four project/community links. | Link identity and topology. Child bibliographies are out of scope. |

The extraction path is recorded in `evidence/self_improving_agents_survey/acquire.sh`.
`pdftotext` emits PDF syntax warnings against the arXiv PDF but still produces
a stable text file; the raw PDF remains the source artifact.

## Alignment with Harp vocabulary

**EVIDENCE. [SIMAS-PAPER], `survey/text/...:1342-1515`.** The survey defines a
foundation-model-based agent as a persistent configuration:

```text
A_t = (theta_t, Sigma_t)
Sigma_t := (p_t, m_t, T_t, g_t)
```

Here `theta_t` is the foundation model, while `Sigma_t` is the operational
scaffold: prompts or system instructions, memory and update policies, tools and
invocation interfaces, and additional control logic such as routing,
scheduling, or safety constraints. The paper separates this intrinsic
configuration from ephemeral execution state `X_t`, such as caches, plans, or
short-term working memory.

**Harp mapping.** This matches Harp's boundary discipline:

| Survey term | Harp term | Interpretation |
|---|---|---|
| `theta_t` | foundation-model substrate | Weight or parameter state. |
| `Sigma_t` | harness/scaffold envelope | Prompt, memory, tool, filesystem, routing, workflow, and safety authority. |
| `X_t` | transient execution state | Useful for a run, but not persistent improvement by itself. |
| `U_theta` | foundation-model adaptation | Update via self-induced data, rewards, preferences, critiques, or verification signals. |
| `U_Sigma` | scaffold/harness improvement | Durable edit to prompts, memory, tool interfaces, or control logic. |

This is a useful survey anchor because it keeps model updates and harness
updates in one coordinate system. Harp should still keep the durable promotion
boundary explicit: a proposed edit, synthetic label, or critique is not an
accepted improvement until an external or typed update rule commits it.

## Two pathways, one promotion problem

**EVIDENCE. [SIMAS-PAPER], `survey/text/...:1468-1515`.** The survey splits
self-improvement into two broad update modes:

- foundation-model improvement updates `theta` while holding the scaffold fixed;
- scaffolding improvement updates `Sigma` while holding model parameters fixed.

**INFERENCE.** Harp should read those as target classes, not proof classes.
Both require a promotion mechanism:

| Target | Typical signal | Promotion object | Harp risk |
|---|---|---|---|
| Foundation model | self-generated trajectories, preferences, rewards, critiques, synthetic labels | data, gradient update, RL/preference update, distillation recipe | Evaluation leakage, weak provenance, hidden compute, no held-out proof of future usefulness. |
| Prompt/context | scalar score, critique, textual gradient, population search | prompt template, instruction, context policy | Optimizer overfits a local evaluator or benchmark. |
| Memory | stored object, graph, vector index, consolidation/deletion policy | memory item, schema, retrieval/update operator | Persistence is confused with correctness; stale memory becomes authority. |
| Tool | routing rule, tool schema, generated tool, repair patch | action space and execution interface | Tool availability changes the task envelope and can bypass evaluator assumptions. |
| Full scaffold | executable agent logic, workflow, archive, search policy | harness program or release candidate | A broader candidate can modify the measurement surface that selects it. |

## What the hub adds beyond the paper

**EVIDENCE. [SIMAS-SITE], `site/index.txt:52-81`.** The hub turns the survey
taxonomy into a browsable bibliography with three top-level branches and
counts: 77 foundation-model-improvement entries, 176 scaffolding-improvement
entries, and 59 evaluation entries, for 312 categorized library entries. It
also links the taxonomy to representative papers such as Self-Instruct,
Constitutional AI, WebRL, Self-Refine, TextGrad, MemoryBank, Voyager, and
Darwin Godel Machine.

**EVIDENCE. [SIMAS-SITE], `site/index.txt:552-690`.** The hub's full-scaffold
section lists works that overlap directly with Harp's harness-search coverage:
STOP, ADAS, DGM, AlphaEvolve, ShinkaEvolve, Continual Harness, Hyperagents, and
other scaffold-editing systems. The evaluation branch adds benchmarks and
measurement surfaces including CORE-Bench, SWE-bench, PaperBench, AstaBench,
MLS-Bench, WebArena, MINT, GAIA, and ToolEmu.

**INFERENCE.** Compared with Harp's Weng-centered packet, this hub contributes:

- a broader foundation-model-improvement lane, especially self-generated data,
  intrinsic feedback, grounded environments, and world models;
- a much larger memory and tool taxonomy than the current Harp synthesis;
- a dedicated evaluation branch that separates measuring improvement from
  benchmarking improvement;
- an update repository that can act as a mutable watchlist; and
- a visible gap list for future deep dives without forcing every link into the
  current claim ledger.

## Relationship to existing Harp packets

| Existing Harp area | Survey overlap | Boundary |
|---|---|---|
| Weng harness engineering | Prompt, memory, tool, workflow, full-scaffold search, and evaluator concerns. | Weng remains the local anchor for harness-engineering interpretation; this survey adds breadth and a formal target taxonomy. |
| DGM | The hub places DGM under full scaffolding. | Harp's DGM packet owns implementation and claim analysis; the survey proves only that the hub classifies DGM there. |
| Meta-Harness, ADAS, AFlow, GEPA, Promptbreeder | The hub places these in scaffold optimization or prompt/workflow search neighborhoods. | Existing source packets own mechanisms. The survey provides crosswalk/topology only. |
| RLM and joint adaptation | The survey's `theta` plus `Sigma` framing matches Harp's model-plus-harness adaptation vocabulary. | RLM-specific code and paper claims remain under the RLM packet. |
| Evaluator integrity | The survey/hub adds many benchmark identities. | Harp's evaluator-integrity packet still owns access-policy, verifier, and benchmark-disclosure analysis. |

## Recommended Harp use

Use this survey as the landscape map when deciding which RSI subsystem to
inspect next. Do not use it as a replacement for primary-source review.

1. Start with the target being changed: `theta`, prompt, memory, tool, full
   scaffold, or joint model-plus-harness.
2. Identify the update signal: demonstration, critique, reward, verifier,
   trajectory, memory consolidation, tool failure, or benchmark score.
3. Ask who commits the update and what evidence proves it was committed.
4. Separate identity-only hub links from inspected source packets.
5. Promote only a small number of linked works to deep dives when they close a
   concrete Harp gap.

## Claim ceiling

This packet supports:

- the survey's formal framing of modern self-improving agents as
  foundation-model-plus-scaffold configurations;
- a source-backed crosswalk between survey taxonomy and Harp's RSI vocabulary;
- one-hop bibliography topology from the hub; and
- identification of future deep-dive candidates.

It does not support:

- correctness of every linked paper or repository;
- benchmark or leaderboard reproduction;
- venue-status verification for every hub entry;
- implementation behavior for linked repositories other than the narrow
  metadata captured here; or
- a claim that any listed system demonstrates recursive self-improvement.

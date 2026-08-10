---
id: self-improving-agents-survey-claim-evidence-ledger
title: Self-improving agents survey - claim evidence ledger
type: claim-ledger
mode: CLAIM LEDGER
status: active
created: 2026-08-09
updated: 2026-08-09
tags: [recursive-self-improvement, claim-ledger, survey, evidence]
confidence: medium
---

# Self-improving agents survey claim evidence ledger

Mode: `CLAIM LEDGER`.

| Claim ID | Claim | Evidence | Status | Boundary |
|---|---|---|---|---|
| SIMAS-C001 | The survey frames a modern foundation-model agent as a configuration coupling model parameters and scaffold state. | [SIMAS-PAPER], `evidence/self_improving_agents_survey/survey/text/self-improvements-modern-agentic-systems-2607.13104v1.txt:1342-1379` | source-backed | Author formalism; not a universal standard. |
| SIMAS-C002 | The survey separates transient execution state from durable intrinsic configuration. | [SIMAS-PAPER], `survey/text/...:1361-1379` | source-backed | Does not prove every listed system enforces this boundary. |
| SIMAS-C003 | The survey defines self-improvement as a self-induced operator that executes an agent policy to produce a learning signal or edit, then commits a durable update. | [SIMAS-PAPER], `survey/text/...:1424-1465` | source-backed | Formal framing only; acceptance criteria remain system-specific. |
| SIMAS-C004 | The top-level update targets are foundation-model parameters and scaffolding components. | [SIMAS-PAPER], `survey/text/...:1468-1515`; [SIMAS-SITE], `site/index.txt:52-81` | source-backed | This is a taxonomy, not proof that listed systems achieve persistent gains. |
| SIMAS-C005 | The scaffold decomposition overlaps Harp's prompt, memory, tool, control-logic, and harness vocabulary. | [SIMAS-PAPER], `survey/text/...:1349-1357`; Harp synthesis in `knowledge/self_improving_agents_survey/synthesis.md` | inference | Crosswalk is Harp-authored; survey owns only its terms. |
| SIMAS-C006 | The hub organizes 312 categorized entries across 77 foundation-model-improvement, 176 scaffolding-improvement, and 59 evaluation entries. | [SIMAS-SITE], `site/index.txt:52-81` | source-backed | Dated hub count; mutable and not independently reconciled against every row. |
| SIMAS-C007 | The hub's one-hop link surface is broad enough that this packet should preserve topology without crawling child bibliographies. | `evidence/self_improving_agents_survey/link_inventory.tsv`; `one_hop/link_summary.json` | source-backed inventory | Link identity only; no mechanism claims for individual linked works. |
| SIMAS-C008 | The hub places DGM, STOP, ADAS, AlphaEvolve, ShinkaEvolve, Continual Harness, and Hyperagents in the full-scaffold neighborhood. | [SIMAS-SITE], `site/index.txt:552-580` | source-backed | Hub classification only; existing Harp packets own inspected mechanisms. |
| SIMAS-C009 | The hub adds an evaluation branch that distinguishes measuring improvement from benchmarking improvement. | [SIMAS-SITE], `site/index.txt:585-690` | source-backed | Does not validate benchmark access policies, hidden tests, or scores. |
| SIMAS-C010 | The Awesome repository is a linked update list with MIT license text captured at commit `57a1d89e5bafcd65db7feb51e809422700aeb48a`. | [SIMAS-AWESOME-REPO], `repos/awesome_repo/default-branch-commit.json`, `README.md`, `LICENSE` | source-backed | Repository metadata and top-level files only; no full tree audit. |

## Ledger rules

- Claims about a linked paper's mechanism must cite that paper or an existing
  Harp packet, not the hub alone.
- Claims about a linked repository's implementation must cite a pinned source
  snapshot, not a GitHub link in `link_inventory.tsv`.
- Counts from the hub are dated page facts. Refresh before using them as a
  current bibliography metric.
- Derived text locators are reading aids. When source identity matters, cite
  the raw PDF, raw HTML, or pinned repository file.

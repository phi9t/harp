---
source_id: COREBENCH
title: "CORE-Bench: Fostering the Credibility of Published Research Through a Computational Reproducibility Agent Benchmark"
weng_locator: reference-34
section_id: future-challenges
primary_url: https://arxiv.org/abs/2409.11363
captured_path: evidence/weng/text/corebench.txt
publication_state: "Manuscript dated September 17, 2024; captured arXiv:2409.11363v2 dated June 22, 2026; Weng labels it TMLR 2024; venue/version relationship not independently resolved"
evidence_state: card-complete
edited_object_family: evaluation-benchmark
claim_ceiling: Sections 2–4 and Appendices A3 and C.3 270-task computational-reproducibility benchmark, test-set task-accuracy definition, three-run CORE-Agent with GPT-4o mean and 95% CI, distinct pass@1/pass@3 retry accounting, GPT-4o-mini comparison, unresolved publication identity, and cost, safety, and construct limits; no independent reproduction
lesson_ids: 0009,0010
card_path: knowledge/rsi/weng-sources/corebench.md
canonical_route: knowledge/rsi/weng/09-future-challenges.md
---

# Evaluating Computational Reproducibility

## Problem

Captured lines 137–140 argue that research agents should reproduce existing results before claims of autonomous novel research.

## Core mechanism

Lines 142–152 and 241–288 define 270 difficulty-level tasks derived from 90 reproducible CodeOcean papers in computer science, social science, and medicine: three tiers per paper. The split is 45 train papers and 45 test papers. A task passes only when every question about reproduced outputs is correct.

## Reported evidence

Across 270 tasks from 90 CodeOcean papers in computer science, social science, and medicine, task accuracy is the proportion with every question correct; higher is better. Table 5 names AutoGPT as the architecture comparator. CORE-Agent was run three times, while AutoGPT values are single-run due to cost. CORE-Agent plus GPT-4o averages 21.48% on 45 hard-tier test tasks; Appendix A3 gives 21.48% ± 2.60 as a 95% CI. Separately, Appendix C.3 reports pass@1 22.2% and pass@3 31.1%. Harp did not reproduce these results.

## Key limitation

The corpus starts from already reproducible CodeOcean capsules, not arbitrary papers. Lines 399–403 cap API cost at four dollars per task; lines 523–534 document unsafe web actions and needed guardrails. Task accuracy measures executing code and retrieving outputs, not validating paper conclusions.

## Why Weng cites it

Weng lines 374–378 says the best agent used "GPT-4o and GPT-4o-mini," which is imprecise. Primary Table 5 attributes 21.48% specifically to CORE-Agent plus GPT-4o; GPT-4o-mini scores 16.30% on the hardest tier.

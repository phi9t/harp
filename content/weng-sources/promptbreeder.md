---
source_id: PROMPTBREEDER
title: "Promptbreeder: Self-Referential Self-Improvement Via Prompt Evolution"
weng_locator: reference-18
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2309.16797
captured_path: evidence/weng/text/promptbreeder.txt
publication_state: "arXiv:2309.16797v1 preprint; peer-reviewed venue not verified"
evidence_state: card-complete
edited_object_family: evolutionary-program-or-population-search
claim_ceiling: Sections 3–6 and Appendix J population-based task-prompt and mutation-prompt evolution, author-reported reasoning results, fixed prompting-topology limit, and evaluation scope; no independent reproduction
lesson_ids: 0007,0010
card_path: content/weng-sources/promptbreeder.md
canonical_route: content/weng/07-evolutionary-search.md
---

# Evolving Prompts and Their Mutators

## Problem

Captured lines 24–40 ask how to automate prompt engineering without the early performance plateau reported for repeated prompt selection.

## Core mechanism

Lines 235–265 define a population whose units pair task prompts with mutation prompts. A binary-tournament genetic algorithm measures task-prompt fitness on 100-example training batches, keeps the stronger unit, and mutates it. Nine operators in five classes appear at lines 289–300 and 400–448. Hypermutation changes mutation prompts, so the search also evolves how it edits task prompts. Model weights remain unchanged.

## Reported evidence

Table 1 at lines 50–74 reports zero-shot PaLM 2-L Promptbreeder accuracy on eight reasoning tasks. It scores 83.9% on GSM8K, a grade-school math word-problem benchmark, versus 60.5% for PaLM 2-L Plan-and-Solve+ and 77.9% for PaLM 2-L Automatic Prompt Engineer; the table mixes same-model baselines with bracketed text-davinci-003 results. Appendix J lines 1364–1377 says fitness used 100-example training batches and the best candidate was evaluated on a held-out test set. Harp did not reproduce these results.

## Key limitation

Lines 503–510 say Promptbreeder changes prompt content but keeps the prompting algorithm and topology fixed. Its evidence is task-specific, and search can overfit the fitness batches; one held-out test evaluation is the reported check.

## Why Weng cites it

Weng lines 259–264 uses it as early evolutionary prompt search whose mutation instructions also evolve.

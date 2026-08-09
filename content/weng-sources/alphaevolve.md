---
source_id: ALPHAEVOLVE
title: "AlphaEvolve: A coding agent for scientific and algorithmic discovery"
weng_locator: reference-20
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2506.13131
captured_path: evidence/weng/text/alphaevolve.txt
publication_state: preprint white paper
evidence_state: card-complete
edited_object_family: evolutionary-program-or-population-search
claim_ceiling: Inspected §§2–3 mechanism and application sections, fixed problem and evaluator boundaries, author-reported discoveries and deployments, and limitations; no independent reproduction
lesson_ids: 0007,0010
card_path: content/weng-sources/alphaevolve.md
canonical_route: content/systems/alphaevolve.md
---

# Evolving Programs under Automatic Evaluation

## Problem

Captured lines 26–56 ask how language models can make scientific or engineering discoveries when candidate algorithms are automatically evaluable.

## Core mechanism

A fixed LLM ensemble proposes diffs to parent programs sampled with inspirations from a program database. External evaluators execute and score children, and promising programs return to the database. Humans fix the problem, evaluator, initial program, and editable blocks; model weights do not update.

## Reported evidence

The white paper reports applications in datacenter scheduling, accelerator circuits, LLM training kernels, attention, mathematics, and algorithms. One bounded result is a 4 by 4 complex matrix-multiplication procedure using 48 scalar multiplications, presented as the first improvement over Strassen for that setting in 56 years. Harp did not reproduce these author-reported applications.

## Key limitation

The source says automated evaluation excludes work requiring manual experimentation and that humans supply the problem, evaluator, and initial program. Harp's boundary is that application results do not show AlphaEvolve improved its own harness, model weights, or later improvement production.

## Why Weng cites it

Weng lines 265–275 uses AlphaEvolve to explain population search over code, evaluator feedback, rich parent context, editable blocks, and optional meta-prompt evolution.

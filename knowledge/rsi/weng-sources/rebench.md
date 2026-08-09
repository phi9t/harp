---
source_id: REBENCH
title: "RE-Bench: Evaluating Frontier AI R&D Capabilities of Language Model Agents against Human Experts"
weng_locator: reference-31
section_id: future-challenges
primary_url: https://proceedings.mlr.press/v267/wijk25a.html
captured_path: evidence/weng/text/rebench.txt
publication_state: ICML 2025 official proceedings
evidence_state: card-complete
edited_object_family: evaluation-benchmark
claim_ceiling: Official §§2–5 time-budget, score@k, human-attempt accounting, reported comparisons, cost, environment scope, and limitations; no independent reproduction
lesson_ids: 0009,0010
card_path: knowledge/rsi/weng-sources/rebench.md
canonical_route: knowledge/rsi/weng/09-future-challenges.md
---

# Comparing Agents and Experts over Time

## Problem

RE-Bench asks how frontier agents compare with human experts on open-ended machine-learning research engineering when both work in closely matched environments.

## Core mechanism

Seven environments provide a scoring function, starting solution, compute, and reference solution. Agents use Modular or AIDE scaffolds. Human evidence includes 71 eight-hour attempts by 61 experts. This benchmark evaluates work under fixed tasks; it does not update models or harnesses.

## Reported evidence

The authors compare score@k using the best observed allocation across independent attempts. Agents score about four times humans at two total hours; humans narrowly exceed agents at eight total hours and score about twice agents at 32 total hours across different attempts, not one continuous 32-hour run. Harp did not reproduce this.

## Key limitation

The source says seven short, self-contained environments, fast feedback, small samples, score access, elicitation work, and unequal costs limit external validity. Harp's boundary is that score@k compares selected independent attempts, not sustained autonomous research.

## Why Weng cites it

Weng uses RE-Bench to show that early agent speed does not imply sustained long-horizon research competence.

---
source_id: AHE
title: "Agentic Harness Engineering: Observability-Driven Automatic Evolution of Coding-Agent Harnesses"
weng_locator: reference-37
section_id: self-improving-harnesses
primary_url: https://arxiv.org/abs/2604.25850
captured_path: evidence/weng/text/ahe.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: harness-code-search
claim_ceiling: Inspected §§2–5 and Limitations protocol, author-reported results, transfer, component ablations, and prediction calibration; no independent reproduction
lesson_ids: 0006,0010
card_path: content/weng-sources/ahe.md
canonical_route: content/systems/ahe.md
---

# Making Harness Edits Observable

## Problem

Captured lines 89–115 argue that automatic harness evolution needs to locate failures in components and state a falsifiable prediction for every edit.

## Core mechanism

AHE exposes prompts, tools, middleware, skills, subagents, and memory as files. Debuggers distill traces into failure evidence; an evolve agent proposes file-level edits and predicted effects. Runs, tracer, verifier, and model configuration are read-only. The base model stays fixed.

## Reported evidence

The authors select the 77.0% best configuration from one ten-iteration Terminal-Bench 2 campaign. Pass@1 is the mean over two rollouts for each of 89 tasks; the seed scores 69.7%. They also report frozen transfer and component ablations. Harp did not reproduce this.

## Key limitation

The source reports regression-prediction precision of 11.8% and recall of 11.1% across nine evaluation rounds, plus component interference and timeout coupling. Harp's boundary is that one selected campaign and transfer panel do not prove successor or recursive improvement.

## Why Weng cites it

Weng lines 240–257 uses AHE to teach component, experience, and decision observability, plus the rule that evaluator and model controls must remain outside editable harness code.

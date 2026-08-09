---
source_id: NOT-SCIENTISTS
title: "Why LLMs Aren't Scientists Yet: Lessons from Four Autonomous Research Attempts"
weng_locator: reference-28
section_id: future-challenges
primary_url: https://arxiv.org/abs/2601.03315
captured_path: evidence/weng/text/not-scientists.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: automated-research-system
claim_ceiling: Inspected §§1–6 four-attempt workflow and outcomes, human intervention, six failure modes, and limitations; case report, no independent reproduction
lesson_ids: 0009,0010
card_path: content/weng-sources/not-scientists.md
canonical_route: content/weng/09-future-challenges.md
---

# Learning from Four Research Attempts

## Problem

Captured lines 20–31 ask how far current models can take an idea to a paper with minimal scaffolding and basic file and search tools.

## Core mechanism

Six agents cover idea generation, hypotheses, planning, output evaluation, revision, and paper outlining in a shared repository. Claude Code implements experiments and drafts papers. Humans select ideas, terminate many failures, provide credentials, and edit final prose; no model weights improve.

## Reported evidence

This is a four-attempt case report, not a controlled benchmark. Three attempts failed during implementation or evaluation. One completed paper entered Agents4Science 2025, passed a code audit, and was accepted among 48 of 254 valid submissions. Harp did not reproduce the execution or review outcome.

## Key limitation

The authors state that humans selected ideas, stopped failures, edited prose, and intervened on degenerate results. Section 6 says four ideas, single implementation runs, and qualitative failure coding prevent statistical conclusions. Harp's boundary is that one success is case evidence, not a benchmark rate.

## Why Weng cites it

Weng lines 317–324 uses the four attempts to ground recurring failure modes and the gap between executable paper production and reliable scientific judgment.

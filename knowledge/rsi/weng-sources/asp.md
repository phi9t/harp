---
source_id: ASP
title: Anchored Self-Play for Code Repair
weng_locator: reference-3
section_id: system-being-improved
primary_url: https://openreview.net/forum?id=lTbBFAoPSA
captured_path: evidence/weng/text/asp.txt
publication_state: ICML 2026 reported by anchor; official status not verified
evidence_state: card-complete
edited_object_family: model-self-play-or-weight-adaptation
claim_ceiling: Sections 3–9 anchored generator–fixer self-play mechanism, BUGSOURCEBENCH author-reported fix rates, ablations, and stated limitations; no independent reproduction
lesson_ids: 0001,0010
card_path: knowledge/rsi/weng-sources/asp.md
canonical_route: knowledge/rsi/weng/01-system-being-improved.md
---

# Anchoring a Generator–Fixer Curriculum

## Problem

The paper asks whether a model can expand scarce code-repair supervision by inventing and fixing bugs. Captured lines 8–35 and 61–112 identify a failure: unit tests verify behavior, but self-play drifts toward unrealistic bugs.

## Core mechanism

Captured lines 267–285 train one model to generate test-failing Python functions and repair them from test output. Lines 386–460 anchor training by mixing reference bugs into fixer examples and rewarding generated edits similar to reference edits. Training updates model weights.

## Reported evidence

BUGSOURCEBENCH fixes tasks and tests while varying human bugs, incorrect GPT-5-mini solutions subsequently edited by human annotators, and incorrect Qwen2.5-Coder-7B-Instruct or GPT-OSS-20B solutions, at lines 197–230 and 462–520. On 127 held-out BUGSOURCEBENCH tasks, lines 523–552 report standard self-play averaging 29.1% fix rate and Anchored Self-Play averaging 36.1%, +7.0 pp / 24% relative, for a Qwen2.5-Coder-7B-Instruct generator-fixer; ablations appear at 571–632. Harp did not reproduce these results.

## Key limitation

Captured lines 694–724 limit evidence to Python function repair with tests, excluding repository-scale work. A frozen embedding model and finite reference pool supply realism; lines 606–632 show pool composition and size steer results.

## Why Weng cites it

Weng’s introduction at evidence/weng/text/weng-harness.txt lines 24–30 places self-play and weight improvement beside recursive self-improvement but outside the harness focus. Anchored Self-Play is not a harness example.

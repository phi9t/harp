---
source_id: META-HARNESS
title: "Meta-Harness: End-to-End Optimization of Model Harnesses"
weng_locator: reference-9
section_id: context-engineering
primary_url: https://arxiv.org/abs/2603.28052
captured_path: evidence/weng/text/meta-harness.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: context-management-mechanism
claim_ceiling: Outer-loop mechanism, filesystem/archive design, fixed base-model and proposer setup, author-reported search and held-out results, and experiment scope; no accepted harness tested as a better later-harness producer, matched-successor result, or RSI efficacy claim; no independent reproduction
lesson_ids: 0004,0010
card_path: content/weng-sources/meta-harness.md
canonical_route: content/systems/meta-harness.md
---

# Searching Executable Context Policy

## Problem

Captured lines 119–147 argue that harness search can generate millions of diagnostic tokens, too much for a fixed summary prompt. The target is executable context-management code, not model weights.

## Core mechanism

A coding-agent proposer navigates a filesystem containing prior harness source, traces, scores, and state. It writes candidate programs; an external loop evaluates and archives them and retains a Pareto frontier. Test results stay hidden in experiments with separate search and test sets. The task model is frozen where specified.

## Reported evidence

After a 40-candidate search over three text-classification datasets, candidates were selected solely by search-set performance and tested on three held-out test sets. The authors report 48.6% average accuracy versus ACE at 40.9%, using 11.4K versus 50.8K context tokens. Harp did not reproduce this.

## Key limitation

The source fixes each domain's base model and uses Claude Code with Opus 4.6 as proposer. TerminalBench-2 is an exception to held-out evaluation: search and final evaluation use the same 89 tasks. Harp's boundary is that no accepted harness is tested as a better producer of later harnesses.

## Why Weng cites it

Weng lines 138–150 uses Meta-Harness to move from context artifacts and skills to code that decides what experience to store, retrieve, and present.

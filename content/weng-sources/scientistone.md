---
source_id: SCIENTISTONE
title: "ScientistOne: Towards Human-Level Autonomous Research via Chain-of-Evidence"
weng_locator: reference-11
section_id: workflow-design-and-search
primary_url: https://arxiv.org/abs/2605.26340
captured_path: evidence/weng/text/scientistone.txt
publication_state: preprint
evidence_state: card-complete
edited_object_family: automated-research-system
claim_ceiling: Sections 3–6 and 9 Chain-of-Evidence standard, three-stage research workflow, author-reported integrity audit results, and stated limitations; no independent reproduction
lesson_ids: 0005,0010
card_path: content/weng-sources/scientistone.md
canonical_route: content/weng/05-workflow-design-and-search.md
---

# Building Research Around Evidence Chains

## Problem

Captured lines 7–20 and 37–64 argue that polished autonomous-research papers can hide fabricated citations, irreproducible scores, and method descriptions that disagree with code because ordinary reviews do not trace claims to evidence.

## Core mechanism

The Chain-of-Evidence standard at lines 136–163 defines evidence requirements for citation, numerical, methodological, and conclusion claims. ScientistOne’s three stages at lines 164–224 turn those requirements into artifacts: the Problem Investigator retrieves full papers, the Discovery Engine preserves evaluator outputs, and the Paper Writer tags claims before a Claim Verifier checks them. Lines 205–240 define four post-hoc integrity audits.

## Reported evidence

Table 1 at lines 303–320 covers 75 papers, 15 per system across five Automated Design of Research Systems tasks. Score verification reruns code; specification-violation review checks task-rule breaches; reference verification resolves bibliography entries; method-code alignment compares paper claims with code. ScientistOne reports 12/12 score matches after excluding three hardware-sensitive expert-parallel load balancing (EPLB) for mixture-of-experts papers, 0/15 specification violations, 0/337 hallucinated references, and 14/15 method-code alignment. Human reviewers examined flags from the first three audits and removed audit false positives before Table 1 counts; method-code judgments received sampled validation. Harp did not reproduce these results.

## Key limitation

Section 9 at lines 586–622 limits validation to systems optimization, does not verify citation entailment, and does not bound audit false negatives. Method-code counts remain language-model majority votes with sampled human validation.

## Why Weng cites it

Weng lines 152–155 uses ScientistOne to show a handcrafted research workflow organized around verifiability rather than search alone.

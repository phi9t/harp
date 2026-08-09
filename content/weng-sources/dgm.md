---
source_id: DGM
title: "Darwin Gödel Machine: Open-Ended Evolution of Self-Improving Agents"
weng_locator: reference-23
section_id: evolutionary-search
primary_url: https://arxiv.org/abs/2505.22954
captured_path: evidence/weng/text/dgm.txt
publication_state: "ICLR 2026 reported by captured arXiv v3 paper; official proceedings record not independently verified"
evidence_state: card-complete
edited_object_family: harness-code-search
claim_ceiling: Abstract, mechanism, reported results, and explicit frozen-FM limitation
lesson_ids: 0007,0010
card_path: content/weng-sources/dgm.md
canonical_route: content/systems/dgm.md
---

# Branching an Archive of Editable Coding Agents

## Problem

Captured lines 98–131 replace proof-certified self-rewrites with empirical benchmark selection. The target is an editable coding-agent repository, not foundation-model weights.

## Core mechanism

DGM samples a coding agent from a branching archive, lets it modify its own harness code, evaluates the child on coding benchmarks, and adds variants as possible stepping stones. A protected outer controller, benchmark, and frozen foundation model govern selection.

## Reported evidence

The authors report 20.0% to 50.0% on a 200-task SWE-bench Verified evaluation, 14.0% to 38.0% on the 50-task Polyglot search subset, and 14.2% to 30.7% on full Polyglot. For Claude 3.7 transfer, Figure 4 says 59.0% while prose says 59.5%. Harp did not reproduce these results.

## Key limitation

The source says benchmark optimization may not capture robustness or safety, and treats benchmark gains as necessary but insufficient indicators of general AI development; Appendix H reports objective hacking. Harp's boundary is that downstream gain does not establish a better producer of later improvements or open-ended RSI.

## Why Weng cites it

Weng lines 285–295 says only sufficiently high performers enter the archive. The primary method instead admits children that compile and retain code-editing functionality, then uses a fixed outer loop for parent selection and evaluation. Harp preserves this discrepancy.

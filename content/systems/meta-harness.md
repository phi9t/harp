---
id: rsi-system-meta-harness
kind: concept
title: Meta-Harness
summary: Outer-loop search over executable context-management code using prior source, traces, scores, filesystem navigation, and Pareto retention.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-harness-engineering
related:
  - kind: generalizes
    target: rsi-system-ace
attachments:
  - content/context_engineering_deep_dive.md
  - knowledge/meta_harness/meta_harness_deep_dive.md
claims: []
human_review: null
---

# Meta-Harness: search over executable context policy

## Canonical route

**EVIDENCE. [META-HARNESS], §3.** A candidate harness is a stateful program
that controls what experience to store, retrieve, transform, and present. A
coding-agent proposer navigates a filesystem containing previous harness code,
execution traces, and scores, then writes another candidate.

[Open the evidence-tiered Meta-Harness deep dive](../../knowledge/meta_harness/meta_harness_deep_dive.md)
for the paper mechanism, dated project-page claims, pinned text-classification
and TerminalBench-2 implementations, experimental Harbor controller, local TRAE
proposal experiment, failure modes, and claim ceiling.

The important evidence split is:

- paper claims remain arXiv v1 author reports;
- project-page tables and trajectories remain dated first-party reports;
- pinned repositories support source behavior only; and
- the local experiment supports proposal-schema and boundary findings only.

<details>
<summary>Original sources for this mechanism</summary>

- Filesystem/search loop: [Meta-Harness, §§1 and 3](https://arxiv.org/abs/2603.28052).
- Trace and feedback ablations: [Meta-Harness, §4.1](https://arxiv.org/abs/2603.28052).
- Reported text, math, and coding results: [Meta-Harness, §4](https://arxiv.org/abs/2603.28052).
- Checked-in text: `evidence/weng/text/meta-harness.txt`.
- Dated page capture: `evidence/meta_harness/site/index.html`.
- Pinned repository: `evidence/implementations/meta_harness/snapshot/`.
- Local run receipt: `evidence/meta_harness/trae_run/receipt.json`.

</details>

## Claim ceiling

Meta-Harness supports author-reported automated harness-code search. The public
repository is cleaned paper code, not an independent reproduction. The local
TRAE iteration produced three schema-valid proposals but zero
upstream-interface-valid candidates and ran no benchmark. No inspected tier
shows that an accepted harness becomes a better producer of later accepted
harnesses under matched conditions.

## Reading routes

- [Benchmark field guide](../../knowledge/harness_benchmarks/harness_benchmark_field_guide.md)
- [Meta-Harness evidence-tiered deep dive](../../knowledge/meta_harness/meta_harness_deep_dive.md)
- [Context engineering deep dive](../context_engineering_deep_dive.md)
- [Searching for better harnesses](../chapters/harness-search.md)
- [Original paper](https://arxiv.org/abs/2603.28052)

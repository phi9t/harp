---
id: self-improving-agents-survey-gap-map
title: Self-improving agents survey - Harp gap map
type: gap-map
mode: RESEARCH PLANNING
status: active
created: 2026-08-09
updated: 2026-08-09
tags: [recursive-self-improvement, gap-map, research-planning, survey]
confidence: medium
---

# Self-improving agents survey gap map

Mode: `RESEARCH PLANNING`.

The survey hub is best used as a breadth map. Harp should promote only a small
number of linked works into source-backed deep dives when they close an
explicit gap.

## Coverage gains from this survey

| Gap | Survey contribution | Suggested Harp action |
|---|---|---|
| Foundation-model improvement was thinner than scaffold improvement. | The survey separates intrinsic generative demonstrations, intrinsic evaluative feedback, grounded executable environments, and generative world models. | Build one packet that compares self-generated data, self-evaluation, and environment-experience loops under a common promotion contract. |
| Memory improvement was under-modeled. | The hub gives memory object, memory structure, and memory processing sections with 65 entries. | Promote one memory-system implementation and one memory-benchmark paper; inspect persistence, deletion, provenance, and retrieval failure modes. |
| Tool governance was broader than current Harp examples. | The hub separates dynamic tool routing, iterative tool refinement, and autonomous tool creation. | Promote a tool-creation system only if it has runnable source or a clear tool-safety boundary. |
| Full-scaffold search has many adjacent systems beyond DGM and Meta-Harness. | The hub lists STOP, ADAS, DGM, AlphaEvolve, ShinkaEvolve, Continual Harness, Hyperagents, Harness-R1, and newer release-engineering style systems. | Compare mutable object, evaluator, archive, and promotion boundary across a short list of 3-5 systems. |
| Evaluation surfaced as its own branch. | The hub distinguishes measuring improvement from benchmarking improvement and lists mechanism and domain benchmarks. | Extend evaluator-integrity coverage with mechanism-level benchmarks such as RSI-Bench, tool-use evaluations, and agent-as-judge papers only after source capture. |

## Candidate deep dives

| Candidate | Why it matters | Current evidence | Claim ceiling until promoted |
|---|---|---|---|
| `SEAL` / Self-Adapting Language Models | Bridges self-generated experience and parameter updates. | Hub link plus existing survey paper framing. | Identity and topology only. |
| `Tool-R0` / tool-learning from zero data | Tests whether tool learning belongs under foundation-model improvement, tool governance, or both. | Hub link inventory. | Identity only. |
| `MemRL`, `EvolveMem`, or `MemoryOS` | Could give Harp a concrete memory-improvement case. | Hub link inventory and Awesome repo list. | Identity only. |
| `Harness-R1` | Directly matches Harp's executable harness-editing interest. | Hub full-scaffold entry. | Identity only. |
| `RSI-Bench` | A mechanism benchmark explicitly named for recursive self-improvement. | Hub evaluation entry and link inventory. | Identity only; needs verifier/access-policy capture before use. |
| `AstaBench` / `MLS-Bench` | Scientific and AI-building benchmark branch overlaps evaluator-integrity work. | Hub evaluation entry. | Identity only. |

## Anti-goals

- Do not import all 467 one-hop links as retained concepts.
- Do not treat GitHub code links as implementation evidence without a pinned
  source snapshot.
- Do not infer licenses for linked repositories or papers from the survey hub.
- Do not collapse foundation-model adaptation and scaffold adaptation into one
  success metric.
- Do not use hub counts as current metrics without rerunning acquisition.

## Practical next packet

The highest-value next packet is a **memory/tool improvement boundary** review:

1. choose one memory-updating system and one tool-creation system from the hub;
2. capture paper, repository metadata, README, license, and narrow source files;
3. map each system's mutable object and promotion boundary;
4. compare against Harp's existing prompt/workflow/full-scaffold cases; and
5. add only source-backed claims to `content/sources/evidence_graph.tsv`.

That packet would close a real Harp gap without expanding the corpus into an
unmaintainable bibliography mirror.

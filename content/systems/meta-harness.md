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
claims: []
human_review: null
---

# Meta-Harness: search over executable context policy

## Search object

**EVIDENCE — [META-HARNESS], §3.** A candidate harness is a stateful program
that controls what experience to store, retrieve, transform, and present. A
coding-agent proposer navigates a filesystem containing previous harness code,
execution traces, and scores, then writes another candidate.

## Outer loop

1. Select retained candidates and their evidence.
2. Let the proposer inspect source, traces, and scores with tools.
3. Generate a new executable harness.
4. Run it on evaluation tasks.
5. retain useful candidates, including cost/performance Pareto points.
6. Repeat under a fixed evaluation budget.

Raw traces provide causal clues that scalar scores omit, but they also consume
substantial context and may expose task-specific details.

<details>
<summary>Original sources for this mechanism</summary>

- Filesystem/search loop: [Meta-Harness, §§1 and 3](https://arxiv.org/abs/2603.28052).
- Trace and feedback ablations: [Meta-Harness, §4.1](https://arxiv.org/abs/2603.28052).
- Reported text, math, and coding results: [Meta-Harness, §4](https://arxiv.org/abs/2603.28052).
- Checked-in text: `evidence/weng/text/meta-harness.txt`.

</details>

## Evaluation and limits

The preprint reports a 7.7-point text-classification gain with four times fewer
context tokens, held-out math gains, and TerminalBench-2 results over compared
harnesses. It also compares access to scores, summaries, and raw traces.

The proposer, evaluator, tasks, model, and outer-loop budget remain external.
This packet contains no independent reproduction.

## Claim ceiling

Meta-Harness demonstrates author-reported automated harness-code search. It
does not show that an accepted harness becomes a better producer of later
accepted harnesses under matched conditions.

## Reading routes

- [Context engineering deep dive](../context_engineering_deep_dive.md)
- [Searching for better harnesses](../chapters/harness-search.md)
- [Original paper](https://arxiv.org/abs/2603.28052)

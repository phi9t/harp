# Harness design patterns are prerequisites, not RSI

## What Weng claims

**CLAIM — [WENG-HARNESS], “Harness Design Patterns.”** Durable agent systems
rely on explicit workflow loops, filesystem-backed persistent state, and
subagents or background jobs. The coding-agent case study combines these
patterns with tools, tests, context management, and recovery.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#harness-design-patterns).

## Mechanism

**INFERENCE.** The patterns create a substrate on which improvement can be
observed and retained:

- workflow automation makes state transitions and stopping conditions explicit;
- files provide inspectable memory, artifacts, checkpoints, and handoffs; and
- subagents provide bounded specialization and parallel work.

These mechanisms improve throughput and reliability only when ownership,
resource accounting, idempotency, and recovery semantics are explicit.

**EVIDENCE — implementation overlay.** The
[[knowledge/rsi/codex_state_continuity_and_compaction|Codex state-continuity companion]]
shows why durable files and event records are not enough by themselves: active
typed history, current world state, compacted checkpoints, and external
execution state have different recovery semantics.

## Hidden assumption

**INFERENCE.** The section assumes more execution structure improves the system
rather than merely giving it more attempts, tokens, or hidden state. A
multi-agent tree can look more capable while simply consuming a larger
unmeasured budget.

## Demonstrated versus proposed

**EVIDENCE.** Production harnesses use these patterns successfully, and their
state is concrete enough to inspect and test.

**MISSING.** A durable loop is not an improvement loop unless it proposes,
evaluates, rejects or accepts, and promotes a changed candidate. It is not
recursive unless the accepted candidate improves later improvement work.

## What would weaken this interpretation

Matched-budget comparisons that remove the apparent gain, or recovery tests
that show filesystem state cannot be replayed safely, would weaken the claim
that the pattern itself contributed capability rather than spend or accidental
state.

## Reader checkpoint

Classify a test-retry loop, a persistent failure notebook, and a child-agent
fanout as task iteration, durable adaptation, or recursive improvement.

<details>
<summary>Check your answer</summary>

A retry loop is task iteration. A failure notebook is durable adaptation when
later runs actually consume it. Child-agent fanout is an execution strategy.
None is recursive without an accepted generation edge and evidence that the
child is a better producer of later valid improvements.

</details>

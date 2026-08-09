# Self-improving harnesses require external promotion

## What Weng claims

**CLAIM — [WENG-HARNESS], “Self-Improving Harness.”** Code provides a broad
representation for prompts, tools, memory, permissions, control flow, and
subagents. STOP improves an improver program; Self-Harness uses
propose–evaluate–accept; AHE grounds edits in observable failures and protects
evaluation surfaces.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#self-improving-harness).

## Mechanism

**INFERENCE.** A bounded harness-improvement generation needs:

1. a declared parent and editable surface;
2. externally supplied failure evidence;
3. a candidate delta with a causal hypothesis;
4. fresh held-in and held-out evaluation under a complete root-tree budget;
5. integrity checks over tests, permissions, archive, and evaluator state; and
6. an external promotion decision that creates one accepted lineage edge.

Rejected candidates and negative results remain in protected archive `Aₜ`.
They are evidence, not disposable transcript tail.

## Hidden assumption

**INFERENCE.** The candidate is assumed unable to alter its judge, hidden
tests, resource accounting, permissions, archive history, or promotion
authority. A read-only instruction inside the candidate prompt is not an
enforced capability boundary.

## Demonstrated versus proposed

**EVIDENCE — [STOP], [SELF-HARNESS], [AHE].** The inspected sources support
bounded improver-code search, harness proposal/evaluation loops, observable
editable components, and author-reported gains and limitations.

**MISSING.** Multi-round task gains do not show that an accepted child is a
better producer of later accepted children under a matched envelope.

## What would weaken this interpretation

An adversarial test that lets a candidate alter evaluator files, launder child
cost, or rewrite rejected branches without invalidating promotion would break
the integrity interpretation. A successor test with no later-cycle gain would
cap the claim at persistent harness improvement.

## Reader checkpoint

What evidence distinguishes another search round from a new generation?

<details>
<summary>Check your answer</summary>

A search round evaluates another candidate under one fixed parent or
optimizer. A generation requires an externally accepted parent–child lineage
edge, immutable receipts for the changed candidate and fixed envelope, and
later execution in which the child becomes the parent. Recursion requires
evidence about that later improvement cycle.

</details>

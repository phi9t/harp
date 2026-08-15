# Future challenges are acceptance criteria

## What Weng claims

**CLAIM — [WENG-HARNESS], “Future Challenges.”** Weak evaluators, context and
memory lifecycle, negative-result retention, diversity collapse, reward
hacking, short-term objectives, and the placement of human judgment are central
obstacles to self-improving systems.

[Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#future-challenges).

## Mechanism

**INFERENCE.** Turn the challenge list into independent promotion gates:

- **outcome:** held-out behavior improves under matched conditions;
- **integrity:** evaluator, budget, permissions, source, and archive receipts
  remain valid;
- **authority:** an external actor accepts the candidate for its declared use;
- **maintainability:** the candidate does not transfer hidden cost or
  complexity to future work; and
- **successor usability:** a fresh improver can understand and safely change
  the accepted system.

Correctness, integrity, and authority should be non-tradeable. Among valid
candidates, maintainability, successor usability, diversity, and cost can
remain visible as a Pareto frontier rather than disappearing inside one score.

**INFERENCE — state continuity as an acceptance criterion.** A long-running
candidate should also prove that compaction preserves decisive facts, rejected
hypotheses, current permission context, and call/output structure across
checkpoint and replay. The
[[knowledge/rsi/codex_state_continuity_and_compaction|Codex continuity companion]]
provides the mechanism map and matched ablations for this gate.

## Hidden assumption

**INFERENCE.** Long-term costs must be observable within the evaluation
horizon. A short sandbox can reward a local special case, duplicated logic, or
partial migration while another owner or later generation inherits the debt.

## Demonstrated versus proposed

**EVIDENCE.** The article identifies the challenge classes and motivates
external evaluator and permission boundaries.

**MISSING.** It does not provide a complete scalable evaluator, a universal
time horizon, or an exchange rate between immediate capability, code quality,
future maintenance, and human oversight.

## What would weaken this interpretation

If follow-on tasks, canary generations, ownership checks, and maintenance
metrics do not predict later failures better than immediate task score, their
added complexity may not be justified in that domain. If independent gates
reject every useful candidate, the contract needs a clearer scope rather than
silent score tradeoffs.

## Reader checkpoint

Which promotion properties are hard gates, and which remain tradeoffs?

<details>
<summary>Check your answer</summary>

Security, evaluator integrity, permission compliance, complete lineage,
correctness for the declared scope, and external authority are hard gates.
Among candidates that pass them, task quality, maintainability, successor
usability, diversity, latency, and cost can remain explicit tradeoffs. The
chosen policy must be precommitted and visible in the decision receipt.

</details>

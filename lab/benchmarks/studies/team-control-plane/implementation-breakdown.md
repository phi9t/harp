# Proposed implementation breakdown

Status: overview; canonical detailed plan and blocker-linked tickets now written
Source commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`
Contract: [SPEC.md](SPEC.md)

The high-level goal is a recoverable engineering team with replaceable reasoning
processes and evidence-bound acceptance. This graph makes the first useful
result a durable, fault-tested control plane rather than an issue UI.

| ID | Outcome | Prerequisites | Acceptance evidence |
| --- | --- | --- | --- |
| A | Resolve language, Git/broker authority, public test seam, budget and first-slice policy | Existing decision tickets 03–09; this draft | Recorded owner decisions; pinned SDK compile/replay feasibility result; no competing canonical tracker |
| B | Evaluate Kata reuse and freeze planning adapter boundary | A authority boundary | Pinned-release API/revision/auth/event/closure tests and adopt/defer decision; no tracker migration implied |
| C | Domain kernel with PostgreSQL admission and command idempotency | A | Concurrent claims, revision/generation rejection, atomic budget/attempt/journal/outbox, crash transaction tests |
| D | Immutable artifacts and verification acceptance | C | Publication crash tests, exact candidate/spec/base matching, missing-object rejection, fixture integration handoff |
| E | Fault runtime and exclusive gateway dispatch journal | C | Lost acknowledgment, delayed old owner, duplicate dispatch, unknown thread creation and quarantine scenarios |
| F | Temporal mission/task lifecycle and DB bridge | C, E | Replay, Activity retries, signal loss, command cursor gaps, Continue-As-New and terminal-generation tests |
| G | Scoped team tools, mailbox, yield and reconstructible lead context | C, F | Authenticated sender, stale worker rejection, presentation/ack distinction, no-poll yield and lead replacement |
| H | Attempt environments and independently supervised Execution handles | C, E | Nested capsule timeout/cancel, survivor inventory, GC registration race and workspace reuse rejection |
| I | Durable kernel proof and basic operator CLI | D, F, G, H | Public API end-to-end failure matrix, cumulative budgets, restart proof and precise stopped states; real PG/Temporal |
| J | Planning integration if Kata evaluation passes | B, I | Approved snapshot binding, duplicate-safe projection, no close/claim authority bypass; otherwise keep Git adapter |
| J2 | Evaluate optional jj workspace backend | I, H, D | Fixture stale/divergent/crash cases, immutable proof after rewrite, compatible Git export and fast-forward handoff; adopt/defer decision |
| K | Measured live Trae adapter | I; runtime probe decision 04 | Pin version/schema, probe start/resume/steer/cancel/history/store identity; distinguish observed support from unknown guarantees |
| L | Codex and Claude lead demonstrations | K, G | Same baseline, separate exact evidence, lead/TUI/runtime failure recovery and verified integration candidate |
| M | Upgrade, backup/restore and rollout handoff | I, K, L; J if selected | Replay compatibility, restore with surviving job, referenced artifact retention, approved configuration and human handoff |

C–I form the first implementation milestone. K–L establish the real multi-agent
reference. M establishes readiness for persistent use. B can run alongside
kernel work after A, but custom issue UI work waits for its reuse decision.
Independent reviewers assess protocol and specification evidence at each
milestone. Landing/committing/deployment retain their existing authorization
requirements. The owner requested detailed planning after specification review.
[PLAN.md](PLAN.md) and tickets 10–25 now provide canonical decomposition;
existing decision tickets remain open until their acceptance is actually met.

J2 is independent of Kata adoption and never blocks the durable Git baseline.
A selected jj backend must pass its own gates before entering live demonstrations.

Detailed review added explicit optional jj implementation ticket 25 and rollout
prerequisites for selected adapters; deferred adapters resolve with exclusions.

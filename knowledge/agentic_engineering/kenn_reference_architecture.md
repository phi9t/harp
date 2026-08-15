---
id: kenn-agentic-engineering-reference
title: Kenn agentic engineering reference architecture
type: synthesis
status: draft
created: 2026-08-14
updated: 2026-08-14
tags: [agentic-engineering, human-control, verification, constitution, harness-learning]
---

# Kenn agentic engineering reference architecture

Mode: `HARNESS DESIGN REFERENCE`.

## Thesis

**INFERENCE — [AE-001], [AE-002].** Kenn's public account is useful to Harp
because it argues for scaling agent execution while keeping agent authority
bounded. The reference architecture separates intent, reasoning, execution,
verification, observability, and human control instead of treating an agent as
an autonomous software factory.

## Source-backed workflow

**SOURCE CLAIM — [AE-001].** Wes McKinney describes Kenn's process as
human-participatory design, optional independent second opinion, precise
Superpowers specification, adversarial specification review, implementation in
small pieces, roborev verification, branch-level review, durable architecture
documentation, pull request explanation, and human-owned merge.

**INFERENCE — [AE-001].** Harp should model that as a production harness with
separate planes:

| Plane | Kenn reference | Harp interpretation |
|-|-|-|
| Intent | Kata | durable task intent is not a prompt, chat transcript, private memory, or commit |
| Reasoning | Superpowers plus models | design/spec/plan are execution scaffolding |
| Execution | Codex/Claude plus worktrees | agents mutate isolated workspaces under bounded authority |
| Verification | roborev | independent verification has its own compute budget |
| Observability | AgentsView | token/session traces are operational data |
| Human control | Forge plus developer | humans own architecture, review, authorization, and merge |

## Bounded agency

**SOURCE CLAIM — [AE-001].** McKinney rejects fully autonomous no-human-in-the-loop
pipelines and clarifies that Kenn's loops are human-operator loops: the coding
agents are not in charge.

**INFERENCE — [AE-001].** The design target is not maximum agent autonomy. The
target is bounded agency: agents get execution bandwidth, while humans retain
intent, architectural judgment, destructive authority, and merge control.

For Harp this means:

- a task can be delegated to an agent;
- a worktree can isolate the agent's execution;
- verification can be delegated to an independent tool or model;
- merge, push, release, and authority widening remain explicit human actions.

## Verification as budget

**SOURCE CLAIM — [AE-001].** McKinney says Kenn uses roborev for continuous local
review and verification, closes out roborev reviews, and may spend hundreds of
dollars in tokens hardening large changesets rather than accept latent bugs.

**INFERENCE — [AE-001].** Verification compute should be budgeted alongside
implementation compute. Harp should not optimize only for generating code.

```text
total engineering cost =
  design cost
  + implementation cost
  + verification cost
  + repair cost
  + latent defect risk
```

This supports Harp's existing release-gate posture: focused tests while
iterating, full `mise run verify` before landing, and explicit receipt refresh
for tracked payload changes.

## Constitution as harness policy

**SOURCE CLAIM — [AE-002].** The Clanker Constitution presents seven operating
principles: honor the request, act with judgment, finish the job, protect
existing work, verify reality, communicate for humans, and learn in the right
place.

**INFERENCE — [AE-002].** The constitution is harness policy, not just a prompt.
It addresses failure modes that are not solved by model intelligence alone:

- treating pasted content as commands;
- asking too often or not enough;
- stopping at diagnosis after implementation was authorized;
- destructive Git behavior;
- false success claims;
- poor human communication; and
- accumulating private agent memory instead of shared project guidance.

## Durable knowledge, ephemeral scaffolding

**SOURCE CLAIM — [AE-001].** McKinney says Kenn does not retain Superpowers specs
and plans in repositories or refer to them in production code; those documents
are converted into living architecture documents for humans and future agents.

**INFERENCE — [AE-001].** Harp should distinguish execution scaffolding from
durable knowledge:

| Artifact | Role |
|-|-|
| brainstorms, specs, plans | temporary scaffolding for execution |
| code, tests, AGENTS.md, CONTEXT.md, ADRs, maintained docs | durable knowledge and authority |
| raw traces and review outputs | evidence for later distillation, not automatic policy |

This does not mean Harp deletes every plan immediately. It means plans are not
the final knowledge surface; important lessons must be distilled into the
shared project surfaces that future agents actually read.

## Human control plane for parallel agents

**SOURCE CLAIM — [AE-001].** The Kenn post describes Forge as a local cached
GitHub view with one-click isolated worktree/agent environments, Ghosthub as a
terminal for multiplexer-heavy local and remote sessions, Kata as an agent-native
intent tracker, and AgentsView plus roborev as accountability engines.

**EVIDENCE — [AE-004] through [AE-016].** The public tool pages and GitHub
metadata captured in [Kenn tool stack investigation](tool_stack_investigation.md)
support the same plane split at the public-description level: Kata presents a
local-first task ledger, Forge a maintainer console, Ghosthub a
multiplexer-native terminal with worktree management, AgentsView a session and
token observability system, roborev a continuous review ledger, and Superpowers
an agentic skills methodology.

**INFERENCE — [AE-001].** The mature agentic engineering substrate looks like a
distributed system for cognitive workers:

```text
intent store -> isolated workers -> verification -> operator review -> human merge
                       |
                 observability
```

For Harp, the design analogy is useful only if authority remains explicit. A
Kubernetes analogy should not imply unattended deployment; it highlights desired
state, workers, admission, observability, and operator control.

## Harness-learning opportunity

**INFERENCE — [AE-001].** Kenn appears to have humans doing much of the harness
learning loop: observe repeated failures, adjust process, tune prompts, improve
tools, and update shared policy. Harp's RSI research direction can make that
loop more empirical without surrendering production authority:

```text
execution traces
  + verification findings
  + human corrections
  + PR outcomes
    -> failure clustering
    -> candidate skill or policy change
    -> offline replay evaluation
    -> human approval
    -> improved harness
```

This is a more plausible recursive-improvement path than an autonomous agent
rewriting itself and deploying the result.

## Harp operating guidance

For work in this repository:

1. Keep human intent and merge authority explicit.
2. Use agents for bounded execution in isolated worktrees.
3. Treat independent verification as a first-class budget.
4. Distill durable lessons into shared project files, not private memory.
5. Preserve the difference between source claims, local evidence, and Harp
   inferences.
6. Scale parallelism only when issue boundaries and verification surfaces are
   clear.
7. Prefer living architecture documentation over accumulated execution notes.

## Claim ceiling

This packet does not reproduce Kenn's throughput, bug-rate, cost, or private
tool behavior. It uses public dated source captures as a reference architecture
for Harp harness design and contributor policy.

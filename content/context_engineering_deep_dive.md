---
id: recursive-self-improvement-context-engineering-deep-dive
title: Context engineering from artifacts to learning procedures
type: technical-deep-dive
mode: TECHNICAL DEEP DIVE
status: active
created: 2026-08-02
updated: 2026-08-02
tags: [recursive-self-improvement, context-engineering, ace, mce, meta-harness, memory, skills]
confidence: medium
---

# Context engineering: from learned artifacts to learned learning procedures

Mode: `TECHNICAL DEEP DIVE`.

> This companion expands Weng's
> [context-engineering section](https://lilianweng.github.io/posts/2026-07-04-harness/#context-engineering)
> through the inspected ACE, MCE, and Meta-Harness primary-paper captures. The
> systems form a useful conceptual ladder, but they do not share one evaluator,
> budget, or experimental protocol and should not be read as a leaderboard.

## Context engineering reading map

Use this companion in five passes:

1. **Anchor Weng's claim.** Read Weng's
   [original context-engineering section](https://lilianweng.github.io/posts/2026-07-04-harness/#context-engineering),
   then use the core thesis below to identify the object being optimized.
2. **Build the runtime model.** Read “Minimal mental model” to separate current
   truth, working state, episodic history, learned knowledge, construction
   policy, and durable execution state.
3. **Trace the learning ladder.** Read
   [ACE](https://arxiv.org/abs/2510.04618) for artifact updates,
   [MCE](https://arxiv.org/abs/2601.21557) for a learned context-learning skill,
   and [Meta-Harness](https://arxiv.org/abs/2603.28052) for search over
   executable context policy.
4. **Cross learning with runtime.** Use “Runtime operations versus learned
   objects” and the
   [Codex state-continuity companion](codex_state_continuity_and_compaction.md)
   to distinguish learning useful context from preserving and activating it.
5. **Design a falsifier.** End with the experimental plan and claim ceiling.
   A task-score gain is not enough; the experiment must isolate the artifact,
   constructor, consumer, evaluator, and total resource budget.

The primary-paper route for each learning system is:

| System | Minimum source route | Why it matters |
|---|---|---|
| [ACE original paper](https://arxiv.org/abs/2510.04618) | §§2–3.2 → §§4.1–4.7 → Appendix A.3–A.6 | Failure model, Generator–Reflector–Curator loop, grow-and-refine updates, ablations, and cost. |
| [MCE original paper](https://arxiv.org/abs/2601.21557) | §§3.1–3.4 → §§4.1–4.3 → §5 → Appendix A | Bi-level objective, agentic crossover, base-level execution, fixed-versus-evolved skill comparison, and limitations. |
| [Meta-Harness original paper](https://arxiv.org/abs/2603.28052) | §3 and Algorithm 1 → §§4.1–4.3 → Appendix A.1–A.2 | Code-space search, filesystem feedback, Pareto retention, trace ablation, and proposer behavior. |

The checked-in source texts are `evidence/weng/text/ace.txt`,
`evidence/weng/text/mce.txt`, and
`evidence/weng/text/meta-harness.txt`.
The source registry records their publication state, capture digest, and claim
ceiling. The paper results below remain author-reported unless explicitly
marked otherwise.

## Core thesis

1. **CLAIM — [WENG-HARNESS], “Context Engineering.”** Weng presents context
   engineering as an increasingly general optimization surface: ACE updates
   structured context, MCE evolves skills that construct and update context,
   and Meta-Harness searches the code that governs the full context lifecycle.
   [Read the original section](https://lilianweng.github.io/posts/2026-07-04-harness/#context-engineering).
2. **INFERENCE.** The three systems change different objects. ACE mainly changes
   persistent artifacts `Dₜ`. MCE also changes context-learning policy in `Hₜ`
   and `Rₜ`. Meta-Harness broadens the search object to executable harness code.
3. **EVIDENCE — [ACE, §§2–4](https://arxiv.org/abs/2510.04618).** ACE responds
   to brevity bias and context collapse with Generator, Reflector, and Curator
   roles, structured bullets, incremental delta updates, and grow-and-refine
   maintenance.
4. **EVIDENCE — [MCE, §3](https://arxiv.org/abs/2601.21557).** MCE formalizes a
   bi-level problem: an outer meta-agent evolves a context-engineering skill,
   while an inner base agent executes that skill to produce a context function
   represented by files and code.
5. **EVIDENCE — [Meta-Harness, §3](https://arxiv.org/abs/2603.28052).**
   Meta-Harness uses a coding-agent proposer with filesystem access to previous
   harness code, traces, and scores, then evaluates proposed harness programs
   and retains a Pareto frontier.
6. **INFERENCE.** Richer context is not automatically better context. Measure
   selected context, cache behavior, output quality, adaptation cost, and
   utilization separately.
7. **INFERENCE.** A learned context artifact can improve task performance
   without improving the procedure that learns future context. Recursion
   requires evidence about that later learning procedure.
8. **MISSING.** No matched experiment in this packet isolates artifact quality,
   learning-procedure quality, model utilization, and resource spend across all
   three systems.

## Minimal mental model

### Context is a function, not a prompt string

For task input `x`, define a context function:

`c(x) = F(x; ρ)`

where:

- `ρ` is persistent context state such as rules, examples, memories, indexes,
  scripts, and retrieval metadata; and
- `F` is the policy that selects, transforms, orders, and presents parts of
  that state for `x`.

The model then produces:

`ŷ = f_W(x, c(x))`.

This separates three questions that a monolithic prompt hides:

1. What persistent knowledge has been learned?
2. How is task-relevant context constructed from it?
3. Can the model actually use the resulting context?

### The context lifecycle

```text
task + current context state
          |
          v
      construct context
          |
          v
     model trajectory
          |
     outcome + trace
          |
          v
 diagnose -> propose delta -> validate -> merge or reject
          |
          v
 next context state
```

The model-visible prompt is one projection of a larger state. The archive
should retain the input, selected context, trajectory, outcome, proposed
change, decision, and provenance even when the next prompt is compact.

### Two axes that should not be collapsed

Context engineering has a **runtime axis** and a **learning axis**.

The runtime axis asks how one task receives context:

`archive -> retrieve -> select -> transform -> order -> render -> activate`

The learning axis asks how later tasks change that process:

`trajectory -> diagnose -> propose -> validate -> publish -> supersede`

ACE, MCE, and Meta-Harness mainly expand the learning axis. Long-running agent
harnesses also need the runtime axis to preserve working state between model
calls. A system can improve one axis while failing on the other. For example:

- a high-quality ACE bullet is useless when retrieval never selects it;
- a correct MCE skill is inert when the runtime does not activate it;
- a good compaction policy preserves a thread without learning any new rule;
- a large memory archive can grow while task-relevant selection gets worse; and
- a better final prompt does not show whether the artifact or constructor
  improved.

**INFERENCE.** “Better context” is therefore an underspecified result. A useful
report identifies which state changed, which runtime operation consumed it,
and which later outcome improved.

### Six context planes

| Plane | Concrete contents | Freshness rule | Main failure |
|---|---|---|---|
| Current truth | cwd, permissions, tools, model, instructions, active processes | Recompute from the runtime | Stale transcript state overrides reality |
| Working state | hypotheses, plan, rejected branches, pending calls, opaque continuation items | Preserve within the active lineage | Re-derivation after each action |
| Episodic history | observations, actions, tool calls, outputs, decisions | Append, normalize, checkpoint | Broken ordering or orphaned call/output pairs |
| Learned knowledge | rules, examples, memories, playbook bullets, skills | Validate, version, supersede | Context collapse, contradiction, or stale advice |
| Construction policy | retrieval, selection, ordering, rendering, compaction | Evaluate as executable policy | Correct records are omitted or badly presented |
| Durable execution state | rollout checkpoints, operation IDs, external side-effect receipts | Persist atomically and reconcile | Resume duplicates or forgets effects |

These planes need different update semantics. Current truth should usually be
recomputed, not recalled from memory. Working state needs continuity but may be
opaque. Learned knowledge needs provenance and retirement rules. Durable
execution state must survive crashes and cannot be replaced by a prose
summary.

### Context success is a pipeline

For a learned entry `d`, separate:

1. **available:** `d` exists in the authoritative archive;
2. **eligible:** its conditions match the current task and world state;
3. **selected:** retrieval and budget policy choose `d`;
4. **rendered:** the constructor presents `d` in a usable form and order;
5. **activated:** the model notices and invokes the instruction or skill;
6. **adhered:** the trajectory continues to follow it when pressure rises; and
7. **effective:** the resulting behavior improves the protected outcome.

A rough decomposition is:

`P(effect) = P(select) × P(activate | select) × P(adhere | activate) × P(success | adhere)`.

This is a diagnostic factorization, not an independence claim. It prevents an
end-to-end score from assigning every failure to artifact quality. The
[Harness Disentangle reading](systems/harness-disentangle.md) provides the
closest inspected evaluation treatment of update quality, activation,
adherence, and downstream benefit.

### Context pipeline checkpoint

Suppose a correct safety rule exists in the persistent archive, but the agent
violates it during a long task. Before revealing the categories below, locate
the earliest failed boundary:

- Was the rule eligible for the current task and world state?
- Did retrieval select it under the available budget?
- Did rendering preserve its conditions and authority?
- Did the model activate it at the relevant decision?
- Did the trajectory continue to adhere after tool feedback arrived?
- Did the rule improve the protected outcome when followed?

This question is intentionally open. “The memory failed” is not a diagnosis
until the failed boundary and its receipt are identified.

### Retention, retrieval, and compaction solve different problems

- **Retention** keeps a prior item available to the active lineage.
- **Retrieval** chooses a relevant item from a larger archive.
- **Compaction** replaces a long active history with a smaller continuation
  state.
- **Consolidation** turns episodes into reusable cross-thread knowledge.
- **Replay** reconstructs a logical state after interruption.

Conflating them creates false guarantees. Retaining everything does not make it
relevant. Retrieval does not preserve exact action order. Compaction does not
create cross-thread knowledge. Replay of a transcript does not recreate an
operating-system process or prove whether an external write completed.

The [Codex state-continuity deep dive](codex_state_continuity_and_compaction.md)
shows these boundaries in one inspected production harness: typed
`ResponseItem` history, opaque reasoning continuation, recomputed world state,
atomic compaction checkpoints, rollout replay, and a separate memories
pipeline. It is an operational companion to the ACE–MCE–Meta-Harness learning
ladder, not a fourth system in Weng's context-engineering comparison.

## ACE: structured artifact adaptation

### Failure being addressed

**EVIDENCE — [ACE, §§2.2–3](https://arxiv.org/abs/2510.04618).** ACE names two
failure modes:

- **brevity bias:** summarization favors concise abstractions and drops
  domain-specific heuristics, tool-use guidance, and failure details; and
- **context collapse:** repeated monolithic rewrites progressively erase useful
  prior information.

The failure is structural. Asking an LLM to regenerate the complete context
turns every update into a destructive rewrite of all prior knowledge.

### Generator, Reflector, Curator

**EVIDENCE — [ACE, §3](https://arxiv.org/abs/2510.04618).** ACE separates:

- **Generator:** uses the current playbook to solve tasks and emits trajectories;
- **Reflector:** analyzes successful and failed trajectories and proposes
  concrete lessons; and
- **Curator:** integrates lessons into the playbook through structured updates.

The separation matters because task execution, diagnosis, and state mutation
have different failure modes. A Generator may solve a task without explaining
why; a Reflector may infer a false lesson; a Curator may overwrite or duplicate
valid knowledge.

### Itemized playbook

**EVIDENCE — [ACE, §3.1](https://arxiv.org/abs/2510.04618).** Context is
represented as identified bullets rather than one prompt blob. This supports:

- localized updates;
- per-item usefulness or harmfulness feedback;
- fine-grained retrieval;
- deterministic merge logic; and
- later deduplication or refinement.

**INFERENCE.** Stable item identity is a provenance primitive. It lets an
experiment ask which rule was selected, which trajectory motivated a change,
and whether a later failure came from retrieval, content, or model adherence.

### Grow and refine

**EVIDENCE — [ACE, §3.2](https://arxiv.org/abs/2510.04618).** New bullets can
be appended; existing bullets can be updated; semantically redundant entries
can be deduplicated. This avoids rewriting the complete playbook on every
observation.

**INFERENCE.** Incremental mutation reduces blast radius but creates lifecycle
work:

- contradictory bullets need conflict semantics;
- stale rules need expiry or supersession;
- deduplication can merge distinctions that matter;
- counters can reward frequently used but wrong rules; and
- unbounded growth can shift cost from adaptation to every inference.

### Evidence and claim ceiling

**EVIDENCE — [ACE, §§4.6–4.7 and Appendix A](https://arxiv.org/abs/2510.04618).**
The paper reports component ablations, cost and latency analysis,
weaker-reflector experiments, harmful feedback stress tests, and an
incremental-update ablation. It reports an 86.9% average adaptation-latency
reduction in its comparison and discusses cache reuse when evaluating longer
playbooks.

### ACE reader checkpoint

If a new ACE playbook improves a held-out task, what additional experiment
would distinguish a useful learned bullet from a stronger Reflector, more
rollouts, or a larger effective context budget?

Record the intervention, frozen components, and resource match before reading
the artifact-versus-procedure ablation in the validation plan.

**MISSING.** These are author-reported results. This packet does not reproduce
the benchmarks, billing assumptions, or cache behavior. The reported benefits
also remain bounded by trace and feedback quality.

## MCE: learn the context-learning procedure

### Why ACE is still a fixed algorithm

**INFERENCE.** ACE adapts context content inside a human-designed
Generator–Reflector–Curator workflow. Its representation, update roles, and
merge semantics are mostly fixed. MCE asks whether the procedure itself should
be a search object.

### Bi-level objective

**EVIDENCE — [MCE, §3.1](https://arxiv.org/abs/2601.21557).** A skill `s`
defines how to construct and learn a context function `cₛ = (ρₛ, Fₛ)`. MCE
solves a bi-level problem:

```text
inner level: execute skill s to learn the best context function c_s
outer level: search for a skill s whose resulting context function validates well
```

This separates:

- **what is learned:** context files, rules, examples, and programs; from
- **how it is learned:** the skill's instructions, scripts, resources, and
  update procedure.

The analogy is to separating model parameters from architecture and training
algorithm, while keeping the underlying foundation model frozen.

### Skill as executable representation

**EVIDENCE — [MCE, §§2–3](https://arxiv.org/abs/2601.21557).** A
context-engineering skill is a folder containing instructions, scripts, and
resources. The base agent receives the skill, training rollouts, the prior best
context, and a workspace. It can build context as files and code rather than
only a flat text list.

**INFERENCE.** This representation increases expressivity:

- retrieval indexes can replace static inclusion;
- scripts can calculate or validate context;
- progressive disclosure can load detail only when relevant;
- task-specific structures can emerge; and
- prior context can be updated rather than rebuilt.

It also increases attack surface. Executable context machinery must be subject
to permissions, deterministic evaluation, dependency policy, and provenance.

### Agentic crossover

**EVIDENCE — [MCE, §3.2](https://arxiv.org/abs/2601.21557).** The meta-agent
reads prior skills, their generated context functions, and evaluation metrics.
It synthesizes a new skill by selectively combining and refining earlier
mechanisms rather than applying one fixed textual recombination rule.

**INFERENCE.** “Crossover” here is deliberative source inspection and code/skill
composition. Its quality depends on archive visibility and causal
interpretability. If the meta-agent sees only final scores, it may combine
correlated mechanisms without understanding why they worked.

### Base-level optimization

**EVIDENCE — [MCE, §§3.3–3.4](https://arxiv.org/abs/2601.21557).** The base
agent executes the current skill, learns from rollouts, updates the context
function, and is evaluated on a validation set. The resulting skill, context,
and metrics enter the skill database for later iterations.

### Evidence and claim ceiling

**EVIDENCE — [MCE, §4.3](https://arxiv.org/abs/2601.21557).** The paper
compares no-skill, fixed-skill, and evolving-skill variants. On the named FiNER
offline setting, the full system is reported above the skill-less and
fixed-skill variants. The paper also states that iterative skill evolution is
unavailable in a single-pass online setting.

### MCE reader checkpoint

An evolved MCE skill and the context files produced by that skill change
together. Which crossed evaluations would let you attribute a gain to the
learning procedure rather than the resulting artifact?

Keep the base model, task split, evaluator, inner-loop budget, and outer-loop
candidate count visible in the proposed comparison.

**MISSING.** The paper reports task-specific experiments across five domains,
not a general proof that skill evolution improves arbitrary context learning.
The primary limitation is model capability: weak agents may not create or
execute useful higher-order skills.

## Meta-Harness: search the complete executable context policy

### Search object

**EVIDENCE — [Meta-Harness, §3](https://arxiv.org/abs/2603.28052).** A harness
is a stateful program that controls what the fixed model sees and how state
changes after each interaction. The search object can include storage,
retrieval, prompt construction, tool logic, and update algorithms.

This is broader than optimizing one playbook or one learning skill.

### Filesystem as feedback channel

**EVIDENCE — [Meta-Harness, §§1 and 3](https://arxiv.org/abs/2603.28052).**
Each evaluated candidate receives a directory containing code, execution
traces, model interactions, and scores. A coding-agent proposer navigates the
growing filesystem with tools, diagnoses failures, and writes new harness
programs.

The filesystem can exceed the proposer's context window. Selective access lets
the proposer inspect raw evidence without putting the entire archive in every
prompt.

### Minimal outer loop

```text
initialize valid harness population
for each search iteration:
    proposer inspects any prior code, traces, and scores
    proposer writes one or more candidate harnesses
    evaluator executes candidates on search tasks
    archive stores code, trajectories, scores, and failures
    update valid population and Pareto frontier
return frontier
```

**EVIDENCE — [Meta-Harness, Algorithm 1](https://arxiv.org/abs/2603.28052).**
Parent selection is not hard-coded; the proposer may inspect any retained
candidate. Accuracy and context cost can be treated as multiple objectives.

### Why raw traces matter

**EVIDENCE — [Meta-Harness, §4.1, Table 3](https://arxiv.org/abs/2603.28052).**
The paper compares scores-only, scores-plus-summary, and full filesystem
access. It reports that access to raw execution traces materially improves
harness search over the more compressed conditions.

**INFERENCE.** A final score says whether a candidate worked. A trace can expose
where storage, retrieval, presentation, or model adherence failed. This is a
credit-assignment advantage, not proof that every trace should remain
model-visible.

### Evidence and claim ceiling

**EVIDENCE — [Meta-Harness, abstract and §4](https://arxiv.org/abs/2603.28052).**
The paper reports text classification, retrieval-augmented math, and
TerminalBench-2 results, including accuracy–context tradeoffs and transfer
tests.

### Meta-Harness reader checkpoint

Raw traces can improve diagnosis while also exposing search-set details and
increasing proposer context. What matched control would show that trace
*structure*, rather than extra information volume or extra model work, caused
the search gain?

**MISSING.** The source is a preprint and the packet contains no independent
reproduction. The search uses a strong coding-agent proposer, domain skills,
finite search tasks, and evaluator feedback. A discovered harness is not
evidence of a recursively improved search operator.

## One ladder, three different mutable surfaces

| Axis | ACE | MCE | Meta-Harness |
|---|---|---|---|
| Primary mutable object | Structured context bullets | Context-learning skill plus context files/code | Full task-specific harness program |
| Main archive | Playbook and trajectory feedback | Skill database, context functions, metrics | Candidate code, raw traces, scores |
| Proposer structure | Fixed Generator–Reflector–Curator | Evolving meta-skill plus base agent | Coding agent with filesystem tools |
| Update granularity | Bullet delta | Skill directory and context-function edits | Arbitrary valid code edit |
| Main evaluator question | Did the new playbook improve task output? | Did this skill learn a better validating context function? | Did this harness improve reward/cost frontier? |
| Attribution strength | Highest of the three when one bullet changes | Harder: skill and learned context co-vary | Hardest: broad code changes and large search space |
| Honest claim ceiling | Context artifact adaptation | Bi-level context-procedure adaptation | Automated harness-code search |

## Runtime operations versus learned objects

The system papers and the runtime lifecycle can be crossed directly:

| Operation | ACE | MCE | Meta-Harness | Runtime continuity concern |
|---|---|---|---|---|
| Store | Identified playbook bullets | Files, code, prior contexts, skills | Candidate source, traces, scores | Preserve typed order and provenance |
| Retrieve | Bullet or playbook selection | Skill-defined lookup and progressive disclosure | Proposer filesystem inspection | Bound recall by relevance and permissions |
| Transform | Reflector proposes lessons | Base agent runs learned scripts and instructions | Candidate code can implement arbitrary transforms | Keep source and transformed state distinguishable |
| Render | Mostly fixed playbook presentation | Skill can learn formatting and assembly | Harness code controls complete presentation | Preserve trained protocol and call/output invariants |
| Update | Curator applies local deltas | Outer agent evolves the update skill | Proposer edits executable harness code | Publish replacement state atomically |
| Compress | Deduplicate and refine bullets | Skill may invent summaries or indexes | Candidate may implement compaction | Retain decisive state and reset invalid baselines |
| Recover | Not the main paper object | Workspace and prior context are inputs | Filesystem archive supports later inspection | Replay from a checkpoint and reconcile side effects |

**INFERENCE.** The editable object gets broader from left to right, while the
number of ways to produce the same visible prompt also grows. Evaluation must
log intermediate context decisions, not only the final prompt and score.

## State and authority mapping

Using `Cₜ = (Wₜ, Hₜ, Dₜ, Rₜ)`:

- ACE bullets are mostly `Dₜ`; selection and merge policy are `Hₜ`; reflection
  belongs to `Rₜ`.
- MCE context files are `Dₜ`; the learned skill spans `Hₜ` and `Rₜ`.
- Meta-Harness candidates are broad `Hₜ` programs; the coding-agent proposer
  and its search policy belong to `Rₜ`.
- All three normally keep `Wₜ` frozen.

External evaluator `E`, resource budget `B`, permission policy `P`, protected
archive `Aₜ`, and promotion authority must remain outside the candidate.

**INFERENCE.** The same filesystem may hold candidate artifacts and evaluator
records, but capability boundaries must prevent candidate writes to scores,
held-out tasks, archive history, and promotion decisions.

## Cost and performance model

Let:

- `T_rollout` be task-execution latency;
- `T_reflect` be diagnosis and lesson extraction;
- `T_update` be artifact, skill, or code mutation;
- `T_eval` be fresh candidate evaluation;
- `N_candidates` be evaluated candidates; and
- `C_context` be selected model-visible context.

Approximate search cost:

`Cost ≈ N_candidates × (T_rollout + T_reflect + T_update + T_eval)`.

The systems shift cost differently:

- ACE reduces full-rewrite work with localized deltas but may increase the
  context carried on every task.
- MCE pays an outer skill-search cost and an inner context-learning cost.
- Meta-Harness pays for many complete candidate harness evaluations and raw
  trace storage, while allowing selective archive inspection.

Raw context length is not billed or latency cost by itself. Distinguish:

- raw persistent archive bytes;
- selected prompt tokens;
- cache-read and cache-write tokens;
- uncached prefill;
- decoded reasoning/output tokens;
- model calls and retries; and
- evaluator executions.

A longer stable prefix can be cheaper than repeatedly generated shorter
contexts when provider caching applies. That is an empirical systems property,
not a semantic-quality argument.

## Failure modes

### Artifact-level failures

- brevity bias removes rare but decisive detail;
- context collapse erodes knowledge through repeated rewrite;
- bullet growth creates contradictions and prompt bloat;
- deduplication merges rules that differ under hidden conditions;
- stale rules survive a distribution shift; and
- feedback leakage writes evaluation answers into context.

### Procedure-level failures

- a Reflector extracts a plausible but false causal lesson;
- a Curator promotes low-confidence feedback;
- a skill overfits validation metrics;
- agentic crossover combines correlated mechanisms without attribution;
- executable context code gains excessive permissions; and
- progressive disclosure fails to activate the right skill.

### Search-level failures

- more candidate evaluations masquerade as a better search algorithm;
- broad code diffs prevent causal diagnosis;
- raw traces contain protected task information;
- the proposer learns score-parser or evaluator quirks;
- Pareto objectives hide a hard correctness failure; and
- the archive retains only winners, concealing search cost.

## Implementation translation

### Typed context artifact

```text
ContextEntry:
    id
    statement or executable locator
    source evidence
    conditions and scope
    confidence
    created_by trajectory
    supersedes / conflicts_with
    usage and outcome receipts
    status: proposed | active | retired
```

### Context constructor

```text
ContextFunction:
    retrieve(query, state) -> candidate entries
    select(candidates, budget) -> selected entries
    render(selected, world state) -> model input
    observe(trajectory, outcome) -> update evidence
```

Each operation should emit a receipt:

```text
ContextReceipt:
    task and world-state digest
    archive and constructor versions
    eligible and selected entry IDs
    exclusion reasons
    rendered-context digest
    active-history checkpoint ID
    token and cache accounting
    activation and adherence observations
```

The receipt makes four superficially similar failures distinguishable:
missing knowledge, failed retrieval, failed model use, and a wrong learned
rule.

### Update transaction

1. Freeze source trajectory, evaluator identity, and current context digest.
2. Propose typed deltas with source locators and expected effect.
3. Validate schema, permissions, conflicts, and budget.
4. Evaluate old and new contexts under matched tasks.
5. Publish accepted deltas atomically; archive all rejected deltas.
6. Recompute retrieval index and receipt.

### Bi-level skill package

```text
skill/
    SKILL.md
    scripts/
    references/
    capability-manifest.json
    evaluation-contract.json
```

The capability manifest declares readable evidence, writable context paths,
allowed executables, network policy, and output schema. The skill may propose
context changes but cannot write evaluator or archive authority.

### Harness-search package

Give each candidate a separate immutable parent revision and writable
workspace. Record:

- candidate code digest;
- parent and proposer identity;
- inspected prior candidates;
- model/provider configuration;
- task split and evaluator digest;
- complete root-tree resource use;
- raw trace and selected-context digests;
- score vector and integrity findings; and
- rejection or promotion decision.

## Experimental validation plan

### 1. Artifact versus procedure ablation

Hold the learned playbook fixed and vary the context-construction procedure;
then hold the procedure fixed and vary the learned playbook. This separates
artifact quality from learning-policy quality.

### 2. ACE component ablation

Compare monolithic rewrite, incremental deltas, no Reflector, no Curator,
single versus repeated reflection, and deduplication on/off. Measure held-out
quality, playbook growth, conflicts, adaptation latency, and token cost.

### 3. MCE bi-level ablation

Compare no skill, fixed skill, evolved skill, and random/resource-matched skill
search. Use delayed validation and a task-family shift to test whether the skill
learns a reusable procedure rather than one dataset's structure.

### 4. Meta-Harness information ablation

Give the proposer scores only, scores plus summaries, selected raw traces, or
the full filesystem. Match candidate-evaluation budget. Measure search
efficiency, final frontier, repeated failure modes, and protected-data leakage.

### 5. Context utilization test

For every accepted artifact, run with the artifact available but not announced,
explicitly loaded, incorrectly selected, and fully removed. Separate retrieval,
activation, adherence, and task outcome.

### 6. Contamination and adversarial feedback

Inject misleading trajectories, conflicting bullets, held-out canaries, stale
rules, and a tempting evaluator file. Verify that confidence, conflict,
permission, and promotion gates fail closed.

### 7. Cost accounting

Report raw archive bytes, selected prompt tokens, cache reads/writes, uncached
input, output/reasoning tokens, model calls, candidate evaluations, wall time,
and dollar cost. Do not compare only final prompt length.

### 8. Later-cycle test

Promote an accepted context procedure, then give parent and child matched new
context-learning problems. Compare the distribution of valid next artifacts,
not only immediate task performance. This is the additional test required for
a recursive claim.

### 9. State-continuity 2×2

Cross reasoning/working-state retention on/off with compaction on/off. Hold
model, tasks, action budget, seeds, and tool policy fixed. Measure repeated
hypotheses, output tokens per action, early-fact recall, actions to completion,
and final score. Retention should mainly reduce reconstruction; compaction
should mainly prevent late-horizon forgetting. The combined cell tests whether
the two mechanisms are complementary.

### 10. Context-pipeline attribution

Plant typed rules with known eligibility conditions. Log availability,
selection, rendering, activation, adherence, and outcome for each rule. Then
perturb one boundary at a time: corrupt the index, reduce the selection budget,
move the rule late in the prompt, suppress skill activation, or inject a
conflicting instruction. A useful diagnostic should identify the perturbed
boundary rather than reporting one generic context failure.

## What evidence would change the claim

The context-engineering claim strengthens if:

- gains survive delayed and out-of-domain tasks;
- artifact/procedure ablations localize the cause;
- random and resource-matched search baselines lose;
- activation and adherence remain high;
- adversarial feedback and leakage tests fail closed;
- accepted procedures improve later context-learning cycles; and
- independent reproductions agree on cost and quality.

It weakens if:

- a longer prompt or stronger proposer explains the gain;
- validation data enters context state;
- selected rules are not used;
- summary-only or random search matches the method;
- broad edits cannot be causally localized;
- later cycles accumulate contradictions or debt; or
- the learned procedure does not outperform its parent on new learning tasks.

## Claim ceiling

ACE supports structured persistent context adaptation. MCE supports bi-level
adaptation of context-learning skills and context artifacts. Meta-Harness
supports automated search over executable harness programs with filesystem
feedback. Together they show a progression from learned context to learned
context-learning machinery.

They do not, in this packet, demonstrate a general autonomous recursive
improvement loop. That claim requires protected external evaluation, complete
resource accounting, accepted generations, and measured improvement of later
improvement work.

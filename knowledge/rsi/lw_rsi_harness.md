# Field notes: Lilian Weng's RSI harness map

> **Status:** Exploratory field-research note. This is a compact, personal
> map of Lilian Weng's *Harness Engineering for Self-Improvement*, not a
> canonical Harp reading and not an authority for paper-level claims. The
> maintained source-grounded interpretation is
> [RSI harness by Lil'Log, deconstructed](rsi_harness_by_lil_log_deconstructed.md).
> The captured article is [WENG-HARNESS](../../evidence/weng/artifacts/html/weng-harness.html).

## Thesis I am tracking

Weng's practical RSI claim is not “a model will soon rewrite its own weights.”
It is that an AI system can increasingly improve the **machinery around a
model**—context, tools, workflow, memory, evaluation, and eventually the
optimizer itself. That machinery is the *harness*.

The central caution is equally important: a system that edits its harness and
gets a better task score has not necessarily become better at producing its
*next* accepted improvement. Harness improvement is relevant to RSI, but it is
not synonymous with demonstrated recursive self-improvement.

## Working model

I will use the following separation while reading the post:

| Candidate that may change | Protected envelope that judges change |
| --- | --- |
| Model weights `W`, harness `H`, retained data/artifacts `D`, and internal retrospective procedures `R` | Tasks and fresh observations `X`, evaluator `E`, budget `B`, permissions `P`, archive and promotion authority `A` |

The article mostly follows improvements to `H`: the deployment-side system that
decides how a base model plans, sees context, calls tools, retains artifacts,
and verifies work. Later sections connect harness improvements to changes in
`W`, the model weights.

**Field-note rule.** A candidate may propose and execute changes, but it should
not be the final authority over the evaluator, permissions, historical
receipts, or promotion decision.

## The post at a glance

| Movement in Weng's argument | Core question | Representative systems | My reading |
| --- | --- | --- | --- |
| 1. System being improved | What counts as the AI system? | Coding-agent harnesses, Autoresearch | The unit is often model-plus-runtime, not weights alone. |
| 2. Harness design patterns | What makes an agent deployment durable? | Loops, files, subagents, jobs | These are preconditions for improvement, not proof of it. |
| 3. Harness versus core intelligence | Can external procedure become internalized? | Harness Disentangle, RLM | Keep compatibility and attribution explicit. |
| 4. Context engineering | What context object is being optimized? | ACE, MCE, Meta-Harness | Move from artifact to mechanism to executable lifecycle. |
| 5. Workflow design and search | How do we search over agent procedures? | AI Scientist, ADAS, AFlow | Search needs a representation, proposal operator, evaluator, and budget. |
| 6. Self-improving harnesses | Can the harness improve its own code? | STOP, Self-Harness, AHE | A generation needs external evidence and an external promotion gate. |
| 7. Evolutionary search | Why retain a population or archive? | AlphaEvolve, DGM | Archives preserve stepping stones; they do not align the objective. |
| 8. Joint optimization | What changes when harness and weights co-adapt? | SIA, Continual Harness | Crossed evaluations are needed before claiming interaction. |
| 9. Future challenges | What blocks safe, durable progress? | Evaluator integrity, memory, human oversight | Treat the list as acceptance criteria, not a footnote. |

## 1. The system being improved

The useful move in the post is to expand “the intelligent system” beyond model
weights. Two deployments with identical weights can differ materially because
their harnesses differ in observation, context, tools, verification, recovery,
and persistent state.

There are still two different systems to keep distinct:

| System | Immediate product | Examples |
| --- | --- | --- |
| Training pipeline | New or adapted weights | data curation, reward design, training, checkpoint selection |
| Deployment harness | Actions and artifacts from current weights | prompting, context construction, tool use, planning, memory, tests, permissions |

The interesting RSI path is the coupling: deployment traces can become data or
proposals for a training pipeline, and better models can improve the harness or
the research process that builds later models. Coupling alone is not recursion.

**Question.** If a deployed coding agent edits a training script and launches a
run, which durable artifact and external acceptance step establish that the
result is a *successor* rather than just another experiment?

Read: [system being improved](weng/01-system-being-improved.md) and
[foundation model inside the loop](chapters/foundation-model-inside-the-loop.md).

## 2. Harness design patterns are prerequisites

The article's practical examples make the harness look like ordinary systems
engineering: iterative control loops, a filesystem or repository for durable
state, tools, subagents, backend jobs, checkpoints, logs, and recovery.

Those patterns matter because an improver needs more than a prompt:

1. a place to retain candidate changes, results, and negative evidence;
2. an execution surface that can actually run and observe a candidate;
3. bounded tools and permissions; and
4. a way to resume, audit, or roll back work after interruption.

But a durable loop is not automatically an improving loop. A file archive may
only persist noise; a subagent may only multiply unmeasured cost; and a
workflow can repeat a bad heuristic more reliably than a good one.

**Field-note test.** For each design pattern, ask: *what state does it make
durable, who may modify it, and how would we observe whether it helped?*

Read: [harness design patterns](weng/02-harness-design-patterns.md)
and [durable improvement workflows](chapters/durable-improvement-workflows.md).

## 3. Harness layer versus core intelligence

An external procedure can cause behavior that the base model does not produce
alone: retrieval rules, a planning template, a tool protocol, a skill, or a
recursive call structure can all change the effective capability of the
deployed composite without changing weights.

This makes **internalization** a real but testable hypothesis. If a procedure
is causally important, compare the model with and without it under matched
tasks and budgets. If a new model is introduced, use the four-cell comparison:

```text
W_old × H_old    W_old × H_new
W_new × H_old    W_new × H_new
```

Measure discovery of the procedure or tool, activation, sustained adherence,
task outcome, cost, and integrity—not only one final score. A harness can
“improve” on paper yet outrun the model's ability to load, invoke, or follow
it.

**Question.** Which useful external procedures should remain explicit because
they provide auditability or authority, even if some of their behavior can be
learned into the model?

Read: [harness layer versus core intelligence](weng/03-harness-layer-vs-core-intelligence.md)
and [procedure internalization](chapters/procedure-internalization.md).

## 4. Context engineering: artifact, skill, mechanism

The context-engineering progression is one of the cleanest parts of the post:

```text
context artifact
  -> context-construction and update skill
  -> executable harness code for the full context lifecycle
```

- **ACE** evolves an itemized playbook from experience.
- **MCE** moves outward to evolve the skills that construct and update context.
- **Meta-Harness** moves outward again to optimize executable code that
  stores, retrieves, and presents information.

The final prompt is not the whole mechanism. A fact or lesson can exist in
storage but be ineligible, unselected, badly rendered, never activated, or
forgotten after compaction. Treat retention, retrieval, rendering, activation,
and replay as separately observable boundaries.

**Question.** What evidence distinguishes useful compression from hidden
leakage, increased context budget, or memorized benchmark answers?

Read: [context engineering](weng/04-context-engineering.md) and
[context-engineering deep dive](context_engineering_deep_dive.md).

## 5. Workflows and search

Weng moves from expert-authored research workflows toward systems that search
over workflows themselves:

- **AI Scientist** is a structured idea-to-experiment-to-paper workflow.
- **ADAS** proposes and selects agent-program designs.
- **AFlow** treats a workflow as an executable graph and searches it with MCTS.

Regardless of the search style, a workflow optimizer needs four contracts:

1. a representation for model actions and deterministic logic;
2. a proposal or mutation operator over that representation;
3. an evaluator that executes candidates under matched tasks and budgets; and
4. an archive or tree policy that allocates the next unit of search.

The searcher is not the task solver; it is a procedure for choosing a solver
configuration. A reported workflow gain can be a better allocation of model
calls, a benchmark-specific heuristic, or a reusable mechanism. Cost and
held-out evaluation decide among those interpretations.

**Question.** If a workflow beats a baseline, did it find a better procedure
or merely buy more attempts, tools, context, or evaluator knowledge?

Read: [workflow design and search](weng/05-workflow-design-and-search.md)
and [harness search](chapters/harness-search.md).

## 6. Self-improving harnesses

Code is a broad harness representation: it can encode prompts, tool adapters,
memory policy, control flow, subagents, permissions, and test logic. That
expressiveness is why systems such as STOP, Self-Harness, and AHE are
interesting—and why the integrity boundary matters.

For a claimed harness-improvement generation, I want to see:

1. a declared parent and editable surface;
2. externally supplied failure evidence;
3. a candidate delta and causal hypothesis;
4. fresh evaluation with model, evaluator, permissions, and root-tree budget
   held fixed;
5. integrity checks over tests, cost, archive, and evaluator state; and
6. an external promotion decision that establishes one accepted lineage edge.

Rejected candidates and negative results belong in the archive. They are
evidence for later diagnosis, not disposable log tail.

**Question.** Is a system showing another search *round*, or did it create an
accepted *generation* that becomes the parent of a later measured cycle?

Read: [self-improving harnesses](weng/06-self-improving-harnesses.md)
and [evaluation, promotion, and containment](chapters/evaluation-promotion-containment.md).

## 7. Evolutionary search and the DGM case

Evolution is useful when the candidate space is irregular and
non-differentiable but candidates can be executed and scored. It contributes
population, parent selection, mutation, evaluation, diversity policy, and
archive management. It does **not** supply an aligned objective by itself.

### DGM as the worked example

Darwin Gödel Machine (DGM) is useful here because it evolves an editable
coding-agent repository rather than just a task answer. Its mechanism has two
layers: a protected outer search loop and an inner self-improvement loop.

#### Outer loop: archive search

```text
archive <- {initial agent}

repeat:
    parent <- sample archive by performance and under-exploration
    target <- choose a failure or improvement target for parent
    child <- self_improve(parent, target)
    outcome <- externally evaluate child

    if child remains a valid code-editing agent:
        archive <- archive ∪ {child, outcome}

    persist lineage and generation metadata
```

The outer loop owns the archive, parent selection, candidate scheduling,
evaluation, admission, and persistence. It searches over a branching
population rather than repeatedly replacing a single incumbent.

#### Inner loop: self-improvement attempt

```text
self_improve(parent, target):
    materialize parent by replaying its lineage patches
    create an isolated working environment
    diagnosis <- turn target and parent evidence into a general improvement task
    patch <- parent edits its own coding-agent repository to address diagnosis

    if patch is empty or invalid:
        return rejected candidate

    return child(parent, patch)
```

The inner loop is self-modification only in a bounded sense: a reconstructed
parent receives an externally framed improvement task, edits its own
repository, and returns a candidate patch. The outer loop—not the candidate—
decides whether that patch is evaluated, retained, or used as a future parent.

Read: [evolutionary search](weng/07-evolutionary-search.md),
[DGM system reading](systems/dgm.md), and
[AlphaEvolve versus DGM](lessons/05-alphaevolve-vs-dgm.md).

## 8. Joint harness and weight optimization

Joint adaptation changes multiple causal surfaces at once. The harness shapes
the observations, tools, and feedback seen by the model; weight updates change
how later harness instructions are activated and followed.

Weng's examples are **SIA**, which routes feedback into harness or parameter
updates, and **Continual Harness**, which combines online harness adaptation
with model learning from teacher-labeled low-reward trajectory windows.

Before crediting an interaction, use alternating interventions and crossed
evaluation:

```text
no update    harness-only    weights-only    harness-and-weights
```

Hold tasks, evaluator, teacher identity, permissions, and root-tree budget
fixed. Then compare `W_old × H_old`, `W_old × H_new`, `W_new × H_old`, and
`W_new × H_new`. Otherwise a stronger teacher, more data, or more compute can
masquerade as a better joint improvement policy.

**Question.** When the composite improves, how much came from the harness,
weights, their interaction, or a changed data/compute budget?

Read: [joint harness and weight optimization](weng/08-joint-harness-weight-optimization.md)
and [joint harness-weight adaptation](chapters/joint-harness-weight-adaptation.md).

## 9. Future challenges as acceptance criteria

The final section should govern the rest of the post. Weak evaluation, reward
hacking, memory failure, diversity collapse, hidden long-term cost, permission
boundaries, and human judgment are not peripheral implementation details.
They determine whether an apparent improvement is trustworthy.

My working promotion checklist:

| Gate | Question |
| --- | --- |
| Outcome | Does behavior improve on held-out work under matched conditions? |
| Integrity | Are evaluator, budget, permission, source, and archive receipts valid? |
| Authority | Did an external actor accept the candidate for a declared scope? |
| Maintainability | Did the candidate transfer hidden debt or operational cost to later work? |
| Successor usability | Can a fresh improver understand, audit, and safely modify the accepted system? |

Correctness, integrity, and authority should be non-tradeable gates. Among
candidates that pass them, quality, maintainability, cost, diversity, and
successor usability can remain visible tradeoffs rather than disappearing into
one scalar score.

**Question.** Which evaluation properties are security-like hard gates, and
which are ordinary engineering tradeoffs that should remain on a Pareto
frontier?

Read: [future challenges](weng/09-future-challenges.md) and
[evaluation, promotion, and containment](chapters/evaluation-promotion-containment.md).

## Claim ladder: avoid collapsing terms

| Level | What it establishes | What it does not establish |
| --- | --- | --- |
| Task iteration | A fixed process improves an answer or action in one run. | Durable learning or harness improvement. |
| Persistent adaptation | A named artifact survives episodes and helps later behavior. | That the update mechanism itself improved. |
| Harness improvement | An executable harness policy beats its predecessor under a matched envelope. | That the accepted child produces better future harnesses. |
| Successor improvement | A matched comparison shows the child produces better later accepted changes than its parent. | Repeated, robust recursion across fresh lineages. |
| Demonstrated recursive improvement | Positive successor-production gain repeats under protected generations, fresh tasks/evaluators, and full cost/integrity accounting. | A general claim beyond the measured envelope. |

No current Harp source card qualifies at the final two levels. The full
evidence contract is in the [RSI claim ladder](../../reference/rsi-claim-ladder.html).

## Open research questions

1. What is the smallest protected evaluator that can detect improvement while
   resisting benchmark overfitting and objective hacking?
2. How should a system retain failed edits, negative experiments, and evaluator
   disagreements so later generations can learn from them?
3. Which context mechanisms truly transfer across tasks and models rather than
   functioning as hidden prompt-budget increases?
4. How can a harness expose new tools or procedures without outrunning the
   consumer model's activation and adherence capabilities?
5. How do we measure “better producer of future improvements” without
   confusing it with a single lucky child, extra budget, or evaluator leakage?
6. Where should human judgment remain indispensable: objective selection,
   evaluator changes, deployment authority, or all three?

## Source and reading route

- [Weng's original post](https://lilianweng.github.io/posts/2026-07-04-harness/)
- [Full Harp deconstruction](rsi_harness_by_lil_log_deconstructed.md)
- [Nine-section Weng reader map](../../content/weng-reading-map.json)
- [Harness comparison matrix](../../reference/harness-comparison-matrix.html)
- [Weng source cards](../../reference/weng-source-cards.html)

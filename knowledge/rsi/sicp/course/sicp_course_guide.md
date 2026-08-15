# SICP systems and agent-harness reader course

## What this course optimizes for

This course uses *Structure and Interpretation of Computer Programs* to sharpen systems judgment: how processes consume resources, where representations create boundaries, how state changes reasoning, and how evaluators and machines make semantics operational. It is a reader course for experienced programmers, not a survey of Scheme syntax or a race to complete every exercise.

The [[content/sicp/course/coverage.tsv|coverage ledger]] distinguishes deep treatment from focused treatment and required reading from skimming. The [[knowledge/rsi/sicp/course/exercise_map|exercise map]] supplies diagnostics and transfer targets without solutions.

## How to use a seminar

Read the assigned pages with the seminar's diagnostic question in view. Trace the named program or machine by hand, state an invariant before discussing implementation, and then test that invariant against the coding-agent transfer target. Record the live question and confidence state in [[knowledge/rsi/sicp/course/dialogue_state|the dialogue cursor]]; disagreement is useful when its evidence is explicit.

## Evidence voices

Use these labels when notes move beyond direct description:

- **CLAIM** — a position or mechanism asserted by a cited source, with the citation attached locally to the assertion.
- **EVIDENCE** — a cited passage, worked trace, program behavior, or other observable support.
- **INFERENCE** — a course-derived conclusion from named evidence whose reasoning can be inspected.
- **SPECULATION** — a useful possibility that currently lacks enough support to adopt.
- **MISSING** — a named evidence gap that prevents a stronger conclusion.

The labels distinguish source-backed statements from course synthesis; they do not assign truth by typography.

## Five conceptual arcs

### Procedures and immutable composition — Primer and Seminars 1-4

Procedures, data abstraction, sequence operations, and generic dispatch show how a small set of compositional rules can support large systems while preserving inspectable boundaries.

### State, mutation, concurrency, and time — Seminars 5-8

Assignment introduces identity and temporal claims; mutable data, interleavings, and delayed streams then expose the protocols needed to control effects, ordering, demand, and retention.

### Languages as executable abstractions — Seminars 9-11

Evaluators, lazy and nondeterministic execution, and query systems turn language design into explicit choices about environments, control, search, and failure.

### Machines, storage, and compilation — Seminar 12

Register machines make control state, stacks, allocation, garbage collection, and compilation concrete enough to count and test.

### Coding-agent harness synthesis — Capstone

The capstone harvests these mechanisms into a harness architecture whose context, authority, state transitions, scheduling, and evidence boundaries remain visible.

## Sequential route

Start with the [[knowledge/rsi/sicp/course/scheme_reading_primer|Scheme reading primer]], then proceed in order:

1. [[knowledge/rsi/sicp/course/seminars/01-process-shape-and-higher-order-abstraction|Seminar 1 — Process shape and higher-order abstraction]]
2. [[knowledge/rsi/sicp/course/seminars/02-data-abstraction-and-immutable-representation|Seminar 2 — Data abstraction and immutable representation]]
3. [[knowledge/rsi/sicp/course/seminars/03-trees-sequences-and-functional-interfaces|Seminar 3 — Trees, sequences, and functional interfaces]]
4. [[knowledge/rsi/sicp/course/seminars/04-symbols-sets-compression-and-generic-dispatch|Seminar 4 — Symbols, sets, compression, and generic dispatch]]
5. [[knowledge/rsi/sicp/course/seminars/05-state-identity-and-environments|Seminar 5 — State, identity, and environments]]
6. [[knowledge/rsi/sicp/course/seminars/06-mutation-simulation-and-constraints|Seminar 6 — Mutation, simulation, and constraints]]
7. [[knowledge/rsi/sicp/course/seminars/07-concurrency-serialization-and-interleavings|Seminar 7 — Concurrency, serialization, and interleavings]]
8. [[knowledge/rsi/sicp/course/seminars/08-streams-delay-and-infinite-processes|Seminar 8 — Streams, delay, and infinite processes]]
9. [[knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics|Seminar 9 — Eval/apply and executable semantics]]
10. Optional [[knowledge/rsi/sicp/agentic_eval_apply|agentic eval/apply MTS synthesis]], a
    semantic-bridge transfer supplement before the capstone, not a source or
    evaluator-semantics authority and not a replacement for Seminar 9.
11. [[knowledge/rsi/sicp/course/seminars/10-lazy-evaluation-and-nondeterministic-search|Seminar 10 — Lazy evaluation and nondeterministic search]]
12. [[knowledge/rsi/sicp/course/seminars/11-logic-programming-and-declarative-query|Seminar 11 — Logic programming and declarative query]]
13. [[knowledge/rsi/sicp/course/seminars/12-machines-storage-control-and-compilation|Seminar 12 — Machines, storage, control, and compilation]]
14. [[knowledge/rsi/sicp/course/capstone/agent_harness_architecture_dossier|Capstone — Agent-harness architecture dossier]]

## Concept routes

These routes deliberately reorder the seminars. Use them to investigate one systems problem across several abstraction levels, then return to the sequential route for dependencies the shorter route omits.

### Functional and immutable systems

Read the [[knowledge/rsi/sicp/course/scheme_reading_primer|primer]], [[knowledge/rsi/sicp/course/seminars/01-process-shape-and-higher-order-abstraction|Seminar 1]], [[knowledge/rsi/sicp/course/seminars/02-data-abstraction-and-immutable-representation|Seminar 2]], and [[knowledge/rsi/sicp/course/seminars/03-trees-sequences-and-functional-interfaces|Seminar 3]]. This route isolates process shape, representation barriers, and sequence algebra before mutation complicates equivalence.

### State and concurrency

Read [[knowledge/rsi/sicp/course/seminars/05-state-identity-and-environments|Seminar 5]], [[knowledge/rsi/sicp/course/seminars/06-mutation-simulation-and-constraints|Seminar 6]], [[knowledge/rsi/sicp/course/seminars/07-concurrency-serialization-and-interleavings|Seminar 7]], and [[knowledge/rsi/sicp/course/seminars/08-streams-delay-and-infinite-processes|Seminar 8]]. This route asks which claims survive identity, effects, interleavings, delayed demand, and retained history.

### Evaluation and language design

Read [[knowledge/rsi/sicp/course/seminars/04-symbols-sets-compression-and-generic-dispatch|Seminar 4]], then [[knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics|Seminar 9]], [[knowledge/rsi/sicp/course/seminars/10-lazy-evaluation-and-nondeterministic-search|Seminar 10]], and [[knowledge/rsi/sicp/course/seminars/11-logic-programming-and-declarative-query|Seminar 11]]. The reordered path connects data-directed dispatch to evaluation strategy, search, and declarative interfaces.

### Runtime and compiler mechanisms

Read [[knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics|Seminar 9]] before [[knowledge/rsi/sicp/course/seminars/12-machines-storage-control-and-compilation|Seminar 12]]. The first establishes evaluator semantics; the second makes their registers, stack discipline, storage, and compiled control explicit.

### Agent-harness design

Read [[knowledge/rsi/sicp/course/seminars/02-data-abstraction-and-immutable-representation|Seminar 2]], [[knowledge/rsi/sicp/course/seminars/05-state-identity-and-environments|Seminar 5]], [[knowledge/rsi/sicp/course/seminars/07-concurrency-serialization-and-interleavings|Seminar 7]], [[knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics|Seminar 9]], the optional [[knowledge/rsi/sicp/agentic_eval_apply|agentic eval/apply MTS synthesis]], and [[knowledge/rsi/sicp/course/seminars/12-machines-storage-control-and-compilation|Seminar 12]], then complete the [[knowledge/rsi/sicp/course/capstone/agent_harness_architecture_dossier|Capstone]]. The synthesis bridges semantic dispatch to external-effect boundaries without replacing Seminar 9 or the capstone; this route traces a harness from representation and authority boundaries through durable state, scheduling, semantic dispatch, external-effect boundaries, and explicit resource control.

## Resume the dialogue

Open [[knowledge/rsi/sicp/course/dialogue_state|dialogue_state.md]], follow the single `CURRENT` link to its `#dialogue` anchor, and answer the recorded question with a CLAIM plus EVIDENCE. Change understanding only after an explicit exchange supports `CONFIRMED` or `CONTESTED`; silence is not evidence and cannot advance the cursor.

## Source and license boundary

The canonical course artifact is the vendored [[evidence/sicp/sicp.pdf|SICP PDF]]. Consult the [[content/sicp/sources/source_registry.tsv|source registry]] for provenance and use constraints, and preserve the boundary described by the vendored [[evidence/sicp/LICENSE.txt|license text]]. Course notes paraphrase and point into the source; they do not replace or silently relicense it.

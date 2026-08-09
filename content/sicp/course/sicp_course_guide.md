# SICP systems and agent-harness reader course

## What this course optimizes for

This course uses *Structure and Interpretation of Computer Programs* to sharpen systems judgment: how processes consume resources, where representations create boundaries, how state changes reasoning, and how evaluators and machines make semantics operational. It is a reader course for experienced programmers, not a survey of Scheme syntax or a race to complete every exercise.

The [coverage ledger](coverage.tsv) distinguishes deep treatment from focused treatment and required reading from skimming. The [exercise map](exercise_map.md) supplies diagnostics and transfer targets without solutions.

## How to use a seminar

Read the assigned pages with the seminar's diagnostic question in view. Trace the named program or machine by hand, state an invariant before discussing implementation, and then test that invariant against the coding-agent transfer target. Record the live question and confidence state in [the dialogue cursor](dialogue_state.md); disagreement is useful when its evidence is explicit.

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

Start with the [Scheme reading primer](scheme_reading_primer.md), then proceed in order:

1. [Seminar 1 — Process shape and higher-order abstraction](seminars/01-process-shape-and-higher-order-abstraction.md)
2. [Seminar 2 — Data abstraction and immutable representation](seminars/02-data-abstraction-and-immutable-representation.md)
3. [Seminar 3 — Trees, sequences, and functional interfaces](seminars/03-trees-sequences-and-functional-interfaces.md)
4. [Seminar 4 — Symbols, sets, compression, and generic dispatch](seminars/04-symbols-sets-compression-and-generic-dispatch.md)
5. [Seminar 5 — State, identity, and environments](seminars/05-state-identity-and-environments.md)
6. [Seminar 6 — Mutation, simulation, and constraints](seminars/06-mutation-simulation-and-constraints.md)
7. [Seminar 7 — Concurrency, serialization, and interleavings](seminars/07-concurrency-serialization-and-interleavings.md)
8. [Seminar 8 — Streams, delay, and infinite processes](seminars/08-streams-delay-and-infinite-processes.md)
9. [Seminar 9 — Eval/apply and executable semantics](seminars/09-eval-apply-and-executable-semantics.md)
10. Optional [agentic eval/apply MTS synthesis](../agentic_eval_apply.md), a
    semantic-bridge transfer supplement before the capstone, not a source or
    evaluator-semantics authority and not a replacement for Seminar 9.
11. [Seminar 10 — Lazy evaluation and nondeterministic search](seminars/10-lazy-evaluation-and-nondeterministic-search.md)
12. [Seminar 11 — Logic programming and declarative query](seminars/11-logic-programming-and-declarative-query.md)
13. [Seminar 12 — Machines, storage, control, and compilation](seminars/12-machines-storage-control-and-compilation.md)
14. [Capstone — Agent-harness architecture dossier](capstone/agent_harness_architecture_dossier.md)

## Concept routes

These routes deliberately reorder the seminars. Use them to investigate one systems problem across several abstraction levels, then return to the sequential route for dependencies the shorter route omits.

### Functional and immutable systems

Read the [primer](scheme_reading_primer.md), [Seminar 1](seminars/01-process-shape-and-higher-order-abstraction.md), [Seminar 2](seminars/02-data-abstraction-and-immutable-representation.md), and [Seminar 3](seminars/03-trees-sequences-and-functional-interfaces.md). This route isolates process shape, representation barriers, and sequence algebra before mutation complicates equivalence.

### State and concurrency

Read [Seminar 5](seminars/05-state-identity-and-environments.md), [Seminar 6](seminars/06-mutation-simulation-and-constraints.md), [Seminar 7](seminars/07-concurrency-serialization-and-interleavings.md), and [Seminar 8](seminars/08-streams-delay-and-infinite-processes.md). This route asks which claims survive identity, effects, interleavings, delayed demand, and retained history.

### Evaluation and language design

Read [Seminar 4](seminars/04-symbols-sets-compression-and-generic-dispatch.md), then [Seminar 9](seminars/09-eval-apply-and-executable-semantics.md), [Seminar 10](seminars/10-lazy-evaluation-and-nondeterministic-search.md), and [Seminar 11](seminars/11-logic-programming-and-declarative-query.md). The reordered path connects data-directed dispatch to evaluation strategy, search, and declarative interfaces.

### Runtime and compiler mechanisms

Read [Seminar 9](seminars/09-eval-apply-and-executable-semantics.md) before [Seminar 12](seminars/12-machines-storage-control-and-compilation.md). The first establishes evaluator semantics; the second makes their registers, stack discipline, storage, and compiled control explicit.

### Agent-harness design

Read [Seminar 2](seminars/02-data-abstraction-and-immutable-representation.md), [Seminar 5](seminars/05-state-identity-and-environments.md), [Seminar 7](seminars/07-concurrency-serialization-and-interleavings.md), [Seminar 9](seminars/09-eval-apply-and-executable-semantics.md), the optional [agentic eval/apply MTS synthesis](../agentic_eval_apply.md), and [Seminar 12](seminars/12-machines-storage-control-and-compilation.md), then complete the [Capstone](capstone/agent_harness_architecture_dossier.md). The synthesis bridges semantic dispatch to external-effect boundaries without replacing Seminar 9 or the capstone; this route traces a harness from representation and authority boundaries through durable state, scheduling, semantic dispatch, external-effect boundaries, and explicit resource control.

## Resume the dialogue

Open [dialogue_state.md](dialogue_state.md), follow the single `CURRENT` link to its `#dialogue` anchor, and answer the recorded question with a CLAIM plus EVIDENCE. Change understanding only after an explicit exchange supports `CONFIRMED` or `CONTESTED`; silence is not evidence and cannot advance the cursor.

## Source and license boundary

The canonical course artifact is the vendored [SICP PDF](../../../evidence/sicp/sicp.pdf). Consult the [source registry](../sources/source_registry.tsv) for provenance and use constraints, and preserve the boundary described by the vendored [license text](../../../evidence/sicp/LICENSE.txt). Course notes paraphrase and point into the source; they do not replace or silently relicense it.

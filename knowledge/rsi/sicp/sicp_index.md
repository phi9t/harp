# SICP research index

**EVIDENCE — source identity.** The vendored seed is *Structure and
Interpretation of Computer Programs*, second edition, Unofficial Texinfo Format
`2.andresraba5.6`, registered as `SICP-2E-UF` in the
[source registry](../../../content/sicp/sources/source_registry.tsv). The exact PDF, license,
manifest, and capture metadata live under
[`evidence/sicp/`](../../../evidence/sicp).

## Start the course

Open the [course guide](course/sicp_course_guide.md) for the complete sequential
route through all twelve seminars and the capstone. If Scheme syntax is not
fresh, begin with the [Scheme reading primer](course/scheme_reading_primer.md)
before Seminar 1. The [coverage ledger](../../../content/sicp/course/coverage.tsv) maps every main
section from §1.1 through §5.5 exactly once.

The course maps exactly 24 selected SICP exercises as practice prompts. It does
not provide their solutions or claim exhaustive exercise coverage.

## Choose a reading route

Choose a route for the systems question in front of you, then return to the
sequential course for dependencies the shorter path omits.

- **Learn the book's main progression.** Follow the [complete sequential
  route](course/sicp_course_guide.md#sequential-route), beginning with the
  [Scheme reading primer](course/scheme_reading_primer.md).
- **Design functional or immutable components.** Follow the [functional and immutable systems](course/sicp_course_guide.md#functional-and-immutable-systems) route
  through process shape, representation barriers, and sequence interfaces.
- **Reason about state, time, or interleavings.** Follow the [state and concurrency](course/sicp_course_guide.md#state-and-concurrency) route before
  drawing conclusions about identity, mutation, serialization, or delayed
  computation.
- **Change or implement language semantics.** Follow the [evaluation and language design](course/sicp_course_guide.md#evaluation-and-language-design) route,
  then use the [eval/apply companion](eval-apply-reader-companion.md) or the
  [metacircular evaluator deep dive](metacircular_evaluator_deep_dive.md).
- **Trace an evaluator down to a machine.** Follow the [runtime and compiler mechanisms](course/sicp_course_guide.md#runtime-and-compiler-mechanisms) route
  before using the [runnable Rust evaluator lab](../../../labs/sicp-evaluator).
- **Transfer the material into an agent harness.** Follow the [agent-harness design](course/sicp_course_guide.md#agent-harness-design) route, complete the
  optional [agentic eval/apply MTS synthesis](agentic_eval_apply.md) as the
  bridge from Seminar 9 to effectful runtime semantics, then complete the
  [capstone dossier](course/capstone/agent_harness_architecture_dossier.md).
  The synthesis supplements rather than replaces either; treat the linked Pi,
  Hermes, and Codex profiles as separate implementation overlays rather than
  evidence about the SICP source.
- **Check source authority and open gaps.** Start with the [domain orientation](domain_orientation.md),
  then use the [source
  registry](../../../content/sicp/sources/source_registry.tsv), [evidence
  graph](../../../content/sicp/sources/evidence_graph.tsv), [missing-evidence ledger](missing_evidence.md),
  and [capture provenance](../../../evidence/sicp/PROVENANCE.md). The course currently has one fetched
  source; the linked harness material does not expand that source boundary.

## Categorized course map

The course guide is the authoritative order of study. This map makes each
maintained seminar and support artifact visible without making all of them a
prerequisite for a focused question.

### Preparation and course control

- [Scheme reading primer](course/scheme_reading_primer.md) establishes the
  notation used in the seminars.
- [Course guide](course/sicp_course_guide.md), [coverage ledger](../../../content/sicp/course/coverage.tsv),
  and [exercise map](course/exercise_map.md) define the reading order, source
  coverage, and selected practice prompts.
- [Dialogue cursor](course/dialogue_state.md) records the single question that
  may advance only after an explicit reader response.

### Procedures, data, and composition

1. [Process shape and higher-order abstraction](course/seminars/01-process-shape-and-higher-order-abstraction.md)
2. [Data abstraction and immutable representation](course/seminars/02-data-abstraction-and-immutable-representation.md)
3. [Trees, sequences, and functional interfaces](course/seminars/03-trees-sequences-and-functional-interfaces.md)
4. [Symbols, sets, compression, and generic dispatch](course/seminars/04-symbols-sets-compression-and-generic-dispatch.md)

### State, concurrency, and delayed computation

5. [State, identity, and environments](course/seminars/05-state-identity-and-environments.md)
6. [Mutation, simulation, and constraints](course/seminars/06-mutation-simulation-and-constraints.md)
7. [Concurrency, serialization, and interleavings](course/seminars/07-concurrency-serialization-and-interleavings.md)
8. [Streams, delay, and infinite processes](course/seminars/08-streams-delay-and-infinite-processes.md)

### Languages, machines, and compilation

9. [Eval/apply and executable semantics](course/seminars/09-eval-apply-and-executable-semantics.md)
10. [Lazy evaluation and nondeterministic search](course/seminars/10-lazy-evaluation-and-nondeterministic-search.md)
11. [Logic programming and declarative query](course/seminars/11-logic-programming-and-declarative-query.md)
12. [Machines, storage, control, and compilation](course/seminars/12-machines-storage-control-and-compilation.md)

### Transfer, implementation, and evidence

- [Capstone — agent-harness architecture dossier](course/capstone/agent_harness_architecture_dossier.md)
  applies the course mechanisms to a coding-agent harness while keeping its
  authority, state, scheduling, and evidence boundaries explicit.
- [Agentic eval/apply MTS synthesis](agentic_eval_apply.md) supplements
  Seminar 9 with an effectful-runtime bridge to the capstone; it does not
  replace either or establish source authority.
- [Eval/apply compatibility pointer](eval-apply-reader-companion.md) and
  [metacircular evaluator deep dive](metacircular_evaluator_deep_dive.md)
  provide the maintained evaluator supplements; the [runnable Rust evaluator
  lab](../../../labs/sicp-evaluator) is executable learning code, not source
  evidence.
- [Pi harness deep dive](../pi_harness_deep_dive.md), [Hermes harness deep
  dive](../hermes_harness_deep_dive.md), and [Codex harness deep
  dive](../codex_harness_deep_dive.md) are implementation overlays for
  comparison, with their own revisions and evidence contracts.
- [domain orientation](domain_orientation.md), [source
  registry](../../../content/sicp/sources/source_registry.tsv), [evidence
  graph](../../../content/sicp/sources/evidence_graph.tsv), [missing-evidence ledger](missing_evidence.md),
  and [capture provenance](../../../evidence/sicp/PROVENANCE.md) hold the packet's source, gap, and
  provenance records.

## Resume the dialogue

The [dialogue cursor](course/dialogue_state.md#dialogue-state) identifies the
single `CURRENT` question. It currently resumes at
[Seminar 1's dialogue](course/seminars/01-process-shape-and-higher-order-abstraction.md#dialogue).
Only an explicit reader response may advance the cursor; silence never records
understanding.

## Quick links

- [Agentic eval/apply MTS synthesis](agentic_eval_apply.md) — optional
  Seminar 9-to-capstone supplement for effectful runtime semantics; not source
  authority
- [Eval/apply compatibility pointer](eval-apply-reader-companion.md)
- [Metacircular evaluator deep dive](metacircular_evaluator_deep_dive.md)
- [Runnable Rust evaluator lab](../../../labs/sicp-evaluator)
- [Pi harness deep dive](../pi_harness_deep_dive.md)
- [Hermes harness deep dive](../hermes_harness_deep_dive.md)
- [Codex harness deep dive](../codex_harness_deep_dive.md)
- [Source registry](../../../content/sicp/sources/source_registry.tsv)
- [Evidence graph](../../../content/sicp/sources/evidence_graph.tsv)
- [Missing-evidence ledger](missing_evidence.md)
- [Capture provenance](../../../evidence/sicp/PROVENANCE.md)

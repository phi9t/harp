# SICP research index

**EVIDENCE — source identity.** The vendored seed is *Structure and
Interpretation of Computer Programs*, second edition, Unofficial Texinfo Format
`2.andresraba5.6`, registered as `SICP-2E-UF` in the
[[content/sicp/sources/source_registry.tsv|source registry]]. The exact PDF, license,
manifest, and capture metadata live under
[[evidence/sicp/PROVENANCE|`evidence/sicp/`]].

## Start the course

Open the [[knowledge/rsi/sicp/course/sicp_course_guide|course guide]] for the complete sequential
route through all twelve seminars and the capstone. If Scheme syntax is not
fresh, begin with the [[knowledge/rsi/sicp/course/scheme_reading_primer|Scheme reading primer]]
before Seminar 1. The [[content/sicp/course/coverage.tsv|coverage ledger]] maps every main
section from §1.1 through §5.5 exactly once.

The course maps exactly 24 selected SICP exercises as practice prompts. It does
not provide their solutions or claim exhaustive exercise coverage.

## Choose a reading route

Choose a route for the systems question in front of you, then return to the
sequential course for dependencies the shorter path omits.

- **Learn the book's main progression.** Follow the [[knowledge/rsi/sicp/course/sicp_course_guide#Sequential route|complete sequential route]], beginning with the
  [[knowledge/rsi/sicp/course/scheme_reading_primer|Scheme reading primer]].
- **Design functional or immutable components.** Follow the [[knowledge/rsi/sicp/course/sicp_course_guide#Functional and immutable systems|functional and immutable systems]] route
  through process shape, representation barriers, and sequence interfaces.
- **Reason about state, time, or interleavings.** Follow the [[knowledge/rsi/sicp/course/sicp_course_guide#State and concurrency|state and concurrency]] route before
  drawing conclusions about identity, mutation, serialization, or delayed
  computation.
- **Change or implement language semantics.** Follow the [[knowledge/rsi/sicp/course/sicp_course_guide#Evaluation and language design|evaluation and language design]] route,
  then use the [[knowledge/rsi/sicp/eval-apply-reader-companion|eval/apply companion]] or the
  [[knowledge/rsi/sicp/metacircular_evaluator_deep_dive|metacircular evaluator deep dive]].
- **Trace an evaluator down to a machine.** Follow the [[knowledge/rsi/sicp/course/sicp_course_guide#Runtime and compiler mechanisms|runtime and compiler mechanisms]] route
  before using the [[labs/sicp-evaluator/README|runnable Rust evaluator lab]].
- **Transfer the material into an agent harness.** Follow the [[knowledge/rsi/sicp/course/sicp_course_guide#Agent-harness design|agent-harness design]] route, complete the
  optional [[knowledge/rsi/sicp/agentic_eval_apply|agentic eval/apply MTS synthesis]] as the
  bridge from Seminar 9 to effectful runtime semantics, then complete the
  [[knowledge/rsi/sicp/course/capstone/agent_harness_architecture_dossier|capstone dossier]].
  The synthesis supplements rather than replaces either; treat the linked Pi,
  Hermes, and Codex profiles as separate implementation overlays rather than
  evidence about the SICP source.
- **Check source authority and open gaps.** Start with the [[knowledge/rsi/sicp/domain_orientation|domain orientation]],
  then use the [[content/sicp/sources/source_registry.tsv|source registry]], [[content/sicp/sources/evidence_graph.tsv|evidence graph]], [[knowledge/rsi/sicp/missing_evidence|missing-evidence ledger]],
  and [[evidence/sicp/PROVENANCE|capture provenance]]. The course currently has one fetched
  source; the linked harness material does not expand that source boundary.

## Categorized course map

The course guide is the authoritative order of study. This map makes each
maintained seminar and support artifact visible without making all of them a
prerequisite for a focused question.

### Preparation and course control

- [[knowledge/rsi/sicp/course/scheme_reading_primer|Scheme reading primer]] establishes the
  notation used in the seminars.
- [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]], [[content/sicp/course/coverage.tsv|coverage ledger]],
  and [[knowledge/rsi/sicp/course/exercise_map|exercise map]] define the reading order, source
  coverage, and selected practice prompts.
- [[knowledge/rsi/sicp/course/dialogue_state|Dialogue cursor]] records the single question that
  may advance only after an explicit reader response.

### Procedures, data, and composition

1. [[knowledge/rsi/sicp/course/seminars/01-process-shape-and-higher-order-abstraction|Process shape and higher-order abstraction]]
2. [[knowledge/rsi/sicp/course/seminars/02-data-abstraction-and-immutable-representation|Data abstraction and immutable representation]]
3. [[knowledge/rsi/sicp/course/seminars/03-trees-sequences-and-functional-interfaces|Trees, sequences, and functional interfaces]]
4. [[knowledge/rsi/sicp/course/seminars/04-symbols-sets-compression-and-generic-dispatch|Symbols, sets, compression, and generic dispatch]]

### State, concurrency, and delayed computation

5. [[knowledge/rsi/sicp/course/seminars/05-state-identity-and-environments|State, identity, and environments]]
6. [[knowledge/rsi/sicp/course/seminars/06-mutation-simulation-and-constraints|Mutation, simulation, and constraints]]
7. [[knowledge/rsi/sicp/course/seminars/07-concurrency-serialization-and-interleavings|Concurrency, serialization, and interleavings]]
8. [[knowledge/rsi/sicp/course/seminars/08-streams-delay-and-infinite-processes|Streams, delay, and infinite processes]]

### Languages, machines, and compilation

9. [[knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics|Eval/apply and executable semantics]]
10. [[knowledge/rsi/sicp/course/seminars/10-lazy-evaluation-and-nondeterministic-search|Lazy evaluation and nondeterministic search]]
11. [[knowledge/rsi/sicp/course/seminars/11-logic-programming-and-declarative-query|Logic programming and declarative query]]
12. [[knowledge/rsi/sicp/course/seminars/12-machines-storage-control-and-compilation|Machines, storage, control, and compilation]]

### Transfer, implementation, and evidence

- [[knowledge/rsi/sicp/course/capstone/agent_harness_architecture_dossier|Capstone — agent-harness architecture dossier]]
  applies the course mechanisms to a coding-agent harness while keeping its
  authority, state, scheduling, and evidence boundaries explicit.
- [[knowledge/rsi/sicp/agentic_eval_apply|Agentic eval/apply MTS synthesis]] supplements
  Seminar 9 with an effectful-runtime bridge to the capstone; it does not
  replace either or establish source authority.
- [[knowledge/rsi/sicp/eval-apply-reader-companion|Eval/apply compatibility pointer]] and
  [[knowledge/rsi/sicp/metacircular_evaluator_deep_dive|metacircular evaluator deep dive]]
  provide the maintained evaluator supplements; the [[labs/sicp-evaluator/README|runnable Rust evaluator lab]] is executable learning code, not source
  evidence.
- [[knowledge/rsi/pi_harness_deep_dive|Pi harness deep dive]], [[knowledge/rsi/hermes_harness_deep_dive|Hermes harness deep dive]], and [[knowledge/rsi/codex_harness_deep_dive|Codex harness deep dive]] are implementation overlays for
  comparison, with their own revisions and evidence contracts.
- [[knowledge/rsi/sicp/domain_orientation|domain orientation]], [[content/sicp/sources/source_registry.tsv|source registry]], [[content/sicp/sources/evidence_graph.tsv|evidence graph]], [[knowledge/rsi/sicp/missing_evidence|missing-evidence ledger]],
  and [[evidence/sicp/PROVENANCE|capture provenance]] hold the packet's source, gap, and
  provenance records.

## Resume the dialogue

The [[knowledge/rsi/sicp/course/dialogue_state#Dialogue state|dialogue cursor]] identifies the
single `CURRENT` question. It currently resumes at
[[knowledge/rsi/sicp/course/seminars/01-process-shape-and-higher-order-abstraction#Dialogue|Seminar 1's dialogue]].
Only an explicit reader response may advance the cursor; silence never records
understanding.

## Quick links

- [[knowledge/rsi/sicp/agentic_eval_apply|Agentic eval/apply MTS synthesis]] — optional
  Seminar 9-to-capstone supplement for effectful runtime semantics; not source
  authority
- [[knowledge/rsi/sicp/eval-apply-reader-companion|Eval/apply compatibility pointer]]
- [[knowledge/rsi/sicp/metacircular_evaluator_deep_dive|Metacircular evaluator deep dive]]
- [[labs/sicp-evaluator/README|Runnable Rust evaluator lab]]
- [[knowledge/rsi/pi_harness_deep_dive|Pi harness deep dive]]
- [[knowledge/rsi/hermes_harness_deep_dive|Hermes harness deep dive]]
- [[knowledge/rsi/codex_harness_deep_dive|Codex harness deep dive]]
- [[content/sicp/sources/source_registry.tsv|Source registry]]
- [[content/sicp/sources/evidence_graph.tsv|Evidence graph]]
- [[knowledge/rsi/sicp/missing_evidence|Missing-evidence ledger]]
- [[evidence/sicp/PROVENANCE|Capture provenance]]

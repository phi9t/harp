# SICP — DOMAIN ORIENTATION

## Scope and claim boundary

**EVIDENCE — active mode.** This document is in **DOMAIN ORIENTATION** mode: it
optimizes for coverage, vocabulary, source lineage, and explicit gaps. The
bounded mechanism analysis begins only in
[`metacircular_evaluator_deep_dive.md`](metacircular_evaluator_deep_dive.md).

**EVIDENCE — seed boundary.** `SICP-2E-UF` is the only fetched content source.
It is an 883-page unofficial Texinfo rendering of the second edition, not an
official MIT Press typeset PDF. Source claims below are anchored by printed page
and PDF page, whose offset is 28 pages through the numbered body.

**CLAIM — pedagogical objective.** The authors frame the course around
techniques for controlling the intellectual complexity of large software
systems rather than around language syntax or isolated clever algorithms
(`SICP-2E-UF`, preface, printed pp. xxii–xxiii, PDF pp. 22–23).

**CLAIM — second-edition emphasis.** The second edition makes competing ways
of representing time central: stateful objects, concurrency, functional
programming, laziness, and nondeterminism (`SICP-2E-UF`, preface, printed p. xx,
PDF p. 20).

**INFERENCE — organizing question.** A productive reading question is: what
representation lets the programmer ignore which detail now, and what cost or
semantic obligation reappears at the next layer?

## Actors and artifacts

| Label | Actor or artifact | Role visible in the seed | Anchor |
|---|---|---|---|
| EVIDENCE | Harold Abelson and Gerald Jay Sussman | Principal authors | Title/license page, PDF p. 2 |
| EVIDENCE | Julie Sussman | Contributing author | Title/license page, PDF p. 2 |
| EVIDENCE | Alan J. Perlis | Foreword author | Foreword, printed pp. xiii–xviii, PDF pp. 13–18 |
| EVIDENCE | MIT 6.001 | Course context from which the material developed | Prefaces, printed pp. xix–xxiv, PDF pp. 19–24 |
| EVIDENCE | Scheme | Small Lisp dialect used as the executable notation | Preface, printed pp. xxiii–xxiv, PDF pp. 23–24 |
| EVIDENCE | Unofficial Texinfo Format `2.andresraba5.6` | Captured rendering, dated 2016-02-02 | Title/license and format notes, PDF pp. 1–3 |
| EVIDENCE | Five chapter-scale systems | Procedures, data, state/time, evaluators/languages, and register machines/compilation | Contents, PDF pp. 3–8 |

## The domain map

| Label | Layer | Representation and question | Main downstream consumer | Source anchor |
|---|---|---|---|---|
| CLAIM | 1. Procedures and processes | Expressions and higher-order procedures describe computational processes; process shape and order of growth expose resource behavior. | A programmer composing and analyzing procedures. | Ch. 1, printed pp. 1–106, PDF pp. 29–134 |
| CLAIM | 2. Data abstraction | Constructors, selectors, abstraction barriers, tags, and generic operations separate use from representation. | Programs that must admit multiple data representations without global rewrites. | Ch. 2, printed pp. 107–293, PDF pp. 135–321 |
| CLAIM | 3. State and time | Environments, mutation, objects, concurrency controls, and delayed streams provide competing organizations for evolving systems. | Models whose history, identity, event ordering, or infinite information flow matters. | Ch. 3, printed pp. 294–486, PDF pp. 322–514 |
| CLAIM | 4. Languages as abstractions | Evaluators turn language design into program design; changing evaluation rules yields lazy, nondeterministic, and logic-programming systems. | Language designers and domain-specific-language implementers. | Ch. 4, printed pp. 487–665, PDF pp. 515–693 |
| CLAIM | 5. Explicit machines and compilation | Registers, stacks, storage allocation, an explicit-control evaluator, and a compiler expose mechanisms hidden by the host Lisp. | Runtime and compiler implementers who must account for control and storage. | Ch. 5, printed pp. 666–833, PDF pp. 694–861 |

**INFERENCE — vertical structure.** The five chapters form a deliberate descent:
first use abstractions, then examine the semantic machinery that makes them
work, and finally lower that machinery into control, storage, and instruction
sequences.

**INFERENCE — recurring triad.** Across the descent, SICP repeatedly asks for
primitives, means of combination, and means of abstraction. The concrete
objects change—from procedures, to data, to languages, to machines—but the
design test remains whether parts compose without forcing readers to reopen
every lower layer.

## Vocabulary needed before a deep dive

| Label | Term | Working meaning anchored to the seed |
|---|---|---|
| CLAIM | Computational process | An evolving abstract entity described by a program and manipulating data (`SICP-2E-UF`, Ch. 1 opening, printed p. 1, PDF p. 29). |
| CLAIM | Abstraction barrier | An interface separating how a data object is used from how it is represented (`SICP-2E-UF`, §2.1.2, printed pp. 118–121, PDF pp. 146–149). |
| CLAIM | Environment | A sequence of frames whose bindings associate names with values; an enclosing link supplies lexical context (`SICP-2E-UF`, §3.2 and §4.1.3, especially printed pp. 512–517, PDF pp. 540–545). |
| CLAIM | Closure | A compound procedure represented by parameters, body, and the environment present when the lambda is evaluated (`SICP-2E-UF`, §4.1.1 and §4.1.3, printed pp. 496–498 and 512–513, PDF pp. 524–526 and 540–541). |
| CLAIM | Evaluator | A procedure that maps a language expression to the actions required to evaluate it (`SICP-2E-UF`, Ch. 4 opening, printed p. 489, PDF p. 517). |
| CLAIM | Metacircular evaluator | An evaluator implemented in the same language that it evaluates (`SICP-2E-UF`, §4.1, printed p. 492, PDF p. 520). |
| CLAIM | Syntactic analysis | Classification and decomposition of an expression, which can be separated from environment-dependent execution (`SICP-2E-UF`, §4.1.7, printed pp. 534–539, PDF pp. 562–567). |
| CLAIM | Lexical address | A compile-time coordinate identifying a variable binding by environment-frame depth and displacement (`SICP-2E-UF`, §5.5.6, printed pp. 817–822, PDF pp. 845–850). |
| CLAIM | Explicit-control evaluator | A register-machine realization that makes evaluator control transfers and stack use explicit (`SICP-2E-UF`, §5.4, printed pp. 741–766, PDF pp. 769–794). |

## Comparison contract: abstraction families

| Label | Family | Representation | Objective | Resolution path | Evaluation identity | Quality/cost boundary |
|---|---|---|---|---|---|---|
| INFERENCE | Procedure abstraction | Named and higher-order procedures | Reuse process patterns while hiding local detail | Evaluate expression → apply procedure | Functional result plus process shape | Orders of growth are explicit; complete empirical timing is MISSING |
| INFERENCE | Data abstraction | Constructors/selectors, tags, generic-operation tables | Admit representation change and additive extension | Operation → interface → selected representation method | Abstract value versus concrete encoding | Indirection and dispatch exist; systematic measurements are MISSING |
| INFERENCE | Objects/state | Mutable bindings and identity-bearing objects | Localize evolving history | Message/operation → current environment/state | Observable behavior over an event history | Reasoning and concurrency hazards are discussed; benchmark data are MISSING |
| INFERENCE | Streams | Delayed sequences | Decouple modeled time from evaluator event order | Demand → force delayed computation → memoized element | Produced sequence prefix | Space/time depend on demand and retention; measured envelopes are MISSING |
| INFERENCE | Evaluators | Programs and data sharing symbolic structure | Make language semantics modifiable | Read → classify → evaluate/apply in an environment | Value under a specified evaluator | Direct dispatch and lookup overhead are identified; quantitative results are MISSING |
| INFERENCE | Register machines and compilers | Registers, stacks, instruction sequences, lexical addresses | Expose and optimize control/storage | Assemble or compile → execute instructions | Machine state trace and resulting value | Stack pushes, instruction counts, and lookup work become observable; end-to-end benchmarks are MISSING |

## Evidence graph and reading priorities

**EVIDENCE — source topology.** The seed explicitly connects Scheme to Lisp,
Algol block structure, lexical scope, and λ-calculus in its first-edition
preface (`SICP-2E-UF`, printed pp. xxiii–xxiv, PDF pp. 23–24). The exact
lineage edges and access states are recorded in
[`sources/evidence_graph.tsv`](sources/evidence_graph.tsv).

**MISSING — blocked reconstruction.** `MCCARTHY-1960`, `CHURCH-1941`,
`STEELE-SUSSMAN-1975`, `REES-ADAMS-1982`, `FEELEY-1986`,
`FEELEY-LAPALME-1987`, and `IEEE-SCHEME-1990` were not fetched. Their contents,
proofs, measurements, and historical nuances are not reconstructed here.

**INFERENCE — next reading order.** After the evaluator deep dive, the
highest-leverage return paths are §3.2 for lexical environments, §4.2–§4.4 for
semantic variation, and §5.4–§5.5 for the interpreter-to-machine-to-compiler
continuum.

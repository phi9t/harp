# Dialogue state

| Seminar | Claim | Cursor status | Understanding status | Question | Link |
|---|---|---|---|---|---|
| 01 | process-shape | CURRENT | PROVISIONAL | When two procedures compute the same values, what evidence would convince you that they generate materially different processes? | [Seminar 1](seminars/01-process-shape-and-higher-order-abstraction.md#dialogue) |
| 02 | representation-authority | PENDING | PROVISIONAL | Which observable equations must remain true for clients when a representation changes, and which additional evidence is required before a visible tool description can be treated as an executable capability? | [Seminar 2](seminars/02-data-abstraction-and-immutable-representation.md#dialogue) |
| 03 | traversal-contract | PENDING | PROVISIONAL | When does decomposing a traversal into enumerate, filter, map, and fold clarify the contract, and when does it hide an ordering or effect that should stay explicit? | [Seminar 3](seminars/03-trees-sequences-and-functional-interfaces.md#dialogue) |
| 04 | dispatch-authority | PENDING | PROVISIONAL | Which facts belong in a dispatch key, and which checks must remain outside lookup so that an additive registry does not confuse recognition with authority? | [Seminar 4](seminars/04-symbols-sets-compression-and-generic-dispatch.md#dialogue) |
| 05 | state-boundaries | PENDING | PROVISIONAL | Which facts in an agent run must be retained as durable history, which should be selectively remembered across sessions, and which can only be re-observed from the external world? | [Seminar 5](seminars/05-state-identity-and-environments.md#dialogue) |
| 06 | effect-receipts | PENDING | PROVISIONAL | What evidence lets a harness distinguish an action that was merely proposed, durably admitted, actually attempted, and verified complete after a crash? | [Seminar 6](seminars/06-mutation-simulation-and-constraints.md#dialogue) |
| 07 | cancellation-lineage | PENDING | PROVISIONAL | When an agent harness cancels one delegated task, what evidence should determine whether only that task, its descendants, its siblings, or the whole root run must stop? | [Seminar 7](seminars/07-concurrency-serialization-and-interleavings.md#dialogue) |
| 08 | compaction-retention | PENDING | PROVISIONAL | When a harness shortens model context, which information may be summarized for future reasoning, and which raw events, authority decisions, lineage edges, or effect receipts must remain separately retrievable? | [Seminar 8](seminars/08-streams-delay-and-infinite-processes.md#dialogue) |
| 09 | eval-apply-boundary | PENDING | PROVISIONAL | Given one tool request, what concrete artifact exists at the source-character or model-text boundary, at the expression-datum or structured-call boundary, at the runtime-value or resolved-capability boundary, and exactly what is handed to the apply-like invocation step? | [Seminar 9](seminars/09-eval-apply-and-executable-semantics.md#dialogue) |
| 10 | branch-retry | PENDING | PROVISIONAL | What evidence would let you call a rejected agent branch safely retryable, rather than merely attributable to a parent and no longer selected for model context? | [Seminar 10](seminars/10-lazy-evaluation-and-nondeterministic-search.md#dialogue) |
| 11 | declarative-operational | PENDING | PROVISIONAL | Which parts of an agent task should describe relationships that may be satisfied in many ways, and which parts must prescribe operational control because order, authority, budget, or effects are observable? | [Seminar 11](seminars/11-logic-programming-and-declarative-query.md#dialogue) |
| 12 | continuation-audit | PENDING | PROVISIONAL | Which harness obligations behave like explicit continuations that must survive a model or tool subcomputation, and which are durable authority or audit records that must not be treated as stack-local state? | [Seminar 12](seminars/12-machines-storage-control-and-compilation.md#dialogue) |

Cursor values are `CURRENT`, `PENDING`, and `COMPLETE`. Exactly one row is `CURRENT`: it identifies the dialogue to resume. A row becomes `COMPLETE` only through an explicit dialogue transition; future rows are `PENDING`.

Understanding values are `PROVISIONAL`, `CONFIRMED`, and `CONTESTED`. `PROVISIONAL` records a claim still being tested, `CONFIRMED` records an evidence-backed agreement, and `CONTESTED` records a specific unresolved objection.

Silence never changes either cursor status or understanding status.

Return to the [course guide](sicp_course_guide.md) to continue by sequence or
concept route without advancing this cursor.

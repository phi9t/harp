# Dialogue state

| Seminar | Claim | Cursor status | Understanding status | Question | Link |
|---|---|---|---|---|---|
| 01 | process-shape | CURRENT | PROVISIONAL | When two procedures compute the same values, what evidence would convince you that they generate materially different processes? | [[knowledge/rsi/sicp/course/seminars/01-process-shape-and-higher-order-abstraction#Dialogue|Seminar 1]] |
| 02 | representation-authority | PENDING | PROVISIONAL | Which observable equations must remain true for clients when a representation changes, and which additional evidence is required before a visible tool description can be treated as an executable capability? | [[knowledge/rsi/sicp/course/seminars/02-data-abstraction-and-immutable-representation#Dialogue|Seminar 2]] |
| 03 | traversal-contract | PENDING | PROVISIONAL | When does decomposing a traversal into enumerate, filter, map, and fold clarify the contract, and when does it hide an ordering or effect that should stay explicit? | [[knowledge/rsi/sicp/course/seminars/03-trees-sequences-and-functional-interfaces#Dialogue|Seminar 3]] |
| 04 | dispatch-authority | PENDING | PROVISIONAL | Which facts belong in a dispatch key, and which checks must remain outside lookup so that an additive registry does not confuse recognition with authority? | [[knowledge/rsi/sicp/course/seminars/04-symbols-sets-compression-and-generic-dispatch#Dialogue|Seminar 4]] |
| 05 | state-boundaries | PENDING | PROVISIONAL | Which facts in an agent run must be retained as durable history, which should be selectively remembered across sessions, and which can only be re-observed from the external world? | [[knowledge/rsi/sicp/course/seminars/05-state-identity-and-environments#Dialogue|Seminar 5]] |
| 06 | effect-receipts | PENDING | PROVISIONAL | What evidence lets a harness distinguish an action that was merely proposed, durably admitted, actually attempted, and verified complete after a crash? | [[knowledge/rsi/sicp/course/seminars/06-mutation-simulation-and-constraints#Dialogue|Seminar 6]] |
| 07 | cancellation-lineage | PENDING | PROVISIONAL | When an agent harness cancels one delegated task, what evidence should determine whether only that task, its descendants, its siblings, or the whole root run must stop? | [[knowledge/rsi/sicp/course/seminars/07-concurrency-serialization-and-interleavings#Dialogue|Seminar 7]] |
| 08 | compaction-retention | PENDING | PROVISIONAL | When a harness shortens model context, which information may be summarized for future reasoning, and which raw events, authority decisions, lineage edges, or effect receipts must remain separately retrievable? | [[knowledge/rsi/sicp/course/seminars/08-streams-delay-and-infinite-processes#Dialogue|Seminar 8]] |
| 09 | eval-apply-boundary | PENDING | PROVISIONAL | Given one tool request, what concrete artifact exists at the source-character or model-text boundary, at the expression-datum or structured-call boundary, at the runtime-value or resolved-capability boundary, and exactly what is handed to the apply-like invocation step? | [[knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics#Dialogue|Seminar 9]] |
| 10 | branch-retry | PENDING | PROVISIONAL | What evidence would let you call a rejected agent branch safely retryable, rather than merely attributable to a parent and no longer selected for model context? | [[knowledge/rsi/sicp/course/seminars/10-lazy-evaluation-and-nondeterministic-search#Dialogue|Seminar 10]] |
| 11 | declarative-operational | PENDING | PROVISIONAL | Which parts of an agent task should describe relationships that may be satisfied in many ways, and which parts must prescribe operational control because order, authority, budget, or effects are observable? | [[knowledge/rsi/sicp/course/seminars/11-logic-programming-and-declarative-query#Dialogue|Seminar 11]] |
| 12 | continuation-audit | PENDING | PROVISIONAL | Which harness obligations behave like explicit continuations that must survive a model or tool subcomputation, and which are durable authority or audit records that must not be treated as stack-local state? | [[knowledge/rsi/sicp/course/seminars/12-machines-storage-control-and-compilation#Dialogue|Seminar 12]] |

Cursor values are `CURRENT`, `PENDING`, and `COMPLETE`. Exactly one row is `CURRENT`: it identifies the dialogue to resume. A row becomes `COMPLETE` only through an explicit dialogue transition; future rows are `PENDING`.

Understanding values are `PROVISIONAL`, `CONFIRMED`, and `CONTESTED`. `PROVISIONAL` records a claim still being tested, `CONFIRMED` records an evidence-backed agreement, and `CONTESTED` records a specific unresolved objection.

Silence never changes either cursor status or understanding status.

Return to the [[knowledge/rsi/sicp/course/sicp_course_guide|course guide]] to continue by sequence or
concept route without advancing this cursor.

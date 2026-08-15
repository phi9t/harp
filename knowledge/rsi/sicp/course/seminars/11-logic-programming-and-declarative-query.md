# Seminar 11 — Logic programming and declarative query

## Reading route

Read the chapter's query language as an executable relation system, not as a promise that logical notation makes control disappear. Keep the goal being described separate from the mechanism that enumerates proofs.

- Required: [[evidence/sicp/sicp.pdf#page=622|the §4.4 introduction and transition into §4.4.1, printed pp. 594–599 / PDF pp. 622–627]]. It positions logic programming as another language whose evaluator has primitives, combination, abstraction, frames, and streams.
- Required: [[evidence/sicp/sicp.pdf#page=627|§4.4.1, printed pp. 599–615 / PDF pp. 627–643]]. Follow simple patterns, compound queries, and rules as three ways to state relationships over the personnel assertions.
- Required: [[evidence/sicp/sicp.pdf#page=643|§4.4.2, printed pp. 615–627 / PDF pp. 643–655]]. Track a query as a transformation from an input stream of frames to a stream of consistent frame extensions; pay particular attention to the difference between pattern matching and unification.
- Required: [[evidence/sicp/sicp.pdf#page=655|§4.4.3, printed pp. 627–635 / PDF pp. 655–663]]. Treat clause order, infinite deductions, `not`, and unbound variables as semantic and operational constraints, not as incidental performance notes.
- Required: [[evidence/sicp/sicp.pdf#page=663|§4.4.4, printed pp. 635–665 / PDF pp. 663–693]]. Read the implementation by layer: syntax and dispatch, stream combination, assertion matching, rule application, variable renaming, unification, indexing, and frame operations.
- Optional: none assigned. The implementation details are needed to support this seminar's claims about execution rather than only its surface notation.
- Skim: none assigned. This is a focused rather than deep seminar, but the complete §4.4 range is required; move quickly through repetitive selectors only after you can state which abstraction boundary they protect.

## System-design problem

How can a system let users describe relationships they want to establish without hand-writing an enumeration loop, while still giving the implementation enough explicit state to search, combine partial bindings, apply reusable rules, delay results, and report failure? The notation can separate a desired relation from some control details, but the implementation must still choose clause order, stream order, indexing, variable scope, and behavior for negation and nontermination.

## Vocabulary

- **Assertion**: a fact stored directly in the data base, represented as list-shaped data.
- **Query**: a pattern or compound form asking for variable assignments that satisfy a relationship.
- **Pattern variable**: a query-language variable such as `?person`; repeated occurrences require consistent values.
- **Frame**: a set of bindings that records a partial assignment of pattern variables to terms. A query either rejects a frame or extends it consistently.
- **Frame stream**: a delayed sequence of possible frames. It represents alternative partial or complete answers without requiring all answers to be materialized first.
- **Pattern matching**: one-sided comparison of a pattern with a datum under an existing frame; variables occur on the pattern side.
- **Unification**: symmetric comparison of two patterns that may both contain variables, producing a consistent frame extension or failure.
- **Rule**: a conclusion pattern plus a body query. Establishing the conclusion means unifying it with the current query and then establishing the body under the resulting frame.
- **Variable renaming**: giving every variable in each rule application a fresh identity so independent applications do not accidentally share bindings.
- **Instantiation**: replacing the variables in a query with the values reachable through a result frame for presentation to the user.
- **Search policy**: the operational choices that determine which assertions, rules, clauses, and answer streams are explored first and whether one path can delay another indefinitely.
- **Closed-world assumption**: treating a proposition that cannot be deduced from the data base as false; this is not the same as proving its logical negation.

## Argument map

1. **CLAIM.** The query language lets a user request relationships by writing patterns and compound queries over stored assertions, then extends those relationships with rules whose conclusions may be established by satisfying their bodies. [[evidence/sicp/sicp.pdf#page=627|§4.4.1, printed pp. 599–615 / PDF pp. 627–643]]. **INFERENCE.** “Declarative” here means that the query can name a desired relationship without spelling out a conventional record-scanning loop; it does not mean that the system has no execution strategy.
2. **CLAIM.** A query consumes a stream of input frames and produces a stream of consistent frame extensions. Conjunction feeds the output frames of one clause into the next clause, while disjunction evaluates alternatives and interleaves their streams. [[evidence/sicp/sicp.pdf#page=643|§4.4.2, printed pp. 615–621 / PDF pp. 643–649]] [[evidence/sicp/sicp.pdf#page=668|§4.4.4.2 implements disjunction with `interleave-delayed`, printed p. 640 / PDF p. 668]]. **INFERENCE.** The frame is both accumulated evidence and branch-local constraint state. A later clause cannot choose bindings independently of the choices recorded by earlier clauses.
3. **CLAIM.** Pattern matching compares a query pattern with assertion data under an existing frame, whereas applying a rule requires unification because variables may occur in both the query and the rule conclusion. [[evidence/sicp/sicp.pdf#page=643|§4.4.2, printed pp. 615–625 / PDF pp. 643–653]]. **INFERENCE.** Matching, unification, and search answer different questions: whether one datum fits a pattern, whether two variable-bearing terms can be made equal, and which candidate derivations should be explored.
4. **CLAIM.** Before applying a rule, the implementation renames every rule variable with a fresh application identity, unifies the renamed conclusion with the query under the input frame, and evaluates the rule body only if that unification succeeds. [[evidence/sicp/sicp.pdf#page=673|§4.4.4.4, printed pp. 645–650 / PDF pp. 673–678]]. **INFERENCE.** Fresh naming supplies the rule-level analogue of local scope. It prevents two invocations that both spell a variable `?x` from imposing an accidental equality constraint on each other.
5. **CLAIM.** The implementation keeps syntax classification and data-directed `qeval` dispatch, stream operations, assertion lookup and matching, rule lookup and unification, data-base indexing, and frame/binding operations as distinct cooperating layers. [[evidence/sicp/sicp.pdf#page=663|§4.4.4, printed pp. 635–665 / PDF pp. 663–693]]. **INFERENCE.** A frame is not a proof procedure, a unifier is not a scheduler, and a typed term representation is not an indexing strategy. Keeping these interfaces distinct makes it possible to test representation invariants without claiming that search behavior follows from them.
6. **CLAIM.** Logical equivalence does not make operational behavior equivalent in this query system: reordering conjuncts can change cost and results involving unbound filters, recursive rules can loop, and `not` means failure to deduce under a closed-world assumption rather than mathematical negation. [[evidence/sicp/sicp.pdf#page=655|§4.4.3, printed pp. 627–635 / PDF pp. 655–663]]. **INFERENCE.** A declarative relation still needs a procedural contract for order, fairness, termination, duplicate derivations, and admissible filtering. Those properties must be inspected separately from the relation's surface reading.

## Mechanism trace

Trace this compound query against the book's personnel data base. The trainee job and the relevant Slumerville addresses are stored assertions [[evidence/sicp/sicp.pdf#page=628|in §4.4.1, printed p. 600 / PDF p. 628]]; `lives-near` and `same` are rules [[evidence/sicp/sicp.pdf#page=636|in §4.4.1, printed pp. 608–609 / PDF pp. 636–637]].

```scheme
(and (job ?p (computer programmer trainee))
     (lives-near ?p (Bitdiddle Ben)))
```

The trace follows one demanded result through the implementation. Bindings below are shown after following variable-to-variable links; an implementation frame may retain those links rather than rewriting every binding.

1. **Start with an input frame stream.** The driver calls `qeval` with the query and `S₀ = stream[F₀]`, where `F₀ = {}`. Data-directed dispatch recognizes `and` and selects conjunction processing. [[evidence/sicp/sicp.pdf#page=666|`qeval` and the registration of `conjoin` for `and` appear in §4.4.4.2, printed pp. 638–640 / PDF pp. 666–668.]]
2. **Pattern-match the first clause.** `conjoin` sends `S₀` through `(job ?p (computer programmer trainee))`. `simple-query` tests relevant assertions against `F₀`, following the simple-query pipeline [[evidence/sicp/sicp.pdf#page=666|in §4.4.4.2, printed pp. 638–639 / PDF pp. 666–667]]. The stored trainee assertion [[evidence/sicp/sicp.pdf#page=628|in §4.4.1, printed p. 600 / PDF p. 628]] matches and extends the frame to `F₁ = {?p ↦ (Reasoner Louis)}`; incompatible job assertions yield failure and are filtered out. For the book's data base and rules, the first clause therefore supplies `stream[F₁]`.
3. **Feed that frame to the second clause.** Conjunction does not restart from the empty frame: `conjoin` passes each clause's output stream to the remaining clauses [[evidence/sicp/sicp.pdf#page=667|in §4.4.4.2, printed pp. 639–640 / PDF pp. 667–668]]. It evaluates `(lives-near ?p (Bitdiddle Ben))` with `stream[F₁]`, so every assertion match or rule application must preserve `?p ↦ (Reasoner Louis)`. No direct `lives-near` assertion matches, so the useful branch comes from the rule cited above.
4. **Rename the rule before applying it.** Suppose this application receives identity `7`. The rule variables become `?person-1-7`, `?person-2-7`, `?town-7`, `?rest-1-7`, and `?rest-2-7`. These names are fresh even if an earlier or concurrent rule application used the original spellings. This rename-before-unify sequence is the rule-application contract [[evidence/sicp/sicp.pdf#page=673|in §4.4.4.4, printed pp. 645–647 / PDF pp. 673–675]].
5. **Unify the query with the renamed conclusion.** Unifying `(lives-near ?p (Bitdiddle Ben))` with `(lives-near ?person-1-7 ?person-2-7)` under `F₁` succeeds. Chasing the existing `?p` binding yields `F₂ = F₁ ∪ {?person-1-7 ↦ (Reasoner Louis), ?person-2-7 ↦ (Bitdiddle Ben)}`. This is symmetric variable-bearing unification, whose consistency and `depends-on?` cases are implemented [[evidence/sicp/sicp.pdf#page=675|in §4.4.4.4, printed pp. 647–650 / PDF pp. 675–678]], not assertion pattern matching.
6. **Evaluate the rule body as a serial frame pipeline.** The first address clause pattern-matches Louis's stored Slumerville address and produces `F₃ = F₂ ∪ {?town-7 ↦ Slumerville, ?rest-1-7 ↦ ((Pine Tree Road) 80)}`. The second address clause must use that same town binding; Ben's stored Slumerville address is consistent, producing `F₄ = F₃ ∪ {?rest-2-7 ↦ ((Ridge Road) 10)}`. [[evidence/sicp/sicp.pdf#page=628|Both address assertions appear in §4.4.1, printed p. 600 / PDF p. 628.]] An address in another town would produce no frame on this branch.
7. **Run the final filter with bound inputs.** `(not (same ?person-1-7 ?person-2-7))` asks whether the current frame can be extended to establish that Louis and Ben are the same. The assertion path finds nothing, and a freshly renamed application of the `same` rule [[evidence/sicp/sicp.pdf#page=636|from §4.4.1, printed p. 608 / PDF p. 636]] cannot bind one repeated variable consistently to two distinct person constants. Because the negated query produces an empty stream, `not` keeps `F₄`, following the singleton-frame/empty-stream filter implemented [[evidence/sicp/sicp.pdf#page=668|in §4.4.4.2, printed pp. 640–641 / PDF pp. 668–669]].
8. **Return and instantiate.** The body returns `stream[F₄]`; the rule application returns that stream to `simple-query`; and the second conjunct returns it as the compound query's output. The driver follows bindings from `F₄` to instantiate the original query with `?p = (Reasoner Louis)`, as described by the driver loop [[evidence/sicp/sicp.pdf#page=664|in §4.4.4.1, printed pp. 636–637 / PDF pp. 664–665]]. Other assertions and rules are examined only as their result streams are demanded, and failed branches contribute no output frame.

This trace separates five mechanisms that are easy to collapse: the compound form orders subqueries; pattern matching extends a frame from an assertion; renaming isolates a rule application; unification connects a query to a rule conclusion; and stream operations enumerate surviving frames. None of those mechanisms alone establishes completeness or termination.

## Rust lens

An idiomatic Rust model can make the representation boundary explicit by assigning variables stable identities and making a substitution map a distinct type:

```rust
// Explanatory sketch; not compiled by this course.
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Var {
    spelling: String,
    rule_application: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Term {
    Atom(String),
    List(Vec<Term>),
    Var(Var),
}

#[derive(Clone, Debug, Default)]
struct Frame(BTreeMap<Var, Term>);

#[derive(Clone, Debug)]
struct Rule {
    conclusion: Term,
    body: Goal,
}

#[derive(Clone, Debug)]
enum Goal {
    Simple(Term),
    And(Vec<Goal>),
    Or(Vec<Goal>),
    Not(Box<Goal>),
}

fn unify(_left: &Term, _right: &Term, _input: &Frame) -> Option<Frame> {
    todo!("walk bindings, check consistency, perform SICP's depends-on? test (the occurs check), then extend")
}

fn rename_rule(_rule: &Rule, _application: u64) -> Rule {
    todo!("replace every rule-local Var identity consistently")
}

// A real evaluator would return a lazy, interleaved stream of Frames.
```

This representation prevents an atom and a variable from being confused merely because both are strings, and the application identifier gives fresh rule variables a place in the type model. `Option<Frame>` can express one unification attempt's success or failure. It does not specify how assertions are indexed, how alternative result streams are interleaved, whether recursive search is fair or complete, whether duplicate derivations are removed, or whether a query terminates. Rust's types can make illegal representation states harder to construct; they cannot prove properties of an omitted search procedure.

## Agent-harness bridge

**INFERENCE (bounded architectural transfer).** Some parts of an agent request can be stated declaratively: a desired repository property, a set of acceptance checks, a relationship to retrieve from indexed evidence, or constraints a candidate result must satisfy. A harness still needs operational rules for evidence acquisition, action order, capability authorization, effect execution, concurrency, budgets, cancellation, persistence, and termination. The useful transfer is to ask which request components describe acceptable relationships and which components must remain an explicit control contract.

**MISSING (intentionally not claimed).** No pinned Pi, Hermes, or Codex source is presented here as a logic-programming system, and this seminar makes no source-backed claim that any of them evaluates requests as frame streams, performs unification, or implements a closed-world query semantics.

**AGENT-HARNESS RESEARCH QUESTION — INFERENCE.** Which portions of a agent task contract could be declarative—goals, invariants, evidence relationships, and acceptance predicates—while execution order, authority, budgets, effects, recovery, and terminal receipts remain explicit operational state?

## Limits of the analogy

A natural-language request is not a formal query with a fixed term grammar, and an LLM-generated candidate is not a substitution frame. Agent evidence may be incomplete, mutable, permission-gated, or contradictory; failure to retrieve a fact does not license closed-world negation. Tool execution changes an external world and can consume non-refundable resources, whereas SICP's query evaluator searches assertions and rules without defining a general effect protocol.

Likewise, constraints do not supply a scheduler. Describing an acceptable patch does not enumerate all patches, define a fair candidate order, prevent an infinite investigation, allocate budgets, authorize tools, or prove that a failed search means no solution exists. Any agent-harness use of declarative task fields would require separately specified operational semantics and receipts; the vocabulary of goals and predicates alone would not turn the harness into a theorem prover or logic-programming runtime.

## Practice

### Manual trace

Starting from a singleton stream containing the empty frame, trace `(and (supervisor ?worker ?manager) (outranked-by ?manager (Warbucks Oliver)))` through the personnel data base. For each demanded output, record the ordered conjunct, input frame, assertion matches, applicable rule conclusion, fresh variable identities, every unification-created binding, recursive body query, and resulting output frame. Stop after two output frames or after the stream is exhausted, whichever occurs first. Mark any duplicate derivation separately from a distinct variable assignment, and do not infer that an unobserved answer is impossible merely because you stopped demanding the stream.

### SICP exercises

- Exercise 4.56: formulate the three requested compound queries over supervisors, addresses, jobs, and salaries. Purpose: test whether each clause extends or filters frames only after the variables it requires have been bound. Invariants: shared pattern-variable names denote the same value across conjuncts, every reported tuple is supported by the required assertions, and clause order does not silently invoke `lisp-value` with an unbound argument. The query forms and results are intentionally not supplied here. [[evidence/sicp/sicp.pdf#page=635|Exercise 4.56, printed p. 607 / PDF p. 635]]
- Exercise 4.77: design the requested delayed-filter repair for `not` and `lisp-value`. Purpose: make the dependency between a filter and its required bindings explicit without postponing every filter longer than necessary. Invariants: a filter never runs before all variables it consumes are bound, it runs as soon as those bindings become available, it is not silently dropped when a frame reaches output, and different conjunct orderings preserve the intended answers for the supported query fragment. No promise representation, scheduling algorithm, or implementation is supplied here. [[evidence/sicp/sicp.pdf#page=691|Exercise 4.77, printed pp. 663–664 / PDF pp. 691–692]]

### Transfer prompt

Take one coding-agent request and partition it into declarative and operational parts. The declarative side may name desired repository relationships, invariants, evidence requirements, and acceptance predicates. The operational side must specify execution order, capability authority, effect boundaries, per-step and total budgets, cancellation, retries, persistence, and terminal receipts. For each declarative clause, state what bindings or evidence must exist before it can be tested. Then explain why failure under the chosen bounded search policy is not proof that no satisfying implementation exists.

## Dialogue

**PROVISIONAL.** `Which parts of an agent task should describe relationships that may be satisfied in many ways, and which parts must prescribe operational control because order, authority, budget, or effects are observable?`

This question is queued; [[knowledge/rsi/sicp/course/dialogue_state|the dialogue state]] still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [[knowledge/rsi/sicp/course/seminars/10-lazy-evaluation-and-nondeterministic-search|Seminar 10 — Lazy evaluation and nondeterministic search]] · [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]] · Next: [[knowledge/rsi/sicp/course/seminars/12-machines-storage-control-and-compilation|Seminar 12 — Machines, storage, control, and compilation]]

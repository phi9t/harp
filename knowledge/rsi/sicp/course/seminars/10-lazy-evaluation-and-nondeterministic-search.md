# Seminar 10 — Lazy evaluation and nondeterministic search

## Reading route

Read these two evaluator variations as answers to different control questions. Lazy evaluation asks when an expression must produce a value. Nondeterministic evaluation asks what continuation should run when the current choice cannot produce an acceptable value. Neither question disappears merely because the evaluator hides its mechanism from the program.

- Required: [[evidence/sicp/sicp.pdf#page=569|the §4.2 introduction and §§4.2.1–4.2.2, printed pp. 541–555 / PDF pp. 569–583]]. Focus on the unused argument in `try`, the expression-and-environment representation of a thunk, the `actual-value` forcing points, and memoization.
- Optional: [[evidence/sicp/sicp.pdf#page=583|§4.2.3, printed pp. 555–558 / PDF pp. 583–586]]. It rebuilds streams as ordinary lazy lists. Read it after Seminar 8 if you want to compare explicit `delay`/`force` with language-wide non-strict procedure arguments.
- Required: [[evidence/sicp/sicp.pdf#page=587|the §4.3 introduction and §4.3.1, printed pp. 559–567 / PDF pp. 587–595]]. Separate the set of choices expressed by `amb` from this evaluator's left-to-right, depth-first search order.
- Skim with this synopsis: [[evidence/sicp/sicp.pdf#page=595|§4.3.2, printed pp. 567–578 / PDF pp. 595–606]] applies the same generate-and-require interface to a multiple-dwelling puzzle, a yacht puzzle, and ambiguous natural-language parsing. The important transfer is that compact constraints can describe acceptable answers while the evaluator still commits to an operational enumeration order; the parser also relies on reversible assignment to its remaining-input cursor. Read one complete example and the discussion around operand order, but do not treat puzzle elegance as evidence of efficient or complete search.
- Required: [[evidence/sicp/sicp.pdf#page=606|§4.3.3, printed pp. 578–593 / PDF pp. 606–621]]. Follow the success and failure continuations through `analyze-amb`, then inspect why ordinary `set!` installs an undo action on the failure path.

## System-design problem

Eager execution can perform work whose value is never needed. A single-path evaluator can also entangle the description of acceptable results with hand-written retry control. Delaying work and representing alternatives make both programs more expressive, but they create new obligations: retain the environment needed by deferred work, define forcing and memoization, choose a search order, preserve a resumable choice point, and reverse state changes before exploring a sibling. Which of these obligations can illuminate branching agent sessions, and which remain properties of a language evaluator rather than of a harness?

## Vocabulary

- **Applicative order**: evaluate procedure arguments before entering the procedure body.
- **Normal order**: delay procedure arguments until their values are required by the computation.
- **Strict argument**: an argument whose value is obtained before the procedure body uses it; SICP's lazy evaluator keeps primitive procedures strict.
- **Non-strict argument**: an argument admitted to a procedure body without first computing its value; SICP makes every compound-procedure argument non-strict in this evaluator.
- **Thunk**: a delayed argument represented by its expression and the environment in which that expression must eventually be evaluated.
- **Force**: obtain the actual value of a thunk when a strict context needs it.
- **Memoized thunk**: a thunk rewritten to store its first computed value so later forces reuse that value and no longer need its saved expression or environment.
- **Choice point**: an `amb` expression whose alternatives provide possible continuations of the computation.
- **Requirement**: a predicate whose false case invokes `(amb)`, signaling that the current branch has no acceptable value.
- **Success continuation**: a procedure that receives a value and a failure continuation capable of resuming the search if use of that value later reaches a dead end.
- **Failure continuation**: a procedure that resumes at an alternative choice, undoes an intercepted assignment, propagates exhaustion to an earlier choice point, or reports that no choices remain.
- **Chronological backtracking**: return to the most recent open choice point and try its next alternative; SICP's evaluator combines this with left-to-right, depth-first search.
- **Search policy**: the operational rule for ordering, pruning, budgeting, and stopping exploration. It is not supplied by the declarative statement of acceptable values alone.

## Argument map

1. **CLAIM.** Applicative order and normal order differ in when a procedure argument is evaluated. In applicative-order Scheme, `(try 0 (/ 1 0))` evaluates the division before entry and errors; with lazy evaluation, `try` returns `1` because its second argument is never needed. [[evidence/sicp/sicp.pdf#page=570|§4.2.1, printed pp. 542–544 / PDF pp. 570–572]]. **INFERENCE.** Deferring a computation changes both its cost timing and whether its errors or effects occur at all. “Same procedure body” therefore does not imply the same observable behavior once strictness changes.
2. **CLAIM.** SICP's lazy evaluator passes operand expressions and their calling environment to compound procedures as thunks, while strict primitive applications, conditional predicates, operators, and the driver loop request actual values. A thunk must retain its environment so its expression is evaluated under the bindings that were in scope when the call occurred. [[evidence/sicp/sicp.pdf#page=572|§4.2.2 modifies application and identifies forcing points (printed pp. 544–549 / PDF pp. 572–577)]]. **INFERENCE.** A thunk is not merely an unevaluated syntax tree. It is suspended computation with captured context and a defined set of consumers that can demand its value.
3. **CLAIM.** The evaluator memoizes a forced thunk by storing the computed value and discarding the saved expression and environment; subsequent forcing returns the stored value. SICP explicitly notes that memoization interacts subtly with assignment. [[evidence/sicp/sicp.pdf#page=577|§4.2.2 represents memoized thunks and discusses their space behavior (printed pp. 549–554 / PDF pp. 577–582)]]. **INFERENCE.** Memoization is not only an optimization: it chooses whether repeated demand repeats computation and effects, and it changes which captured environment remains reachable after the first force.
4. **CLAIM.** `amb` lets a program state alternative values and rejection constraints without spelling out the traversal loop, but this abstraction does not determine one inevitable search order. The evaluator in the book chooses the first alternative at each point and chronologically backtracks depth-first; the text contrasts this with parallel or random exploration. [[evidence/sicp/sicp.pdf#page=589|§4.3.1 defines `amb`, failure, and its operational search strategy (printed pp. 561–567 / PDF pp. 589–595)]]. **INFERENCE.** The declarative answer set and the operational exploration policy are separate specifications. Order affects which answer appears first, whether an infinite branch starves siblings, and how much work occurs before success.
5. **CLAIM.** In the `amb` evaluator, every analyzed execution procedure receives an environment plus success and failure continuations. Success receives both a value and the failure continuation to invoke if later use of that value reaches a dead end; failure tries another branch or propagates exhaustion to an earlier choice point. [[evidence/sicp/sicp.pdf#page=606|§4.3.3 introduces the continuation protocol (printed pp. 578–583 / PDF pp. 606–611)]]. **INFERENCE.** Backtracking is executable control state, not a metaphor. The evaluator can resume because the outstanding continuation encodes what to do next at the correct prior point.
6. **CLAIM.** Backtracking over mutable state requires explicit compensation. SICP's analyzed `set!` saves the old variable value and installs a failure continuation that restores it before propagating failure; the section later distinguishes this reversible assignment from `permanent-set!`, whose effect is not undone on failure. [[evidence/sicp/sicp.pdf#page=608|§4.3.3 explains assignment rollback and introduces permanent assignment (printed pp. 580–592 / PDF pp. 608–620)]]. **INFERENCE.** A continuation restores only the state its implementation deliberately captures and compensates. It does not automatically undo output, network calls, subprocesses, file writes, money movement, or other external effects.

## Mechanism trace

### Trace A: an argument is delayed and never forced

Use the memoizing lazy evaluator from §4.2.2:

```scheme
(define (first a b) a)
(first (+ 20 22) (/ 1 0))
```

Let `E0` be the environment where `first` is defined and where the application is evaluated.

```text
1. Evaluate the operator `first` in E0.
   lookup -> compound procedure <parameters (a b), body a, defining env E0>
   The operator is needed for dispatch, so it is an actual procedure value.

2. Apply the compound procedure without evaluating either operand.
   T_a = thunk(expression = (+ 20 22), environment = E0, state = pending)
   T_b = thunk(expression = (/ 1 0), environment = E0, state = pending)
   Extend E0 with a call frame E1:
      a -> T_a
      b -> T_b

3. Evaluate the body `a` in E1.
   lookup a -> T_a
   No reference to b occurs; T_b remains pending.

4. The driver loop requests the application's actual value.
   force T_a:
      evaluate (+ 20 22) in saved E0
      primitive `+` is strict, so its numeric operands become 20 and 22
      result -> 42
      rewrite T_a as evaluated-thunk(value = 42)
      discard T_a's expression and saved E0 reference

5. Print 42.
   T_b was created but never forced.
   Therefore `/` is never invoked, division by zero never occurs, and T_b
   can become unreachable when the call frame E1 is no longer retained.
```

The expression datum `(/ 1 0)` existed and a thunk captured it; no runtime division value or error was produced. If the procedure body had referred to `b` in a strict context, its saved `E0` would have supplied the lookup context at that later time. Memoization matters only for a thunk that is actually forced.

### Trace B: failure resumes the preceding choice point

Trace this program under SICP's left-to-right operand evaluation and first-alternative, depth-first `amb` policy:

```scheme
(let ((x (amb 1 2 3))
      (y (amb 1 2)))
  (require (= (+ x y) 4))
  (list x y))
```

Write `F₀` for “no choices remain,” `Fₓ(n)` for the continuation that resumes with the remaining `x` alternatives beginning at `n`, and `Fᵧ(n; Fₓ)` for the continuation that resumes with the remaining `y` alternatives beginning at `n`. Even after the last `y` has been chosen, the evaluator installs `Fᵧ(∅; Fₓ)`: invoking that closure finds no inner alternative and explicitly propagates failure to `Fₓ`.

```text
choose x = 1
  succeed with 1 and failure continuation Fₓ(2)

  choose y = 1
    succeed with 1 and failure continuation Fᵧ(2; Fₓ(2))
    require (= (+ 1 1) 4) -> false -> (amb) -> fail

  invoke the most recent failure continuation Fᵧ(2; Fₓ(2))
    choose y = 2
    succeed with 2 and failure continuation Fᵧ(∅; Fₓ(2))
    require (= (+ 1 2) 4) -> false -> fail

  invoke Fᵧ(∅; Fₓ(2))
    no y alternative remains, so this closure invokes Fₓ(2)

  invoke Fₓ(2); failure now resumes at the preceding x choice point
    choose x = 2
    install Fₓ(3)

  create a fresh y choice under x = 2
    choose y = 1
    install Fᵧ(2; Fₓ(3))
    require (= (+ 2 1) 4) -> false -> fail

  invoke Fᵧ(2; Fₓ(3))
    choose y = 2
    succeed with 2 and failure continuation Fᵧ(∅; Fₓ(3))
    require (= (+ 2 2) 4) -> true
    succeed with (2 2), carrying Fᵧ(∅; Fₓ(3)) as the route to another answer

later try-again
  invoke the saved Fᵧ(∅; Fₓ(3))
    no y alternative remains, so this closure invokes Fₓ(3)
    choose x = 3 and create a fresh y choice under that binding
```

The result does not prove that `(2 2)` is the only answer. It is merely the first success under this evaluator's choice order. The saved `try-again` route is the exhausted inner continuation, not a direct pointer to `Fₓ(3)`; its invocation performs the propagation to the preceding choice point. If a branch had executed ordinary `set!`, the evaluator's assignment-specific failure continuation would need to restore the old binding before the next sibling ran.

## Rust lens

```rust
// Explanatory sketch; not compiled by this course.
enum Search<T> {
    Success {
        value: T,
        // Calling resume continues with the next alternative.
        resume: Box<dyn FnOnce() -> Search<T>>,
    },
    Exhausted,
}

struct Failure<T>(Box<dyn FnOnce() -> Search<T>>);

fn choose_then<T, U>(
    alternatives: Vec<T>,
    succeed: Box<dyn Fn(T, Failure<U>) -> Search<U>>,
    outer_fail: Failure<U>,
) -> Search<U> {
    // Concept only: select the first alternative and construct a Failure<U>
    // that captures the remaining alternatives plus outer_fail. A rejection
    // invokes that failure; exhaustion invokes outer_fail.
    todo!()
}
```

`Search<T>` makes “a current success plus a route to the next candidate” visible. A production Rust design would need explicit lifetimes, ownership of captured state, stack-safety, cancellation, fairness, and error types; `Box<dyn FnOnce>` is only a compact continuation-shaped lens.

An ordinary Rust iterator can enumerate candidates and combinators can filter them, but an iterator alone does **not** roll back external effects performed while producing a rejected item. A continuation does not do so automatically either. Reversal requires an explicit transaction, isolated snapshot, idempotent operation, or tested compensation action for every state transition that must be undone. Dropping a future, iterator, or closure is cancellation of further local computation, not proof that prior effects were reversed.

## Agent-harness bridge

**SICP CLAIM.** `amb` separates a program's alternative-and-constraint description from the evaluator's operational search order, and it resumes rejected work through explicit failure continuations whose rollback behavior covers only effects the evaluator has arranged to undo. [[evidence/sicp/sicp.pdf#page=589|§4.3.1, printed pp. 561–567 / PDF pp. 589–595]] [[evidence/sicp/sicp.pdf#page=606|§4.3.3, printed pp. 578–587 / PDF pp. 606–615]]

**HARNESS EVIDENCE — Pi.** At pinned Pi commit `4488ad55c18f07ae89a489096c90de8667b3adfb`, session entries are appended with the current leaf as `parentId`; the store append is awaited before the in-memory leaf advances. `moveTo` validates a target, appends a leaf-selection entry, and can then append a branch summary. [[evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts|`packages/agent/src/harness/session/session.ts:333–515`]] ([exact lines 333–515](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L333-L515)) For one already selected path, context construction finds the latest compaction entry, projects its summary and retained tail, and converts branch summaries and messages into model-visible messages. [[evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts|`packages/agent/src/harness/session/session.ts:61–149`]] ([exact lines 61–149](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L61-L149)) These ranges evidence parent-linked entries, a movable session leaf, ordered store-before-leaf advancement, and path projection. They do not establish durability guarantees for every store implementation, automatic branch scoring, or reversal of effects performed from either branch.

**HARNESS EVIDENCE — Codex.** At pinned Codex commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, a forked agent requires a parent spawn-call identity and a fork mode, resolves the parent thread, ensures queued rollout items are materialized, flushes the rollout, and loads the parent's model context. Last-N-turn mode truncates that context; full-history mode can retain its reference-context item depending on the compacted checkpoint, and the code defines a filter for inherited agent messages, multi-agent usage hints, and developer-instruction replacement. [[evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control/spawn.rs|`codex-rs/core/src/agent/control/spawn.rs:570–720`]] ([exact lines 570–720](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control/spawn.rs#L570-L720)) The following range applies that filter with `retain_mut` to top-level and compacted replacement histories, then passes the filtered `forked_rollout_items` to `fork_thread_with_source` with both parent and fork-source thread IDs. [[evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control/spawn.rs|`codex-rs/core/src/agent/control/spawn.rs:721–826`]] ([exact lines 721–826](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control/spawn.rs#L721-L826)) This is evidence for explicit parent identity and materialization of a selected, filtered shared prefix under a declared inheritance mode. The ranges do not show candidate evaluation, choice ordering, completeness, sibling merge, or rollback.

**INFERENCE.** Parent-linked entries and fork materialization can give candidate branches a shared-prefix identity and attributable lineage. They do not supply the missing search semantics: what objective ranks a candidate, which branch runs first, whether exploration is depth-first, breadth-first, parallel, stochastic, or adaptive, when a budget stops it, what evidence counts as failure, or how results merge. They also do not make sibling branches isolated from the same filesystem, subprocess, credential, service, or human-visible side effect. A branch can be historically attributable and still be unsafe to retry or discard.

**AGENT-HARNESS QUESTION — INFERENCE.** What explicit branch record should an agent harness use to bind parent lineage, inherited context range, evaluation objective, search policy, budget charge, effect-isolation boundary, terminal receipt, and merge decision—while refusing to label cancellation or branch rejection as rollback unless every relevant effect has a verified transaction or compensation path?

## Limits of the analogy

A lazy Scheme thunk is an expression plus a lexical environment governed by precise evaluator forcing rules. A queued tool call, deferred model request, cached context projection, or sleeping task has different identity, failure, and authority semantics. Delaying a tool does not grant permission to run it later, and memoizing one result is unsafe when the operation depends on changing world state.

An `amb` branch is also not an agent conversation. The book's evaluator knows its alternatives, chooses them in a defined order, and can restore evaluator-managed assignments. Agent branches may generate new actions dynamically, share external state, observe time-dependent results, consume non-refundable budget, and lack a decidable acceptance predicate. A recorded parent edge supplies provenance, not an evaluation objective, fair or complete search, deterministic replay, effect isolation, or a correct merge. Parallel fan-out is an execution policy, not evidence that all possible worlds were explored.

## Practice

### Manual trace

Perform one two-part ledger trace. First, under the memoizing lazy evaluator, trace `(keep-left (begin (set! counter (+ counter 1)) 7) (begin (set! counter (+ counter 100)) 9))`, where `(define (keep-left x y) (+ x x))` and `counter` initially equals zero. Record thunk creation, saved environments, every forcing request, each cache hit, each executed assignment, and which captured environment can become unreachable. Second, trace `(let ((a (amb 1 2 3)) (b (amb 2 3))) (require (= (* a b) 6)) (list a b))` under left-to-right depth-first search. Draw every installed success/failure continuation through the first success and one subsequent `try-again`; mark exactly where an exhausted inner choice propagates to the outer choice. Do not treat memoization as rollback or `try-again` as re-running the program from source.

### SICP exercises

- Exercise 4.27: fill in and explain the lazy-evaluator interaction involving `count`, `id`, and `w`. Purpose: test whether you can distinguish definition-time evaluation, thunk creation, first forcing, and memoized reuse in the presence of assignment. Invariants: use the evaluator's actual forcing points, track the same thunk rather than substituting call-by-value intuition, and record each mutation only when its expression is evaluated. The missing values are intentionally not supplied here. [[evidence/sicp/sicp.pdf#page=579|Exercise 4.27, printed p. 551 / PDF p. 579]]
- Exercise 4.44: write the requested nondeterministic program for the eight-queens puzzle. Purpose: separate the constraints defining a safe placement from the evaluator's enumeration of candidate placements. Invariants: every queen occupies a distinct row and column, no pair shares a diagonal, each rejection reaches a valid preceding choice point, and any pruning preserves the intended solution set. No program, ordering, or solution set is supplied here. [[evidence/sicp/sicp.pdf#page=599|Exercise 4.44, printed p. 571 / PDF p. 599]]

### Transfer prompt

Design a three-candidate coding-agent investigation that shares one evidence prefix. Specify the candidate-generation rule, evaluation objective, exploration order, per-branch and total budgets, cancellation behavior, filesystem and service isolation, authoritative receipts, and merge rule. For a branch that writes a file and then fails its evaluator, state whether the write is prevented, transactionally isolated, idempotently replayable, compensated, or permanent. Show how the parent can retain lineage and a rejected result without pretending that discarding model context undid the branch's world effects.

## Dialogue

**PROVISIONAL.** `What evidence would let you call a rejected agent branch safely retryable, rather than merely attributable to a parent and no longer selected for model context?`

This question is queued; [[knowledge/rsi/sicp/course/dialogue_state|the dialogue state]] still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [[knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics|Seminar 09 — Eval/apply and executable semantics]] · [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]] · Next: [[knowledge/rsi/sicp/course/seminars/11-logic-programming-and-declarative-query|Seminar 11 — Logic programming and declarative query]]

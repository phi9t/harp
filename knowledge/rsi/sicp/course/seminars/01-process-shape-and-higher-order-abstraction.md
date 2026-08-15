# Seminar 01 — Process shape and higher-order abstraction

## Reading route

Skim §1.1 for the vocabulary of expressions, combinations, and procedures. Read §§1.2–1.3 closely: they turn a procedure from a piece of notation into an account of a process and then show how to abstract over recurring process patterns.

- Skim: [[evidence/sicp/sicp.pdf#page=34|§1.1, printed pp. 6–39]].
- Required: [[evidence/sicp/sicp.pdf#page=68|§1.2, printed pp. 40–73]].
- Required: [[evidence/sicp/sicp.pdf#page=102|§1.3, printed pp. 74–106]].
- Optional: none assigned.

## System-design problem

Two implementations can agree on every returned value yet require different amounts of pending work, retained state, and time as inputs grow. How do we choose a representation of computation that makes those operational differences visible, and then package the useful pattern without erasing its invariant?

## Vocabulary

- **Computational process**: the evolving sequence of states and deferred operations produced while a procedure runs.
- **Recursive process**: a process that accumulates deferred operations to be completed later.
- **Iterative process**: a process whose complete future is summarized by a fixed number of explicit state variables.
- **Order of growth**: how required resources scale with problem size, expressed independently of a particular machine's clock rate.
- **Higher-order procedure**: a procedure that takes procedures as arguments or returns a procedure, exposing a reusable process pattern.
- **Invariant**: a relation over state that holds initially and is preserved by every transition.

## Argument map

1. **CLAIM.** A program is a description of a computational process, not merely a formula for a final value. [[evidence/sicp/sicp.pdf#page=29|Chapter 1 opens by distinguishing procedures from the processes they generate (printed p. 1; PDF p. 29)]]. **EVIDENCE.** The chapter immediately asks what a procedure's evaluation process is, which makes the intermediate work an object of analysis.
2. **CLAIM.** The substitution model exposes process shape by making expansions and contractions observable. [[evidence/sicp/sicp.pdf#page=46|§1.1.5 works through substitution and evaluation (printed pp. 18–22; PDF pp. 46–50)]]. **INFERENCE.** Counting the still-unperformed operations in an expansion gives a portable way to distinguish process shapes before discussing a runtime.
3. **CLAIM.** A recursive procedure does not entail a recursive process: its form can generate either a recursive or iterative process. [[evidence/sicp/sicp.pdf#page=69|§1.2.1 contrasts the two factorial processes (printed pp. 41–47; PDF pp. 69–75)]]. **EVIDENCE.** The same mathematical factorial relation is presented with deferred multiplications in one formulation and a fixed tuple of evolving values in the other.
4. **CLAIM.** Orders of growth describe scaling behavior, rather than elapsed time on one machine. [[evidence/sicp/sicp.pdf#page=82|§1.2.3 analyzes growth of processes (printed pp. 54–57; PDF pp. 82–85)]]. **INFERENCE.** A faster processor can change a constant factor, but it cannot turn linear growth into logarithmic growth.
5. **CLAIM.** Higher-order procedures make recurring process patterns composable by parameterizing the varying pieces. [[evidence/sicp/sicp.pdf#page=102|§1.3 develops higher-order abstractions (printed pp. 74–106; PDF pp. 102–134)]]. **EVIDENCE.** Procedures become values that can supply a term, a transformation, or a combining rule while the traversal structure stays reusable.

## Mechanism trace

Consider factorial for n = 5. The recursive formulation says `fact(n) = n × fact(n − 1)` with `fact(1) = 1`. Its calculation expands before it contracts:

```text
fact(5)
5 × fact(4)
5 × (4 × fact(3))
5 × (4 × (3 × fact(2)))
5 × (4 × (3 × (2 × fact(1))))
5 × (4 × (3 × (2 × 1)))
5 × (4 × (3 × 2))
5 × (4 × 6)
5 × 24
120
```

At the deepest point, four multiplications are deferred: `5 ×`, `4 ×`, `3 ×`, and `2 ×`. The amount of pending work grows with n, so this is a recursive process even though the definition is compact.

An iterative factorial procedure instead carries `(product, counter, max-count)`:

```text
start        (1,   1, 5)
after step   (1,   2, 5)   ; product = 1 × 1
after step   (2,   3, 5)   ; product = 1 × 2
after step   (6,   4, 5)   ; product = 2 × 3
after step   (24,  5, 5)   ; product = 6 × 4
after step   (120, 6, 5)   ; product = 24 × 5
stop: counter > max-count, return product = 120
```

Its invariant is `product = (counter − 1)!`, with `1 ≤ counter ≤ max-count + 1`; `max-count` remains the requested n. Each step preserves the invariant by multiplying by `counter` and then incrementing it. The final state has `counter = max-count + 1`, so the invariant yields `product = max-count! = 120`. Only the three named state values are needed at each step; there is no growing chain of deferred multiplications.

Both processes use Θ(n) steps. The recursive process retains Θ(n) deferred-control storage, whereas the accumulator process has Θ(1) explicit process state. These are process-model claims, not unconditional claims about physical call-stack use: an implementation's tail-call behavior determines whether the accumulator procedure also runs with constant stack space.

## Rust lens

The following is a process-shape sketch, not a claim that Rust and Scheme have the same execution semantics.

```rust
// Explanatory sketch; not compiled by this course.
fn fast_expt(mut base: u64, mut exponent: u64) -> u64 {
    let mut accumulator = 1;
    while exponent > 0 {
        if exponent % 2 == 0 {
            base *= base;
            exponent /= 2;
        } else {
            accumulator *= base;
            exponent -= 1;
        }
    }
    accumulator
}
```

The explicit state is `(base, exponent, accumulator)`. For inputs whose exact result and every intermediate value remain representable as `u64`, its invariant is `accumulator × base^exponent = original_base^original_exponent`; under that precondition, the loop exits at exponent zero with the answer in the accumulator. The sketch does not model overflow, so it makes no arithmetic-correctness claim for inputs outside that range. Rust's `while` makes this iterative control structure explicit, but that fact proves nothing about Scheme tail-call optimization: Scheme procedure form and a particular runtime's call-stack behavior are separate questions.

## Agent-harness bridge

**INFERENCE (course-design transfer).** This paragraph is course synthesis, not a SICP source claim. No pinned bridge is required here. The terminology—state tuple, deferred work, invariant, and order of growth—will be reused when the course reaches explicit agent loops, where a retry counter, budget, queue, and evidence record must be made inspectable rather than left implicit in recursion or callbacks.

## Limits of the analogy

A recursive mathematical definition does not determine runtime call-stack behavior. A language implementation can execute a tail-recursive process with constant control storage, while another runtime may retain stack frames; inspect the process and the implementation contract separately.

## Practice

### Manual trace

For n = 5, trace both recursive and iterative factorial. In the recursive trace, list the deferred multiplications at each expansion; in the iterative trace, list every `(product, counter, max-count)` state. Compare how the same result arises from deferred operations versus explicit state evolution, and state the iterative invariant in words.

### SICP exercises

- Exercise 1.9: use the two addition procedures to diagnose whether the recursive syntax produces a recursive or iterative process. For initial arguments `(a0,b0)`, check only these relations: after k recursive descents, the expression has shape `inc^k(+(a0-k,b0))` and `(a0-k)+b0+k=a0+b0`; after k iterative steps, it has shape `+(a0-k,b0+k)` and `a+b=a0+b0`. These are check relations, not a solution trace.
- Exercise 1.16: design an iterative exponentiation process; the purpose is to make a logarithmic-time transformation explicit, and the invariant relates the accumulator, current base, and remaining exponent to the original power.

### Transfer prompt

Specify a bounded retry operation as an explicit state machine. Name its state tuple, transition guard, success and terminal-failure states, the invariant that preserves the original request and retry budget, and the maximum number of attempts. Explain which deferred work would be hidden if the loop were written as direct recursion.

## Dialogue

**PROVISIONAL.** `When two procedures compute the same values, what evidence would convince you that they generate materially different processes?`

Continue from the [[knowledge/rsi/sicp/course/dialogue_state|dialogue state]], which records this seminar as the current question.

## Navigation

Previous: [[knowledge/rsi/sicp/course/scheme_reading_primer|Scheme reading primer]] · [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]] · Next: [[knowledge/rsi/sicp/course/seminars/02-data-abstraction-and-immutable-representation|Seminar 02 — Data abstraction and immutable representation]]

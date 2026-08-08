# SICP evaluator kernel — TECHNICAL DEEP DIVE

## Scope and falsifiable target

**EVIDENCE — active mode.** This document is in **TECHNICAL DEEP DIVE** mode.
It narrows the book-wide map to the strict evaluator kernel, lexical
environments, the split between analysis and execution, and the later lexical-
address optimization.

**EVIDENCE — source slice.** The primary anchors are `SICP-2E-UF` §3.2,
§4.1–§4.1.7, and §5.5.6 (printed pp. 320–340, 492–540, and 817–822; PDF
pp. 348–368, 520–568, and 845–850).

**INFERENCE — falsifiable target.** A model has captured this slice only if it
can evaluate nested applications, preserve lexical rather than dynamic scope,
evaluate only the selected `if` branch, preserve quoted syntax as data, reject
arity mismatches, and expose the cost of environment lookup.

## Formal kernel

**CLAIM — evaluation function.** The evaluator can be summarized as a partial
mapping `Eval : Expr × Env → Value`; it is partial because malformed forms,
unbound variables, and invalid applications signal errors (`SICP-2E-UF`,
§4.1.1–§4.1.4, printed pp. 495–521, PDF pp. 523–549).

**CLAIM — application function.** Procedure application is a second partial
mapping `Apply : Proc × Value* → Value`. A primitive procedure crosses into the
host runtime; a compound procedure extends its captured environment with
parameter–argument bindings and evaluates its body there (`SICP-2E-UF`, §4.1.1
and §4.1.4, printed pp. 498–500 and 518–520, PDF pp. 526–528 and 546–548).

**CLAIM — core transition rules.** The source's evaluator distinguishes these
cases (`SICP-2E-UF`, §4.1.1, printed pp. 495–500, PDF pp. 523–528):

1. **CLAIM:** A self-evaluating literal returns itself; a variable resolves
   through its environment; quotation returns unevaluated datum structure.
2. **CLAIM:** `if` evaluates its predicate and exactly one branch.
3. **CLAIM:** Evaluating `lambda` packages parameters, body, and the current
   environment into a compound procedure.
4. **CLAIM:** An ordinary combination evaluates operator and operands, then
   passes the resulting procedure and argument values to `Apply`.
5. **CLAIM:** Applying a compound procedure creates a frame binding formals to
   actuals, links it to the procedure's saved environment, and evaluates the
   body in that extended environment.

**INFERENCE — semantic fixed point.** `Eval` reduces syntax to values and
procedures; `Apply` turns procedures and values back into an evaluation problem
under a new environment. The mutually recursive boundary is the mechanism
behind the source's eval–apply cycle diagram (`SICP-2E-UF`, Figure 4.1, printed
p. 494, PDF p. 522).

## Executable algorithm and systems realization

**EVIDENCE — runnable companion.** The dependency-free Rust crate at
[`labs/sicp-evaluator/`](../../../labs/sicp-evaluator/) implements the
kernel as `Evaluator::eval` and `Evaluator::apply`; it includes an S-expression
parser, mutable environment frames, lexical closures, primitives, special
forms, instrumentation, and six behavior tests.

```text
read source
  → parse into Expr
  → Eval(expr, env)
      literal              → value
      symbol               → lookup(env, name)
      special form         → form-specific transition
      ordinary list        → Eval(operator), Eval(each operand), Apply(...)
  → Apply(proc, values)
      primitive            → host function(values)
      closure              → Eval(body, extend(captured_env, params, values))
  → value or explicit error
```

**INFERENCE — executable invariant.** The one field that makes lexical scope
work is the closure's saved environment. Replacing it with the caller's
environment would produce dynamic scope and fail the companion's
`closures_use_the_environment_captured_at_definition` test.

**EVIDENCE — deliberate representation choice.** The companion stores each
frame as a linear vector of name–value pairs so that instrumentation preserves
the source's deep-binding search behavior. It does not silently replace the
teaching representation with a hash table.

### Definition-time versus call-time environment trace

**EVIDENCE — test program.** The executable trace defines global `x = 10`,
defines `add-x = λ(y). x + y`, and then calls `add-x 1` from inside another
closure whose local `x = 100`.

```text
G:  x=10, add-x=<closure env=G>
│
└─ Fcaller: x=100

Apply add-x(1):
G
└─ Fadd-x: y=1
   lookup y → 1
   lookup x → parent G → 10
   result → 11
```

**EVIDENCE — observed result.** `cargo run --quiet --manifest-path
labs/sicp-evaluator/Cargo.toml` returns `11` for the final expression and
reports 10 expression dispatches, 3 applications, 7 visited frames, and 16
binding comparisons. The corresponding unit test passes.

**INFERENCE — counterfactual.** Under dynamic scope, the `x` lookup would
follow the caller frame and produce 101. The observed 11 therefore
distinguishes the lexical-environment mechanism from that alternative; it is
not merely a generic arithmetic test.

## Representation boundaries

| Label | Boundary | `SICP-2E-UF` | Rust companion | Consequence |
|---|---|---|---|---|
| EVIDENCE | Implementation language | Scheme evaluating Scheme, hence metacircular | Rust evaluating a Scheme subset | The companion tests semantics but is not itself metacircular |
| EVIDENCE | Expression representation | Lists plus abstract syntax predicates/selectors | Typed `Expr` enum produced by a small parser | Parser and evaluator are more sharply separated in the companion |
| EVIDENCE | Frame representation | Parallel variable/value lists in §4.1.3 | Linear vector of name–value pairs | Both make within-frame lookup linear; mutation details differ |
| EVIDENCE | Forms | Broad teaching evaluator with derived forms and exercises | Numbers, booleans, symbols, quote, if, define, lambda, begin, application | Unsupported Scheme behavior is outside the lab's claim boundary |
| EVIDENCE | Operand order | The source evaluator delegates operand traversal to its Scheme implementation | Explicit left-to-right loop | Programs depending on operand side effects are not cross-implementation evidence |
| EVIDENCE | Control | Host Scheme call/return remains implicit until Ch. 5 | Host Rust call stack remains implicit | Neither implementation demonstrates an explicit-control evaluator |
| EVIDENCE | Tail calls and storage | Addressed later by explicit control, compilation, allocation, and garbage collection | Not implemented | Constant-space tail recursion and GC behavior are MISSING |

## Quality and performance model

**CLAIM — direct-evaluator cost.** The source calls the direct evaluator simple
but inefficient because it repeats syntactic classification and decomposition
every time an expression executes (`SICP-2E-UF`, §4.1.7, printed pp. 534–535,
PDF pp. 562–563).

**CLAIM — analysis/execution split.** The source's `analyze` phase converts an
expression once into an environment-taking execution procedure; repeated calls
reuse that procedure (`SICP-2E-UF`, §4.1.7, printed pp. 535–539, PDF
pp. 563–567).

**INFERENCE — repeated-execution model.** Let `A` be one-time syntax analysis,
`X` be environment-dependent execution, `H` be added execution-procedure
overhead, and `r` be the number of executions of unchanged syntax. Then
`T_direct(r) ≈ r(A + X)`, while `T_analyzed(r) ≈ A + r(X + H)`. The split wins
when the avoided repeated analysis, `(r − 1)A`, exceeds `rH`; the source gives
the qualitative optimization but no values for `A`, `X`, or `H`.

**CLAIM — name lookup cost.** The teaching environment representation searches
bindings in the current frame and then enclosing frames; the source says this
deep-binding strategy is not production-efficient and points to lexical
addressing (`SICP-2E-UF`, §4.1.3 note 15, printed pp. 515–517, PDF pp. 543–545).

**INFERENCE — lookup model.** If a reference resolves at depth `d` and frame
`j` contains `bⱼ` bindings scanned before resolution or rejection, worst-case
name comparisons are `∑ⱼ₌₀ᵈ bⱼ`. The companion's counters make this particular
cost observable rather than collapsing it into wall-clock noise.

**CLAIM — lexical-address optimization.** Section 5.5.6 lets the compiler encode
a binding's frame depth and displacement, replacing repeated name search with
known-coordinate access (`SICP-2E-UF`, printed pp. 817–822, PDF pp. 845–850).

**INFERENCE — cost boundary.** A lexical address removes symbol comparisons but
does not by itself guarantee constant-time access: the exact cost still depends
on how frames and enclosing links are represented. The source does not provide
a hardware-neutral asymptotic or measured latency for this access, so both are
**MISSING**.

## Direct, analyzed, and compiled paths

| Label | Axis | Direct evaluator | Analyzed evaluator | Compiler with lexical addressing |
|---|---|---|---|---|
| CLAIM | Representation | Source expression plus runtime environment | Execution procedure plus runtime environment | Register-machine instruction sequence plus lexical coordinates |
| CLAIM | Downstream consumer | `eval`/`apply` | Execution procedures and `execute-application` | Explicit evaluator machine/runtime |
| CLAIM | Objective | Semantic clarity and modifiability | Avoid repeated syntax analysis | Avoid interpretive dispatch and repeated binding-name search |
| CLAIM | Resolution path | Reclassify syntax on each visit | Analyze once, invoke many times | Compile once, execute instruction sequence |
| INFERENCE | Evaluation identity | Value under evaluator rules | Same intended value under factored rules | Same intended value under compiled/runtime contract |
| EVIDENCE | Quantitative evidence | MISSING in fetched source | Exercise 4.24 requests experiments; results are MISSING | End-to-end compiler speedups are MISSING |

## Failure modes and falsification plan

| Label | Failure | Observable signal | Existing or next check |
|---|---|---|---|
| INFERENCE | Dynamic-scope regression | Lexical example returns 101 instead of 11 | Existing lexical-closure unit test |
| INFERENCE | Eager branch bug | `(if #t 7 missing)` reports an unbound variable | Existing selected-branch unit test |
| INFERENCE | Code/data collapse | Quoted list attempts to call its first symbol | Existing quote-as-data unit test |
| INFERENCE | Silent argument mismatch | Extra/missing arguments are ignored or misbound | Existing arity-error unit test |
| INFERENCE | Lookup-cost opacity | Nested lookup succeeds but search work cannot be observed | Existing frame/comparison counters |
| SPECULATION | Analysis split gives material speedup in this Rust lab | Repeated parsed/evaluated workload is slower than a future pre-analyzed form | MISSING benchmark and analyzed implementation |
| SPECULATION | Lexical coordinates dominate hash lookup for small lexical frames | Coordinate implementation wins under controlled depth/width sweeps | MISSING implementation and benchmark |

## What the mechanism teaches

**INFERENCE — central lesson.** The evaluator is an executable semantic
boundary: syntax is data until `Eval` classifies it; a procedure is data until
`Apply` realizes it; and lexical scope is data because the defining environment
is stored inside the closure.

**INFERENCE — systems lesson.** The same boundary exposes an optimization path.
First hoist syntax-only work out of repeated execution; then replace repeated
symbolic lookup with coordinates; finally make control and storage explicit.
This is a compact instance of the book's larger method: preserve meaning while
changing the representation at the layer where cost becomes visible.

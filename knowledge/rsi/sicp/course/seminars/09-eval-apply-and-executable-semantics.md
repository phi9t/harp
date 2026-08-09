# Seminar 09 — Eval/apply and executable semantics

## Reading route

Read §4.1 closely. It constructs an evaluator in the language being evaluated, then factors repeated syntactic work into an analysis stage. Keep four objects separate while reading: source characters, expression data, runtime values, and analyzed execution procedures.

- Required: [§4.1, printed pp. 492–540 / PDF pp. 520–568](../../../../../evidence/sicp/sicp.pdf#page=520).
- Optional: none assigned.
- Skim: none assigned. The complete §4.1 range is required because the
  evaluator's syntax, environment, procedure, and analysis boundaries depend
  on the implementation details.

## System-design problem

A notation becomes executable only after several boundaries assign it structure, meaning, authority, and effects. Where exactly do reading, evaluation, application, lexical scope, and analysis meet, and which of those boundaries transfer usefully to an agent harness without pretending that a language model is a Scheme evaluator?

## Vocabulary

- **Source characters**: the textual input consumed by a reader, such as the characters in `"(+ 1 (* 2 3))"`.
- **S-expression notation**: the parenthesized written notation used to spell or print symbolic data; the notation is not itself the in-memory object.
- **Expression datum**: structured data produced by `read` and classified by the evaluator's syntax predicates and selectors.
- **Runtime value**: the result of evaluation, such as a number, primitive procedure, or compound procedure carrying an environment.
- **Environment**: a sequence of frames that maps names to values and supplies the context for evaluation.
- **Procedure value**: a primitive supplied by the host or a compound procedure containing parameters, body, and its defining environment.
- **Execution procedure**: the environment-taking procedure produced by `analyze`; unlike the direct evaluator's expression datum, this is a genuinely IR-like executable stage.
- **Metacircular evaluator**: an evaluator written in the same language whose semantics it describes.

## Argument map

### Claim unit 1 — Reading crosses from characters to data

**CLAIM.** `read` maps source characters to expression data before the evaluator sees the program. In the driver loop, the next complete expression returned by `read` is passed with the global environment to `eval`. [§4.1.4 note 18, printed p. 520 / PDF p. 548](../../../../../evidence/sicp/sicp.pdf#page=548)

**EVIDENCE.** The note explains that typing `(+ 23 x)` yields a list containing the symbol `+`, the number `23`, and the symbol `x`; abbreviated quotation is likewise expanded into list structure by the reader.

**INFERENCE.** The characters, their s-expression notation, and the resulting expression datum can look similar when printed, but they are different objects at different boundaries. Evaluation begins from structured data, not directly from keystrokes.

**MISSING.** This evaluator section does not specify a complete character-level reader algorithm or grammar, so it cannot settle every lexical or reader-extension design.

### Claim unit 2 — Syntax operations define expression representation

**CLAIM.** Syntax predicates and selectors define the representation that `eval` consumes: they recognize self-evaluating expressions, variables, quotations, assignments, definitions, conditionals, lambdas, sequences, and applications, then expose each form's parts. [§4.1.2, printed pp. 501–507 / PDF pp. 529–535](../../../../../evidence/sicp/sicp.pdf#page=529)

**EVIDENCE.** The evaluator calls operations such as `variable?`, `quoted?`, `assignment?`, `lambda?`, `application?`, `operator`, and `operands` rather than scattering list access throughout its control logic.

**INFERENCE.** These operations form a representation barrier. Changing the datum representation can be localized behind them if their semantic contract remains stable; adding a new expression form changes the evaluator's classification and transition rules.

**MISSING.** The abstraction barrier does not by itself prove that every representation change is local: a reader, printer, macro system, or error reporter may also depend on the chosen form.

### Claim unit 3 — Eval maps expression and environment to value

**CLAIM.** `eval` maps an expression together with an environment to a runtime value. It classifies the expression, performs the form-specific rule, and recursively evaluates the subexpressions that rule requires. [§§4.1–4.1.1, printed pp. 492–498 / PDF pp. 520–526](../../../../../evidence/sicp/sicp.pdf#page=520)

**EVIDENCE.** A variable triggers environment lookup, a quotation returns its datum, a conditional evaluates its predicate and only the selected branch, a lambda creates a procedure value, and an ordinary application evaluates its operator and operands.

**INFERENCE.** A useful compact signature is `eval : expression datum × environment → runtime value`. It is a partial mapping because malformed forms, unbound variables, and invalid applications can report errors rather than produce values.

**MISSING.** This semantic signature hides operand-order, error-reporting, tail-call, and storage behavior; those require the evaluator and runtime contracts around the kernel.

### Claim unit 4 — Apply consumes values, not syntax

**CLAIM.** `apply` consumes a procedure value and already-evaluated argument values. For an ordinary combination, `eval` first evaluates the operator and operands and only then transfers those results to `apply`. [§4.1.1, printed pp. 497–498 / PDF pp. 525–526](../../../../../evidence/sicp/sicp.pdf#page=525)

**EVIDENCE.** The source's `apply` branches on primitive versus compound procedure values. Its input is not the original application datum: the procedure and argument expressions have crossed the evaluation boundary.

**INFERENCE.** A useful compact signature is `apply : procedure value × argument values → runtime value`. Special forms cannot be reduced to this rule without care because they control whether or where their operands are evaluated.

**MISSING.** The high-level cycle does not establish one universal operand evaluation order; the source explicitly leaves that order to the underlying Lisp.

### Claim unit 5 — Closures save context; primitives cross the host boundary

**CLAIM.** A compound procedure contains its parameters, body, and defining environment. Applying it extends that saved environment with parameter-to-argument bindings and evaluates the body there. Primitive procedures instead cross into operations supplied by the underlying host Lisp. [§4.1.3, printed pp. 512–517 / PDF pp. 540–545](../../../../../evidence/sicp/sicp.pdf#page=540) [§4.1.4, printed pp. 518–520 / PDF pp. 546–548](../../../../../evidence/sicp/sicp.pdf#page=548)

**EVIDENCE.** The compound-procedure representation stores an environment, while primitive objects are installed in the initial environment and dispatched through the host's primitive-application mechanism.

**INFERENCE.** The saved environment is the mechanism for lexical scope, not incidental metadata. The primitive branch also marks the floor of metacircularity: the Scheme evaluator eventually relies on behavior implemented outside itself.

**MISSING.** The chapter's small evaluator does not model production concerns such as capability policy, operating-system permissions, sandbox enforcement, resource accounting, or revocation.

### Claim unit 6 — Analyze introduces the genuinely IR-like stage

**CLAIM.** `analyze` consumes an expression datum and produces an execution procedure that later consumes an environment. This separates syntax analysis from environment-dependent execution and avoids repeating the same classification work on every visit. [§4.1.7, printed pp. 534–539 / PDF pp. 562–567](../../../../../evidence/sicp/sicp.pdf#page=562)

**EVIDENCE.** Each analyzer recursively analyzes its subexpressions once and combines their execution procedures; the resulting procedure performs lookup, branching, sequencing, or application when given an environment.

**INFERENCE.** `analyze : expression datum → execution procedure` is closer to an IR-producing phase than direct `eval`. The direct evaluator interprets the expression datum; it does not first turn every expression into a separate executable representation.

**MISSING.** The source gives a qualitative cost argument and [Exercise 4.24 requests measurement, printed p. 541 / PDF p. 569](../../../../../evidence/sicp/sicp.pdf#page=569), but no measured speedup or portable performance envelope is established here.

## Mechanism trace

The first trace marks every representation boundary:

```text
"(+ 1 (* 2 3))" --read--> expression datum --eval in E--> proc/arg values --apply--> 7
```

The left side is source text: a sequence of characters. `(+ 1 (* 2 3))` is the s-expression notation that denotes—and can print—the nested expression datum, but the datum itself is structured list-and-atom data. Direct `eval` classifies that datum in environment `E`. For the outer application it evaluates `+` to a procedure value, `1` to a numeric runtime value, and `(* 2 3)` through another eval/apply cycle to the runtime value `6`. `apply` receives the primitive `+` value and argument values `[1, 6]`, then produces the runtime value `7`. It never receives the source string or the entire outer expression datum.

The analyzed evaluator adds a distinct stage:

```text
expression datum --analyze--> execution procedure --invoke with E--> runtime value
```

That execution procedure is genuinely IR-like: syntax-dependent work has been factored into a reusable executable object. Calling the direct evaluator's expression datum “the IR” blurs this additional transformation.

Lexical scope depends on which environment a compound procedure saves:

```text
G:       x = 10
         add-x = <closure params=(y), body=(+ x y), saved-env=G>

Fcaller: x = 100, parent = G

call add-x(1) from Fcaller
  apply closure with saved environment G
  extend G with Fadd-x: y = 1
  lookup y -> 1 in Fadd-x
  lookup x -> 10 in G
  apply + to [10, 1] -> 11
```

The result is `11`, not `101`. Using `Fcaller` instead of the closure's saved `G` would implement dynamic scope and would cross a different semantic boundary.

## Rust lens

The existing [metacircular evaluator deep dive](../../metacircular_evaluator_deep_dive.md) gives the fuller kernel, representation, cost, and falsification account. The runnable [`labs/sicp-evaluator/` crate](../../../../../labs/sicp-evaluator) is a semantic supplement: it models a Scheme subset's eval/apply cycle, lexical closures, selected special forms, primitive procedures, and observable linear environment lookup in Rust.

Because Rust is not the language being interpreted, the crate is not metacircular. It also has no analyzed evaluator and no compilation stage, so it cannot demonstrate §4.1.7's execution-procedure split or the compiler path developed later in SICP. Its six tests are useful evidence for the modeled semantic invariants, not evidence that it implements full Scheme.

## Agent-harness bridge

**EVIDENCE.** In the pinned OpenAI Codex source at commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, response-item processing is located at [`codex-rs/core/src/session/turn.rs:149–520`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L520), and tool-call normalization, lookup, and dispatch through `ToolRouter` are located at [`codex-rs/core/src/tools/router.rs:31–289`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289).

**INFERENCE (course-design transfer).** At that pinned commit, a useful boundary model is:

```text
model text/response item -> structured tool call -> resolved runtime capability -> effect/result
```

The transfer is the separation of representations and consumers. Model output can denote a requested operation without being the runtime capability that performs it; structured-call processing establishes a machine-readable request, and `ToolRouter` resolves and invokes registered behavior. This resembles the discipline of separating expression data, semantic evaluation, procedure values, and application, but it is a systems-design inference from the exact source locators above—not a claim made by SICP.

**AGENT-HARNESS RESEARCH QUESTION — INFERENCE.** How can a agent harness preserve and expose the distinctions among a parsed structured action, a resolved capability, an authorization decision, and execution, so evidence from one boundary cannot be mistaken for evidence that a later boundary was crossed?

## Limits of the analogy

An LLM is not `eval`: it does not implement SICP's deterministic expression-and-environment mapping. A tool runtime is not literally `apply`: its inputs, dispatch rules, failures, and effects follow the harness contract rather than Scheme procedure semantics. Most importantly, host permission and sandbox authority have no direct analogue in the small evaluator. A model-visible name or structured call does not itself grant authority; capability registration, policy, sandboxing, and operating-system enforcement remain separate runtime boundaries.

## Practice

### Manual trace

Trace `((lambda (f) (f 3)) (lambda (x) (+ x 4)))` from its expression datum. For each application, record the current environment, the operator expression, the resulting procedure value, each operand's runtime value, the environment extended by compound `apply`, and the final value. Mark precisely where syntax stops being consumed and values become the inputs to `apply`; do not collapse the two lambdas into source strings after `read` has produced data.

### SICP exercises

- Exercise 4.6: add `let` as a derived expression to test the syntax/evaluator representation barrier. Purpose: decide which syntax predicates, selectors, and transformations are responsible for the new form. Invariant: evaluating the transformed application in the same environment must preserve the bindings, lexical scope, body order, and resulting value of the `let` form.
- Exercise 4.24: compare the direct and analyzed evaluators to test the claimed benefit of separating analysis from execution. Purpose: isolate repeated syntax-processing work from environment-dependent work. Invariant: both evaluators must produce the same values and errors on the chosen supported programs before any timing or operation-count comparison is interpreted.

### Transfer prompt

Choose one agent-harness tool request and specify four separate boundaries: parsing model output into a structured request, interpreting that request under the harness protocol, resolving an authorized runtime capability, and invoking it to obtain an effect or result. For each boundary, name its input representation, output representation, failure mode, and authority. Which bug would arise if any two stages were treated as the same object?

## Dialogue

**PROVISIONAL.** `Given one tool request, what concrete artifact exists at the source-character or model-text boundary, at the expression-datum or structured-call boundary, at the runtime-value or resolved-capability boundary, and exactly what is handed to the apply-like invocation step?`

## Navigation

Previous: [Seminar 08 — Streams, delay, and infinite processes](08-streams-delay-and-infinite-processes.md) · [Course guide](../sicp_course_guide.md) · Next: [Seminar 10 — Lazy evaluation and nondeterministic search](10-lazy-evaluation-and-nondeterministic-search.md)

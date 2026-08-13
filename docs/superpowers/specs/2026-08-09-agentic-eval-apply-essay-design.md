# Agentic eval/apply essay design

## Objective

Add a canonical, MTS-level essay that turns the SICP `eval`/`apply`
relationship into an engineering model for agentic execution. The essay must
make the mutually recursive relationship precise without claiming that an LLM
is literally an evaluator, a tool call is literally Scheme `apply`, or the
existing harnesses implement a shared formal semantics.

## Authority and location

- Canonical technical prose: `content/sicp/agentic_eval_apply.md`.
- Navigation updates: `content/sicp/sicp_index.md` and
  `content/sicp/course/sicp_course_guide.md`.
- Derived artifacts: `atlas/src/content/generated/corpus.json` and
  `atlas/dist/harp-atlas.html`, regenerated together from canonical inputs.
- Evidence: reuse the checked-in SICP source and the existing pinned Pi,
  Hermes, and Codex overlays. Do not add, edit, or infer new source evidence.

## Audience and scope

The reader is MTS-level: comfortable with SICP's evaluator, closure, special
form, environment, and mutation vocabulary, and looking for an architecture
model rather than introductory Scheme instruction.

The essay is a source-aware synthesis. It may label a transfer conclusion as
`INFERENCE`, but it must distinguish this from:

- `CLAIM` about SICP's evaluator;
- `EVIDENCE` about a specific pinned implementation; and
- `MISSING` evidence needed for stronger runtime or reproduction claims.

It must not prescribe a production implementation, change the evaluator lab,
or claim benchmark results.

## Essay structure

### 1. Thesis and analogy boundary

Present an agent runtime as a partially observable, effectful evaluator. State
the analogy limit: the runtime interprets representation and control policy
while a probabilistic model proposes content inside that runtime.

### 2. Mutual recursion

Define the two operations:

```text
agentic_eval(program_or_instruction, context) -> value | action | observation
agentic_apply(capability, arguments, context) -> observation
```

Show:

- evaluation can apply read capabilities to acquire relevant evidence;
- application of a compound capability evaluates its body in an extended
  context; and
- approval, retry, cancellation, branch, and transaction are special forms
  with deliberately non-eager evaluation rules.

### 3. Context and materialization

Separate:

- immutable turn snapshots;
- durable event history;
- model-visible context projection;
- capability registry and policy;
- budgets and lineage; and
- external world state.

Reject the premise that context is always fully loaded. Treat it as a
provenance-bearing, demand-driven materialization of relevant state.

### 4. Intent-to-invocation pipeline

Trace a structured flow:

```text
model events
  -> structured action
  -> normalized identity
  -> resolved capability
  -> action intent
  -> effect attempt
  -> action receipt
  -> new observation
```

Use existing Pi, Hermes, and Codex deep dives only to support the cited
implementation-level observations. The unified pipeline remains an
`INFERENCE`.

### 5. Effects, uncertainty, and recovery

Explain the engineering delta from SICP's compact evaluator:
authorization, permission checks, persistence-before-effect, idempotency,
asynchronous completion, ambiguous outcomes after crashes, reconciliation,
compensation, and receipt-backed completion claims.

### 6. Compound procedures and delegation

Map skills, workflows, and subagents to compound procedures. Cover
context inheritance, isolated workspaces, child lineage, budget delegation,
cancellation, and the fact that a child report is an observation requiring
validation, not automatically a trusted return value.

### 7. Two meanings of evaluation

Separate execution-time `agentic_eval` from the practice called "agentic
evaluation": benchmark/evaluator/selector infrastructure surrounding an agent
run. Explain why conflation creates faulty claims about capability, safety, or
self-improvement.

### 8. Failure taxonomy and review invariants

Include reviewable errors:

- LLM output treated as semantic authority;
- visible tool schema treated as executable authority;
- old context treated as current world state;
- uncertain mutation retried as if it had failed;
- import treated as harmless validation;
- cancelled work treated as rolled-back work.

End with concrete invariants and architecture-review questions.

### 9. Worked code-repair trace

Provide one bounded trace showing nested `agentic_eval -> agentic_apply ->
agentic_eval`: evidence acquisition, a mutation with intent/receipt boundary,
test execution, and an observation becoming input to a later evaluation step.

## Navigation changes

The SICP index will link the new essay from:

- the evaluation and language-design route;
- the agent-harness design route; and
- quick links.

The course guide will include the essay as an optional MTS supplement after
Seminar 9 and before the capstone, so it extends rather than replaces the
existing sequential course.

## Validation

After the content and navigation updates:

1. run `cargo run -p harp -- check`;
2. regenerate the corpus with `cargo run -p harp -- build`;
3. regenerate the Atlas export with the repository's existing Atlas export
   command;
4. run `cargo run -p harp -- build --check`;
5. run `cd atlas && corepack pnpm run test:export`; and
6. run `mise run verify` before any commit.

No commit is included in this design because it requires explicit user
authorization.

## Self-review

- No placeholders, unresolved choices, or duplicate authorities remain.
- The essay is limited to canonical Markdown and navigation changes; generated
  artifacts remain derived.
- The design preserves evidence ceilings and does not equate agent systems
  with SICP's evaluator.
- The validation sequence includes both canonical corpus and offline export
  checks.

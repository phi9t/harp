# Agentic Eval/Apply Essay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish an MTS-level canonical SICP essay that models agentic execution as effectful, partially observable `eval`/`apply`, makes the analogy limits explicit, and is available in the checked-in Atlas.

**Architecture:** Add one canonical document under `content/sicp/`, then expose it through the SICP index and course guide. The document reuses existing SICP and pinned-harness evidence links, while treating the unified agent-runtime semantics as an inference. Regenerate the corpus and offline Atlas export only after canonical Markdown is complete.

**Tech Stack:** Markdown, existing Harp Rust corpus compiler, React/TypeScript Atlas export, `mise`.

---

## File structure

- Create: `content/sicp/agentic_eval_apply.md` — canonical MTS essay.
- Modify: `content/sicp/sicp_index.md` — navigation from semantic and agent-harness routes.
- Modify: `content/sicp/course/sicp_course_guide.md` — sequential and concept-route placement.
- Modify: `atlas/src/content/generated/corpus.json` — generated corpus.
- Modify: `atlas/dist/harp-atlas.html` — generated offline reader.
- Modify: `atlas/dist/harp-atlas.receipt.json` — generated export receipt.

## Task 1: Write the canonical agentic eval/apply essay

**Files:**
- Create: `content/sicp/agentic_eval_apply.md`
- Read: `content/sicp/course/seminars/09-eval-apply-and-executable-semantics.md`
- Read: `content/sicp/course/capstone/agent_harness_architecture_dossier.md`
- Read: `content/pi_harness_deep_dive.md`
- Read: `content/hermes_harness_deep_dive.md`
- Read: `content/codex_harness_deep_dive.md`

- [ ] **Step 1: Create the essay with an explicit semantic boundary**

Write the opening that distinguishes:

```text
SICP evaluator: expression data + lexical environment -> value
Agent runtime: instruction/action data + policy-governed context -> value,
action proposal, or observation
```

State that the transfer is an `INFERENCE`; an LLM proposes content but does
not itself attest parsing, authorization, invocation, completion, or the
freshness of external state.

- [ ] **Step 2: Define the mutually recursive operations**

Include the following skeletal semantics and explain each branch:

```text
agentic_eval(Call(capability_expression, argument_expressions), context):
  capability = agentic_eval(capability_expression, context)
  arguments = evaluate_arguments(argument_expressions, context)
  return agentic_apply(capability, arguments, context)

agentic_apply(primitive_capability, arguments, context):
  intent = admit_effect(primitive_capability, arguments, context)
  receipt = invoke_and_observe(intent)
  return record_observation(receipt, context)

agentic_apply(compound_capability, arguments, context):
  child_context = extend_context(compound_capability, arguments, context)
  return agentic_eval(compound_capability.body, child_context)
```

Explain that read capabilities may materialize needed evidence during
evaluation, while skills, workflows, and subagents act as compound
capabilities whose bodies require another evaluation.

- [ ] **Step 3: Define context materialization and special forms**

Separate immutable turn snapshot, append-only session events, context
projection, capability registry, policy/budget, lineage, and re-observable
world state. Define approval, retry, branch, cancellation, and transaction
as special forms with non-eager evaluation rules.

- [ ] **Step 4: Derive the intent-to-invocation and effect sequence**

Use this exact pipeline:

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

Connect the implementation evidence to existing pinned Pi, Hermes, and Codex
sources without claiming a shared source-level architecture.

- [ ] **Step 5: Cover effects, delegation, and evaluation**

Explain authorization, persistence-before-effect, idempotency, ambiguity after
crash, reconciliation, compensation, child lineage, context inheritance,
budget transfer, isolation, and cancellation. Separate execution-time
`agentic_eval` from benchmark/evaluator/selector infrastructure commonly
called “agentic evaluation.”

- [ ] **Step 6: Add a failure taxonomy, review invariants, and worked trace**

Cover these failure modes:

```text
LLM output mistaken for semantic authority
visible schema mistaken for execution authority
old context mistaken for current world state
uncertain mutation retried as a known failure
import treated as harmless validation
cancelled work treated as rolled back
```

End with a bounded code-repair trace that shows:

```text
evaluate repair request
  -> apply read/search tools
  -> evaluate patch instruction against acquired evidence
  -> admit and apply write effect
  -> apply test runner
  -> record receipts as observations
  -> evaluate completion against fresh observations
```

- [ ] **Step 7: Run canonical corpus validation**

Run:

```sh
cargo run -p harp -- check
```

Expected: exit status 0 and a corpus summary.

## Task 2: Register the essay in SICP navigation

**Files:**
- Modify: `content/sicp/sicp_index.md`
- Modify: `content/sicp/course/sicp_course_guide.md`
- Test: `cargo run -p harp -- check`

- [ ] **Step 1: Link the essay from the SICP research index**

In `content/sicp/sicp_index.md`, add
`agentic_eval_apply.md` to the “Change or implement language semantics” route,
the “Transfer the material into an agent harness” route, the transfer section,
and quick links. Describe it as an MTS supplement, not a source-authority
replacement for the course or capstone.

- [ ] **Step 2: Link the essay from the course guide**

In `content/sicp/course/sicp_course_guide.md`, include it as an optional
supplement after Seminar 9 and before the capstone. Add it to the
agent-harness design route as a semantic-to-effect-boundary bridge.

- [ ] **Step 3: Validate internal navigation through the corpus compiler**

Run:

```sh
cargo run -p harp -- check
```

Expected: exit status 0.

## Task 3: Regenerate and validate derived Atlas artifacts

**Files:**
- Modify: `atlas/src/content/generated/corpus.json`
- Modify: `atlas/dist/harp-atlas.html`
- Modify: `atlas/dist/harp-atlas.receipt.json`

- [ ] **Step 1: Regenerate the corpus**

Run:

```sh
cargo run -p harp -- build
```

Expected: exit status 0 and a generated corpus at
`atlas/src/content/generated/corpus.json`.

- [ ] **Step 2: Regenerate the offline Atlas export**

Read `atlas/package.json` and invoke its existing export command:

```sh
cd atlas && corepack pnpm run build
```

Expected: exit status 0 and updated
`atlas/dist/harp-atlas.html` plus `atlas/dist/harp-atlas.receipt.json`.

- [ ] **Step 3: Verify generated-artifact fixed point and decoded export**

Run:

```sh
cargo run -p harp -- build --check
cd atlas && corepack pnpm run test:export
```

Expected: both commands exit 0.

## Task 4: Run the release gate and inspect delivery surface

**Files:**
- Verify: `content/sicp/agentic_eval_apply.md`
- Verify: `content/sicp/sicp_index.md`
- Verify: `content/sicp/course/sicp_course_guide.md`
- Verify: `atlas/src/content/generated/corpus.json`
- Verify: `atlas/dist/harp-atlas.html`
- Verify: `atlas/dist/harp-atlas.receipt.json`

- [ ] **Step 1: Run the full repository verification gate**

Run:

```sh
mise run verify
```

Expected: exit status 0 after Rust formatting/lint/tests, corpus/source/search
checks, supplemental tests, Atlas lint/typecheck/tests/export, and repository
verification.

- [ ] **Step 2: Inspect the exact delivery surface**

Run:

```sh
git status --short
git diff --check
git diff -- content/sicp/agentic_eval_apply.md content/sicp/sicp_index.md content/sicp/course/sicp_course_guide.md
```

Expected: no whitespace errors and a diff limited to the canonical essay,
navigation, generated artifacts, and prior user-owned unrelated files.

- [ ] **Step 3: Commit only if explicitly requested**

If the user requests a commit, stage only the essay, navigation, and derived
artifacts. Use a message whose final trailer block contains exactly:

```text
Co-authored-by: TRAE CLI <noreply@bytedance.com>
```

## Self-review

### Spec coverage

- Thesis, analogy boundary, mutual recursion, context materialization, pipeline,
  effect semantics, delegation, evaluation distinction, failure taxonomy, and
  worked trace: Task 1.
- SICP navigation and course placement: Task 2.
- Corpus and Atlas generation: Task 3.
- Full verification and exact delivery inspection: Task 4.

### Placeholder scan

The plan names all edited paths, semantic structures, commands, and expected
results. It contains no `TBD`, `TODO`, deferred implementation instruction, or
undefined helper.

### Consistency

The canonical document remains `content/sicp/agentic_eval_apply.md` in every
task. The generated corpus and Atlas export are both rebuilt after canonical
input changes. No task assumes changes to runtime source code or evidence
bundles.

---
id: dynamic-workflow-need-and-eval
title: Dynamic Workflow - need cases and benchmark design
type: benchmark-design
status: draft
created: 2026-08-12
updated: 2026-08-12
tags: [dynamic-workflow, harnesses, evaluation, durable-execution, rlm]
---

# Dynamic Workflow: need cases and benchmark design

This note defines when Harp should use a Dynamic Workflow instead of a
single agent loop, and proposes deterministic benchmarks for evaluating the
Rust IR and durable execution path. It does not claim model-quality gains.
The first evaluation target is orchestration correctness: order, barriers,
pipeline progress, recovery, schema enforcement, and side-effect boundaries.

## Evidence classes

| Source class | Evidence used | Claim ceiling |
|-|-|-|
| Harp-local implementation | `crates/harp-contracts/src/dynamic_workflow.rs`, `crates/harp-engine/src/dynamic_workflow.rs`, `crates/harp/src/main.rs`, and focused tests in this branch | Current Rust IR and durable execution behavior |
| Harp-local research packet | Ignored Lark capture at `knowledge/investigations/local/lark-doc-parser/bytetech-7666762975667355688-20260812-1059/` in the main checkout | Source-backed interpretation of Dynamic Workflow ideas, not canonical tracked authority |
| Existing Harp benchmark guide | `knowledge/harness_benchmarks/harness_benchmark_field_guide.md` | Evaluation discipline and claim ceilings for harness benchmarks |
| Weng-derived Harp notes | `knowledge/rsi/weng/02-harness-design-patterns.md`, `knowledge/rsi/lw_rsi_harness.md` | Harness-mechanism framing and benchmark cautions |
| Supplied benchmark taxonomy | User-provided 47-benchmark operational and reasoning ladder, dated 2026-08-12 | Provisional map from benchmark families to Dynamic Workflow stressors |

## When Dynamic Workflow is needed

Dynamic Workflow is justified when the task is not hard because one model call
needs better prose, but because the harness must preserve a reliable execution
shape across many semantic calls.

### 1. Required-step control

**SOURCE CLAIM.** The Dynamic Workflow source argues that Skills are useful for
domain knowledge, but become fragile when they own every control step: the
agent must reinterpret the checklist on each turn, and required work can be
skipped or stopped early. The source explicitly says lint, typecheck, and test
do not need to be re-decided by the model each time; they belong in program
control flow.

**INFERENCE.** A Dynamic Workflow is needed when missing a required step is a
material failure, and the step can be represented mechanically. Examples:
release gates, compliance checklists, contract-section coverage, migration
coverage, or benchmark finalization.

**Benchmark implication.** The benchmark must detect omitted or reordered
steps, not only final success text.

### 2. Barrier fan-out

**SOURCE CLAIM.** The Dynamic Workflow source distinguishes `parallel()` from
generic concurrency: the next decision sometimes needs every branch result.
It uses lint/typecheck/test and release-build fan-out as examples where a
barrier is necessary.

**INFERENCE.** A Dynamic Workflow is needed when independent agents can work in
parallel, but a downstream reducer must see the complete result set before
making a decision. Examples: code review from multiple angles, fact checking
from independent sources, multi-platform release builds, or independent
security scans.

**Benchmark implication.** The benchmark must prove the reducer cannot run
before all required branches have terminal durable results.

### 3. Item pipelines without global stage barriers

**SOURCE CLAIM.** The Dynamic Workflow source distinguishes `pipeline()` from
round-based `Promise.all` stages. If lint failure is known, diagnosis and repair
can begin without waiting for a slow integration test. The same pattern is
applied to claim extraction, citation binding, contradiction marking, and
format normalization in research workflows.

**INFERENCE.** A Dynamic Workflow is needed when each item has its own staged
lifecycle and unrelated items should not block progress. Examples: per-file
repair, per-claim verification, per-document extraction, or per-platform
sign/check/upload.

**Benchmark implication.** The benchmark must distinguish per-item chains from
global stage barriers. A system that waits for all stage-1 items before any
stage-2 item starts should fail this benchmark.

### 4. Durable replay and restart

**SOURCE CLAIM.** The Dynamic Workflow source frames journaled replay as reuse
of completed `agent()` results, not a JavaScript VM snapshot and not a
filesystem rollback. It also warns that unfinished side effects can remain and
therefore need idempotent prompts, smaller units, read/write phase separation,
worktree isolation, Git baselines, and business idempotency keys.

**INFERENCE.** A Dynamic Workflow is needed when long-running work should
survive interruption without recomputing completed branches. It is not enough
for the transcript to be long-lived; the runtime must know which semantic call
completed and whether its result can be reused safely.

**Benchmark implication.** The benchmark must kill or interrupt a run, resume,
and prove completed calls are accepted exactly once while unfinished calls are
restarted or continued according to durable state.

### 5. Structured result contracts

**SOURCE CLAIM.** The Dynamic Workflow source treats JSON Schema as the stable
interface when downstream code consumes agent output. Harp's current durable
engine enforces an even narrower first-slice contract: every compiled node is
decoded as a `ResultEnvelope`, with answer, evidence, trace, token usage, and
status recorded as artifacts.

**INFERENCE.** A Dynamic Workflow is needed when downstream control should not
parse prose. It is especially valuable when a later branch depends on exact
fields, result status, or evidence references.

**Benchmark implication.** The benchmark must fail malformed outputs at the
durable result boundary before downstream reducers accept them.

## When Dynamic Workflow is not needed

| Situation | Better representation |
|-|-|
| One localized edit or answer | Single agent loop or `harp run --workflow general_coding` |
| Pure domain guidance with no required execution order | Skill / context-control playbook |
| Simple deterministic transformation | Ordinary Rust code, not agent orchestration |
| Exploration where stop conditions are intentionally open | Open-loop research agent, with explicit budget cap |
| Human wants one judgment from one evidence set | One agent call with a clear schema |

## Benchmark-pattern study

The supplied 47-benchmark ladder separates operational complexity from
reasoning depth. That distinction is important for Harp because Dynamic
Workflow should not be justified by intellectual difficulty alone. A static
FrontierMath-style problem can be difficult without needing workflow
orchestration. Conversely, a lower-reasoning but stateful tool benchmark can
need durable sequencing, barriers, and replay.

| Operational level | Representative benchmarks from supplied taxonomy | Dynamic Workflow relevance | Harp benchmark implication |
|-|-|-|-|
| L0 static answer | AIME, GPQA Diamond, FrontierMath | Low. These stress model reasoning, not harness control flow. | Do not use these as primary Dynamic Workflow evidence. A single call plus answer checker is enough. |
| L1 structured output | IFBench, LiveCodeBench, BigCodeBench, SciCode, ARC-AGI-2 | Low to medium. Useful when exact output schemas or executable tests gate success, but most tasks remain single-call or single-loop. | Reuse schema-contract and required-step fixtures; avoid claiming workflow advantage without multi-step state. |
| L2 long or multimodal context | BEAM, AA-LCR, LongBench v2, OmniDocBench, CIMemories | Medium. Dynamic Workflow helps when the context must be partitioned into claim extraction, verification, contradiction handling, and reduction. | Add claim/document pipeline fixtures before claiming long-context usefulness. |
| L3 bounded tools, retrieval, or memory | BFCL, GAIA, LongMemEval-V2 | Medium. Serial and parallel tool composition can benefit, but many cases fit normal function-calling. | Evaluate tool-call dependency graphs and memory-retrieval reuse separately from broad agent orchestration. |
| L4 stateful closed world | ToolSandbox, tau-bench | High. These need policy order, mutable state, clarification loops, and idempotent state-changing actions. | Add stateful-policy-order and resume-after-partial-action fixtures once Harp exposes a closed-world simulator adapter. |
| L5 broad operational environments | MCP Atlas, SkillsBench, OSWorld v1, SWE-bench, WebArena, AppWorld, Terminal-Bench | High. Broad tool ecosystems and repository or application state need durable work partitioning, branch isolation, and reducer contracts. | Current sequence, barrier, pipeline, and resume fixtures are the minimum local proxy for this level. |
| L6 dynamic, asynchronous, or adversarial | AgentDojo, AgentHarm, WildClawBench, GAIA2, OSWorld 2.0 | Very high. Events, changing requirements, injected instructions, and safety boundaries require explicit control flow and provenance. | Add tainted-input isolation, event-driven replan, and safety refusal fixtures before claiming L6 coverage. |
| L7 open-ended production or research | GDPval, PaperBench, ResearchClawBench | High, but not because of arbitrary branching alone. The value is staged artifact production, rubric evaluation, and auditable evidence. | Add rubric-gated artifact pipelines; do not reduce these to LLM-judge prose scoring. |

### Capability tracks that matter for Harp

The most useful upstream tracks for Dynamic Workflow are not the static
reasoning tracks. They are the ones where the harness must preserve state and
control boundaries:

| Track | Dynamic Workflow pattern | Local proxy already present | Missing proxy |
|-|-|-|-|
| Tool use and state management | Required policy order, state checks before mutation, retry without duplicate side effects | `dw-sequence-required-steps`, `dw-resume-completed-calls` | Closed-world state simulator and idempotent mutation receipt |
| GUI and computer use | Long action chains with interrupts, partial completion, and late requirement changes | `dw-pipeline-no-global-barrier` | Event-driven replan and external-state verifier |
| Coding and research engineering | Parallel diagnosis, reducer over evidence, deterministic gates, resumable repair loops | `dw-parallel-barrier`, `dw-schema-contract` | Repository patch verifier and per-file side-effect isolation |
| Long context and memory | Per-document or per-claim extraction pipelines, contradiction marking, durable memory reuse | `dw-pipeline-no-global-barrier` | Claim-level artifact schema and memory-hit provenance |
| Web research | Exhaustive set building, dedupe, source coverage, stop-condition evidence | `dw-parallel-barrier` | Exhaustive-set metric and source coverage receipt |
| Safety | Tainted input boundaries, benign utility separated from attack success, refusal as terminal result | `dw-schema-contract` | Tainted evidence labels and safety-policy result status |

### Benchmark design consequences

1. Dynamic Workflow should be evaluated first on L4-L7 operational patterns,
   not on L0-L2 static reasoning benchmarks.
2. A benchmark should preserve the upstream evaluation semantics it is
   proxying: final-state checks for stateful worlds, executable tests for
   coding, claim coverage for MCP-style tasks, and separate benign utility
   from attack success for security.
3. Harp's local benchmark fixtures should stay deterministic. Real TraeCLI
   agent calls are useful as smoke tests for adapter integration, but they
   should not be the canonical correctness metric for the Rust IR.
4. The current fixture suite mostly covers L5 control-flow prerequisites. It
   does not yet cover L6 dynamic events or adversarial taint, and it does not
   prove L7 artifact quality.

## Benchmark suite design

The first suite should be deterministic and local. It should use Harp's fake
CLI runtime and fixture result envelopes, not paid model calls. Model quality
can be evaluated later only after the orchestration semantics are stable.

| ID | Need case | Fixture | Pass condition | Failure caught |
|-|-|-|-|-|
| `dw-sequence-required-steps` | Required-step control | Three sequential agent calls and one reducer | State shows dependency chain and all accepted results | Skipped or reordered required step |
| `dw-parallel-barrier` | Barrier fan-out | One discovery node, two parallel branches, reducer | Reducer depends on all analysis tasks and cannot validate otherwise | Reducer runs from partial branch set |
| `dw-pipeline-no-global-barrier` | Item pipeline | Two items, two stages | Per-item stage-2 depends only on same item stage-1 | Global stage barrier accidentally introduced |
| `dw-resume-completed-calls` | Durable replay | Fake CLI barrier after first activity, then resume | Completed call has one accepted result; incomplete calls continue | Duplicate acceptance or full recompute |
| `dw-schema-contract` | Structured result contracts | Fake CLI returns malformed final message for one node | Run fails with output-schema/JSON error before reducer success | Prose or malformed JSON accepted |
| `dw-side-effect-idempotency` | Side-effect boundary | Later write/worktree fixture | Re-run converges without duplicate append and records evidence | Replay treated as rollback |

The first five are ready for near-term implementation. `dw-side-effect-idempotency`
should wait until the write/worktree story is stronger, because current Harp
Dynamic Workflow nodes execute through the RLM scratch/workspace machinery but
do not yet expose a user-facing side-effect policy beyond the existing
workspace mode and result-envelope discipline.

## Measurement contract

Each benchmark should record:

- input workflow IR digest;
- compiled `TaskGraph` digest;
- expected graph shape;
- runtime mode and fake CLI scenario;
- accepted result count;
- event or status evidence for recovery behavior;
- whether the result proves orchestration correctness, runtime durability, or
  model-quality behavior.

For now every benchmark in this note proves orchestration correctness or
runtime durability only. None proves that Dynamic Workflow improves model
quality, task success on real workloads, or recursive self-improvement.

## Recommended implementation order

1. Add tracked benchmark fixtures under `benchmarks/dynamic-workflow/` for the
   first five cases.
2. Add schema/manifest validation for the fixture packet.
3. Add contract-level tests for sequence, barrier, and pipeline compilation.
4. Add CLI tests for run/status and resume using `harp_rlm_fake_cli`.
5. Add a negative schema-contract test.
6. Defer side-effect/idempotency until Harp exposes a clearer write policy for
   dynamic workflow activities.

## Open questions

- Should benchmark fixtures live as raw JSON files only, or also have a small
  generated `manifest.json` with expected graph digests?
- Should `phase` and `log` become durable event records before we evaluate
  user-facing observability?
- Should authored agent `output_schema` be preserved above the current
  `ResultEnvelope` boundary as a nested answer schema, or is `ResultEnvelope`
  the only first-slice contract?

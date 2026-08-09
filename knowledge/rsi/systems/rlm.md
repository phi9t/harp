---
id: rsi-system-rlm
kind: concept
title: Recursive Language Models
summary: Recursive inference over external REPL state, the minimal teaching kernel, the full implementation's control planes and adapter boundaries, author-reported transfer, and the separation between within-episode computation and RSI.
primary_parent: rsi-joint-harness-weight-adaptation
additional_parents:
  - rsi-harness-engineering
related:
  - kind: implemented-by
    target: rsi-harness-engineering
attachments:
  - content/recursive_language_models_compositional_generalization.md
  - content/sicp/agentic_eval_apply.md
  - evidence/implementations/rlm-minimal/snapshot
  - evidence/implementations/rlm/snapshot
claims: []
human_review: null
---

# Recursive Language Models: recursive inference over external state

## Decision brief

**EVIDENCE — [RLM-PAPER], §2 and Algorithm 1.** A Recursive Language Model
stores the supplied prompt in an external REPL, lets a root language model
inspect and transform selected portions with code, and exposes language-model
calls as REPL functions. At greater configured depth, a subcall can run another
RLM episode.

The Transformer does not gain recursive layers. The recursion belongs to an
inference-time program around a fixed model. RLM expands what one inference
episode can compute. It does not update the model, rewrite the controller,
evaluate descendants, or transfer improvement authority to a successor.

**INFERENCE — MTS decision.** Test RLM when a high-value task has a supplied
artifact much larger than one useful context, dense coverage matters, and the
decomposition must change after intermediate findings. Prefer a direct call
when the artifact fits, retrieval for sparse lookup, and fixed decomposition
when partitions and aggregation are known in advance. RLM adds variable calls,
generated code, mutable state, and another failure tree. Those costs need a
workload-specific reason.

**MISSING — production decision.** The inspected sources do not establish
independent replication, a matched quality-cost frontier against simpler
systems, durable recovery, evaluator integrity, or production safety. Treat the
official code as a reference implementation to inspect and reproduce, not as a
production control plane.

## Evidence boundary

This reading separates four authorities:

- **[RLM-PAPER]** owns the preprint algorithm description and author-reported
  experiments at arXiv revision `2512.24601v3`.
- **[A1ZHANG-HARNESS-BLOG]** owns the later first-party post-training and
  compositional-generalization interpretation.
- **[RLM-MINIMAL]** is the teaching implementation at commit
  `973f8d4acf3af2c86dc170af91607bf8b0c4d0ea`.
- **[RLM-REPO]** is the fuller implementation at commit
  `72d6940142ddfb84ee6be573dc999a37e633e671`.

The two repositories make mechanisms inspectable. They do not inherit the
paper's result authority. The minimal pin is not byte-equivalent to the paper
runtime, and the full pin's adapter code is not evidence of equivalent safety
or feature coverage across environments.

<details>
<summary>Original sources for this mechanism</summary>

- Architecture, algorithm, and reported evaluation:
  [RLM paper, arXiv v3](https://arxiv.org/abs/2512.24601).
- Later post-training and transfer interpretation:
  [author technical blog](https://alexzhang13.github.io/blog/2026/harness/).
- Minimal teaching implementation:
  `evidence/implementations/rlm-minimal/snapshot/`.
- Full implementation:
  `evidence/implementations/rlm/snapshot/`.
- Longer paper/blog companion:
  [RLM compositional-generalization deep dive](../recursive_language_models_compositional_generalization.md).

</details>

## The computational move

Let `P` be the supplied problem artifact, `E` the REPL environment, `M` the
fixed model, and `H` the controller. Ordinary inference places the whole
artifact in model context:

```text
answer = M(P)
```

RLM instead keeps `P` in `E`. At iteration `t`, `H` builds a bounded
model-visible history, asks `M` for the next response, extracts proposed code,
applies that code through `E`, and returns selected output to the next model
iteration:

```text
P -> E
repeat:
  proposal_t = M(model_visible_history_t)
  observation_t = E.execute(proposal_t)
  model_visible_history_(t+1) += observation_t
until E exposes a controller-recognized answer or a limit ends the loop
```

Generated code can inspect `P`, keep intermediate values in REPL variables,
call a plain submodel, or, where the runtime supports it, launch a child RLM.
This makes decomposition data-dependent. The model can revise its partition or
aggregation plan after observing intermediate results instead of committing to
a fixed map-reduce graph before execution.

**INFERENCE — context virtualization.** Model context becomes a bounded working
set. The REPL holds the full supplied artifact and intermediate state. This
extends the effective computation beyond one prompt, but it does not make
context or compute unbounded. The controller still has finite iterations,
model calls, runtime, memory, and provider limits.

## Grounding the minimal kernel

**EVIDENCE — [RLM-MINIMAL],
[`rlm/rlm_repl.py:47-121`](../../../evidence/implementations/rlm-minimal/snapshot/rlm/rlm_repl.py#L47),
[`rlm/repl.py:71-194`](../../../evidence/implementations/rlm-minimal/snapshot/rlm/repl.py#L71),
and
[`rlm/utils/utils.py:8-39,183-213`](../../../evidence/implementations/rlm-minimal/snapshot/rlm/utils/utils.py#L8).**
The minimal
implementation exposes the shortest end-to-end path:

1. `setup_context` converts the supplied context and loads it into one
   episode-local `REPLEnv`.
2. `completion` asks the root model for a response on each iteration.
3. The controller finds fenced `repl` blocks and executes them.
4. The REPL exposes a plain `llm_query` function backed by `Sub_RLM`.
5. `FINAL(...)` returns literal response text, while `FINAL_VAR(...)` resolves
   a named REPL local.
6. If no completion signal appears within `max_iterations`, the controller asks
   the root model for one fallback answer.

This pin implements depth one. The README says deeper recursion would require
replacing `Sub_RLM` and adapting the `exec`-based environments. It also says the
teaching version omits much of the experiment runtime's logging, cost
tracking, prompting, and execution detail.

**EVIDENCE — [RLM-MINIMAL],
[`rlm/repl.py:89-160,232-300`](../../../evidence/implementations/rlm-minimal/snapshot/rlm/repl.py#L89).**
The REPL is not a
sandbox. It executes generated Python in the host process, permits imports and
file access, and temporarily mutates process-global streams and the working
directory. One instance uses a lock around those mutations, but separate
instances can still share the process. The code is useful for mechanism
inspection, not benchmark parity or production isolation.

The minimal repository reveals the stable kernel:

| Step | Controller responsibility | Environment responsibility |
|---|---|---|
| Initialize | build the root history | store supplied context |
| Evaluate | sample the next response | retain prior variables |
| Apply | select fenced code | execute code and plain subcalls |
| Observe | append formatted output | return stdout, stderr, and locals |
| Complete | recognize `FINAL` or `FINAL_VAR` | resolve a final variable |

## From the kernel to the full implementation

The selected full-repository files expose a substantially different
architecture from the minimal teaching implementation.

| Obligation | Minimal teaching code | Full pinned implementation |
|---|---|---|
| Root loop | one `RLM_REPL.completion` loop | `RLM.completion` with typed iterations, callbacks, usage, and stop checks |
| Model calls | direct OpenAI wrapper | host `LMHandler` with configured root and secondary backends |
| Code execution | one host-process `REPLEnv` | seven registered adapters with different controller-wired features |
| State | one episode-local namespace | per-episode state plus optional live cross-completion reuse |
| Recursion | plain depth-one subcall | plain calls and selected child-RLM paths |
| Completion | `FINAL` or `FINAL_VAR` | `answer["content"]` with `answer["ready"]` |
| Inspection | optional display logging | typed iteration and trajectory metadata |

### Controller and model-call topology

**EVIDENCE — [RLM-REPO],
[`rlm/core/rlm.py:49-490`](../../../evidence/implementations/rlm/snapshot/rlm/core/rlm.py#L49).**
A non-persistent completion creates a provider client, starts a host-side
`LMHandler`, creates an environment, builds root history, runs the root loop,
and cleans up. The environment executes code and reports a `REPLResult`; the
controller formats that result into the next root turn.

This separation matters. Model sampling, generated-code execution, external
state, and completion recognition are different boundaries even when one
Python class coordinates them.

### Seven registered adapters, different controller contracts

**EVIDENCE — [RLM-REPO],
[`rlm/environments/__init__.py`](../../../evidence/implementations/rlm/snapshot/rlm/environments/__init__.py)
and
[`rlm/core/rlm.py:258-291`](../../../evidence/implementations/rlm/snapshot/rlm/core/rlm.py#L258).**
The pin registers seven execution adapters. The controller does not wire the
same recursion, cross-call state, or compaction features to all seven.

| Registered environment | Child RLM wired by controller | Cross-call state allowed by controller | Compaction wired by controller |
|---|---:|---:|---:|
| Local | yes | yes | yes |
| IPython | yes | yes | no |
| Docker | yes | yes | yes |
| Modal | no | no | no |
| Prime | no | no | no |
| Daytona | no | no | no |
| E2B | no | no | no |

**EVIDENCE — [RLM-REPO],
[`rlm/environments/local_repl.py:147-227`](../../../evidence/implementations/rlm/snapshot/rlm/environments/local_repl.py#L147).**
The default Local adapter executes generated Python in the controller's host
process and exposes plain and recursive query functions in that process.
Adapter registration and controller wiring do not prove equivalent execution,
cancellation, resource accounting, credential isolation, custom-tool support,
or test depth across the other six adapters.

### Recursive calls and budgets

**EVIDENCE — [RLM-REPO],
[`rlm/core/rlm.py:275-287,551-566,706-870`](../../../evidence/implementations/rlm/snapshot/rlm/core/rlm.py#L275).**
True child RLMs are wired only for Local, IPython, and Docker when
`max_depth > 1`. A child receives its own controller and environment. At the
depth boundary, the implementation falls back to one plain model call.

`max_depth` bounds nesting depth. `max_concurrent_subcalls` bounds selected
batched concurrency. Neither bounds total subcall count across a run. The
parent computes an apparent remaining dollar budget for each child, but
parallel children do not reserve against one shared remainder. Each child also
receives the full configured token ceiling rather than a reserved share.

Child cost is added to the parent's `_cumulative_cost` when the child returns,
but the parent's subsequent post-iteration check replaces that field with usage
from the parent handler alone. Child spend can therefore disappear from
enforcement even without parallelism. `max_budget` is not a reliable tree-wide cap. These are local cooperative checks, not aggregate admission control.

**EVIDENCE — [RLM-REPO],
[`rlm/core/rlm.py:361-425,492-585`](../../../evidence/implementations/rlm/snapshot/rlm/core/rlm.py#L361).**
Timeout is
checked before a root iteration. Consecutive-error, reported-cost, and token
checks run after the root response and all extracted code blocks execute.
Those are useful stop checks, but they are between-iteration accounting, not
hard admission or preemption. One iteration can cross a configured limit before
the controller observes it.

### State reuse, compaction, and recovery

**EVIDENCE — [RLM-REPO],
[`rlm/core/rlm.py:226-299,872-907`](../../../evidence/implementations/rlm/snapshot/rlm/core/rlm.py#L226)
and
[`rlm/environments/local_repl.py:147-227,399-546`](../../../evidence/implementations/rlm/snapshot/rlm/environments/local_repl.py#L147).**
The controller's persistence allow-list contains Local, IPython, and Docker.
The captured Local environment retains variables across code blocks and adds
versioned `context_N` and `history_N` values when reused across completion
calls. Closing the object destroys that live state. This is not evidence that
all three adapters implement identical retention semantics, and it is not a
durable checkpoint or memory service.

**EVIDENCE — [RLM-REPO],
[`rlm/core/rlm.py:365-378,587-644`](../../../evidence/implementations/rlm/snapshot/rlm/core/rlm.py#L365)
and
[`rlm/environments/local_repl.py:181-198,481-490`](../../../evidence/implementations/rlm/snapshot/rlm/environments/local_repl.py#L181).**
The controller enables optional compaction only for Local and Docker. It
watches estimated root-history tokens, asks the root model for a summary near a
configured threshold, and shortens root message history. The captured Local
implementation stores trajectory segments and summaries in a REPL `history`
list. This is a lossy, model-generated context projection, not evidence that
Docker retains identical state. It is not an immutable event log, durable
checkpoint, or provenance-preserving memory.

**MISSING — recovery contract.** Selected limit and cancellation exceptions can
carry the most recently retained nonempty root response. Because limit checks
run before the current response becomes that field, the value may lag the
limit-crossing iteration or be absent. It is raw model text, not a validated
partial answer, serialized REPL state, effect receipt, or resume point.

### The depth-one training adapter

**EVIDENCE — [RLM-REPO],
[`training/src/rlm_train/env.py`](../../../evidence/implementations/rlm/snapshot/training/src/rlm_train/env.py),
[`training/src/rlm_train/repl/subprocess.py`](../../../evidence/implementations/rlm/snapshot/training/src/rlm_train/repl/subprocess.py),
and
[`training/src/rlm_train/worker.py:142-180,235-255`](../../../evidence/implementations/rlm/snapshot/training/src/rlm_train/worker.py#L142).**
The repository also has
a separate depth-one training adapter. `RLMTrainEnv` maps the loop into a
`verifiers.MultiTurnEnv`, starts a subprocess REPL worker per rollout, and
routes submodel requests through a localhost proxy to the trainer client. In
the worker, `rlm_query` aliases `llm_query`, and the batched recursive name
aliases the batched plain call. This path trains a depth-one loop policy. It
does not execute the full child-RLM runtime and is separate from the paper's
trajectory-distillation recipe.

## RLM through the SICP eval/apply lens

The [agentic eval/apply companion](../sicp/agentic_eval_apply.md) supplies a
useful discipline for reading this loop:

```text
evaluate(task, selected_view_of_repl_state)
  -> propose code, a semantic call, or completion
apply(proposal through the selected runtime boundary)
  -> produce an observation
evaluate(task, extended_state and new observation)
```

**INFERENCE — structural analogy.** RLM is a partially observable, effectful
eval/apply loop. The controller interprets the current task under a selected
view of REPL state. Generated code and semantic calls cross an application
boundary. Their outputs influence the next evaluation.

The analogy is not formal equivalence:

- the LLM is a probabilistic proposal producer, not SICP's `eval`;
- generated code is not an already resolved and authorized procedure value;
- Python execution and model calls can cross process, provider, filesystem,
  and network boundaries;
- a returned REPL value or child report is an observation, not automatically a
  verified fact; and
- `FINAL`, `FINAL_VAR`, or `answer["ready"]` closes a cooperative controller
  protocol. It does not prove that child work stopped, external effects
  settled, or the result was durably committed.

This is the philosophical connection that survives contact with the code:
interpretation, application, environment, authority, and returned values must
remain distinct enough to audit.

## Learning interpretation and reported evaluation

**SOURCE CLAIM — [RLM-PAPER], Abstract and §4.** Across four heterogeneous
long-context benchmarks, the authors report GPT-5 RLM median relative gains of
26% over their compaction baseline, 130% over their CodeAct-with-subcalls
baseline, and 13% over Claude Opus 4.1 running Claude Code v2.0.0 with
file-based context offloading. The paper also reports that the median RLM run
was cheaper than the median base-model run, while the RLM mean was higher
because of outlier trajectories. These are author-reported per-query API-cost
comparisons with missing entries for some baselines, not an independently
reproduced end-to-end latency or operating-cost result.

**EVIDENCE — [RLM-PAPER], rendered v3 abstract, §1, Figure 3, and arXiv
metadata.** Same-version representations conflict. The rendered paper says
RLMs process inputs more than one order of magnitude beyond model context
limits, while the captured arXiv summary says up to two orders. The rendered
abstract and §1 call the RLM-Qwen3-8B gain a median of 28% or 28.3%, while the
captured metadata calls 28.3% an average. The rendered experiment body, table,
and figure define Harp's quantitative boundary; the metadata conflicts remain
visible and no universal capacity ratio is inferred.

**SOURCE CLAIM — [A1ZHANG-HARNESS-BLOG].** The authors argue that a harness can
act as a higher-level inductive bias by mapping globally unfamiliar tasks into
locally familiar calls and reusable decomposition trajectories. Their
first-party post reports training one Qwen3 30B-family model, transfer to six
task settings described as 8 to 32 times longer, and three paired cross-domain
settings. The multiplier refers to task-specific input length, output length,
or instruction count rather than one uniform context measure.

**SOURCE CLAIM — [A1ZHANG-HARNESS-BLOG], training-cost section.** The post
reports 1.5 to 3 times longer RLM training runtime on similarly sized tasks in
its length and strategy experiments because samples use multiple steps and
wait for subcalls. It notes additional memory overhead without quantifying it
and gives no absolute accelerator-hour, dollar, or end-to-end cost. This is a
training-runtime claim, not inference cost.

**INFERENCE — claim boundary.** These results support further tests of
harness-induced compositional transfer. The claim weakens if matched
model-harness comparisons remove the reported transfer gap. It is falsified
for the audited task pairs if repeated held-out experiments with equal model,
data access, tokens, calls, wall time, and cost show no RLM advantage.
Trajectory similarity remains a proxy for reusable strategy, not proof of
equal model-internal computation. The packet contains no independent
reproduction.

## Failure modes and limits

- **Bad decomposition.** The controller can spend more calls exploring a poor
  partition than a fixed workflow would.
- **Hidden cost.** Similar root traces can conceal different child trees,
  retries, waits, and provider charges.
- **State confusion.** Live variables, compacted summaries, model-visible
  history, retained logs, and world state have different freshness and
  durability.
- **Authority collapse.** Treating generated code or a child report as trusted
  merely because it returned turns a proposal into an unearned fact.
- **Unsafe execution.** Local execution shares the host process. Remote and
  container adapters add brokers, mounts, credentials, and network policy.
- **Weak finalization.** Cooperative completion does not establish global
  quiescence or exactly-once effects.
- **Evaluation leakage.** More adaptive calls and compute can look like a
  better architecture if the comparison does not equalize the full budget.
- **Training mismatch.** The separate training path is depth one and does not
  reproduce every inference adapter or recursive behavior.

## Product and research program

The first useful prototype is replay-only:

1. Bind every input artifact by digest and mount it read-only.
2. Run generated code outside the host process with no ambient credentials.
3. Route model egress through one audited adapter.
4. Cap depth at two and record calls, tokens, wall time, and cost for the whole
   call tree.
5. Require typed findings with artifact references.
6. Return explicit completed, partial, failed, and unresolved outcomes.
7. Compare the same tasks against direct context, retrieval, and fixed
   decomposition.

The matched evaluation should hold the base model, artifact access, answer
rubric, provider destination, wall-time cap, token and call accounting, retry
policy, and safety requirements constant. Report paper reproduction separately
from the cross-system comparison. A reproduction asks whether the source result
can be recovered. A matched comparison asks whether adaptive recursion earns
its added complexity on the target workload.

**INFERENCE — product gate.** Continue only when RLM supplies a required
capability or a repeatable quality-integrity gain that survives full
end-to-end cost and latency accounting. Stop for that workload when a simpler
system is equivalent within a pre-registered margin and cheaper to operate.
This gate is Harp's synthesis, not a threshold supplied by the paper.

## Relation to harness search and RSI

RLM, ADAS, and DGM act on different objects and time scales:

| System | Mutable object | Time scale | What remains fixed |
|---|---|---|---|
| RLM | execution plan, REPL state, and call tree | one inference episode | model weights and controller implementation |
| [ADAS](adas.md) | downstream agent program | searched candidates | foundation models, evaluator, and outer search |
| [DGM](dgm.md) | coding-agent implementation | descendant generations | foundation models, outer archive policy, and benchmark envelope |

An RLM-like runtime could help a future improvement system inspect large code,
papers, traces, and prior mutations. That would make RLM an execution component
inside the larger loop. It would not turn within-episode recursion into
recursive self-improvement.

## Claim ceiling

The evidence supports:

- recursive inference over external prompt state;
- dynamic code and semantic-call decomposition;
- a compact depth-one teaching implementation;
- a fuller implementation with adapter, state, typed iteration and trajectory
  metadata, limit, compaction, and training control planes; and
- author-reported long-context and compositional-transfer results.

It does not support:

- a recursive Transformer architecture;
- unbounded context or compute;
- equivalent safety or semantics across adapters;
- durable, exactly-once, or resumable execution;
- universal advantage over direct context, retrieval, or fixed workflows; or
- recursive self-improvement of the model-harness pair.

## Reading routes

- [Agentic eval/apply](../sicp/agentic_eval_apply.md)
- [Weng: harness versus core intelligence](../weng/03-harness-layer-vs-core-intelligence.md)
- [RLM mechanism deep dive](../recursive_language_models_compositional_generalization.md)
- [Joint harness and model-weight adaptation](../chapters/joint-harness-weight-adaptation.md)
- [Original paper](https://arxiv.org/abs/2512.24601)

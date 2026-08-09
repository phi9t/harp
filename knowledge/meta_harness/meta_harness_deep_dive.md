---
id: rsi-meta-harness-deep-dive
title: Meta-Harness - evidence-tiered implementation deep dive
type: technical-deep-dive
mode: TECHNICAL REVIEW
status: active
created: 2026-08-08
updated: 2026-08-08
tags: [recursive-self-improvement, meta-harness, context-engineering, harness-search, provenance]
confidence: high
---

# Meta-Harness: from paper mechanism to pinned interfaces

Mode: `TECHNICAL REVIEW`.

> Meta-Harness searches executable policy around a fixed model. This document
> separates what the paper reports, what the dated project page reports, what
> the pinned public code does, and what one local TRAE proposal experiment
> actually established. It does not merge those tiers into one reproduction
> claim.

## Source tiers and claim ceilings

| Tier | Identity and local evidence | What it supports | Claim ceiling |
|---|---|---|---|
| arXiv v1 paper | [META-HARNESS], `evidence/weng/text/meta-harness.txt` | Paper-era outer-loop design, experiment protocol, ablations, and author-reported results. | Primary author report, not independent reproduction. |
| dated project page | [META-HARNESS-SITE], `evidence/meta_harness/site/index.html`, captured 2026-08-08 with HTTP `Last-Modified` 2026-08-04 | First-party tables, Pareto data, the visualized TerminalBench trajectory, and links to code and artifact repositories. | Mutable first-party presentation. "COLM 2026" is a dated site claim because no official venue or OpenReview record was located. |
| pinned public repositories | [META-HARNESS-REPO] at `44b9942127847f7421db70d8c7e48407f09a3c70` and [META-HARNESS-TB2-ARTIFACT] at `57fefdb2ff84af3fd81b69d67814acbe69bd0743` | Present-day interfaces, control flow, evaluation boundaries, and tests at those revisions. | Source behavior only. The main repository is cleaned paper code, not an independent reproduction; the artifact has no license file at its pinned revision. |
| local TRAE proposal experiment | [META-HARNESS-TRAE-RUN], `evidence/meta_harness/trae_run/receipt.json` | One `gpt-5.4` proposal iteration, strict response/write boundaries, and independent candidate-interface assessment. | Proposal-schema evidence only. No benchmark score, paid model evaluation, or held-out result was reproduced. |

The page's venue metadata appears in
`evidence/meta_harness/site/index.html:36-45`; its capture metadata and bytes are
fixed in `evidence/meta_harness/site_manifest.tsv`. The repository describes
itself as cleaned paper code and says it was not tested beyond verifying that
it runs in
`evidence/implementations/meta_harness/snapshot/README.md:47-56`.

## Candidate, archive, proposer, evaluator, and Pareto flow

**EVIDENCE. [META-HARNESS], §§1 and 3,
`evidence/weng/text/meta-harness.txt:100-103,197-265`.** The paper's abstract
machine is:

```text
candidate harness H_i
  -> evaluator on search tasks
  -> score vector + execution traces
  -> append code, scores, and traces to archive D
  -> proposer inspects selected parts of D
  -> candidate harness H_(i+1)
  -> Pareto retention
```

The proposer is not given one precompressed state. It uses filesystem tools to
select prior code, scores, and traces from an archive larger than its context
window. The external controller owns task execution, scoring, archive
publication, the fixed iteration budget, and final test-set evaluation. The
paper says the proposer sees search-set results but never test-set results
(`evidence/weng/text/meta-harness.txt:237-245`).

**INFERENCE.** The filesystem is both memory and query interface. This removes
a fixed summarizer from the critical path, but does not remove selection bias:
the proposer still chooses which evidence to inspect, and the evaluator still
defines what survives.

## Text-classification contract

### Candidate interface

**EVIDENCE. [META-HARNESS-REPO].** Every candidate is expected to subclass
`MemorySystem`, accept an LLM callable, predict before seeing ground truth,
learn from a batch afterward, and round-trip serializable state. The interface
also exposes prompt metadata and context length
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/memory_system.py:61-126`).

The loader converts `agents/name.py` to a module below the
`text_classification` package, imports it, and returns the first concrete
`MemorySystem` subclass
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/inner_loop.py:479-504`).
This makes package identity part of the candidate contract; Python syntax alone
is insufficient.

### Search and finalization

**EVIDENCE. [META-HARNESS-REPO].** The outer loop declares baseline files,
run-local `pending_eval.json`, validation frontier, evolution summary, result
directory, and finalization receipt
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/meta_harness.py:26-47`).
It import-checks proposed systems before benchmarking
(`meta_harness.py:192-215`), evaluates candidates on validation data during
evolution (`meta_harness.py:470-527`), and exposes held-out evaluation only
through explicit finalization (`meta_harness.py:286-357`).

The provider-free tests enforce that ordinary evolution never passes `--test`,
that finalization writes a blocking state, and that incomplete finalization
does not falsely close the run
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/tests/test_evolution_contracts.py:66-188`).
In a fresh `uv --native-tls` environment, all 28 shipped provider-free tests
passed on 2026-08-08. The bounded execution findings are recorded in
`evidence/meta_harness/source_test_findings.tsv`.

**BOUNDARY.** The public repository includes test data. Its README calls the
split operational isolation rather than access control
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/README.md:43-58`).
A malicious or overly privileged proposer could still read those bytes unless
the operating-system boundary prevents it.

## TerminalBench-2 boundaries

**EVIDENCE. [META-HARNESS-REPO].** The reference experiment fixes two
baselines, the 89-task full set, a model, trial count, concurrency, and timeout
(`evidence/implementations/meta_harness/snapshot/reference_examples/terminal_bench_2/meta_harness.py:80-112`).
Candidate validation imports the named class and requires a `Terminus2`
subclass before evaluation (`meta_harness.py:140-155`). A smoke task can reject
runtime failures before the expensive sweep (`meta_harness.py:459-520`).

The evaluation script validates the import path before invoking Harbor, chooses
either a 30-task development subset or the full dataset, and assembles the
model, environment, concurrency, attempts, and timeout arguments
(`evidence/implementations/meta_harness/snapshot/reference_examples/terminal_bench_2/scripts/run_eval.sh:1-98`).
The outer loop then:

1. reads `pending_eval.json`;
2. normalizes the named class to `AgentHarness`;
3. import-checks and optionally smoke-tests each candidate;
4. evaluates only survivors;
5. updates per-task and aggregate frontier state; and
6. optionally performs a five-trial final run
   (`meta_harness.py:680-880`).

**IMPORT BOUNDARY.** Validation imports candidate code in the outer Python
process. Import is execution: module-level code runs before subclass checks.
The released flow therefore relies on proposer trust, environment isolation,
and narrow writable surfaces in addition to type checks.

**ENVIRONMENT FINDING.** On 2026-08-08, a fresh
`uv sync --native-tls` could not resolve the pinned
`terminal-bench` Git revision
`1a6ffa9674b571da0ed040c470cb40c4d85f9b9b`. Harp performed static syntax and
trust-boundary review instead of claiming a TerminalBench runtime
reproduction.
The exact resolution failure is retained in
`evidence/meta_harness/source_test_findings.tsv`.

The separate artifact repository reports 76.4% over 89 tasks and five trials
and attributes the released harness to environment bootstrapping
(`evidence/implementations/meta_harness_tb2_artifact/snapshot/README.md:5-36`).
Those are artifact-author claims; Harp did not execute the benchmark.

## Experimental Harbor controller

The experimental Harbor path strengthens one dangerous boundary.

**EVIDENCE. [META-HARNESS-REPO].** Its trusted controller states that
candidate code is not imported by the long-lived outer agent. It first scans
for task-specific forbidden references, parses the AST, and requires an async
`AgentHarness.run` on a `BaseAgent` subclass
(`evidence/implementations/meta_harness/snapshot/experimental/harbor_meta_harness/controller.py:1-5,127-175`).

Accepted source is copied to a temporary directory as `candidate.py`; only the
child Harbor process receives that directory through `PYTHONPATH`
(`controller.py:201-292`). The test suite proves that leakage rejection occurs
before any child run and that the staged directory is exposed only to the
child (`evidence/implementations/meta_harness/snapshot/experimental/harbor_meta_harness/tests/test_controller.py:112-140,214-235`).

The outer agent uses an immutable bounded `EvaluationState`: `record` returns a
new value with one less evaluation and an appended result
(`evidence/implementations/meta_harness/snapshot/experimental/harbor_meta_harness/agents/meta_harness.py:33-39`).
Each scarce `evaluate_harness` call downloads candidate source, launches the
short-lived controller, records the result, and returns remaining budget
(`agents/meta_harness.py:53-101`). Its test checks that the original state
remains unchanged (`tests/test_controller.py:238-245`).

**ENVIRONMENT FINDING.** Harbor dependency resolution reached the LiteLLM Rust
bridge but failed because transitive AWS crates require Rust 1.94.1 while Harp
pins Rust 1.92.0. Static syntax, source, and test-boundary inspection therefore
do not become a claim that Harbor's full suite ran locally.
The exact toolchain finding is retained in
`evidence/meta_harness/source_test_findings.tsv`.

## Dated site state

### Pareto data

**CLAIM. [META-HARNESS-SITE].** The page says the text-classification search
uses three datasets, 20 iterations, two candidates per iteration, and holds test
sets until final evaluation
(`evidence/meta_harness/site/index.html:281-292`). It reports a 48.6% best
accuracy at 45.5K additional context characters, compared with ACE at 40.9%
and 203K. The exact plotted points are preserved in
`evidence/meta_harness/site/static/data/pareto.js:1`.

These are first-party reported results. The site capture does not convert them
into local measurements.

### Illustrative trajectory

**CLAIM. [META-HARNESS-SITE].** The page explicitly says its hard 19-task,
10-iteration evolution is illustrative, not the final reported run. It starts
from Terminus-KIRA at 28.5% and reaches 46.5% at iteration 7
(`evidence/meta_harness/site/index.html:104-120`). The complete trajectory is
preserved in `evidence/meta_harness/site/static/data/evo.js:1`.

The visible path is instructive because regressions remain visible:

- shorter output and forced early completion lose information or end work too
  soon;
- safe file inspection and database-copy rules improve robustness;
- testing as the target service user produces the largest displayed jump; and
- both iteration-8 proposals fail to import.

That last failure is not incidental. A code-search loop must preserve invalid
programs as evidence while preventing them from contaminating the evaluator.

## Local TRAE proposal experiment

### Boundary

**EVIDENCE. [META-HARNESS-TRAE-RUN].** Harp prepared an ignored workspace from
the pinned `MemorySystem` interface, three shipped baselines, and dated Pareto
data. The reference state explicitly says that per-dataset validation history,
`frontier_val.json`, validation traces, raw upstream proposer traces, benchmark
history, and held-out results were not reconstructed
(`evidence/meta_harness/trae_run/normalized/reference_state.json`).

The first schema-valid attempt used explicit `gpt-5.4`, exactly three
candidates, a workspace-write sandbox, no-approval headless execution, and only
read/search, shell, write, and edit tools. TRAE CLI 0.200.19 rejects simultaneous
legacy permission-mode and sandbox overrides; the raw archive preserves that
infrastructure attempt and the compatible
`approval_policy=never` plus `sandbox=workspace-write` command.

### Result

The proposal schema and exact write set passed. The valid candidate count is zero. All three files import
`reference.text_classification.memory_system`, while the pinned loader imports
candidates below the `text_classification` package. Independent isolated
imports therefore failed before cold-start, synthetic `predict`,
`learn_from_batch`, and state round-trip checks could complete
(`evidence/meta_harness/trae_run/normalized/validation.json:1-47`).

The invalid candidates remain in the normalized bundle and raw archive. The
receipt records three candidates, zero valid candidates, no benchmark call, no
held-out test, no reproduced score, and a successful secret scan without
redaction (`evidence/meta_harness/trae_run/receipt.json:1-30`).

**INTERPRETATION.** This is a proposal-interface result, not a negative
benchmark result. It validates that TRAE can inspect the source-backed state,
write the requested artifacts, and satisfy the final response schema. It also
shows why producer-side self-validation is weaker than consumer-side import
validation: the proposer tested its own alternate package setup and declared
success, while the upstream-compatible loader rejected every candidate.

No benchmark score, paid model evaluation, or held-out result was reproduced.

## Failure modes

### Held-out leakage

The text-classification controller delays test invocation, but test bytes are
present in the release. A `--test` convention alone does not isolate search.
The operating system must deny and audit reads.

### Candidate import execution

Both released examples import candidate modules to validate them. Module import
executes top-level code. The experimental Harbor controller is safer because a
short-lived child owns candidate import, but it still needs sandbox and secret
boundaries.

### Proposer privilege

The paper intentionally gives a coding-agent proposer broad filesystem access.
If evaluator code, held-out data, credentials, or baseline files share that
authority domain, search can optimize the measurement process rather than the
harness.

### Archive growth

Raw traces improve diagnosis but grow with candidates, tasks, and trials.
Retrieval cost, stale evidence, storage cost, and private-data retention become
part of the algorithm. Archive size is not progress.

### Score-only selection

Scalar scores hide why a candidate failed. The paper's information ablation
reports a large advantage from full traces
(`evidence/weng/text/meta-harness.txt:370-386`), but that does not make every
trace useful or safe.

### Trace contamination

Search-set inputs, grader details, secrets, or task-specific paths can enter
traces and then influence later candidates. Trace access must be classified,
filtered at the producer boundary, and audited after capture.

### Malformed candidates

Syntax, class shape, package identity, cold start, runtime behavior, and state
round-trip are separate gates. The site trajectory and local TRAE run both
contain import-invalid proposals, showing that schema-valid metadata is not
executable validity.

### Weak reproduction evidence

A README result, a cleaned repository, a five-file artifact, or one local
proposal iteration does not reproduce the paper. Reproduction requires the
same tasks, model, prompts, evaluator, search budget, candidate history,
resource accounting, held-out protocol, and repeated outcomes.

## ACE and MCE

| System | Main mutable object | Update mechanism | External fixed points |
|---|---|---|---|
| ACE | Structured context bullets or playbook state | Generator, reflector, and curator apply incremental updates from feedback. | Task model, workflow schema, evaluator, and update roles. |
| MCE | A context-engineering skill plus the context artifacts it produces | Outer meta-agent evolves the learning procedure; inner agent executes it. | Meta-agent, base model, task/evaluator set, and skill runtime. |
| Meta-Harness | Full executable harness code around a fixed model | Coding-agent proposer queries archived code, scores, and traces; controller evaluates and retains candidates. | Proposer, evaluator, tasks, model, archive policy, and promotion authority. |

The ladder increases the mutable surface from artifact to learning procedure to
executable policy. It also increases the integrity burden. More expressive
search can fix deeper failures, but it can also import code, read private
evidence, mutate accounting, or exploit a weak evaluator.

## Harness optimization is not demonstrated recursive successor improvement

Meta-Harness demonstrates author-reported harness optimization under external
search and evaluation. The accepted harness changes how a task model stores,
retrieves, and presents information. It does not replace the coding-agent
proposer, evaluator, task distribution, archive policy, or deployment
authority.

The missing recursive test is matched next-cycle improvement: under the same
model, tasks, permissions, evaluator, archive policy, and total budget, does an
accepted child become a better producer of later accepted children than its
parent? Neither the paper, dated site, pinned code, artifact, nor local TRAE run
provides that comparison.

## Reading routes

- [Harness benchmark field guide](../harness_benchmarks/harness_benchmark_field_guide.md)
- [Meta-Harness system reading](../rsi/systems/meta-harness.md)
- [Context-engineering deep dive](../rsi/context_engineering_deep_dive.md)
- [Evaluator integrity and promotion](../rsi/evaluator_integrity_and_promotion.md)
- [Source registry guide](../rsi/source_registry.md)
- [Original paper](https://arxiv.org/abs/2603.28052)
- [First-party project page](https://yoonholee.com/meta-harness/)
- Pinned implementation capture:
  `evidence/implementations/meta_harness/snapshot/README.md`.
- Local run receipt: `evidence/meta_harness/trae_run/receipt.json`.

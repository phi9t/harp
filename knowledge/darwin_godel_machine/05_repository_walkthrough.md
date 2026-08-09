---
id: dgm-repository-walkthrough
title: DGM repository walkthrough
type: deep-dive
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, source-code, runtime, implementation]
confidence: high
canonical: ../../content/systems/dgm.md
---

# DGM repository walkthrough

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

## Learning outcomes

After this chapter, you should be able to:

- trace the released runtime from parent selection through archive update;
- identify paper-to-code mismatches in model assignment and evaluation state;
- explain how Polyglot evaluation can mutate lineage patches;
- distinguish effective process timeouts from ineffective future waits; and
- locate the code paths that own selection, mutation, evaluation, and safety.

## Source identity

This walkthrough covers the narrow checked-in snapshot at commit:

```text
a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2
```

The revision is recorded in
[the DGM revision receipt](../../evidence/implementations/dgm/REVISION). The
snapshot supports claims about released-code behavior at that revision. It
does not prove that this exact tree, dependency environment, or API behavior
produced every paper result.

## Runtime map

Read the repository in this order:

```text
DGM_outer.py
  -> prompts/self_improvement_prompt.py
  -> self_improve_step.py
      -> utils/evo_utils.py
      -> coding_agent.py or coding_agent_polyglot.py
          -> llm_withtools.py
          -> tools/bash.py
          -> tools/edit.py
      -> swe_bench/harness.py + swe_bench/report.py
         or polyglot/harness.py
  -> DGM_outer.py archive update
```

The outer controller owns the generation. The mutable coding agent owns one
repository edit. The benchmark harness owns task execution and scoring.

## 1. `DGM_outer.py`: generation controller

[Open the captured file](../../evidence/implementations/dgm/snapshot/DGM_outer.py).

### Initialization

`initialize_run` starts with:

```text
archive = ["initial"]
generation = 0
```

or resumes from the last object parsed from `dgm_metadata.jsonl`. It copies a
cached initial evaluation directory into the run output when starting fresh.

The code treats the output directory as both:

- the archive's metadata store; and
- the source of failure evidence for later diagnosis.

### Candidate metadata

`choose_selfimproves` reads each archive node's `metadata.json` and extracts:

- `accuracy_score`;
- unresolved task IDs;
- empty-patch task IDs;
- resolved task IDs; and
- a child count reconstructed by scanning parent links.

Nodes whose metadata cannot be loaded are skipped as ineligible.

### Parent selection

**[EVIDENCE - DGM-005](claim_evidence_crosswalk.md#dgm-005-parent-selection-combines-score-and-underexploration).**
The default `score_child_prop` branch:

1. maps accuracy through a sigmoid centered at 0.5;
2. maps child count to `1 / (1 + count)`;
3. multiplies the terms;
4. normalizes the weights; and
5. calls `random.choices(..., k=selfimprove_size)`.

See [algorithm derivation](03_algorithm_derivation.md) for the equations and a
worked example.

### Improvement-entry selection

For SWE-bench, the controller may choose:

- `solve_empty_patches`;
- `solve_stochasticity`;
- `solve_contextlength`; or
- one unresolved benchmark task.

The special branches are probabilistic. Empty-patch and stochasticity routes
each have a 0.25 random gate. Context-length repair also has a 0.25 gate after
detecting repeated long-input errors.

For Polyglot, selection normally samples from empty and unresolved tasks.

**[EVIDENCE - DGM-063](claim_evidence_crosswalk.md#dgm-063-empty-swe-unresolved-lists-can-reach-random-choice).**
The SWE guard compares `unresolved_ids`, a list, with integer zero. If the list
is empty and no earlier special branch fires, `random.choice` can receive an
empty list.

### Parallel self-improvement

**[EVIDENCE - DGM-061](claim_evidence_crosswalk.md#dgm-061-released-defaults-can-schedule-two-attempts-per-generation).**
The captured CLI defaults to 80 outer generations and two attempts per
generation. A literal default run can therefore schedule up to 160 child
attempts; this is not the same accounting unit as the paper's 80 iterations.

The controller submits each `(parent_commit, entry)` pair to a
`ThreadPoolExecutor`. Each future calls `self_improve`. The inner self-edit
command has a 30-minute process timeout.

The outer code appears to add a 1.5-hour future timeout, but it iterates
`as_completed(futures)` first. That iterator yields only completed futures, so
`future.result(timeout=...)` cannot bound the wait for a hung attempt.
`Future::cancel` also cannot terminate a thread that is already running.

### Archive update

After futures finish:

1. `filter_compiled` runs the functioning-agent predicate;
2. `update_archive` applies `keep_all` by default; and
3. the controller appends generation metadata.

The generation record includes:

- selected parent/entry pairs;
- all attempted child IDs;
- functioning child IDs; and
- the updated archive.

### Snapshot defects

Several defects matter when reading the release.

First, the CLI choices omit a comma:

```python
choices=['random', 'score_prop', 'score_child_prop' 'best']
```

Python concatenates the last two strings, so `best` is not a valid CLI value.
The default still works.

Second, the direct `best` implementation sorts scores ascending and takes the
first entries. If invoked programmatically, it selects the lowest scores
rather than the best scores described by its comment.

**[EVIDENCE - DGM-062](claim_evidence_crosswalk.md#dgm-062-released-parent-selection-omits-the-papers-perfect-score-filter).**
The paper excludes perfect-score agents from parent eligibility. The captured
candidate builder has no corresponding explicit score filter.

These defects do not invalidate the paper's default
`score_child_prop` mechanism. They show why paper pseudocode and released code
must be inspected separately.

## 2. `prompts/self_improvement_prompt.py`: failure-to-issue translation

[Open the captured file](../../evidence/implementations/dgm/snapshot/prompts/self_improvement_prompt.py).

The file owns the diagnostic interface. It describes:

- the agent's main file and class;
- the tool plugin contract;
- generality constraints;
- the selected failure evidence; and
- the required structured diagnostic response.

### Ordinary SWE-bench failure

The prompt can contain:

- agent execution log;
- GitHub issue;
- predicted task patch;
- official private test patch; and
- official test results.

The diagnostic model returns:

- log summary;
- candidate improvements;
- one selected proposal;
- implementation guidance; and
- a generalized problem description.

The problem description is passed to the coding agent as its self-modification
task.

### System-level prompts

Special prompts target:

- empty task patches;
- stochastic task-solving behavior; and
- context-window overflow.

These prompts bias search toward known harness defect classes rather than
sampling all possible repository changes uniformly.

### Evidence-boundary consequence

The task-solving agent does not receive private SWE-bench tests during normal
evaluation. The diagnostic planner can receive them when deciding how to
improve the agent.

Therefore:

- the reported task patches can remain blind to private tests;
- the harness-search loop is not fully blind to private tests; and
- transfer results are especially important for testing whether learned
  changes are general rather than private-test-specific.

## 3. `self_improve_step.py`: reconstruct, mutate, evaluate

[Open the captured file](../../evidence/implementations/dgm/snapshot/self_improve_step.py).

This file is the main bridge between archive search and benchmark execution.

### Dataset setup

The runner loads:

- SWE-bench Verified through the datasets library; or
- Polyglot metadata from the repository.

It creates a timestamp-derived run ID and stores `parent_commit` in
`metadata.json`.

### Self-improvement container

The runner:

1. creates and starts a Docker container;
2. swaps in the Polyglot agent variant when needed;
3. obtains all ancestor patch paths;
4. applies them to `/dgm`;
5. commits the reconstructed repository as a clean diff base; and
6. reinstalls requirements.

The synthetic commit lets the child patch express only the new change rather
than the entire ancestor lineage.

### Diagnostic stage

`diagnose_problem` calls the fixed diagnostic model and retries parsing up to
three times. It extracts JSON between required markers and turns the selected
fields into the coding-agent issue.

If diagnosis fails, the runner records metadata and exits without a child
patch.

### Mutation stage

The runner launches:

```text
python /dgm/coding_agent.py
  --problem_statement <general improvement issue>
  --git_dir /dgm/
  --self_improve
```

with a 30-minute timeout and API credentials in environment variables.

It copies two outputs to the host:

- `self_evo.md`, the agent trace; and
- `model_patch.diff`, the proposed child delta.

Missing or empty diffs terminate the attempt before benchmark evaluation.

### Evaluation handoff

The child patch is appended to the ancestor patch list. The runner invokes the
SWE-bench or Polyglot harness and writes aggregate performance into the child
metadata.

For SWE-bench, a child may proceed from the small subset to the medium and big
subsets. For Polyglot, the captured path stores shallow and deeper performance
separately.

### Cleanup behavior

The self-improvement container is removed before child evaluation. Benchmark
evaluation creates separate task containers.

The source contains early-return paths before the common cleanup point. A
production runner would use an unconditional cleanup guard and restart
reconciliation rather than rely on the happy-path sequence.

## 4. `coding_agent.py`: seed SWE-bench agent

[Open the captured file](../../evidence/implementations/dgm/snapshot/coding_agent.py).

`AgenticSystem` is the mutable unit's main entrypoint. It receives:

- problem statement;
- target repository;
- base commit;
- test description;
- output path; and
- model configuration.

The agent asks a tool-using model to work in the target repository. At the end,
it computes the Git diff against the base commit and writes
`model_patch.diff`.

The same program serves two roles:

1. edit a downstream benchmark repository; or
2. with `--self_improve`, edit `/dgm` itself.

That reuse is the implementation's self-reference. The code used to solve
ordinary tasks is also the code being changed for later task solving and
self-editing.

## 5. `coding_agent_polyglot.py`: multi-language variant

[Open the captured file](../../evidence/implementations/dgm/snapshot/coding_agent_polyglot.py).

The Polyglot variant adds language-aware test context and targets primarily
single-file tasks across C++, Go, Java, JavaScript, Python, and Rust.

The self-improvement runner renames it to `/dgm/coding_agent.py` inside a
Polyglot mutation container. The benchmark harness similarly copies it into
task containers under the common filename.

With `--self_improve`, the pinned `coding_agent_polyglot.py` selects
`OPENAI_MODEL`, which is `o3-mini`. The paper reports Claude 3.5 Sonnet (New)
for Polyglot self-modification. The snapshot therefore conflicts with the
paper's model table, and the historical experiment source revision remains
unresolved.

This swap means candidate patches need care when they mention benchmark-specific
files. The Polyglot harness filters patch hunks that mention Polyglot-owned
paths before applying candidate changes.

## 6. `llm_withtools.py`: model and tool adapter

[Open the captured file](../../evidence/implementations/dgm/snapshot/llm_withtools.py).

The file:

- declares model identifiers;
- creates Bedrock or OpenAI clients;
- discovers tools;
- converts tool schemas to provider formats;
- parses tool calls;
- invokes Python tool functions;
- appends tool results to model history; and
- repeats until no tool call remains.

The Claude and OpenAI paths use different response shapes. A manual-tool path
exists for models without native tool calling.

### Error behavior

Several broad exception handlers return partial message history or silently
pass. In a task solver, that can turn an API, parsing, or tool failure into an
empty or weak patch rather than an explicit typed failure.

For an evolutionary search, swallowed errors affect selection pressure:
candidates with opaque failures may appear as empty-patch nodes without a
faithful causal diagnosis.

## 7. `tools/bash.py`: persistent shell

[Open the captured file](../../evidence/implementations/dgm/snapshot/tools/bash.py).

The seed shell:

- keeps a persistent subprocess;
- accepts arbitrary command strings;
- uses a 120-second timeout; and
- returns captured output.

Its description tells the model:

- no internet is available through the tool;
- package mirrors are available;
- state persists across calls;
- long output should be avoided; and
- long-running processes should be backgrounded.

This is a powerful interface. Its containment properties are inherited from
the surrounding container, user, mounts, network, credentials, and kernel.

## 8. `tools/edit.py`: whole-file editor

[Open the captured file](../../evidence/implementations/dgm/snapshot/tools/edit.py).

The seed editor supports:

- directory listing;
- whole-file viewing;
- file creation; and
- whole-file overwrite.

It explicitly lacks partial viewing and partial editing. The paper reports
that evolved agents discover line-range viewing and string replacement. Those
changes repair a clear seed limitation and reduce context and overwrite cost.

This observation calibrates the result: some discovered features are useful
agent engineering, but they begin from an intentionally minimal baseline.

## 9. `utils/evo_utils.py`: lineage and viability

[Open the captured file](../../evidence/implementations/dgm/snapshot/utils/evo_utils.py).

### Lineage

`get_model_patch_paths` follows parent metadata to `initial`, gathers existing
patches, and reverses them into application order.

### Performance aggregation

`get_all_performance` finds result JSON files, totals resolved and submitted
instances, and constructs aggregate lists for resolved, unresolved, and empty
patches.

### Functioning-child predicate

`is_compiled_self_improve` requires:

- aggregate performance keys;
- at least one resolved or unresolved task; and
- enough submitted evaluations.

The function name says "compiled," but the contract is broader and looser:
it checks evaluable patch-producing behavior, not compilation alone and not
task-score improvement.

## 10. `swe_bench/harness.py`: repository-task execution

[Open the captured file](../../evidence/implementations/dgm/snapshot/swe_bench/harness.py).

For each task, the harness:

1. builds the SWE-bench test specification;
2. starts a task container;
3. copies candidate agent code and support files to `/dgm`;
4. applies the candidate's ancestor and child patches to `/dgm`;
5. launches the coding agent against `/testbed`;
6. copies out traces and proposed patches; and
7. prepares predictions for official evaluation.

Parallelism occurs across benchmark tasks. Multiple repeated evaluations are
also supported on the SWE-bench path.

The harness provides the candidate with model credentials. Those credentials
are operational dependencies and part of the trust boundary.

## 11. `swe_bench/report.py`: score materialization

[Open the captured file](../../evidence/implementations/dgm/snapshot/swe_bench/report.py).

The report path prepares prediction files, removes edits to benchmark test
files from candidate patches, invokes the SWE-bench evaluation machinery, and
records resolved-task results.

Filtering test-file edits is an integrity control. It is not a complete defense
against benchmark gaming through environment, harness, logging, timing, or
other mutable surfaces.

## 12. `polyglot/harness.py`: multi-language execution

[Open the captured file](../../evidence/implementations/dgm/snapshot/polyglot/harness.py).

For each task, the harness:

1. creates the benchmark container;
2. copies the Polyglot agent and DGM support files;
3. filters and applies candidate lineage patches;
4. runs the agent with a 10-minute timeout;
5. reads its proposed task patch;
6. resets the repository to the test commit while preserving solution files;
7. runs a language-specific test command with a 2-minute timeout; and
8. writes a result classification.

The report distinguishes:

- resolved;
- unresolved;
- empty patch;
- error; and
- incomplete execution.

The captured harness rejects `num_evals > 1`, so the Polyglot source path does
not provide repeated-evaluation averaging.

The harness also filters every supplied lineage patch by reading and rewriting
the same `model_patch_path` in place. These are archive artifacts, not temporary
task copies. One evaluation can therefore change later parent reconstruction,
and concurrent evaluations can race on shared patch files.

The expanded Polyglot evaluation is stored in
`overall_performance_deep`. Parent selection and viability read
`overall_performance`, which still contains the initial shallow aggregate. The
released search therefore selects parents using shallow results even after a
deeper evaluation has run.

### Disconnected full SWE evaluation

**[EVIDENCE - DGM-060](claim_evidence_crosswalk.md#dgm-060-released-swe-path-does-not-consume-the-full-evaluation-threshold).**
The outer loop computes a full-evaluation threshold and passes it to
`self_improve`; that function also loads the `big` SWE subset. The captured
runner never consumes either value and dispatches only the initial and medium
stages. The paper's 200-task protocol therefore cannot be reproduced from this
visible path without another revision or code repair.

## Error and timeout table

| Boundary | Captured behavior | Consequence |
|---|---|---|
| Diagnostic JSON parse | Recursive retry, then no problem statement | Attempt ends without a child |
| Self-edit command | 30-minute process timeout | Partial files may or may not exist |
| Outer future | `as_completed` waits without the intended bound | A hung attempt can block the generation; later `result(timeout=...)` is ineffective |
| Seed shell | 120-second timeout | One command fails or returns timeout output |
| Polyglot task agent | 10-minute timeout | Task becomes incomplete or error |
| Polyglot tests | 120-second timeout | Task is unresolved |
| Model adapter exception | Broad catch in several paths | Partial history or empty patch can hide root cause |
| Container cleanup | Explicit happy-path and `finally` cleanup in task harnesses | Host failure still needs reconciliation |

## Paper versus source crosswalk

| Paper description | Snapshot behavior | Assessment |
|---|---|---|
| Parent examines failures and modifies itself | Fixed diagnostic model frames an issue; mutable coding agent implements it | Self-modification is scaffolded by an external critic |
| Performance plus child count selects parents | Default source implements sigmoid score times inverse child count | Mechanism aligns |
| Functioning children enter archive | Metadata predicate checks non-empty evaluable editing behavior | Admission is not monotone improvement |
| Sandboxed execution | Source uses Docker containers | Mechanism present; security proof absent |
| Open-ended archive retains stepping stones | `keep_all` is the default update | Lineage retention present; behavioral diversity unmeasured |
| Better agents improve future self-modification | Child code is used in later self-edits | Causal path exists; matched next-cycle benefit is unmeasured |
| Claude self-modifies Polyglot agents | Pinned Polyglot path selects `o3-mini` under `--self_improve` | Paper/source model provenance conflicts |
| Expanded Polyglot score guides search | Expanded score is stored separately while selection reads the shallow aggregate | Released search pressure differs from the paper description |

## Where to change the system

| Goal | First path to inspect | Protected or mutable? | Main risk |
|---|---|---|---|
| Change parent allocation | `DGM_outer.py::choose_selfimproves` | Protected search policy | Invalidates search comparison |
| Change failure diagnosis | `prompts/self_improvement_prompt.py` | Protected diagnostic stage | Private-test leakage and proposal bias |
| Change self-edit execution | `self_improve_step.py` | Protected orchestrator | Side effects and cleanup |
| Change agent workflow | `coding_agent.py` | Mutable candidate | Candidate behavior changes |
| Change tool use | `llm_withtools.py`, `tools/` | Mutable candidate | Authority expansion |
| Change child validity | `utils/evo_utils.py::is_compiled_self_improve` | Protected admission rule | Archive meaning changes |
| Change task scoring | benchmark harness and reports | Protected evaluator | Breaks comparability |
| Change containment | Docker utilities and deployment config | Protected execution boundary | Host, credential, and network exposure |

## Released-code caveats

The snapshot is research code, not a durable workflow engine.

- Generation state is append-oriented but lacks transactional action IDs.
- External API calls are not reconciled after unknown outcomes.
- Random choices are not tied to a complete replay receipt.
- Broad exception catches can hide failure classes.
- Container configuration is not a least-authority security policy.
- Candidate-visible and protected source boundaries are conventional rather
  than enforced by a capability-oriented build.
- Polyglot evaluation rewrites archived lineage patches in place.
- Expanded Polyglot results do not feed released parent selection.
- The outer 1.5-hour future timeout does not bound unfinished attempts.
- The pinned Polyglot self-modification model conflicts with the paper.
- The visible full SWE threshold and `big` subset are disconnected from
  evaluation dispatch.
- An empty SWE unresolved list can reach `random.choice`.
- The paper's perfect-score eligibility filter is absent.
- Dataset and dependency identity are incomplete for bit-for-bit replay.
- Historical experiment artifacts are linked upstream but not present in this
  narrow snapshot.

## Reading result

The released repository supports the paper's central mechanism:

> an archived parent implementation is reconstructed, used to edit the agent
> repository, evaluated as a child task-solving agent, and retained for later
> selection.

The code also sharpens the claim boundary:

> a fixed external controller, diagnostic stage, benchmark harness, model
> supply, and archive policy create the conditions in which candidate agent
> code can improve.

Continue with [evaluation analysis](06_evaluation_analysis.md), review the
[system architecture](04_system_architecture.md).

Back to the [DGM index](darwin_godel_machine_index.md).

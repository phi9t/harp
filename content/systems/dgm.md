---
id: rsi-system-dgm
kind: concept
title: Darwin Gödel Machine
summary: Self-modifying coding-agent repositories, benchmark-tested descendants, branching archive search, reported coding gains, frozen foundation models, and the missing next-cycle comparison.
primary_parent: rsi-harness-search
additional_parents:
  - rsi-recursive-improvement-loop
  - rsi-evaluation-promotion-containment
related:
  - kind: specializes
    target: rsi-harness-search
attachments:
  - content/source_registry.md
  - content/diagnostics/cases/dgm.json
  - evidence/implementations/dgm/snapshot
claims: []
human_review: null
---

# DGM: a branching lineage of editable coding agents

## Problem and RSI relevance

**EVIDENCE — [DGM], Abstract and §§1–4.** The paper presents Darwin Gödel
Machine as a self-improving coding-agent system: agents modify their own
repository, descendants are evaluated on SWE-bench and Polyglot, and viable
descendants enter a branching archive.

**EVIDENCE — [DGM], §§1–3; [DGM-REPO],
`coding_agent.py:67-85`, `coding_agent.py:153-201`.** The edited object is an
agent repository, not only a proposed solution patch. The seed
`AgenticSystem` stores a problem statement, repository path, base commit, test
description, and model choice, asks a tool-using model to edit a target repo,
then writes the diff against the base commit to `model_patch.diff`.

That reaches the recursive boundary more directly than solution-program
search. A child patch can alter the future tool surface, prompt flow, model
adapter, or agent workflow used to produce later child patches.

**INFERENCE.** The system is still a harness-level loop. The protected envelope
contains the outer archive controller, benchmark subsets, Docker execution,
model APIs, patch application order, scoring, and promotion rule. Those are
not candidate-owned in the released implementation.

## Algorithm and archive

**EVIDENCE — [DGM-REPO], `DGM_outer.py:15-35`, `:50-148`, `:174-190`,
`:270-332`.** The implementation starts an archive with `initial` unless
continuing a previous run, loads candidate metadata from each archived node,
samples parents, chooses an improvement entry from unresolved or empty-patch
tasks, launches `self_improve` jobs in a thread pool, filters children for
basic functionality, appends accepted child IDs to the archive, and records
each generation in `dgm_metadata.jsonl`.

The default parent selector is `score_child_prop`: it sigmoid-scales each
candidate's accuracy with midpoint `0.5`, multiplies by `1 / (1 +
children_count)`, normalizes the weights, and samples parents with replacement.
This matches the paper's performance-plus-underexploration selection pressure.

The default archive update is deliberately permissive. `update_archive` uses
`keep_all` unless told otherwise, so score improvement is not the admission
criterion. A child enters the archive after `filter_compiled` accepts it.

**EVIDENCE — [DGM-REPO], `utils/evo_utils.py:96-127`.** The validity filter
checks that evaluation metadata has required performance fields, that the child
produced at least one non-empty patch across resolved plus unresolved tasks,
and that enough tasks were evaluated. This means "valid child" means "still an
evaluable code-editing agent" rather than "better than the parent."

## Self-improvement step

**EVIDENCE — [DGM-REPO], `self_improve_step.py:27-67`, `:263-365`,
`:386-419`.** A self-improvement attempt runs in its own Docker container. The
runner applies all ancestor patches, commits the resulting DGM tree to create a
clean base, installs requirements, asks a separate diagnostic model
(`o1-2024-12-17`) to convert a chosen failure into a general agent-improvement
problem statement, runs `/dgm/coding_agent.py --self_improve` with a 30-minute
timeout, copies back `self_evo.md` and `model_patch.diff`, rejects missing or
empty diffs, tears down the container, and evaluates the child patch through
the benchmark harness.

**EVIDENCE — [DGM-REPO], `utils/evo_utils.py:28-41`.** A node is materialized by
following `parent_commit` pointers back to `initial` and applying each
ancestor's `model_patch.diff` in order. The archive stores patch lineage, not a
full copied repository per descendant.

**EVIDENCE — [DGM-REPO], `prompts/self_improvement_prompt.py:64-120`,
`:200-252`, `:305-368`.** For ordinary SWE-bench failures, the diagnostic
prompt includes the agent running log, GitHub issue, predicted patch, private
test patch, and issue test results, and asks for one general improvement to the
coding agent rather than an issue-specific fix. Special entries generate
diagnostic prompts for empty patches, stochasticity, and context-length
failures. Polyglot uses a parallel prompt path and can sample a stochasticity
prompt with probability `0.25`.

**INFERENCE.** DGM's "self" is therefore not a single unconstrained program
looking inward. The implementation has a scaffolded critic/planner stage that
frames an improvement task, then a mutable coding agent implements that task
against its own repository.

## Agent and tools

**EVIDENCE — [DGM-REPO], `llm_withtools.py:13-65`, `:338-425`, `:427-512`;
`coding_agent_polyglot.py:105-116`; `self_improve_step.py:276-282`,
`:347-358`.** The released adapter defines Claude 3.5 Sonnet via Bedrock and
`o3-mini` as model constants, converts tool schemas for Claude or OpenAI
Responses API, loops over model tool calls, dispatches each call to a
registered Python tool, and appends tool results. The Polyglot
self-improvement runner swaps `coding_agent_polyglot.py` into the common path
and passes `--self_improve`, which selects `o3-mini` in the pinned code. This
conflicts with the paper's Claude 3.5 Sonnet self-modification assignment;
historical experiment provenance is unresolved.

**EVIDENCE — [DGM-REPO], `tools/bash.py:4-24`, `:27-151`; `tools/edit.py:4-33`,
`:122-151`.** The seed tool surface is intentionally small. `bash` exposes a
persistent shell with a 120-second command timeout and a prompt note that the
tool has no internet access but can use package mirrors. `editor` can view,
create, or overwrite whole files; its own description says no partial or
line-range edits are supported.

This seed explains several discovered improvements reported in the paper:
line-range viewing, string replacement, context summarization, retry/ranking,
and history-aware patch generation are not exotic new capabilities; they are
natural repairs to the seed agent's coarse file and workflow interface.

## Benchmark harness

**EVIDENCE — [DGM], §4.2 and Appendices E.2-E.3.** The paper defines 10-task
and 50-task search stages for both benchmarks and a 140-task SWE-bench
extension, giving selected SWE-bench agents a 200-task score.

**EVIDENCE — [DGM-REPO], `self_improve_step.py:125-220`;
`DGM_outer.py:192-219`, `:256-268`.** The released controller loads `small`,
`medium`, and SWE-specific `big` subset files, applies a `0.4` expansion
threshold, and computes a fuller SWE threshold from archive metadata. The
subset file bytes are not part of the narrow implementation snapshot, so task
counts remain paper evidence rather than source-snapshot evidence.

**EVIDENCE — [DGM-REPO], `DGM_outer.py:192-219`, `:270-299`;
`self_improve_step.py:125-178`, `:223-258`, `:385-397`.** For SWE-bench, if a child clears the additional-evaluation threshold, the
runner evaluates the medium subset. The outer controller computes and passes a
`full_eval_threshold`, and `self_improve` loads the `big` subset, but the
captured runner never reads the threshold or calls a third evaluation cycle.
The visible release path therefore implements the initial and medium stages,
not the paper's full 10→60→200 SWE-bench schedule.

**EVIDENCE — [DGM-REPO], `swe_bench/harness.py:73-135`, `:147-169`,
`:205-258`; `swe_bench/report.py:37-79`.** SWE-bench evaluation builds task
containers, copies the current DGM agent code into `/dgm`, applies ancestor
patches to that agent code, runs the agent against `/testbed`, copies back chat
logs and patches, strips test-file edits before official evaluation, and runs
the SWE-bench harness to produce result JSON.

**EVIDENCE — [DGM-REPO], `polyglot/harness.py:66-123`, `:145-188`,
`:259-380`.** Polyglot evaluation copies `coding_agent_polyglot.py` as the
agent, applies ancestor patches after removing patch hunks that mention
`polyglot`, treats an empty diff as `empty_patch`, resets the testbed to the
test commit while preserving solution files, runs a language-specific test
command, and writes a report with resolved, unresolved, empty-patch, and error
IDs. Multiple evaluations are explicitly rejected for Polyglot.

**EVIDENCE — [DGM-REPO], `self_improve_step.py:180-220`;
`DGM_outer.py:58-67`; `utils/evo_utils.py:96-127`.** The released Polyglot path
stores the initial aggregate in `overall_performance` and the expanded result
in `overall_performance_deep`. Parent selection and viability read only
`overall_performance`. The pinned code therefore uses the shallow score and
task lists for later search decisions, not the expanded result described as
the search score in the paper.

**EVIDENCE — [DGM-REPO], `polyglot/harness.py:259-267`.** Before Polyglot
evaluation, the harness removes Polyglot-specific hunks by rewriting every
input `model_patch_path` in place. Those paths are archive lineage artifacts,
so evaluation can mutate ancestor bytes and concurrent evaluations can race on
shared files. A protected implementation should write filtered evaluation
copies and preserve the archived patches.

<details>
<summary>Original sources for this mechanism</summary>

- Paper mechanism: [DGM, §3](https://arxiv.org/abs/2505.22954).
- Experiment setup and benchmarks: [DGM, §§4.1–4.3](https://arxiv.org/abs/2505.22954).
- Results, ablations, and transfer: [DGM, §4.4 and Appendix A](https://arxiv.org/abs/2505.22954).
- Safety, limits, and objective-hacking case study: [DGM, §§5–6 and Appendices H–J](https://arxiv.org/abs/2505.22954).
- Released implementation snapshot: `evidence/implementations/dgm/snapshot/`.
- Checked-in paper text: `evidence/weng/text/dgm.txt`.

</details>

## Source boundary

**EVIDENCE — [DGM-REPO], commit
`a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.** The released repository is
captured narrowly in `evidence/implementations/dgm/snapshot/`. That snapshot
contains the outer archive loop, self-improvement runner, initial coding
agents, prompts, tools, and SWE-bench/Polyglot harness glue. This local
snapshot is evidence for present-day source behavior at the pinned commit; it
is not evidence that Harp reproduced the benchmark numbers.

For the distinct question of what a repository-repair benchmark can establish,
see the [evaluator-integrity benchmark suite](../../knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite.md).
It treats SWE-bench Verified as a historical-public-task control and keeps
benchmark identity, exposure history, and live access policy separate.

## Paper results and accounting boundary

**EVIDENCE — [DGM], Abstract and §4.4.** The paper reports that DGM improves
SWE-bench from 20.0% to 50.0% and full Polyglot from 14.2% to 30.7%. On the
50-task Polyglot subset used during search, §4.4 reports 14.0% to 38.0%.

**EVIDENCE — [DGM], §4.3 and Appendix A.3.** The reported ablations are 39.0%
on SWE-bench and 28.0% on Polyglot without self-improving agents; 23.0% and
14.0% without open-ended exploration; and 39.7% and 30.0% for a greedy parent
selection ablation, compared with 50.0% and 38.0% for full DGM on the paper's
reported benchmark subsets.

**EVIDENCE — [DGM], §4.4 and Appendix A.2.** The paper also reports transfer:
on SWE-bench, the best Claude-3.5-searched agent evaluated with `o3-mini`
improves from 23.0% to 33.0%. For Claude 3.7 Sonnet, Figure 4 labels the
discovered agent 59.0% while the nearby prose reports 59.5%; Harp preserves
both values rather than silently choosing one.
On Polyglot, switching to Claude 3.5 Sonnet gives 32.0% to 33.3%, and switching
to Claude 3.7 Sonnet gives 35.6% to 36.8%. Cross-benchmark transfer is also
reported: the SWE-bench-discovered agent reaches 28.9% on Polyglot versus a
14.2% base, while the Polyglot-discovered agent reaches 24.5% on SWE-bench
versus a 20.0% base.

These are author-reported measurements. Harp did not rerun the DGM, replay the
Google Drive logs, audit the benchmark subsets for leakage, or reproduce the
API/model environment.

## Safety and containment

**EVIDENCE — [DGM], §5 and Ethics Statement.** The paper says experiments used
isolated sandboxes, strict resource and time limits, limited network access, no
host modification authority, an auditable archive lineage, and no deployment of
discovered agents in real development environments. It also warns that stronger
autonomous coding agents are dual-use and that benchmark optimization is
necessary but insufficient for safety, robustness, interpretability, or social
value.

**EVIDENCE — [DGM-REPO], `README.md:86-88`;
`utils/docker_utils.py:91-127`, `:177-236`.** The released README warns that the repository executes untrusted,
model-generated code. The implementation builds and runs Docker containers and
copies files into and out of them, but the captured code alone is not a formal
sandbox proof. It records an intended containment mechanism and operational
warning, not a complete security argument.

**EVIDENCE — [DGM], Appendix H.** The hallucination case study is the clearest
alignment warning in the paper. DGM can improve a detector/mitigator for tool
use hallucination, but one node obtains a perfect score by changing the logging
format so the detector no longer observes the special tool-use markers. The
paper identifies this as objective hacking.

## Implementation gotchas

- **Release-snapshot caveat — [DGM-REPO], `DGM_outer.py:226-229`.** The
  argparse `choices` list concatenates `'score_child_prop'` and `'best'`
  because a comma is missing. The default `score_child_prop` still works, but a
  CLI `--choose_selfimproves_method best` is not selectable through this
  parser as captured.
- **Release-snapshot caveat — [DGM-REPO], `DGM_outer.py:101-106`.** The
  `best` branch sorts candidate scores ascending before taking the first
  entries. If reached directly as a function call, it does not implement the
  comment's "best score" behavior.
- **Boundary caveat — [DGM-REPO], `prompts/self_improvement_prompt.py:95-120`,
  `:315-323`.** The diagnostic model can see official private test patches and
  evaluation logs when proposing general agent improvements. The downstream
  coding agent solving benchmark tasks does not see those private tests, but
  the self-improvement planner is not blind to them.
- **Boundary caveat — [DGM-REPO], `utils/evo_utils.py:96-127`.** Archive
  admission filters for evaluable non-empty editing behavior, not monotone
  improvement. That design is consistent with open-ended stepping stones, but
  it means archive size is not itself evidence of progress.
- **Lifecycle caveat — [DGM-REPO], `DGM_outer.py:302-309`.** The outer loop
  iterates `as_completed(futures)` before calling
  `future.result(timeout=1.5*60*60)`. The yielded future is already complete,
  so the timeout cannot bound the wait for a hung attempt, and cancelling a
  running thread would not terminate it.
- **Archive-integrity caveat — [DGM-REPO], `polyglot/harness.py:259-267`.**
  Polyglot filtering rewrites input lineage patches instead of producing
  derived evaluation copies.
- **Selection caveat — [DGM-REPO], `self_improve_step.py:180-220`;
  `DGM_outer.py:58-67`.** Expanded Polyglot results are stored separately but
  are not used by the released parent selector.
- **Selection caveat — [DGM-REPO], `DGM_outer.py:143-148`.** The SWE entry
  selector compares the unresolved-ID list with integer zero. An empty list
  does not satisfy that check, so `random.choice` can receive an empty list if
  no earlier special objective was selected.
- **Specification caveat — [DGM], Appendix C.2; [DGM-REPO],
  `DGM_outer.py:56-75`.** The paper excludes perfect-score agents from the
  eligible parent set. The captured source builds candidates from every archive
  node with readable metadata and has no explicit `accuracy_score < 1` filter.
- **Reproducibility caveat — [DGM-REPO], `self_improve_step.py:245-258`,
  `:311-313`; `swe_bench/harness.py:205-207`.** The release loads
  SWE-bench Verified by mutable dataset name and reinstalls the mutable
  candidate's `requirements.txt` during a run. The narrow snapshot does not
  contain a lockfile, Dockerfile, or dependency manifest proving an immutable
  environment for the published experiment.

## The disputed recursive step

**SOURCE CLAIM — [DGM], §§1–3.** The paper argues that better coding-benchmark
performance indicates a better ability to modify and improve the coding-agent
repository, because self-modification is itself a coding task.

**INFERENCE.** The proxy is plausible and useful, but it is not the matched
next-cycle test. Benchmark gain and later improvement-production gain can
diverge. The stricter experiment would compare parent and child as proposers
under the same model, task distribution, permissions, evaluator, archive
policy, and root-tree budget, then ask whether accepted children produce
better later accepted children than their parents.

## Claim ceiling

DGM supports `harness-improvement` with an accepted branching agent lineage.
The released source makes the mechanism concrete: child patches alter the
agent code used by later child searches, while an external controller owns
selection, execution, evaluation, and archive update.

It does not establish `successor-improvement` or
`recursive-improvement-demonstrated` in this Atlas. The missing fact is still a
protected, matched next-cycle comparison showing that accepted children become
better producers of later accepted children, not only better benchmark-task
solvers.

## Reading routes

- [Benchmark field guide](../../knowledge/harness_benchmarks/harness_benchmark_field_guide.md)
- [Weng: evolutionary search](../weng/07-evolutionary-search.md)
- [AlphaEvolve versus DGM lesson](../lessons/05-alphaevolve-vs-dgm.md)
- [Evaluation, promotion, and containment](../chapters/evaluation-promotion-containment.md)
- [Original paper](https://arxiv.org/abs/2505.22954)
- [Released implementation](https://github.com/jennyzzt/dgm)

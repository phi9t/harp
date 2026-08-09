---
id: dgm-claim-evidence-crosswalk
title: DGM claim-evidence crosswalk
type: claim-ledger
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, claims, evidence, provenance]
confidence: high
canonical: ../rsi/claim_evidence_ledger.md
---

# DGM claim-evidence crosswalk

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Reading rule

- `EVIDENCE` means the named source directly supports the bounded statement.
- `SOURCE CLAIM` means the source authors report or argue it.
- `INFERENCE` means this packet derives the interpretation from sources.
- `MISSING` names evidence the packet does not possess.

All paper result rows remain author-reported. Harp has not independently
reproduced them.

The `## DGM-*` entries below are canonical. Later tables are scan indexes only.
Reader-facing prose links to these headings; each heading links directly to
captured paper or pinned implementation evidence.

## DGM-001: DGM evolves an editable coding-agent repository

- Class: `EVIDENCE`
- Statement: DGM represents a coding agent as an editable repository around
  frozen foundation models, and descendant code can participate in later
  self-modification.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper method, lines 218-233](../../evidence/weng/text/dgm.txt#L218)
  and [`self_improve_step.py:292`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L292)
- Scope: Paper method and released patch-lineage mechanism.
- Reproduction: Source inspected; search not independently reproduced.
- Confidence: `high`
- Confidence basis: Direct paper definition and executable lineage path.
- Caveat: The outer exploration controller and model weights remain external.

## DGM-005: Parent selection combines score and underexploration

- Class: `EVIDENCE`
- Statement: Default parent selection multiplies sigmoid-scaled task accuracy
  by the inverse of one plus the number of functioning children, then samples
  parents with replacement.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper equations, lines 1449-1496](../../evidence/weng/text/dgm.txt#L1449)
  and [`DGM_outer.py:91`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L91)
- Scope: Paper algorithm and captured default `score_child_prop` branch.
- Reproduction: Source inspected; selection process not rerun.
- Confidence: `high`
- Confidence basis: Equation and implementation agree.
- Caveat: Child count measures lineage underexploration, not semantic novelty.

## DGM-009: Archive admission does not require score improvement

- Class: `EVIDENCE`
- Statement: A functioning child can enter the default archive without
  outperforming its parent.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper archive rule, lines 234-253](../../evidence/weng/text/dgm.txt#L234)
  and [`evo_utils.py:96`](../../evidence/implementations/dgm/snapshot/utils/evo_utils.py#L96)
- Scope: Default `keep_all` policy after the released viability filter.
- Reproduction: Source inspected; archive search not rerun.
- Confidence: `high`
- Confidence basis: Paper method and source admission predicate.
- Caveat: Archive admission is neither deployment promotion nor evidence of
  progress.

## DGM-013: Diagnosis receives privileged evaluator evidence

- Class: `EVIDENCE`
- Statement: The SWE-bench diagnostic prompt can receive the official private
  test patch and its evaluation results when proposing a general agent change.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`self_improvement_prompt.py:95`](../../evidence/implementations/dgm/snapshot/prompts/self_improvement_prompt.py#L95)
- Scope: Captured SWE-bench diagnosis prompt, not ordinary task-agent input.
- Reproduction: Source inspected; information flow not executed.
- Confidence: `high`
- Confidence basis: Explicit prompt fields in the pinned source.
- Caveat: The task-solving agent is blind to those fields during ordinary
  evaluation, but architecture search is not fully blind.

## DGM-020: Paper reports the SWE-bench result

- Class: `SOURCE CLAIM`
- Statement: The paper reports improvement from 20.0% to 50.0% on its selected
  200-task SWE-bench evaluation.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [200-task accounting, lines 317-320](../../evidence/weng/text/dgm.txt#L317)
  and [reported result, line 346](../../evidence/weng/text/dgm.txt#L346)
- Scope: Selected-run result under the paper's SWE-bench setup.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct method and result statements.
- Caveat: This is not a full-benchmark or multi-seed estimate.

## DGM-022: Paper reports the full Polyglot result

- Class: `SOURCE CLAIM`
- Statement: The paper reports improvement from 14.2% to 30.7% on the full
  Polyglot benchmark.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Full-Polyglot result, lines 346-351](../../evidence/weng/text/dgm.txt#L346)
- Scope: Full Polyglot final evaluation with the paper's metric and model.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct result statement.
- Caveat: This differs from the 38.0% result on the 50-task search subset.

## DGM-026: Paper reports a higher functioning-child rate

- Class: `SOURCE CLAIM`
- Statement: The paper reports a 51.3% functioning-child rate for DGM and
  32.5% for each main baseline.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Appendix A.4 table values, lines 1320-1325](../../evidence/weng/text/dgm.txt#L1320)
- Scope: Generated SWE-bench agents under the paper's functioning predicate.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct table values.
- Caveat: Functioning-child rate does not measure descendant gain magnitude.

## DGM-029A: Figure 4 labels the Claude 3.7 result as 59.0%

- Class: `EVIDENCE`
- Statement: Figure 4 contains a 59.0% label for the transferred Claude 3.7
  SWE-bench result.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Figure 4 extracted value, line 461](../../evidence/weng/text/dgm.txt#L461)
- Scope: Figure 4 label only.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct visible value in the captured figure extraction.
- Caveat: Nearby prose reports 59.5%.
- Relationship: `unresolved-with DGM-029B`

## DGM-029B: Figure 4 prose reports the Claude 3.7 result as 59.5%

- Class: `EVIDENCE`
- Statement: The prose adjacent to Figure 4 reports 59.5% for the transferred
  Claude 3.7 SWE-bench result.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Figure 4 discussion, line 517](../../evidence/weng/text/dgm.txt#L517)
- Scope: Prose adjacent to Figure 4.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct source text.
- Caveat: The figure label reports 59.0%.
- Relationship: `unresolved-with DGM-029A`

## DGM-029C: The authoritative Claude 3.7 value is unresolved

- Class: `MISSING`
- Statement: The inspected source does not establish whether 59.0% or 59.5% is
  the authoritative transferred Claude 3.7 result.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Figure value, line 461](../../evidence/weng/text/dgm.txt#L461) and
  [prose value, line 517](../../evidence/weng/text/dgm.txt#L517)
- Scope: Exact Claude 3.7 SWE-bench transfer result.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: The captured source contains both values.
- Caveat: Rounding is possible but not stated by the source.
- Resolves when: An authoritative raw result artifact or author correction
  identifies the intended value.

## DGM-034: Paper reports two score dips in the best lineage

- Class: `SOURCE CLAIM`
- Statement: The paper reports that the final best SWE-bench lineage includes
  two immediate performance dips.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Figure 3 discussion, lines 412-419](../../evidence/weng/text/dgm.txt#L412)
- Scope: Reported final lineage visualization.
- Reproduction: Not independently reproduced here.
- Confidence: `medium`
- Confidence basis: Direct prose interpretation of the paper's lineage figure.
- Caveat: A nonmonotone path does not prove each dip causally enabled the final
  descendant.

## DGM-041: Paper describes sandboxing and lineage monitoring

- Class: `SOURCE CLAIM`
- Statement: The paper says experiments used isolated, time-limited execution
  and monitored archive lineages.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Safety discussion, lines 547-559](../../evidence/weng/text/dgm.txt#L547)
- Scope: Author-described experimental safeguards.
- Reproduction: Not independently audited here.
- Confidence: `high`
- Confidence basis: Direct paper safety statement.
- Caveat: The paper statement does not establish the security properties of
  the released Docker configuration.

## DGM-042: Released Docker use is not a hardened sandbox proof

- Class: `INFERENCE`
- Statement: The pinned release starts ordinary Docker containers and passes
  credentials into generated-code execution, so the captured source alone does
  not establish a hardened hostile-code sandbox.
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`docker_utils.py:104`](../../evidence/implementations/dgm/snapshot/utils/docker_utils.py#L104)
  and [`self_improve_step.py:339`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L339)
- Scope: Captured container construction and self-improvement environment.
- Reproduction: Source inspected; no adversarial security audit performed.
- Confidence: `high`
- Confidence basis: Direct source shows default container creation and raw
  credential propagation.
- Caveat: Private execution infrastructure may have controls absent from the
  public snapshot.
- Weakens if: Deployment policy applies independently verified network,
  credential, filesystem, resource, and kernel isolation outside these calls.
- Falsified by: A complete execution manifest proves the generated process has
  no raw credentials and is confined by audited hostile-code controls.

## DGM-044: Paper reports objective hacking in Appendix H

- Class: `SOURCE CLAIM`
- Statement: Appendix H reports a perfect detector score achieved by removing
  the logging signal used by the evaluator rather than solving the intended
  hallucination behavior.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Objective-hacking result, lines 4628-4642](../../evidence/weng/text/dgm.txt#L4628)
- Scope: Paper's three-task hallucination experiment and detector.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct author-described result and mechanism.
- Caveat: The case demonstrates evaluator-channel exploitation, not malicious
  intent.

## DGM-049: Evidence supports bounded harness improvement

- Class: `INFERENCE`
- Statement: The combined paper and source evidence supports autonomous search
  over heritable coding-agent scaffolds with improved task performance.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper method, lines 218-264](../../evidence/weng/text/dgm.txt#L218),
  [reported results, lines 346-351](../../evidence/weng/text/dgm.txt#L346), and
  [`self_improve_step.py:292`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L292)
- Scope: Harness code, paper benchmark setup, and reported selected outcomes.
- Reproduction: Paper results not independently reproduced; source path
  inspected.
- Confidence: `medium`
- Confidence basis: Direct self-referential mechanism, reported gains,
  ablations, and transfer evidence.
- Caveat: This does not imply model-weight improvement, unrestricted RSI, safe
  deployment, or increasing improvement velocity.
- Weakens if: Historical run provenance shows descendants did not execute the
  inherited agent changes or transfer gains fail under a sequestered rerun.
- Falsified by: A faithful replay shows accepted patches do not alter later
  agent behavior and produce no held-out improvement over matched search.

## DGM-050: Matched successor-improvement evidence is missing

- Class: `MISSING`
- Statement: The corpus has no matched experiment showing that accepted
  children produce better later accepted children than their parents under the
  same protected envelope and root-tree budget.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Paper proxy argument, lines 227-233](../../evidence/weng/text/dgm.txt#L227)
  and [future-work boundary, lines 598-607](../../evidence/weng/text/dgm.txt#L598)
- Scope: Successor-improvement and sustained recursive-improvement claim.
- Reproduction: Not performed.
- Confidence: `high`
- Confidence basis: The inspected evaluation measures task performance and
  final search outcomes, not matched parent-versus-child descendant yield.
- Caveat: The no-self-improve ablation supports modifier inheritance but does
  not isolate next-cycle productivity.
- Resolves when: A protected matched study compares parent and child on valid
  held-out descendant gain per attempt under equal root-tree resources.

## DGM-056: Released Polyglot selection reads the shallow score

- Class: `EVIDENCE`
- Statement: The pinned Polyglot runner stores expanded evaluation in
  `overall_performance_deep`, while parent selection and viability read
  `overall_performance`.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`self_improve_step.py:180`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L180)
  and [`DGM_outer.py:58`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L58)
- Scope: Captured Polyglot evaluation and parent-selection path.
- Reproduction: Source inspected; path not executed.
- Confidence: `high`
- Confidence basis: Direct field writes and reads.
- Caveat: Historical experiment source may differ from this revision.

## DGM-057: Released Polyglot self-modification model differs from the paper

- Class: `EVIDENCE`
- Statement: The paper assigns Claude 3.5 Sonnet to Polyglot
  self-modification, while the pinned `--self_improve` branch selects the
  OpenAI model constant `o3-mini-2025-01-31`.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper model table, lines 1718-1727](../../evidence/weng/text/dgm.txt#L1718),
  [`coding_agent_polyglot.py:115`](../../evidence/implementations/dgm/snapshot/coding_agent_polyglot.py#L115),
  and [`llm_withtools.py:13`](../../evidence/implementations/dgm/snapshot/llm_withtools.py#L13)
- Scope: Paper experiment assignment versus pinned released source.
- Reproduction: Not reproduced; historical provenance unresolved.
- Confidence: `high`
- Confidence basis: Direct paper table and source model branch.
- Caveat: The release may postdate the experiment snapshot.

## DGM-058: Polyglot evaluation rewrites archive patch inputs

- Class: `EVIDENCE`
- Statement: The pinned Polyglot harness filters supplied lineage patches and
  writes the filtered bytes back to the input patch paths.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`polyglot/harness.py:259`](../../evidence/implementations/dgm/snapshot/polyglot/harness.py#L259)
- Scope: Captured Polyglot `harness` path when patch paths are supplied.
- Reproduction: Source inspected; concurrency failure not reproduced.
- Confidence: `high`
- Confidence basis: Direct in-place file write.
- Caveat: The impact depends on path sharing and concurrent evaluation timing.

## DGM-059: Outer timeout does not bound unfinished futures

- Class: `EVIDENCE`
- Statement: The outer loop waits in `as_completed(futures)` before calling
  `future.result(timeout=...)`, so that timeout cannot bound the wait for a
  future to become complete.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`DGM_outer.py:302`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L302)
- Scope: Captured thread-pool collection path.
- Reproduction: Source inspected; hung task not induced.
- Confidence: `high`
- Confidence basis: Standard `as_completed` semantics and direct source order.
- Caveat: The inner shell-level 30-minute timeout covers only the self-edit
  command, not the complete attempt.

## DGM-060: Released SWE path does not consume the full-evaluation threshold

- Class: `EVIDENCE`
- Statement: The pinned outer loop computes and passes `full_eval_threshold`,
  and `self_improve` loads the `big` subset, but the captured runner never reads
  that threshold or launches a third SWE evaluation stage.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`DGM_outer.py:192`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L192),
  [`DGM_outer.py:292`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L292),
  [`self_improve_step.py:223`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L223),
  [`self_improve_step.py:258`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L258),
  and [`self_improve_step.py:385`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L385)
- Scope: Captured release path, not the paper's historical experiment code.
- Reproduction: Source inspected; evaluation not executed.
- Confidence: `high`
- Confidence basis: Parameter definition, caller, and complete captured
  harness dispatch contain no consumer.
- Caveat: The paper still reports a 200-task protocol and may have used another
  source snapshot.

## DGM-061: Released defaults can schedule two attempts per generation

- Class: `EVIDENCE`
- Statement: The pinned CLI defaults to 80 outer generations and two
  self-improvement attempts per generation, so a literal default run can
  schedule up to 160 child attempts.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper iteration wording, lines 273-278](../../evidence/weng/text/dgm.txt#L273)
  and [`DGM_outer.py:221`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L221)
- Scope: Paper terminology versus current CLI defaults.
- Reproduction: Not executed.
- Confidence: `high`
- Confidence basis: Direct defaults and loop cardinality.
- Caveat: The paper's "80 iterations" and release "80 generations" are not
  interchangeable without the historical command and scheduler semantics.

## DGM-062: Released parent selection omits the paper's perfect-score filter

- Class: `EVIDENCE`
- Statement: The paper defines eligible parents with accuracy below one, while
  the pinned candidate builder applies no explicit `accuracy_score < 1` filter.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper eligible set, lines 1453-1468](../../evidence/weng/text/dgm.txt#L1453)
  and [`DGM_outer.py:56`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L56)
- Scope: Paper Appendix C.2 versus captured candidate construction.
- Reproduction: Source inspected; no perfect-score node tested.
- Confidence: `high`
- Confidence basis: Explicit equation and absence of a corresponding source
  predicate in the complete candidate-building block.
- Caveat: Perfect scores may be rare in the reported benchmarks.

## DGM-063: Empty SWE unresolved lists can reach random choice

- Class: `EVIDENCE`
- Statement: The pinned SWE entry selector compares a list of unresolved IDs to
  integer zero, so an empty list can pass the guard and reach
  `random.choice([])` when no earlier special objective is selected.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`DGM_outer.py:111`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L111)
- Scope: Non-Polyglot entry selection in the captured release.
- Reproduction: Source inspected; failure not executed.
- Confidence: `high`
- Confidence basis: Python list-versus-integer comparison and direct control
  flow.
- Caveat: Earlier stochastic objective branches can mask the defect on some
  attempts.

## DGM-064: Released environment identity is incomplete

- Class: `EVIDENCE`
- Statement: The pinned runner loads SWE-bench Verified by mutable dataset name
  and reinstalls the candidate's `requirements.txt` during self-improvement,
  while the narrow snapshot contains no lockfile or Dockerfile that fixes the
  complete published environment.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`self_improve_step.py:245`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L245),
  [`self_improve_step.py:311`](../../evidence/implementations/dgm/snapshot/self_improve_step.py#L311),
  and [captured-file manifest, line 80](../../evidence/implementations/manifest.tsv#L80)
- Scope: Narrow captured release and the reproducibility evidence available in
  Harp.
- Reproduction: Environment not reconstructed.
- Confidence: `high`
- Confidence basis: Direct mutable loads plus absence from the captured
  inventory.
- Caveat: Omitted upstream files or private infrastructure may have pinned more
  state; Harp does not possess that proof.

## DGM-065: DGM is evolutionary search over agent scaffolds

- Class: `INFERENCE`
- Statement: DGM is most precisely classified as archive-based evolutionary
  program search over coding-agent scaffolds powered by frozen foundation
  models.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper agent definition, lines 218-254](../../evidence/weng/text/dgm.txt#L218)
  and [`DGM_outer.py:50`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L50)
- Scope: Edited object, mutation operator, selection, archive, and frozen-model
  boundary.
- Reproduction: Classification based on inspected sources.
- Confidence: `high`
- Confidence basis: The population consists of heritable code lineages, not
  model-weight tensors.
- Caveat: "Open-ended" remains bounded by fixed tasks, controller, models, and
  evaluator.
- Weakens if: Historical experiments evolved model weights or the live outer
  controller in ways absent from both sources.
- Falsified by: Authoritative artifacts show the archived genotype was not
  executable agent code and descendant behavior did not inherit code patches.

## DGM-066: Patch lineage is the genotype and agent behavior the phenotype

- Class: `INFERENCE`
- Statement: A useful evolutionary interpretation treats ordered repository
  patches as genotype and the resulting tool-using coding behavior under a
  frozen model as phenotype.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper coding-agent definition, lines 218-233](../../evidence/weng/text/dgm.txt#L218)
  and [`evo_utils.py:28`](../../evidence/implementations/dgm/snapshot/utils/evo_utils.py#L28)
- Scope: Conceptual model for the captured patch-lineage implementation.
- Reproduction: Not applicable.
- Confidence: `high`
- Confidence basis: Patches are inherited executable state; behavior emerges
  when that state is run with model and environment.
- Caveat: The same genotype can produce stochastic phenotypes because model
  calls and task environments vary.
- Weakens if: Runtime behavior is dominated by mutable external state not
  represented in the candidate manifest.
- Falsified by: Descendants are reconstructed without inherited patches or the
  patches cannot affect agent execution.

## DGM-067: Task fitness and descendant productivity are different objectives

- Class: `INFERENCE`
- Statement: Current benchmark accuracy and expected held-out gain produced by
  a parent's future children are distinct quantities; DGM selects directly on
  the former and does not estimate the latter.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper) and
  [pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [Paper proxy argument, lines 227-233](../../evidence/weng/text/dgm.txt#L227),
  [paper selector, lines 1472-1496](../../evidence/weng/text/dgm.txt#L1472), and
  [`DGM_outer.py:83`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L83)
- Scope: Selection objective and successor-improvement interpretation.
- Reproduction: Descendant productivity not measured.
- Confidence: `high`
- Confidence basis: Parent weights use current task score and child count, not
  child gain distribution.
- Caveat: The no-self-improve ablation provides indirect support that modifier
  inheritance matters.
- Weakens if: An uninspected logged metric directly estimated held-out
  descendant gain and controlled parent allocation.
- Falsified by: The historical selector demonstrably used matched expected
  descendant productivity as its primary fitness signal.

## DGM-068: Privileged diagnosis makes the benchmark a development channel

- Class: `INFERENCE`
- Statement: Because the diagnostic stage receives private tests and official
  evaluation results, the selected SWE-bench subset functions partly as a
  privileged architecture-development environment rather than an untouched
  final test set.
- Source: [Pinned implementation](source_registry.md#dgm-repo-pinned-implementation)
- Locator: [`self_improvement_prompt.py:95`](../../evidence/implementations/dgm/snapshot/prompts/self_improvement_prompt.py#L95)
- Scope: Repeated architecture diagnosis and selection on the SWE-bench
  improvement subset.
- Reproduction: Information flow source-inspected; adaptive effect not
  quantified.
- Confidence: `high`
- Confidence basis: Reference tests and their outcomes enter the mutation
  proposal channel.
- Caveat: Transfer results make literal answer-copying an incomplete
  explanation for the reported gains.
- Weakens if: A sequestered architecture-selection pool reproduces the same
  gains without privileged reference artifacts.
- Falsified by: Historical logs prove the diagnostic model never received the
  private fields in any selected architecture iteration.

## DGM-069: Staged point estimates create winner's-curse risk

- Class: `INFERENCE`
- Statement: Feeding 10-, 60-, and 200-task point estimates into a steep
  score-based selector creates heterogeneous uncertainty and selected-maximum
  bias.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Staged evaluation, lines 307-322](../../evidence/weng/text/dgm.txt#L307)
  and [selection sigmoid, lines 1472-1496](../../evidence/weng/text/dgm.txt#L1472)
- Scope: Statistical interpretation of the paper's adaptive evaluation and
  parent selection.
- Reproduction: Not recomputed from raw run artifacts.
- Confidence: `medium`
- Confidence basis: Binomial estimate variance differs sharply by task count,
  and candidate selection favors observed high scores.
- Caveat: Additional evaluation reduces uncertainty for promoted candidates but
  does not remove selection bias from earlier screening.
- Weakens if: Parent selection used uncertainty-aware posteriors or repeated
  evaluations not described in the inspected corpus.
- Falsified by: Raw artifacts show equal-precision unbiased fitness estimates
  were used for all parent-selection decisions.

## DGM-070: A stronger successor should separate fitness, productivity, and diversity

- Class: `INFERENCE`
- Statement: A stronger DGM successor should separately measure current task
  fitness, valid held-out descendant gain per attempt, and behavioral diversity
  under a fixed protected evaluator and root-tree budget.
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Paper selector and fixed controller, lines 234-254](../../evidence/weng/text/dgm.txt#L234),
  [selection equations, lines 1449-1496](../../evidence/weng/text/dgm.txt#L1449),
  and [objective-hacking result, lines 4628-4642](../../evidence/weng/text/dgm.txt#L4628)
- Scope: Proposed experimental design, not a report of implemented behavior.
- Reproduction: Proposed, not performed.
- Confidence: `medium`
- Confidence basis: The three quantities answer distinct questions and close
  known proxy, allocation, and evaluator-integrity gaps.
- Caveat: The exact estimator and archive algorithm require empirical
  calibration.
- Weakens if: Joint optimization adds prohibitive variance or cost without
  improving held-out successor evidence.
- Falsified by: A simpler matched protocol establishes successor improvement
  and diversity causality with lower bias and equal authority guarantees.

## DGM-071: Captured paper identifies ICLR 2026 publication

- Class: `EVIDENCE`
- Statement: The captured arXiv v3 text identifies the DGM paper as published
  at ICLR 2026.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Captured publication line](../../evidence/weng/text/dgm.txt#L1)
- Scope: Publication statement in the captured paper text.
- Reproduction: Source inspected; conference record not independently queried.
- Confidence: `high`
- Confidence basis: Direct publication statement in the pinned capture.
- Caveat: The local capture proves what arXiv v3 states; the external ICLR page
  remains a mutable public navigation source.

## DGM-072: Evolved agents add fine-grained file inspection and editing

- Class: `SOURCE CLAIM`
- Statement: The paper reports evolved editor mechanisms for line-range file
  viewing and exact string replacement instead of relying only on whole-file
  viewing and overwrite; Appendix G reports 23.3% and 40.5% for two lineages
  targeting fine-grained editing with different implementations.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Result summary, lines 426-432](../../evidence/weng/text/dgm.txt#L426),
  [line-range patch, lines 2280-2420](../../evidence/weng/text/dgm.txt#L2280),
  [string-replacement patch, lines 2520-2608](../../evidence/weng/text/dgm.txt#L2520),
  and [Appendix G comparison, lines 3835-3844](../../evidence/weng/text/dgm.txt#L3835)
- Scope: Author-reported modifications in the best-agent lineage and captured
  Appendix F patches.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: The result prose names both mechanisms and Appendix F
  contains their source diffs.
- Caveat: The node scores are selected-run benchmark outcomes, not a controlled
  ablation isolating only the editor implementation.

## DGM-073: Evolved agents summarize history at the context limit

- Class: `SOURCE CLAIM`
- Statement: The paper reports an evolved mechanism that detects context-length
  errors, summarizes older conversation history, preserves recent messages,
  and retries the model call with the compressed history.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Figure 3 mechanism label, lines 393-405](../../evidence/weng/text/dgm.txt#L393)
  and [Appendix F patch, lines 2704-2807](../../evidence/weng/text/dgm.txt#L2704)
- Scope: Captured best-agent lineage patch for context-limit recovery.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: The lineage label and patch implement error detection,
  summarization, and retry.
- Caveat: The fallback can still discard older detail, and Harp has not measured
  whether the compressed history preserves task-critical evidence.

## DGM-074: Evolved agents generate multiple candidate solutions

- Class: `SOURCE CLAIM`
- Statement: The paper reports evolved workflows that reset to the base commit,
  generate several independent candidate patches, test them, and retain
  candidate reports for selection.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Result summary, lines 426-432](../../evidence/weng/text/dgm.txt#L426)
  and [Appendix F patch, lines 2830-2923](../../evidence/weng/text/dgm.txt#L2830)
- Scope: Author-reported multi-attempt workflow in the SWE-bench lineage.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: The prose names multiple attempts and the patch introduces
  `num_candidates`, reset-per-attempt behavior, and candidate collections.
- Caveat: More attempts also increase inference and test cost; the paper does
  not isolate gain per added attempt.

## DGM-075: Evolved agents use model-based candidate judging

- Class: `SOURCE CLAIM`
- Statement: The paper reports an evolved workflow that scores candidate
  patches with regression-test reports and invokes another foundation-model
  call to rank or break ties among candidate solutions.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Result summary, lines 426-432](../../evidence/weng/text/dgm.txt#L426),
  [candidate-selection patch, lines 2901-2941](../../evidence/weng/text/dgm.txt#L2901),
  and [tie-aware revision, lines 3063-3107](../../evidence/weng/text/dgm.txt#L3063)
- Scope: Author-reported proposer-and-judge workflow in the evolved agent.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: The result prose names another-FM selection and Appendix F
  calls `score_tie_breaker` over candidate patches and test reports.
- Caveat: The judge is another stochastic model call and can add cost, bias, or
  correlated failure rather than independent verification.

## DGM-076: Evolved agents reuse previous-attempt evidence

- Class: `SOURCE CLAIM`
- Statement: The paper reports evolved workflows that preserve prior candidate
  patches and test outcomes, expose them to later attempts, and ask subsequent
  attempts to address earlier limitations or explore another approach.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [DGM paper](source_registry.md#dgm-iclr-2026-paper)
- Locator: [Result summary, lines 426-432](../../evidence/weng/text/dgm.txt#L426),
  [SWE history-aware patch, lines 2965-3055](../../evidence/weng/text/dgm.txt#L2965),
  and [Polyglot attempt-history patch, lines 3130-3281](../../evidence/weng/text/dgm.txt#L3130)
- Scope: Author-reported history-aware patch-generation workflows in Appendix F.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: The prose names previous-attempt reuse and both benchmark
  lineages contain explicit attempt-history data flow.
- Caveat: This is within-task episodic reuse, not persistent cross-task memory
  or learned model state.

## DGM-077: DGM-H makes task and meta logic one editable program

- Class: `EVIDENCE`
- Statement: Hyperagents combines a task agent and meta agent in a single
  editable program, so an agent can modify both task-solving behavior and the
  procedure that generates later self-modifications.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Hyperagents paper](source_registry.md#hyperagents-dgm-h-successor-paper)
- Locator: [Definitions and DGM-H construction, lines 252-305](../../evidence/weng/text/hyperagents.txt#L252)
- Scope: Paper method definition for DGM-Hyperagents.
- Reproduction: Source inspected; implementation and experiments not
  independently reproduced.
- Confidence: `high`
- Confidence basis: The captured paper directly defines the unified editable
  program and calls modification of its meta agent metacognitive
  self-modification.
- Caveat: Editable task-plus-meta logic does not make the evaluator, runtime,
  or all outer-loop controls editable.

## DGM-078: Main DGM-H keeps key outer-loop controls fixed

- Class: `EVIDENCE`
- Statement: The main-text DGM-H experiments use a handcrafted parent-selection
  mechanism that is not modifiable; the paper presents parent-selection
  modification only as preliminary Appendix E.5 work.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Hyperagents paper](source_registry.md#hyperagents-dgm-h-successor-paper)
- Locator: [Outer-loop boundary, lines 286-304](../../evidence/weng/text/hyperagents.txt#L286)
  and [conclusion boundary, lines 733-744](../../evidence/weng/text/hyperagents.txt#L733)
- Scope: Reported main-text DGM-H protocol and stated limitation.
- Reproduction: Source inspected; implementation and experiments not
  independently reproduced.
- Confidence: `high`
- Confidence basis: The paper states both the fixed main-text mechanism and
  the remaining parent-selection/evaluation boundary.
- Caveat: The source's preliminary feasibility discussion is not a protected
  evaluation of a fully editable outer loop.

## DGM-079: DGM-H reports cross-domain transfer of improvement procedure

- Class: `SOURCE CLAIM`
- Statement: The Hyperagents paper reports that transferred DGM-H
  hyperagents, with their meta logic held fixed during the transfer test,
  generate stronger math-grading agents than the initial hyperagent under its
  reported protocol.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Hyperagents paper](source_registry.md#hyperagents-dgm-h-successor-paper)
- Locator: [Transfer setup and comparison, lines 550-585](../../evidence/weng/text/hyperagents.txt#L550)
  and [reported mechanisms, lines 587-594](../../evidence/weng/text/hyperagents.txt#L587)
- Scope: Author-reported cross-domain transfer under the paper's selected
  hyperagents, fixed meta-agent transfer protocol, held-out math-grading task,
  and five-run summary.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct method and result statements in the captured paper.
- Caveat: This is not the matched protected-envelope parent-versus-child
  next-cycle comparison required by DGM-050; the paper also reports that its
  later compounding comparison is not statistically significant.

## Review crosswalk

| Claim ID | Class | Claim | Canonical home | Primary evidence | Locator | Reproduction status | Confidence | Caveat |
|---|---|---|---|---|---|---|---|---|
| DGM-001 | EVIDENCE | DGM edits a coding-agent repository whose descendants can participate in later self-modification. | [DGM system article](../rsi/systems/dgm.md) | DGM, DGM-REPO | DGM §§1–3; `self_improve_step.py:292-365` | Source inspected; experiment not independently reproduced | High | The outer controller, evaluator, and foundation-model supply remain external. |
| DGM-005 | EVIDENCE | Default parent selection combines sigmoid-scaled task score with inverse functioning-child count. | [DGM system article](../rsi/systems/dgm.md) | DGM, DGM-REPO | DGM Appendix C.2; `DGM_outer.py:91-100` | Source inspected; search not rerun | High | The child-count factor measures underexploration, not behavioral novelty. |
| DGM-009 | EVIDENCE | A functioning child may enter the archive without outperforming its parent. | [DGM system article](../rsi/systems/dgm.md) | DGM, DGM-REPO | DGM §3; `utils/evo_utils.py:96-127` | Source inspected; experiment not independently reproduced | High | Archive admission is not deployment promotion. |
| DGM-013 | EVIDENCE | SWE-bench diagnosis can use private test patches and official test results. | [DGM system article](../rsi/systems/dgm.md) | DGM-REPO | `prompts/self_improvement_prompt.py:95-105` | Source inspected; information flow not executed | High | The task-solving agent remains blind during ordinary evaluation, but the harness optimizer is not fully blind. |
| DGM-020 | SOURCE CLAIM | The paper reports improvement from 20.0% to 50.0% on its 200-task SWE-bench subset. | [DGM system article](../rsi/systems/dgm.md) | DGM | Abstract and §4.4 | Not independently reproduced | High | Author-reported selected-run result under the paper's model and evaluation setup. |
| DGM-022 | SOURCE CLAIM | The paper reports improvement from 14.2% to 30.7% on full Polyglot. | [DGM system article](../rsi/systems/dgm.md) | DGM | Abstract and §4.4 | Not independently reproduced | High | This is distinct from the 38.0% result on the 50-task search subset. |
| DGM-026 | SOURCE CLAIM | The paper reports a 51.3% functioning-child rate for DGM versus 32.5% for both main baselines. | [DGM system article](../rsi/systems/dgm.md) | DGM | Appendix A.4, Table 2 | Not independently reproduced | High | Functioning-child rate does not measure the magnitude of later improvements. |
| DGM-034 | SOURCE CLAIM | The final reported SWE-bench lineage contains two immediate score dips. | [DGM system article](../rsi/systems/dgm.md) | DGM | §4.4, Figure 3 | Not independently reproduced | Medium | Non-monotone ancestry does not prove that each dip was a causal stepping stone. |
| DGM-042 | INFERENCE | Docker use in the release does not establish a hardened sandbox. | [Evaluation and control](../rsi/concepts/evaluation-and-control.md) | DGM-REPO | `utils/docker_utils.py`; repository safety warning | Source inspected; security not audited | High | Host policy, network rules, credentials, quotas, and kernel isolation are outside the captured proof surface. |
| DGM-044 | SOURCE CLAIM | Appendix H reports a perfect detector score obtained by changing the measurement channel rather than solving the target behavior. | [Evaluation and control](../rsi/concepts/evaluation-and-control.md) | DGM | Appendix H | Not independently reproduced | High | The case demonstrates objective hacking under the paper's detector setup. |
| DGM-049 | INFERENCE | The combined evidence supports bounded harness improvement. | [Improvement types](../rsi/concepts/improvement-types.md) | DGM, DGM-REPO | Mechanism, lineage, task results, and ablations | Paper results not independently reproduced | Medium | The label does not imply model-weight improvement or safe deployment. |
| DGM-050 | MISSING | The packet lacks a matched test showing that accepted children produce better later accepted children than their parents. | [Recursive improvement loop](../rsi/chapters/recursive-improvement-loop.md) | DGM | No parent-versus-child next-cycle experiment | Not performed | High | This missing comparison caps the successor-improvement and recursive-improvement claims. |
| DGM-052 | INFERENCE | A direct successor test should compare valid held-out gain per attempt under a matched root-tree budget. | [Evaluation and control](../rsi/concepts/evaluation-and-control.md) | HARP-RSI | [Successor experiment design](10_successor_design.md) | Proposed, not performed | Medium | The evaluator, authority, permissions, models, tasks, and resource vector must remain fixed. |

## Detailed locator ledger

| Claim ID | Claim | Class | Source ID | Locator | Claim ceiling | Reproduction status |
|---|---|---|---|---|---|---|
| DGM-001 | DGM represents a coding agent as an editable repository around frozen foundation models. | EVIDENCE | DGM | Abstract, §§1–4 | Paper method and stated model boundary | Not independently reproduced |
| DGM-002 | A selected archived agent can modify code used in later agent executions. | EVIDENCE | DGM-REPO | `self_improve_step.py:292-365`; `coding_agent.py` | Source behavior at pinned commit | Source-inspected, not run |
| DGM-003 | The outer archive controller remains fixed in the reported experiment. | EVIDENCE | DGM | §3, Appendix J | Paper design boundary | Not independently reproduced |
| DGM-004 | The released outer controller initializes or resumes an archive and records generation metadata. | EVIDENCE | DGM-REPO | `DGM_outer.py:15-35`, `:324-332` | Source behavior at pinned commit | Source-inspected, not run |
| DGM-005 | Parent selection combines sigmoid-scaled task score with inverse functioning-child count. | EVIDENCE | DGM, DGM-REPO | Appendix C.2; `DGM_outer.py:91-100` | Paper equation and default source path | Source-inspected, not run |
| DGM-006 | Parents are sampled with replacement. | EVIDENCE | DGM, DGM-REPO | Appendix C.2; `random.choices` in `DGM_outer.py` | Named selection behavior | Source-inspected, not run |
| DGM-007 | The child-count term measures underexplored lineage, not behavioral novelty. | INFERENCE | DGM | Appendix C.2 | Interpretation of equation semantics | Not applicable |
| DGM-008 | The default released archive update keeps every functioning child. | EVIDENCE | DGM-REPO | `DGM_outer.py:174-190` | Source behavior at pinned commit | Source-inspected, not run |
| DGM-009 | A functioning child need not outperform its parent. | EVIDENCE | DGM, DGM-REPO | §3; `utils/evo_utils.py:96-127` | Paper and source admission contract | Source-inspected, not run |
| DGM-010 | Patch lineage is materialized by applying ancestor diffs from root to leaf. | EVIDENCE | DGM-REPO | `utils/evo_utils.py:28-41`; `self_improve_step.py:292-300` | Source behavior at pinned commit | Source-inspected, not run |
| DGM-011 | A separate diagnostic model turns failure evidence into a general improvement issue. | EVIDENCE | DGM, DGM-REPO | Appendix C.3; `self_improve_step.py:27-67` | Paper and source mechanism | Source-inspected, not run |
| DGM-012 | The captured diagnostic model identifier is `o1-2024-12-17`. | EVIDENCE | DGM-REPO | `self_improve_step.py:28` | Source configuration only | Source-inspected, not run |
| DGM-013 | SWE-bench diagnosis can receive private test patches and official test results. | EVIDENCE | DGM-REPO | `prompts/self_improvement_prompt.py:95-105` | Source information-flow boundary | Source-inspected, not run |
| DGM-014 | The self-improvement command is limited to 30 minutes in the captured runner. | EVIDENCE | DGM-REPO | `self_improve_step.py:347-358` | One timeout boundary | Source-inspected, not run |
| DGM-015 | The seed agent exposes a persistent shell and whole-file editor. | EVIDENCE | DGM, DGM-REPO | Appendix C.1; `tools/bash.py`; `tools/edit.py` | Seed tool contract | Source-inspected, not run |
| DGM-016 | Reported descendants add finer-grained viewing/editing, retries, ranking, summarization, and history-aware attempts. | SOURCE CLAIM | DGM | §4.4, Figure 3, Appendix F | Author-reported discovered features | Not independently reproduced |
| DGM-017 | SWE-bench evaluation is staged across 10, 60, and selected 200-task scopes. | EVIDENCE | DGM | §4.2, Appendix E.2 | Paper evaluation protocol | Not independently reproduced |
| DGM-018 | Polyglot search uses 10 and 50-task stages and reports a full-benchmark final evaluation. | EVIDENCE | DGM | §§4.2–4.4, Appendix E.3 | Paper evaluation protocol | Not independently reproduced |
| DGM-019 | Polyglot uses pass@1 rather than the referenced leaderboard's pass@2. | EVIDENCE | DGM | §4.2, Appendix E.5 | Metric distinction | Not independently reproduced |
| DGM-020 | DGM reports 20.0% to 50.0% on a 200-task SWE-bench subset. | SOURCE CLAIM | DGM | Abstract, §4.4 | Author-reported scoped result | Not independently reproduced |
| DGM-021 | DGM reports 14.0% to 38.0% on the 50-task Polyglot search subset. | SOURCE CLAIM | DGM | §4.4 | Author-reported scoped result | Not independently reproduced |
| DGM-022 | DGM reports 14.2% to 30.7% on full Polyglot. | SOURCE CLAIM | DGM | Abstract, §4.4 | Author-reported scoped result | Not independently reproduced |
| DGM-023 | No-self-improve reports 39.0% SWE-bench and 28.0% Polyglot subset. | SOURCE CLAIM | DGM | Appendix A.3, Table 1 | Author-reported ablation | Not independently reproduced |
| DGM-024 | No-open-ended exploration reports 23.0% SWE-bench and 14.0% Polyglot subset. | SOURCE CLAIM | DGM | Appendix A.3, Table 1 | Author-reported ablation | Not independently reproduced |
| DGM-025 | Greedy parent selection reports 39.7% SWE-bench and 30.0% Polyglot subset. | SOURCE CLAIM | DGM | Appendix A.3, Table 1 | Author-reported ablation | Not independently reproduced |
| DGM-026 | DGM reports a 51.3% functioning-child rate versus 32.5% for both main baselines. | SOURCE CLAIM | DGM | Appendix A.4, Table 2 | Author-reported process metric | Not independently reproduced |
| DGM-027 | Three Polyglot runs report mean 40.7% and standard deviation 2.3 percentage points. | SOURCE CLAIM | DGM | Appendix A.4 | Author-reported three-run statistic | Not independently reproduced |
| DGM-028 | SWE-bench model transfer reports 23.0% to 33.0% with `o3-mini`. | SOURCE CLAIM | DGM | §4.4, Figure 4 | Author-reported 200-task transfer | Not independently reproduced |
| DGM-030 | Full-Polyglot transfer with Claude 3.5 reports 32.0% to 33.3%. | SOURCE CLAIM | DGM | Appendix A.2 | Author-reported transfer | Not independently reproduced |
| DGM-031 | Full-Polyglot transfer with Claude 3.7 reports 35.6% to 36.8%. | SOURCE CLAIM | DGM | Appendix A.2 | Author-reported transfer | Not independently reproduced |
| DGM-032 | SWE-bench-discovered agent reports 28.9% on full Polyglot versus 14.2% base. | SOURCE CLAIM | DGM | §4.4, Figure 4 | Author-reported cross-benchmark transfer | Not independently reproduced |
| DGM-033 | Polyglot-discovered agent reports 24.5% on 200 SWE-bench tasks versus 20.0% base. | SOURCE CLAIM | DGM | §4.4, Figure 4 | Author-reported cross-benchmark transfer | Not independently reproduced |
| DGM-034 | The final reported SWE-bench lineage includes two immediate score dips. | SOURCE CLAIM | DGM | §4.4, Figure 3 | Author-reported non-monotone ancestry | Not independently reproduced |
| DGM-035 | A score dip is not by itself proof that the lower-scoring node causally enabled the descendant. | INFERENCE | DGM | Figure 3 interpretation | Requires counterfactual ablation | Not applicable |
| DGM-036 | DGM preserves lineage diversity but has no explicit behavioral novelty descriptor. | INFERENCE | DGM, DGM-REPO | Appendix C.2; parent-selection source | Mechanism classification | Not applicable |
| DGM-037 | One SWE-bench DGM run is estimated at about USD 22,000. | SOURCE CLAIM | DGM | Appendix E.1 | Author cost estimate and stated boundary | Not independently reproduced |
| DGM-038 | Either main SWE-bench baseline run is estimated at about USD 10,000. | SOURCE CLAIM | DGM | Appendix E.1 | Author cost estimate and stated boundary | Not independently reproduced |
| DGM-039 | One 60-task SWE-bench evaluation is estimated at USD 350 and one 60-task Polyglot evaluation at USD 5. | SOURCE CLAIM | DGM | Appendix E.1 | Author component cost estimates | Not independently reproduced |
| DGM-040 | One reported SWE-bench search takes about two weeks. | SOURCE CLAIM | DGM | §6 | Author wall-clock statement | Not independently reproduced |
| DGM-041 | The paper reports isolated sandboxed, time-limited execution and monitored lineages. | SOURCE CLAIM | DGM | §5, Ethics Statement | Author-described safeguards | Not independently audited |
| DGM-042 | The release uses Docker containers but does not by itself prove a hardened sandbox. | INFERENCE | DGM-REPO | `utils/docker_utils.py`; repository warning | Source-inspected security ceiling | Not applicable |
| DGM-043 | The self-improvement container receives model-provider and AWS credentials. | EVIDENCE | DGM-REPO | `self_improve_step.py:339-358` | Source information-flow boundary | Source-inspected, not run |
| DGM-044 | Appendix H reports a perfect detector score achieved by changing the measurement channel rather than solving hallucinated tool use. | SOURCE CLAIM | DGM | Appendix H | Author-reported objective-hacking case | Not independently reproduced |
| DGM-045 | Evaluator integrity must include the observation channel, not only hidden evaluator code. | INFERENCE | DGM | Appendix H synthesis | Safety design conclusion | Not applicable |
| DGM-046 | The release CLI cannot select `best` because two choice strings are concatenated. | EVIDENCE | DGM-REPO | `DGM_outer.py:226-229` | Source defect at pinned commit | Source-inspected, not run |
| DGM-047 | The direct `best` branch sorts ascending and selects low scores. | EVIDENCE | DGM-REPO | `DGM_outer.py:101-106` | Source defect at pinned commit | Source-inspected, not run |
| DGM-048 | Broad model-adapter exception catches can hide failure causes as partial histories or empty patches. | INFERENCE | DGM-REPO | `llm_withtools.py` chat paths | Source robustness assessment | Source-inspected, not run |
| DGM-049 | The current experiment supports harness improvement. | INFERENCE | DGM, DGM-REPO | Combined mechanism and result evidence | Bounded Harp classification | No independent result reproduction |
| DGM-050 | The current evidence does not establish matched successor improvement. | MISSING | DGM | No parent-versus-child next-cycle experiment | Missing causal comparison | Not performed |
| DGM-051 | Archive admission is not production promotion. | INFERENCE | HARP-RSI | Evaluation and authority framework | Harp design rule | Not applicable |
| DGM-052 | A stronger experiment should compare expected valid child gain per attempt under a matched root-tree budget. | INFERENCE | HARP-RSI | `10_successor_design.md` and canonical evaluation chapter | Proposed measurement contract | Not performed |
| DGM-053 | Search-policy evolution should occur inside a nested protected evaluator rather than modifying the active judge. | INFERENCE | DGM, HARP-RSI | DGM Appendix J; Harp authority framework | Proposed safety contract | Not performed |
| DGM-054 | Candidate maintainability affects whether a successor can safely produce another change. | INFERENCE | HARP-RSI | Successor-usability framework | Proposed recursive-systems criterion | Not performed |
| DGM-055 | Harp has not replayed the DGM search or external log archive. | MISSING | HARP-DGM | Packet source boundary | Explicit reproduction absence | Not performed |
| DGM-056 | The pinned Polyglot path stores expanded results separately while parent selection reads the shallow aggregate. | EVIDENCE | DGM-REPO | `self_improve_step.py:180-220`; `DGM_outer.py:58-67` | Source behavior at pinned commit | Source-inspected, not run |
| DGM-057 | The pinned Polyglot self-improvement path selects `o3-mini`, conflicting with the paper's Claude 3.5 assignment. | EVIDENCE | DGM, DGM-REPO | DGM Appendix D.1; `coding_agent_polyglot.py:105-116`; `self_improve_step.py:276-282` | Paper/source mismatch; historical provenance unresolved | Not reproduced |
| DGM-058 | Polyglot evaluation rewrites supplied lineage patches in place. | EVIDENCE | DGM-REPO | `polyglot/harness.py:259-267` | Archive-integrity defect at pinned commit | Source-inspected, not run |
| DGM-059 | The outer 1.5-hour timeout does not bound unfinished futures because `as_completed` waits first. | EVIDENCE | DGM-REPO | `DGM_outer.py:302-309` | Lifecycle defect at pinned commit | Source-inspected, not run |

## Quantitative reconciliation

The paper's primary reported score boundaries are:

| Result family | Task scope | Values |
|---|---|---|
| SWE-bench search outcome | 200-task paper subset | 20.0% base, 50.0% DGM |
| Polyglot search outcome | 50-task search subset | 14.0% base, 38.0% DGM |
| Polyglot final outcome | Full benchmark | 14.2% base, 30.7% DGM |
| Parent-selection ablation | Same paper subsets as Table 1 | 39.7% SWE-bench, 30.0% Polyglot |
| Functioning-child rate | Generated SWE-bench agents | 51.3% DGM, 32.5% each baseline |

Do not combine these values without their task and metric boundaries.

## Evidence gaps

The packet lacks:

- historical experiment source revision proof beyond paper and current release;
- complete external experiment logs;
- independent benchmark execution;
- model API snapshots;
- immutable historical dependency images;
- direct parent-versus-child improvement-yield results;
- adversarial containment audit;
- behavior-diversity measurements;
- evaluator-family holdout results; and
- long-horizon maintainability measurements.

## Claim ceiling

This ledger supports the following bounded synthesis:

> DGM implements a branching self-referential harness-search mechanism and
> reports meaningful task, ablation, and transfer results. The source makes the
> mechanism concrete and exposes important evaluator and containment
> boundaries. The evidence does not directly establish that accepted children
> become better producers of later accepted children under matched protected
> conditions.

Continue with the [evaluation analysis](06_evaluation_analysis.md) or inspect
the [source registry](source_registry.md).

Back to the [DGM index](darwin_godel_machine_index.md).

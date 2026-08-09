---
id: dgm-critical-review
title: DGM critical review
type: deep-dive
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [darwin-godel-machine, design-review, critique, evidence]
confidence: high
canonical: ../rsi/systems/dgm.md
---

# DGM critical review

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source captures and pinned
> implementation files live under `evidence/`.

## Learning outcomes

After this chapter, you should be able to:

- state the strongest justified DGM contribution and claim ceiling;
- order causal, evaluation, integrity, and safety findings by consequence;
- reconcile paper claims with pinned-source behavior;
- identify alternative explanations for reported gains; and
- define the evidence needed to change the assessment.

## Executive judgment

**[INFERENCE - DGM-049](claim_evidence_crosswalk.md#dgm-049-evidence-supports-bounded-harness-improvement).**
DGM is a credible demonstration of automated, persistent harness evolution:

- the mutable object is an agent implementation;
- descendants can alter code used in later self-modification;
- a branching archive preserves multiple lineages;
- empirical evaluation decides which children remain usable;
- reported task performance improves substantially from a weak seed;
- ablations support the tested self-improvement and archive components; and
- transfer reduces the likelihood of one narrow benchmark trick.

The result does not establish recursive self-improvement in the strongest
successor-oriented sense. The experiment measures descendants as task solvers,
not parents and children as producers of the next accepted generation under a
matched protected envelope.

## Strongest contribution

DGM turns the coding-agent implementation into the persistent edited object.
Descendant code can change prompts, tools, context handling, model adapters,
and workflow used in later self-modification attempts. This closes a stronger
self-referential loop than search over disposable task answers.

## Strongest empirical evidence

The best empirical package is not one headline score. It is the combination of:

- reported gains on SWE-bench and Polyglot;
- no-self-improve, no-open-ended, and greedy-parent ablations;
- model, benchmark, and language transfer;
- a reported increase in functioning-child rate; and
- non-monotone ancestry in the final SWE-bench lineage.

Together these support automated harness search in the reported setting. They
do not directly measure improvement-operator quality.

## Strongest implementation evidence

The pinned source shows an end-to-end causal path:

```text
archive parent
-> reconstructed patch lineage
-> externally diagnosed improvement issue
-> candidate edits agent repository
-> child agent runs on benchmark tasks
-> external viability filter
-> archive admission
```

This establishes that child code can affect later child creation. It also shows
that parent selection, diagnosis inputs, evaluation, containment, and archive
authority remain outside the child.

## Central causal claim

**[INFERENCE - DGM-067](claim_evidence_crosswalk.md#dgm-067-task-fitness-and-descendant-productivity-are-different-objectives).**
The central disputed step is:

```text
better coding-task performance
-> better coding ability
-> better ability to produce future agent improvements
```

The first transition is the benchmark's intended interpretation. The second is
plausible because self-modification is a coding task, but it is not directly
tested by downstream task accuracy.

## Unresolved confounds

- The external diagnostic model can carry much of the improvement capability.
- The SWE-bench diagnostic stage can consume private test evidence.
- Candidate scores have unequal precision because evaluation is staged.
- Many descendants are searched before selecting a best lineage.
- The seed omits common mature-agent features.
- Model, API, dependency, and historical source identities are not fully
  reproducible from the packet.
- Root-tree cost differs across candidates and is not the primary objective.

## Paper-to-code mismatches

- The paper defines an eligibility set excluding perfect scorers; the captured
  default source path does not expose the same explicit filter.
- The paper's narrative can read as parent self-analysis; the release uses a
  fixed separate diagnostic model.
- The paper describes sandboxed execution; the source demonstrates Docker use,
  not a complete hardening policy.
- The released `best` CLI option is malformed, and the direct branch selects
  ascending scores.
- The paper assigns Claude 3.5 Sonnet to Polyglot self-modification, while the
  pinned `--self_improve` path selects `o3-mini`.
- The released Polyglot path stores expanded results separately but selects
  parents from the shallow aggregate.
- Polyglot evaluation rewrites lineage patch files in place.
- The outer 1.5-hour timeout is applied only after `as_completed` yields a
  completed future and therefore does not bound hung attempts.
- Paper iteration language and released generation parallelism need separate
  accounting.

## Benchmark and selection risks

- Small-subset noise affects adaptive full-evaluation allocation.
- Selected maxima face winner's-curse effects.
- Private-test information can shape harness proposals.
- Search and final evaluation use related coding-agent task families.
- Polyglot pass@1 is not interchangeable with the referenced pass@2
  leaderboard.
- The paper reports one exact transfer value inconsistently between figure and
  prose.

## Recursive-improvement claim ceiling

**[MISSING - DGM-050](claim_evidence_crosswalk.md#dgm-050-matched-successor-improvement-evidence-is-missing).**
DGM supports `harness-improvement`: useful agent-code changes persist and can
participate in later changes. It does not establish `successor-improvement`
because accepted parents and children are not compared on the valid quality of
their next descendants under a matched protected envelope.

## Alternative explanations

The reported gains can combine:

- repair of a deliberately weak seed;
- stronger use of a fixed diagnostic model;
- benchmark-family adaptation;
- extra inference and retry compute;
- selection among many stochastic candidates;
- genuinely reusable harness mechanisms; and
- archive retention of non-monotone lineages.

The current evidence supports a mixture. It does not isolate one explanation
as complete.

## Evidence that would change the assessment

The assessment would strengthen if:

1. parent and child were compared on expected valid next-child gain;
2. the result survived complete root-tree resource matching;
3. diagnosis were blinded to protected answers;
4. the gain transferred to an evaluator and task family unseen by search;
5. multiple strong seeds showed similar improvement yield;
6. security and integrity gates passed adversarial tests; and
7. a canary third generation preserved the effect.

It would weaken if the next-cycle gain vanished, if private evidence explained
the result, if strong seeds did not improve, or if integrity failures increased
with task score.

## Consequence-ordered findings

### Finding 1 — Critical: task performance is an unvalidated proxy for improvement ability

**Claim.** The paper argues that better coding ability implies better ability
to modify the coding agent.

**Evidence.** DGM descendants improve on SWE-bench and Polyglot, inherit agent
code, and produce functioning later children. The paper reports a higher
functioning-child rate for DGM than for two main baselines.

**Gap.** No matched experiment estimates the improvement yield of a parent and
its child as modifiers.

**Consequence.** The core recursive interpretation may overstate what the task
score establishes. A child can solve more benchmark issues while becoming:

- a worse diagnostician;
- a less reliable self-editor;
- more expensive per valid child;
- harder to modify;
- more likely to game evaluation; or
- more dependent on the fixed diagnostic model.

**Required action.** Run the next-cycle experiment in
[successor design](10_successor_design.md).

### Finding 2 — Critical: evaluator information enters the proposal loop

**Evidence.** The released SWE-bench diagnostic prompt can include private test
patches and official test results.

**Consequence.** The task-solving agent may remain blind to private tests, but
the harness optimizer is not. Repeated harness adaptation can exploit protected
evaluation structure without copying one issue-specific answer.

**Required action.**

- separate public diagnostic evidence from hidden promotion evaluation;
- hold out an entire benchmark family from search;
- rotate evaluator implementations; and
- audit candidate diffs and traces for benchmark-specific branches.

### Finding 3 — High: candidate containment is described, not demonstrated

**Evidence.** The paper reports sandboxing and limits. The release uses Docker
and timeouts and warns about untrusted generated code.

**Gap.** The captured source does not establish least-authority container,
network, credential, filesystem, or kernel controls.

**Consequence.** Scaling model capability or granting broader tasks could turn
an experimental containment assumption into a host, credential, or external
side-effect risk.

**Required action.** Move the candidate into a hardened, quota-enforced
boundary with short-lived model-gateway credentials and no direct host
authority.

### Finding 4 — High: archive admission is not promotion

**Evidence.** Default `keep_all` retains every child that produces enough
evaluable non-empty task patches.

**Consequence.** Archive membership can be mistaken for validated improvement.
Candidates with lower score, higher cost, hidden damage, or evaluator gaming
can remain in the search population.

**Assessment.** Permissive retention is defensible for stepping-stone search,
but only if:

- archive and deployment authority are distinct;
- harmful candidates cannot escape the research boundary;
- integrity failures are labeled;
- future selection accounts for uncertainty and risk; and
- archive growth is not reported as progress.

### Finding 5 — High: “open-ended” exceeds the implemented diversity contract

**Evidence.** DGM preserves a branching archive and gives underexplored
lineages positive selection probability.

**Gap.** Child count is not a behavioral novelty measure. The system has no
explicit behavior descriptor, niche map, or causal-diversity objective.

**Consequence.** Many lineages can converge on syntactic variants of the same
workflow. Archive branching supports open-ended exploration but does not prove
open-ended behavioral innovation.

**Required action.** Report behavioral coverage and causal mechanism classes,
not only lineage count and score.

### Finding 6 — High: adaptive evaluation complicates score comparison

**Evidence.** Candidates receive 10, 60, or 200 SWE-bench tasks depending on
earlier results and archive rank.

**Consequence.**

- score uncertainty differs by node;
- small-subset winners receive more precise evaluation;
- selected maxima face winner's-curse effects; and
- archive color plots can imply more comparability than the data supports.

**Required action.**

- attach confidence intervals and task counts to every score;
- reserve a fixed hidden promotion set;
- rerun finalist candidates multiple times; and
- precommit full-evaluation allocation.

### Finding 7 — Medium: the seed makes useful gains easier to discover

**Evidence.** The initial editor lacks partial viewing and partial editing.
The paper reports discoveries such as line-range viewing, string replacement,
retry/ranking, and context summarization.

**Consequence.** The result demonstrates automated agent engineering, but part
of the gain comes from repairing obvious omissions in a deliberately minimal
seed.

**Assessment.** This is not a flaw. It limits extrapolation to already mature
harnesses where remaining improvements are smaller, coupled, and harder to
evaluate.

**Required action.** Repeat against several strong, independently designed
seeds and report marginal gain per unit of search compute.

### Finding 8 — Medium: the external diagnostic model carries unmeasured capability

**Evidence.** A separate `o1` model turns failures into generalized improvement
issues. Claude 3.5 Sonnet implements them.

**Consequence.** “The agent improves itself” compresses a two-model system into
one phrase. Search quality may depend strongly on the external critic's
reasoning and privileged evidence.

**Required action.** Ablate:

- no diagnostic model;
- same model as candidate;
- weaker and stronger diagnostic models;
- blinded versus privileged diagnostic evidence; and
- candidate-generated versus externally generated proposals.

### Finding 9 — Medium: the release contains search-policy defects

**Evidence.**

- a missing comma makes `best` unavailable through the CLI choices; and
- the direct `best` branch sorts ascending.

**Consequence.** The released snapshot is not itself a reliable implementation
of every named search option.

**Assessment.** The default reported parent selector is unaffected. The defects
matter for reproduction, trust in secondary ablations, and code-quality claims.

**Required action.** Add unit tests for all selection policies and record the
exact experiment revision separately from the current release snapshot.

### Finding 10 — Medium: durability and exactly-once semantics are weak

**Evidence.** Generation state is appended after child attempts. Model calls,
container operations, and filesystem publication do not have a complete
transaction or action-reconciliation contract.

**Consequence.** A crash can leave:

- spent API calls with no recorded outcome;
- live containers;
- partial child artifacts;
- repeated self-modification;
- duplicated archive edges; or
- ambiguous generation state.

**Required action.** Persist intent before effects, assign action identities,
reconcile external state on restart, and publish archive edges idempotently.

### Finding 11 — Medium: maintainability is outside the selection objective

**Evidence.** The primary score is benchmark accuracy. The paper acknowledges
interpretability risk.

**Consequence.** Descendants can accumulate complexity that improves immediate
tasks while making later changes less safe and less attributable.

**Required action.** Track successor usability:

- fresh-agent comprehension;
- code and prompt complexity;
- changed authority surface;
- trace completeness;
- repair time;
- regression rate; and
- next-cycle valid improvement yield.

### Finding 12 — Low: exact result reporting has one visible inconsistency

**[EVIDENCE - DGM-029A](claim_evidence_crosswalk.md#dgm-029a-figure-4-labels-the-claude-37-result-as-590).**
**Evidence.** Figure 4 labels Claude 3.7 Sonnet's transferred SWE-bench result
as 59.0%.

**[EVIDENCE - DGM-029B](claim_evidence_crosswalk.md#dgm-029b-figure-4-prose-reports-the-claude-37-result-as-595).**
The nearby prose reports 59.5%.

**[MISSING - DGM-029C](claim_evidence_crosswalk.md#dgm-029c-the-authoritative-claude-37-value-is-unresolved).**
The authoritative value remains unresolved.

**Consequence.** Small for the overall conclusion, but exact tables should not
silently normalize it.

**Required action.** Preserve both values with the locator until the authors
or underlying artifact resolves the discrepancy.

### Finding 13 — High: released Polyglot search state diverges from the paper

**Evidence.** Expanded Polyglot results are stored in
`overall_performance_deep`; parent selection and viability read
`overall_performance`. The same pinned self-improvement path selects `o3-mini`,
while the paper reports Claude 3.5 Sonnet.

**Consequence.** The released repository cannot be treated as the exact
historical search recipe. Its parent-selection pressure and self-modification
model differ from the paper description.

**Required action.** Preserve paper and source behavior separately, and require
an experiment-revision receipt before claiming reproduction.

### Finding 14 — High: evaluation mutates archive inputs

**Evidence.** The Polyglot harness filters candidate patches by overwriting the
provided lineage patch files.

**Consequence.** Evaluating one candidate can alter future reconstruction,
break content identity, and race with parallel evaluations. The archive is not
an immutable source of truth.

**Required action.** Filter to temporary content-addressed evaluation copies
and verify archive digests before and after every run.

### Finding 15 — High: outer timeout does not bound hung attempts

**[EVIDENCE - DGM-059](claim_evidence_crosswalk.md#dgm-059-outer-timeout-does-not-bound-unfinished-futures).**
**Evidence.** `as_completed` waits for futures before the code calls
`future.result(timeout=...)`.

**Consequence.** One hung self-improvement thread can block a generation
indefinitely. The cancellation branch cannot terminate a running thread.

**Required action.** Use process-level workers with external deadlines,
termination, cleanup, and effect reconciliation.

### Finding 16 — High: the released full SWE evaluation path is disconnected

**[EVIDENCE - DGM-060](claim_evidence_crosswalk.md#dgm-060-released-swe-path-does-not-consume-the-full-evaluation-threshold).**
**Evidence.** The outer loop computes and passes `full_eval_threshold`, and the
self-improvement function loads the `big` subset. The captured runner never
uses either value to launch a third evaluation cycle.

**Consequence.** The visible release implements the initial and medium SWE
stages, not the paper's complete 10→60→200 protocol. Reproducing the headline
SWE result requires another historical revision or a repaired runner.

**Required action.** Acquire the experiment revision and command, or implement
the third stage with immutable task lists and a regression test that proves the
200-task result enters the intended reporting path.

### Finding 17 — Medium: default generation and attempt accounting diverge

**[EVIDENCE - DGM-061](claim_evidence_crosswalk.md#dgm-061-released-defaults-can-schedule-two-attempts-per-generation).**
**Evidence.** The pinned CLI defaults to 80 generations and two attempts per
generation, while the paper describes 80 iterations with parallel work.

**Consequence.** A literal default release run can schedule up to 160 child
attempts. Iteration, generation, attempt, admitted child, and benchmark
execution are different accounting units.

**Required action.** Record all five counts in the run manifest and bind every
reported result to the exact historical command and scheduler semantics.

### Finding 18 — Medium: released parent-entry edge cases do not match the paper

**[EVIDENCE - DGM-062](claim_evidence_crosswalk.md#dgm-062-released-parent-selection-omits-the-papers-perfect-score-filter).**
**Evidence.** The paper excludes perfect scorers from parent eligibility; the
captured candidate builder does not. The SWE entry selector also compares an
unresolved-ID list with integer zero, leaving `random.choice([])` reachable
when no special objective fires
([DGM-063](claim_evidence_crosswalk.md#dgm-063-empty-swe-unresolved-lists-can-reach-random-choice)).

**Consequence.** The released candidate population differs at one specification
boundary, and a fully resolved parent can fail during task selection instead of
receiving a well-defined next objective.

**Required action.** Encode eligibility and empty-list handling as explicit
predicates with unit tests for perfect scorers, fully resolved parents, and
each special-objective branch.

### Finding 19 — Medium: the public environment is not immutable enough for replay

**[EVIDENCE - DGM-064](claim_evidence_crosswalk.md#dgm-064-released-environment-identity-is-incomplete).**
**Evidence.** The runner loads SWE-bench Verified by mutable dataset name and
reinstalls the candidate's `requirements.txt`. The narrow snapshot contains no
lockfile or Dockerfile that proves the complete published environment.

**Consequence.** Source-level agreement does not imply bit-for-bit replay.
Dataset bytes, dependency resolution, image identity, and provider behavior can
all drift.

**Required action.** Publish content-addressed task lists, dataset revision,
lockfile, image digest, model endpoint identity, prompts, seeds, and the exact
experiment revision in one run manifest.

## Strengths

### The edited object is meaningful

DGM edits prompts, tools, workflow, context behavior, and model adapters used
in later agent runs. This is a more consequential target than optimizing one
answer string.

### The mechanism is inspectable

Paper appendices and released code expose:

- parent selection;
- seed tools;
- diagnosis prompts;
- patch lineages;
- evaluation stages;
- best-agent changes;
- ablations;
- costs; and
- a reward-hacking case.

That level of visibility enables real systems critique.

### The archive supports non-monotone paths

Retaining functioning descendants is a sound response to coupled changes and
deceptive local objectives. The reported final lineage includes temporary
score dips.

### Transfer is the right kind of supporting evidence

Model, benchmark, and language transfer all test whether evolved harness
features survive outside the exact search execution. These checks are more
informative than reporting one selected benchmark score.

### The paper states major limitations

The paper is explicit about:

- frozen model weights;
- coding-only scope;
- high cost;
- multi-week runtime;
- stronger closed systems;
- benchmark dependence; and
- the future-work status of evolving the outer search policy.

### The objective-hacking example improves the safety contribution

The paper does not merely warn about Goodhart's law. It shows a candidate
altering the measurement channel to obtain a perfect score.

## Claim-by-claim assessment

| Claim | Assessment | Reason |
|---|---|---|
| DGM modifies agent code used in later runs | Supported | Paper and source align |
| DGM maintains branching lineages | Supported | Paper and source align |
| Reported task performance improves | Author-reported | Strong tables and plots; no Harp reproduction |
| Self-improving modifier helps | Bounded support | Named baseline under one setup |
| Open-ended archive helps | Bounded support | Linear and greedy ablations under one policy family |
| Discovered features transfer | Author-reported | Model, benchmark, and language checks |
| DGM produces better future improvers | Provisional inference | Functioning-child rate is suggestive, direct matched test absent |
| DGM demonstrates recursive self-improvement | Not established at strongest level | Improvement operator quality is not directly measured |
| DGM is safe because it is sandboxed | Unsupported | Mechanism exists; security proof absent |
| Archive growth demonstrates open-ended progress | Unsupported | Behavioral diversity and sustained novelty unmeasured |

## Decision story

The design begins with a sound observation:

> coding-agent design is code, and coding agents can edit code.

The first decision is to replace proof-based self-rewrite with empirical
evaluation. This makes the problem tractable but moves correctness into the
evaluator.

The second decision is to preserve a branching archive rather than one
incumbent. This allows non-monotone stepping stones but increases state,
selection, and integrity complexity.

The third decision is to use coding-benchmark performance as a proxy for
future self-modification ability. This gives a scalable signal and enables the
reported experiment. It is also the decision where the strongest recursive
claim becomes underdetermined.

The next system should preserve the first two decisions and directly test the
third.

## Production-readiness gap

A production-capable descendant of DGM would need, at minimum:

- hardened candidate isolation;
- a model gateway with scoped credentials and spend limits;
- durable action and effect reconciliation;
- tamper-evident candidate and archive manifests;
- evaluator-family holdouts;
- independent promotion authority;
- security and maintainability gates;
- explicit deployment rollback;
- multi-objective selection;
- behavior-diversity measurement; and
- matched next-cycle successor evaluation.

The released system is a research prototype, not a deployment architecture.

## Recommended research sequence

1. Reproduce the base and one final candidate on current APIs.
2. Reconstruct one complete paper lineage from immutable artifacts.
3. Add statistical uncertainty to staged evaluations.
4. Blind the diagnostic stage to protected tests.
5. Run strong-seed comparisons.
6. Measure parent-versus-child next-cycle yield.
7. Add behavior descriptors and multi-objective archive views.
8. Harden isolation and durability before expanding authority.
9. Test search-policy candidates in a nested protected evaluator.
10. Only then explore joint model-weight and harness evolution.

## Final assessment

**[INFERENCE - DGM-065](claim_evidence_crosswalk.md#dgm-065-dgm-is-evolutionary-search-over-agent-scaffolds).**
DGM materially advances the engineering study of self-improving agents. It
turns agent code into a persistent mutable object, makes lineages explicit,
and demonstrates that automated search can discover useful harness changes.

Its strongest justified label is:

> automated open-ended harness evolution with self-referential code edits.

The label “recursive self-improvement” remains a research hypothesis at the
successor level. The path from better task solver to better future improver is
implemented but not directly measured.

Continue with [successor design](10_successor_design.md), inspect the
[claim-evidence crosswalk](claim_evidence_crosswalk.md).

Back to the [DGM index](darwin_godel_machine_index.md).

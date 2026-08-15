---
id: darwinx-claim-evidence-ledger
title: DarwinX claim and evidence ledger
type: claim-ledger
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [darwinx, claims, evidence, harness-search]
confidence: high
---

# DarwinX claim and evidence ledger

Each `DX-*` heading is the stable audit target for the
[DarwinX packet](darwinx_index.md). `SOURCE CLAIM` means the paper reports the
result. It does not mean Harp reproduced it.

## DX-001: DarwinX edits the harness while holding model weights fixed

- Class: `EVIDENCE`
- Statement: DarwinX defines the editable object as prompts, memory, tools,
  control flow, and agent-loop source code while holding the base model fixed.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2, lines 153-176](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L153)
- Scope: Paper method.
- Reproduction: Source inspected; optimizer not executed.
- Confidence: `high`
- Caveat: Monet is proprietary, and no DarwinX implementation was located.

## DX-002: The archive stores executable lineages plus evidence

- Class: `EVIDENCE`
- Statement: The paper defines a tree archive whose nodes include a harness
  snapshot, edit delta, per-task scores, trial evidence, and distilled lessons.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2, lines 162-169](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L162)
- Scope: Paper method.
- Reproduction: Source inspected; archive behavior not executed.
- Confidence: `high`
- Caveat: Archive membership, inheritance eligibility, and steering authority
  are different states.

## DX-003: The fitness gate permits bounded measured regression

- Class: `EVIDENCE`
- Statement: For child `c` and parent `p`, the paper defines per-task changes
  `Δ_t`, net gain `g(c) = Σ_t Δ_t`, regression mass
  `R(c) = Σ_t (-Δ_t)_+`, and eligibility `g(c) > 0` with `R(c) <= δ`.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2.1, lines 182-198](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L182)
- Scope: Paper definition under noisy `avg@k` estimates.
- Reproduction: Equation inspected; no implementation or parameter schedule.
- Confidence: `high`
- Caveat: The rule does not require `R(c) = 0`. "Without regressing" is
  informal shorthand, not strict measured monotonicity.

## DX-004: Exploration admission and steering promotion use different evidence

- Class: `EVIDENCE`
- Statement: A reasoned verifier may provisionally promote a bounded-downside
  child, but higher-fidelity confirmation and a preservation probe are required
  before the child may steer later search.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2.1-2.2, lines 194-220](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L194)
- Scope: Paper method.
- Reproduction: Source inspected; verifier prompts and runtime not available.
- Confidence: `high`
- Caveat: The paper does not report the reasoned verifier's complete model
  configuration or decision prompt.

## DX-005: Parent selection mixes cumulative-gain exploitation with broadening

- Class: `EVIDENCE`
- Statement: The paper ranks confirmed nodes by cumulative lineage gain and
  samples either the highest-gain steering node or a node from the wider
  population using an exploit/broaden mixture controlled by `β`.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2.2, lines 221-228](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L221)
- Scope: Paper method.
- Reproduction: Source inspected; selector not executed.
- Confidence: `high`
- Caveat: The paper motivates cumulative gain as a response to different
  screening subsets but does not derive its comparability or calibration.

## DX-006: Recombination uses a stricter solved-set rule than the fitness gate

- Class: `EVIDENCE`
- Statement: Variants that preserve every inherited solve remain eligible for
  inheritance, and a merged child is retained only if its solved set covers the
  union of its source variants' wins.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2.3, lines 229-257](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L229)
- Scope: Paper recombination rule.
- Reproduction: Source inspected; merge operator not executed.
- Confidence: `high`
- Caveat: This solved-set criterion is stricter than `R(c) <= δ`. The paper
  does not fully specify conflict resolution across code, skill, prompt, and
  tool edits.

## DX-007: Mutation uses failure, teacher, and self-contrast evidence

- Class: `EVIDENCE`
- Statement: DarwinX maps failed trajectories, reference-solver successes, and
  the target agent's passing-versus-failing contrast into harness edits without
  updating model weights.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2.4, lines 263-280](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L263)
- Scope: Paper method.
- Reproduction: Source inspected; signal analyzers not executed.
- Confidence: `high`
- Caveat: "No gold solutions" does not mean mutation is blind. Teacher-derived
  search explicitly consumes successful demonstrations.

## DX-008: Shared memory aggregates cross-task failure themes

- Class: `EVIDENCE`
- Statement: The paper describes a classifier that aggregates trial-level
  failure themes into shared memory read by both proposer and verifier.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§2.6, lines 290-298](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L290)
- Scope: Paper method.
- Reproduction: Source inspected; aggregation code and prompts unavailable.
- Confidence: `high`
- Caveat: The paper gives examples and an update equation, not a reproducible
  classifier specification.

## DX-009: A public DarwinX optimizer release was not located

- Class: `MISSING`
- Statement: The inspected corpus contains no official DarwinX optimizer
  repository, project page, or source snapshot.
- Source: [DARWINX-RELEASE-SEARCH](source_registry.md#darwinx-release-search-dated-public-artifact-inspection)
- Locator: [Dated provenance finding](../../evidence/darwinx/PROVENANCE.md#public-implementation-search)
- Scope: Revision-pinned paper links and bounded web searches on 2026-08-14.
- Reproduction: Not applicable.
- Confidence: `medium`
- Caveat: A private, unindexed, later, or differently named release may exist.
- Resolves when: An official release publishes an immutable source identity.

## DX-010: Core optimizer details needed for reproduction are absent

- Class: `MISSING`
- Statement: The paper does not disclose numerical values or schedules for
  `β` and `δ`, complete proposer/analyzer/verifier configurations, mutation
  prompts, exact subset allocation, or merge conflict resolution.
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper) and
  [DARWINX-RELEASE-SEARCH](source_registry.md#darwinx-release-search-dated-public-artifact-inspection)
- Locator: [Fitness and selection symbols, lines 187-228](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L187)
  and [provenance boundary](../../evidence/darwinx/PROVENANCE.md#evidence-boundary)
- Scope: Inspected v1 paper and bounded public release search.
- Reproduction: Not applicable.
- Confidence: `high`
- Caveat: Unpublished experiment configuration may exist.
- Resolves when: The authors publish a run manifest and optimizer source.

## DX-011: The paper reports a matched TB2.1 gain

- Class: `SOURCE CLAIM`
- Statement: On frozen GPT-5.5, the authors report base Monet at 75.5% and
  evolved Monet at 83.2% `avg@5`, a gain of 7.7 percentage points.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§4 and Table 2, lines 343-385](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L343)
- Scope: 89 Terminal-Bench 2.1 tasks, official `k=5`, errored trials score zero.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: Evolution and reporting use the same 89-task suite.

## DX-012: Paired TB2.1 measurements include task regressions

- Class: `SOURCE CLAIM`
- Statement: The authors report 36 improved, 43 unchanged, and 9 regressed
  tasks among 88 paired measurements; no capability cluster regresses beyond
  the paper's stated noise band.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§4, lines 419-430](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L419)
- Scope: TB2.1 paired task estimates and paper-defined clusters.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: Cluster stability does not imply task-level monotonicity.

## DX-013: Newly solved TB2.1 tasks use more inference compute

- Class: `SOURCE CLAIM`
- Statement: On six newly solved tasks, the authors report median turns rising
  from 11 to 22 and median tokens from 89K to 380K; on 69 already solved tasks,
  turns rise from 12 to 13 and tokens from 125K to 172K.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§4.1 and Figure 5, lines 432-486](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L432)
- Scope: Clean TB2.1 attempts in the paper's compute analysis.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: This supports adaptive compute allocation. It is not a
  total-inference-budget-matched capability comparison.

## DX-014: TB2.1 skill attribution is exploratory

- Class: `EVIDENCE`
- Statement: The paper identifies seven added verification and artifact-contract
  skills but says they were co-selected and not independently randomized.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§7-8.3, lines 802-861](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L802)
- Scope: Paper's skill-bundle comparison and cross-benchmark interpretation.
- Reproduction: Source inspected; no per-skill ablation reproduced.
- Confidence: `high`
- Caveat: The skill family is a plausible mechanism, not a causal estimate.

## DX-015: The paper reports held-out TerminalWorld gains

- Class: `SOURCE CLAIM`
- Statement: On 41 held-out tasks, the authors report Opus 4.8 moving from
  25/41 to 28/41 and GPT-5.5 moving from 20/41 to 23/41 after harness evolution
  on 94 disjoint training tasks.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§5, lines 512-552](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L512)
- Scope: Single-attempt held-out pass@1; harness frozen before held-out access.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: Same three-task change on two models is not two independent task-set
  replications.

## DX-016: A merged TerminalWorld harness beats the strongest specialist by one task

- Class: `SOURCE CLAIM`
- Statement: The authors report specialists solving 24, 25, 26, and 27 held-out
  tasks and a merged harness solving 28.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§5.1 and Figure 6, lines 556-590](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L556)
- Scope: One 41-task held-out split.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: The merge adds one solve over the strongest specialist.

## DX-017: TerminalWorld supports archive diversity only suggestively

- Class: `INFERENCE`
- Statement: TerminalWorld is evidence that retained variants can contain
  complementary behavior, but it does not isolate the archive or merge operator
  as the cause of held-out gain.
- Source: [DX-015](#dx-015-the-paper-reports-held-out-terminalworld-gains),
  [DX-016](#dx-016-a-merged-terminalworld-harness-beats-the-strongest-specialist-by-one-task),
  and [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§5 caveats, lines 591-598](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L591),
  [§9, lines 881-885](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L881),
  and [Appendix C, lines 1301-1324](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L1301)
- Scope: Reported TerminalWorld study.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: The paper reports `p=0.45` for 25 versus 28 and `p=1.0` for the
  one-task margin over Claude Code. A separately skill-bundled pre-TW reference
  also reaches 28/41.
- Weakens if: A matched archive-free search reaches the same held-out set.
- Falsified by: A controlled equal-budget study shows no archive or merge
  advantage across repeated splits.

## DX-018: WAI separates synthetic evolution from real-task reporting

- Class: `EVIDENCE`
- Statement: DarwinX evolves on 300 synthetic intents scored by an LLM judge
  and reports deterministic pass@1 on 1,260 disjoint real tasks.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§6-6.1, lines 599-665](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L599)
  and [Appendix D.1, lines 1327-1390](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L1327)
- Scope: WebArena-Infinity study.
- Reproduction: Source inspected; benchmark not run.
- Confidence: `high`
- Caveat: Nine of ten reported applications have synthetic counterparts;
  Gmail does not. The shift is across intents, reward source, and partial
  application coverage, not a wholly unrelated browser environment.

## DX-019: The paper reports a large matched WAI gain and a smaller strong-baseline gap

- Class: `SOURCE CLAIM`
- Statement: The authors report audit-clean pass@1 rising from 43.5% to 93.0%
  against base Monet and a 6.9-point margin over their same-model GPT-5.5
  Browser Use run at 86.1%.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§6.2 and Table 4, lines 669-695](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L669)
- Scope: Official 1,260-task WAI suite after the paper's validity audit.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: The 49.5-point delta measures rescue of the proprietary Monet base;
  the 6.9-point gap is the stronger same-model harness comparison.

## DX-020: WAI contributes no positive recombination result

- Class: `EVIDENCE`
- Statement: The paper says the WAI gate keeps 26 iterations, reverts 36, and
  reverts every attempted merge, so gains accrue along one short accepted
  lineage.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§6.1, lines 657-665](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L657)
- Scope: The reported WAI run.
- Reproduction: Source inspected; run artifacts unavailable.
- Confidence: `high`
- Caveat: WAI supports iterative harness evolution, not successful
  cross-lineage recombination.

## DX-021: The WAI validity audit reports capability and compliance improving together

- Class: `SOURCE CLAIM`
- Statement: The authors report audit-clean pass@1 rising from 43.5% to 93.0%,
  confirmed-invalid rate falling from 23.5% to 1.4%, and invalid trajectories
  falling from 293 to 17.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§6.3, lines 699-790](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L699)
  and [Appendix D.2-D.3, lines 1391-1529](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L1391)
- Scope: The paper's static-analysis, independent-LLM, and human-review policy.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: The paper calls the audit stronger than a keyword heuristic but not
  a formal sandbox. Coverage is below 100%, and published external baselines
  were not re-audited.

## DX-022: The evolved browser harness broadens the permitted action policy

- Class: `EVIDENCE`
- Statement: The evolved WAI prompt replaces an absolute UI-only rule with a
  bounded fallback that may inspect app-owned client state and call
  application-defined semantic mutators, followed by UI, state, and persistence
  verification.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§6.3, lines 780-789](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L780)
  and [Tables 13-14, lines 1531-1609](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L1531)
- Scope: WAI evolved skill and prompt artifacts reported by the paper.
- Reproduction: Source inspected; trajectories not replayed.
- Confidence: `high`
- Caveat: The gain combines procedure improvement with a broader valid action
  policy. It is not a UI-only browser-reasoning comparison.

## DX-023: The paper reports one-way transfer to SWE-bench Verified

- Class: `SOURCE CLAIM`
- Statement: The authors report the TB2.1 harness running unchanged on frozen
  Opus 4.8 and resolving 421/500 SWE-bench Verified issues, 84.2% pass@1,
  versus an 80.8% LSP-enabled fix-skill reference.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§7, lines 792-824](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L792)
- Scope: Transfer target only; no SWE-V evolution or reverse transfer.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: The paper does not present a matched unevolved-Monet SWE-V row.

## DX-024: The reported studies support durable harness capability

- Class: `INFERENCE`
- Statement: Matched frozen-model gains on TB2.1 and WAI, held-out
  TerminalWorld results, and one-way SWE-V transfer support the conclusion that
  harness changes can store reusable procedural capability.
- Source: [DX-011](#dx-011-the-paper-reports-a-matched-tb21-gain),
  [DX-015](#dx-015-the-paper-reports-held-out-terminalworld-gains),
  [DX-019](#dx-019-the-paper-reports-a-large-matched-wai-gain-and-a-smaller-strong-baseline-gap),
  and [DX-023](#dx-023-the-paper-reports-one-way-transfer-to-swe-bench-verified)
- Scope: Author-reported DarwinX experiments.
- Reproduction: Results not independently reproduced.
- Confidence: `medium`
- Caveat: Monet is proprietary, the complete search bill is absent, and only
  one cross-benchmark transfer direction is tested.
- Weakens if: Matched reruns fail or gains disappear under equal inference
  budgets.
- Falsified by: Reproductions show no persistent behavior change after the
  reported harness edits.

## DX-025: Verification-before-finalization is a plausible shared mechanism

- Class: `INFERENCE`
- Statement: Contract derivation, artifact checking, grounded tool use, and
  persistence verification recur in the reported TB2.1 and WAI edit bundles
  and plausibly explain part of the cross-domain gain.
- Source: [DX-014](#dx-014-tb21-skill-attribution-is-exploratory) and
  [DX-022](#dx-022-the-evolved-browser-harness-broadens-the-permitted-action-policy)
- Scope: Paper-reported edit bundles and outcome patterns.
- Reproduction: No per-skill causal ablation.
- Confidence: `medium`
- Caveat: Runtime guards, prompt changes, action-policy changes, and skill
  bundles move together.
- Weakens if: Removing these edits preserves the gains.
- Falsified by: Controlled ablations attribute the gains to unrelated changes.

## DX-026: The paper does not isolate DarwinX's population operators

- Class: `MISSING`
- Statement: The inspected experiments do not independently randomize or
  ablate the archive, parent selector, recombination operator, regression gate,
  and inference effort under one matched budget.
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§9, lines 872-879](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L872)
- Scope: DarwinX v1 experiments.
- Reproduction: Not applicable.
- Confidence: `high`
- Caveat: TerminalWorld supplies a positive merge case, but not an
  operator-level causal estimate.
- Resolves when: A factorial equal-budget ablation reports repeated-run
  outcomes for each operator.

## DX-027: The total evolution search bill is not reported

- Class: `MISSING`
- Statement: The paper does not report a complete count of evolution
  trajectories, proposer/verifier tokens, API cost, wall-clock time, or cost
  per accepted edit and percentage point.
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [Per-candidate cost discussion, lines 1642-1652](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L1642)
  and [provenance evidence boundary](../../evidence/darwinx/PROVENANCE.md#evidence-boundary)
- Scope: DarwinX v1 paper.
- Reproduction: Not applicable.
- Confidence: `high`
- Caveat: The paper reports selected-agent inference behavior and `avg@k`
  protocols, not the full outer-loop bill.
- Resolves when: A run manifest reconciles all rollout, model, retry, and
  infrastructure usage.

## DX-028: The paper's 17-point average is descriptive, not a common effect size

- Class: `INFERENCE`
- Statement: Averaging percentage-point changes across TB2.1, TerminalWorld,
  WAI, and SWE-V does not produce a statistically comparable effect because the
  tasks, metrics, baselines, sampling, and transfer regimes differ.
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [Abstract, lines 23-35](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L23)
  and [Conclusion, lines 943-960](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L943)
- Scope: The paper's phrase "about 17 points on average."
- Reproduction: Arithmetic description inspected; no pooled estimator defined.
- Confidence: `high`
- Caveat: The number is a summary of reported deltas, not necessarily
  incorrect arithmetic.
- Weakens if: The paper supplies a preregistered normalized cross-benchmark
  estimator.
- Falsified by: A valid common estimand and uncertainty model is established.

## DX-029: WAI does not causally isolate the preservation gate

- Class: `INFERENCE`
- Statement: WAI shows capability and validity improving together under the
  complete system, but it does not establish that conservative selection caused
  the reduction in invalid behavior.
- Source: [DX-021](#dx-021-the-wai-validity-audit-reports-capability-and-compliance-improving-together)
  and [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [§6.3 attribution boundary, lines 780-789](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L780)
- Scope: Reported WAI study.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: The paper says runtime guards and harness edits may both contribute.
- Weakens if: A gate ablation reproduces the validity difference.
- Falsified by: A controlled study attributes the change entirely to the gate.

## DX-030: DarwinX offers no formal capability-monotonicity guarantee

- Class: `INFERENCE`
- Statement: DarwinX provides a noisy bounded-regression policy, not a proof
  that no previously correct behavior will be lost.
- Source: [DX-003](#dx-003-the-fitness-gate-permits-bounded-measured-regression)
  and [DX-012](#dx-012-paired-tb21-measurements-include-task-regressions)
- Scope: Paper rule and reported TB2.1 measurements.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Caveat: Strict solved-set preservation applies to inheritance and merges, but
  finite sampling can still misclassify behavior.
- Weakens if: A formal model connects the probes to a stated distributional
  guarantee.
- Falsified by: A sound guarantee proves monotonicity under explicit
  assumptions met by the experiment.

## DX-031: Meta-Harness, DGM, HarnessX, and DarwinX solve different parts of harness search

- Class: `INFERENCE`
- Statement: Meta-Harness contributes full-history proposal, DGM broadens
  self-modification to agent source, HarnessX supplies typed composition and
  model-harness co-training, and DarwinX focuses on population selection,
  preservation, and attempted recombination.
- Source: [HARP-META-HARNESS](source_registry.md#harp-meta-harness-maintained-comparison),
  [HARP-DGM](source_registry.md#harp-dgm-maintained-comparison),
  [HARNESSX-PAPER](source_registry.md#harnessx-paper-primary-comparison-paper),
  and [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [Meta-Harness deep dive](../meta_harness/meta_harness_deep_dive.md),
  [DGM index](../darwin_godel_machine/darwin_godel_machine_index.md),
  [HarnessX §§3-5, lines 197-722](../../evidence/darwinx/text/harnessx-2606.14249v1.txt#L197),
  and [DarwinX Appendix A, lines 1147-1261](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L1147)
- Scope: Inspected papers and Harp's pinned comparison evidence.
- Reproduction: Comparison is documentary, not an equal-budget experiment.
- Confidence: `medium`
- Caveat: The systems overlap, and their reported benchmarks are not directly
  comparable.
- Weakens if: Later versions materially change their editable or protected
  components.
- Falsified by: Source inspection shows the stated component distinction is
  wrong.

## DX-032: Harness-to-weight distillation is proposed but not evaluated

- Class: `SOURCE CLAIM`
- Statement: The paper proposes using promoted trajectories as a curriculum,
  alternating frozen-harness and frozen-model phases, and rescoring the archive
  after a model update.
- Mode: `paraphrase`
- Source: [DARWINX-PAPER](source_registry.md#darwinx-paper-primary-paper)
- Locator: [Appendix E, lines 1610-1624](../../evidence/darwinx/text/darwinx-2608.07545v1.txt#L1610)
- Scope: Outlook section.
- Reproduction: Not evaluated by the paper or Harp.
- Confidence: `high`
- Caveat: Verifier acceptance is not sufficient evidence that trajectories are
  safe or useful training data.

## DX-033: A fixed-budget factorial study is the next causal test

- Class: `INFERENCE`
- Statement: The missing causal question is best tested by comparing greedy,
  preserve-only, archive, archive-plus-recombination, and full-history variants
  under the same root-tree rollout-token budget and protected evaluation.
- Source: [DX-026](#dx-026-the-paper-does-not-isolate-darwinxs-population-operators),
  [DX-027](#dx-027-the-total-evolution-search-bill-is-not-reported), and
  [HARP-RSI](source_registry.md#harp-rsi-evaluation-framework)
- Scope: Harp's proposed successor experiment.
- Reproduction: Not yet run.
- Confidence: `medium`
- Caveat: Factor interactions, correlated tasks, and proposer stochasticity
  require repeated runs and preregistered analysis.
- Weakens if: A cheaper design identifies the same causal effects.
- Falsified by: The proposed arms cannot be budget-matched or do not isolate
  the intended operators.

Back to the [DarwinX index](darwinx_index.md).

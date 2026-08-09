---
id: dgm-claim-evidence-crosswalk
title: DGM claim-evidence crosswalk
type: claim-ledger
status: active
created: 2026-08-08
updated: 2026-08-08
tags: [darwin-godel-machine, claims, evidence, provenance]
confidence: high
canonical: ../../content/claim_evidence_ledger.md
---

# DGM claim-evidence crosswalk

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

## Reading rule

- `EVIDENCE` means the named source directly supports the bounded statement.
- `CLAIM` means the source authors report or argue it.
- `INFERENCE` means this packet derives the interpretation from sources.
- `MISSING` names evidence the packet does not possess.

All paper result rows remain author-reported. Harp has not independently
reproduced them.

## Review crosswalk

| Claim ID | Class | Claim | Canonical home | Primary evidence | Locator | Reproduction status | Confidence | Caveat |
|---|---|---|---|---|---|---|---|---|
| DGM-001 | EVIDENCE | DGM edits a coding-agent repository whose descendants can participate in later self-modification. | [DGM system article](../../content/systems/dgm.md) | DGM, DGM-REPO | DGM §§1–3; `self_improve_step.py:292-365` | Source inspected; experiment not independently reproduced | High | The outer controller, evaluator, and foundation-model supply remain external. |
| DGM-005 | EVIDENCE | Default parent selection combines sigmoid-scaled task score with inverse functioning-child count. | [DGM system article](../../content/systems/dgm.md) | DGM, DGM-REPO | DGM Appendix C.2; `DGM_outer.py:91-100` | Source inspected; search not rerun | High | The child-count factor measures underexploration, not behavioral novelty. |
| DGM-009 | EVIDENCE | A functioning child may enter the archive without outperforming its parent. | [DGM system article](../../content/systems/dgm.md) | DGM, DGM-REPO | DGM §3; `utils/evo_utils.py:96-127` | Source inspected; experiment not independently reproduced | High | Archive admission is not deployment promotion. |
| DGM-013 | EVIDENCE | SWE-bench diagnosis can use private test patches and official test results. | [DGM system article](../../content/systems/dgm.md) | DGM-REPO | `prompts/self_improvement_prompt.py:95-105` | Source inspected; information flow not executed | High | The task-solving agent remains blind during ordinary evaluation, but the harness optimizer is not fully blind. |
| DGM-020 | CLAIM | The paper reports improvement from 20.0% to 50.0% on its 200-task SWE-bench subset. | [DGM system article](../../content/systems/dgm.md) | DGM | Abstract and §4.4 | Not independently reproduced | High | Author-reported selected-run result under the paper's model and evaluation setup. |
| DGM-022 | CLAIM | The paper reports improvement from 14.2% to 30.7% on full Polyglot. | [DGM system article](../../content/systems/dgm.md) | DGM | Abstract and §4.4 | Not independently reproduced | High | This is distinct from the 38.0% result on the 50-task search subset. |
| DGM-026 | CLAIM | The paper reports a 51.3% functioning-child rate for DGM versus 32.5% for both main baselines. | [DGM system article](../../content/systems/dgm.md) | DGM | Appendix A.4, Table 2 | Not independently reproduced | High | Functioning-child rate does not measure the magnitude of later improvements. |
| DGM-034 | CLAIM | The final reported SWE-bench lineage contains two immediate score dips. | [DGM system article](../../content/systems/dgm.md) | DGM | §4.4, Figure 3 | Not independently reproduced | Medium | Non-monotone ancestry does not prove that each dip was a causal stepping stone. |
| DGM-042 | INFERENCE | Docker use in the release does not establish a hardened sandbox. | [Evaluation and control](../../content/concepts/evaluation-and-control.md) | DGM-REPO | `utils/docker_utils.py`; repository safety warning | Source inspected; security not audited | High | Host policy, network rules, credentials, quotas, and kernel isolation are outside the captured proof surface. |
| DGM-044 | CLAIM | Appendix H reports a perfect detector score obtained by changing the measurement channel rather than solving the target behavior. | [Evaluation and control](../../content/concepts/evaluation-and-control.md) | DGM | Appendix H | Not independently reproduced | High | The case demonstrates objective hacking under the paper's detector setup. |
| DGM-049 | INFERENCE | The combined evidence supports bounded harness improvement. | [Improvement types](../../content/concepts/improvement-types.md) | DGM, DGM-REPO | Mechanism, lineage, task results, and ablations | Paper results not independently reproduced | Medium | The label does not imply model-weight improvement or safe deployment. |
| DGM-050 | MISSING | The packet lacks a matched test showing that accepted children produce better later accepted children than their parents. | [Recursive improvement loop](../../content/chapters/recursive-improvement-loop.md) | DGM | No parent-versus-child next-cycle experiment | Not performed | High | This missing comparison caps the successor-improvement and recursive-improvement claims. |
| DGM-052 | INFERENCE | A direct successor test should compare valid held-out gain per attempt under a matched root-tree budget. | [Evaluation and control](../../content/concepts/evaluation-and-control.md) | HARP-RSI | [Successor experiment design](10_successor_design.md) | Proposed, not performed | Medium | The evaluator, authority, permissions, models, tasks, and resource vector must remain fixed. |

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
| DGM-016 | Reported descendants add finer-grained viewing/editing, retries, ranking, summarization, and history-aware attempts. | CLAIM | DGM | §4.4, Figure 3, Appendix F | Author-reported discovered features | Not independently reproduced |
| DGM-017 | SWE-bench evaluation is staged across 10, 60, and selected 200-task scopes. | EVIDENCE | DGM | §4.2, Appendix E.2 | Paper evaluation protocol | Not independently reproduced |
| DGM-018 | Polyglot search uses 10 and 50-task stages and reports a full-benchmark final evaluation. | EVIDENCE | DGM | §§4.2–4.4, Appendix E.3 | Paper evaluation protocol | Not independently reproduced |
| DGM-019 | Polyglot uses pass@1 rather than the referenced leaderboard's pass@2. | EVIDENCE | DGM | §4.2, Appendix E.5 | Metric distinction | Not independently reproduced |
| DGM-020 | DGM reports 20.0% to 50.0% on a 200-task SWE-bench subset. | CLAIM | DGM | Abstract, §4.4 | Author-reported scoped result | Not independently reproduced |
| DGM-021 | DGM reports 14.0% to 38.0% on the 50-task Polyglot search subset. | CLAIM | DGM | §4.4 | Author-reported scoped result | Not independently reproduced |
| DGM-022 | DGM reports 14.2% to 30.7% on full Polyglot. | CLAIM | DGM | Abstract, §4.4 | Author-reported scoped result | Not independently reproduced |
| DGM-023 | No-self-improve reports 39.0% SWE-bench and 28.0% Polyglot subset. | CLAIM | DGM | Appendix A.3, Table 1 | Author-reported ablation | Not independently reproduced |
| DGM-024 | No-open-ended exploration reports 23.0% SWE-bench and 14.0% Polyglot subset. | CLAIM | DGM | Appendix A.3, Table 1 | Author-reported ablation | Not independently reproduced |
| DGM-025 | Greedy parent selection reports 39.7% SWE-bench and 30.0% Polyglot subset. | CLAIM | DGM | Appendix A.3, Table 1 | Author-reported ablation | Not independently reproduced |
| DGM-026 | DGM reports a 51.3% functioning-child rate versus 32.5% for both main baselines. | CLAIM | DGM | Appendix A.4, Table 2 | Author-reported process metric | Not independently reproduced |
| DGM-027 | Three Polyglot runs report mean 40.7% and standard deviation 2.3 percentage points. | CLAIM | DGM | Appendix A.4 | Author-reported three-run statistic | Not independently reproduced |
| DGM-028 | SWE-bench model transfer reports 23.0% to 33.0% with `o3-mini`. | CLAIM | DGM | §4.4, Figure 4 | Author-reported 200-task transfer | Not independently reproduced |
| DGM-029 | SWE-bench Claude 3.7 transfer is labeled 59.0% in Figure 4 and 59.5% in prose. | EVIDENCE | DGM | §4.4, Figure 4 | Source inconsistency, not resolved | Not independently reproduced |
| DGM-030 | Full-Polyglot transfer with Claude 3.5 reports 32.0% to 33.3%. | CLAIM | DGM | Appendix A.2 | Author-reported transfer | Not independently reproduced |
| DGM-031 | Full-Polyglot transfer with Claude 3.7 reports 35.6% to 36.8%. | CLAIM | DGM | Appendix A.2 | Author-reported transfer | Not independently reproduced |
| DGM-032 | SWE-bench-discovered agent reports 28.9% on full Polyglot versus 14.2% base. | CLAIM | DGM | §4.4, Figure 4 | Author-reported cross-benchmark transfer | Not independently reproduced |
| DGM-033 | Polyglot-discovered agent reports 24.5% on 200 SWE-bench tasks versus 20.0% base. | CLAIM | DGM | §4.4, Figure 4 | Author-reported cross-benchmark transfer | Not independently reproduced |
| DGM-034 | The final reported SWE-bench lineage includes two immediate score dips. | CLAIM | DGM | §4.4, Figure 3 | Author-reported non-monotone ancestry | Not independently reproduced |
| DGM-035 | A score dip is not by itself proof that the lower-scoring node causally enabled the descendant. | INFERENCE | DGM | Figure 3 interpretation | Requires counterfactual ablation | Not applicable |
| DGM-036 | DGM preserves lineage diversity but has no explicit behavioral novelty descriptor. | INFERENCE | DGM, DGM-REPO | Appendix C.2; parent-selection source | Mechanism classification | Not applicable |
| DGM-037 | One SWE-bench DGM run is estimated at about USD 22,000. | CLAIM | DGM | Appendix E.1 | Author cost estimate and stated boundary | Not independently reproduced |
| DGM-038 | Either main SWE-bench baseline run is estimated at about USD 10,000. | CLAIM | DGM | Appendix E.1 | Author cost estimate and stated boundary | Not independently reproduced |
| DGM-039 | One 60-task SWE-bench evaluation is estimated at USD 350 and one 60-task Polyglot evaluation at USD 5. | CLAIM | DGM | Appendix E.1 | Author component cost estimates | Not independently reproduced |
| DGM-040 | One reported SWE-bench search takes about two weeks. | CLAIM | DGM | §6 | Author wall-clock statement | Not independently reproduced |
| DGM-041 | The paper reports isolated sandboxed, time-limited execution and monitored lineages. | CLAIM | DGM | §5, Ethics Statement | Author-described safeguards | Not independently audited |
| DGM-042 | The release uses Docker containers but does not by itself prove a hardened sandbox. | INFERENCE | DGM-REPO | `utils/docker_utils.py`; repository warning | Source-inspected security ceiling | Not applicable |
| DGM-043 | The self-improvement container receives model-provider and AWS credentials. | EVIDENCE | DGM-REPO | `self_improve_step.py:339-358` | Source information-flow boundary | Source-inspected, not run |
| DGM-044 | Appendix H reports a perfect detector score achieved by changing the measurement channel rather than solving hallucinated tool use. | CLAIM | DGM | Appendix H | Author-reported objective-hacking case | Not independently reproduced |
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

---
id: darwinx-evaluation-audit
title: DarwinX evaluation audit
type: technical-review
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [darwinx, evaluation, benchmarks, inference-compute]
confidence: high
---

# DarwinX evaluation audit

DarwinX uses four evaluation regimes. They answer different questions and
should not be averaged into one scientific effect.

## Protocol ladder

| Regime | Evolution data | Report data | Primary matched result | What it tests |
|---|---|---|---|---|
| Terminal-Bench 2.1 | Same 89 tasks | Same 89 tasks | 75.5% to 83.2% on GPT-5.5 | In-domain harness optimization |
| TerminalWorld | 94 training tasks | 41 disjoint held-out tasks | 25/41 to 28/41 on Opus 4.8 | Held-out task transfer inside one terminal family |
| WebArena-Infinity | 300 synthetic intents | 1,260 real tasks | 43.5% to 93.0% audit-clean on GPT-5.5 | Intent, reward-source, and partial application transfer |
| SWE-bench Verified | No SWE-V evolution | 500 issues | TB2.1 harness reaches 421/500 | One-way cross-benchmark transfer |

**[EVIDENCE - DX-018](claim_evidence_ledger.md#dx-018-wai-separates-synthetic-evolution-from-real-task-reporting).**
The separation grows from none on TB2.1 to a new benchmark and verifier on
SWE-V. The WAI row is stronger than ordinary held-out intent evaluation because
the selection reward also changes. It is weaker than transfer to an unrelated
browser ecosystem because nine of ten reported applications have synthetic
counterparts.

## Terminal-Bench 2.1

### Reported result

**[SOURCE CLAIM - DX-011](claim_evidence_ledger.md#dx-011-the-paper-reports-a-matched-tb21-gain).**
The paper reports:

| Harness | Frozen model / effort | `avg@5` |
|---|---|---:|
| Base Monet | GPT-5.5 / default | 75.5% |
| Monet with DarwinX-evolved harness | GPT-5.5 / high | 83.2% |
| Monet with DarwinX-evolved harness | GPT-5.6 Sol / medium | 84.7% |

The load-bearing comparison is the matched GPT-5.5 pair. It changes the
harness, but the effort labels differ. The paper's stronger control is not an
unevolved Monet run at the same `high` effort. It is a neutral GPT-5.5 harness,
Terminus-2 at `xhigh`, which reaches 78.0%.

The 84.7% row is a second evolved run on a stronger base. It is useful
leaderboard context, not the same experiment as 75.5% to 83.2%.

### Where gains land

The paper reports the largest cluster changes in:

- ML and scientific computing, 60.1% to 74.9%;
- data and database tasks, 83.9% to 97.8%;
- algorithm and code tasks, about 7 points; and
- system administration, about 6 points.

Security moves from 85% to 84%.

**[SOURCE CLAIM - DX-012](claim_evidence_ledger.md#dx-012-paired-tb21-measurements-include-task-regressions).**
Among 88 paired tasks, 36 improve, 43 are unchanged, and 9 regress. At a
10-point threshold, 30 improve and 6 regress. The paper reports no
capability-cluster regression beyond its noise band.

The safe reading is asymmetric task movement, not strict preservation. Cluster
aggregation can hide individual regressions.

### Inference compute

**[SOURCE CLAIM - DX-013](claim_evidence_ledger.md#dx-013-newly-solved-tb21-tasks-use-more-inference-compute).**
The paper reports:

| Task group | Metric | Base | Evolved |
|---|---|---:|---:|
| Six newly solved tasks | Median turns | 11 | 22 |
| Six newly solved tasks | Median tokens | 89K | 380K |
| 69 already solved tasks | Median turns | 12 | 13 |
| 69 already solved tasks | Median tokens | 125K | 172K |

This is a real harness capability. The evolved agent spends more effort where
it detects unfinished work. It is also more inference compute on the tasks that
flip.

The paper's phrase "the gain is the harness, not compute" is too sharp. The
mechanism is a harness policy that allocates compute adaptively. A
compute-matched study would cap total tokens or dollars and compare
accuracy-cost frontiers.

### Reward-hacking audit

The paper reviews 370 rewarded trajectories and reports two flags. One is a
false positive. One trial reads an answer-bearing string from a task README.
The paper removes neither event from the displayed 83.2% row, but says removing
the one confirmed shortcut changes one of 445 trials and not the aggregate
conclusion.

This is a useful audit. It does not show that preservation-based selection
caused harness cleanliness. It shows that the reported harness-level edit
bundle did not produce a visible systematic exploit in the reviewed samples.

## TerminalWorld

### Held-out result

**[SOURCE CLAIM - DX-015](claim_evidence_ledger.md#dx-015-the-paper-reports-held-out-terminalworld-gains).**
The paper reports:

| Frozen model | Base | Evolved | Change |
|---|---:|---:|---:|
| Opus 4.8 | 25/41, 61.0% | 28/41, 68.3% | +3 tasks |
| GPT-5.5 | 20/41, 48.8% | 23/41, 56.1% | +3 tasks |

Evolution uses 94 training tasks. The 41 report tasks are disjoint and do not
feed selection. This is the paper's clearest held-out task result.

The same absolute three-task gain on two models is encouraging. Both rows use
the same held-out task set, so they are not independent replications of the
task distribution.

GPT-5.5 evolved Monet remains below Terminus-2 at 61.0%. The result does not
show that the evolved harness dominates a neutral terminal harness on every
base model.

### Specialists and merge

**[SOURCE CLAIM - DX-016](claim_evidence_ledger.md#dx-016-a-merged-terminalworld-harness-beats-the-strongest-specialist-by-one-task).**
Four specialists solve 24, 25, 26, and 27 tasks. The merged harness solves 28.
This is the only positive recombination result in the paper.

The statistical comparisons need care:

- 25/41 versus 28/41 for matched Opus base and evolved Monet has exact McNemar
  `p=0.45`;
- 28/41 versus Claude Code's 27/41 has exact McNemar `p=1.0`.

The first asks whether evolution improves the matched base. The second asks
whether the final harness beats the strongest external agent. They are not
interchangeable.

**[INFERENCE - DX-017](claim_evidence_ledger.md#dx-017-terminalworld-supports-archive-diversity-only-suggestively).**
The merge demonstrates complementary solved sets in one run. It does not
establish that population search beats a resource-matched single lineage.

Three caveats reduce the causal weight:

1. the merge adds one task over the strongest specialist;
2. a separately skill-bundled pre-TerminalWorld reference also reaches 28/41;
3. the initial specialist sweep had 12 to 17 infrastructure errors per variant
   and required policy-defined retries, while final Monet stayed at 28.

The retry result supports robustness of final Monet. It also shows how much the
specialist ordering depends on infrastructure policy.

## WebArena-Infinity

### What synthetic-to-real means

DarwinX evolves on 300 synthetic intents generated from application
description documents. The pipeline does not read official benchmark tasks.
It reports on 1,260 real tasks with deterministic verifiers.

The synthetic pool covers 12 served applications. Nine of ten reported
applications have synthetic counterparts. Gmail, 60 report tasks, is absent
from evolution. Three synthetic applications are absent from reporting.

This is transfer across:

- unseen intents;
- an LLM-judge to deterministic-verifier reward change; and
- partial application coverage.

It is not transfer to a completely unrelated browser environment.

### Base rescue and strong-harness comparison

**[SOURCE CLAIM - DX-019](claim_evidence_ledger.md#dx-019-the-paper-reports-a-large-matched-wai-gain-and-a-smaller-strong-baseline-gap).**
The paper reports:

| Harness | Frozen model | Audit-clean pass@1 |
|---|---|---:|
| Base Monet | GPT-5.5 | 43.5% |
| Browser Use | GPT-5.5 | 86.1% |
| DarwinX-evolved Monet | GPT-5.5 | 93.0% |

Two conclusions follow:

- DarwinX rescues a weak proprietary base by 49.5 points.
- It exceeds the paper's strong same-model Browser Use run by 6.9 points.

Both matter. Reporting only 49.5 points hides how much a competent browser
harness already provides.

### Recombination

**[EVIDENCE - DX-020](claim_evidence_ledger.md#dx-020-wai-contributes-no-positive-recombination-result).**
The WAI run keeps 26 iterations, reverts 36, and reverts every attempted merge.
The selected result comes from a short primary lineage.

WAI supports iterative harness editing, screening, and confirmation. It does
not support successful recombination.

### Action validity

The browser action model lets the agent write JavaScript through Chrome DevTools
Protocol. The audit permits:

- UI and DOM interaction;
- client-visible frontend assets;
- browser runtime state;
- normal authenticated application APIs; and
- application-defined semantic mutators that preserve business logic.

It rejects:

- host-only source, configuration, logs, and environment data;
- hidden task, verifier, reset, and state endpoints;
- direct storage and scored-state fabrication;
- direct database manipulation; and
- exploits, privilege escalation, or benchmark modification.

**[SOURCE CLAIM - DX-021](claim_evidence_ledger.md#dx-021-the-wai-validity-audit-reports-capability-and-compliance-improving-together).**
The paper reports:

| Metric | Base | Evolved |
|---|---:|---:|
| Raw pass@1 | 53.0% | 94.4% |
| Audit-clean pass@1 | 43.5% | 93.0% |
| Confirmed invalid | 23.5% | 1.4% |
| Invalid trajectories | 293 | 17 |
| Human review | 5.1% | 0.1% |

The detector has two stages. A static analyzer de-obfuscates scripts, detects
scored fields and semantic mutators, and taint-tracks scored collections.
Flagged trajectories go to an Opus 4.8 judge. Disagreements remain for human
review.

The audit coverage is 99.0% for base and 99.4% for evolved trajectories. The
paper calls the method stronger than regex checks and weaker than a formal
sandbox.

Published external baselines are not re-audited. The paper argues this is
conservative because DarwinX loses invalid successes while external scores do
not. That is possible, but it also means the public-agent rows do not share one
measurement pipeline.

### Action-policy change

**[EVIDENCE - DX-022](claim_evidence_ledger.md#dx-022-the-evolved-browser-harness-broadens-the-permitted-action-policy).**
The base prompt says to interact only through the UI and stop after a
screenshot. The evolved prompt prefers UI controls but permits a bounded
fallback to app-owned stores, reducers, loaded modules, public helpers, and
semantic update methods when no visible UI path can satisfy the task. It then
requires state readback, rendered UI verification, and a reload or navigation
check for persistence.

This is a legitimate policy under the paper's rubric. It is also an action-space
change. The 93.0% row measures a hybrid UI plus application-semantics agent,
not only improved visual navigation.

**[INFERENCE - DX-029](claim_evidence_ledger.md#dx-029-wai-does-not-causally-isolate-the-preservation-gate).**
The audit shows behavioral co-improvement. It does not show that the
preservation gate caused the validity improvement. Runtime guards, contract
skills, the prompt rewrite, and action-policy broadening all move together.

## SWE-bench Verified

**[SOURCE CLAIM - DX-023](claim_evidence_ledger.md#dx-023-the-paper-reports-one-way-transfer-to-swe-bench-verified).**
The paper runs the TB2.1-evolved harness unchanged on frozen Opus 4.8 and
reports 421/500, 84.2% pass@1, against an 80.8% LSP-enabled fix-skill
reference.

This supports one-way transfer from terminal tasks to repository repair. It
does not isolate:

- the transfer gain over unevolved Monet, because that row is absent;
- reverse transfer from SWE-V to TB2.1;
- in-domain SWE-V evolution; or
- a broad performance difference, because reported references span 80.8% to
  84.2%.

The official SWE-V test harness grades the final row. That makes it a real
transfer measurement under the paper's setup. It remains an author report.

## What actually changed

The TB2.1 lineage adds seven skills:

- two acceptance-contract skills;
- two graded-artifact verification loops;
- two real-tool grounding skills; and
- one security-contract repair skill.

The WAI lineage adds four browser contract skills and rewrites the interaction
policy. Both bundles require an explicit acceptance condition and a check
against real state before stopping.

**[INFERENCE - DX-025](claim_evidence_ledger.md#dx-025-verification-before-finalization-is-a-plausible-shared-mechanism).**
Verification-before-finalization is the most plausible shared procedural
mechanism in the paper.

**[EVIDENCE - DX-014](claim_evidence_ledger.md#dx-014-tb21-skill-attribution-is-exploratory).**
The paper correctly calls this exploratory attribution. The skills are
co-selected. No per-skill causal ablation exists.

## Cross-benchmark arithmetic

**[INFERENCE - DX-028](claim_evidence_ledger.md#dx-028-the-papers-17-point-average-is-descriptive-not-a-common-effect-size).**
The paper's "about 17 points on average" combines:

- `avg@5` on an in-domain 89-task suite;
- pass@1 on 41 held-out terminal tasks;
- audit-clean pass@1 on 1,260 browser tasks; and
- pass@1 transfer to 500 repository issues.

The baselines and causal questions differ. The number is descriptive arithmetic,
not a pooled effect size with a common estimand.

## Evaluation verdict

The experiments support three conclusions with different strength:

1. Harness changes can produce large matched-model behavior changes.
2. Contract and verification procedures recur across terminal and browser
   tasks and transfer to some held-out settings.
3. The full DarwinX system works in the reported experiments.

They do not establish:

- strict capability monotonicity;
- a compute-matched advantage;
- a causal effect for the archive, parent selector, gate, or merge operator;
- favorable end-to-end search economics; or
- reproduction of the proprietary system.

Continue with the [critical review](03_critical_review.md) or return to the
[DarwinX index](darwinx_index.md).

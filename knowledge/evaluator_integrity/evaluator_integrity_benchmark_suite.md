---
id: rsi-evaluator-integrity-benchmark-suite
title: Evaluator-integrity field guide for GDPval, DeepSWE, FrontierCode 1.1, and SWE-bench Verified
type: technical-review
mode: EVALUATOR-INTEGRITY REVIEW
status: active
created: 2026-08-09
updated: 2026-08-09
tags: [evaluation, benchmarks, evaluator-integrity, coding-agents, access-policy, contamination]
confidence: medium
---

# Evaluator-integrity field guide

Mode: `EVALUATOR-INTEGRITY REVIEW`.

This is the canonical comparison packet for **GDPval**, **DeepSWE**,
**FrontierCode 1.1**, and **SWE-bench Verified**. It asks what each evaluation
can establish about an agent or harness under its documented protocol. It does
not turn unlike evaluators into a synthetic leaderboard.

**No cross-benchmark ranking.** A pass rate, expert preference, or reported
leaderboard position is not a universal capability number. The task object,
hidden material, evaluator, allowed access, and aggregation differ. This
packet therefore gives conditional conclusions in the form: Under the documented access policy and verifier disclosure, this evaluator supports a narrow claim.

## Evidence contract and claim ceilings

| Evidence tier | What is present here | What it can support | What it cannot support |
|---|---|---|---|
| Immutable local receipt | Hash-pinned benchmark manifest for all four evaluations. | Reader route identity, dated source identity, and explicit rights boundary. | Execution, score reproduction, or stronger disclosure than the receipt contains. |
| Pinned implementation artifact | DeepSWE Apache-licensed README, sample `task.toml`, and license; SWE-bench MIT grading file and license. | Inspection of those explicit task/verifier contracts. | A full-suite audit, historical leaderboard reproduction, or enforcement proof beyond the captured files. |
| First-party reader route | GDPval PDF and FrontierCode 1.1 page, remote hash-pinned only. | First-party methodology and policy claims at the stated claim ceiling. | A full benchmark row: no locally pinned task corpus, verifier/policy implementation, trace set, or independent reproduction. |
| Historical-public-task control | SWE-bench Verified data revision and evaluator receipt. | A stable reference to a human-validated repair corpus and its evaluator source. | Contamination-free measurement: public historical issues and repositories are explicitly a pre-exposure risk. |
| Missing evidence | No local benchmark containers, paid-model calls, remote sandboxes, raw scored runs, or independent reproductions. | A boundary statement. | Any local capability result. |

Receipts live in `evidence/benchmarks/manifest.tsv`. The allowed local
snapshots remain verifier-only: reader-facing task data is remote-only unless
redistribution status is explicit.

## Vocabulary: evaluator integrity is not one variable

| Term | Operational question |
|---|---|
| **Access boundary** | What can the agent reach during the rollout: task repository, tests, internet, tools, credentials, and prior attempts? Who can change those permissions? |
| **Contamination history** | Could the model, agent, or harness have encountered the task, repository state, solution, or close derivative before the measured rollout? |
| **Full benchmark row** | A dated primary reader source plus at least one pinned methodological artifact: task schema, verifier, access policy, run data, or dataset revision. |
| **Evaluator disclosure** | Separate disclosure of task/repository state; verifier/test logic; tools and internet policy; traces/cost/steps; and model/agent configuration. |
| **Pre-exposure resistance** | Whether task provenance, secrecy, timing, and access make memorization or prior exposure less plausible. |
| **Live-evaluation resistance** | Whether a running agent is prevented from observing, modifying, bypassing, or gaming the evaluator. |
| **Agent visibility** | Which task material, hidden tests, tools, network resources, and feedback the agent may see during a run. |
| **Auditor visibility** | Which task material, policies, code, traces, and accounting an independent reviewer can inspect. |

Task provenance and task secrecy are distinct. A newly authored task can still
be visible to an agent; a historical task can be held out at evaluation time
while remaining broadly available in model-training history. Likewise, a
stated policy is not enforcement evidence: a strong access claim needs both a
policy and observable enforcement or audit evidence.

## Comparison matrix

| Benchmark | Task object and provenance | Mutable object | Environment and access boundary | Verifier and score | Disclosure / repetition | Conditional conclusion |
|---|---|---|---|---|---|---|
| GDPval | 1,320 knowledge-work tasks across 44 occupations and 9 sectors, based on work products from expert professionals; a 220-task open gold subset is reported. | Submitted work product, not an executable repository. | First-party PDF describes task/reference-file design but no locally pinned runnable environment or agent-access policy. | Blinded expert head-to-head comparison is the primary metric; an automated grader service is described experimentally for the 220-task gold subset. | Reader-only: task corpus, expert rubric implementation, full model configuration, traces, and cost accounting are not pinned here. | Under the reported expert-comparison protocol, GDPval can support a knowledge-work-output claim; it cannot here support an auditable live-access or contamination-resistance claim. |
| DeepSWE | 113 original long-horizon tasks from active open-source repositories across TypeScript, Go, Python, JavaScript, and Rust. | Patch committed by the agent against a named repository base commit. | Captured sample task declares agent and verifier `no-network`; the verifier runs in a separate environment and grades a patch applied to a pristine container. | Program-based verifier; `reward.json`, CTRF report, stdout, logs, and reports are specified as outputs. | Full row for the captured contract, but not full-suite enforcement: model configuration, leaderboard runs, and complete task/test snapshots are not captured here. | Under the captured task contract, DeepSWE supports a bounded long-horizon repository-repair evaluation with an explicit separation design; enforcement and suite-wide access claims remain unproven here. |
| FrontierCode 1.1 | First-party page describes a coding benchmark authored with open-source maintainers; task corpus is not locally pinned. | Agent-produced code change. | The dated first-party description says it distinguishes legitimate from unfair internet use via a fair-use prompt and a classical verifier that zeros unfair runs. | First-party page describes a classical unfair-use verifier; code, task material, and audit logs are not pinned. | Reader-only: no policy implementation, verifier implementation, task revision, traces, or model-agent configuration is locally inspectable. | Under Cognition's stated policy, FrontierCode 1.1 can motivate an internet-use integrity question; it cannot here establish access-policy enforcement or a reproducible tool-policy result. |
| SWE-bench Verified | 500 human-validated issue/PR repair instances at Hugging Face revision `c104f840…`; historical public repositories and issues. | Solution patch to a repository task. | Dataset row is remote-only; this packet does not establish the evaluated agent's network, repository, or hidden-test policy. | MIT-licensed `grading.py` is locally hash-verified at SWE-bench revision `cd37836…`. | Full row for dataset identity and a captured evaluator file, but task data remains remote-only and historical-public exposure is intrinsic. | Under the pinned dataset/evaluator identities, SWE-bench Verified supports a repository-repair control; it is not a contamination-free capability measurement. |

## Benchmark records

### GDPval — economically meaningful knowledge-work output

**EVIDENCE — [GDPVAL], official PDF receipt in
`evidence/benchmarks/manifest.tsv`.** GDPval reports 1,320 tasks, a 220-task
gold/open subset, 44 occupations across nine sectors, and tasks derived from
expert professional work products. Gold tasks can include up to 17 reference
files; full tasks can include up to 38. The PDF reports an average expert-work
estimate of seven hours, with some tasks extending to weeks.

**CLAIM — [GDPVAL].** The primary outcome is blinded, head-to-head expert
comparison of a model/agent result against a professional result. An automated
grader service is experimental and limited to the 220 open gold tasks.

**Access boundary.** The current receipt is intentionally reader-facing and
remote-only. It does not expose the full task corpus, evaluator service,
expert-comparison rubrics, prompt policy, tool permissions, task assignment
timing, raw ballots, rollout traces, model/agent versions, or spend/step
accounting. It follows that this packet cannot distinguish a strong policy
from a strong technical enforcement boundary.

**Integrity interpretation.** The professional-work origin improves face
validity for economically meaningful output; it does not, by itself, establish
pre-exposure resistance. The public gold subset is useful for inspection but
may have a different exposure profile from withheld work. Expert blinding is
an evaluator-quality control, not a substitute for disclosed agent access.

**Conditional conclusion.** Under the reported blinded expert-comparison
protocol, GDPval is appropriate when the claim is economically meaningful
knowledge-work output. It should not be selected alone to claim
tool/internet-policy robustness, reproducible live-evaluation resistance, or
absence of prior exposure.

### DeepSWE — original long-horizon repository repair

**EVIDENCE — [DEEPSWE], `evidence/benchmarks/deep-swe/README.md` at
`git:435ee89ec2f2e2289f33b0da4f992f0b7b7266b9`.** The benchmark describes 113
original tasks in five programming-language ecosystems. A task contains
`task.toml`, agent-visible `instruction.md`, environment build material,
tests/verifier configuration, and a reference solution held out from the
agent. The README says the reference patch is not used at grading time.

**EVIDENCE — [DEEPSWE],
`evidence/benchmarks/deep-swe/task.toml`.** The captured
`abs-module-cache-flags` task names an upstream repository and fixed base
commit, sets both agent and verifier `network_mode = "no-network"`, specifies
`environment_mode = "separate"`, and collects the agent's patch via a
`git diff` from the named base commit. This makes the mutable object and the
intended boundary concrete: the agent produces a patch; a pristine verifier
environment grades it.

**Disclosure and limits.** The locally captured files support inspection of
the published contract, not proof that every hosted rollout obeyed it. The
README describes output artifacts (`reward.json`, CTRF report, stdout, logs,
and reports), but no run set is vendored. The repository's Apache-2.0 license
covers Datacurve's contributions; `PROVENANCE.md` states that upstream
repositories retain their own licenses. This packet therefore vendors only
Datacurve-authored methodology metadata, not upstream task environments or
solutions.

**Integrity interpretation.** Original task curation and a no-network
contract can improve pre-exposure resistance and live-evaluation resistance
relative to a public historical issue—but neither property follows merely from
the words “original” or “no-network.” The missing evidence is suite-wide
policy enforcement, task release timing, model training cutoff/exposure audit,
network enforcement logs, and complete agent trajectories.

**Conditional conclusion.** Under the captured separate-verifier and
no-network contract, DeepSWE is the strongest packet member for a
long-horizon engineering-execution claim. It is not, on this evidence alone,
a proof that every result is contamination-resistant or technically
air-gapped.

### FrontierCode 1.1 — internet-use policy as an evaluator object

**CLAIM — [FRONTIERCODE-1-1], dated first-party page receipt in
`evidence/benchmarks/manifest.tsv`.** FrontierCode 1.1 says it refines the
methodology to distinguish legitimate internet use from unfair use. The
first-party account describes two safeguards: a fair-internet-use prompt and a
classical verifier that detects unfair use and zeros such runs.

The methodological contribution is not simply “internet on” or “internet
off.” It is a proposed *policy classification* boundary: internet use may be
allowed when it is legitimate, and disallowed when it bypasses the intended
task. That makes it directly relevant to harnesses whose tool policy is part
of the candidate behavior.

**Missing enforcement evidence.** The packet has no locally pinned fair-use
policy text, verifier source, task revision, test suite, environment image,
detector accuracy analysis, false-positive/false-negative audit, run trace,
or model/agent configuration. “Zeroes out unfair runs” is a first-party
methodology claim, not an independently inspectable control plane.

**Conditional conclusion.** Under Cognition's documented access policy,
FrontierCode 1.1 is a useful evaluator-shape comparator for
tool/internet-policy robustness. Until policy implementation and enforcement
artifacts are available, it should not be used here as evidence of
live-evaluation resistance.

### SWE-bench Verified — historical-public-task control

**EVIDENCE — [BENCH-SWE], `evidence/benchmarks/manifest.tsv`.** The pinned
Hugging Face revision is `c104f840cc67f8b6eec6f759ebc8b2693d585d4a`; it
identifies 500 human-validated issue/PR examples. The MIT-licensed evaluator
file is locally snapshot-vendored at
`evidence/benchmarks/swe-bench/grading.py`, hashed to revision
`cd37836ffec01d01a0d699a80a039d84ff2cebfe`.

This is a valuable **historical-public-task control**, not a false
contamination-free baseline. Repository issues, pull requests, test changes,
and patches were public historical artifacts. Pinning a dataset revision fixes
which rows are intended; it does not erase pretraining, agent retrieval,
benchmark-tuning, or repository-history exposure.

**Disclosure and limits.** The locally inspectable MIT grading code exposes
part of the evaluator. The selected dataset row remains remote-only because
redistribution status is unconfirmed. The receipt does not establish the
agent's available tools, network access, test visibility, model cutoff,
retrieval corpus, repeated-run count, or a specific reported score.

**Conditional conclusion.** Under the pinned dataset and evaluator identities,
SWE-bench Verified supports a repository-repair comparison against a stable,
human-validated historical corpus. Any result must disclose exposure controls
and evaluation-time access separately; otherwise it is not evidence of
pre-exposure resistance.

## Disclosure checklist for an evaluable claim

Before interpreting a result, demand a row for each independent surface:

| Surface | Minimum disclosure | What is still missing in this packet |
|---|---|---|
| Task specification / repository state | Immutable task or dataset revision, base commits, release timing, and agent-visible files. | Full GDPval and FrontierCode task material; full DeepSWE suite task snapshot; SWE-bench row redistribution. |
| Verifier / test logic | Pinned implementation or precise audit route; hidden-test boundary; aggregation rule. | GDPval expert-rubric implementation; FrontierCode verifier; full DeepSWE tests; complete SWE-bench execution image. |
| Allowed tools / internet | Stated policy plus technical enforcement evidence and exceptions. | Enforcement logs/audits for every benchmark; FrontierCode policy implementation. |
| Rollout traces / cost / steps | Per-run trace, retries, timeouts, token or action counts, and failure handling. | All four benchmark result traces and cost receipts. |
| Model / agent configuration | Model identity/version, agent commit, prompts, retrieval, tool adapters, sampling, and seeds. | Complete configurations for all reported evaluations. |

Agent visibility is not auditor visibility. A hidden test can be unavailable to
an agent and still be inspectable to an auditor under a controlled review
process; a secret evaluator with no audit route cannot establish either strong
auditor visibility or strong public reproducibility.

## What this packet does and does not compare

| Claim target | Most relevant packet member | Why | Non-substitute |
|---|---|---|---|
| **repository-repair capability** | SWE-bench Verified, with DeepSWE as a distinct original-task complement. | Both score a code-change artifact against a repository-oriented evaluator. | Neither score says the agent can perform professional knowledge work outside the task contract. |
| **long-horizon engineering execution** | DeepSWE. | Its captured contract names a repository base commit, isolated agent environment, separate verifier environment, and patch handoff. | A single task schema does not prove suite-wide access enforcement. |
| **tool/internet-policy robustness** | FrontierCode 1.1, only as an evaluator-design comparator. | It foregrounds legitimate versus unfair internet use as a measurement problem. | A stated fair-use policy without implementation/audit evidence is not an enforcement result. |
| **economically meaningful knowledge-work output** | GDPval. | Expert-created task products and blinded professional comparison target work-product quality directly. | Expert preference does not expose a coding-agent verifier or internet boundary. |

## Benchmark-selection protocol

Use the claim, not familiarity or a single aggregate score, to select an
evaluation:

1. **Repository-repair capability.** Start with SWE-bench Verified as the
   historical-public-task control. Disclose model/retrieval exposure, task
   revision, grader version, test visibility, tools, and repetitions. Add
   DeepSWE when the claim also needs original-task provenance and a separate
   verifier design.
2. **Long-horizon engineering execution.** Select DeepSWE only after checking
   the concrete task's base commit, task contract, agent network policy,
   verifier environment, patch collection, timeout, and output artifacts.
   Report whether the stated no-network policy was technically enforced and
   audit the release/exposure timeline.
3. **Tool/internet-policy robustness.** Treat FrontierCode 1.1 as a policy
   evaluation hypothesis. Require the fair-use rule, classifier/verifier
   implementation, exception process, false-positive/negative analysis, tool
   traces, and enforcement evidence before promoting a result beyond
   first-party reporting.
4. **Economically meaningful knowledge-work output.** Use GDPval when
   blinded expert comparison and work-product usefulness are central. Require
   task sampling, professional comparator protocol, rater blinding and
   agreement, reference-material policy, agent access, and cost/time
   accounting.

For every choice, preserve the candidate/envelope split from
`content/evaluator_integrity_and_promotion.md`: the agent may improve within
its declared mutable surface, while task selection, access policy, verifier,
budget accounting, archive, and promotion authority remain independently
controlled. A benchmark score becomes useful evidence only after those
boundaries are disclosed.

## No local reproduction

No local reproduction was run for GDPval, DeepSWE, FrontierCode 1.1, or
SWE-bench Verified. This work made no paid model calls, benchmark-container
runs, remote-sandbox runs, or score claims. The packet is a source- and
receipt-grounded evaluator-integrity review, not a new benchmark result.

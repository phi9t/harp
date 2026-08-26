# Lean proof infrastructure research brief

Status: research input, not an accepted architecture

Snapshot: 2026-08-23

Local reference tree: `8723d1c01bb6aae1e6eab133a3614c3056cdd002`

## Executive findings

The strongest lesson from LeanDojo is not a particular prover. It is the
combination of an immutable repository identity, machine-readable Lean
interaction, extracted dependency context, and kernel-checked outcomes. The
original implementation made that combination practical; its own README now
directs new users to LeanDojo-v2. The v2 repository broadens the scope to
tracing, dynamic databases, training, retrieval, proof search, and deployment,
but its public implementation does not yet establish a stable substrate
contract across all those layers.

Five conclusions survive the project-by-project comparison:

1. **There is no shared Lean machine-interaction contract.** The official
   language server, community REPL, Pantograph, LeanInteract, and Kimina Lean
   Server expose different state identities, lifecycle rules, request forms,
   error models, and persistence semantics. A proof system that directly
   embeds any one of them leaks those choices into every caller.
2. **The useful unit is a proof program, not one tactic state.** Crouzeix-class
   work spans theorem contracts, informal obligations, many Lean declarations,
   route-specific import policies, source correspondence, reviews, and
   publication. Tactic stepping is one operation inside that lifecycle.
3. **Fast interaction and trustworthy acceptance are different services.** A
   persistent Lean process is valuable for iteration. Acceptance needs a fresh,
   sealed environment, an exact command envelope, axiom/import checks, and an
   immutable receipt. Reusing an interactive process as the certifier makes
   mutable process state part of the trust boundary.
4. **A cache hit is not an environment identity.** Lean toolchain, repository
   tree, Lake manifest, dependency closure, platform, options, generated files,
   and build products all affect meaning or reproducibility. Existing systems
   pin different subsets and frequently couple setup with network mutation.
5. **Harp's bottleneck is evidence closure, not candidate generation.** At the
   frozen local tree, substantial Jin, Lorist--Schwenninger, and Harp Lean
   sources exist, while the three expected route manifests are absent, the
   verifier skips those absences, and the local formalization bundle is
   optional. Producing more proofs before closing this lifecycle would increase
   throughput without increasing justified confidence.

The resulting design hypothesis is a small set of deep modules around a
versioned environment contract, a source-aware proof graph, a two-lane Lean
execution boundary, Harp's existing durable task engine, and independent
evidence publication. This is deliberately a hypothesis. The product boundary
and module design remain open pending the decision questions at the end of this
brief.

## Research question and evaluation rubric

The research question is:

> What infrastructure would let humans and heterogeneous agents develop,
> evaluate, reproduce, and promote repository-scale Lean proofs efficiently,
> without letting optimization state, model output, caches, or orchestration
> records acquire proof authority?

Projects were compared on eight dimensions:

| Dimension | Question |
|---|---|
| Environment identity | What exact repository, toolchain, package closure, platform, and options are bound to an operation? |
| Interaction | What can a caller ask Lean to do, and how are states, edits, goals, diagnostics, cancellation, and crashes represented? |
| Semantic scope | Does the system understand one goal, a file, a package, a dependency graph, an informal source graph, or a full proof campaign? |
| Recovery | Can work resume without losing or duplicating accepted effects? Are states replayable or merely process-local? |
| Trust | Which component establishes kernel acceptance, allowed axioms/imports, source correspondence, and mathematical review? |
| Evidence | Are attempts, failures, candidates, evaluations, environments, and final claims immutable and linked? |
| Scale | What is parallelized, cached, measured, and bounded? What becomes a shared writer? |
| Research validity | Are datasets, splits, budgets, environments, and failure categories sufficient to interpret reported results? |

## Corpus, method, and stopping rule

### LeanDojo organization

The GitHub organization API returned 15 public repositories on 2026-08-23.
All 15 were inspected at independently resolved full commit SHAs. None was
marked archived, fork, or template, though the original LeanDojo and
LeanDojoChatGPT READMEs describe themselves as deprecated and outdated,
respectively. Inspection was capped at 12 implementation or paper artifacts
per core repository and three per peripheral repository. Only official
repositories, project pages, and author papers linked one hop from them were
used.

### Adjacent ecosystem

The related-effort cohort was capped before inspection at 24 systems across
six strata: environment/build, official authoring, machine proof interaction,
extraction/retrieval, generation/search, and evaluation/provenance. Lean 4,
Lake, and the official server count as one system because they share the same
frozen source artifact; the public Crouzeix repository counts only as a
provenance stress test. Inclusion otherwise required a first-party
implementation or specification that materially affects a next-generation
Lean runtime. The aim was architectural coverage, not a ranking of every Lean
model or benchmark.

### Local Crouzeix workload

Local claims use only the tracked reference tree above, accepted ADRs and
specifications, and explicitly labeled observations from the named Crouzeix
worktrees. No Lean build, dependency hydration, or cache update was run. Source
presence is therefore not reported as fresh compilation or certification.

### Evidence classes

- Immutable source at a full commit supports what an implementation contains.
- Versioned author papers support what authors report, not independent
  replication.
- Dated first-party metadata supports only the state observed on that date.
- Local tracked receipts support only the exact operation and claim named by
  the receipt.
- Architectural conclusions in this document are researcher inferences.
- `MISSING` means evidence needed for a stronger claim was absent or not
  available within the capped first-party corpus.

## Complete LeanDojo organization census

| Repository | Role | Durable contribution | Material limitation at the inspected revision |
|---|---|---|---|
| [ReProver](https://github.com/lean-dojo/ReProver/tree/fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa) | Reference neural prover | Accessibility-aware premise retrieval, explicit proof-search graph, separate actor/environment timing | Old model baseline; likelihood-led search; no release or general service contract |
| [LeanDojoWebsite](https://github.com/lean-dojo/LeanDojoWebsite/tree/95aa617c239806ecee6b4da0c8c4c9ea76edf17a) | Official publication and project index | First-party map of the program and papers | Static index, not runtime or reproducibility evidence |
| [LeanDojo](https://github.com/lean-dojo/LeanDojo/tree/7a9f600b250a89a62e991c05ffe81ba033018e6e) | Historical trace and interaction substrate | Commit-keyed repositories, premise-aware extraction, typed tactic outcomes | README deprecates it for new work; `pexpect`/temporary-file protocol and trace schema are not a separately versioned public contract |
| [LeanDojoChatGPT](https://github.com/lean-dojo/LeanDojoChatGPT/tree/1fbb7506dca60e0bdc031d423014718715f63ccd) | Legacy ChatGPT-plugin demo | Early human-facing example of model/Lean interaction | README says obsolete after OpenAI API changes |
| [LeanCopilot](https://github.com/lean-dojo/LeanCopilot/tree/60738032a1398a4e83524107424a11322a753609) | In-Lean model and proof-assistance layer | Very small provider interfaces; Lean remains checker; interactive search and premise suggestions | External HTTP adapter lacks a versioned provenance/cancellation/security contract; reported evaluation uses a simulated user |
| [LeanAgent](https://github.com/lean-dojo/LeanAgent/tree/13f63bd8ed42057dbf909791ba39ea6f8759c690) | Continual-learning research prototype | Commit-keyed dynamic knowledge, progressive adaptation, retention metrics | Same-repository adaptation complicates generalization claims; placeholder exploits expose the gap between kernel acceptance and theorem value; no license found at HEAD |
| [LeanMillenniumPrizeProblems](https://github.com/lean-dojo/LeanMillenniumPrizeProblems/tree/fd5207106c8c13c40cd4eeb0acb169c2c4e58aeb) | Formal-statement workload | Domain-rich, intentionally open theorem surfaces | Statements with `sorry` are targets, not solutions; statement fidelity still requires domain review |
| [lean4code](https://github.com/lean-dojo/lean4code/tree/3215b7e833e16b1be5a0a689277358208fe4f97b) | VSCodium distribution | Packages Lean, LeanCopilot, tracing, and agent integrations for users | Distribution shell, not an authoritative environment or proof-evidence protocol |
| [LeanDojo-v2](https://github.com/lean-dojo/LeanDojo-v2/tree/baed5eae6e87a65a446d9f54af07aab2154e7599) | Active end-to-end integration stack | Tracing, dynamic database, agents, SFT/GRPO/retrieval, proving, and external inference in one project | Alpha classifier; sparse conformance coverage; broad dependency/setup behavior; unclear replacement equivalence; Apache/MIT license metadata conflict at inspected revision |
| [LeanProgress](https://github.com/lean-dojo/LeanProgress/tree/328ca46ede8ab9e3506805ba7658859af74aa7be) | Learned proof-progress research | History-aware, policy-conditioned estimate of remaining successful steps | Labels are shortest observed successful ReProver paths, not intrinsic mathematical distance; failures are censored |
| [TorchLean](https://github.com/lean-dojo/TorchLean/tree/4caafbb692139908da104a4cf1c3699a54acd4bc) | Verified neural-runtime and certificate project | Shared typed semantic IR, multiple interpreters/checkers, unusually explicit trust boundaries | Different problem from theorem search; native/CUDA/imported artifacts and floating-point assumptions sit outside the strongest theorem boundary |
| [BRIDGE](https://github.com/lean-dojo/BRIDGE/tree/152ffbeaa1d9d320208b95b28fc39cf824b8780b) | Code/specification/proof generation research | Functional intermediate representations and executable artifact checks | Executability does not establish semantic fidelity; intended SFT checkpoint and complete task provenance were missing at HEAD |
| [QuantumLean-Bench](https://github.com/lean-dojo/QuantumLean-Bench/tree/dac8c9da3ccdbb0319b8e9dc76b94c11f7dcf39b) | Quantum-domain benchmark | Tests cross-library, scientific-domain reasoning rather than only contest algebra | Benchmark compilation does not itself establish faithful physics semantics |
| [ITPEval](https://github.com/lean-dojo/ITPEval/tree/30dc5fe5f0ec7d6e6fae4b2d7eb3d56c6da7fb2a) | Cross-proof-assistant evaluation | Isolated Lean/Rocq/Isabelle/HOL Light execution over aligned tasks | Target execution does not generally certify source-target equivalence; no immutable quantitative result ledger found at HEAD |
| [LeanProfiler](https://github.com/lean-dojo/LeanProfiler/tree/ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4) | Operational observability | Strict versioned timing artifacts, nested spans, Perfetto export, invariant validation, dual-threshold regression policy | No published overhead/variance study; manual instrumentation and truncation affect interpretation |

Operational metadata below was observed on 2026-08-23. A release count is a
distribution/maturity clue, not a quality score. “Direction” records principal
inputs and visible in-organization consumers rather than a complete dependency
graph.

| Repository | HEAD date | License at HEAD | Releases observed | Principal dependency/consumer direction |
|---|---:|---|---:|---|
| ReProver | 2025-01-30 | MIT | 0 | Original LeanDojo + PyTorch/Hugging Face/Lightning/Ray → LeanCopilot models and LeanAgent/LeanProgress baselines |
| LeanDojoWebsite | 2026-07-26 | MIT | 0 | Static web stack → human index for LeanDojo, LeanCopilot, LeanProgress, BRIDGE, and ITPEval |
| LeanDojo | 2026-01-18 | MIT | 35; latest `v4.20.0` | Lean/Git/elan/Python → ReProver, LeanDojoChatGPT, LeanAgent; migration points to v2 |
| LeanDojoChatGPT | 2024-04-04 | MIT | 0 | Original LeanDojo + historical ChatGPT-plugin/Quart APIs → no current supported consumer stated |
| LeanCopilot | 2026-08-22 | MIT | 49; latest `v4.33.0` | Lean + CTranslate2/OpenBLAS or HTTP providers → lean4code and search/progress consumers |
| LeanAgent | 2025-06-13 | **MISSING** | 0 | Original LeanDojo/ReProver + Ray/Lightning → ideas described as integrated into v2 |
| LeanMillenniumPrizeProblems | 2026-07-11 | Apache-2.0 | 1; latest `v.1.0` | Lean/mathlib + explicit domain interfaces → proof-system workload |
| lean4code | 2026-08-18 | MIT | 5; latest `v1.4.0` | VSCodium/Lean extension + LeanCopilot/tracing/agents → end users |
| LeanDojo-v2 | 2026-08-10 | Apache-2.0 file; README/manifest say MIT | 4; latest `v1.0.9` | Pantograph + PyTorch/Hugging Face/DeepSpeed/Ray/model APIs → intended new LeanDojo clients |
| LeanProgress | 2026-01-10 | MIT | 0 | ReProver trajectories + DeepSeek/Qwen models → v2 search stack |
| TorchLean | 2026-08-22 | MIT | 4; latest `v1.3` | Lean/mathlib + optional native/CUDA/PyTorch/Julia → verified-ML consumers and optional LeanProfiler spans |
| BRIDGE | 2026-05-15 | MIT | 0 | Lean/mathlib + Python/model endpoints → formalization front ends and benchmark users |
| QuantumLean-Bench | 2026-06-22 | MIT | 0 | Lean 4.24.0 + mathlib/PhysLean + model endpoints → quantum-domain evaluation |
| ITPEval | 2026-07-08 | MIT | 0 | Lean/Rocq/Isabelle/HOL Light + model APIs → cross-ITP evaluators |
| LeanProfiler | 2026-08-11 | MIT | 0 | Lean + optional TorchLean spans → runtime diagnostics and CI regression gates |

The census supports a program-level classification:

- **Substrate:** original LeanDojo, LeanDojo-v2.
- **Proof policy and learning:** ReProver, LeanAgent, LeanProgress,
  LeanCopilot.
- **Artifact semantics and trust:** TorchLean, BRIDGE, ITPEval.
- **Operational and user surfaces:** LeanProfiler, lean4code.
- **Workloads and communication:** LeanMillenniumPrizeProblems,
  QuantumLean-Bench, LeanDojoWebsite, LeanDojoChatGPT.

No one repository supplies a complete environment, session, proof-program,
certification, and promotion contract.

## Core LeanDojo project findings

### Original LeanDojo and ReProver

The original LeanDojo's lasting abstraction is a theorem inside a repository
identified by URL and commit, plus a typed transition boundary whose terminal
states distinguish a new tactic state, proof completion, Lean error, give-up,
timeout, and process failure. Its paper reports that premise-aware extraction
reduced proof misclassification relative to `lean-gym` and that retrieval
improved ReProver on both random and novel-premise splits. Those results support
the value of accessible-premise context and commit-coupled tracing. They do not
validate the exact Python/process implementation as a durable infrastructure
boundary.

ReProver makes three useful policies explicit: retrieval is restricted to
premises accessible at the theorem location, Lean execution decides validity,
and graph statuses propagate separately from model scores. Its cumulative
log-probability best-first priority is a research policy, not a substrate
invariant. A future runtime should allow success probability, cost-to-go,
uncertainty, novelty, diversity, and resource risk to coexist rather than
forcing every prover through one scalar.

Primary evidence: the pinned [LeanDojo README](https://github.com/lean-dojo/LeanDojo/blob/7a9f600b250a89a62e991c05ffe81ba033018e6e/README.md), [interaction implementation](https://github.com/lean-dojo/LeanDojo/blob/7a9f600b250a89a62e991c05ffe81ba033018e6e/src/lean_dojo/interaction/dojo.py), [ReProver search](https://github.com/lean-dojo/ReProver/blob/fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa/prover/proof_search.py), and [LeanDojo/ReProver paper](https://arxiv.org/abs/2306.15626v2).

### LeanCopilot

LeanCopilot shows how little provider API is actually needed inside Lean:
text-to-text generation and text-to-vector encoding. Lean owns the local goal
state and checks suggestions. That seam is deeper than a feature-rich client
SDK because native, local, and remote providers can change without changing
the tactic caller.

The missing operational fields are equally instructive. A production response
needs the model and artifact digest, protocol version, score semantics, seed,
deadline, cancellation outcome, and request/response identity. Long-running
search should not be owned by the editor process. Its reported automation rate
comes from a simulated user following known textbook proofs, so it demonstrates
utility under that protocol rather than autonomous theorem-proving quality.

Primary evidence: [LeanCopilot provider interface](https://github.com/lean-dojo/LeanCopilot/blob/60738032a1398a4e83524107424a11322a753609/LeanCopilot/Models/Interface.lean), [external adapter](https://github.com/lean-dojo/LeanCopilot/blob/60738032a1398a4e83524107424a11322a753609/LeanCopilot/Models/External.lean), and [paper](https://arxiv.org/abs/2404.12534v3).

### LeanAgent and LeanProgress

LeanAgent turns a repository's completed proofs into a dynamic theorem database
before attempting its holes. That is a useful continual-maintenance workflow,
but it must be labeled adaptation rather than held-out generalization. The
paper's discovery of accepted placeholder exploits is a decisive warning:
kernel acceptance answers whether the formal statement follows, not whether
the statement captures a meaningful intended theorem.

LeanProgress improves search by predicting remaining successful-policy steps
from a state and history. Its labels are generated from shortest observed
successful trajectories, so they should be interpreted as a model- and
policy-conditioned cost-to-go with uncertainty. Failed and dead-end
trajectories are essential training evidence; discarding them creates survivor
bias and weakens failure prediction.

Primary evidence: [LeanAgent](https://github.com/lean-dojo/LeanAgent/tree/13f63bd8ed42057dbf909791ba39ea6f8759c690), [LeanAgent paper](https://arxiv.org/abs/2410.06209v8), [LeanProgress](https://github.com/lean-dojo/LeanProgress/tree/328ca46ede8ab9e3506805ba7658859af74aa7be), and [LeanProgress paper](https://arxiv.org/abs/2502.17925v3).

### LeanDojo-v2

LeanDojo-v2 is the authors' intended integration destination. It combines
repository tracing, a dynamic database, Pantograph interaction, agent and
prover abstractions, supervised and reinforcement-learning paths, retrieval,
and external model APIs. This breadth is valuable as a component laboratory.

The same breadth makes a missing substrate contract costly. At the inspected
revision, setup can clone and build mutable work directories and opportunistically
request Lake cache artifacts; important session/search behavior is not covered
by a public conformance suite; the package declares Alpha status; dependency
and license metadata are inconsistent; and the public search implementation is
not simply the original ReProver algorithm. The correct conclusion is
“intended successor with reusable components,” not “drop-in, operationally
equivalent platform.”

Before adoption as infrastructure, v2 would need versioned contracts and
fixtures for environment identity, trace schema, state transitions,
cancellation, replay, package closure, search semantics, and migration from the
original corpus.

Primary evidence: [LeanDojo-v2 README](https://github.com/lean-dojo/LeanDojo-v2/blob/baed5eae6e87a65a446d9f54af07aab2154e7599/README.md), [package manifest](https://github.com/lean-dojo/LeanDojo-v2/blob/baed5eae6e87a65a446d9f54af07aab2154e7599/pyproject.toml), and [checked-in license](https://github.com/lean-dojo/LeanDojo-v2/blob/baed5eae6e87a65a446d9f54af07aab2154e7599/LICENSE). A linked quantitative paper could not be extracted from OpenReview during this audit, and the repository does not expose an equivalent immutable results ledger.

### TorchLean

TorchLean contributes the strongest trust-architecture pattern in the
organization: one typed semantic object is interpreted by execution,
transformation, bound propagation, and proof/certificate checkers, while each
claim explicitly names what remains trusted. External CPU/CUDA producers may
create artifacts, but Lean checks a narrowly stated predicate over those
artifacts rather than assuming the producer correct.

That pattern generalizes to theorem infrastructure. A candidate, route graph,
environment, build receipt, correspondence review, and promotion decision
should share stable identities and have separate checkers. A producer should
never acquire authority merely because its artifact was parseable or fast.

Primary evidence: [TorchLean README](https://github.com/lean-dojo/TorchLean/blob/4caafbb692139908da104a4cf1c3699a54acd4bc/README.md), [trust boundaries](https://github.com/lean-dojo/TorchLean/blob/4caafbb692139908da104a4cf1c3699a54acd4bc/TRUST_BOUNDARIES.md), and [paper](https://arxiv.org/abs/2602.22631v2).

### BRIDGE

BRIDGE tests whether functional intermediate representations improve
translation among code, specification, and proof. Its reported gains support
using explicit intermediate artifacts during autoformalization. They also
expose the boundary: executable code and elaborating specifications can still
be vacuous, implementation-shaped, or semantically wrong. Statement review,
equivalence/refinement obligations, adversarial tests, and provenance must run
before such artifacts enter a theorem campaign.

Primary evidence: [BRIDGE README](https://github.com/lean-dojo/BRIDGE/blob/152ffbeaa1d9d320208b95b28fc39cf824b8780b/README.md), [reproducibility checklist](https://github.com/lean-dojo/BRIDGE/blob/152ffbeaa1d9d320208b95b28fc39cf824b8780b/docs/reproducibility_checklist.md), and [paper](https://arxiv.org/abs/2511.21104v4).

### ITPEval

ITPEval's four-assistant execution harness is a useful model for isolated,
toolchain-specific checking and normalized failure reporting. The crucial
limit is that a target proof can compile while the translated statement differs
from the source. Cross-system interoperability therefore needs an explicit
correspondence obligation, environment images or digests, immutable result
manifests, and round-trip or metamorphic checks where semantic equivalence is
not directly expressible.

Primary evidence: [ITPEval README](https://github.com/lean-dojo/ITPEval/blob/30dc5fe5f0ec7d6e6fae4b2d7eb3d56c6da7fb2a/README.md) and [evaluation contract](https://github.com/lean-dojo/ITPEval/blob/30dc5fe5f0ec7d6e6fae4b2d7eb3d56c6da7fb2a/eval/README.md).

### LeanProfiler

LeanProfiler demonstrates that observability can itself have a strict,
versioned artifact contract. Stable span order, nested timing invariants,
inclusive/self-time relationships, event caps, and dual absolute/relative
regression thresholds are more useful than ad hoc log scraping. A proof runtime
should add environment, task, candidate, and model identities; repeated-run
distributions; dropped-event accounting; and causal links across retrieval,
generation, Lean interaction, and certification.

Primary evidence: [LeanProfiler README](https://github.com/lean-dojo/LeanProfiler/blob/ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4/README.md), [capture implementation](https://github.com/lean-dojo/LeanProfiler/blob/ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4/LeanProfiler/Runtime/Capture.lean), and [artifact validation](https://github.com/lean-dojo/LeanProfiler/blob/ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4/LeanProfiler/Artifact/Validation.lean).

## Adjacent ecosystem comparison

The comparison below records the architectural contract worth preserving and
the boundary it does not close. It is not a maturity or performance ranking.

| Layer and system | Useful contract | Unclosed boundary |
|---|---|---|
| Build/authoring: [Lean 4, Lake, and official server](https://github.com/leanprover/lean4/tree/d8b18978322de05a8f3dba51ef03cf5461676c17) | Toolchain-selected language, manifest-driven packages/targets and module traces; watchdog/per-file-worker isolation, asynchronous snapshots, incremental reuse, diagnostics, `InfoTree`, LSP/RPC | Build receipts, document snapshots, and search transactions have different lifecycles; none alone is a sealed proof environment or certification receipt |
| Build: [elan](https://github.com/leanprover/elan/tree/b6cec7e10fe4965a605aaf60d1cb4a5837f0462b) | Lean toolchain selection and installation | Toolchain name alone does not identify repository, packages, options, platform, or artifacts |
| Build: [mathlib4 and cache](https://github.com/leanprover-community/mathlib4/tree/520045ab14e26149ee970e2e617ca04b09bde5d6) | Large shared formal library and content-sensitive prebuilt artifact workflow | Cache keys identify inputs and trust flow but do not authenticate cached artifact bytes; cache mutation remains a separate authority |
| Build: [Reservoir](https://github.com/leanprover/reservoir/tree/4bde19c2dba9244d97336768ce8a97aaddbd2b72) | Package discovery plus observed build/test status across Lean releases | Registry metadata is mutable discovery evidence, not a campaign's locked dependency closure |
| Session: [community REPL](https://github.com/leanprover-community/repl/tree/5d5c49d13dfc0c1d2df43a27c3e56e02ad81b9c3) | JSON commands and tactics, environment/proof-state backtracking and optional pickling | Serialized state has version/lifecycle limitations and remains coupled to exact Lean/project internals |
| Session: [Pantograph](https://github.com/leanprover/Pantograph/tree/d704b851542b1d2caf1287f65c49f5011f687c05) | Machine-to-machine tactics, metavariable coupling, constant inspection, tactic extraction, whole-file checking | Its state/protocol and build assumptions differ from REPL/LSP clients; adoption does not seal the surrounding environment |
| Session: [LeanInteract](https://github.com/augustepoiroux/LeanInteract/tree/976edd7d38a99e1ea4c2dfabeb8ad98baffca3c8) | Typed Python commands, multiple Lean versions/projects, environment and proof-state pickling, crash-aware server wrapper | Custom REPL compatibility and setup remain client concerns; recovery is not a durable workflow/evidence contract |
| Session: [Lean4Kit description](https://openreview.net/forum?id=98w016SKJg) | Lightweight extraction and interaction used by automated theorem-generation work | A public implementation, schema, release, and compatibility matrix were not found in the capped primary corpus |
| Session: [Kimina Lean Server](https://github.com/project-numina/kimina-lean-server/tree/fb2393de3461db35eda4c714e3fd21187e92ec90) | Parallel REPL pool behind a REST API, client SDK, reuse/memory controls, large-batch checking | Header/project-keyed pools optimize throughput but do not by themselves bind every request to a sealed source and evidence identity |
| Knowledge: LeanDojo-v2 | Repository tracing and a dynamic theorem database joined to agent/training/prover code | Representation/version/migration guarantees need explicit conformance tests |
| Knowledge: [doc-gen4](https://github.com/leanprover/doc-gen4/tree/97d4ecdfc8e09e7f511724c25e303d448de6a3db) | Declaration documentation and cross-linked library structure | Documentation indices omit failed traces, proof-search state, informal-source correspondence, and evaluation policy |
| Knowledge: [LeanSearch v2](https://github.com/frenzymath/LeanSearch-v2/tree/94f4888cbaf9f4322535755f86cbac690ec18080) | Natural-language and semantic retrieval over Lean declarations | Retrieval relevance is advisory; accessibility, environment version, and proof-policy constraints must be enforced downstream |
| Knowledge: [Lean Workbook](https://huggingface.co/datasets/internlm/Lean-Workbook/tree/2e066e310b2c6d2c27616927ae131f82901c8f1c) | Large synthetic formal/informal statement and proof corpus | Synthetic filtering and compilation do not eliminate mistranslation, leakage, duplicate families, or shallow proof distributions |
| Prover: ReProver | Accessible-premise retrieval plus graph search | Static benchmark/search policy; not a repository-scale proof-program runtime |
| Prover: LeanCopilot | In-Lean replaceable providers and checked suggestions | Editor assistance is not durable orchestration or independent certification |
| Prover: [DeepSeek-Prover-V2](https://github.com/deepseek-ai/DeepSeek-Prover-V2/tree/e598a57ea3284997d4a2a168a069fdd5064afbc8) | Subgoal decomposition and large-model proof generation | Pass-rate gains do not specify environment/session/evidence interoperability; sampling budget dominates operational cost |
| Prover: [Goedel-Prover-V2](https://github.com/Goedel-LM/Goedel-Prover-V2/tree/2e9036e118464aa96a8bebaf9f5b9d091aa3585c) | Scaffolded data synthesis, verifier-guided self-correction, model averaging | Benchmark-level self-correction can overfit verifier feedback and does not resolve source correspondence or campaign promotion |
| Prover: [Kimina-Prover](https://github.com/MoonshotAI/Kimina-Prover-Preview/tree/7abd61a5d9861bf5c15b195c216c1bb233ac32e4) | Reinforcement-learned formal reasoning paired with a scalable Lean server; released proof artifacts expose benchmark defects | Model and server throughput do not supply a general proof-program or human authority contract |
| Evaluation: [lean-eval](https://github.com/leanprover/lean-eval/tree/b91d4757aa0d7776c02540c9089df54fa0d0658a) | Comparator-based checking, sandbox boundary, immutable submission/result separation | Comparator correctness and problem semantics still need explicit review; theorem score is not production certification |
| Evaluation: [miniF2F](https://github.com/facebookresearch/miniF2F/tree/e4f113090ad82d64f8ce064d2f55b613a9b6bded) | Widely used formal contest-math split | Small, saturated, version-fragmented benchmark; theorem families and statement defects complicate comparisons |
| Evaluation: [PutnamBench](https://github.com/trishullab/PutnamBench/tree/dfb0a47a1c1ec3a10f2a9acfdf41a2043920f33c) | Harder multi-assistant undergraduate mathematics benchmark | Still theorem-level and benchmark-specific; many open problems turn absence of proof into an ambiguous signal |
| Evaluation: [Formal Conjectures](https://github.com/google-deepmind/formal-conjectures/tree/488aade228ec37880b8fec178c173c07d279bb53) | Versioned, evolving, source-linked open conjecture statements with explicit accuracy warnings | Human statement fidelity remains central; benchmark snapshots must not be silently repaired in place |
| Evaluation: [ProofNet](https://github.com/zhangir-azerbayev/ProofNet/tree/509ad79710ed4f46ff5c282ed5640c1aa9ac3f30) | Natural-language/formal statement pairs for autoformalization and proving | Paired statements need correspondence review; pass rates conflate formalization and proof difficulty unless staged |
| Provenance stress test: [public Crouzeix snapshot](https://github.com/jinshanmu/CrouzeixConjecture/tree/9df07838327b988e3924453daa29c8cd726d34b0) | Public prompt, manuscript/proof artifacts, Lean declaration map, and manuscript audit expose part of the artifact lineage | Worker transcripts, route registry, model/tool identities, rejected attempts, budgets, temporal independence, and independent verifier receipts are absent; it is not proof-infrastructure validation |

### Cross-ecosystem synthesis

The ecosystem converges on several good ideas without converging on a protocol:

- Exact Lean and mathlib versions matter.
- Long-lived processes and state reuse are needed for throughput.
- Lean must check every generated proof action or final proof.
- Repository extraction and accessible-premise context improve learned systems.
- Large-scale evaluation needs parallel isolation and normalized outcomes.
- Benchmark artifacts need stable snapshots and provenance.

The incompatibilities are equally consistent:

- State IDs range from LSP snapshots to REPL indices, pickles, process-local
  objects, or HTTP jobs.
- A “command” can mean file text, a top-level command, a tactic, a proof state
  transition, or a batch verification request.
- Environment setup may be caller-managed, auto-cloned, mutable, containerized,
  or implicit in a shared server.
- Timeouts, cancellations, worker death, malformed output, and Lean errors have
  different vocabularies.
- None of the interaction layers is also a complete source-correspondence and
  claim-promotion system.

This suggests an adapter boundary over existing Lean mechanisms, not another
fork of Lean and not a universal replacement for LSP, REPL, or Pantograph.

## Harp and Crouzeix as the reference workload

### Scale and current state

At the reference tree:

- The Crouzeix-named Lean source surface is approximately 20,000 lines across
  more than 100 files, with three route aggregates and substantial shared
  foundations.
- The top-level Python proof-reproduction implementation is 28,042 lines and
  its test directory is 22,113 lines under the audit's explicit counting
  boundary.
- The Rust Crouzeix verifier surfaces total 9,445 lines across
  `crates/harp/src/sources/crouzeix.rs` and
  `crates/harp/src/sources/crouzeix/route.rs`.
- The canonical dependency cache is approximately 14 GiB. One historical
  worktree had another real cache of approximately 13 GiB, while the current
  certification worktree links to the canonical cache as policy requires.
- The expected Jin, Lorist--Schwenninger, and Harp route-manifest files are
  absent from the frozen tree. The Rust verifier skips an absent route manifest,
  and `REQUIRE_LOCAL_FORMALIZATION_MANIFEST` is false.
- Canonical prose, later Lean source, mutable tracker state, and route evidence
  currently describe different stages of completion. This is a conflict to
  preserve, not permission to infer that one silently upgrades the others.

The exact file-level counts differ depending on whether aggregate wrapper files
are included. This brief uses the broad “Crouzeix-named source surface” only as
an order-of-magnitude workload measure, not as a semantic proof count.

Key frozen local anchors are below. `HEAD:` means the reference tree named at
the top of this document; it does not mean the mutable primary checkout.

| Claim surface | Reference-tree evidence |
|---|---|
| Attempt/result/node/evaluation/archive/candidate vocabulary | `HEAD:CONTEXT.md:49-82` |
| Cache and worktree discipline | `HEAD:AGENTS.md:47-80` |
| Human authority and separation of intent/execution/verification | `HEAD:AGENTS.md:82-99` |
| Dynamic workflow compiles to the existing durable graph | `HEAD:docs/adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md:13-32,52-73` |
| Deferred ambiguous-crash reconciliation | `HEAD:docs/adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md:92-99` |
| Provider call ordering and orphan recovery | `HEAD:labs/crouzeix_proof_reproduction/expert_runner.py:342-390,593-611` |
| Blind evaluation identity checks and candidate probes | `HEAD:labs/crouzeix_proof_reproduction/frontier.py:204-279,330-381` |
| Blind correctness-context projection | `HEAD:labs/crouzeix_proof_reproduction/review_contracts.py:201-286` |
| Route identifiers, expected manifest paths, and reuse policy | `HEAD:labs/crouzeix_proof_reproduction/route_validation.py:29-74` |
| Proof/source graph node contract | `HEAD:labs/crouzeix_proof_reproduction/route_validation.py:81-115` |
| Route receipt and review contract | `HEAD:labs/crouzeix_proof_reproduction/route_validation.py:139-156` |
| Duplicated Rust route identifiers and policy | `HEAD:crates/harp/src/sources/crouzeix/route.rs:16-38` |
| Missing route manifests are skipped | `HEAD:crates/harp/src/sources/crouzeix/route.rs:760-789` |
| Local formalization bundle is optional | `HEAD:crates/harp/src/sources/crouzeix.rs:61-104,472-505` |
| Route-specific Lake targets | `HEAD:formalization/lean/lakefile.toml:1-40` |
| Focused target, cache, and policy checks | `HEAD:scripts/check_lean_library.sh:80-172,235-608` |
| Cache/worktree receipt validation | `HEAD:labs/crouzeix_proof_reproduction/proof_evidence.py:677-780` |
| Canonical status and independence cautions | `HEAD:knowledge/crouzeix_conjecture/09_status_and_critical_assessment.md:19-51,69-96,117-133` |
| Pinned Jin formal target | `HEAD:labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/formal-target.lock.json:34-65` |

The source-size figures are read-only measurements, not proof metrics. The
28,042-line implementation figure uses only top-level Python files under
`labs/crouzeix_proof_reproduction`; the 22,113-line figure uses only top-level
Python files in its `tests` directory. The Rust figure is the sum of `wc -l`
over the two named verifier files. Manifest absence was checked with
`git ls-tree -r` at the frozen tree. Cache sizes and worktree-link state are
dated machine-local observations from 2026-08-23 and have no claim authority.

### Existing Harp strengths to preserve

Harp already distinguishes expert attempt, expert result, immutable
mathematical node, admission decision, blind node evaluation, archive entry,
and candidate projection in `CONTEXT.md`. It also already specifies:

- a human-owned claim ladder from authored source through compiled, audited,
  mapped, receipt-backed, and complete-local states;
- independent route targets and forbidden-import policies;
- source graphs, formal-target locks, proof-slice receipts, axiom audits, and
  route-level evidence schemas;
- separate generator, evaluator, reviewer, certifier, and promoter roles;
- isolated worktrees and a canonical `.lake` dependency cache;
- a durable `TaskGraph`; accepted ADR 0002 requires dynamic workflows to
  compile to it rather than becoming a second scheduler or state store;
- create-only candidate publication followed by validation and atomic
  promotion.

The next infrastructure should consolidate these contracts. It should not
create a parallel “Lean agent platform” with its own durable scheduler, archive,
or truth vocabulary.

### Failure modes the infrastructure must make impossible

| Failure | Current evidence or risk | Required invariant |
|---|---|---|
| Compile is mistaken for proof completion | Source, canonical prose, and certification artifacts differ | `interactive_check != certified_build != correspondence_review != promoted_claim` in the type and data model |
| Missing evidence silently passes | Route verifier skips absent manifests and local bundle is optional | Release policy declares required routes; an expected absence is a typed incomplete state and a fail-closed gate |
| Upstream reproduction and local port are conflated | Pinned Jin upstream and Harp local tree use different Lean/mathlib identities | Artifact channel is explicit: upstream reproduction, local source-faithful port, or local derivation; status never crosses channels implicitly |
| Shared cache becomes shared mutable state | Large `.lake` trees and parallel worktrees | Dependencies are content-identified/read-only; Harp-owned outputs are job/worktree-scoped; maintenance is a separate authorized workflow |
| Model-call crash creates ambiguous work | Attempt can be recorded running before provider effect; ADR defers full ambiguous-crash reconciliation | Durable outbox, effect idempotency key, lease/heartbeat, terminal receipt, and reconciliation before retry |
| “Independent” evaluators are the same model/provider | Blind contexts are stronger than provider separation | Independence is a vector over context, run, model family, provider, organization, method, and shared source/training exposure |
| Python and Rust acceptance drift | Route IDs, paths, axioms, and reuse rules appear in both implementations | One canonical boundary schema/policy plus shared positive/negative conformance fixtures or generated consumers |
| Failed attempts disappear into logs | Repeated environment and elaboration failures are not first-class mathematical evidence | Typed diagnostics are bound to task, source span, environment, attempted change, and resolution; retrieval requires exact compatibility |
| Three routes are counted as independent by name | Harp intentionally reuses selected LS support modules and all routes share foundations | Independence report computes shared source/import/constant cut sets and records targeted review of common roots |

## Provisional infrastructure contract

This section records requirements implied by the research. It is not the
approved module design.

### Core identities

1. **Environment identity** binds repository tree, Lean toolchain, Lake
   manifest, resolved dependency trees, platform-relevant build inputs,
   compiler/options policy, generated inputs, and allowed cache provenance.
2. **Proof-task identity** binds a human-owned theorem contract, exact Lean
   declaration/type or gap, source/provenance node, dependency cut, policy, and
   environment identity.
3. **Candidate identity** binds immutable source changes or proof artifacts to
   their parent task and attempt provenance.
4. **Observation identity** binds a structured Lean result to operation,
   environment, worker incarnation, input state/candidate, and resource
   envelope.
5. **Receipt identity** binds a fresh certification result and its complete
   closure. It is not a mutable flag on the candidate.
6. **Promotion identity** is a separate human decision over a complete evidence
   bundle digest.

### Minimal conceptual operations

The research repeatedly implies three operations even though their exact API
shape is undecided:

```text
prepare(EnvironmentSpec) -> EnvironmentRef | EnvironmentBlock

advance(EnvironmentRef, SnapshotRef, ProofAction)
  -> Observation | TypedFailure

certify(EnvironmentRef, CandidateRef, CertificationPolicy)
  -> ProofReceipt | CertificationFailure
```

`prepare` is pure with respect to dependency acquisition during proof work: it
resolves existing sealed inputs or reports a typed block. `advance` may use a
persistent worker and returns observations, never proof authority. `certify`
runs in a fresh isolated worker and can emit a receipt, but cannot perform
source correspondence, mathematical review, or human promotion.

At the executor-adapter layer, the same contract can be refined without
exposing a backend's raw numeric state handles:

```text
Handshake -> {protocol_schema, Lean_identity, capabilities, limits}
Open(EnvironmentRef, imports, ResourceLease) -> SessionRef
Apply(SessionRef, parent: StateRef?, action, deadline) -> ActionResult
Snapshot(StateRef) -> executor-local checkpoint + compatibility predicate
Close(SessionRef) -> ResourceReceipt
```

`ActionResult` must distinguish checked elaboration rejection, stale or
incompatible state, environment block, timeout, memory exhaustion, worker
crash, protocol error, sandbox violation, and operator cancellation. Retrying
an infrastructure failure is not a new mathematical attempt; resampling after
a checked rejection is.

### Macro and micro execution

Harp's durable `TaskGraph` is suitable for macro steps: prepare a proof task,
run one or more bounded search activities, materialize a candidate, schedule
blind evaluations, certify a route, collect reviews, and publish a bundle.

High-frequency tactic actions should not each become durable graph nodes. A
Lean activity adapter can own the micro-loop, stream structured events, and
checkpoint content-addressed candidate/source snapshots. Recovery resumes from
those stable snapshots or replays actions; it does not rely on an opaque
process token as durable truth.

### Two Lean lanes

| Interactive lane | Certification lane |
|---|---|
| Long-lived process, incremental edits, queries, tactics, goal inspection, semantic search | Fresh process/container or equivalent sealed worker, exact route target, deterministic envelope |
| Optimized for p50/p95 feedback latency | Optimized for reproducibility and complete evidence |
| May reuse process-local snapshots | Uses only sealed source/environment inputs |
| Emits observations and candidate checkpoints | Emits build/type/axiom/import/closure/log receipt |
| Crash recovery may replay from last checkpoint | Any ambiguity is a failed/incomplete certification, never implicit success |

### Proof/source graph

The coordination object must cover more than Lean imports. Nodes represent
informal obligations, exact Lean declarations, explicit gaps, derived lemmas,
terminal claims, and reviews. Edges distinguish logical dependency, Lean
import/constant use, source correspondence, route reuse, refinement, and
candidate lineage. This graph drives:

- task decomposition and affected-closure checks;
- accessible-premise and semantic retrieval;
- independent proof-route and shared-root analysis;
- source-to-formal coverage;
- reviewer context without search-policy leakage;
- claim-status projections and stale-prose detection.

### Artifact and evidence planes

The artifact plane stores immutable repository trees, environment manifests,
traces, attempts, candidates, diagnostics, reviews, and receipts. A mutable
name may point to the current accepted bundle, but the referents are create-only.

Publication is transactional: validate every referenced object, write the
complete route/evidence bundle, and then atomically advance the named claim
state with a human decision. A missing or changed referent invalidates the
transaction. Kernel acceptance, source correspondence, mathematical review,
and promotion remain separate gates.

### Observability and evaluation

Every stage should expose queue time, active time, cache/preflight result,
worker reuse, Lean diagnostic class, provider/model identity, calls/tokens,
candidate duplication, search branching, build jobs, invalidated modules,
review churn, receipt status, and stale projections. Metrics must be linked to
the same environment/task/candidate identities as the proof artifacts.

Research evaluation should use at least four split types:

- theorem-random for continuity with prior work;
- novel-premise or dependency-time splits for retrieval transfer;
- repository-held-out and chronological splits for adaptation claims;
- theorem-family/source-route splits for leakage-resistant Crouzeix-class work.

Infrastructure evaluation must separately measure setup/preflight, first
diagnostic, action latency, file/route validation, fresh certification,
recovery after injected crashes, cache duplication, and evidence-publication
correctness. Pass@k alone does not measure any of these.

### Benchmark identity and contamination

A benchmark name is not an identity. The minimum identity is the tuple of exact
task bytes, original-source record, formal language/toolchain/environment,
semantic-review status, correction or migration lineage, split policy,
disclosure/contamination class, resource budget, and verifier/comparator.

- Kernel validity and semantic validity remain separate. Formal Conjectures
  warns about misformalization, and Kimina's published corrections to miniF2F
  show that accepted benchmark statements can still be wrong targets.
- Public corpora and published proof bundles are trainable bytes. miniF2F,
  ProofNet, Lean Workbook, public prover solutions, and public Crouzeix material
  should default to `public-regression`, not `held-out`, unless the model/data
  lineage proves a cutoff.
- A Lean 3 to Lean 4 port, mathlib upgrade, or statement repair creates a new
  benchmark version and needs a migration/correction receipt.
- Attempt count, Lean time, wall/CPU/GPU time, tokens, retrieval calls,
  parallelism, cache warmth, and timeout policy are part of the score.
- When early proof disclosure would contaminate a hard benchmark, delayed or
  access-controlled release can preserve exact encrypted/hashed audit bytes and
  independent custody without treating secrecy as unverifiable authority.

## Reference workload and acceptance experiment

The highest-information first experiment is not another benchmark leaderboard.
It is one complete Crouzeix golden trace:

1. Freeze exact theorem contracts and the Jin, Lorist--Schwenninger, and Harp
   route/source graphs.
2. Resolve the approved Lean environment without hydrating dependencies.
3. Run bounded interactive work from isolated worktrees while recording typed
   observations and candidate checkpoints.
4. Inject crashes before and after provider effects, Lean effects, and state
   transitions; prove reconciliation has no duplicate accepted result or lost
   terminal receipt.
5. Certify each frozen route in a fresh worker with exact types, axioms,
   imports, closures, logs, and environment digest.
6. Run separate source-correspondence and mathematical/adversarial reviews.
7. Atomically publish the complete evidence bundles and an explicit human
   promotion or rejection.
8. Reproduce the same normalized receipts offline from the sealed inputs.

This trace would expose which retrieval, progress prediction, prover, session,
and cache mechanisms improve the actual proof lifecycle. It also closes a
current local evidence gap rather than creating a separate showcase.

Candidate metrics, to be calibrated rather than treated as promises:

| Dimension | Initial target |
|---|---|
| Preflight | Validate toolchain, manifest, cache ancestry/artifacts, worktree trust, disk, target, and output ownership without Lake in at most two seconds on the reference workstation |
| Interactive feedback | Leaf edit to first diagnostic p50 at most two seconds and p95 at most ten seconds after warm preparation |
| Warm affected-route acceptance | p50 at most 30 seconds and p95 at most 60 seconds, while reporting exactly what closure and jobs were checked |
| Reproducibility | At least 99% of sealed deterministic jobs reproduce normalized status/artifact digests; every exception is a typed nondeterminism incident |
| Orchestration safety | Crash injection at each persisted/effect boundary yields no duplicate accepted result, no lost receipt, and eventual reconciliation of every lease |
| Trust | Every promoted terminal and consequence declaration has exact type and axiom output bound to the fresh route build |
| Publication | No partial route bundles; every required absence fails closed; one human decision receipt names the complete bundle digest |
| Workspace/cache safety | Every writer has an exclusive worktree/path lease; no proof job hydrates dependencies; no unaccounted duplicate dependency cache above the chosen threshold |

## Decisions still required before architecture

The research leaves several hard-to-reverse choices. They should be answered
one at a time before comparing concrete module designs:

1. Is the first product a Harp-integrated Crouzeix-class proof runtime with
   portable contracts, or a standalone general-purpose LeanDojo successor?
2. Is the durable task unit a theorem/gap, a source-aware proof capsule, or an
   entire repository/route program?
3. Must the first release support training and reinforcement learning, or only
   emit versioned datasets/events that external trainers consume?
4. Which environments must be portable across machines, and which may remain
   machine-local caches with reproducible manifests?
5. What evaluator-independence tier is mandatory for a promoted mathematical
   claim?
6. Is source-to-formal correspondence a first-release gate or a later review
   layer?
7. Which state may be retained for self-improvement, for how long, and under
   what provenance/access policy?

## MISSING evidence and cautions

- No fresh Lean builds or axiom audits were run for the current Harp tree.
- Jin, Lorist--Schwenninger, and Harp route manifests, complete receipts,
  correspondence reviews, mathematical reviews, and mandatory local evidence
  rows are absent at the reference tree.
- A clean-room build of the pinned upstream Jin implementation remains
  unestablished in the inspected local evidence.
- Original AI-search transcripts, rejected candidates, and model/provider
  provenance for the Crouzeix proof efforts are unavailable.
- The local audit does not establish that two blind evaluator calls use
  independent providers or model families.
- Representative latency and resource distributions across Crouzeix edits,
  machines, and route closures have not been measured.
- Redistribution rights for some pinned upstream artifacts remain missing and
  must not be inferred.
- LeanDojo-v2's linked quantitative paper could not be extracted during this
  audit; no end-to-end equivalence with original LeanDojo was inferred.
- LeanProfiler's production overhead and variance are not established by a
  published study at the inspected revision.
- Benchmark pass rates cited by author projects use different Lean/mathlib
  versions, task repairs, sampling budgets, filtering rules, and validation
  servers. They are not normalized head-to-head rankings.

## Adjacent-effort primary artifact ledger

This ledger fixes the 38-artifact comparison corpus. Repository entries are
immutable snapshots; paper versions and publication records are named
explicitly. OpenReview pages are dated observations where no conventional
release tag exists.

| ID | System | Frozen primary artifact |
|---|---|---|
| A01 | Lean 4, Lake, and official server | [Lean `v4.33.0`, commit `d8b1897`](https://github.com/leanprover/lean4/tree/d8b18978322de05a8f3dba51ef03cf5461676c17) |
| A02 | elan | [elan `v4.2.3`, commit `b6cec7e`](https://github.com/leanprover/elan/tree/b6cec7e10fe4965a605aaf60d1cb4a5837f0462b) |
| A03 | mathlib4 cache | [mathlib `v4.32.1`, commit `520045a`](https://github.com/leanprover-community/mathlib4/tree/520045ab14e26149ee970e2e617ca04b09bde5d6) |
| A04 | Reservoir | [commit `4bde19c`](https://github.com/leanprover/reservoir/tree/4bde19c2dba9244d97336768ce8a97aaddbd2b72) |
| A05 | community REPL | [commit `5d5c49d`](https://github.com/leanprover-community/repl/tree/5d5c49d13dfc0c1d2df43a27c3e56e02ad81b9c3) |
| A06 | Pantograph | [`dev` commit `d704b85`](https://github.com/leanprover/Pantograph/tree/d704b851542b1d2caf1287f65c49f5011f687c05) |
| A07 | LeanInteract | [commit `976edd7`](https://github.com/augustepoiroux/LeanInteract/tree/976edd7d38a99e1ea4c2dfabeb8ad98baffca3c8) |
| A08 | Lean4Kit description | [ATG4CI OpenReview record `98w016SKJg`, observed 2026-08-23](https://openreview.net/forum?id=98w016SKJg) |
| A09 | Kimina Lean Server | [commit `fb2393d`](https://github.com/project-numina/kimina-lean-server/tree/fb2393de3461db35eda4c714e3fd21187e92ec90) |
| A10 | LeanDojo-v2 | [commit `baed5ea`](https://github.com/lean-dojo/LeanDojo-v2/tree/baed5eae6e87a65a446d9f54af07aab2154e7599) |
| A11 | LeanDojo-v2 paper | [content-hashed OpenReview PDF](https://openreview.net/pdf/d6fa8f8d79609b804b41e728eecd4cc1e9597a0b.pdf) |
| A12 | doc-gen4 | [commit `97d4ecd`](https://github.com/leanprover/doc-gen4/tree/97d4ecdfc8e09e7f511724c25e303d448de6a3db) |
| A13 | LeanSearch v2 | [commit `94f4888`](https://github.com/frenzymath/LeanSearch-v2/tree/94f4888cbaf9f4322535755f86cbac690ec18080) |
| A14 | LeanSearch v2 paper | [arXiv `2605.13137v2`](https://arxiv.org/abs/2605.13137v2) |
| A15 | LeanSearchClient | [commit `ba67e21`](https://github.com/leanprover-community/LeanSearchClient/tree/ba67e212be1197b84c1f1f6299488a10a3002713) |
| A16 | Lean Workbook dataset | [Hugging Face revision `2e066e3`](https://huggingface.co/datasets/internlm/Lean-Workbook/tree/2e066e310b2c6d2c27616927ae131f82901c8f1c) |
| A17 | Lean Workbook paper | [NeurIPS 2024 Datasets and Benchmarks record](https://papers.nips.cc/paper_files/paper/2024/hash/bf236666a2cc5f3ae05d2e08485efc4c-Abstract-Datasets_and_Benchmarks_Track.html) |
| A18 | ReProver | [commit `fd6d99c`](https://github.com/lean-dojo/ReProver/tree/fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa) |
| A19 | LeanDojo/ReProver paper | [NeurIPS 2023 Datasets and Benchmarks record](https://proceedings.neurips.cc/paper_files/paper/2023/hash/4441469427094f8873d0fecb0c4e1cee-Abstract-Datasets_and_Benchmarks.html) |
| A20 | LeanCopilot | [commit `6073803`](https://github.com/lean-dojo/LeanCopilot/tree/60738032a1398a4e83524107424a11322a753609) |
| A21 | LeanCopilot paper | [PMLR 288](https://proceedings.mlr.press/v288/song25a.html) |
| A22 | DeepSeek-Prover-V2 | [commit `e598a57`](https://github.com/deepseek-ai/DeepSeek-Prover-V2/tree/e598a57ea3284997d4a2a168a069fdd5064afbc8) |
| A23 | DeepSeek-Prover-V2 paper | [arXiv `2504.21801v2`](https://arxiv.org/abs/2504.21801v2) |
| A24 | Goedel-Prover-V2 | [commit `2e9036e`](https://github.com/Goedel-LM/Goedel-Prover-V2/tree/2e9036e118464aa96a8bebaf9f5b9d091aa3585c) |
| A25 | Goedel-Prover-V2 paper | [arXiv `2508.03613v1`](https://arxiv.org/abs/2508.03613v1) |
| A26 | Kimina-Prover Preview | [commit `7abd61a`](https://github.com/MoonshotAI/Kimina-Prover-Preview/tree/7abd61a5d9861bf5c15b195c216c1bb233ac32e4) |
| A27 | Kimina-Prover paper | [arXiv `2504.11354v1`](https://arxiv.org/abs/2504.11354v1) |
| A28 | lean-eval | [commit `b91d475`](https://github.com/leanprover/lean-eval/tree/b91d4757aa0d7776c02540c9089df54fa0d0658a) |
| A29 | lean-eval submissions | [commit `90383f7`](https://github.com/leanprover/lean-eval-submissions/tree/90383f788caa07808e8a36ee457357b11bb02964) |
| A30 | miniF2F | [commit `e4f1130`](https://github.com/facebookresearch/miniF2F/tree/e4f113090ad82d64f8ce064d2f55b613a9b6bded) |
| A31 | miniF2F paper | [ICLR 2022 OpenReview record `9ZPegFuFTFv`](https://openreview.net/forum?id=9ZPegFuFTFv) |
| A32 | PutnamBench | [commit `dfb0a47`](https://github.com/trishullab/PutnamBench/tree/dfb0a47a1c1ec3a10f2a9acfdf41a2043920f33c) |
| A33 | PutnamBench paper | [NeurIPS 2024 Datasets and Benchmarks record](https://proceedings.neurips.cc/paper_files/paper/2024/hash/1582eaf9e0cf349e1e5a6ee453100aa1-Abstract-Datasets_and_Benchmarks_Track.html) |
| A34 | Formal Conjectures | [commit `488aade`](https://github.com/google-deepmind/formal-conjectures/tree/488aade228ec37880b8fec178c173c07d279bb53) |
| A35 | Formal Conjectures paper | [arXiv `2605.13171v1`](https://arxiv.org/abs/2605.13171v1) |
| A36 | ProofNet | [commit `509ad79`](https://github.com/zhangir-azerbayev/ProofNet/tree/509ad79710ed4f46ff5c282ed5640c1aa9ac3f30) |
| A37 | ProofNet paper | [arXiv `2302.12433v1`](https://arxiv.org/abs/2302.12433v1) |
| A38 | Public Crouzeix snapshot | [commit `9df0783`](https://github.com/jinshanmu/CrouzeixConjecture/tree/9df07838327b988e3924453daa29c8cd726d34b0) |

## Bottom line

LeanDojo proved that commit-aware extraction plus programmatic, kernel-checked
interaction can support a serious learning ecosystem. Its surrounding projects
then exposed the missing pieces: dynamic knowledge, progress prediction,
replaceable providers, representation bridges, trust ledgers, cross-system
evaluation, and operational profiling. Adjacent projects improve each of those
layers, but they do not converge on one environment or session protocol.

For the next generation, the central abstraction should be a sealed,
source-aware proof program whose fast interactive actions are observational and
whose accepted claims are backed by independently reproducible evidence. Harp
already has the durable orchestration and claim-separation vocabulary needed to
host that system. The next design should deepen those modules and standardize
the Lean boundary, not introduce another monolithic prover stack or another
durable workflow engine.

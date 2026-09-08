# Production Lean research: FLT, Prove2Me, LeanDojo, and Harp

Date: 2026-09-07. Status: evidence-backed research proposal, not an implemented
or accepted specification. This extends the prior research; it does not replace
the existing candidate design or alter the Crouzeix evidence.

Subsequent user direction: the
[verifier-first curriculum](verifier-first-curriculum-2026-09-07.md) supersedes
this note's Crouzeix-first delivery sequence. Start with Cauchy–Schwarz and
then source-anchored Spivak/Lax/Bishop workloads. The larger architecture below
remains a research hypothesis to test against those small workloads.

## Recommendation

Build a theorem-oriented research system on Harp's durable executor. Its core
objects should be immutable mathematical statements and checked reductions.
Proof attempts, tactic states, models, and worker processes are replaceable
ways to construct evidence for those objects. Final acceptance must consume an
assembled proof against a separately trusted target.

Combine Prove2Me's campaign decomposition with LeanDojo's local retrieval and
search. Use independently specified certification and publication contracts to
connect them. The best initial product is a bounded local research workbench
that can recover, explain its unfinished obligations, and export a reproducible
result. Internet-scale collaboration and online model training can follow once
that product passes adversarial acceptance tests.

## Evidence and scope

The supplied [HN item](https://news.ycombinator.com/item?id=49568506) was resolved
through the [official item API](https://hacker-news.firebaseio.com/v0/item/49568506.json)
after the HTML fetch failed. It links Anthropic's report and Kevin Buzzard's
response. Comments were not treated as verification evidence.

This pass inspected those two reports, Prove2Me paper v2 and public workflow
docs, six pinned FLT repository artifacts, the LeanDojo organization inventory,
selected LeanDojo primary sources, and current Harp execution contracts. The
previous [research extension](research-extension-2026-09-07.md) supplies the
bounded ATLAS, LeanEval, Formal Conjectures, and Lean interaction comparisons.
No accounts, API submissions, external services, installs, builds, or proof
replays were performed. Public agent instructions were read as documentation,
not adopted as authority to operate the platform.

The prior untracked Harp note at
`docs/workstream/crouzeix-proof-reproduction/formalizing-flt-prove2me-adoption.md`
was a useful hypothesis and remains unchanged. Its factual claims were checked
against their primary owners rather than accepted from that note.

Pinned FLT revision: `aa2d8b34692b16c70f699536de0d8e75b9a3e9ef`.
Prove2Me paper: `2608.28433v2`. Mutable protocol observations were made on this
date; freshly fetched raw content SHA-256 values were:

| Artifact | SHA-256 |
|---|---|
| `https://prove2.me/skill.md`, version 0.9.8 | `31c845c2c5e150d330da50fa234cc44793314d4a3f0e4420f4c8b4e5ef6118a2` |
| `https://prove2.me/references/prove.md` | `615f3a7e7060e028fc2f87c1d7cbbf0ba7cddcb4c06ee9a585b1bab89d4050a1` |
| `https://prove2.me/references/mission_captain.md` | `5ca9d0102cb1341a44129d19b269d1cdc37156f33908a6ba5f551f57c8a4e1a4` |

These identify fetched content, not archived captures or a verified server
deployment. Backend enforcement beyond the inspected artifacts remains open.

## What the FLT evidence actually says

Anthropic reports an 11-day campaign using dozens of agents and about six
billion output tokens, producing roughly 30,300 theorem proofs, with roughly
29,500 used in the final result. It credits the change to Prove2Me with improved
coordination after earlier attempts lost project state. Its three stated
mechanisms are a theorem DAG, separate statement/proof files, and searchable
natural-language theorem descriptions. This is a successful case study, not a
controlled ablation establishing the marginal effect of each mechanism, model,
or compute budget. The work formalizes existing mathematics, rather than
discovering FLT. [Anthropic report](https://www.anthropic.com/research/formalizing-fermats-last-theorem)

Buzzard reports independently compiling the repository and running comparator.
He distinguishes this result from producing reusable Mathlib infrastructure and
a human-readable account of the modern proof. That distinction should become
a product requirement: verified output and maintainable mathematical knowledge
are separate deliverables.
[Buzzard's account](https://xenaproject.wordpress.com/2026/09/04/flt-anthropic-has-beaten-me-to-it/)

The source supports more than a dashboard status. The
[terminal theorem module](https://github.com/anthropics/fermats-last-theorem/blob/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef/Theorems/Thm_fermat_last_theorem.lean)
imports its proof module and gives the theorem a body. The
[solution module](https://github.com/anthropics/fermats-last-theorem/blob/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef/P2M/Sol/S_fermat_last_theorem.lean)
uses a lower-level FLT result. [FinalCheck](https://github.com/anthropics/fermats-last-theorem/blob/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef/FinalCheck.lean)
guards the terminal theorem's axiom list and derives Mathlib's
`FermatLastTheorem`. Thus the endpoint is an assembled theorem chain, not merely
a set of conditional sketches. The complete chain and assembly helper were
not independently inspected or executed in this pass.

The [README](https://github.com/anthropics/fermats-last-theorem/blob/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef/README.md)
reports a source build, comparator replay, and acceptance by a patched nanoda.
Those remain reported verdicts here. The inspected
[comparator script](https://github.com/anthropics/fermats-last-theorem/blob/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef/verification/comparator/run.sh)
uses existing compiled artifacts and can reuse checker directories without
asserting exact revisions; it also has a fallback without sandbox enforcement.
These are replay-harness limitations, not evidence invalidating the theorem.
Harp should record full checker/build identities and fail closed when its
required containment is absent.

The [proof walkthrough](https://github.com/anthropics/fermats-last-theorem/blob/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef/PROOF-PATH.md)
also distinguishes versions of classical results actually needed by the proof
and discloses an unused hypothesis. Import edges therefore help navigation but
cannot substitute for semantic dependency and hypothesis-use analysis.

## What to take from Prove2Me

The paper's central contract separates immutable statements from multiple
proofs. A checked sketch establishes a goal conditional on named children;
closing children permits composition. Its human audit surface is the mission
goal, defining concepts, and selected milestones. A blind read-back agent sees
Lean and dependent definitions without the original prose, letting a human
compare the two accounts. This is a practical aid to semantic review, not a
proof of faithfulness. [Paper, Sections 3–4](https://arxiv.org/html/2608.28433v2)

The [submission protocol](https://prove2.me/references/prove.md) distinguishes
accepted sketches with open imports from accepted proofs, and fixes each
submission to its target's environment. The
[solver guidance](https://prove2.me/references/mission_solver.md) starts from
milestones and open leaves, asks solvers to inspect failed attempts and
rejected milestone histories, and encourages reusable reductions. These are
good planning inputs; the inspected documents do not establish an optimal
queueing policy, transactional leases, exactly-once publication, or recovery
guarantees.

The [captain guidance](https://prove2.me/references/mission_captain.md) says
milestone order is advisory and has no dependency tracking of its own. It also
documents deletion cascading to milestone history. Harp needs explicit
dependencies and append-only correction records for durable research. Adopt
the theorem interface and audit discipline, while specifying those operational
contracts independently.

## What LeanDojo contributes

The fresh organization API returned 15 repositories, matching the prior census.
This is a refreshed inventory, not a new implementation audit of every project.
The following HEADs were re-resolved; they match the prior research pins.

| Work | Production role to evaluate | Boundary |
|---|---|---|
| [LeanDojo-v2](https://github.com/lean-dojo/LeanDojo-v2/tree/baed5eae6e87a65a446d9f54af07aab2154e7599) | Tracing, datasets, retrieval/search integration | Candidate machinery; do not inherit it as scheduler or certifier |
| [ReProver](https://github.com/lean-dojo/ReProver/tree/fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa) | Premise retrieval and tactic search | Filter suggestions to the attempt's accessible environment |
| [LeanProgress](https://github.com/lean-dojo/LeanProgress/tree/328ca46ede8ab9e3506805ba7658859af74aa7be) | Learned proof-progress estimates | A search heuristic, never a completion verdict |
| [LeanAgent](https://github.com/lean-dojo/LeanAgent/tree/13f63bd8ed42057dbf909791ba39ea6f8759c690) | Growing theorem database and continual-learning experiments | Freeze dataset/model versions per run and test forgetting/leakage offline |
| [LeanCopilot](https://github.com/lean-dojo/LeanCopilot/tree/60738032a1398a4e83524107424a11322a753609) | In-editor suggestions and human collaboration | Shares candidate contracts; has no promotion authority |
| [LeanProfiler](https://github.com/lean-dojo/LeanProfiler/tree/ecbcb4bef3bbeb7ea8c2e6e48d196b49b75192f4) | Named runtime spans and timing reports | Complement Lean elaboration profiling and host/process metrics |

Natural-language retrieval should retrieve candidates, followed by exact
environment and type checks. A theorem described as equivalent is not the same
theorem identity. Any equivalence used for reuse needs a checked bridge.

ATLAS contributes distinct measures of source-target coverage, proof integrity,
and code quality; LeanEval contributes trusted-challenge comparison and checker
policy; Formal Conjectures contributes immutable benchmark revisions. Their
sources and limits are in the [September extension](research-extension-2026-09-07.md).
Together they cover different layers rather than competing to own the product.

## First-principles model

The consumer wants an answer to a precise mathematical question, with evidence
they can inspect and reproduce. A useful objective is accepted, source-faithful
research outcomes per elapsed time, compute cost, and human review effort.
Generated lines, tactic successes, and theorem counts are diagnostic metrics.

Six object families are sufficient to start:

| Object | Identity and authority |
|---|---|
| Mission revision | Intended question, exact sources, approved roots/definitions/milestones, budget and claim scope |
| Statement revision | Lean type, universe parameters, referenced definition closure, environment; immutable after admission |
| Proof candidate or reduction | Target revision, sealed source, actual assumptions/dependencies, parent attempt and checker result |
| Attempt | Leased work, frozen model/tools/context, resource limits, actions, outputs and typed failure |
| Certification | Selected finite proof closure, assembled bytes, target comparison, axiom/checker results and environment |
| Review/promotion event | What a human or reviewer attests, exact inputs, current validity and explicit supersession |

Use stable nominal identifiers plus exact content fingerprints. A fingerprint
is an integrity check, not a decision procedure for mathematical equivalence.
Its serialization and extraction version must itself be pinned.

Separate these relationships:

1. Source correspondence: which mathematical text a statement represents.
2. Candidate proof graph: alternative reductions and the children each needs.
3. Selected semantic closure: the actual declarations/definitions used by the
   final proof, extracted from Lean rather than inferred from imports.
4. Execution graph: which worker activities depend on which artifacts.

These graphs overlap but are not interchangeable. One proof attempt can work
on several local tactics; one statement can have many attempts; one supporting
lemma can serve several routes.

### A proof search campaign is an AND/OR graph

Suppose T has two candidate reductions: A and B together imply T, or C implies
T. A plain union of dependencies incorrectly requires A, B, and C. Store each
reduction as its own node: its children are AND requirements; reductions for a
statement are OR alternatives. A direct proof is an alternative with no open
children. This is our proposed data model, not a claim about Prove2Me's backend.

Closure is the least fixed point under a named environment, route, and
axiom/checker policy, seeded by admissible direct proofs and explicitly trusted
foundation results. A reduction closes its target only
after all children have closure witnesses under compatible identities and
policy. An ungrounded cycle such as A depending on B and B on A never closes.
Store rejected or cyclic proposals for diagnosis if useful; the selected final
certificate must carry a finite acyclic witness. Adding alternatives must not
invalidate an already sound witness.

Cache each closure witness against that policy identity, the selected
reduction, and its child witnesses. A statement closed for one route can remain
unavailable for another; never use a global cached closed flag to authorize
reuse across routes.

Distinguish at least `unproved`, `reduction-checked`, `closure-ready`, and
`certified` on the proof axis. Keep semantic review, claim promotion, and
statement retirement on separate axes. A refutation is checked evidence for
the negation of a statement, not an infrastructure failure. A disproved helper
blocks that reduction and prompts replanning; it need not end the mission.

### Statement/proof separation is a build optimization with a proof obligation

During exploration, a reduction may be checked under explicitly registered
child assumptions. Record exactly which obligations remain, including hidden
transitive references. Arbitrary `sorryAx` cannot be interpreted as a safe list
of named obligations. A dedicated reduction checker must account for every
assumption through explicit binders or controlled statement stubs.

For final acceptance, select proofs, discharge the assumptions, materialize
the assembled source/export, compare with the trusted target and definition
closure, and check its transitive axioms in a fresh environment. A scheduler's
closure calculation alone cannot replace this check.

Freeze statement interfaces so a new proof candidate need not invalidate
unrelated interactive work. Give every candidate a new identity; old receipts
remain valid for their original sealed bytes. Recompute the selected closure
when its proof selection changes. Withdrawal or a checker defect requires an
explicit validity event. Definition/type changes create new revisions and make
dependent statements, reviews, retrieval entries, and compiled artifacts stale
for the new revision; historical records retain their original scope.
Any finer reuse requires a verified compatibility rule, not a textual guess.

## Planning and scheduling policy

Planning proposes mathematics; scheduling allocates resources. The model can
suggest a decomposition, but only a checked reduction adds logical progress.
A process-safe dispatcher validates and journals the next bounded work batch.

Start with an explainable policy before learning a scheduler:

1. Find work contributing to approved roots or milestones. Show why each task
   matters; separate speculative library work into an explicit bounded budget.
2. Retrieve reusable results and previous failed approaches before dispatch.
3. Prefer bottlenecks whose closure unlocks valuable selected paths; estimate
   cost from observed similar attempts, with uncertainty visible.
4. Reserve verification capacity. Throttle generation when candidate backlog,
   memory pressure, or review delay exceeds the configured bound.
5. Lease each attempt with a generation/fencing token. Retries may compute twice;
   only the current admissible publication can update accepted state.
6. Permit a small explicit portfolio on difficult bottlenecks. Vary strategies
   deliberately, charge every attempt, and cancel remaining work only after
   the winner has checked evidence.
7. Replan after bounded repeated failure, helper refutation, statement change,
   or exhausted budget. Persist the reason and alternative, not just a new prompt.

A heuristic can rank expected approved-root progress per resource cost, but
these probabilities are initially estimates. Do not present them as calibrated
or multiply dependent success probabilities as though they were independent.
Use age/fairness rules so difficult necessary branches do not starve behind
easy theorem counts. Compare alternative policies under the same total budget
and acceptance rules before changing defaults.

The long-lived state should be structured artifacts and events. Reconstruct a
short worker context from the current goal, exact definitions, selected
dependencies, useful premises, and relevant failures. A transcript is optional
supporting evidence, not the plan or recovery record.

### Research needs a second loop around formalization

FLT demonstrates translation and assembly of known mathematics. Automated
research additionally needs to choose questions and revise conjectures. Keep
that loop explicit: literature and numerical exploration propose a conjecture;
statement review fixes its meaning; proof search or a checked counterexample
tests it; failures may propose a successor conjecture with changed hypotheses.
Each successor retains its relationship to the original question. Numerical
agreement and an unsuccessful counterexample search are observations, not
proofs. A finite test of matrices cannot by itself settle an unrestricted
operator theorem.

For Crouzeix, track finite-dimensional bounds, Hilbert-space consequences,
regularity conditions, and route reuse as explicit scope. If an agent adds a
compactness assumption or proves only a restricted class, publish that useful
partial result under its actual statement and keep the original obligation
open. An impossible helper can be refuted and replaced without changing the
mission's goal. Compare competing informal arguments through their explicit
formal obligations instead of letting one persuasive narrative become state.

Run learning as a third, slower loop over attributed outcomes. Start with
searchable failure records and reusable checked lemmas. Later train retrieval
or search policies on fixed snapshots, using held-out theorem families,
repositories, and time boundaries to detect leakage. Deploy a model version
only after evaluation; never change the model or reward definition inside an
attempt retrospectively. A successfully compiled but unfaithful statement is
not a positive example for source-faithful formalization.

## Runtime and certification

Use fast interactive workers pooled by exact environment and compatible import
header. Worker handles include generation so stale IDs cannot survive restart.
Recover from sealed commands/source; validate reconstructed observations.
Pantograph, LeanInteract, and REPL require different capability tests, as the
prior extension documents. Keep tactic branching inside an attempt to avoid
one durable scheduler event per tactic.

Certification uses a frozen source closure, explicitly provisioned dependency
cache, fresh worker, resource bounds, and controlled filesystem/network access.
It must account for all descendant processes, seal build outputs after writers
stop, and export/check those exact bytes. Record full checker revisions and
binary hashes, runtime/platform, patches, axiom policy, and all outcomes.
Different kernel implementations add evidence; common vulnerabilities remain
possible. Advisory-driven replay is an append-only lifecycle event.

Publish immutable bundles atomically with an idempotency key. A crash after
artifact creation but before the journal update must reconcile the existing
artifact, not invent another success. Reviewer identities bind exact frozen
inputs; mutation requires a new candidate and review.

Production output includes formal proof, source correspondence, a readable
explanation, dependency/reuse view, limits, and replay instructions. Internal
auxiliary lemmas need not all receive human prose audits for a closed root to
be logically sound. If a lemma becomes a public named mathematical claim, its
statement and definitions need their own semantic review. This keeps audit
effort proportional to the published interface without silently endorsing
every generated lemma's name or description.

After first certification, a separate maintenance pass can minimize imports,
compress duplicated proof material, align definitions with Mathlib, and improve
exposition. It produces a new checked candidate. Preserve the original proof
as a replay anchor rather than combining discovery, certification, and library
cleanup into one moving target.

## Current Harp integration, not the August assumption

At primary commit `2e5d57da`, `crates/harp-engine/src/execution_plan.rs` already
defines `ExecutionPlan`, bundling `TaskGraph`, graph policy, and projection
policy. It still wraps the agent-oriented graph. Runtime `ActivitySpec` and
recovery records still use thread/turn identities. This needs a targeted
compatibility analysis, not a second executor or a plan to introduce an
abstraction that already exists.

`crates/harp-contracts/src/graph.rs` bounds each TaskGraph at 64 nodes. An FLT-sized
theorem store must therefore remain separate from an execution batch. Compile
bounded frontier batches into the existing execution boundary, recording the
mission revision, frontier inputs, and batch decisions. The domain planner is
a client of Engine; Engine remains sole authority for leases, attempts,
recovery, resource allocation, and durable execution.

Crouzeix already supplies route-specific statements, source maps, receipts,
semantic audits, and the completed six-row bundle. Project those into the new
objects without rewriting evidence. Keep Jin, Lorist–Schwenninger, and Harp
route policies separate even when nodes are shared. Preserve Harp's derived
status and eleven approved lower-level LS dependencies. Route diversity is not
automatically independent mathematical evidence.

## Alternatives and recommended sequence

Using Prove2Me as the whole system buys its collaboration surface but leaves
Harp's local recovery, evidence, environment, and publication contracts to be
reconciled. Using LeanDojo-v2 as the whole system buys search/training machinery
but leaves campaign intent and final promotion unspecified. Building every
layer ourselves would duplicate useful interaction and retrieval work.
The recommended approach owns the narrow mathematical/evidence contracts in
Harp and evaluates replaceable integrations behind them.

| Stage | Deliverable | Evidence required to advance |
|---|---|---|
| 1. Model existing results | Read-only Crouzeix projection and frontier/reuse report | All six existing rows and route limits preserved; no new mathematical claims |
| 2. Prove composition semantics | Small synthetic reductions, alternatives, and assembly | Reject ungrounded cycles, hidden assumptions, changed definitions, and stale witnesses; accept a valid alternative without demanding every branch |
| 3. Make execution recoverable | One synthetic proof through Engine and fresh certification | Crash injection at each publication boundary; no partial acceptance, stale lease publication, or surviving writer |
| 4. Demonstrate useful automation | A bounded Crouzeix-related obligation with audited statement | Reproducible end-to-end result and full cost/latency/review accounting |
| 5. Improve efficiency | Matched retrieval/adapter/scheduling comparisons | Better accepted target throughput without degraded correctness or inflated scope |
| 6. Broaden collaboration | Portable bundle import/export and optional Prove2Me bridge | Foreign results pass local identity, license, closure, and checker policy; external publication separately authorized |

Evaluate recovery correctness, rejection of invalid proofs, and statement
faithfulness before claiming throughput gains. Count target outcomes with their
original denominators, failures and abandoned branches included. Report p50/p95
startup/check/recovery time, peak memory, verification queue age, model and
tool cost, useful lemma reuse, and human review minutes. Do not choose arbitrary
latency SLOs before the first representative workload is measured.

The next concrete research artifact should be the Crouzeix projection plus the
synthetic AND/OR closure test corpus. That will expose whether the proposed
contracts actually support the workflow which made the FLT campaign effective,
while preserving Harp's stronger evidence and recovery requirements.

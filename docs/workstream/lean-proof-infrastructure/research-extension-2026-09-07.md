# Lean infrastructure research extension — 2026-09-07

Status: research findings and proposed experiments; no architecture acceptance,
implementation, toolchain migration, or fresh proof certification is implied.

## Scope and method

This extends the August research with five interaction systems (LeanDojo-v2,
Pantograph, LeanInteract, community REPL, Kimina Lean Server), LeanEval, Formal
Conjectures, Lean checker guidance, and ATLAS/its companion paper. ATLAS was
explicitly added to investigate repository-scale formalization and statement
faithfulness. Searches were discovery only; claims below use first-party
sources. Inspection stayed within those projects and one-hop primary links.
The interaction review was capped at three artifacts per project.

Git default refs were resolved with `git ls-remote` on 2026-09-07. Raw files
used below from ATLAS, LeanEval, and Formal Conjectures returned HTTP 200 at
their pinned commits. Lean's release page failed through the browser tool but
returned HTTP 200 through curl; it remains a dated documentation observation.
No upstream packages were installed or run. Reported results, source inspection,
local historical evidence, and proposed experiments are distinguished below.

Local baseline: primary Harp commit
`2e5d57da`, read on 2026-09-07. The existing infrastructure design worktree is
still at `e5b45c22` with earlier uncommitted review repairs. This note does not
change that design.

## What changes the research agenda

### 1. The Crouzeix prerequisite is now evidence we can replay

Local committed records show CPFR-087 through CPFR-091 completed. The
Phase-8 retrospective is referenced by repository path
`docs/workstream/crouzeix-proof-reproduction/retrospective-003.md` at the
primary baseline above; this design worktree predates its landing. The exact
`program-verification-003.json` bytes have SHA-256
`f3fbf06e69717b8b15840e182bbc62c1c240b452177ddc2a751103bafc3a2280`.
It records a 1,015-second serial gate, three complete-local routes, allowed
axioms, route reuse, and an unchanged dependency-cache digest. Its scope is
the frozen candidate, with later closeout tracked separately.

The retrospective records a later 604-second post-cleanup full gate with
28 seconds attributed to the Lean build. That quotient is about 4.6%; it is
a historical wall-time comparison, not a CPU profile or an estimate that all
remaining time is removable overhead. The record does not justify treating
8,775 Lake jobs as 8,775 freshly compiled modules.

Researcher assessment: measure the full acceptance path before optimizing
tactic throughput. Candidate generation, compilation, audit, review, metadata
reconciliation, and publication need separate timing spans. The old design's
instruction to finish CPFR-087–091 is stale; its first experiment can now use
the completed evidence as an immutable expected result.

### 2. Statement identity must include the definitions that give it meaning

LeanEval's pinned security model describes a trusted Challenge, solver-owned
Submission, and generated Solution bridge. Comparator compares reachable
constants and axioms between separately built environments. Definition holes
are an explicit exception: their types must match, but authors must provide
theorems sufficiently constraining their values. This is a useful example of
why a matching theorem name or printed type is insufficient.
[Security model](https://github.com/leanprover/lean-eval/blob/6b4b87b672f5301f24983a12fda65dac608453ce/SECURITY.md)

Researcher assessment: bind the trusted statement and its semantic definition
closure before dispatching a solver. A candidate that changes a norm,
numerical-range definition, domain, or hypothesis must start a new statement
revision. Mechanical identity comparison verifies agreement with the approved
formal target; human mathematical review still establishes correspondence
with Crouzeix's intended mathematics.

### 3. Certification requires a lifecycle for processes and checker versions

The same LeanEval security document acknowledges unresolved writable-build
output concerns from surviving descendants at its pinned comparator version;
it does not claim a demonstrated exploit. Its
[WorkspaceTest implementation](https://github.com/leanprover/lean-eval/blob/6b4b87b672f5301f24983a12fda65dac608453ce/templates/WorkspaceTest.lean)
explicitly forces nanoda replay when invoking comparator. These are source
observations, not a locally reproduced security test.

Lean's [4.33.1 release notes](https://lean-lang.org/doc/reference/stable/releases/v4.33.1/)
describe runtime and kernel soundness repairs, including one defect also
accepted by nanoda. They recommend upgrading earlier versions. The Crouzeix
record uses Lean 4.32.1. This establishes the need for an applicability review;
it does not establish that a Crouzeix proof triggers a defect.

Researcher assessment: require build-process quiescence and seal outputs before
export/checking. Record each checker binary, runtime, export format, policy,
and outcome separately. A second checker adds evidence but does not replace
source correspondence or eliminate shared defects. Preserve historical
receipts when a policy changes; append an applicability decision and any
required replay. Toolchain/cache maintenance remains a separate operation.

### 4. Formalization quality needs distinct denominators

ATLAS v1 reports 42,837 proved declarations out of 46,203, but 2,855 formalized
source targets out of 4,007. These are different populations: 92.7% declaration
proof coverage does not mean 92.7% of textbook statements were faithfully
formalized. The README provides per-book totals and describes source targets
and automated evaluation reports. These are author-reported numbers, not a
reproduction. Its current root says v2 is in development and preserves v1
separately; v1 retains its own license.
[Pinned v1 report](https://github.com/facebookresearch/atlas-lean/blob/e8b31c5cb0bec89b487ce33fe525a2c0b0f8b9c6/v1/README.md),
[version boundary](https://github.com/facebookresearch/atlas-lean/tree/e8b31c5cb0bec89b487ce33fe525a2c0b0f8b9c6)

The companion paper combines dependency inspection with LLM judgments of
faithfulness, integrity, and code quality. It reports repeated failed
strategies, misleading foundational definitions, and coordination failures;
its parallelism ablation is evidence for testing speculative workers, not a
universal worker-count recommendation. Paper contract: informal textbook
targets are the inputs; usable formalized targets are the consumer outcome.
Its hypothesis is that coordinated agents can build libraries at scale.
Reported evidence is the library and ablations; unresolved transfer factors
include models, budgets, task mix, judge reliability, and different acceptance
policies. Harp assessment: import the evaluation distinctions and test the
scheduling idea under matched conditions.
[Paper v1, Sections 3–5 and Appendix D](https://arxiv.org/html/2605.29955v1)

For Harp, report separate counts for intended source targets, matched formal
statements, kernel-accepted proofs, accepted semantic reviews, complete
bundles, and promoted claims. Missing matches count as missing targets. LLM
reviews remain attributed judgments with rationale and review version.

### 5. Benchmark correction is part of environment identity

Formal Conjectures now documents immutable benchmark tags separating problem
set version from Lean version. Adding/removing problems or correcting a
misformalization advances the benchmark version. The project also explicitly
recognizes formalization inaccuracies and human review as a mitigation.
[Pinned versioning policy](https://github.com/google-deepmind/formal-conjectures/blob/2c817e975be7a95478b72a8429155ca568e1a3de/README.md)

Researcher assessment: a Harp comparison needs both problem revision and
environment identity. Corrected statements cannot inherit old success labels.
Keep original results addressable and publish a new benchmark membership set.

## Interaction systems: concrete recovery contracts

Ten successful artifacts were inspected across these five projects. Four HEADs
were unchanged from August 25; Pantograph advanced. No change-by-change
Pantograph audit was performed, so the observations do not date features.

| Project | Observed commit | Recovery or execution constraint | Harp consequence (inference) |
|---|---|---|---|
| LeanDojo-v2 | `baed5eae6e87a65a446d9f54af07aab2154e7599` | Prover records a search stack and graph; step and trial limits are embedded in code | Make budgets explicit and replay extracted programs before acceptance |
| Pantograph, dev | `92d4818a4b343d7be293731e03359a19e8082626` | Saved goals omit the environment; timeout is cooperative | Bind snapshots to an exact environment and enforce external resource limits |
| LeanInteract | `976edd7d38a99e1ea4c2dfabeb8ad98baffca3c8` | Default cache lazily replays commands and remaps session IDs after restart | Seal replay inputs and impose a reconstruction budget |
| Community REPL | `5d5c49d13dfc0c1d2df43a27c3e56e02ad81b9c3` | Documentation warns of scoped-extension pickle limitations | Test scoped notation and rebuild completed declarations from source |
| Kimina Lean Server | `fb2393de3461db35eda4c714e3fd21187e92ec90` | Documented pooled workers; per-worker memory enforcement is Linux-only | Qualify guarantees by platform and measure admission delay separately |

LeanDojo-v2's graph nodes use stack depth as identity. Backtracking appears
able to reuse an identifier while prior graph edges remain: a source-derived
aliasing concern, not a reproduced defect. Harp should assign transition IDs
independently of stack position.
[Prover implementation](https://github.com/lean-dojo/LeanDojo-v2/blob/baed5eae6e87a65a446d9f54af07aab2154e7599/lean_dojo_v2/prover/base_prover.py)

Pantograph requires imports before startup and identical environments for
goal restoration. Its contract distinguishes state removal and branching,
and acknowledges that cooperative timeout cannot contain every stalled tactic.
[Protocol](https://github.com/leanprover/Pantograph/blob/92d4818a4b343d7be293731e03359a19e8082626/doc/repl.md),
[rationale](https://github.com/leanprover/Pantograph/blob/92d4818a4b343d7be293731e03359a19e8082626/doc/rationale.md)

LeanInteract serializes the underlying exchange; async calls dispatch blocking
work to a thread. Replay stores command objects, including file-oriented
requests, and checks response class rather than an original semantic digest.
Reconstruction has no finite default timeout in the inspected path. Killing a
worker and remapping IDs is useful recovery machinery, but it does not prove
the reconstructed state has the original meaning.
[Server](https://github.com/augustepoiroux/LeanInteract/blob/976edd7d38a99e1ea4c2dfabeb8ad98baffca3c8/src/lean_interact/server.py),
[session cache](https://github.com/augustepoiroux/LeanInteract/blob/976edd7d38a99e1ea4c2dfabeb8ad98baffca3c8/src/lean_interact/sessioncache.py)

Community REPL's README still describes tactic mode as experimental and says
completed tactic states cannot be promoted into an environment replacing the
original sorry. This is a documentation observation; whether newer code
exceeds that contract was not checked.
[README](https://github.com/leanprover-community/repl/blob/5d5c49d13dfc0c1d2df43a27c3e56e02ad81b9c3/README.md)

Kimina documents worker counts, recycling, waiting, memory controls, and header
initialization. Its README does not establish content-bound environment
identity; a pool implementation artifact was unavailable, so implementation
semantics remain unverified in this pass.
[README](https://github.com/project-numina/kimina-lean-server/blob/fb2393de3461db35eda4c714e3fd21187e92ec90/README.md)

The practical shortlist remains provisional: Pantograph for explicit tactic
branching, LeanInteract for Python-facing recovery ergonomics, and Kimina as a
throughput reference. No evidence here selects a universal winner. Disposable
adapter state should be identified by environment, worker generation, and
local handle; durable recovery should reference sealed source and commands.

## Proposed experiments, in order

These are research protocols, not experiments run in this extension.

| Experiment | Fixed inputs and deliberate change | Measurements and decision |
|---|---|---|
| Crouzeix acceptance replay | Freeze the six-row bundle and route contracts; project them through the proposed contracts without changing proofs | Reproduce all route/axiom/reuse identities; record time for each acceptance stage. Any changed mathematical claim fails the projection. |
| Statement-drift negatives | Use small synthetic theorems; alter a referenced definition, strengthen a hypothesis, or substitute a different target | Every change must be rejected against the original target or explicitly classified as a new statement revision. |
| Output sealing and recovery | Use synthetic workers; interrupt after build, after sealing, and after receipt creation | No surviving writer at checking time, no accepted partial bundle, and one published result after recovery. Record platform-specific enforcement evidence. |
| Interactive adapter comparison | Same pinned Lean/project/imports, fixed tactic script, resource limits, and candidate source across adapters | Cold/warm startup, p50/p95 response latency, peak RSS, timeout cleanup, crash recovery, and final fresh-certification agreement. Unsupported version is a result, not permission to upgrade. |
| Bounded parallel search | Same target cohort, model, prompts, total token/tool budget and deadline; compare one worker with three isolated candidates | Source-faithful accepted targets per budget and time, duplicate work, merge conflicts, verification queue time. Keep certification policy constant. |

The first two experiments determine whether the contracts preserve the intended
mathematics. The third tests whether receipts refer to the bytes actually
checked. Adapter and search optimization follow once those results exist.

The adapter workload should create a state, branch twice, kill its worker,
restore one branch, and compile the resulting declaration afresh. Include
scoped notation, file-content drift, stale handles, changed imports, and
noncooperative cancellation. This tests the specific documented weaknesses
above while holding the mathematical task constant.

## Limits

This is a bounded update, not a second exhaustive census of the Lean ecosystem.
No performance ranking, exploit reproduction, external-kernel installation,
proof replay, or independent validation of ATLAS statistics was performed.
The research supports changes to the next experimental plan, not a claim that
any adapter or upstream evaluation stack is ready to become Harp's runtime.

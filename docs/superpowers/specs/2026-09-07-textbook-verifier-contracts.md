# Textbook verifier contracts

Design proposal, 2026-09-07. No verifier described here has been implemented or
qualified by this planning work. This is an engineering specification, not a
new authority for mathematical knowledge. Accepted mathematical prose belongs
in registered `knowledge/` packets; structured contracts belong in `content/`.

## Trust boundary and acceptance

The generator proposes source and proof terms. It cannot choose the challenge,
dependency policy, toolchain, checker, acceptance result, or receipt destination.
The scheduler assigns work but cannot mark a theorem accepted. A human owns
source interpretation, policy exceptions, architecture changes, and release.

The acceptance predicate is:

    accepted source item = reviewed source correspondence
      AND fixed-target proof accepted by qualified checker
      AND dependency/policy checks passed
      AND fresh replay evidence bound to the same inputs
      AND all required sub-obligations accepted

Machine acceptance alone does not establish that a formal statement expresses
the book. Numerical tests, a model's self-review, and successful compilation of
a different theorem cannot substitute for any conjunct.

## Records and state transitions

Use canonical serialized records with versioned schemas, reject duplicate keys
and unknown fields, and retain immutable prior revisions. IDs identify logical
objects; SHA-256 digests identify exact bytes. Hashes establish binding, not
the trustworthiness of the bytes being hashed.

| Record | Required fields and invariants |
|---|---|
| Source item | `item_id`, `book_id`, `source_digest`, `printing`, `printed_page`, `pdf_page`, `section`, `locator`, `kind`, `original_claim_summary`, `correction_refs`, `inventory_status`; source identity cannot change within a revision |
| Target revision | `target_id`, `revision`, `source_item_ids`, `challenge_digest`, `declaration`, `definition_closure_digest`, `environment_digest`, `policy_digest`, `prerequisite_target_revisions`, `semantic_review_digest`; no mutable prerequisite IDs |
| Policy | `policy_id`, `revision`, `axiom_allowlist`, `dependency_mode`, `permitted_foundation_digest`, `resource_limits`, `sandbox_profile_digest`, `checker_profile_digest`; generator cannot override |
| Attempt | `attempt_id`, `target_revision_digest`, `candidate_digest`, `worker_id`, `budget`, `started_at`, `termination_reason`, `sealed_bundle_digest`; never reused for modified input |
| Check result | tagged union `accepted`, `rejected`, `infrastructure_error`, `policy_blocked`; includes reason code and evidence references, never just a Boolean |
| Receipt | target/candidate/environment/policy/checker digests, exporter/parser revisions, dependency/axiom report digest, sandbox qualification identity, replay ID, authoritative result, timestamps and signer identity if crossing hosts |

Inventory, proof, semantic review, and replay are separate state machines.
Typical execution is `queued -> leased -> running -> stopping -> sealed ->
checking -> terminal`. Cancellation before sealing cannot yield acceptance.
An accepted proof can remain semantically unreviewed. A stale review or replay
does not erase the historical result, but it prevents current acceptance.

Result codes must distinguish `target_mismatch`, `invalid_proof`,
`forbidden_axiom`, `forbidden_dependency`, `malformed_candidate`,
`digest_mismatch`, `timeout`, `oom`, `worker_lost`, `missing_dependency`,
`unqualified_environment`, and `unqualified_sandbox`. A timeout is neither a
counterexample nor an invalid theorem. A mismatched old receipt is rejected
as evidence; it says nothing about whether a new candidate might be provable.

## Verification protocol

1. **Admit the target.** Read protected source/target/policy records. Verify the
   source locator, explicit hypotheses, scalar field, dimensions, domains,
   totality conventions, and prerequisite revisions. Human semantic review
   produces a separate artifact. The candidate cannot supply this review.
2. **Preflight without execution.** Verify toolchain/package digests, cache
   ancestry, qualified platform, checker binaries, writable-directory policy,
   and resource limits. Missing cache is blocked; do not hydrate during proof
   iteration. Reject ambient toolchain or project redirection in production.
3. **Run in confinement.** All candidate elaboration, imports, macros,
   initializers, native extensions, and build-related execution are untrusted.
   Deny network and access to protected challenge, credentials, receipts, and
   writable shared dependencies. Permit only the isolated attempt workspace.
4. **Stop writers and seal.** Establish that the entire worker containment unit
   has stopped, including escaped child writers. If this cannot be established,
   return `policy_blocked`. Do not rely solely on the parent process exit or a
   process-group signal. Snapshot allowed regular files through pinned file
   descriptors; reject links, special files, duplicate paths, traversal, and
   unsupported hard links. Revalidate size and identity while hashing.
5. **Export and check independently.** A qualified checker reconstructs/checks
   the candidate in the fixed environment and compares against the protected
   challenge. Export is also confined: importing adversarial `.olean` files
   can execute initializers. Neither exporter stdout nor candidate-written
   reports are authoritative. A protected controller receives bounded data,
   checks protocol integrity, and owns the verdict.
6. **Compare statements and definitions.** Compare elaborated expressions with
   universes and implicit/typeclass parameters, not printed theorem strings.
   Allow binder renaming and definitional equality under the fixed environment.
   Require a checked bridge for equivalence beyond that. Validate referenced
   definition closures: matching a theorem name and surface type is insufficient
   when an operation has been redefined. No open definition holes in the first
   profile.
7. **Check all dependencies.** Traverse the accepted declaration's transitive
   proof dependencies. Reject `sorryAx` and unapproved axioms, including hidden
   helper dependencies. The initial axiom set is a subset of
   `{propext, Quot.sound, Classical.choice}`. A proof need not use all three.
   Native reduction trust paths require separate qualification and are blocked
   in the initial strict profile. Text scans are diagnostics only.
8. **Publish atomically.** Bind exact sealed bytes, checker and environment
   identities, and reports. Only the trusted controller publishes a receipt.
   Interrupted publication leaves no accepted record. Deduplicate retries by
   input identity without confusing a receipt lookup with a fresh replay.
9. **Replay and promote.** A fresh qualified worker checks the same immutable
   target and candidate. Promotion requires agreement plus semantic review.
   A disagreement blocks promotion and preserves both reports for diagnosis.

An independent kernel implementation is defense in depth, not automatic truth.
Qualify supported expression forms, axioms, universe handling, export format,
and rejection behavior. Unsupported features block that profile; silently
falling back to a weaker checker is forbidden. The initial trusted-computing
base must be listed in each checker profile.

## Verifier qualification corpus

Each row requires a runnable fixture, expected result code, and captured fresh
evidence before its gate is enabled. No skipped test counts as qualification.
Trusted hand-authored mathematical fixtures may run before adversarial execution
is safe; arbitrary model submissions must wait for confinement qualification.

| ID | Fixture or fault | Required oracle |
|---|---|---|
| Q01 | Exact finite-real Cauchy–Schwarz library wrapper | Accept as library reuse |
| Q02 | A distinct valid proof of the same target | Accept independently; proof bytes differ |
| Q03 | Binder renaming and definitional aliases in permitted closure | Accept |
| Q04 | Valid theorem restricted to `n = 1`, or extra nonzero hypothesis | `target_mismatch`, even though Lean compiles it |
| Q05 | Reversed inequality | Reject invalid proof or target mismatch according to submitted artifact |
| Q06 | Direct `sorry`; hidden helper `sorry`; custom axiom behind a helper | `forbidden_axiom` |
| Q07 | Shadowed sum, multiplication, norm, or target definition | `target_mismatch` or forbidden closure change |
| Q08 | Correct target plus malformed proof | `invalid_proof`/`malformed_candidate`, not infrastructure failure |
| Q09 | Old receipt with changed proof, target, policy, or dependency bytes | `digest_mismatch`; no promotion |
| Q10 | Forged checker JSON, exit zero, and printed success string | No acceptance without authentic checker result |
| Q11 | Symlink swap, hardlink mutation, path traversal, FIFO, oversized export | Reject artifact or block sealing; never read outside allowlist |
| Q12 | Child writes after apparent worker exit | No seal until containment is quiescent; otherwise blocked |
| Q13 | Import initializer tries to change challenge/checker/receipt | Confinement prevents write; no forged acceptance |
| Q14 | Timeout, OOM, worker crash, truncated IPC | Distinct infrastructure errors |
| Q15 | Missing package; redirected environment variable/PATH | Block exact environment, no fetch or alternate compiler |
| Q16 | Crash immediately before/after seal and receipt publication | Retry yields one authoritative result, no half-published acceptance |
| Q17 | Duplicate delivery, expired lease, stale worker result | Idempotent processing; stale attempt cannot replace accepted revision |
| Q18 | Unsupported exporter node or native reduction dependency | `policy_blocked`, no silent downgrade |
| Q19 | Definitionally unequal but mathematically equivalent target | Requires an accepted bridge; do not blindly reject the mathematics |
| Q20 | Correct local proof with unproved prerequisite | Conditional result only; source item remains incomplete |

Freeze a qualification suite and retain withheld adversarial fixtures outside
generator retrieval. Every known false-accept fixture must fail acceptance.
Zero observed false accepts is a release gate, not an estimate that future
false-accept risk is zero. Report suite composition and test counts.

## Domain-specific semantic fixtures

These target packs precede automated proving in their domains. Each pack needs
an exact positive statement, a source interpretation review, a Lean-valid
weakened-statement mutant, and a counterexample or review argument exposing
the missing hypothesis. Mutants are not required to prove false propositions.

| Pack | Positive contracts | Required distinctions and negative controls |
|---|---|---|
| Algebra | Field-qualified finite-dimensional maps, bases, projections, adjoints | Dimension zero, rank deficiency, transpose versus conjugate transpose; real rotation without real eigenvectors; Jordan block not diagonalizable |
| Spectral/order | Correct self-adjoint/normal/positive matrix assumptions | PSD versus positive definite versus entrywise positive; multiplicity/crossings; no imaginary ordering of complex scalars |
| Differential calculus | Fréchet derivatives, chain rule, local inverse/implicit results | Partial derivatives alone; singular Jacobian; wrong domain or regularity; ReLU kink has no classical derivative |
| Integration | Source Riemann statements or proved agreement with library integral | Content zero versus measure zero; nonintegrability; unjustified exchange of limits/integrals; signed versus absolute Jacobian |
| Geometry | Alternating forms, chains, pullback, oriented compact manifold-with-boundary Stokes | Boundary-of-boundary sign cancellation; reversed orientation; manifold boundary versus ambient boundary; nonorientable object cannot meet oriented interface |
| Probability | Finite measures/probability kernels, densities, conditioning | Zero conditioning mass, singular covariance, nonnormalization, a.e. versus everywhere, support and absolute continuity |
| Optimization | Explicit objective/domain, stationarity, descent/convergence contracts | Stationary point not automatically global optimum; absent constraint qualification; bad step size; nonconvexity; finite time versus asymptotic guarantee |
| Variational inference | KL and ELBO with support/integrability conventions | Infinite KL; `0 log 0` convention; EM improvement versus global optimum; approximate E-step not exact EM |
| AD and networks | Typed shapes and real-arithmetic forward/reverse semantics | Shared subexpressions require gradient accumulation; broadcast reduction; nonsmooth node; wrong transpose; empty/all-masked softmax |
| Symmetries | Precisely stated convolution/graph/attention equivariance | Padding/stride/boundary conditions; positional encodings and masks break some symmetries; permutation action must be specified |
| Numerical algorithms | Separate exact correctness, convergence, floating-point error | Zero pivot; missing spectral gap; overflow/underflow; rounding and stopping rule; numerical finite differences are diagnostics |
| Generative models | Normalizing-flow bijections, sampler invariance, finite diffusion identities | Singular Jacobian/nonbijective flow; stationarity not mixing; incompatible supports; finite-step identities do not establish an SDE theorem |

Probability and calculus-of-variations targets must declare their function
spaces. Continuous-time diffusion requires a separate stochastic-analysis
dependency packet. Do not obtain apparent book coverage by replacing it with
finite Gaussian algebra without recording the narrower correspondence.

## Scheduling and cost controls

Keep the full item registry separate from Harp's bounded `TaskGraph`. The
current graph permits at most 64 nodes; compile ready obligations into bounded
batches rather than increasing that bound to contain whole books.

An obligation is ready only when its target revision is reviewed, verifier
pack qualified, environment available, dependencies accepted, and budget
approved. Alternative proof plans form OR choices; prerequisites within a plan
form AND dependencies. No cyclic justification is accepted. Speculative work
can exist, but it cannot unblock dependents until checked.

Priority considers dependency fan-out, missing central results, estimated cost,
and age in the residual queue. Begin with one worker and deterministic policy.
Measure before adding parallel search. Cancel obsolete branches after an
accepted alternative, then establish quiescence before publishing artifacts.

Record wall time, elaboration/check time, tokens, peak memory, retries, human
review time, cold/warm cache status, and acceptance by proof mode. Hold out
chapter/lemma families, not merely random theorem rows. Prevent retrieval of
held-out solutions; record existing Mathlib theorem reuse as reuse rather than
novel success. Compare automation against the simplest library-search baseline.

## Implementation boundaries in Harp

Proposed new modules: `labs/textbook_verification/contracts.py` for record
parsing, `seal.py` for immutable inputs, `runner.py` for confined execution,
`check.py` for trusted checking, `receipts.py` for publication/replay, and
`tests/` for real fixture tests. These names describe responsibilities; they
are not claims that the modules exist. Minimize dependencies and reuse stable
public helpers only. Do not import private Crouzeix route internals as a new
public API or refactor that route wholesale.

Use existing `ExecutionPlan`, `TaskGraph`, runtime activity, and recovery
contracts when scheduling becomes justified. Keep immutable curriculum data
in `content/` and human mathematical explanations in registered `knowledge/`
packets. Repository-wide schema registration and generated atlas changes are
paired landable slices; regenerate derived corpus and atlas together.

The current primary checkout was inspected at `2e5d57da`; this design worktree
is based on older `e5b45c22`. Reconcile paths and current contracts at execution
start. The repository pins Lean 4.32.1. Existing research records soundness
advisories affecting that line; do not relabel it production-qualified. Compiler
upgrade/cache maintenance needs owner authorization and a separate migration
gate. Qualification is about an exact environment, not a version label alone.

## Research references versus adoption

The [FLT/Prove2Me synthesis](../../workstream/lean-proof-infrastructure/flt-prove2me-production-synthesis-2026-09-07.md)
and [research extension](../../workstream/lean-proof-infrastructure/research-extension-2026-09-07.md)
record source evidence and pins. Adopt proof-obligation planning as a workflow
pattern, not as evidence that a particular reported FLT result proves Harp's
reliability. LeanDojo-style extraction/retrieval belongs after trustworthy
acceptance and dependency accounting. Comparator/exporter/independent-kernel
tools are qualification candidates, not preapproved trust anchors.

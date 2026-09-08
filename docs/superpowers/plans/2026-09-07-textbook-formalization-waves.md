# Textbook formalization waves implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a qualified verifier first, then maximize defensible coverage
of Lax 2007 second edition, Spivak, Bishop PRML, and Bishop/Bishop Deep Learning
before advancing to grand research campaigns.

**Architecture:** Keep source interpretation, fixed Lean targets, proof search,
independent verification, and coverage reporting separate. Use Harp's existing
bounded execution engine only after small verified workloads expose a scheduling
need. Shared mathematical foundations support four separately audited source maps.

**Tech Stack:** Lean 4 and Mathlib, existing Harp Rust execution contracts,
standard-library Python for initial verification tooling, immutable structured
records, current repository verification tasks.

---

This is the subsystem roadmap and release-contract plan. It does not pretend
that source statements not yet extracted have implementation-ready Lean APIs.
The immediately executable trusted-fixture packet is
[Wave 1](2026-09-07-textbook-wave-01-target-fixtures.md). Each later wave must
produce an executable packet with exact statements and test code after its
entry gate; a topic table does not authorize unattended implementation.

Normative planning inputs are the [verifier contracts](../specs/2026-09-07-textbook-verifier-contracts.md)
and [full coverage map](../../workstream/lean-proof-infrastructure/book-coverage-map-2026-09-07.md).
No book or chapter is complete merely because a capstone has been proved.

## Wave dependency and release table

| Wave | Deliverable and prerequisite | Required exit evidence |
|---|---|---|
| W0 | Source manifests, complete chapter/appendix containers, target/policy schema design; starts now | All four sources identified; Lax 2007 confirmed; every chapter and appendix assigned; source-printing gaps explicit |
| W1 | Trusted finite-real Cauchy–Schwarz target and positive controls | Exact target handles dimension zero; reference proof compiled on pinned warm environment; no verifier/automation completion claim |
| W2 | Independent verifier and adversarial qualification; requires W1 and approved environment | All Q01–Q20 contracts executed in qualified confinement; no false accepts; clean replay; semantic review; explicit trusted-computing base |
| W3 | Linear algebra and Euclidean foundations | Lax 1–8, algebraic appendices, Spivak 1, both Bishop matrix appendices inventoried and dependency-ordered; algebra/spectral packs pass |
| W4 | Differential and matrix calculus | Spivak 2, Lax 9, Bishop derivative foundations; Fréchet/locality/regularity mutants pass; source-to-library derivative bridges checked |
| W5 | Integration, probability, convexity, linear statistical models | Spivak 3; Lax 10/12–16 and compactness; PRML 1–4/B/E; DL 2–5/C; integration/probability/optimization packs pass |
| W6 | Forms, chains, manifolds, Stokes | Spivak 4–5 and relevant tensors/symplectic algebra; orientation and boundary fixtures pass; source integration bridge accepted |
| W7 | Remaining Lax applications/algorithms; kernels, inference, latent/sequential models | Lax 11/17–19 and remaining appendices; PRML 6–10/12–13/D; DL 11/15–16/B; exact/convergence/numerical claims separated |
| W8 | Networks, AD, optimization, architecture mathematics | PRML 5/14; DL 6–10/12–13; compositional AD proof, shape contracts, nonsmooth and symmetry fixtures pass |
| W9 | Sampling and generative models | PRML 11 and stochastic sequential results; DL 14/17–20; sampler/flow/variational packs pass; continuous SDE obligations separately accounted |
| W10 | Coverage reconciliation, production operations, research-campaign admission | Every source item accounted for; no denominator reduction; residual blockers itemized; recovery/resource/replay gates pass before Crouzeix admission |

Logical branches are allowed: W6 and mature W7 packets need not wait for every
unrelated W5 exercise. Readiness is per accepted prerequisite revision, never
just a wave number. Breadth inventory begins at W0; proof acceptance expands
only after domain verifiers qualify. Exercises and less prominent sections
are scheduled throughout, not left exclusively for W10.

## Work packages and gates

### W0: source and scope control

- [ ] Freeze source-file digests and edition/printing records. For Spivak,
  incorporate the corrected-printing preface and Addenda before item extraction.
- [ ] Populate every section's items, including substantive unnumbered displays,
  exercises, examples, definitions, and algorithm claims. Use a second inventory
  review to catch missing sections and exercises.
- [ ] Produce target packets of 5–15 related obligations as an initial batch
  policy. A packet has reviewed statements, dependencies, proof mode, verifier
  fixture IDs, and limits. Adjust batch size from measured compile/review cost.
- [ ] Register machine records through Harp's existing corpus contracts before
  adding them to managed `content/`; never add Markdown there. Pair registration,
  fixtures, canonical knowledge and generated atlas updates where required.

W0 can proceed while W1 is built. Complete item extraction is substantial work;
the current chapter map is its coverage scaffold, not its completion evidence.

### W1: trusted mathematical controls

- [ ] Execute the separate target-fixture plan. Keep it small and human-authored.
- [ ] Review universality over `Fin n`, including `n = 0`, and no extra hypotheses.
- [ ] Label Mathlib application as reuse. Plan reconstruction as a different
  dependency-policy experiment, never as a claim of theorem discovery.

### W2: verifier implementation, in landable sub-waves

- [ ] **W2a, records and protected challenge.** Implement bounded strict parsing,
  target revision binding, structured results, and explicit test-only roots.
  Activate malformed/duplicate-key/unknown-field/ambient-redirection tests with
  the implementation. No untrusted Lean execution yet.
- [ ] **W2b, confinement and sealing.** Qualify the actual host/worker mechanism,
  protected filesystem and process-tree termination; execute Q11–Q15 and Q16
  sealing failures using isolated disposable fixtures. Missing capability blocks
  the worker rather than weakening policy.
- [ ] **W2c, checker integration.** Qualify a pinned comparator/exporter and
  independent checker where supported. Execute Q01–Q10, Q13, Q18–Q20 against
  real Lean artifacts. Do not mock Lean acceptance or trust generated stdout.
- [ ] **W2d, receipt and replay.** Execute Q09, Q16, Q17 against actual publication,
  duplicate delivery, and fresh-worker replay. Attach source semantic review.
- [ ] Freeze the full Q01–Q20 report. Only now admit arbitrary agent candidates.

Each sub-wave receives its own exact-code implementation packet before work
begins. Tests depending on a later subsystem remain contract entries until that
subsystem exists; do not land skipped qualification tests as evidence of safety.

### W3–W5: shared foundations before specialization

- [ ] Audit Mathlib correspondences before proposing new abstractions. Record
  exact declaration signatures and source correspondence, not search hits alone.
- [ ] Freeze source-faithful statements and produce semantic mutants before
  adding automated proof attempts for each packet.
- [ ] Reuse accepted foundations across books with separate correspondence
  reviews. Keep algebraic duality/adjoints, Riemann/Lebesgue integration, and
  finite/distributional probability interfaces explicit.
- [ ] Accept packets only with checked closure, reviewed correspondence, fresh
  replay, and completed prerequisites. Report residual chapter work after every
  packet, even if its main theorem passes.

### W6–W9: advanced mathematics and algorithms

- [ ] Split definitions, algebraic identities, correctness, convergence, and
  finite-precision claims into distinct obligations linked to their source item.
- [ ] Prove the semantic bridges required by each domain pack before claiming
  correspondence. Do not replace Stokes orientations, a.e. statements, stochastic
  processes, or nondifferentiable operations with easier unstated conventions.
- [ ] For AD, begin with a small typed real-arithmetic language and a compositional
  derivative theorem; add sharing, shape/broadcast rules, then supported network
  operators. Require accumulation and transpose mutants before graph-scale work.
- [ ] For numerical linear algebra and training algorithms, state the iteration,
  initial conditions, stopping rule, arithmetic model, and guarantee separately.
  A real-arithmetic derivation earns no floating-point guarantee.
- [ ] Track mathematical infrastructure blockers by exact missing theorem/API.
  Preserve open obligations in coverage rather than silently narrowing claims.

### Orchestration activation and W10 production admission

- [ ] Measure a serial baseline on accepted target packets: proof-search latency,
  check latency, tokens, memory, review time, retries, and dependency wait time.
- [ ] Add retrieval only if recorded failures show premise/API lookup costs.
  Add bounded proof-plan scheduling only if dependency/coordination costs justify
  it. Reuse Harp `ExecutionPlan` and `TaskGraph`; no parallel scheduler product.
- [ ] Compare any new strategy against library search and serial execution on
  a frozen held-out family split, with identical budgets and verifier policy.
- [ ] Fault-test cancellation, restart, stale leases, cache corruption, receipt
  disagreement, and budget exhaustion. No corrupt artifact may become accepted.
- [ ] Require owner-approved resource and service-level budgets, runbooks for
  each infrastructure failure code, reproducible clean-worker replay, and a
  rollback rehearsal before calling the service production-ready.
- [ ] Reconcile every chapter/appendix item. Report accepted, represented,
  empirical/expository, refuted-as-written, corrected, and blocked separately.
  Never mark all four books complete with unresolved eligible items hidden.
- [ ] Admit a Crouzeix campaign only after demonstrated curriculum reliability
  and explicit human review of remaining coverage gaps. A limited regression
  using existing Crouzeix evidence is not admission of the grand campaign.

## Environment qualification and authority

The current repository pins Lean 4.32.1 and a warm cache. W1 can establish
experimental fixtures there. Production qualification must resolve the known
soundness-advisory concern recorded in the research extension. No version or
cache migration is authorized by this planning document.

A candidate replacement is Lean 4.33.1, git
`819816b2e0a3bf405af45ae5c7af2491d8f5bee6`, paired with audited Mathlib revision
`0df444a360eaa60ab8c11dca51a86af692955474`. For an owner-approved macOS ARM64
qualification, the authoritative [release metadata](https://api.github.com/repos/leanprover/lean4/releases/tags/v4.33.1)
identifies `lean-4.33.1-darwin_aarch64.tar.zst`, SHA-256
`88c45aad985b5d2a8d925fe10bd1296bd35f66f408480ab182d3facccd065a9d`.
Recheck metadata and archive digest before extraction; reject path traversal
and extract only into a dedicated versioned qualification directory. Never
pipe downloaded code into a shell. The eventual migration packet must pin all
checker binaries and packages too; this compiler digest alone is insufficient.

Before expensive gates, inspect toolchain/cache ancestry and mise trust without
Lake. Missing state blocks. Owner-authorized provisioning and cache maintenance
are a separate packet; no `lake update`, broad cache hydration, daemon setup,
push, or remote service installation occurs as an incidental proof step.

## Verification and handoff discipline

Each executable packet names exact files, complete code changes, commands and
expected behavior. Run focused tests while iterating; freeze the candidate,
then run `mise run verify` before local landing. Stage explicit path groups and
preserve unrelated changes. Reconcile the older design-worktree base with the
current primary contracts before coding.

For mathematical acceptance, retain real kernel/checker outputs and replay
records. Unit tests can simulate filesystem faults but cannot substitute mocked
Lean success for proof acceptance. Documentation review is not a passed verifier.

Current delivery: design, coverage map, detailed verifier contracts and first
trusted-fixture implementation packet. No proof, production verifier, book
inventory census, benchmark result, or environment upgrade is claimed complete.

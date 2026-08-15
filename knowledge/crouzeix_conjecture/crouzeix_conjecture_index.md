---
id: crouzeix-conjecture-index
title: Crouzeix conjecture two-proof index
type: index
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, numerical-range, spectral-set, formal-verification]
confidence: medium
canonical: crouzeix_conjecture_index.md
---

# Crouzeix conjecture two-proof index

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

This packet reconstructs two 2026 candidate proofs of the constant-two
Crouzeix conjecture, audits one accompanying Lean development, and separates
documented AI contributions from missing process evidence. Both mathematical
artifacts are preprints. A source-backed derivation is not a substitute for
peer review or independent reproduction.

## Five-minute model

The earlier double-layer calculus gives a positive symmetrized expression

$$
2\Phi(f)=f(A)+\alpha(f)(A)^*.
$$

The old barrier is that positivity controls the coupled expression, while the
target is only $f(A)$. Bounding the companion independently loses the
coupling and stops above $2$.

The two new routes preserve a complete family before extracting a norm bound:

1. **Jin:** push the Cayley family
   $(1+wf)/(1-wf)$ through $\Phi$. In an auxiliary eigenbasis, the
   adjoint-algebra defect becomes diagonal. Sample the matrix Herglotz kernel
   at $\overline{\lambda_i}/2$, add a compensating origin sample, cancel the
   unknown correction, and compare weighted Gramians.
2. **Lorist-Schwenninger:** apply the double-layer identity to every power
   $f^n$. The resulting uniformly bounded commuting perturbations $E_n$
   fit a 2-dilation lemma. A scalar recurrence along a top singular vector
   rules out $\|f(A)\|>2$.

The shared-power-family framing is Harp's comparison, not terminology claimed
by either source.

## Source and status boundary

- Jin's formalization-matched manuscript is pinned at
  `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`.
- The inspected repository head is
  `9df07838327b988e3924453daa29c8cd726d34b0`; its v4 and
  Annals-formatted manuscripts are distinct artifacts.
- The Preprints.org route is metadata only because headless acquisition
  returned HTTP `403`; no posted bytes are mapped to a Git revision.
- Lorist-Schwenninger is pinned to arXiv `2608.03841v1`.
- Harp's source scans passed. Both clean-room Lean build attempts are recorded
  as `blocked` by the 8 GiB disk preflight, not passed or failed.

See the [source registry](source_registry.md), [claim ledger](claim_evidence_ledger.md),
and [critical assessment](09_status_and_critical_assessment.md) for exact
claim ceilings.

## Guided route

1. [Problem and prior barrier](01_problem_and_prior_barrier.md)
2. [Shared power family](02_shared_power_family.md)
3. [Jin proof spine](03_jin_proof_spine.md)
4. [Jin positive-real completion](04_jin_positive_real_completion.md)
5. [Lorist-Schwenninger proof](05_lorist_schwenninger_proof.md)
6. [Proof comparison](06_proof_comparison.md)
7. [Jin Lean verification](07_jin_lean_verification.md)
8. [AI-assisted discovery](08_ai_assisted_discovery.md)
9. [Status and critical assessment](09_status_and_critical_assessment.md)

## Expert route

Read the [positive-real completion](04_jin_positive_real_completion.md), the
[2-dilation proof](05_lorist_schwenninger_proof.md), and the
[fixed comparison](06_proof_comparison.md) first. Then use the
[Lean audit](07_jin_lean_verification.md) and
[claim ledger](claim_evidence_ledger.md) to inspect theorem and evidence
boundaries.

## Question routes

- **How does Jin remove the adjoint correction?**
  Read [exact cancellation](04_jin_positive_real_completion.md#exact-cancellation).
- **How does the $2$-dilation recurrence work?**
  Read [the recurrence](05_lorist_schwenninger_proof.md#the-perturbation-recurrence).
- **What does Lean actually verify?**
  Read [the theorem graph](07_jin_lean_verification.md#theorem-and-consequence-graphs).
- **What did the AI systems contribute?**
  Read the [action matrix](08_ai_assisted_discovery.md#documented-action-matrix).
- **What evidence remains missing?**
  Read the [confidence-raising evidence](09_status_and_critical_assessment.md#evidence-that-would-raise-confidence).

## Reference documents

- [Glossary](glossary.md)
- [Source registry](source_registry.md)
- [Claim-evidence ledger](claim_evidence_ledger.md)

The canonical packet consists of:

- [crouzeix_conjecture_index.md](crouzeix_conjecture_index.md)
- [01_problem_and_prior_barrier.md](01_problem_and_prior_barrier.md)
- [02_shared_power_family.md](02_shared_power_family.md)
- [03_jin_proof_spine.md](03_jin_proof_spine.md)
- [04_jin_positive_real_completion.md](04_jin_positive_real_completion.md)
- [05_lorist_schwenninger_proof.md](05_lorist_schwenninger_proof.md)
- [06_proof_comparison.md](06_proof_comparison.md)
- [07_jin_lean_verification.md](07_jin_lean_verification.md)
- [08_ai_assisted_discovery.md](08_ai_assisted_discovery.md)
- [09_status_and_critical_assessment.md](09_status_and_critical_assessment.md)
- [glossary.md](glossary.md)
- [source_registry.md](source_registry.md)
- [claim_evidence_ledger.md](claim_evidence_ledger.md)

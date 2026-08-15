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

See the [[knowledge/crouzeix_conjecture/source_registry|source registry]], [[knowledge/crouzeix_conjecture/claim_evidence_ledger|claim ledger]],
and [[knowledge/crouzeix_conjecture/09_status_and_critical_assessment|critical assessment]] for exact
claim ceilings.

## Guided route

1. [[knowledge/crouzeix_conjecture/01_problem_and_prior_barrier|Problem and prior barrier]]
2. [[knowledge/crouzeix_conjecture/02_shared_power_family|Shared power family]]
3. [[knowledge/crouzeix_conjecture/03_jin_proof_spine|Jin proof spine]]
4. [[knowledge/crouzeix_conjecture/04_jin_positive_real_completion|Jin positive-real completion]]
5. [[knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof|Lorist-Schwenninger proof]]
6. [[knowledge/crouzeix_conjecture/06_proof_comparison|Proof comparison]]
7. [[knowledge/crouzeix_conjecture/07_jin_lean_verification|Jin Lean verification]]
8. [[knowledge/crouzeix_conjecture/08_ai_assisted_discovery|AI-assisted discovery]]
9. [[knowledge/crouzeix_conjecture/09_status_and_critical_assessment|Status and critical assessment]]
10. [[knowledge/crouzeix_conjecture/reproduction_research|Proof-reproduction research]]

## Expert route

Read the [[knowledge/crouzeix_conjecture/04_jin_positive_real_completion|positive-real completion]], the
[[knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof|2-dilation proof]], and the
[[knowledge/crouzeix_conjecture/06_proof_comparison|fixed comparison]] first. Then use the
[[knowledge/crouzeix_conjecture/07_jin_lean_verification|Lean audit]] and
[[knowledge/crouzeix_conjecture/claim_evidence_ledger|claim ledger]] to inspect theorem and evidence
boundaries.

## Question routes

- **How does Jin remove the adjoint correction?**
  Read [[knowledge/crouzeix_conjecture/04_jin_positive_real_completion#Exact cancellation|exact cancellation]].
- **How does the $2$-dilation recurrence work?**
  Read [[knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof#The perturbation recurrence|the recurrence]].
- **What does Lean actually verify?**
  Read [[knowledge/crouzeix_conjecture/07_jin_lean_verification#Theorem and consequence graphs|the theorem graph]].
- **What did the AI systems contribute?**
  Read the [[knowledge/crouzeix_conjecture/08_ai_assisted_discovery#Documented action matrix|action matrix]].
- **What evidence remains missing?**
  Read the [[knowledge/crouzeix_conjecture/09_status_and_critical_assessment#Evidence that would raise confidence|confidence-raising evidence]].
- **What part of the original proof-discovery process can be reproduced?**
  Read the [[knowledge/crouzeix_conjecture/reproduction_research#Reproduction anchor|proof-reproduction research]].

## Reference documents

- [[knowledge/crouzeix_conjecture/glossary|Glossary]]
- [[knowledge/crouzeix_conjecture/source_registry|Source registry]]
- [[knowledge/crouzeix_conjecture/claim_evidence_ledger|Claim-evidence ledger]]
- [[knowledge/crouzeix_conjecture/reproduction_research|Proof-reproduction research]]

The canonical packet consists of:

- [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|crouzeix_conjecture_index.md]]
- [[knowledge/crouzeix_conjecture/01_problem_and_prior_barrier|01_problem_and_prior_barrier.md]]
- [[knowledge/crouzeix_conjecture/02_shared_power_family|02_shared_power_family.md]]
- [[knowledge/crouzeix_conjecture/03_jin_proof_spine|03_jin_proof_spine.md]]
- [[knowledge/crouzeix_conjecture/04_jin_positive_real_completion|04_jin_positive_real_completion.md]]
- [[knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof|05_lorist_schwenninger_proof.md]]
- [[knowledge/crouzeix_conjecture/06_proof_comparison|06_proof_comparison.md]]
- [[knowledge/crouzeix_conjecture/07_jin_lean_verification|07_jin_lean_verification.md]]
- [[knowledge/crouzeix_conjecture/08_ai_assisted_discovery|08_ai_assisted_discovery.md]]
- [[knowledge/crouzeix_conjecture/09_status_and_critical_assessment|09_status_and_critical_assessment.md]]
- [[knowledge/crouzeix_conjecture/reproduction_research|reproduction_research.md]]
- [[knowledge/crouzeix_conjecture/glossary|glossary.md]]
- [[knowledge/crouzeix_conjecture/source_registry|source_registry.md]]
- [[knowledge/crouzeix_conjecture/claim_evidence_ledger|claim_evidence_ledger.md]]

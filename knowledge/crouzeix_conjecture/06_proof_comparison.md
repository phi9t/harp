---
id: crouzeix-proof-comparison
title: Comparison of the Jin and Lorist-Schwenninger proofs
type: deep-dive
status: active
created: 2026-08-14
updated: 2026-08-26
tags: [crouzeix-conjecture, proof-comparison, positive-real, dilation]
confidence: medium
canonical: 06_proof_comparison.md
---

# Comparison of the Jin and Lorist-Schwenninger proofs

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

The two proofs share double-layer prerequisites and preserve a complete power
family, but their finite-dimensional engines are not the same theorem in
different notation.

## Fixed comparison {#fixed-comparison}

| Contract | Jin | Lorist-Schwenninger |
|---|---|---|
| Retained family | Cayley generating function $1+2\sum_{m\ge1}w^mf^m$ | Explicit perturbations $E_n$ for every $f^n$ |
| Finite-dimensional lemma | Positive-real completion relative to an auxiliary simple-spectrum basis | Uniformly bounded commuting perturbations of a compressed contraction-power family |
| Decisive estimate | Origin sample cancels the diagonal correction; sampled positivity compares weighted Gramians | Completing the square produces a scalar recurrence; a terminal estimate and singular-vector inequality rule out $\kappa>2$ |
| Order-sensitive input | $YG^{-1}P$, $PG^{-1}Y$, and the two Gramian weights cannot be collapsed or reordered | $E_nT=TE_n$ is needed to link adjacent recurrence terms |
| Double-layer route | Positive unital map applied to the full Cayley family creates $H$ and an adjoint-algebra defect | Boundary-density Stinespring form creates $V,Q$, while $\alpha(f^n)(A)$ supplies $E_n$ |
| Formalization evidence | Source-level Lean theorem graph and pinned audit; upstream Harp clean-room build blocked by disk preflight, but Harp-local route certification is `complete-local` for the Jin terminal and consequence declarations | Harp-local route certification is `complete-local` for the Lorist-Schwenninger terminal theorem and closed-range consequence, with source-faithful review and `source_fidelity_check` passed |
| Completely bounded limitation | No direct matrix-amplified conclusion established | Source says commutation is unavailable after amplification |

## Shared structure without equivalence {#shared-structure-without-equivalence}

The [[knowledge/crouzeix_conjecture/02_shared_power_family|shared-power-family inference]] explains why the
two papers can both extract more from the old symmetrized identity than a
single norm estimate. It does not identify:

- a map from every positive-real completion to a perturbation family satisfying
  the Lorist-Schwenninger recurrence;
- a map from every abstract 2-dilation family to Jin's auxiliary-basis
  correction algebra; or
- a common amplified theorem covering the completely bounded problem.

The strongest safe comparison is operational: both delay the norm estimate
until a relation among all powers has been used.

## Independent and different {#independent-and-different}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-041: Lorist-Schwenninger call Jin independent and different|EVIDENCE - CC-041]].**
Lorist and Schwenninger state in arXiv v1 that Jin's proof appeared
independently while their note was being prepared, and characterize it as a
different function-theoretic, matrix-valued route. This is the authors'
historical report. Harp does not have private timestamps or correspondence
that independently establishes the full discovery chronology.

## Which route exposes what {#which-route-exposes-what}

Jin makes the de-symmetrization mechanism explicit:

1. use simple spectrum only for the auxiliary basis;
2. turn the unknown defect into a diagonal analytic correction;
3. cancel that correction with a designed origin vector;
4. use positive-kernel coupling to obtain ordered Gramian inequalities.

Lorist-Schwenninger isolate a shorter operator-theoretic lever:

1. encode every power as a perturbation of a compressed contraction power;
2. commute the perturbations with $T$;
3. run a scalar recurrence along a norm-attaining vector;
4. close the contradiction with one first-power compression estimate.

The first route has a longer finite-dimensional algebraic core and a direct
Lean development. The second route has a compact abstract lemma and a more
modular Stinespring-style extension.

## Evidence asymmetry {#evidence-asymmetry}

The mathematical comparison is source-backed, but the verification evidence is
still asymmetric. Jin's repository exposes Lean source, declaration maps,
audits, and revision history. Lorist-Schwenninger arXiv v1 exposes TeX and PDF
but no upstream registered formalization. Harp's local scans cover only the two
Jin revisions, and both upstream Jin clean-room builds remain blocked.

The Harp-local certification surface is broader than the earlier packet state.
The Jin, Lorist-Schwenninger, and Harp routes each now have published
`complete-local` receipts and matching reviews
([[evidence/crouzeix_conjecture/routes/jin/receipt.json|jin]],
[[evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json|lorist-schwenninger]],
[[evidence/crouzeix_conjecture/routes/harp/receipt.json|harp]]).
For the certified Lorist-Schwenninger route in particular, the local review is
source-faithful and records `source_fidelity_check` as `passed`.
The six-row bundle
([[evidence/crouzeix_conjecture/local_formalization/manifest.tsv|manifest]])
binds terminal and closed-range rows for all three routes, with aggregate
SHA-256 `efc8469255219938b958d687d4a62506df937918bd659df18e47eaeb85a71de7`.

That asymmetry is a reason to separate proof mechanism from verification
status, not a reason to rank the mathematical arguments.

## Harp derivation boundary {#harp-derivation-boundary}

Harp's route is not mathematically independent of Lorist-Schwenninger. The
published Harp route review marks derivation reuse as passed, and the Harp
manifest records exactly eleven approved lower-level Lorist-Schwenninger reuse
modules:

1. `BoundarySquareRoot`
2. `BoundaryEmbedding`
3. `BoundaryMultiplier`
4. `CompressionMoments`
5. `Dilation`
6. `NormAttainment`
7. `CompletedSquare`
8. `CompanionAlgebra`
9. `PolynomialPowerCauchy`
10. `Recurrence`
11. `Scalar`

After those reused modules, Harp derives its own finite cubature, finite-horizon
recurrence, perturbation endpoint, terminal theorem, and closed-range
consequence. The safe statement is therefore: Harp is a derived route with
exactly eleven approved lower-level Lorist-Schwenninger support modules, not a
mathematically independent third proof.

## Common limitation {#common-limitation}

The scalar constant-two conclusion does not automatically become a completely
bounded spectral-set theorem. Jin's sampled matrix kernel is used at one matrix
level; Lorist-Schwenninger's commutative perturbation interface is not stable
under arbitrary matrix amplification. A new compatibility mechanism would be
needed.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

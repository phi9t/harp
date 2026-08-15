---
id: crouzeix-shared-power-family
title: Shared power family across the two proofs
type: concept
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, cayley-family, power-family, proof-comparison]
confidence: medium
canonical: 02_shared_power_family.md
---

# Shared power family across the two proofs

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

## The comparison

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-040: Both proofs retain a complete power family|INFERENCE - CC-040]].**
Both proofs avoid collapsing the double-layer identity to one estimate for one
function. They retain information for every power, either explicitly or
through a generating function.

### Jin: powers packaged by the Cayley transform

For $\|f\|_\infty\le1$,

$$
\frac{1+wf}{1-wf}=1+2\sum_{m=1}^{\infty}w^m f^m.
$$

The double-layer map transports this locally uniformly convergent series. The
result is one analytic matrix function $H(w)$ whose Herglotz kernel couples
different samples. Its resolvent part carries all $T^m$, where $T=f(B)$.

### Lorist-Schwenninger: powers kept as a sequence

The second route applies the double-layer identity to each $f^n$:

$$
E_n=2V^*Q^{*n}V-T^{*n}.
$$

Uniform boundedness and commutation let the perturbation lemma compare
successive scalar quantities
$m_n=\operatorname{Re}\langle E_nT^nx,x\rangle$.

## What is genuinely shared

The shared invariant is not merely “use the double-layer potential.” It is:

1. normalize $f$ on a domain containing the numerical range;
2. preserve the indexed family $f^n$;
3. exploit a relation connecting different members of that family; and
4. postpone the norm conclusion until after that relation is used.

Jin uses positive-kernel coupling between sample points and weighted Gramian
coefficients. Lorist-Schwenninger use a recurrence between adjacent powers.

## Limits of the comparison

This is Harp terminology. It weakens if the Cayley proof can be reconstructed
without any order-sensitive power identities, or if the perturbation proof can
be reduced to an estimate involving only $E_1$. It does not imply that the
positive-real completion theorem and the 2-dilation lemma are equivalent:
their hypotheses, auxiliary spaces, and decisive inequalities differ.

Proceed to the [[knowledge/crouzeix_conjecture/03_jin_proof_spine|Jin proof spine]] or directly to the
[[knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof|Lorist-Schwenninger proof]].

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

---
id: crouzeix-problem-and-prior-barrier
title: Crouzeix problem and the prior barrier
type: concept
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, numerical-range, spectral-set, double-layer]
confidence: medium
canonical: 01_problem_and_prior_barrier.md
---

# Crouzeix problem and the prior barrier

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

## The constant-two statement

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-001: Crouzeix constant-two conjecture|SOURCE CLAIM - CC-001]].**
For every complex square matrix $A$ and polynomial $p$,

$$
\|p(A)\|\le 2\max_{z\in W(A)}|p(z)|.
$$

The equivalent rational formulation says that the closure of $W(A)$ is a
$2$-spectral set for $A$. In infinite dimension the closure matters
because the numerical range need not be closed.

## Why two cannot be improved

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-002: Two-by-two nilpotent sharpness|EVIDENCE - CC-002]].**
Take

$$
A=\begin{pmatrix}0&2\\0&0\end{pmatrix},\qquad p(z)=z.
$$

For a unit vector $x=(x_1,x_2)$,
$\langle Ax,x\rangle=2x_2\overline{x_1}$, so
$|\langle Ax,x\rangle|\le1$, with equality attainable. Thus $W(A)$ is the
closed unit disk, while $\|A\|=2$. Any universal constant is therefore at
least $2$.

## The preceding universal bound

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-003: Crouzeix-Palencia one-plus-square-root-two|SOURCE CLAIM - CC-003]].**
Crouzeix and Palencia established the universal constant
$1+\sqrt2$. The two new preprints both start from the same broad
double-layer tradition but use different information that an isolated
one-step norm estimate discards.

## The symmetrized identity

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-004: Symmetrized double-layer identity|EVIDENCE - CC-004]].**
For a suitable convex domain $\Omega\supset W(A)$, the positive double-layer
map has the form

$$
2\Phi(f)=f(A)+\alpha(f)(A)^*.
$$

The map $\Phi$ is positive and unital. The transform $\alpha$ is bounded
and antilinear. This identity is powerful because it couples the desired
functional-calculus value to a structured companion.

## Where the one-step argument loses information

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-005: One-step treatment loses coupling|INFERENCE - CC-005]].**
Positivity controls the sum, not $f(A)$ alone. If one replaces the companion
by an unrelated operator satisfying only a norm bound, the algebraic relation
between the two terms disappears. The two 2026 routes instead retain a
power-indexed family through the decisive finite-dimensional step.

The next chapter isolates that common structure without claiming that the two
proofs are instances of one theorem.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

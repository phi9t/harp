---
id: crouzeix-jin-proof-spine
title: Jin proof spine from outer domains to constant two
type: deep-dive
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, jin-proof, cayley-family, outer-limit]
confidence: medium
canonical: 03_jin_proof_spine.md
---

# Jin proof spine from outer domains to constant two

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

This chapter follows the formalization-matched v4 artifact. It keeps the
positive-real completion theorem as a named finite-dimensional boundary; the
next chapter opens that boundary and derives its cancellation and Gramian
steps.

## Normalize on a fixed outer domain {#normalize-on-a-fixed-outer-domain}

Fix a matrix $A$, put $K=W(A)$, and choose an open neighborhood $U$ on
which $f$ is holomorphic. For

$$
\Omega_\varepsilon=\{z:\operatorname{dist}(z,K)<\varepsilon\}
$$

with sufficiently small $\varepsilon>0$, the closure of
$\Omega_\varepsilon$ lies in $U$. Define

$$
m_\varepsilon=
\max_{z\in\overline{\Omega_\varepsilon}}|f(z)|.
$$

If $m_\varepsilon=0$, functional calculus gives $f(A)=0$. Otherwise,
replace $f$ by $f/m_\varepsilon$. The fixed-domain problem is now to prove
$\|f(B)\|\le2$ whenever $B$ has simple spectrum and
$W(B)\subset\Omega_\varepsilon$.

The normalization matters twice: it puts the target values
$\lambda_i=f(\beta_i)$ in the closed disk, and it keeps every scalar
denominator in the Cayley and sampled-kernel formulas nonzero.

## Build the full Cayley family {#build-the-full-cayley-family}

Let $\Phi_\gamma$ be the positive unital double-layer map for the fixed
outer boundary. For $w\in\mathbb D$, define

$$
c_w(t)=\frac{1+w f(\gamma(t))}{1-w f(\gamma(t))}
=1+2\sum_{m=1}^{\infty}w^m f(\gamma(t))^m
$$

and

$$
H(w)=\Phi_\gamma(c_w).
$$

Local uniform convergence and boundedness of $\Phi_\gamma$ make $H$
analytic. Positivity of the scalar real part of $c_w$, followed by positivity
and star preservation of $\Phi_\gamma$, gives

$$
H(0)=I,\qquad \operatorname{Re}H(w)\succeq0.
$$

The first Cauchy layer sends every $f^m$ to $T^m$, where $T=f(B)$.
Consequently,

$$
I+2\sum_{m=1}^{\infty}w^mT^m
=(I+wT)(I-wT)^{-1}.
$$

This is the order-sensitive family carried into the completion theorem.

## Produce a completion modulo the auxiliary adjoint algebra {#produce-a-completion-modulo-the-auxiliary-adjoint-algebra}

Splitting the double-layer density gives

$$
2H(w)=(I+wT)(I-wT)^{-1}+G(w)^*,
$$

where $G(w)\in\operatorname{alg}(B)$. Since

$$
(I+wT)(I-wT)^{-1}=2(I-wT)^{-1}-I,
$$

one obtains

$$
H(w)-(I-wT)^{-1}
=\frac12(G(w)^*-I)\in\operatorname{alg}(B^*).
$$

The simple-spectrum assumption belongs to $B$, not $T$. The eigenbasis of
$B$ also diagonalizes $T=f(B)$, even when several
$f(\beta_i)$ coincide.

## Apply positive-real completion {#apply-positive-real-completion}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-010: Jin completion hypotheses|EVIDENCE - CC-010]].**
The preceding construction satisfies the complete theorem interface: same
basis, target values in the closed disk, analytic positive-real $H$,
normalization at zero, and the adjoint-algebra defect.

The [[knowledge/crouzeix_conjecture/04_jin_positive_real_completion|positive-real completion derivation]]
then proves

$$
\|T\|=\|f(B)\|\le2.
$$

This step is where the correction is canceled rather than norm-bounded.

## Pass from simple spectrum to the target matrix {#pass-from-simple-spectrum-to-the-target-matrix}

Choose simple-spectrum matrices $B_k\to A$. For fixed $\varepsilon$, all
sufficiently large $k$ satisfy

$$
W(B_k)\subset\Omega_\varepsilon.
$$

Applying the normalized fixed-domain theorem yields

$$
\|f(B_k)\|\le2m_\varepsilon.
$$

The holomorphic functional calculus is continuous along $B_k\to A$ using a
fixed contour inside $U$, so

$$
\|f(A)\|\le2m_\varepsilon.
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-017: Fixed-domain limit precedes outer-domain limit|EVIDENCE - CC-017]].**
This is the first limit. The domain $\Omega_\varepsilon$ is held fixed while
$k\to\infty$. The argument does not ask for estimates uniform in a
simultaneously moving matrix and boundary.

## Shrink the outer domains {#shrink-the-outer-domains}

Uniform continuity of $f$ on a fixed compact neighborhood gives

$$
0\le m_\varepsilon-
\max_{z\in K}|f(z)|
\le\omega_f(\varepsilon).
$$

Only now send $\varepsilon\downarrow0$. The result is

$$
\|f(A)\|\le2\max_{z\in W(A)}|f(z)|.
$$

The order is therefore:

```text
fixed epsilon:
  B_k -> A
then:
  epsilon -> 0
```

## Polynomial and rational consequences {#polynomial-and-rational-consequences}

Polynomials are holomorphic everywhere, so the main conjectured inequality is
an immediate specialization. A rational function with poles off the compact
numerical range is holomorphic on a neighborhood of it, giving the finite
rational spectral-set consequence. The infinite-dimensional polynomial route
uses finite Krylov compression; the rational route requires the additional
approximation and inverse compatibility exposed in the Lean source.

## Revision-history boundary {#revision-history-boundary}

The repository contains earlier proof routes, including radial and Stein-style
formulations. This packet follows the formalization-matched v4 completion
route. It does not merge equations from older revisions into the audited proof
spine, and it treats the later repository-head v4 as a distinct source
artifact.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

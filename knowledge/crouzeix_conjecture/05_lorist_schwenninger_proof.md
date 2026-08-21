---
id: crouzeix-lorist-schwenninger-proof
title: Lorist-Schwenninger 2-dilation proof
type: derivation
status: active
created: 2026-08-14
updated: 2026-08-20
tags: [crouzeix-conjecture, lorist-schwenninger, dilation, perturbation]
confidence: medium
canonical: 05_lorist_schwenninger_proof.md
---

# Lorist-Schwenninger 2-dilation proof

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

Lorist and Schwenninger preserve the powers explicitly. Their finite-dimensional
lemma says that a uniformly bounded commuting perturbation of a compressed
contraction-power family cannot have norm larger than two.

## The perturbation interface {#the-perturbation-interface}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-030: Lorist-Schwenninger perturbation lemma|SOURCE CLAIM - CC-030]].**
Let $T$ act on a finite-dimensional Hilbert space $H$. Assume there are a
Hilbert space $K$, a contraction $Q$ on $K$, and an isometry
$V:H\to K$ such that

$$
E_n=2V^*Q^{*n}V-T^{*n},\qquad n\in\mathbb N,
$$

are uniformly bounded and commute with $T$. Then

$$
\|T\|\le2.
$$

The commutation hypothesis is used when adjacent powers are combined in the
recurrence. Uniform boundedness controls the terminal term and verifies the
interface in the numerical-range application.

## Equation 1 controls the terminal term {#equation-one-controls-the-terminal-term}

Let

$$
M=\sup_{n\ge1}\|E_n\|<\infty.
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-032: Equation one uniformly bounds E-n T-n|INFERENCE - CC-032]].**
Equation 1 itself gives

$$
\|T^n\|=\|T^{*n}\|
\le2\|V^*Q^{*n}V\|+\|E_n\|
\le2+M.
$$

Therefore

$$
\|E_nT^n\|\le M(2+M).
$$

No additional boundedness hypothesis is needed. The boundedness of $E_n$,
the isometry $V$, and the contraction $Q$ are exactly the inputs.

Harp now has this terminal boundedness estimate as a compiled Lean support
lemma for the abstract finite-dimensional perturbation interface:
`CrouzeixConjecture.LoristSchwenninger.perturbation_mul_target_power_norm_le`
in `formalization/lean/Crouzeix/LoristSchwenninger/Perturbation.lean`.
The proof-slice receipt is
[[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/proof-slices/ls-equation-one-terminal-bound/attempt-001/receipt.json|ls-equation-one-terminal-bound/attempt-001]].
This is not yet a proof of Lemma 1: the scalar recurrence below and the
double-layer realization remain separate formal obligations.

## The perturbation recurrence {#the-perturbation-recurrence}

Set $\kappa=\|T\|$. The case $\kappa\le1$ is immediate, so suppose
$\kappa>1$. Finite dimensionality gives a unit vector $x$ with

$$
T^*Tx=\kappa^2x.
$$

Define

$$
S_n^*=2V^*Q^{*n}V,\qquad
m_n=\operatorname{Re}\langle E_nT^nx,x\rangle.
$$

Since $E_nT=TE_n$, adjacent terms can be compared. Put

$$
y_n=(S_{n+1}^*T-\kappa S_n^*)x.
$$

Completing the square gives

$$
\begin{aligned}
\kappa m_n-m_{n+1}
&=(\kappa^2-\kappa)
\left\|T^{*n}x-\frac{y_n}{2(\kappa^2-\kappa)}\right\|^2
-\frac{\|y_n\|^2}{4(\kappa^2-\kappa)}\\
&\ge
-\frac{\|y_n\|^2}{4(\kappa^2-\kappa)}
=:r_n.
\end{aligned}
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-031: Lorist-Schwenninger recurrence|EVIDENCE - CC-031]].**
Dividing by $\kappa$ and iterating yields

$$
m_1\ge\kappa^{-N}m_{N+1}
 +\sum_{n=1}^{N}\kappa^{-n}r_n.
$$

The compression structure makes the error term independent of $n$:

$$
\begin{aligned}
\|y_n\|
&=\|(S_{n+1}^*T-\kappa S_n^*)x\|\\
&\le
2\|V^*Q^{*n}\|
\|Q^*VTx-\kappa Vx\|\\
&\le2\|Q^*VTx-\kappa Vx\|.
\end{aligned}
$$

Hence

$$
r_n\ge
-\frac{\|Q^*VTx-\kappa Vx\|^2}{\kappa^2-\kappa}.
$$

The product bound from the preceding section makes $m_{N+1}$ uniformly
bounded. Since $\kappa>1$,
$\kappa^{-N}m_{N+1}\to0$. Taking $N\to\infty$ gives

$$
\operatorname{Re}\langle E_1Tx,x\rangle
\ge
-\frac{\|Q^*VTx-\kappa Vx\|^2}
{\kappa(\kappa-1)^2}.
$$

## The contradiction above two {#the-contradiction-above-two}

Using $V^*V=I$, $\|Q\|\le1$, the top-singular-vector equation, and

$$
E_1=2V^*Q^*V-T^*,
$$

the source obtains

$$
\|Q^*VTx-\kappa Vx\|^2
\le
2\kappa^2
-\kappa\operatorname{Re}\langle E_1Tx,x\rangle
-\kappa^3.
$$

Combining this with the lower bound gives

$$
\|Q^*VTx-\kappa Vx\|^2
\left(1-\frac1{(\kappa-1)^2}\right)
\le2\kappa^2-\kappa^3.
$$

If $\kappa>2$, the coefficient on the left is positive and the left side is
nonnegative, while the right side is negative. This contradiction proves
$\kappa\le2$.

## Double-layer realization {#double-layer-realization}

Work first in finite dimension with a smoothly bounded open convex
$\Omega\supset\overline{W(A)}$. Let $P_\Omega(\sigma)$ be the positive
double-layer density and

$$
\Phi(f)=\frac12
\int_{\partial\Omega}
f(\sigma)P_\Omega(\sigma)\,|d\sigma|.
$$

Unitality, $\Phi(1)=I$, makes

$$
Vx=2^{-1/2}P_\Omega(\cdot)^{1/2}x
$$

an isometry into
$K=L^2(\partial\Omega,|d\sigma|;H)$. For normalized
$\|f\|_\infty=1$, define the contraction

$$
Qg=fg.
$$

Then

$$
V^*Q^nV=\Phi(f^n).
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-033: Double-layer realizes the perturbation family|EVIDENCE - CC-033]].**
With $T=f(A)$, the companion identity gives

$$
E_n
=2V^*Q^{*n}V-T^{*n}
=2\Phi(f^n)^*-f(A)^{*n}
=\alpha(f^n)(A).
$$

The holomorphic functional calculus is commutative, so $E_nT=TE_n$. Bounded
$\alpha$ and bounded functional calculus give

$$
\|E_n\|
\le\|\theta\|\,\|\alpha\|\,\|f^n\|_\infty
\le\|\theta\|\,\|\alpha\|.
$$

All hypotheses of the perturbation lemma are now present.

## Application-level product bound {#application-level-product-bound}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-034: Functional calculus independently bounds E-n T-n|INFERENCE - CC-034]].**
The application has an additional route to the same terminal boundedness:

$$
E_nT^n
=\alpha(f^n)(A)f(A)^n
=\bigl(\alpha(f^n)f^n\bigr)(A).
$$

Thus

$$
\|E_nT^n\|
\le\|\theta\|\,\|\alpha\|\,\|f^n\|_\infty^2
\le\|\theta\|\,\|\alpha\|.
$$

This argument uses the function-algebra realization and is not part of the
abstract lemma. It is a second justification, not a replacement for the direct
Equation 1 estimate.

## Rational consequence {#rational-consequence}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-035: Lorist-Schwenninger rational two-spectral-set conclusion|SOURCE CLAIM - CC-035]].**
After the standard finite-dimensional and smooth-outer-domain reductions, the
lemma gives

$$
\|f(A)\|\le2\sup_{z\in W(A)}|f(z)|
$$

for rational $f$ with poles off $\overline{W(A)}$. The source states this
as the bounded-operator 2-spectral-set theorem.

## Abstract uniform-algebra variant {#abstract-uniform-algebra-variant}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-036: Lorist-Schwenninger abstract uniform-algebra variant|SOURCE CLAIM - CC-036]].**
The source also states an abstract version. Let $\mathcal A$ be a commutative
uniform algebra, let $\alpha:\mathcal A\to\mathcal A$ be unital, bounded, and
antilinear, and let
$\theta:\mathcal A\to\mathcal L(H)$ be a unital bounded homomorphism. If

$$
\Phi=\frac12\left(
\theta(\cdot)+\theta(\alpha(\cdot))^*
\right)
$$

is completely positive, then $\|\theta\|\le2$. Arveson extension and
Stinespring dilation produce the $V,Q$ interface for the perturbation lemma.

## Completely bounded boundary {#completely-bounded-boundary}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-037: Neither proof directly yields the completely bounded case|SOURCE CLAIM - CC-037]].**
Lorist and Schwenninger explicitly state that neither their route nor Jin's
directly yields the completely bounded case. In their lemma, the commutativity
of $E_n$ with $T$ is not preserved by the required matrix amplification.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

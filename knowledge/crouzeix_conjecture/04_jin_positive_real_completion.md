---
id: crouzeix-jin-positive-real-completion
title: Jin positive-real completion derivation
type: derivation
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, positive-real, herglotz-kernel, gramian]
confidence: medium
canonical: 04_jin_positive_real_completion.md
---

# Jin positive-real completion derivation

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

The completion theorem is the finite-dimensional core of Jin's audited v4
argument. This derivation keeps every hypothesis visible because dropping one
of them changes either the correction algebra, sampled-kernel positivity, or
the norm endpoint.

## Complete interface {#complete-interface}

Write

$$
B=S\operatorname{diag}(\beta_1,\ldots,\beta_n)S^{-1},\qquad
T=S\operatorname{diag}(\lambda_1,\ldots,\lambda_n)S^{-1}.
$$

The $\beta_i$ are distinct. The $\lambda_i$ need not be.

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-011: Auxiliary basis permits repeated target values|EVIDENCE - CC-011]].**
Simple spectrum is used to identify

$$
\operatorname{alg}(B^*)
=\{(S^{-1})^*DS^*:D\text{ diagonal}\}.
$$

The remaining hypotheses are

$$
|\lambda_i|\le1,\qquad
H\text{ analytic on }\mathbb D,\qquad H(0)=I,
$$

$$
\operatorname{Re}H(w)\succeq0,
$$

and

$$
H(w)-(I-wT)^{-1}\in\operatorname{alg}(B^*).
$$

Put $G=S^*S\succ0$. In the auxiliary basis, the defect is a diagonal analytic
matrix $\Theta(w)$, with $\Theta(0)=0$, and

$$
\widetilde H(w)=S^*H(w)S
=G(I-w\Lambda)^{-1}+\Theta(w)G.
$$

## Herglotz-kernel positivity {#herglotz-kernel-positivity}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-012: Herglotz kernel is positive|EVIDENCE - CC-012]].**
For analytic $F$ with positive semidefinite real part,

$$
\mathcal L_F(w,z)=\frac{F(w)+F(z)^*}{1-w\overline z}
$$

is a positive matrix kernel. For finite samples $w_i,\xi_i$, this means

$$
\sum_{i,j}\xi_i^*\mathcal L_F(w_i,w_j)\xi_j\ge0.
$$

One direct proof inserts

$$
\eta(\zeta)=\sum_j\frac{\xi_j}{1-\overline{w_j}\zeta}
$$

into the circle average of $2\operatorname{Re}F(r\zeta)$, then lets
$r\uparrow1$. No matrix-valued measure representation is needed.

## Prepare the two kernels {#prepare-the-two-kernels}

Define Hermitian matrices

$$
P_{ij}=\frac{G_{ij}}
{1-\overline{\lambda_i}\lambda_j/4},\qquad
Q_{ij}=\frac{G_{ij}}
{1-\overline{\lambda_i}\lambda_j/2},\qquad
Y=Q-P.
$$

The closed-disk bound on the $\lambda_i$ keeps all denominators nonzero.
For arbitrary $u=(u_i)$, choose the sample data

$$
w_i=\frac{\overline{\lambda_i}}2,\qquad
\xi_i=u_ie_i,
$$

and add

$$
w_0=0,\qquad v=-G^{-1}Pu.
$$

Repeated $\lambda_i$, repeated $w_i$, and $w_i=w_0$ are all allowed:
positive-kernel sampling is indexed and does not require distinct points.

## Exact cancellation {#exact-cancellation}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-013: Origin sample cancels the correction|EVIDENCE - CC-013]].**
The contribution of the diagonal correction to the sampled quadratic form is

$$
2\operatorname{Re}\sum_{i=1}^n
\overline{u_i}\theta_i(w_i)
\left(
(Gv)_i+
\sum_{j=1}^n
\frac{G_{ij}u_j}{1-w_i\overline{w_j}}
\right).
$$

Because

$$
1-w_i\overline{w_j}
=1-\overline{\lambda_i}\lambda_j/4,
$$

the parenthesis is exactly $(Gv+Pu)_i$. The chosen origin vector gives

$$
Gv+Pu=0.
$$

The unknown analytic correction has disappeared algebraically. It was not
estimated, truncated, or assigned a favorable sign.

## Ordered pre-Gramian inequality {#ordered-pre-gramian-inequality}

The resolvent contribution gives the sample-sample block $4Q-2P$, both
sample-origin blocks $G+Q$, and the origin block $2G$. Evaluate the block
quadratic form only on the graph $v=-G^{-1}Pu$:

$$
\begin{aligned}
0
&\le
\begin{pmatrix}u\\v\end{pmatrix}^{\!*}
\begin{pmatrix}
4Q-2P&G+Q\\
G+Q&2G
\end{pmatrix}
\begin{pmatrix}u\\v\end{pmatrix}\\
&=
u^*(4Y-YG^{-1}P-PG^{-1}Y)u.
\end{aligned}
$$

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-014: Ordered pre-Gramian inequality|EVIDENCE - CC-014]].**
Since $u$ is arbitrary,

$$
4Y-YG^{-1}P-PG^{-1}Y\succeq0.
$$

The order of the noncommuting factors is part of the claim. The source does not
assert that the displayed $2\times2$ block is positive for independent
$u,v$.

## Balance and expose the Gramians {#balance-and-expose-the-gramians}

Set

$$
\widetilde T=G^{1/2}\Lambda G^{-1/2},\qquad
\widehat P=G^{-1/2}PG^{-1/2},
$$

$$
\widehat Q=G^{-1/2}QG^{-1/2},\qquad
\widehat Y=\widehat Q-\widehat P.
$$

Geometric expansion gives norm-convergent series

$$
\widehat P=
\sum_{k=0}^{\infty}4^{-k}\widetilde T^{*k}\widetilde T^k,
\qquad
\widehat Q=
\sum_{k=0}^{\infty}2^{-k}\widetilde T^{*k}\widetilde T^k.
$$

Therefore

$$
\widehat Y=
\sum_{k=1}^{\infty}
(2^{-k}-4^{-k})\widetilde T^{*k}\widetilde T^k
\succeq0.
$$

Congruence transforms the ordered inequality into

$$
4\widehat Y-\widehat Y\widehat P-\widehat P\widehat Y\succeq0.
$$

## Bound P-hat {#bound-p-hat}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-015: Gramian anticommutator bounds P-hat|EVIDENCE - CC-015]].**
Let $\widehat Pe=\alpha e$. If $\alpha>2$, evaluating the last inequality on
$e$ gives

$$
0\le2(2-\alpha)e^*\widehat Ye.
$$

Because $\widehat Y\succeq0$, this forces
$e^*\widehat Ye=0$. The $k=1$ term of $\widehat Y$ is
$\widetilde T^*\widetilde T/4$, so $\widetilde Te=0$. But then the series
for $\widehat P$ gives $\widehat Pe=e$, contradicting
$\alpha>2$. Hence

$$
I\preceq\widehat P\preceq2I.
$$

## First-term norm endpoint {#first-term-norm-endpoint}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-016: First nonconstant Gramian term gives norm two|EVIDENCE - CC-016]].**
The first nonconstant term in $\widehat P$ yields

$$
\frac14\widetilde T^*\widetilde T
\preceq\widehat P-I\preceq I.
$$

Thus $\|\widetilde T\|\le2$. Similarity would not preserve the norm, so the
last step uses the polar decomposition $S=UG^{1/2}$:

$$
T=U\widetilde TU^*.
$$

Unitary invariance gives $\|T\|\le2$.

## What the completion theorem does and does not do {#completion-boundary}

It turns a particular positive-real completion into the sharp norm bound. It
does not construct $H$; the double-layer Cayley argument owns that
obligation. It does not require distinct target values. It does not prove a
matrix-amplified bound, and it does not license reordering the Gramian factors.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

---
id: cft-harp-mathematical-audit
title: Harp mathematical audit
type: reference
status: active
created: 2026-09-07
updated: 2026-09-07
tags: [crouzeix-textbook, mathematics, harp, lean, audit]
confidence: medium
canonical: harp_mathematical_audit.md
---

# Harp mathematical audit

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]

This supplement asks a narrower question than whether a proof file compiles.
Does its statement mean the inequality we intend, and do its constructions
supply the hypotheses consumed at each step?

**Audit status.** The expanded statement and its proof-only checks compile
locally. Separate agent reviews checked the statement correspondence and
the proof exposition; this is not independent human peer review, which remains
pending. Repository acceptance is recorded separately in the workstream review.
This unindexed supplement adds neither a chapter nor a registered theorem to
the 216-row coverage roster.

## The statement before the proof

Let $n$ be a nonempty finite index set, $A\in\mathbb C^{n\times n}$, and
$p(z)=\sum_{k=0}^d a_kz^k$ a scalar complex polynomial. Define

$$
p(A)=\sum_{k=0}^d a_kA^k,\qquad
W(A)=\{x^\dagger Ax:x\in\mathbb C^n,\ \|x\|_2=1\}.
$$

Here $A^0=I$, the dagger means conjugate transpose, and
$\|A\|_{2\to2}=\sup_{\|x\|_2=1}\|Ax\|_2$. The target is

$$
\forall A\ \forall p,\qquad
\|p(A)\|_{2\to2}\le2\sup_{z\in W(A)}|p(z)|.
$$

The constant does not depend on dimension or polynomial degree. Scalar
coefficients matter. A matrix-valued polynomial bound is a different
statement and does not follow by changing notation.

Lean's finite index type has Fintype, DecidableEq, and Nonempty instances.
Decidable equality supplies finite-coordinate bookkeeping. Nonemptiness
records positive dimension: a zero-dimensional space has no unit vector.
We do not use a totalized supremum operation to conceal an empty numerical
range.

Lean correspondence: MainTheoremStatement, PolynomialCrouzeixBound, and
maxPolynomialModulusOnNumericalRange in
[Statements.lean](../../formalization/lean/CrouzeixConjecture/Statements.lean).
Polynomial evaluation is Polynomial.aeval A p, not entrywise evaluation;
see [Definitions.lean](../../formalization/lean/CrouzeixConjecture/Definitions.lean).
The independent expansion is in
[HarpStatementAudit.lean](../../formalization/lean/CrouzeixTextbook/HarpStatementAudit.lean).
Its equivalence compares statements; it is not another terminal proof.

### The unindexed Lean checks

The namespace is CrouzeixTextbook.HarpStatementAudit. The proposition is
universe-polymorphic; the boundary examples use ordinary finite index types.
The declaration textbookBound_iff_mainTheoremStatement is proved here by
definitional reduction. It does not invoke the terminal theorem.
The supporting declarations polynomialModulusImage_nonempty,
polynomialModulusImage_isCompact, polynomialModulusImage_bddAbove and
polynomialModulusImage_maximum reuse the maintained numerical-range and
compactness proofs. They establish well-definedness of the right-hand side,
not the terminal inequality.

The test namespace CrouzeixTextbook.HarpStatementAuditTests contains the
named zeroMatrix_bound, constantPolynomial_bound, oneDimensional_bound,
nilpotentTwo_norm, nilpotentTwo_X_maximum, rankOneTwo_norms and
weightedShiftTwo_norms checks. The theorem vanishing_eval_of_textbookBound
explicitly assumes textbookBound. These source-level distinctions are not
additional indexed coverage classifications.

Four compiler fixtures try to reuse a proof at the wrong exact type:
a Frobenius norm, an omitted unit-vector condition, reordered quantifiers,
and an added terminal premise. A positive control accepts the intended type.
The reordered statement is also proved logically equivalent, while the
added-premise statement is proved tautological. Rejection here tests exact
correspondence, not the falsity of every altered statement.

## Check 1: Which norm and which supremum?

Use $\|x\|_2^2=\sum_i|x_i|^2$ and
$\langle x,y\rangle=\sum_i\overline{x_i}y_i$. Thus

$$
\langle x,Ax\rangle=\sum_{i,j}\overline{x_i}A_{ij}x_j=x^\dagger Ax.
$$

The first inner-product variable is conjugate-linear. Switching the arguments
without conjugating the result changes the quadratic form.

The project selects Mathlib's Matrix.Norms.L2Operator instance. Its norm is
definitionally the norm of Matrix.toEuclideanCLM A, the associated continuous
linear operator on Euclidean space. The declaration
matrix_norm_eq_euclidean_operator_norm proves this identification by reflexivity.
The coordinate check is inner_euclideanOperator_eq_star_dotProduct. Both are in
[Euclidean.lean](../../formalization/lean/CrouzeixConjecture/Euclidean.lean).

Two nonnormal examples distinguish the relevant measurements:

$$
R=\begin{bmatrix}1&1\\0&0\end{bmatrix},
\qquad H=\begin{bmatrix}0&2\\1&0\end{bmatrix}.
$$

For $R$, $\|R(x,y)\|_2^2=|x+y|^2\le2(|x|^2+|y|^2)$, with equality on
$(1,1)/\sqrt2$. Its induced norm is $\sqrt2$, although its largest entry
magnitude is $1$. For $H$,
$\|H(x,y)\|_2^2=|x|^2+4|y|^2\le4(|x|^2+|y|^2)$, with equality at
$(0,1)$. Its induced norm is $2$, whereas its Frobenius norm, the square root
of the sum of squared entry magnitudes, is $\sqrt5$.
The examples need not separate every pair of norms simultaneously.

Now justify the supremum. A coordinate unit vector shows $W(A)$ is nonempty.
The unit sphere is compact in finite dimension and
$x\mapsto\langle x,Ax\rangle$ is continuous. Its image $W(A)$ is compact.
Applying the continuous function $z\mapsto|p(z)|$ gives a nonempty compact
real set, hence a bounded set attaining its supremum.

Sources: numericalRange_nonempty in
[Definitions.lean](../../formalization/lean/CrouzeixConjecture/Definitions.lean),
isCompact_numericalRange in
[NumericalRange.lean](../../formalization/lean/CrouzeixConjecture/NumericalRange.lean),
and exists_maxPolynomialModulusOnSet in
[OuterApproximationLimit.lean](../../formalization/lean/CrouzeixConjecture/OuterApproximationLimit.lean).
The last theorem explicitly requires compactness and nonemptiness.

**Disposition.** The maintained norm and supremum have the intended meaning.
Final acceptance also requires the expanded-statement and boundary checks.

## Check 2: Two normalizations

For a nonzero finite positive measure $\mu$ on a compact carrier and a
continuous finite-dimensional real observable $g$, positive cubature supplies
finitely many nodes $s_j$ and weights $w_j>0$ satisfying

$$
\sum_jw_j=\mu(\text{whole space}),\qquad
\sum_jw_jg(s_j)=\int g\,d\mu.
$$

Write $a=\mu(\text{whole space})>0$ and $\nu=\mu/a$. The integral against
the probability measure $\nu$ belongs to the convex hull of the compact
image of $g$. Carathéodory reduction expresses it as
$\sum_j\alpha_jg(s_j)$ with positive retained weights summing to one.
Multiplication by $a$ gives $w_j=a\alpha_j$. Zero weights are omitted.

The formal providers are exists_positive_cubature in
[PositiveCubature.lean](../../formalization/lean/Crouzeix/Harp/PositiveCubature.lean)
and exists_positive_cubature_finite_measure in
[FiniteMeasureCubature.lean](../../formalization/lean/Crouzeix/Harp/FiniteMeasureCubature.lean).
Their continuity, support, compactness, finite-dimensionality and measure
hypotheses remain part of the theorem.

Here is the boundary data that supplies these hypotheses. Fix a bounded convex
open domain $\Omega$ containing $W(B)$, its compact parameterized supported
boundary $\Gamma$, and its nonzero finite parameter measure $\mu$. At each
parameter $s$, write $\zeta=\Gamma(s)$, outward unit normal $\nu(s)$, and
nonnegative speed $\sigma(s)$. The point, normal and speed are continuous
maps on the compact parameter space. The supported-boundary condition says that
the domain lies on the inner side of the tangent support line. Define

$$
F(s)=\frac{\sigma(s)}{2\pi}\nu(s)(\zeta I-B)^{-1},\quad
D(s)=F(s)+F(s)^\dagger,\quad
\Phi(f)=\frac12\int f(s)D(s)\,d\mu(s).
$$

Since the spectrum is contained in $W(B)\subset\Omega$, $\zeta I-B$ is
invertible. Set $R=(\zeta I-B)^{-1}$ and
$G=\operatorname{Re}(\overline{\nu(s)}(\zeta I-B))$, where
$\operatorname{Re}C=(C+C^\dagger)/2$. For a unit vector $x$,
$\langle x,Gx\rangle=\operatorname{Re}(\overline{\nu(s)}
(\zeta-\langle x,Bx\rangle))\ge0$ by support. Scaling gives the inequality
for every $x$, so $G\succeq0$. Direct multiplication gives

$$
D(s)=\frac{\sigma(s)}{\pi}R^\dagger G R\succeq0.
$$

The polynomial Cauchy formula for this boundary and measure states
$\int r(\Gamma(s))F(s)\,d\mu(s)=r(B)$ for every polynomial $r$.
For $r=1$, this gives $\int F\,d\mu=I$ and hence $\int D\,d\mu=2I$.
A finite measure alone does not imply this formula; it is a separate
hypothesis, supplied by the oriented boundary in the terminal proof.
The definitions and positivity providers are in
[ParametricBoundary.lean](../../formalization/lean/CrouzeixConjecture/ParametricBoundary.lean);
the mass and companion identities are in
[ParametricDoubleLayerIdentity.lean](../../formalization/lean/CrouzeixConjecture/ParametricDoubleLayerIdentity.lean).

Harp bundles all moments $h(s)^kD(s)$ for $0\le k\le N+1$ into one
real finite-dimensional observable. Here $D(s)$ is the positive
double-layer matrix and $h(s)=q(\Gamma(s))$. One cubature formula preserves
every moment simultaneously. Independent choices for each power would not
supply the common compression needed later.

Absorb scalar weights into matrices $B_j=w_jD(s_j)\succeq0$. The zeroth
moment and the double-layer mass formula give

$$
\sum_jB_j=2I.
$$

This matrix identity is not $\sum_jw_j=2$.
The providers are exists_positive_matrix_moment_cubature in
[FiniteAtomicDilation.lean](../../formalization/lean/Crouzeix/Harp/FiniteAtomicDilation.lean)
and finiteAtomicDoubleLayer_mass_eq_two_one in
[FiniteAtomicL2Dilation.lean](../../formalization/lean/Crouzeix/Harp/FiniteAtomicL2Dilation.lean).
The latter consumes a polynomial Cauchy formula.

**Disposition.** Scalar measure mass and matrix mass are kept separate.
The boundary construction must supply the Cauchy-formula hypothesis.

## Check 3: Square roots and adjoint orientation

On the finite direct sum $\mathcal K_N=\bigoplus_j\mathbb C^n$, with
counting-measure $L^2$ norm, define

$$
(V_Nx)_j=2^{-1/2}B_j^{1/2}x,\qquad (Q_Ny)_j=h(s_j)y_j.
$$

Positivity gives a self-adjoint square root with
$(B_j^{1/2})^\dagger B_j^{1/2}=B_j$. Therefore

$$
\|V_Nx\|^2=\frac12\sum_j\langle x,B_jx\rangle
=\frac12\langle x,2Ix\rangle=\|x\|^2.
$$

The factor $2^{-1/2}$ is necessary. Omitting it gives
$V_N^\dagger V_N=2I$, not an isometry. The boundary normalization
$|h(s_j)|\le1$ also gives
$\|Q_Ny\|^2=\sum_j|h(s_j)|^2\|y_j\|^2\le\|y\|^2$.

For $k\le N+1$, the preserved moment gives

$$
V_N^\dagger Q_N^kV_N=\frac12\sum_jh(s_j)^kB_j=\Phi(h^k).
$$

Taking adjoints yields
$V_N^\dagger(Q_N^\dagger)^kV_N=\Phi(h^k)^\dagger$.
The Cauchy/companion identity now supplies

$$
E_k=2V_N^\dagger(Q_N^\dagger)^kV_N-(T^\dagger)^k.
$$

Adjoint powers cannot be replaced by unadjointed powers merely because both
multipliers are contractions.

Lean support is shared explicitly. boundaryEmbeddingField,
norm_boundaryEmbeddingToLp and boundaryEmbedding in
[BoundaryEmbedding.lean](../../formalization/lean/Crouzeix/LoristSchwenninger/BoundaryEmbedding.lean)
construct the normalized isometry. The power identity is
boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding in
[CompressionMoments.lean](../../formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean).
Harp's finiteAtomic_compression_eq_of_moments and finiteAtomicL2DilationWitness
transport and package these results in
[FiniteAtomicL2Dilation.lean](../../formalization/lean/Crouzeix/Harp/FiniteAtomicL2Dilation.lean).

**Disposition.** Normalization and adjoint orientation match the recurrence
interface. Moment preservation stops at the specified horizon.

## Check 4: What remains fixed?

Fix the boundary data from Check 2 and a polynomial $q$ with
$|q(z)|\le1$ on $\overline\Omega$. Put $h=q\circ\Gamma$ and, identifying
matrices with their Euclidean operators, define

$$
T=q(B),\qquad E_k=\int\overline{h(s)^k}F(s)\,d\mu(s),\qquad
L=\int\|F(s)\|\,d\mu(s).
$$

The resolvent, point, normal and speed are continuous, so $F$ is continuous
and bounded on the compact parameter space. The measure is finite; therefore
$L<\infty$. This continuity also makes each matrix-moment observable
continuous, as required by cubature. Since $|h|\le1$, the integral norm inequality gives
$\|E_k\|\le\int |h|^k\|F\|\,d\mu\le L$ independently of $k$.
Each resolvent commutes with $B$ and hence with $q(B)$. Multiplication by
this fixed matrix commutes with the integral, proving $E_kT=TE_k$.
Finally Cauchy's formula and conjugate transpose of the integral give

$$
2\Phi(h^k)=\int h^kF\,d\mu+\int h^kF^\dagger\,d\mu
=T^k+E_k^\dagger.
$$

The providers parametricBoundaryCompanion_norm_le and
parametricBoundaryCompanion_commute_of_mem_generatedAlgebra are in
[CompanionAlgebra.lean](../../formalization/lean/Crouzeix/LoristSchwenninger/CompanionAlgebra.lean).
The polynomial-power Cauchy provider is in
[PolynomialPowerCauchy.lean](../../formalization/lean/Crouzeix/LoristSchwenninger/PolynomialPowerCauchy.lean).
The declaration finiteHorizonPolynomialCore packages these hypotheses.
Only then choose the finite witnesses:

$$
\text{fixed }(T,(E_k),L),\qquad
\forall N\ \exists(\mathcal K_N,V_N,Q_N)\
\forall k\le N+1:\ \text{compression identity}.
$$

One finite space need not realize all powers. Conversely, $T$, $E_k$, and
$L$ may not change with $N$ during the scalar limit.
CommutingPerturbationData and FiniteHorizonDilationData in
[FiniteHorizonDilation.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonDilation.lean)
encode this separation.

For a fixed unit vector $x$, put
$m_k=\operatorname{Re}\langle x,E_kT^kx\rangle$.
The compression identity and contraction imply
$\|T^k\|\le2+L$ on the realized horizon. Consequently

$$
|m_k|\le\|E_k\|\,\|T^k\|\le L(2+L),\qquad k\le N+1.
$$

For each fixed $k$, choose a horizon containing it. The bound is then valid
at every index of the common sequence. This is recurrenceScalar_abs_le in
the same module. A collection of unrelated horizon-specific bounds would not
justify this inference.

**Disposition.** The fixed-core quantifiers support the later limit.
Finite-dimensional norm attainment occurs in the original space, not a
hypothetical common dilation space.

## Check 5: The finite recurrence and its sign

Assume $\kappa=\|T\|>1$. Finite-dimensional norm attainment supplies
$\|x\|=1$ with $T^\dagger Tx=\kappa^2x$. For one witness define

$$
d_N=Q_N^\dagger V_NTx-\kappa V_Nx,\qquad b_N=\|d_N\|^2.
$$

Write $C_k=2V_N^\dagger(Q_N^\dagger)^kV_N$,
$u_k=(T^\dagger)^kx$, and

$$
v_{N,k}=(C_{k+1}T-\kappa C_k)x
=2V_N^\dagger(Q_N^\dagger)^kd_N.
$$

The last equality factors adjacent powers. Contractivity gives
$\|v_{N,k}\|\le2\|d_N\|$. Commutation and the adjoint identity first rewrite
$m_k=\operatorname{Re}\langle E_kx,u_k\rangle$ and
$m_{k+1}=\operatorname{Re}\langle E_{k+1}Tx,u_k\rangle$. Substitute
$E_k=C_k-(T^\dagger)^k$ and the singular-vector equation:

$$
\begin{aligned}
\kappa E_kx-E_{k+1}Tx
&=-v_{N,k}-\kappa u_k+(T^\dagger)^{k+1}Tx\\
&=-v_{N,k}+(\kappa^2-\kappa)u_k.
\end{aligned}
$$

Pairing with $u_k$ and taking real parts yields the exact identity

$$
\kappa m_k-m_{k+1}
=(\kappa^2-\kappa)\|u_k\|^2
-\operatorname{Re}\langle v_{N,k},u_k\rangle.
$$

For $a>0$, completing the square gives
$a\|u\|^2-\operatorname{Re}\langle v,u\rangle\ge-\|v\|^2/(4a)$.
Apply this with $a=\kappa^2-\kappa>0$ to the recurrence identity:

$$
-\frac{b_N}{\kappa^2-\kappa}\le\kappa m_k-m_{k+1},
\qquad 1\le k\le N.
$$

Multiply the $k$th inequality by $\kappa^{-k}$ and sum. The intermediate
moments cancel, leaving

$$
\kappa^{-N}m_{N+1}
+\left(\sum_{k=1}^N\kappa^{-k}\right)
  \frac{-b_N}{\kappa^2-\kappa}\le m_1.
$$

The terminal moment remains at finite $N$. The source declarations
adjacentPowerDefect_factorization, recurrence_difference_identity,
recurrence_difference_lower_bound and equation_three_finite_lower_bound are in
[FiniteHorizonOperatorRecurrence.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean).

Put $a_N=Q_N^\dagger V_NTx$. The singular-vector and first-moment
identities give
$m_1=2\operatorname{Re}\langle a_N,V_Nx\rangle-\kappa^2$ and
$\|a_N\|\le\kappa$. Expanding,

$$
\begin{aligned}
b_N&=\|a_N\|^2-2\kappa\operatorname{Re}\langle a_N,V_Nx\rangle+\kappa^2\\
&\le2\kappa^2-\kappa(m_1+\kappa^2)
=2\kappa^2-\kappa m_1-\kappa^3=:C.
\end{aligned}
$$

Since $b_N\ge0$, also $C\ge0$. The coefficient has a minus sign:

$$
-\frac{C}{\kappa^2-\kappa}\le-\frac{b_N}{\kappa^2-\kappa}.
$$

Multiplying by a nonnegative sum of weights preserves this direction.
Substituting $C$ makes the finite left side smaller. The providers are
displacementSq_le in the operator recurrence module and
norm_target_le_two_of_finiteHorizonDilationData in
[FiniteHorizonPerturbation.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonPerturbation.lean).

**Disposition.** Denominator, sign and terminal term agree.
The quantitative supplement will expose the remaining finite error.

## Check 6: Three limits, in order

For the fixed normalized data, boundedness gives
$\kappa^{-N}m_{N+1}\to0$. The geometric weights tend to $1/(\kappa-1)$.
Taking this first limit yields

$$
-\frac{C}{\kappa(\kappa-1)^2}\le m_1.
$$

Multiply by the positive denominator and substitute the definition of $C$.
After collecting terms the inequality becomes
$(\kappa-2)(m_1-1)\ge0$ after dividing by $\kappa^2>0$.
If $\kappa>2$, this forces $m_1\ge1$, whereas $C\ge0$ gives
$m_1\le2\kappa-\kappa^2<0$. This contradiction proves $\kappa\le2$.
The case $\|T\|\le1$ needs no division.

Lean correspondence: finite_weighted_inequalities_to_limit_lower_bound in
[FiniteHorizonRecurrence.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonRecurrence.lean)
and scalar_endpoint_le_two in
[Scalar.lean](../../formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean).

Next fix an outer domain $\Omega$ and approximate $A$ by simple-spectrum
matrices $B_j$ with $W(B_j)\subset\Omega$ eventually. Set
$M=\max_{\overline\Omega}|p|$. If $M>0$, apply the normalized argument to
$q=p/M$, obtaining $\|p(B_j)\|\le2M$. Continuity of matrix polynomial
evaluation permits $j\to\infty$, with the domain still fixed.

If $M=0$, do not form $p/M$. Every eigenvalue of $B_j$ lies in
$\overline\Omega$, so $p$ vanishes there. The simple-spectrum $B_j$ is
diagonalizable, hence $p(B_j)=0$. The limit again proves the bound.
This does not assert diagonalizability of $A$.

Finally shrink the outer domains toward $W(A)$. Their compact closures
lie in a common compact neighborhood and approach $W(A)$ in distance.
Continuity of $|p|$ on that neighborhood makes their maxima converge to
$\max_{W(A)}|p|$. The last limit proves the intended bound.

The nesting and zero branch are in harpFiniteHorizonMainTheorem in
[MainTheorem.lean](../../formalization/lean/Crouzeix/Harp/MainTheorem.lean).
The transport providers are norm_polynomialEval_le_of_tendsto in
[Limiting.lean](../../formalization/lean/CrouzeixConjecture/Limiting.lean)
and tendsto_maxPolynomialModulusOnSet_of_outerApproximation in
[OuterApproximationLimit.lean](../../formalization/lean/CrouzeixConjecture/OuterApproximationLimit.lean).

**Disposition.** The order is horizon, matrix approximation, outer domain.
A different order would need further uniform estimates.

## Check 7: Harp's contribution and shared support

Harp constructs positive finite cubature, realizes the sampled moments on
finite counting-measure dilations, and proves the horizon-indexed perturbation
argument. It shares boundary embedding, multiplication, companion algebra,
norm attainment, completed-square estimates, scalar recurrence support and
the scalar endpoint with the Lorist–Schwenninger development.

The relevant independence claim is that Harp does not invoke another route's
terminal Crouzeix theorem. It is not independence from all LS mathematics.
The maintained route validator checks prohibited imports; kernel compilation
alone does not describe that provenance for readers.

In [Consequences.lean](../../formalization/lean/Crouzeix/Harp/Consequences.lean),
harpFiniteHorizonFiniteMatrixMainTheorem supplies every standard finite
dimension. Neutral adapters give the rational bound with its pole
restriction that poles avoid $W(A)$ in finite dimension, and bounded-operator results on complete, nontrivial complex
Hilbert spaces. The spectral-set form uses the closed numerical range.
In that Hilbert-space spectral-set form, poles must avoid the closed
operator numerical range, not just the spectrum.
These extra hypotheses and scopes are not silently part of the first
finite-matrix polynomial statement.

**Historical context and provenance.** Route names describe repository
developments and dependencies, not priority or external acceptance. See the
[[knowledge/crouzeix_textbook/source_registry|source registry]] for literature
and captured source records.

**Disposition.** Shared support and the terminal provider are distinguishable.
Fresh provider/axiom checks remain part of acceptance. Standard Lean axioms,
including classical choice, must be named accurately; they are not hidden
project-specific assumptions.

## Boundary checks beside the theorem

For $A=0$, $p(A)=p(0)I$ and $W(A)=\{0\}$, so the statement reduces to
$|p(0)|\le2|p(0)|$. For a constant polynomial $p=c$, it reduces to
$|c|\le2|c|$. Positive dimension ensures $\|I\|_{2\to2}=1$.

In dimension one, $A=[a]$, $W(A)=\{a\}$ and $p(A)=[p(a)]$.
The special case has constant one already; it satisfies the universal
constant-two statement.

For the nonzero nilpotent

$$
S=\begin{bmatrix}0&1\\0&0\end{bmatrix},
\qquad S^2=0,\qquad \|S\|_{2\to2}=1,
$$

the quadratic form on a unit vector is $\overline{x_1}x_2$.
Its modulus is at most $(|x_1|^2+|x_2|^2)/2=1/2$, with equality at
$x_1=x_2=1/\sqrt2$. Thus $\max_{W(S)}|z|=1/2$, and $p(z)=z$ attains
the constant two. Eigenvalues alone miss this effect: both are zero.
No diagonalization of $S$ is used.

If $p$ vanishes on $W(A)$, the terminal inequality gives $\|p(A)\|\le0$,
so $p(A)=0$. In the proof, the outer maxima can remain positive at every
stage and merely tend to zero. This boundary must not be confused with the
zero-outer-maximum branch.

Named unindexed formal checks are in
[HarpStatementAuditTests.lean](../../formalization/lean/CrouzeixTextbook/HarpStatementAuditTests.lean).
A consequence that assumes the terminal proposition is labeled as such;
it does not establish that proposition.

## Motivation for ML researchers

A fixed linear update $x_{t+1}=Ax_t$ applies $A^t$ after $t$ steps.
A polynomial filter applies $p(A)$. The induced norm measures worst-case
Euclidean amplification. The numerical range retains information the
spectrum of a nonnormal matrix can hide, as the nilpotent calculation shows.

**ML analogy.** A fixed Jacobian can motivate the calculation under explicit
locality assumptions. A changing sequence of Jacobians gives a product of
different matrices, not a polynomial in one matrix. This supplement proves
neither nonlinear-training stability nor floating-point certification.
The finite cubature construction remains existential exact mathematics.

---
id: cft-chapter-36-harp-finite-horizon-proof
title: The Harp finite-horizon proof
type: textbook-chapter
status: active
created: 2026-09-04
updated: 2026-09-04
tags: [mathematics, lean, crouzeix-textbook, chapter36, part6]
canonical: 36_harp_finite_horizon_proof.md
chapter: 36
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 36: The Harp finite-horizon proof

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI: Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries|Chapter 35: Comparison, verification, and boundaries]]
Next: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Return to the book index]]

## Opening problem

How can finite objects prove a bound whose argument uses arbitrarily high powers? Harp answers this by constructing a different finite object for each requested horizon. The objects agree on the quantities that enter a scalar recurrence. A uniform estimate then allows a limit of scalar inequalities.

Our destination is the polynomial estimate

$$
\|p(A)\|\le 2\max_{z\in W(A)}|p(z)|,
\qquad W(A)=\{\langle x,Ax\rangle:\|x\|=1\},
$$

for every complex matrix of positive finite dimension. The norm is the Euclidean operator norm. We use the conjugate-linear first-slot convention
$\langle x,y\rangle=x^*y$, so $\langle ax,y\rangle=\bar a\langle x,y\rangle$ and $\langle x,ay\rangle=a\langle x,y\rangle$.

An ML reader can keep a nonnormal linear layer in mind. Repeated application of a matrix, or a polynomial filter built from it, can amplify inputs even when its eigenvalues look harmless. That is motivation for estimating $\|p(A)\|$. It supplies no step of the proof. Our task is to track exactly which finite measurements suffice for an operator inequality.

Harp is a locally derived finite-horizon reconstruction. It shares lower-level support with the Lorist–Schwenninger development, including boundary embeddings and scalar estimates. This chapter makes no independent-manuscript or priority claim. Five teaching interfaces are explicit reexports. The sixth packages an existing data witness as a proposition asserting existence. The proofs below explain the mathematics behind those interfaces without changing that provenance.

## Conceptual model

First fix a matrix $B$, a suitable outer domain $\Omega$, and a polynomial $q$ bounded by one on $\overline\Omega$. Put $T=q(B)$. Boundary integration constructs a companion family $E_k$ which commutes with $T$ and has one uniform norm bound $L$. These are the fixed data.

For each integer $N\ge0$, positive cubature preserves the matrix moments of orders $0,\ldots,N+1$. Square roots of the sampled positive matrices give an isometry $V_N$ into a finite Hilbert space $K_N$. A diagonal multiplier $Q_N$ is a contraction and satisfies

$$
E_k=2V_N^*(Q_N^*)^kV_N-(T^*)^k,
\qquad 0\le k\le N+1.
$$

The order of quantifiers is essential:

$$
\text{one }(T,(E_k)_{k\ge0},L),\qquad
\forall N\ \exists(K_N,V_N,Q_N)\text{ realizing these moments}.
$$

There is no requirement of nested node sets, compatible embeddings between the $K_N$, or one finite space realizing all powers. The scalar sequence extracted from $T,E_k$ is common to every horizon. The displacement measured in $K_N$ can vary, but one common upper bound controls all such displacements. This distinction is the reason the scalar limit is legitimate.

Prerequisites are finite-dimensional spectral theory for positive matrices, adjoints, polynomial matrix evaluation, compactness, convex combinations, and geometric series. Boundary Cauchy evaluation is stated as a precise hypothesis before it is supplied by the geometric provider. Chapters 33–35 provide the companion argument and neutral extension results, but the decisive finite-horizon algebra is developed here.

## Formal development

### CFT-36-001 — Positive cubature for a common moment tuple {#cft-36-001}

#### Purpose

Replace finitely many integrals by one positive finite sum.

#### Statement

Let $X$ carry a finite nonzero measure $\mu$, let $\alpha\subseteq X$ be a compact measurable set supporting $\mu$ almost everywhere, and let $g:X\to\mathcal E$ be continuous on $\alpha$, where $\mathcal E$ is a finite-dimensional real normed space. There are nodes $s_j\in\alpha$ and real weights $w_j>0$ such that

$$
\sum_{j\in J}w_jg(s_j)=\int g\,d\mu,
\quad \sum_{j\in J}w_j=\mu(X),
\quad |J|\le\dim_{\mathbb R}\mathcal E+1.
$$

#### Hypothesis ledger

The measure is finite and nonzero, the carrier is compact and measurable with almost-everywhere full measure, and the observable is continuous on that carrier with finite real-dimensional codomain. The measure is defined on a topological measurable space whose open sets are measurable. These hypotheses give integrability on the compact carrier. In the chapter interface the carrier is the entire compact boundary parameter space.

#### Proof roadmap

Prove compactness of the convex hull, represent a probability integral as a finite convex combination, then rescale by the measure's mass and project the moment tuple.

#### Proof

For a probability measure, the integral is a barycenter. To use that observation without silently replacing a convex hull by its closure, first prove that the convex hull of a nonempty compact set $S\subseteq\mathcal E$ is compact. Write $D=\dim_{\mathbb R}\mathcal E$. Carathéodory's theorem represents every point of the hull by at most $D+1$ points. For each $1\le l\le D+1$, consider the continuous barycentric map

$$
\Delta_{l-1}\times S^l\longrightarrow\mathcal E,
\qquad (a,z)\longmapsto\sum_{j=1}^{l}a_jz_j.
$$

The finite-node assertion itself follows by eliminating affine dependence. Start with a convex combination with strictly positive coefficients $a_j$. If it has more than $D+1$ points, there are real numbers $c_j$, not all zero, with $\sum_jc_j=0$ and $\sum_jc_jz_j=0$. Some $c_j$ is positive. Set $t=\min_{c_j>0}a_j/c_j>0$ and replace $a_j$ by $a_j-tc_j$. All coefficients remain nonnegative, their sum and barycenter are unchanged, and at least one becomes zero. Delete zero terms and repeat. Each repetition decreases the finite number of points, so the process reaches at most $D+1$.

The simplex and finite product are compact. Their image is compact, and the finite union of these images is exactly the convex hull. Thus the hull is closed as well as convex. In our application $S=g(\alpha)$ is nonempty because a nonzero measure is supported on $\alpha$.

Take $S=g(\alpha)$. The probability integral belongs to this closed convex hull because the integrand belongs to it almost everywhere. Indeed, if the integral $u$ were outside the compact convex hull, finite-dimensional separation would give a real linear functional $\ell$ and a number $a$ with $\ell(u)>a\ge\ell(z)$ for every point $z$ of the hull. But $\ell(u)=\int\ell(g)\,d\mu\le\int a\,d\mu=a$, a contradiction. This is the integral-in-convex-set step used by the provider. Carathéodory now gives a finite convex combination for the integral. Delete every term with zero coefficient; the remaining coefficients are strictly positive, retain total mass one, and use at most $D+1$ points. For each value in $g(\alpha)$, choose a preimage in $\alpha$.

For general finite nonzero $\mu$, set $a=\mu(X)>0$ and apply this argument to $\nu=a^{-1}\mu$. If $v_j$ are its coefficients, take $w_j=av_j$. Then

$$
\sum_jw_jg(s_j)=a\int g\,d\nu=\int g\,d\mu,
\qquad \sum_jw_j=a.
$$

For the application, define $h=q\circ\sigma$, where $\sigma$ parametrizes the boundary, and let $\rho$ be its positive matrix density, defined in the next card. Use the single observable

$$
g_N(s)=\big[h(s)^k\rho(s)\big]_{k=0}^{N+1}
\in\big(M_d(\mathbb C)\big)^{N+2}.
$$

Regard this space as real: a complex $d\times d$ matrix has $2d^2$ real coordinates. The resulting node count is at most $2(N+2)d^2+1$. Projecting the tuple identity onto coordinate $k$ yields

$$
\sum_jw_jh(s_j)^k\rho(s_j)=\int h(s)^k\rho(s)\,d\mu(s).
$$

The same nodes and weights work for every $k$. Choosing a separate cubature for each power would not provide the common multiplier needed later. The scalar sum of weights is $\mu(X)$, which in general equals neither one nor two.

#### Boundary case

At $N=0$ the tuple still contains powers zero and one. The zero moment will normalize the isometry, and the first moment will control displacement. If the measure were zero, a strictly positive representation would need a separate formulation; the stated theorem assumes nonzero mass.

#### Pedagogical prerequisites

The prerequisites are compact convex hulls and finite-dimensional integration. For the matrix application, CFT-26-006 and CFT-28-001 supply the positive-density and Cauchy background.

#### Lean correspondence

`harp_positive_moment_cubature` is a reexport of `Harp.exists_positive_matrix_moment_cubature`. Its card bound is stated using real `finrank`; the explicit $2(N+2)d^2+1$ count is also a checked field of the later witness.

Sources: [[formalization/lean/Crouzeix/Harp/PositiveCubature.lean|compact convex hull and positive cubature]], [proof](../../../formalization/lean/Crouzeix/Harp/PositiveCubature.lean#L28); [[formalization/lean/Crouzeix/Harp/FiniteMeasureCubature.lean|finite-measure rescaling]], [proof](../../../formalization/lean/Crouzeix/Harp/FiniteMeasureCubature.lean#L24); [[formalization/lean/Crouzeix/Harp/FiniteAtomicDilation.lean|matrix moment observable]], [definition and theorem](../../../formalization/lean/Crouzeix/Harp/FiniteAtomicDilation.lean#L33).

#### Historical context

Historical context. The local proof uses Carathéodory's convex-geometric theorem through Mathlib and adds the finite-measure and matrix-tuple specialization. This is the Harp cubature provider's provenance.

#### ML analogy

Motivation. Moment matching resembles preserving a finite collection of rollout statistics. Exact transfer: one positive weighted sample preserves all coordinates of this particular observable exactly. Nontransfer: this existence theorem does not give a sampling algorithm, floating-point error tolerance, or a guarantee for unlisted statistics.

### CFT-36-002 — From positive atoms to finite dilation {#cft-36-002}

#### Purpose

Construct the finite witness consumed by the recurrence.

#### Statement

Under the geometric, normalized-polynomial, and Cauchy hypotheses below, every horizon admits a finite-dimensional dilation witness for the fixed polynomial core, realizing powers through $N+1$.

#### Hypothesis ledger

Let the parameter space $X$ be compact, with measurable open sets and a finite nonzero ordinary parameter measure $\mu$. A `ParametricConvexBoundary` supplies continuous functions $\sigma:X\to\mathbb C$, $\nu:X\to\mathbb C$, and $v:X\to\mathbb R$, satisfying

$$
v(s)\ge0,\quad \sigma(s)\in\partial\Omega,\quad |\nu(s)|=1,
\quad \operatorname{Re}\big(\overline{\nu(s)}[\sigma(s)-z]\big)\ge0
\quad(z\in\Omega),
$$

with $\Omega$ open. In the geometric construction $v$ is the speed. Suppose $W(B)\subseteq\Omega$, $d\ge1$, and $|q(z)|\le1$ on $\overline\Omega$. In addition, require the Cauchy formula for every polynomial $a$:

$$
\int a(\sigma(s))F(s)\,d\mu(s)=a(B),
\qquad F(s)=\frac{v(s)}{2\pi}\nu(s)[\sigma(s)I-B]^{-1}.
$$

The geometric structure alone does not assert this analytic identity. The inverse exists because boundary points lie outside $W(B)$, hence outside the spectrum. Define

$$
\rho=F+F^*,\quad h=q\circ\sigma,\quad T=q(B),\quad
\Phi(f)=\frac12\int f\rho\,d\mu,\quad E_k=\int\overline{h^k}F\,d\mu.
$$

Then for each $N$ a finite-dimensional witness exists with isometric $V$, contractive $Q$, and the companion identity through $N+1$.

#### Proof roadmap

Prove positivity and total matrix mass, absorb the cubature weights, calculate the square-root embedding and compression, and identify the companion by Cauchy evaluation and adjoints.

#### Proof

Positivity is worth checking. Set $R=[\sigma I-B]^{-1}$. For any vector $y$, the supporting inequality applied to its normalized numerical-range value gives
$\operatorname{Re}\langle y,\bar\nu[\sigma I-B]y\rangle\ge0$. The identity

$$
\nu R+\bar\nu R^*
=R^*\big(\bar\nu[\sigma I-B]+\nu[\bar\sigma I-B^*]\big)R
$$

therefore proves $\rho\succeq0$, after multiplication by $v/(2\pi)$. Continuity and compactness make all the displayed integrals finite. Cauchy with $a=1$ gives $\int F\,d\mu=I$, hence $\int\rho\,d\mu=2I$.

Apply CFT-36-001 and absorb weights into the matrices

$$
D_j=w_j\rho(s_j)\succeq0,\qquad \sum_jD_j=2I.
$$

Use counting measure on the finite node set $J$. Then $K_N=\ell^2(J;\mathbb C^d)$ has inner product $\sum_j\langle f_j,g_j\rangle$. Positive matrix square roots define

$$
(Vx)_j=\frac1{\sqrt2}D_j^{1/2}x,
\qquad (Qf)_j=h(s_j)f_j.
$$

For all $x,y$,

$$
\langle Vx,Vy\rangle
=\frac12\sum_j\langle D_j^{1/2}x,D_j^{1/2}y\rangle
=\frac12\langle x,(\sum_jD_j)y\rangle
=\langle x,y\rangle.
$$

Thus $V^*V=I$. Since $|h(s_j)|\le1$,
$\|Qf\|^2=\sum_j|h(s_j)|^2\|f_j\|^2\le\|f\|^2$, so $\|Q\|\le1$. Its adjoint multiplies by $\overline{h(s_j)}$. Directly,

$$
V^*Q^kV=\frac12\sum_jh(s_j)^kD_j=\Phi(h^k)
\quad(0\le k\le N+1).
$$

Cauchy applied to $q^k$ gives $\int h^kF\,d\mu=T^k$. Taking adjoints under the other integral gives

$$
2\Phi(h^k)=\int h^kF\,d\mu+\int h^kF^*\,d\mu
=T^k+E_k^*.
$$

Take adjoints again and rearrange to obtain
$E_k=2V^*(Q^*)^kV-(T^*)^k$. Every resolvent of $B$ commutes with every polynomial of $B$; integration therefore gives $E_kT=TE_k$. Finally,

$$
\|E_k\|\le\int |h(s)|^k\|F(s)\|\,d\mu(s)
\le L:=\int\|F(s)\|\,d\mu(s).
$$

Neither this $L$ nor $T,E_k$ depends on the cubature horizon. The speed already occurs in $F$. Integrating this same formula against arclength once more would count that Jacobian twice.

#### Boundary case

A positive semidefinite $D_j$ can be singular or zero. Its square root still exists; no inverse square root is used. At $N=0$ the construction still realizes both required moments. The witness records $\dim_{\mathbb C}K_N=|J|d$ and $|J|\le2(N+2)d^2+1$.

#### Pedagogical prerequisites

Prerequisites are positive square roots, Cauchy evaluation, CFT-36-001, and the boundary embedding developed in CFT-34-001.

#### Lean correspondence

The chapter theorem `harp_finite_dilation_exists` is proved here only as `Nonempty` packaging of `Harp.finiteAtomicL2DilationWitness`. The mathematical construction belongs to that provider; the packaging is not an original route proof.

Sources: [[formalization/lean/CrouzeixConjecture/ParametricBoundary.lean|boundary hypotheses and normalization]], [geometry](../../../formalization/lean/CrouzeixConjecture/ParametricBoundary.lean#L25), [Cauchy hypothesis](../../../formalization/lean/CrouzeixConjecture/ParametricBoundary.lean#L239); [[formalization/lean/Crouzeix/Harp/FiniteAtomicL2Dilation.lean|finite atomic witness]], [compression](../../../formalization/lean/Crouzeix/Harp/FiniteAtomicL2Dilation.lean#L164), [witness fields](../../../formalization/lean/Crouzeix/Harp/FiniteAtomicL2Dilation.lean#L198), [construction](../../../formalization/lean/Crouzeix/Harp/FiniteAtomicL2Dilation.lean#L308).

#### Historical context

Historical context. Harp reuses the Lorist–Schwenninger boundary embedding and multiplier on a finite counting-measure space. The finite nodes come from Harp's cubature specialization.

#### ML analogy

Motivation. A larger latent space can represent selected moments of an original linear layer by compression. Exact transfer: the formulas above are finite-dimensional linear algebra. Nontransfer: the selected latent space is an existence witness, with no assertion that training finds it or that it is small in practice.

### CFT-36-003 — The finite recurrence keeps its endpoint {#cft-36-003}

#### Purpose

Extract a scalar inequality from one horizon witness.

#### Statement

The fixed core and a horizon-$N$ witness imply the weighted inequality (36.1) below, with its terminal moment retained.

#### Hypothesis ledger

Let $E$ be a complex Hilbert space, let the fixed core satisfy $\|E_k\|\le L$ and $E_kT=TE_k$, and assume an isometry $V:E\to K_N$, a contraction $Q$, and the preceding identities through $N+1$. Let $\kappa>1$ and suppose

$$
T^*Tx=\kappa^2x.
$$

For this recurrence alone, $x$ need not be unit and $\kappa$ need not be the operator norm. The terminal application will make those choices. Define

$$
m_k=\operatorname{Re}\langle x,E_kT^kx\rangle,
\quad d_N=Q^*VTx-\kappa Vx,\quad b_N=\|d_N\|^2.
$$

Then

$$
\kappa^{-N}m_{N+1}
+\left(\sum_{k=1}^N\kappa^{-k}\right)
\frac{-b_N}{\kappa^2-\kappa}\le m_1.
\tag{36.1}
$$

#### Proof roadmap

Factor the adjacent-power defect through the displacement, complete a square, and sum the resulting inequalities with geometric weights.

#### Proof

The commutation relation and the adjoint rule show
$m_k=\operatorname{Re}\langle E_kx,(T^*)^kx\rangle$. The exchange of inner-product slots changes a complex value to its conjugate, leaving its real part unchanged. Put

$$
S_k=2V^*(Q^*)^kV,\qquad u_k=(T^*)^kx,\qquad
y_k=(S_{k+1}T-\kappa S_k)x.
$$

Factor the defect:

$$
y_k=2V^*(Q^*)^k(Q^*VTx-\kappa Vx)=2V^*(Q^*)^kd_N,
\quad \|y_k\|\le2\sqrt{b_N}.
$$

For $1\le k\le N$, both $E_k=S_k-(T^*)^k$ and its successor identity are available. Commutation at $k+1$ gives
$m_{k+1}=\operatorname{Re}\langle E_{k+1}Tx,u_k\rangle$.
The singular-vector equation gives
$(T^*)^{k+1}Tx=\kappa^2u_k$. Consequently

$$
\begin{aligned}
\kappa m_k-m_{k+1}
&=\operatorname{Re}\langle\kappa E_kx-E_{k+1}Tx,u_k\rangle\\
&=(\kappa^2-\kappa)\|u_k\|^2-\operatorname{Re}\langle y_k,u_k\rangle.
\end{aligned}
$$

Write $a=\kappa^2-\kappa>0$. Completing the square, with real scalar $1/(2a)$, yields

$$
a\|u\|^2-\operatorname{Re}\langle y,u\rangle
=a\left\|u-\frac{y}{2a}\right\|^2-\frac{\|y\|^2}{4a}
\ge-\frac{b_N}{a}.
$$

Thus $c_N=-b_N/a\le\kappa m_k-m_{k+1}$. Multiply the $k$-th inequality by $\kappa^{-k}>0$ and sum. The right side telescopes:

$$
\sum_{k=1}^N(\kappa^{1-k}m_k-\kappa^{-k}m_{k+1})
=m_1-\kappa^{-N}m_{N+1}.
$$

Rearrangement gives (36.1). The last adjacent pair uses powers $N$ and $N+1$, explaining the extra moment in cubature. Discarding the terminal term without knowing its sign would be invalid.

#### Boundary case

For $N=0$, the sum is empty and (36.1) is $m_1\le m_1$. No recurrence step is used. Finite-dimensionality of $E$ is unnecessary for this identity; it will be needed to obtain a top singular vector in the next card.

#### Pedagogical prerequisites

Prerequisites are Cauchy–Schwarz, adjoints, finite telescoping, CFT-36-002, and the recurrence background in CFT-33-002.

#### Lean correspondence

`harp_finite_recurrence` reexports `equation_three_finite_lower_bound`. The provider proves the adjacent identity first, then its square bound and finite iteration.

Source: [[formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean|finite operator recurrence]], [defect](../../../formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean#L97), [identity](../../../formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean#L154), [finite inequality](../../../formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean#L224).

#### Historical context

Historical context. The Harp provider reconstructs the adjacent-power argument at a bounded horizon and uses lower-level Lorist–Schwenninger square and iteration support. Its teaching interface is a reexport.

#### ML analogy

Motivation. Truncated rollout calculations often retain an endpoint residual. Exact transfer: telescoping identities require their endpoint terms. Nontransfer: this particular $m_k$ uses companion operators constructed from boundary integration, not an arbitrary empirical loss sequence.

### CFT-36-004 — One common bound permits the scalar limit {#cft-36-004}

#### Purpose

Replace varying displacement errors by a common scalar bound so that the horizon can tend to infinity.

#### Statement

Suppose $E$ is nonzero and finite-dimensional, the core $T,E_k,L$ is fixed, and every $N$ has a dilation witness. Then $\|T\|\le2$.

#### Hypothesis ledger

The core has a nonnegative uniform companion bound and commutation, as in CFT-36-003. For every natural $N$, a witness supplies isometry, contractivity, and identities through $N+1$. The spaces $K_N$ may vary and need only be complete complex Hilbert spaces for this abstract theorem; their finite-dimensional construction is supplied separately.

#### Proof roadmap

Choose a unit top singular vector, bound every displacement by the same $C$, remove the bounded terminal term by geometric decay, and obtain a sign contradiction if the norm exceeds two.

#### Proof

If $\|T\|\le1$, the conclusion follows. Otherwise set $\kappa=\|T\|>1$ and choose a unit top singular vector $x$. Such a vector exists because the unit sphere is compact, or by diagonalizing $T^*T$. It satisfies $T^*Tx=\kappa^2x$.

For each horizon put $A_N=Q_N^*V_NTx$. Contractivity and isometry imply $\|A_N\|\le\kappa$. The first companion identity gives

$$
m_1=2\operatorname{Re}\langle A_N,V_Nx\rangle-\kappa^2.
$$

Expand the displacement square explicitly:

$$
\begin{aligned}
b_N&=\|A_N-\kappa V_Nx\|^2\\
&=\|A_N\|^2-2\kappa\operatorname{Re}\langle A_N,V_Nx\rangle+\kappa^2\\
&\le2\kappa^2-\kappa(m_1+\kappa^2)
=:C=2\kappa^2-\kappa m_1-\kappa^3.
\end{aligned}
$$

Unlike $b_N$, $C$ depends only on the fixed core and vector. The horizon-zero witness proves $0\le b_0\le C$, so $C\ge0$.

We must replace the varying error before taking a limit. Since the denominator and geometric weights are nonnegative,

$$
-\frac{C}{\kappa^2-\kappa}\le-\frac{b_N}{\kappa^2-\kappa}.
$$

Substituting the smaller expression into the left side of (36.1) preserves the inequality. This gives one scalar inequality for each $N$, now with a fixed coefficient.

To control the terminal term, fix any $k$ and choose a horizon at least $k$. The compression identity and $\|E_k\|\le L$ imply

$$
\|(T^*)^k\|\le2\|V_N^*\|\|(Q_N^*)^k\|\|V_N\|+\|E_k\|
\le2+L.
$$

Adjoints preserve the operator norm, so $\|T^k\|\le2+L$. Therefore
$|m_k|\le\|E_k\|\|T^k\|\le L(2+L)$. With $r=\kappa^{-1}\in(0,1)$,

$$
|r^Nm_{N+1}|\le r^NL(2+L)\longrightarrow0,
\qquad \sum_{k=1}^Nr^k\longrightarrow\frac{1}{\kappa-1}.
$$

The limit of the common scalar inequality is

$$
-\frac{C}{\kappa(\kappa-1)^2}\le m_1.
\tag{36.2}
$$

Combine this with the definition of $C$:

$$
C=2\kappa^2-\kappa m_1-\kappa^3
\le\kappa^2(2-\kappa)+\frac{C}{(\kappa-1)^2},
$$

and hence

$$
C\left[1-\frac1{(\kappa-1)^2}\right]\le\kappa^2(2-\kappa).
$$

If $\kappa>2$, the left side is nonnegative and the right side is strictly negative. This contradiction proves $\kappa\le2$. The argument took no limit of $b_N$, $V_N$, or $Q_N$.

#### Boundary case

At $\kappa=1$ the recurrence denominator vanishes, so use the initial easy case. At $\kappa=2$ the final sign calculation is compatible with equality. Positive finite dimension provides a unit vector; the zero space is excluded by the interface.

#### Pedagogical prerequisites

Prerequisites are norm attainment, real limits, CFT-36-003, and the scalar endpoint in CFT-33-005.

#### Lean correspondence

`harp_normalized_norm_two` reexports `Harp.norm_target_le_two_of_finiteHorizonDilationData`. Its dependent family `L : ℕ → Type*` permits a different space at each horizon.

Sources: [[formalization/lean/Crouzeix/Harp/FiniteHorizonPerturbation.lean|Harp terminal perturbation theorem]], [proof](../../../formalization/lean/Crouzeix/Harp/FiniteHorizonPerturbation.lean#L18); [[formalization/lean/Crouzeix/Harp/FiniteHorizonRecurrence.lean|scalar limit bridge]], [proof](../../../formalization/lean/Crouzeix/Harp/FiniteHorizonRecurrence.lean#L20); [[formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean|displacement upper bound]], [proof](../../../formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean#L277).

#### Historical context

Historical context. This is the Harp finite-horizon terminal provider, with Lorist–Schwenninger norm-attainment and scalar endpoint support. Sharing those lemmas is part of the dependency account.

#### ML analogy

Motivation. Bounds across different rollout lengths require constants that do not change with length. Exact transfer: a uniformly bounded sequence multiplied by a geometric decay factor tends to zero. Nontransfer: finitely many numerical horizon checks do not establish the universal hypothesis.

### CFT-36-005 — Normalize, then take the two outer limits {#cft-36-005}

#### Purpose

Remove the normalization and auxiliary simple-spectrum outer-domain setup.

#### Statement

For every $A\in M_d(\mathbb C)$, $d\ge1$, and every polynomial $p$, prove
$\|p(A)\|\le2\max_{W(A)}|p|$. No normality, diagonalizability, or simple-spectrum condition remains in the conclusion.

#### Hypothesis ledger

The matrix is complex of positive finite dimension, the norm is Euclidean operator norm, and $p$ is an arbitrary complex polynomial. Smooth boundaries and distinct eigenvalues are intermediate construction data, not hypotheses on $A$.

#### Proof roadmap

Fix an outer domain, prove the estimate for sufficiently close simple-spectrum matrices by a zero/positive maximum split, pass to the matrix limit, and then shrink the domain.

#### Proof

Let $K=W(A)$, a nonempty compact convex subset of $\mathbb C$. Fix one canonical outer domain $\Omega_r$ containing $K$, with compact closure and a suitable oriented radial boundary. The label $r$ denotes an outer-approximation index. Let $B_j\to A$ be the simple-spectrum approximations used by the provider. Numerical-range continuity and openness give
$W(B_j)\subseteq\Omega_r$ eventually in $j$.

Keep $r$ fixed and put $M_r=\max_{\overline\Omega_r}|p|$. Compactness and nonemptiness give a finite nonnegative maximum. For each sufficiently large $j$, consider two cases.

If $M_r=0$, every eigenvalue $\lambda_i$ of $B_j$ lies in $W(B_j)\subseteq\overline\Omega_r$, so $p(\lambda_i)=0$. Simple spectrum gives a diagonalization $B_j=S\operatorname{diag}(\lambda_i)S^{-1}$. Thus

$$
p(B_j)=S\operatorname{diag}(p(\lambda_i))S^{-1}=0.
$$

This step uses diagonalizability. Vanishing at eigenvalues alone does not imply $p(B)=0$ for a defective matrix.

If $M_r>0$, define $q=M_r^{-1}p$. Then $|q|\le1$ on the whole closure. The canonical boundary supplies the polynomial Cauchy formula for this simple-spectrum $B_j$. Cards 001–004, with $r,j$ fixed throughout the horizon limit, give $\|q(B_j)\|\le2$. Since polynomial evaluation respects scalar multiplication,

$$
\|q(B_j)\|=M_r^{-1}\|p(B_j)\|,
\qquad \|p(B_j)\|\le2M_r.
$$

In both cases the right side is independent of $j$. Matrix polynomial evaluation is continuous, so $p(B_j)\to p(A)$, and continuity of the norm yields
$\|p(A)\|\le2M_r$.

Now vary the domain index. The canonical closures lie in one fixed compact neighborhood of $K$, contain $K$, and approach it with distances tending uniformly to zero. Uniform continuity of $|p|$ on that neighborhood gives

$$
\max_K|p|\le M_r\le\max_K|p|+\varepsilon_r,
\qquad\varepsilon_r\to0.
$$

For the upper estimate, choose a maximizer on each compact closure and a nearby point of $K$; uniform continuity bounds the difference of their polynomial moduli. Hence $M_r\to\max_K|p|$, proving the desired result.

The order is $N\to\infty$ at fixed $r,j$, then $j\to\infty$ with the domain fixed, then the outer-domain limit. The companion bound $L$ may depend on $r,j$; it only needed to be independent of $N$.

#### Boundary case

The zero-maximum branch avoids division by zero. A singleton numerical range is allowed in the final theorem; the outer domains supply the geometry before the last limit. The exact Lean matrix index universe is `{n : Type}` with `Fintype n`, `DecidableEq n`, and `Nonempty n`.

#### Pedagogical prerequisites

Prerequisites are simple-spectrum density, polynomial continuity, compact outer approximation, CFT-36-004, and the target statement in CFT-29-003.

#### Lean correspondence

`harp_polynomial_constant_two` reexports `Harp.harpFiniteHorizonMainTheorem`.

Source: [[formalization/lean/Crouzeix/Harp/MainTheorem.lean|Harp matrix endpoint]], [complete provider proof](../../../formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25).

#### Historical context

Historical context. Harp supplies the finite-horizon normalized endpoint and uses the repository's canonical geometric and limiting infrastructure. The local theorem is the source of this card.

#### ML analogy

Motivation. Rescaling a polynomial filter separates its scalar scale from its operator amplification. Exact transfer: $p\mapsto p/M$ and norm continuity apply to a finite linear layer. Nontransfer: diagonalizability of intermediate approximations does not imply that a trained matrix is normal or well-conditioned.

### CFT-36-006 — Rational and Hilbert-space consequences {#cft-36-006}

#### Purpose

Transport the finite-matrix endpoint to rational functions and general Hilbert spaces.

#### Statement

For a bounded operator $A$ on a nonzero complex Hilbert space, let $K=\overline{W(A)}$. Then $K$ is compact, contains the spectrum of $A$, and for every rational function $r=a/b$ pole-free on $K$,

$$
\|r(A)\|=\|a(A)b(A)^{-1}\|\le2\max_{z\in K}|r(z)|.
$$

#### Hypothesis ledger

The space is a nonzero complete complex inner-product space, $A$ is bounded and complex linear, and $r$ is pole-free on the closed numerical range. The pole-free condition is the one for the rational function's denominator in the formal interface. Invertibility of the evaluated denominator is established by the spectral and approximation arguments.

#### Proof roadmap

Use finite Krylov compression for the polynomial bound, establish spectral containment in the closed numerical range, and pass from uniform polynomial approximation to the rational functional calculus.

#### Proof

First transport the polynomial theorem. For a unit vector $x$ and polynomial $p$ of degree at most $l$, form the finite Krylov space

$$
E_x=\operatorname{span}\{x,Ax,\ldots,A^lx\},
\qquad B=P_{E_x}A|_{E_x}.
$$

Induction gives $B^kx=A^kx$ for $k\le l$, because each vector on the right still lies in $E_x$. Thus $p(B)x=p(A)x$. Also $W(B)\subseteq W(A)$, since $\langle y,P_{E_x}Ay\rangle=\langle y,Ay\rangle$ for $y\in E_x$. Choose orthonormal coordinates and apply the finite-matrix theorem:

$$
\|p(A)x\|\le\|p(B)\|\le2\sup_{W(A)}|p|.
$$

Take the supremum over unit $x$. This proves the Hilbert-space polynomial bound without assuming that an infinite-dimensional operator attains its norm.

The closed numerical range is bounded and closed in $\mathbb C$, hence compact, and it is convex. To see spectral containment, take $\lambda\notin K$. The distance from $\lambda$ to $K$ gives a strictly positive lower bound for $\|(\lambda I-A)x\|$ on unit vectors, using the inner product and Cauchy–Schwarz. The analogous estimate for the adjoint gives a trivial orthogonal complement to the range. The range is closed by the first bound and dense by the adjoint bound, so $\lambda I-A$ is invertible.

Polynomial approximation on the compact convex $K$ now handles a pole-free rational function. There are polynomials $p_j\to r$ uniformly on $K$. The polynomial estimate gives
$\|p_j(A)-p_k(A)\|\le2\|p_j-p_k\|_{K}$, so $p_j(A)$ converges in operator norm. Moreover $bp_j\to a$ uniformly on $K$, and the same estimate implies
$b(A)p_j(A)\to a(A)$. Since $b(A)$ is invertible, the limit is $b(A)^{-1}a(A)=r(A)$. Passing to the norm bound proves the rational estimate. This is the scalar spectral-set conclusion.

#### Boundary case

Closure matters in Hilbert space because $W(A)$ need not be closed. A pole on $K$ is outside the theorem's hypotheses. The conclusion is for scalar polynomials and rational functions, with no complete-boundedness claim or algorithmic numerical guarantee.

#### Pedagogical prerequisites

Prerequisites are CFT-36-005, the spectral-set formulation in CFT-21-005, finite Krylov compression, spectral containment, and uniform rational approximation. The full neutral adapter arguments are developed in [[knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries#cft-35-003|Chapter 35, rational transport]], [[knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries#cft-35-004|Hilbert transport]], and [[knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries#cft-35-005|spectral-set packaging]].

#### Lean correspondence

`harp_two_spectral_set` reexports `harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet`.

Sources: [[formalization/lean/Crouzeix/Harp/Consequences.lean|Harp consequence providers]], [finite and rational providers](../../../formalization/lean/Crouzeix/Harp/Consequences.lean#L22), [closed spectral-set provider](../../../formalization/lean/Crouzeix/Harp/Consequences.lean#L57); [[formalization/lean/CrouzeixConjecture/HilbertSpectralSetCore.lean|neutral spectral-set contract]], [exact conjunction](../../../formalization/lean/CrouzeixConjecture/HilbertSpectralSetCore.lean#L484).

#### Historical context

Historical context. These are provider-neutral transport arguments instantiated with Harp's matrix theorem. Their reuse does not turn the Harp route into an independent proof of each adapter.

#### ML analogy

Motivation. Krylov spaces organize finite rollout information even when the state space is large or infinite. Exact transfer: compression preserves the powers of the chosen starting vector through the selected degree. Nontransfer: it does not preserve all trajectories, nonlinear layers, or matrix-valued function norms.

## Worked examples

### A two-node scalar dilation

Consider abstract positive atomic data $D_1=D_2=[1]$ in dimension one, with $h_1=1$, $h_2=-1$. This is a worked calculation, not a separate Lean exercise declaration or a claim that these nodes arise from a chosen contour. Its formal support is the general compression construction in CFT-36-002. Here

$$
Vx=\frac1{\sqrt2}\begin{bmatrix}x\\x\end{bmatrix},\qquad
Q=\begin{bmatrix}1&0\\0&-1\end{bmatrix},\qquad
V^*Q^kV=\begin{cases}1&k\text{ even},\\0&k\text{ odd}.\end{cases}
$$

The mass identity $[1]+[1]=[2]$ explains the factor $1/\sqrt2$. Omitting that factor gives $V^*V=2I$. Moving the already absorbed cubature weights into the counting measure as well would change the calculation.

### A nonnormal layer with sharp amplification

Let

$$
A=\begin{bmatrix}0&2\\0&0\end{bmatrix},\qquad p(z)=z.
$$

For $x=[x_1\ x_2]^\mathsf T$ of norm one,
$\langle x,Ax\rangle=2\bar x_1x_2$, whose modulus is at most one. Varying the magnitudes and relative phase fills the unit disk. Thus $\max_{W(A)}|p|=1$, whereas $\|p(A)\|=2$. The endpoint permits equality. Both eigenvalues are zero, yet $p(A)\ne0$, which also shows why the zero-maximum branch in CFT-36-005 explicitly uses diagonalizability.

This is the sharp example from [[knowledge/crouzeix_textbook/part_06_constant_two_routes/29_crouzeix_problem_and_sharpness|Chapter 29]]. Its exact formal support is [[formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean|the Chapter 29 sharpness provider]], [`jordan_two_attains_two`](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L197).

### A horizon-two calculation

With $c_N=-b_N/(\kappa^2-\kappa)$, two adjacent steps give
$\kappa m_1-m_2\ge c_N$ and $\kappa m_2-m_3\ge c_N$. Multiply by $\kappa^{-1}$ and $\kappa^{-2}$, then add:

$$
m_1\ge\kappa^{-2}m_3+(\kappa^{-1}+\kappa^{-2})c_N.
$$

This uses moments through three. There is no reason for $m_3$ to have a favorable sign. The uniform bound, rather than its sign, makes the eventual terminal term disappear.

## ML bridge

For a fixed linear layer $A$, a finite rollout filter has the form $p(A)x=\sum_{k=0}^lc_kA^kx$. The theorem compares its worst-case Euclidean amplification with the scalar polynomial modulus on the numerical range. The nilpotent example shows why eigenvalues alone can miss that amplification.

The proof's cubature moments are matrix-valued boundary quantities, not samples of a training dataset. Their positive weights and exact preservation are mathematical hypotheses and conclusions. A numerical cubature solver may only approximate them; an empirical residual would require a separate error analysis before it could certify an operator bound. The formal witness uses classical choice and provides no runtime or conditioning estimate.

Likewise, checking a finite list of rollout horizons does not establish the assertion that every horizon has a witness with the same core. In this proof that universal statement comes from a construction parameterized by $N$. The transferable lesson for a researcher is to identify which quantities remain fixed when the horizon varies and to retain terminal errors until a justified bound removes them. The analogy is diagnostic, not a proof of stability for nonlinear or changing layers.

## Lean translation

The file [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Chapter 36 teaching interfaces]] imports the Harp consequences provider. Its six correspondences distinguish theorem aliases from the existence wrapper:

| Card | Chapter declaration | Proof provenance |
| --- | --- | --- |
| CFT-36-001 | [`harp_positive_moment_cubature`](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L25) | Reexport of positive matrix-moment cubature |
| CFT-36-002 | [`harp_finite_dilation_exists`](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L29) | `Nonempty` packaging of a provider data witness |
| CFT-36-003 | [`harp_finite_recurrence`](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L47) | Reexport of the finite weighted inequality |
| CFT-36-004 | [`harp_normalized_norm_two`](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L58) | Reexport of the horizon-indexed endpoint |
| CFT-36-005 | [`harp_polynomial_constant_two`](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L63) | Reexport of the Harp matrix theorem |
| CFT-36-006 | [`harp_two_spectral_set`](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L68) | Reexport of the Harp Hilbert consequence |

`CommutingPerturbationData` stores `T`, the perturbations, their common bound, and commutation. `FiniteHorizonDilationData core K N` separately stores `V`, `Q`, isometry, contractivity, and identities only when `k ≤ N + 1`. `FiniteAtomicL2DilationWitness` hides the chosen node type and Hilbert-space instances while retaining the node count and dimension equations. These types prevent an accidental change of core between horizons.

Lean's `Fin (N + 2)` contains indices zero through $N+1$. Its real scalar weights act on complex matrix tuples by real scalar multiplication. Matrices and their Euclidean continuous-linear-map representations have equal operator norms. These translations explain apparent changes of notation between the cubature and recurrence providers.

The six exercises below prove smaller statements directly. Their existence does not mean each exercise reconstructs its parent provider. In particular, E02 takes an inner-product calculation as an explicit premise, and E06 invokes scalar limiting support rather than a terminal operator theorem. Compiler provenance belongs in the managed receipt and coverage records; no compiler hash is asserted in this prose.

## Exercises

### CFT-36-E01 — Project the preserved tuple {#exercise-cft-36-e01}

Statement. Let $J$ be a finite type, $N\in\mathbb N$, and let the matrix index type be finite with decidable equality. Given real weights $w_j$, tuples $M_j:\operatorname{Fin}(N+2)\to M_d(\mathbb C)$, and a target tuple $Z$, assume $\sum_jw_jM_j=Z$. For any index $k$, prove $\sum_jw_jM_j(k)=Z(k)$. No positivity or measure hypothesis is required.

Solution. Apply the function $f\mapsto f(k)$ to both sides. Evaluation commutes with finite sums and scalar multiplication, producing the claimed identity. Commuting an integral with evaluation is a separate fact needed by the cubature provider, not part of this exercise.

Exact correspondence: `Exercises.Chapter36.exercise_01_solution`, [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Lean exercise source]], [declaration](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L77).

### CFT-36-E02 — Use the normalized inner-product calculation {#exercise-cft-36-e02}

Statement. Let $D_j$ be a finite family of positive semidefinite complex matrices with $\sum_jD_j=2I$, and let $V:\mathbb C^d\to K$ be complex linear into a complex inner-product space. Assume explicitly, for every $x$,

$$
\langle Vx,Vx\rangle=\frac12\left\langle x,\left(\sum_jD_j\right)x\right\rangle.
$$

Prove $\|Vx\|=\|x\|$. The finite matrix index type has decidable equality; neither it nor $K$ is required to be nonzero, and $K$ need not be complete.

Solution. Substitute $2I$ into the displayed premise. Linearity in the second slot cancels the factor two, giving $\langle Vx,Vx\rangle=\langle x,x\rangle$. Taking real parts yields equality of squared norms. Both norms are nonnegative, so their squares determine them uniquely. The exercise does not construct square roots or derive its `hinner` premise; those calculations appear in CFT-36-002.

Exact correspondence: `Exercises.Chapter36.exercise_02_solution`, [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Lean exercise source]], [declaration](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L89).

### CFT-36-E03 — Prove finite weighted telescoping by induction {#exercise-cft-36-e03}

Statement. For real $r,c$, assume $r\ge0$. Let $m:\mathbb N\to\mathbb R$ and $N\in\mathbb N$. Suppose $rm_{j+2}+rc\le m_{j+1}$ for all $j<N$. Prove

$$
r^Nm_{N+1}+\left(\sum_{j=0}^{N-1}r^{j+1}\right)c\le m_1.
$$

Solution. At $N=0$, this is equality. For the successor case, the induction hypothesis bounds $r^Nm_{N+1}$ plus the first $N$ weights. Multiply the last step by $r^N\ge0$ to obtain
$r^{N+1}m_{N+2}+r^{N+1}c\le r^Nm_{N+1}$. Substitute this lower expression for the terminal term in the induction inequality. Adding the new weight gives the required sum through $j=N$. No assumption $r<1$ is needed for a finite induction.

Exact correspondence: `Exercises.Chapter36.exercise_03_solution`, [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Lean exercise source]], [declaration](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L105).

### CFT-36-E04 — Replace the displacement with the correct sign {#exercise-cft-36-e04}

Statement. For real $b,C,d,w,t,m$, assume $b\le C$, $d>0$, $w\ge0$, and $t+w(-b/d)\le m$. Prove $t+w(-C/d)\le m$. There is no hypothesis that $b$ or $C$ is nonnegative.

Solution. Negation reverses $b\le C$, giving $-C\le-b$. Division by positive $d$ preserves the resulting order, as does multiplication by $w\ge0$. Adding $t$ and using transitivity proves the claim. Taking $t$ to be the terminal term makes this exactly the common-error substitution used before the horizon limit.

Exact correspondence: `Exercises.Chapter36.exercise_04_solution`, [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Lean exercise source]], [declaration](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L118).

### CFT-36-E05 — Undo positive normalization {#exercise-cft-36-e05}

Statement. For real $a,s$, suppose $s>0$ and $a/s\le2$. Prove $a\le2s$.

Solution. Multiply the inequality by positive $s$. The order is preserved and $(a/s)s=a$. The premise does not require $a\ge0$. The matrix application sets $a=\|p(B)\|$, but the exercise is the more general real inequality. At $s=0$ this derivation is unavailable; the polynomial theorem handles that case separately.

Exact correspondence: `Exercises.Chapter36.exercise_05_solution`, [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Lean exercise source]], [declaration](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L126).

### CFT-36-E06 — Complete the scalar limit {#exercise-cft-36-e06}

Statement. Let $\kappa,C,M$ be real and $m:\mathbb N\to\mathbb R$. Assume $\kappa>1$, $C\ge0$, $|m_j|\le M$ for every $j$, and for every $N$,

$$
\kappa^{-N}m_{N+1}
+\left(\sum_{j=0}^{N-1}\kappa^{-(j+1)}\right)
\frac{-C}{\kappa^2-\kappa}\le m_1.
$$

Assume also $C\le2\kappa^2-\kappa m_1-\kappa^3$. Prove $\kappa\le2$. The final premise is an inequality, not necessarily the equality used to define $C$ in the operator application.

Solution. The uniform bound gives $\kappa^{-N}m_{N+1}\to0$, and the weight sum tends to $1/(\kappa-1)$. Addition and multiplication by the fixed coefficient preserve convergence. Closedness of the real order therefore gives

$$
\frac{-C/(\kappa^2-\kappa)}{\kappa-1}\le m_1.
$$

Since $\kappa\ne0$, $\kappa-1\ne0$, and $\kappa^2-\kappa=\kappa(\kappa-1)$, this is (36.2). Combining it with the assumed upper bound gives the same final sign contradiction as CFT-36-004. In Lean, the proof combines `bounded_recurrence_terminal_tendsto_zero` and `inverse_power_weight_sum_tendsto` with `Tendsto.add`, `mul_const`, and `le_of_tendsto'`, then invokes `scalar_endpoint_le_two`. It never invokes the operator endpoint to prove this scalar exercise.

Exact correspondence: `Exercises.Chapter36.exercise_06_solution`, [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Lean exercise source]], [declaration](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L132).

## Synthesis and forward dependencies

The finite proof depends on two exact identities and one uniform replacement. Positive cubature preserves a common tuple through $N+1$; the atomic square-root embedding turns those moments into compression identities; and the bound $b_N\le C$ replaces varying geometric data by a fixed scalar error. Only then does the geometric-series limit apply.

The endpoint removes normalization, simple spectrum, and the outer domain in that order. Neutral adapters then extend it to pole-free rational functions and bounded Hilbert-space operators. Retain their hypotheses, especially the common core and the pole-free condition on the closed numerical range. For route comparisons and stronger-claim boundaries, see [[knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries|Chapter 35]]; the [[knowledge/crouzeix_textbook/crouzeix_textbook_index|book index]] locates the prerequisites.

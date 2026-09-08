---
id: cft-harp-finite-horizon-remainder
title: The finite-horizon remainder
type: reference
status: active
created: 2026-09-07
updated: 2026-09-07
tags: [crouzeix-textbook, mathematics, harp, lean, recurrence]
confidence: medium
canonical: harp_finite_horizon_remainder.md
---

# The finite-horizon remainder

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]

The horizon limit in Chapter 36 removes a bounded terminal moment. What
exactly remains before taking that limit? This supplement keeps the error
visible and derives a bound from one finite witness.

**Formal status.** The scalar estimates and one-witness application compile
locally in Lean. Their exact hypotheses remain part of every claim below;
compilation is not human peer review. Repository acceptance is recorded
separately in the workstream review. This is an unindexed supplement: its
helpers and examples do not increase the 216 registered theorem or exercise counts.

## The finite data

Fix $\kappa>1$, real numbers $c,M$ with $M\ge0$, a real sequence
$(m_k)_{k\ge0}$, and a nonnegative integer $N$. Put $r=\kappa^{-1}$.
The only assumptions about the sequence needed at this horizon are

$$
|m_{N+1}|\le M,\qquad
r^Nm_{N+1}+c\sum_{i=0}^{N-1}r^{i+1}\le m_1.
\tag{R.1}
$$

There is no assumption about another horizon, and no sign assumption on $c$.
The sequence begins at index zero for Lean bookkeeping; the recurrence
estimate starts at $m_1$.

Let $S_N=\sum_{i=0}^{N-1}r^{i+1}$, with $S_0=0$. Subtracting the
shifted sum gives
$(1-r)S_N=r-r^{N+1}=r(1-r^N)$.
Since $1-r=(\kappa-1)/\kappa>0$, division gives the exact identity

$$
S_N=\frac{1-r^N}{\kappa-1}.
\tag{R.2}
$$

The formula is valid at $N=0$ too. No convergence theorem is involved.

Lean correspondence: inverse_power_weight_sum_exact in
[FiniteHorizonRemainder.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonRemainder.lean).
All scalar declarations in this supplement are in the
CrouzeixConjecture.Harp namespace. Their scope is real scalar inequalities;
they do not assume a terminal Crouzeix bound.

## Two remainder bounds

Since $r^N\ge0$, the terminal assumption implies

$$
|r^Nm_{N+1}|=r^N|m_{N+1}|\le Mr^N,\qquad
-Mr^N\le r^Nm_{N+1}.
$$

Substitute the lower bound and the exact geometric sum into R.1:

$$
\begin{aligned}
m_1
&\ge -Mr^N+\frac{c(1-r^N)}{\kappa-1}\\
&=\frac{c}{\kappa-1}
-r^N\left(M+\frac{c}{\kappa-1}\right).
\end{aligned}
\tag{R.3}
$$

This is the sign-sensitive estimate. The parenthesized coefficient can be
negative; the proof did not assume otherwise. It is therefore premature
to call that coefficient a nonnegative error budget.

To obtain a convenient nonnegative budget, define

$$
D=M+\frac{|c|}{\kappa-1}\ge0.
$$

Because $c\le|c|$ and $\kappa-1>0$, we have
$M+c/(\kappa-1)\le D$. Multiplication by $r^N\ge0$ preserves that
inequality, and subtraction reverses it. Thus R.3 implies

$$
\boxed{\frac{c}{\kappa-1}-r^ND\le m_1.}
\tag{R.4}
$$

R.4 can be weaker than R.3. Taking an absolute value buys a nonnegative
budget, not a sharper bound.

Lean correspondence: finite_terminal_abs_le proves the terminal estimate;
finite_lower_bound_with_remainder proves R.3;
finite_lower_bound_with_abs_remainder proves R.4. Their exact statements
and proofs are in
[FiniteHorizonRemainder.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonRemainder.lean).
The terminal bound and weighted inequality are explicit hypotheses.

## A tolerance and an eventual horizon

If $\varepsilon\ge0$ and $r^ND\le\varepsilon$, R.4 immediately gives

$$
\frac{c}{\kappa-1}-\varepsilon\le m_1.
\tag{R.5}
$$

The power inequality is an explicit sufficient condition on a proposed
horizon. It is not itself a procedure for constructing the finite witness.

For every $\varepsilon>0$, there exists $N_0$ such that
$r^ND\le\varepsilon$ for every $N\ge N_0$. If $D=0$, choose $N_0=0$.
Otherwise, $0<r<1$ gives $r^N\to0$, hence $r^ND\to0$. By the definition
of convergence, eventually $|r^ND|<\varepsilon$; nonnegativity removes
the absolute value. The natural-number eventuality supplies the claimed
threshold.

Lean correspondence: finite_lower_bound_of_error_budget and
exists_uniform_remainder_horizon in the same scalar module.
The latter is an existence theorem, not an executable horizon-selection
algorithm. The Lean proof treats $D=0$ first and uses $\varepsilon/D$
only in the branch $D>0$. No logarithm or rounding convention is needed.

If R.1 holds for every $N$ with the same $\kappa,c,M,m$, the left side of
R.4 tends to $c/(\kappa-1)$. Closedness of the order relation gives
$c/(\kappa-1)\le m_1$. When the older interface only states
$|m_k|\le M$ for all $k$, recover $M\ge0$ from
$0\le|m_0|\le M$. There is no need to strengthen that interface.
The added comparison corollary
finite_weighted_inequalities_to_limit_lower_bound_via_remainder leaves the existing provider
finite_weighted_inequalities_to_limit_lower_bound unchanged.

## Instantiating one Harp witness

Fix the common data $T,(E_k),L$ described in the
[[knowledge/crouzeix_textbook/harp_mathematical_audit|mathematical audit]].
For one horizon $N$, assume an isometry $V_N$ and contraction $Q_N$
realize the compression identities through $N+1$. Assume a vector $x$
and a real number $\kappa$ satisfy

$$
\|x\|=1,\qquad \kappa=\|T\|>1,\qquad
T^\dagger Tx=\kappa^2x.
$$

These are supplied hypotheses of the application. An arbitrary vector is
not automatically a top singular vector. The Lean interface uses complete
complex inner-product spaces $E$ and $K$, with $E$ nontrivial. It does not
require finite dimensionality when the unit singular vector is supplied.
There is one witness at this $N$, not a family quantified over all horizons.
Define

$$
\begin{aligned}
m_k&=\operatorname{Re}\langle x,E_kT^kx\rangle,\\
M&=L(2+L),\\
C&=2\kappa^2-\kappa m_1-\kappa^3,\qquad
c=-\frac{C}{\kappa^2-\kappa}.
\end{aligned}
$$

We have $M\ge0$ from $L\ge0$. The bound recurrenceScalar_abs_le gives
$|m_{N+1}|\le M$ using this same witness. Its squared displacement
$b_N=\|Q_N^\dagger V_NTx-\kappa V_Nx\|^2$ satisfies $0\le b_N\le C$.
Thus $C\ge0$ and the denominator $\kappa^2-\kappa$ is positive.

The witness supplies R.1 with coefficient $-b_N/(\kappa^2-\kappa)$.
Since $b_N\le C$, we have
$c\le-b_N/(\kappa^2-\kappa)$. The weights are nonnegative, so replacing
that coefficient by $c$ preserves R.1. Apply R.4:

$$
\frac{c}{\kappa-1}
-\kappa^{-N}\left(L(2+L)+\frac{|c|}{\kappa-1}\right)\le m_1.
\tag{R.6}
$$

Lean correspondence: the application
finite_horizon_scalar_lower_bound_with_remainder is in
[FiniteHorizonRemainderApplication.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonRemainderApplication.lean).
Its providers equation_three_finite_lower_bound and displacementSq_le are
in [FiniteHorizonOperatorRecurrence.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean).
The terminal bound is in
[FiniteHorizonDilation.lean](../../formalization/lean/Crouzeix/Harp/FiniteHorizonDilation.lean).
No terminal norm theorem is needed for this application.
The companion helper finite_horizon_displacement_envelope_nonneg proves
$C\ge0$ from this witness. The two exact interface clients are in
[HarpRemainderApplicationTests.lean](../../formalization/lean/CrouzeixTextbook/HarpRemainderApplicationTests.lean).

The budget depends on $\kappa$, and $C$ itself contains $m_1$.
This is an implicit scalar recurrence estimate, not an independently
computable error certificate. In particular R.6 is not advertised as an
effective $2+\varepsilon$ operator theorem.

## Worked checks

At $N=0$, $S_N=0$ and the weighted premise is $m_1\le m_1$.
R.3 reduces to $-M\le m_1$, exactly the terminal bound's lower half.
R.4 may be weaker when $c<0$.
At $N=1$, $S_N=r$; at $N=2$, $S_N=r+r^2$.
Substituting these expressions in R.2 checks both the indexing and the
placement of the first weight.

For $\kappa=2,M=3,c=-1,N=2$, $r^N=1/4$ and $D=4$.
The nonnegative remainder is exactly $1$. The two lower bounds are

$$
-1-\tfrac14(3-1)=-\tfrac32,\qquad
-1-\tfrac14(3+1)=-2.
$$

Taking $m_3=-3$ makes the finite left side
$(-3)/4-(1/2+1/4)=-3/2$. Choosing $m_1=-3/2$ attains the tighter
bound. Values at other indices are irrelevant to this one-horizon example.

For a genuinely negative signed coefficient, take $\kappa=2,M=0,c=-5,N=1$
and $m_2=0$. The finite premise is $-5/2\le m_1$.
R.3 gives $-5-\tfrac12(-5)=-5/2$, while R.4 gives
$-5-\tfrac12(5)=-15/2$. A nonnegative-coefficient premise would wrongly
exclude this valid use of the stronger estimate.

For positive $c$, R.3 and R.4 agree because $|c|=c$.
For $c=0$, both say $-Mr^N\le m_1$.
If $M=0$, the terminal value must be zero; the proof remains valid.
If also $c=0$, then $D=0$, and both bounds give $0\le m_1$ directly
from the finite premise. No logarithm of zero is taken.
For this zero budget, R.5 also permits $\varepsilon=0$ and proves
$0\le m_1$ exactly. This uses the finite error-budget theorem, whose
tolerance is nonnegative; the separate eventual-horizon theorem requires a
strictly positive tolerance.

Why retain the finite premise? Set $\kappa=2,c=0,M=0,N=1$,
$m_2=0$ and $m_1=-1$. The terminal bound is true, but the purported
unconditional conclusion $0\le-1$ is false. Exactly the missing weighted
premise would have excluded this sequence.

The proof-only checks, including rejected applications missing essential
premises, are in
[HarpRemainderTests.lean](../../formalization/lean/CrouzeixTextbook/HarpRemainderTests.lean).
These examples are unindexed and do not count as new solved textbook exercises.

The following names, in namespace CrouzeixTextbook.HarpRemainderTests, locate
their exact formal statements in that linked module. The concrete inequality
fixtures invoke the general result with the displayed sequence and parameters;
they are not extra general theorems inferred from a numerical experiment.

| Written check | Lean declaration |
| --- | --- |
| Empty, one-term and two-term sums | weight_empty, weight_one, weight_two |
| Horizon-zero lower bound | horizon_zero |
| Positive and negative coefficients at one step | positive_c_one, negative_coefficient_one |
| Zero coefficient and terminal bound | zero_c_zero_M |
| Zero tolerance under the finite premises | zero_error_budget |
| Zero and positive eventual budgets | zero_budget_horizon, positive_budget_horizon |
| The $c=-1,N=2$ sequence satisfies both premises | negative_example_terminal, negative_example_finite |
| Budget one, tight and relaxed numerical values | negative_example_values |
| Tight, relaxed and tolerance conclusions | negative_example_tight, negative_example_relaxed, negative_example_error_budget |
| Valid application and rejected missing premises | exact_contract_positive_control, rejects_missing_kappa, rejects_missing_terminal |
| Counterexample without the finite premise | missing_finite_counterexample |
| Old limit interface recovered | comparison_old_interface |

## Motivation and limits of the result

**Motivation.** Keeping the terminal term tells us how the existing limit
argument approaches its scalar conclusion. It also separates a valid
finite inequality from a statement that requires arbitrarily large horizons.

**Historical context.** This is a quantitative refinement of the repository's
existing finite-horizon argument, not a claim of a new terminal proof or of
priority for geometric-series estimates. The
[[knowledge/crouzeix_textbook/source_registry|source registry]] records the
underlying literature and proof-route provenance.

**ML analogy.** The power $r^N$ resembles a contraction-based truncation
estimate, but the analogy has a boundary. The Harp construction supplies exact
moments and exact positive weights. Approximate cubature would introduce
moment residuals, positivity and square-root errors, and possible failures of
isometry or contraction. None is bounded here. A small numerical residual
does not replace those missing mathematical estimates.

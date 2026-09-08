# Harp finite-horizon remainder specification

Status: approved for implementation. Mathematics below was checked algebraically
when specified; compilation is a separate implementation acceptance condition. Parent:
[mathematics program](2026-09-07-harp-mathematics-program-design.md).

## Goal and boundary

Expose a finite-N error term underlying the existing asymptotic scalar
argument. Prove it with only one horizon's weighted inequality and terminal
bound, then instantiate it for the exact Harp data. Do not assume witnesses
for every N merely to prove a conclusion about one N.

This is a quantitative refinement of the existing argument, not a claim of
mathematical novelty or an independent proof of the terminal theorem. It
does not certify approximate cubature or compute an operator norm.

## Proposed files and dependencies

- New `formalization/lean/Crouzeix/Harp/FiniteHorizonRemainder.lean`: scalar
  geometric identities, terminal bound, finite lower bound, tolerance result.
- New `formalization/lean/Crouzeix/Harp/FiniteHorizonRemainderApplication.lean`:
  instantiate scalar results using exact finite-horizon operator data.
- Update `formalization/lean/CrouzeixHarp.lean` to import the application in
  addition to its existing consequence import. Do not replace the terminal
  provider or alter any of its hypotheses.
- New canonical supplement
  `knowledge/crouzeix_textbook/harp_finite_horizon_remainder.md`, linked from
  Chapter 36 and the book index without assigning a Chapter 37 identity.
- A proof-only test module with positive and rejected-premise fixtures under
  `formalization/lean/CrouzeixTextbook/`, imported by its root target.

The scalar module may reuse `FiniteHorizonRecurrence.lean` but must not depend
on `Harp.MainTheorem`, `Harp.Consequences`, or any route's terminal result.
The application may import `FiniteHorizonOperatorRecurrence`; it must not call
`norm_target_le_two_of_finiteHorizonDilationData` to obtain the remainder.

Implementation adjustment (2026-09-07): retain the sealed CrouzeixHarp aggregate
unchanged. The route validator rejected the proposed extra import as an unmapped
module in its historical closure. Export through the named application module
and CrouzeixTextbook instead; its root compilation includes all new helpers and
clients. This changes the integration surface, not the mathematical contract.

Use existing helper-declaration inspection for code links and proof-only
evidence. These new named theorems are initially an explicitly unindexed
supplement. Do not change the 216-item coverage roster or disguise supporting
declarations as extra registered CFT rows. Their separate status must be clear
in the supplement and final report.

## Mathematical contract

Fix kappa > 1, real c and M, M >= 0, a real sequence m, and N in N. Set
r = 1/kappa. Assume only

$$
|m_{N+1}|\le M,\qquad
r^N m_{N+1}+c\sum_{i=0}^{N-1}r^{i+1}\le m_1.
$$

Prove the following named results in namespace `CrouzeixConjecture.Harp`:

1. `inverse_power_weight_sum_exact`:
   sum_{i=0}^{N-1} r^(i+1) = (1-r^N)/(kappa-1).
2. `finite_terminal_abs_le`:
   |r^N m_(N+1)| <= M r^N.
3. `finite_lower_bound_with_remainder`, the sign-sensitive form:

$$
\frac{c}{\kappa-1}
-\kappa^{-N}\left(M+\frac{c}{\kappa-1}\right)\le m_1.
$$

4. `finite_lower_bound_with_abs_remainder`, the convenient nonnegative budget:

$$
\frac{c}{\kappa-1}
-\kappa^{-N}\left(M+\frac{|c|}{\kappa-1}\right)\le m_1.
$$

The third result does not require its parenthesized coefficient to be positive.
The fourth is deliberately weaker and has a nonnegative error budget. Preserve
this distinction rather than inserting an unjustified sign assumption on c.

Proposed Lean interface for the fourth result:

```lean
theorem finite_lower_bound_with_abs_remainder
    {kappa c M : Real} {m : Nat -> Real} {N : Nat}
    (hk : 1 < kappa) (hM : 0 <= M)
    (hterminal : |m (N + 1)| <= M)
    (hfinite :
      (kappa⁻¹) ^ N * m (N + 1) +
        (Finset.sum (Finset.range N) fun i => (kappa⁻¹) ^ (i + 1)) * c
          <= m 1) :
    c / (kappa - 1) -
      (kappa⁻¹) ^ N * (M + |c| / (kappa - 1)) <= m 1
```

This is a proposed signature, not a claimed existing declaration. Freeze its
elaborated statement and universe information during implementation.

### Proof roadmap

From the terminal bound and r^N >= 0, derive
-M r^N <= r^N m_(N+1). Substitute into the finite weighted inequality and use
the exact finite geometric sum. Rearrange to the third result. Since
c <= |c| and kappa-1 > 0, enlarging the subtracted remainder gives the fourth.
No limiting theorem or terminal operator bound is needed for this derivation.

## Tolerance and limiting statements

Write D = M + |c|/(kappa-1). Prove
`finite_lower_bound_of_error_budget`: if epsilon >= 0 and
kappa^(-N) D <= epsilon, then c/(kappa-1)-epsilon <= m_1.

Prove `exists_uniform_remainder_horizon`: if epsilon > 0, there exists N0 such
that every N >= N0 satisfies kappa^(-N) D <= epsilon. This concerns the scalar
error only; applying it to m requires the appropriate finite inequalities.
Handle D=0 explicitly. This existence lemma is not an executable horizon
selection algorithm. The finite power inequality is the explicit sufficient
condition; a logarithmic formula or certified rational implementation can be
specified separately after the base result is accepted.

When the finite premises hold for every N with the same kappa,c,M,m, recover
the existing limit lower bound as a corollary. If the existing interface omits
M>=0, derive it from any absolute-value bound instead of strengthening that
interface. Do not replace the existing
provider until equivalence and downstream checks have been reviewed; the
default implementation adds a comparison corollary and leaves it intact.

## Harp instantiation

For a fixed `CommutingPerturbationData` core, a unit norm-attaining singular
vector x, kappa = norm(T) > 1, and one horizon-N witness, use the existing
operator recurrence and terminal bound. Set

$$
m_k=\operatorname{Re}\langle x,E_kT^kx\rangle,\quad
M=L(2+L),\quad C=2\kappa^2-\kappa m_1-\kappa^3,\quad
c=-\frac{C}{\kappa^2-\kappa}.
$$

Use the existing displacement bound to justify replacement by C with the
correct negative sign. Prove the resulting finite scalar lower bound in
`finite_horizon_scalar_lower_bound_with_remainder`. Its public assumptions
must list the exact unit-vector, singular-vector, norm-identification, and
witness hypotheses actually used. Do not infer C nonnegative or division
legality without proof.

The budget contains kappa and C, and C itself contains m_1. State this implicit
dependence; it is not yet an independently computable error certificate.
The conclusion is a scalar recurrence estimate. Do not advertise an effective
2+epsilon operator theorem until its additional algebra and dependencies have
their own proof and reviewed statement.

## Tests and examples

- N=0: empty sum equals zero; denominators remain guarded by kappa>1.
- N=1 and N=2: expand the sums by hand and in Lean.
- c positive, zero, and negative: both remainder forms retain valid signs.
- M=0 and D=0: avoid invalid logarithms or division by the error budget.
- kappa=2, M=3, c=-1, N=2: the absolute remainder budget is exactly 1;
  the sign-sensitive lower bound is -3/2 and the relaxed bound is -2.
- Reject fixture applications missing kappa>1 or the terminal bound.
- With the finite premise removed, a chosen negative m_1 must not satisfy
  a purported unconditional conclusion. Use a concrete counterexample.
- Check the new modules' transitive dependencies for terminal shortcuts and
  unapproved axioms. Compile against the pinned warm cache.

Provide full written solutions for all these checks and source links to their
formal declarations. The supplement's examples are not counted as additional
indexed textbook exercises.

## Acceptance and follow-up research

All six scalar results, the comparison corollary, and the Harp application
compile; prose matches their exact types; tests include every boundary above;
the full gate and publication checks pass after the candidate is frozen.
Rebuild and inspect the book PDF because the index and Chapter 36 links change.

Deferred research is explicitly separate: approximate moment identities,
approximate isometry or contraction, conditioning of matrix square roots,
certified positivity, and floating-point error propagation. A numerical
residual alone proves none of those missing implications. No solver or
stability guarantee is included in acceptance for this workstream.

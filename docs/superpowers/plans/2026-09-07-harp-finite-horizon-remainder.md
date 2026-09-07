# Finite-horizon remainder implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development. Review scalar mathematics before operator application.

**Goal:** Expose and prove the finite-horizon remainder without invoking a terminal Crouzeix bound.

**Architecture:** Separate scalar estimates, exact operator application, and proof-only fixtures. Register a canonical unindexed supplement through existing owners.

**Tech stack:** Lean 4.32.1, Mathlib, Markdown, Rust publication, Atlas/PDF.

Specification: ../specs/2026-09-07-harp-finite-horizon-remainder-design.md.
Prerequisite: accepted fixed-core, weighted-recurrence and sign checks from theorem assurance.

## Task 1: Scalar estimates

Files: formalization/lean/Crouzeix/Harp/FiniteHorizonRemainder.lean;
formalization/lean/CrouzeixTextbook/HarpRemainderTests.lean.

- [x] Write clients for all six proposed declarations and observe missing-declaration failures.
- [x] Prove inverse_power_weight_sum_exact and finite_terminal_abs_le by geometric sum and nonnegative inverse powers.
- [x] Prove finite_lower_bound_with_remainder from terminal lower bound and exact sum, without a sign premise on c.
- [x] Prove finite_lower_bound_with_abs_remainder by c ≤ |c|, then finite_lower_bound_of_error_budget.
- [x] Prove exists_uniform_remainder_horizon using convergence of inverse powers, explicitly covering zero budget.
- [x] Recover the existing limiting statement as a comparison corollary without changing its provider or strengthening its hypotheses.
- [x] Compile tests for N=0,1,2; all signs of c; zero M/budget; and kappa=2,M=3,c=-1,N=2 giving tight -3/2, loose -2 and budget 1.
- [x] Reject missing kappa/terminal premises and prove a concrete counterexample without the finite premise.
- [x] Check axioms and absence of terminal imports. Run lake build Crouzeix.Harp.FiniteHorizonRemainder and direct test-module compile, then both reviews.

Public finite estimate:

```lean
theorem finite_lower_bound_with_abs_remainder
    {kappa c M : Real} {m : Nat → Real} {N : Nat}
    (hk : 1 < kappa) (hM : 0 ≤ M)
    (hterminal : |m (N + 1)| ≤ M)
    (hfinite : (kappa⁻¹)^N * m (N+1) +
      (Finset.sum (Finset.range N) fun i => (kappa⁻¹)^(i+1)) * c ≤ m 1) :
    c / (kappa-1) - (kappa⁻¹)^N * (M + |c|/(kappa-1)) ≤ m 1
```

## Task 2: Harp application

Files: formalization/lean/Crouzeix/Harp/FiniteHorizonRemainderApplication.lean.

- [x] Add a failing client for finite_horizon_scalar_lower_bound_with_remainder with one horizon witness.
- [x] Instantiate m_k=Re(inner x (E_k*T^k)x), M=L(2+L), C=2kappa²-kappa*m_1-kappa³, c=-C/(kappa²-kappa).
- [x] Derive M nonnegative, denominator positive, displacement replacement sign, and terminal bound from the actual data.
- [x] Keep unit, singular-vector and norm-identification assumptions explicit. Do not call norm_target_le_two_of_finiteHorizonDilationData.
- [x] Run lake build Crouzeix.Harp.FiniteHorizonRemainderApplication; inspect provider closure; obtain both reviews.

## Task 3: Supplement and integration

Files: knowledge/crouzeix_textbook/harp_finite_horizon_remainder.md;
formalization/lean/CrouzeixTextbook.lean, index, Chapter 36;
managed registry and tests, derived corpus/export. Parent owns shared files.

- [x] Write full finite-sum algebra and examples with exact compiled declaration links.
- [x] State that the budget is implicit in kappa and m_1, that eventual-horizon existence is not an algorithm, and that approximate cubature and effective operator guarantees are deferred.
- [x] Add a failing supplement-resolution test, register a nonchapter document, and link both entry points.
- [x] Import the application and proof-only fixtures. Keep 216 indexed rows unchanged.
- [x] Produce actual compiler helper evidence, perform both content reviews and join the unblocked slices' acceptance.
- [x] Run mise run verify; rebuild/inspect PDFs and reader links. Commit explicit paths only when green; no push.

Integration adjustment: export the application through its named module and
the textbook root, not CrouzeixHarp. The latter is a sealed terminal-route
aggregate: adding unused supplement imports makes its existing route manifest
fail with an unmapped active-closure module. Keeping that terminal aggregate
unchanged preserves the exact historical route claim; the textbook gate still
compiles every new helper and client. The selected local evidence is refreshed
because its broader source inventory includes the new Harp files.

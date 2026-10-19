import Crouzeix.LoristSchwenninger.ConcreteDilation
import CrouzeixConjecture.CanonicalParallelRadialGeometry
import CrouzeixConjecture.Limiting
import CrouzeixConjecture.OuterApproximationLimit
import CrouzeixConjecture.PerturbationInsideDomain

/-!
The source-clean Lorist--Schwenninger route to the finite-matrix polynomial
Crouzeix bound.
-/

noncomputable section

open Filter
open scoped Matrix Matrix.Norms.L2Operator Topology

namespace CrouzeixConjecture

variable {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]

/-- The Lorist--Schwenninger dilation, followed by the fixed-domain
simple-spectrum limit and the canonical outer-domain limit, proves the
finite-matrix polynomial Crouzeix bound with constant two. -/
theorem loristSchwenningerMainTheorem : MainTheoremStatement (n := n) := by
  intro A p
  let K := numericalRange A
  let C := Metric.cthickening 1 K
  have hKcompact : IsCompact K := isCompact_numericalRange A
  have hKne : K.Nonempty := numericalRange_nonempty A
  have hCcompact : IsCompact C := fixedOuterNeighborhood_isCompact hKcompact
  have hbound (k : ℕ) :
      ‖polynomialEval p A‖ ≤
        2 * maxPolynomialModulusOnSet
          (closure (parallelOuterDomain K k)) p := by
    have hOuter := parallelOuterDomain_data
      hKcompact (numericalRange_convex A) hKne k
    have hInside : ∀ᶠ j in atTop,
        numericalRange (simpleSpectrumApproximation A j) ⊆
          parallelOuterDomain K k := by
      simpa only [K] using
        eventually_simpleSpectrumApproximation_numericalRange_subset_open
          A hOuter.domain_isOpen hOuter.contains
    obtain ⟨R, c, ⟨G⟩⟩ :=
      canonicalParallelOrientedRadialBoundaryStatement (n := n) A k
    have hSimpleBound : ∀ᶠ j in atTop,
        ‖polynomialEval p (simpleSpectrumApproximation A j)‖ ≤
          2 * maxPolynomialModulusOnSet
            (closure (parallelOuterDomain K k)) p := by
      filter_upwards [hInside] with j hWB
      let B := simpleSpectrumApproximation A j
      let M := maxPolynomialModulusOnSet
        (closure (parallelOuterDomain K k)) p
      let hDiag := simpleDiagonalization_of_hasDistinctEigenvalues B
        (simpleSpectrumApproximation_hasDistinctEigenvalues A j)
      have hM : 0 ≤ M := by
        simpa only [M] using maxPolynomialModulusOnSet_nonneg
          hOuter.closure_isCompact hOuter.closure_nonempty p
      rcases hM.eq_or_lt with hMzero | hMpos
      · have heigenvalue (i : n) :
            hDiag.eigenvalues i ∈ matrixSpectrum B :=
          hDiag.eigenvalue_mem_matrixSpectrum i
        have heigenvalueClosure (i : n) :
            hDiag.eigenvalues i ∈
              closure (parallelOuterDomain K k) :=
          subset_closure (hWB
            (matrixSpectrum_subset_numericalRange B (heigenvalue i)))
        have hzero (i : n) :
            Polynomial.eval (hDiag.eigenvalues i) p = 0 := by
          apply norm_eq_zero.mp
          have hnorm :
              ‖Polynomial.eval (hDiag.eigenvalues i) p‖ ≤ M := by
            simpa only [M] using norm_polynomial_eval_le_maxOnSet
              hOuter.closure_isCompact hOuter.closure_nonempty p
              (heigenvalueClosure i)
          rw [← hMzero] at hnorm
          exact le_antisymm hnorm (norm_nonneg _)
        have hpBzero : polynomialEval p B = 0 := by
          rw [hDiag.polynomialEval_eq_innerConjugation_diagonal]
          simp [hzero]
        change ‖polynomialEval p B‖ ≤ 2 * M
        simp only [hpBzero, ← hMzero, norm_zero, mul_zero, le_refl]
      · let cM : ℂ := ((M : ℂ))⁻¹
        let q : Polynomial ℂ := Polynomial.C cM * p
        have hq : ∀ z ∈ closure (parallelOuterDomain K k),
            ‖Polynomial.eval z q‖ ≤ 1 := by
          intro z hz
          rw [Polynomial.eval_mul, Polynomial.eval_C, norm_mul]
          change ‖cM‖ * ‖Polynomial.eval z p‖ ≤ 1
          rw [show ‖cM‖ = M⁻¹ by
            simp [cM, norm_inv, Complex.norm_real, Real.norm_eq_abs,
              abs_of_pos hMpos]]
          apply (inv_mul_le_one₀ hMpos).mpr
          simpa only [M] using norm_polynomial_eval_le_maxOnSet
            hOuter.closure_isCompact hOuter.closure_nonempty p hz
        have hCauchy :=
          G.hasParametricPolynomialCauchyFormula_of_simpleDiagonalization
            R c B hDiag hWB
        have hnormalizedOperator :
            ‖euclideanOperator (polynomialEval q B)‖ ≤ 2 :=
          LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary
            (G.parametricBoundary R c) B hWB q hq hCauchy
        have hnormalized : ‖polynomialEval q B‖ ≤ 2 := by
          simpa only [matrix_norm_eq_euclidean_operator_norm] using
            hnormalizedOperator
        have heval : polynomialEval q B =
            cM • polynomialEval p B := by
          simp [q, cM, polynomialEval, Algebra.smul_def]
        rw [heval, norm_smul] at hnormalized
        have hcMnorm : ‖cM‖ = M⁻¹ := by
          simp [cM, norm_inv, Complex.norm_real, Real.norm_eq_abs,
            abs_of_pos hMpos]
        rw [hcMnorm] at hnormalized
        have hdiv : ‖polynomialEval p B‖ / M ≤ 2 := by
          simpa [div_eq_mul_inv, mul_comm] using hnormalized
        simpa only [B, M] using (div_le_iff₀ hMpos).mp hdiv
    exact norm_polynomialEval_le_of_tendsto p
      (tendsto_simpleSpectrumApproximation A) hSimpleBound
  have hmax : Tendsto
      (fun k ↦ maxPolynomialModulusOnSet
        (closure (parallelOuterDomain K k)) p)
      atTop (nhds (maxPolynomialModulusOnNumericalRange A p)) := by
    simpa only [K, maxPolynomialModulusOnSet_numericalRange] using
      tendsto_maxPolynomialModulusOnSet_of_outerApproximation
        p hKcompact hKne hCcompact
        (fun k ↦
          (parallelOuterDomain_data
            hKcompact (numericalRange_convex A) hKne k).closure_isCompact)
        (fun k ↦
          (parallelOuterDomain_data
            hKcompact (numericalRange_convex A) hKne k).contains.trans subset_closure)
        (fun k ↦ parallelOuterDomain_closure_subset_fixedNeighborhood K k)
        (by
          change K ⊆ Metric.cthickening 1 K
          exact Metric.self_subset_cthickening K)
        tendsto_outerApproximationRadius
        (fun k z hz ↦ parallelOuterDomain_closure_near hKcompact k hz)
  change ‖polynomialEval p A‖ ≤
    2 * maxPolynomialModulusOnNumericalRange A p
  exact le_of_tendsto_of_tendsto' tendsto_const_nhds
    (hmax.const_mul 2) hbound

end CrouzeixConjecture

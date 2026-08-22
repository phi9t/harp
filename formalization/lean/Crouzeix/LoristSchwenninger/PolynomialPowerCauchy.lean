import CrouzeixConjecture.ParametricBoundary
import CrouzeixConjecture.Spectrum

/-!
Reusable polynomial-power lemmas for the Lorist--Schwenninger contour boundary
integrals.
-/

noncomputable section

open MeasureTheory
open scoped BoundedContinuousFunction Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {i n : Type*} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n] [Nonempty n]
  {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}

omit [MeasurableSpace i] [OpensMeasurableSpace i] in
theorem parametricPolynomialBoundaryFunction_pow
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (q : Polynomial ℂ) (k : ℕ) :
    (parametricPolynomialBoundaryFunction Gamma q) ^ k =
      parametricPolynomialBoundaryFunction Gamma (q ^ k) := by
  ext x
  simp [parametricPolynomialBoundaryFunction, Polynomial.eval_pow]

omit [MeasurableSpace i] [OpensMeasurableSpace i] in
theorem parametricPolynomialBoundaryFunction_norm_le_one
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Omega, ‖Polynomial.eval z q‖ ≤ 1) :
    ‖parametricPolynomialBoundaryFunction Gamma q‖ ≤ 1 := by
  apply (BoundedContinuousFunction.norm_le
    (f := parametricPolynomialBoundaryFunction Gamma q) (by norm_num)).2
  intro x
  simpa [parametricPolynomialBoundaryFunction_apply] using
    hq (Gamma.point x) (frontier_subset_closure (Gamma.supported x).boundary_point)

omit [OpensMeasurableSpace i] [Nonempty n] [IsFiniteMeasure mu] in
theorem parametricPolynomialPowerCauchy
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (q : Polynomial ℂ) (B : SquareMatrix n)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B)
    (k : ℕ) :
    ∫ x, ((parametricPolynomialBoundaryFunction Gamma q) x) ^ k •
      parametricBoundaryFirstPart Gamma B x ∂mu = (polynomialEval q B) ^ k := by
  have hboundaryPower :
      (fun x ↦ ((parametricPolynomialBoundaryFunction Gamma q) x) ^ k •
        parametricBoundaryFirstPart Gamma B x) =
        fun x ↦ (parametricPolynomialBoundaryFunction Gamma (q ^ k)) x •
          parametricBoundaryFirstPart Gamma B x := by
    funext x
    rw [← BoundedContinuousFunction.pow_apply, parametricPolynomialBoundaryFunction_pow]
  rw [hboundaryPower]
  simpa [polynomialEval, map_pow] using hCauchy (q ^ k)

omit [Nonempty n] in
theorem polynomialEval_pow_mem_generatedAlgebra
    (q : Polynomial ℂ) (B : SquareMatrix n) (k : ℕ) :
    (polynomialEval q B) ^ k ∈ generatedAlgebra B := by
  exact Subalgebra.pow_mem _ (polynomialEval_mem_generatedAlgebra (p := q) (A := B)) k

omit [MeasurableSpace i] [OpensMeasurableSpace i] in
theorem parametricPolynomialBoundaryFunction_pow_norm_le_one
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Omega, ‖Polynomial.eval z q‖ ≤ 1)
    (k : ℕ) :
    ‖(parametricPolynomialBoundaryFunction Gamma q) ^ k‖ ≤ 1 := by
  apply (BoundedContinuousFunction.norm_le
    (f := (parametricPolynomialBoundaryFunction Gamma q) ^ k) (by norm_num)).2
  intro x
  rw [BoundedContinuousFunction.pow_apply, norm_pow]
  have hx :
      ‖parametricPolynomialBoundaryFunction Gamma q x‖ ≤ 1 := by
    simpa [parametricPolynomialBoundaryFunction_apply] using
      hq (Gamma.point x) (frontier_subset_closure (Gamma.supported x).boundary_point)
  exact pow_le_one₀ (norm_nonneg _) hx

end LoristSchwenninger
end CrouzeixConjecture

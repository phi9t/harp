import Crouzeix.Harp.FiniteMeasureCubature
import Crouzeix.Harp.FiniteHorizonDilation
import Crouzeix.LoristSchwenninger.CompanionAlgebra
import Crouzeix.LoristSchwenninger.PolynomialPowerCauchy
import Mathlib.Analysis.SpecialFunctions.ContinuousFunctionalCalculus.Rpow.Basic

/-!
Finite atomic dilations for the Harp finite-horizon Crouzeix argument.

The construction samples all matrix moments through the requested horizon in
one finite-dimensional real observable, factors the resulting positive atomic
matrices, and realizes the moments on a finite Euclidean block space.
-/

noncomputable section

namespace CrouzeixConjecture
namespace Harp

open MeasureTheory Set
open scoped BoundedContinuousFunction ComplexOrder Matrix MatrixOrder
  Matrix.Norms.L2Operator

universe u

variable {i : Type u} {n : Type} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n] [Nonempty n]
  {μ : Measure i} [IsFiniteMeasure μ] {Ω : Set ℂ}

/-- The single real finite-dimensional observable whose coordinates are the
matrix moments from power zero through `N + 1`. -/
def matrixMomentObservable (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (B : SquareMatrix n) (q : Polynomial ℂ) :
    i → (Fin (N + 2) → SquareMatrix n) :=
  fun x k ↦ ((parametricPolynomialBoundaryFunction Gamma q x) ^ (k : ℕ)) •
    parametricDoubleLayerDensity Gamma B x

omit [MeasurableSpace i] [OpensMeasurableSpace i] [Nonempty n] in
/-- The finite matrix-moment observable is continuous. -/
theorem continuous_matrixMomentObservable (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ) :
    Continuous (matrixMomentObservable N Gamma B q) := by
  apply continuous_pi
  intro k
  exact ((parametricPolynomialBoundaryFunction Gamma q).continuous.pow (k : ℕ)).smul
    (continuous_parametricDoubleLayerDensity Gamma B hWB)

omit [Nonempty n] in
/-- One positive cubature formula simultaneously preserves every matrix
moment from power zero through `N + 1`. -/
theorem exists_positive_matrix_moment_cubature (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (μ : Measure i) [IsFiniteMeasure μ] [NeZero μ]
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ) :
    ∃ (ι : Type) (_ : Fintype ι) (nodes : ι → i) (weights : ι → ℝ),
      Fintype.card ι ≤
        Module.finrank ℝ (Fin (N + 2) → SquareMatrix n) + 1 ∧
      (∀ j, 0 < weights j) ∧
      ∑ j, weights j = μ.real Set.univ ∧
      ∀ k : Fin (N + 2),
        ∑ j, weights j • matrixMomentObservable N Gamma B q (nodes j) k =
          ∫ x, matrixMomentObservable N Gamma B q x k ∂μ := by
  obtain ⟨ι, hι, nodes, weights, hcard, _hnodes, hweights_pos,
      hweights_sum, hweighted_sum⟩ :=
    exists_positive_cubature_finite_measure μ isCompact_univ MeasurableSet.univ
      (Filter.Eventually.of_forall fun _ ↦ Set.mem_univ _)
      (matrixMomentObservable N Gamma B q)
      (continuous_matrixMomentObservable N Gamma B hWB q).continuousOn
  refine ⟨ι, hι, nodes, weights, hcard, hweights_pos, hweights_sum, ?_⟩
  intro k
  have hall_integrable : ∀ l : Fin (N + 2), Integrable
      (fun x ↦ matrixMomentObservable N Gamma B q x l) μ := by
    intro l
    exact integrable_of_continuous_compact
      ((continuous_apply l).comp
        (continuous_matrixMomentObservable N Gamma B hWB q))
  calc
    ∑ j, weights j • matrixMomentObservable N Gamma B q (nodes j) k =
        (∑ j, weights j • matrixMomentObservable N Gamma B q (nodes j)) k := by
      simp
    _ = (∫ x, matrixMomentObservable N Gamma B q x ∂μ) k :=
      congrFun hweighted_sum k
    _ = ∫ x, matrixMomentObservable N Gamma B q x k ∂μ := by
      simpa using MeasureTheory.eval_integral hall_integrable k

/-- The horizon-independent target and companion family underlying every
finite atomic dilation. -/
def finiteHorizonPolynomialCore
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (μ : Measure i) [IsFiniteMeasure μ]
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1) :
    CommutingPerturbationData (EuclideanVector n) := by
  let h := parametricPolynomialBoundaryFunction Gamma q
  let P := polynomialEval q B
  let companion : ℕ → EuclideanVector n →L[ℂ] EuclideanVector n :=
    fun k ↦ euclideanOperator
      (parametricBoundaryCompanion Gamma μ B (h ^ k))
  let bound := ∫ x, ‖parametricBoundaryFirstPart Gamma B x‖ ∂μ
  refine
    { T := euclideanOperator P
      perturbation := companion
      bound := bound
      bound_nonneg := integral_nonneg fun _ ↦ norm_nonneg _
      perturbation_norm_le := ?_
      commutes_with_target := ?_ }
  · intro k
    change ‖parametricBoundaryCompanion Gamma μ B (h ^ k)‖ ≤ bound
    calc
      ‖parametricBoundaryCompanion Gamma μ B (h ^ k)‖
          ≤ ‖h ^ k‖ * bound :=
        LoristSchwenninger.parametricBoundaryCompanion_norm_le
          Gamma B hWB (h ^ k)
      _ ≤ 1 * bound := mul_le_mul_of_nonneg_right
        (LoristSchwenninger.parametricPolynomialBoundaryFunction_pow_norm_le_one
          Gamma q hq k)
        (integral_nonneg fun _ ↦ norm_nonneg _)
      _ = bound := one_mul _
  · intro k
    have hcommute :=
      LoristSchwenninger.parametricBoundaryCompanion_commute_of_mem_generatedAlgebra
        (mu := μ) Gamma B P hWB (h ^ k)
        (polynomialEval_mem_generatedAlgebra (p := q) (A := B))
    rw [commute_iff_eq] at hcommute ⊢
    simpa only [companion, P, map_mul] using
      congrArg (fun A : SquareMatrix n ↦ euclideanOperator A) hcommute

end Harp
end CrouzeixConjecture

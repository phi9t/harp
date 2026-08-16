import Mathlib.Analysis.InnerProductSpace.PiL2
import Mathlib.LinearAlgebra.Matrix.PosDef
import Mathlib.LinearAlgebra.Matrix.DotProduct

open Matrix

/-!
Finite-dimensional real-vector orthogonality and positive-definite quadratic-form facts.

This module covers only the explicitly stated finite-coordinate results below.  In particular,
it makes no claim about eigendecompositions, spectral theorems, or general orthogonal
decompositions.
-/

namespace MathematicalFoundations.Orthogonality

/-- A total finite real-vector line-projection expression.  Nonzero `u` is required only by the
orthogonality and nondegeneracy results that use this expression. -/
noncomputable def lineProjection {n : ℕ} (u x : Fin n → ℝ) : Fin n → ℝ :=
  (dotProduct x u / dotProduct u u) • u

/-- Every finite real vector is the sum of its line projection and its residual. -/
theorem line_projection_residual_decomposition {n : ℕ} (u x : Fin n → ℝ) :
    x = lineProjection u x + (x - lineProjection u x) := by
  abel

/-- The residual after projection onto a nonzero line is orthogonal to that line. -/
theorem line_projection_residual_inner_eq_zero {n : ℕ} {u x : Fin n → ℝ} (hu : u ≠ 0) :
    dotProduct (x - lineProjection u x) u = 0 := by
  have hdenom_ne : dotProduct u u ≠ 0 := by
    exact fun h => hu (dotProduct_self_eq_zero.mp h)
  rw [lineProjection, sub_dotProduct, smul_dotProduct]
  simp [smul_eq_mul]
  field_simp [hdenom_ne]
  ring

/-- A symmetric finite real matrix with a positive quadratic form on each nonzero vector has
strictly positive quadratic form at any explicitly nonzero vector. -/
theorem symmetric_positive_definite_quadratic_positive {n : ℕ}
    (A : Matrix (Fin n) (Fin n) ℝ) (_hSymm : A.IsSymm)
    (hPositive : ∀ x : Fin n → ℝ, x ≠ 0 → 0 < dotProduct x (A.mulVec x))
    {x : Fin n → ℝ} (hx : x ≠ 0) :
    0 < dotProduct x (A.mulVec x) := by
  exact hPositive x hx

end MathematicalFoundations.Orthogonality

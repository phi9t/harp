import Mathlib.Analysis.InnerProductSpace.PiL2
import Mathlib.LinearAlgebra.Matrix.PosDef

/-!
Finite-dimensional real-vector orthogonality and positive-definite quadratic-form facts.

This module covers only the explicitly stated finite-coordinate results below.  In particular,
it makes no claim about eigendecompositions, spectral theorems, or general orthogonal
decompositions.
-/

namespace MathematicalFoundations.Orthogonality

/-- The projection of `x` onto the line spanned by a nonzero finite real vector `u`. -/
def lineProjection {n : ℕ} (u x : Fin n → ℝ) : Fin n → ℝ :=
  (⟪x, u⟫_ℝ / ⟪u, u⟫_ℝ) • u

/-- Every finite real vector is the sum of its line projection and its residual. -/
theorem line_projection_residual_decomposition {n : ℕ} (u x : Fin n → ℝ) :
    x = lineProjection u x + (x - lineProjection u x) := by
  abel

/-- The residual after projection onto a nonzero line is orthogonal to that line. -/
theorem line_projection_residual_inner_eq_zero {n : ℕ} {u x : Fin n → ℝ} (hu : u ≠ 0) :
    ⟪x - lineProjection u x, u⟫_ℝ = 0 := by
  have hdenom_pos : 0 < ⟪u, u⟫_ℝ := real_inner_self_pos.mpr hu
  have hdenom_ne : ⟪u, u⟫_ℝ ≠ 0 := ne_of_gt hdenom_pos
  rw [lineProjection, inner_sub_left, real_inner_smul_left]
  field_simp

/-- A symmetric finite real matrix with a positive quadratic form on each nonzero vector has
strictly positive quadratic form at any explicitly nonzero vector. -/
theorem symmetric_positive_definite_quadratic_positive {n : ℕ}
    (A : Matrix (Fin n) (Fin n) ℝ) (hSymm : A.IsSymm)
    (hPositive : ∀ x : Fin n → ℝ, x ≠ 0 → 0 < x ⬝ᵥ (A *ᵥ x))
    {x : Fin n → ℝ} (hx : x ≠ 0) :
    0 < x ⬝ᵥ (A *ᵥ x) := by
  exact hPositive x hx

end MathematicalFoundations.Orthogonality

import Mathlib.LinearAlgebra.Matrix.ToLin

/-!
Finite real-coordinate linear-model identities.

This module formalizes only exact algebraic identities for matrices and vectors with `Fin` indices.
It does not assert that normal equations characterize minimizers.
-/

namespace MathematicalFoundations.LinearModels

/-- The finite-coordinate squared-error ridge objective. -/
def ridgeObjective {m n : ℕ} (A : Matrix (Fin m) (Fin n) ℝ)
    (y : Fin m → ℝ) (λ : ℝ) (β : Fin n → ℝ) : ℝ :=
  (A *ᵥ β - y) ⬝ᵥ (A *ᵥ β - y) + λ * (β ⬝ᵥ β)

/-- A zero normal-equation residual is equivalent to equality of the two displayed terms.
This is an algebraic identity, not a minimization claim. -/
theorem normal_equation_residual_identity {m n : ℕ} (A : Matrix (Fin m) (Fin n) ℝ)
    (y : Fin m → ℝ) (β : Fin n → ℝ)
    (h : A.transpose *ᵥ (A *ᵥ β - y) = 0) :
    A.transpose *ᵥ (A *ᵥ β) = A.transpose *ᵥ y := by
  rw [Matrix.mulVec_sub] at h
  exact sub_eq_zero.mp h

/-- At the zero coefficient vector, the ridge objective is the squared response length. -/
theorem ridge_objective_zero_coefficients {m n : ℕ} (A : Matrix (Fin m) (Fin n) ℝ)
    (y : Fin m → ℝ) (λ : ℝ) :
    ridgeObjective A y λ 0 = y ⬝ᵥ y := by
  simp [ridgeObjective]

end MathematicalFoundations.LinearModels

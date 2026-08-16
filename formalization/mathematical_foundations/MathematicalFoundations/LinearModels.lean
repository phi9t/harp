import Mathlib.LinearAlgebra.Matrix.ToLin
import Mathlib.Data.Real.Basic

open Matrix

/-!
Finite real-coordinate linear-model identities.

This module formalizes only exact algebraic identities for matrices and vectors with `Fin` indices.
It does not assert that normal equations characterize minimizers.
-/

namespace MathematicalFoundations.LinearModels

/-- The finite-coordinate squared-error ridge objective. -/
def ridgeObjective {m n : ℕ} (A : Matrix (Fin m) (Fin n) ℝ)
    (y : Fin m → ℝ) (lambda : ℝ) (β : Fin n → ℝ) : ℝ :=
  dotProduct (A.mulVec β - y) (A.mulVec β - y) + lambda * dotProduct β β

/-- A zero normal-equation residual implies equality of the two displayed terms.
This is an algebraic identity, not a minimization claim. -/
theorem normal_equation_residual_identity {m n : ℕ} (A : Matrix (Fin m) (Fin n) ℝ)
    (y : Fin m → ℝ) (β : Fin n → ℝ)
    (h : A.transpose.mulVec (A.mulVec β - y) = 0) :
    A.transpose.mulVec (A.mulVec β) = A.transpose.mulVec y := by
  rw [Matrix.mulVec_sub] at h
  exact sub_eq_zero.mp h

/-- At the zero coefficient vector, the ridge objective is the squared response length. -/
theorem ridge_objective_zero_coefficients {m n : ℕ} (A : Matrix (Fin m) (Fin n) ℝ)
    (y : Fin m → ℝ) (lambda : ℝ) :
    ridgeObjective A y lambda (fun _ => 0) = dotProduct y y := by
  simp [ridgeObjective, dotProduct, Matrix.mulVec]

end MathematicalFoundations.LinearModels

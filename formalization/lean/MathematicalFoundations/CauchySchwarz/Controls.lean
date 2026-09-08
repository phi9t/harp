import MathematicalFoundations.CauchySchwarz.Quadratic
import MathematicalFoundations.CauchySchwarz.Lagrange
import MathematicalFoundations.CauchySchwarz.Normalization
import MathematicalFoundations.CauchySchwarz.Induction
import MathematicalFoundations.CauchySchwarz.Equality
import Mathlib.Tactic.NormNum

open scoped BigOperators
namespace TextbookBench.Controls

example : ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤ (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  cauchySchwarzQuadratic

example : ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤ (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  cauchySchwarzLagrange

example : ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤ (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  cauchySchwarzNormalization

example : ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤ (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  cauchySchwarzInduction

example (x y : Fin 0 → ℝ) :
    (∑ i, x i * y i) ^ 2 = (∑ i, x i ^ 2) * (∑ i, y i ^ 2) := by simp

example (n : ℕ) (y : Fin n → ℝ) :
    (∑ i, (0 : ℝ) * y i) ^ 2 ≤ (∑ _i : Fin n, (0 : ℝ) ^ 2) * (∑ i, y i ^ 2) :=
  cauchySchwarzQuadratic n (fun _ => 0) y

example (n : ℕ) (x : Fin n → ℝ) :
    (∑ i, x i * (0 : ℝ)) ^ 2 ≤ (∑ i, x i ^ 2) * (∑ _i : Fin n, (0 : ℝ) ^ 2) :=
  cauchySchwarzNormalization n x (fun _ => 0)

example (n : ℕ) (x : Fin n → ℝ) :
    (∑ i, x i * (-2 * x i)) ^ 2 = (∑ i, x i ^ 2) * (∑ i, (-2 * x i) ^ 2) :=
  (cauchySchwarzEqualityIffProportional n x _).mpr (Or.inr ⟨-2, rfl⟩)

example : (∑ i : Fin 2, (![1, 0] : Fin 2 → ℝ) i * ![0, 1] i) ^ 2 <
    (∑ i : Fin 2, (![1, 0] : Fin 2 → ℝ) i ^ 2) *
    (∑ i : Fin 2, (![0, 1] : Fin 2 → ℝ) i ^ 2) := by
  apply cauchySchwarzStrictOfMinor
  exact ⟨0, 1, by norm_num⟩

-- The x = 0 alternative cannot be dropped from the equality characterization.
example : ¬ (∀ (x y : Fin 1 → ℝ),
    (∑ i, x i * y i) ^ 2 = (∑ i, x i ^ 2) * (∑ i, y i ^ 2) →
      ∃ t : ℝ, y = fun i => t * x i) := by
  intro h
  obtain ⟨t, ht⟩ := h (fun _ => 0) (fun _ => 1) (by simp)
  have := congrFun ht 0
  norm_num at this

-- Equality itself is false without a dependence hypothesis.
example : ¬ (∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 = (∑ i, x i ^ 2) * (∑ i, y i ^ 2)) := by
  intro h
  have hh := h 2 (![1, 0]) (![0, 1])
  have hm := (cauchySchwarzEqualityIffMinors 2 (![1, 0]) (![0, 1])).mp hh 0 1
  norm_num at hm

end TextbookBench.Controls

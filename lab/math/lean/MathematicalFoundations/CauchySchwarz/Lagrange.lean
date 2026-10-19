import MathematicalFoundations.CauchySchwarz.Elementary
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

open scoped BigOperators
namespace TextbookBench

/-- The pairwise-minor identity, expanded directly from the finite sums. -/
theorem lagrangeExpansion (n : ℕ) (x y : Fin n → ℝ) :
    (∑ i, ∑ j, (x i * y j - x j * y i) ^ 2) =
      2 * ((∑ i, x i ^ 2) * (∑ i, y i ^ 2) - (∑ i, x i * y i) ^ 2) := by
  have term (i j : Fin n) : (x i * y j - x j * y i) ^ 2 =
      x i ^ 2 * y j ^ 2 + y i ^ 2 * x j ^ 2 - 2 * (x i * y i) * (x j * y j) := by ring
  simp_rw [term, Finset.sum_sub_distrib, Finset.sum_add_distrib,
    ← Finset.mul_sum, ← Finset.sum_mul]
  have hc : (∑ i, 2 * (x i * y i)) = 2 * ∑ i, x i * y i := by rw [Finset.mul_sum]
  rw [hc]
  ring

/-- A sum of pairwise-minor squares is nonnegative. -/
theorem cauchySchwarzLagrange : CauchySchwarzTarget := by
  intro n x y
  have h : 0 ≤ ∑ i, ∑ j, (x i * y j - x j * y i) ^ 2 :=
    Finset.sum_nonneg fun i _ => Elementary.sum_sq_nonneg _ _
  rw [lagrangeExpansion] at h
  linarith

end TextbookBench

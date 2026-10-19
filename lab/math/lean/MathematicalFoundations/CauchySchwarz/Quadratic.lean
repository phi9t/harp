import MathematicalFoundations.CauchySchwarz.Elementary
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

open scoped BigOperators
namespace TextbookBench

/-- Nonnegativity of a quadratic, evaluated at its minimizing parameter. -/
theorem cauchySchwarzQuadratic : CauchySchwarzTarget := by
  intro n x y
  let A := ∑ i, x i ^ 2
  let B := ∑ i, y i ^ 2
  let C := ∑ i, x i * y i
  change C ^ 2 ≤ A * B
  have hB : 0 ≤ B := Elementary.sum_sq_nonneg _ y
  by_cases hz : B = 0
  · have hy := Elementary.zero_of_sum_sq_zero y hz
    have hC : C = 0 := by simp [C, hy]
    simp [hC, hz]
  · have hp : 0 < B := lt_of_le_of_ne hB (Ne.symm hz)
    have expand (t : ℝ) : ∑ i, (x i - t * y i) ^ 2 = A - 2 * t * C + t ^ 2 * B := by
      simp only [A, B, C, sub_sq, mul_pow, Finset.sum_add_distrib,
        Finset.sum_sub_distrib, Finset.mul_sum]
      congr 1
      congr 1
      apply Finset.sum_congr rfl
      intro i _
      ring
    have h := Elementary.sum_sq_nonneg Finset.univ (fun i => x i - (C / B) * y i)
    rw [expand] at h
    have hmul := mul_nonneg h hB
    field_simp at hmul
    nlinarith [sq_nonneg C]

end TextbookBench

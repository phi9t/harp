import MathematicalFoundations.CauchySchwarz.Elementary
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Positivity
import Mathlib.Tactic.Ring

open scoped BigOperators
namespace TextbookBench

/-- Inserting one coordinate preserves the nonnegative determinant defect. -/
theorem cauchySchwarzInduction : CauchySchwarzTarget := by
  intro n x y
  have finite (s : Finset (Fin n)) :
      (∑ i ∈ s, x i * y i) ^ 2 ≤ (∑ i ∈ s, x i ^ 2) * (∑ i ∈ s, y i ^ 2) := by
    induction s using Finset.induction_on with
    | empty => simp
    | @insert k s hk ih =>
      let A := ∑ i ∈ s, x i ^ 2
      let B := ∑ i ∈ s, y i ^ 2
      let C := ∑ i ∈ s, x i * y i
      let a := x k
      let b := y k
      have hA : 0 ≤ A := Elementary.sum_sq_nonneg _ _
      have hB : 0 ≤ B := Elementary.sum_sq_nonneg _ _
      have hdef : 0 ≤ A * B - C ^ 2 := sub_nonneg.mpr ih
      have hid : (a ^ 2 * B + b ^ 2 * A) ^ 2 - (2 * a * b * C) ^ 2 =
          (a ^ 2 * B - b ^ 2 * A) ^ 2 + 4 * a ^ 2 * b ^ 2 * (A * B - C ^ 2) := by ring
      have hproduct : 0 ≤ 4 * a ^ 2 * b ^ 2 * (A * B - C ^ 2) := by positivity
      have hbase : 0 ≤ a ^ 2 * B + b ^ 2 * A := by positivity
      have hcross : 2 * a * b * C ≤ a ^ 2 * B + b ^ 2 * A := by
        nlinarith [sq_nonneg (a ^ 2 * B - b ^ 2 * A)]
      simp only [Finset.sum_insert hk]
      change (a * b + C) ^ 2 ≤ (a ^ 2 + A) * (b ^ 2 + B)
      nlinarith
  exact finite Finset.univ

end TextbookBench

import MathematicalFoundations.TextbookCauchySchwarzTarget
import Mathlib.Tactic.Linarith

open scoped BigOperators
namespace TextbookBench.Elementary

theorem sum_sq_nonneg {ι : Type*} (s : Finset ι) (x : ι → ℝ) :
    0 ≤ ∑ i ∈ s, x i ^ 2 :=
  Finset.sum_nonneg fun i _ => sq_nonneg (x i)

theorem zero_of_sum_sq_zero {n : ℕ} (x : Fin n → ℝ)
    (h : ∑ i, x i ^ 2 = 0) : ∀ i, x i = 0 := by
  intro i
  have hi : x i ^ 2 ≤ ∑ j, x j ^ 2 :=
    Finset.single_le_sum (fun j _ => sq_nonneg (x j)) (Finset.mem_univ i)
  have : x i ^ 2 = 0 := le_antisymm (h ▸ hi) (sq_nonneg _)
  nlinarith [sq_nonneg (x i)]

end TextbookBench.Elementary

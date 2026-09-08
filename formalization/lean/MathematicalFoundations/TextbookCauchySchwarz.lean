import Mathlib.Data.Real.Basic
import Mathlib.Algebra.Order.BigOperators.Ring.Finset

open scoped BigOperators

namespace TextbookBench

/-- The fixed first workload, including dimension zero and zero vectors. -/
def CauchySchwarzTarget : Prop :=
  ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2)

/-- A library-reuse control, not evidence of theorem discovery. -/
theorem cauchySchwarzReference : CauchySchwarzTarget := by
  intro n x y
  simpa using Finset.sum_mul_sq_le_sq_mul_sq Finset.univ x y

end TextbookBench

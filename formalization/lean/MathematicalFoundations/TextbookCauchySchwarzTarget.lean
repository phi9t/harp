import Mathlib.Data.Real.Basic
import Mathlib.Algebra.Order.BigOperators.Ring.Finset

open scoped BigOperators
namespace TextbookBench

/-- The fixed first workload, including dimension zero and zero vectors. -/
def CauchySchwarzTarget : Prop :=
  ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2)

end TextbookBench

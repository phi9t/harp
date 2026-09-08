import MathematicalFoundations.TextbookCauchySchwarzTarget

open scoped BigOperators

namespace TextbookBench

/-- A library-reuse control, not evidence of theorem discovery. -/
theorem cauchySchwarzReference : CauchySchwarzTarget := by
  intro n x y
  simpa using Finset.sum_mul_sq_le_sq_mul_sq Finset.univ x y

end TextbookBench

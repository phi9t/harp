import MathematicalFoundations.TextbookCauchySchwarz

open scoped BigOperators

namespace TextbookBench.Controls

-- This spells out the target independently of the named proposition.
example : ∀ (n : ℕ) (x y : Fin n → ℝ),
    (∑ i, x i * y i) ^ 2 ≤
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  TextbookBench.cauchySchwarzReference

-- The public reference must remain applicable at dimension zero.
example (x y : Fin 0 → ℝ) :
    (∑ i, x i * y i) ^ 2 ≤
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2) :=
  TextbookBench.cauchySchwarzReference 0 x y

-- Zero vectors require no side condition.
example (n : ℕ) (y : Fin n → ℝ) :
    (∑ i, (0 : ℝ) * y i) ^ 2 ≤
      (∑ _i : Fin n, (0 : ℝ) ^ 2) * (∑ i, y i ^ 2) :=
  TextbookBench.cauchySchwarzReference n (fun _ => 0) y

-- An independent simplification control for the empty sum convention.
example (x y : Fin 0 → ℝ) :
    (∑ i, x i * y i) ^ 2 =
      (∑ i, x i ^ 2) * (∑ i, y i ^ 2) := by
  simp

end TextbookBench.Controls

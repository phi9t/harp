import MathematicalFoundations.CauchySchwarz.Lagrange
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Push
import Mathlib.Tactic.Ring

open scoped BigOperators
namespace TextbookBench

theorem cauchySchwarzEqualityIffMinors (n : ℕ) (x y : Fin n → ℝ) :
    (∑ i, x i * y i) ^ 2 = (∑ i, x i ^ 2) * (∑ i, y i ^ 2) ↔
      ∀ i j, x i * y j = x j * y i := by
  constructor
  · intro h i j
    have hz : (∑ i, ∑ j, (x i * y j - x j * y i) ^ 2) = 0 := by
      rw [lagrangeExpansion, h]; ring
    have hrow : (∑ j, (x i * y j - x j * y i) ^ 2) ≤
        ∑ i, ∑ j, (x i * y j - x j * y i) ^ 2 :=
      Finset.single_le_sum (fun i _ => Elementary.sum_sq_nonneg Finset.univ
        (fun j => x i * y j - x j * y i)) (Finset.mem_univ i)
    have hterm : (x i * y j - x j * y i) ^ 2 ≤
        ∑ j, (x i * y j - x j * y i) ^ 2 :=
      Finset.single_le_sum (fun j _ => sq_nonneg (x i * y j - x j * y i)) (Finset.mem_univ j)
    nlinarith [sq_nonneg (x i * y j - x j * y i)]
  · intro h
    have hz : (∑ i, ∑ j, (x i * y j - x j * y i) ^ 2) = 0 := by simp [h]
    rw [lagrangeExpansion] at hz
    linarith

theorem cauchySchwarzEqualityIffProportional (n : ℕ) (x y : Fin n → ℝ) :
    (∑ i, x i * y i) ^ 2 = (∑ i, x i ^ 2) * (∑ i, y i ^ 2) ↔
      x = 0 ∨ ∃ t : ℝ, y = fun i => t * x i := by
  rw [cauchySchwarzEqualityIffMinors]
  constructor
  · intro h
    by_cases hx : x = 0
    · exact Or.inl hx
    · right
      have hex : ∃ k, x k ≠ 0 := by
        by_contra hn
        push Not at hn
        exact hx (funext hn)
      obtain ⟨k, hk⟩ := hex
      refine ⟨y k / x k, funext fun i => ?_⟩
      have hi := h k i
      field_simp
      nlinarith [hi]
  · rintro (hx | ⟨t, rfl⟩) i j
    · simp [hx]
    · ring

theorem cauchySchwarzStrictOfMinor (n : ℕ) (x y : Fin n → ℝ)
    (h : ∃ i j, x i * y j ≠ x j * y i) :
    (∑ i, x i * y i) ^ 2 < (∑ i, x i ^ 2) * (∑ i, y i ^ 2) := by
  refine lt_of_le_of_ne (cauchySchwarzLagrange n x y) ?_
  intro he
  obtain ⟨i, j, hij⟩ := h
  exact hij ((cauchySchwarzEqualityIffMinors n x y).mp he i j)

end TextbookBench

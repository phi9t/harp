import MathematicalFoundations.CauchySchwarz.Elementary
import Mathlib.Analysis.Real.Sqrt
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

open scoped BigOperators
namespace TextbookBench

/-- Normalize both vectors, bound their pairing with plus/minus squares, then rescale. -/
theorem cauchySchwarzNormalization : CauchySchwarzTarget := by
  intro n x y
  let A := ∑ i, x i ^ 2
  let B := ∑ i, y i ^ 2
  have hA : 0 ≤ A := Elementary.sum_sq_nonneg _ _
  have hB : 0 ≤ B := Elementary.sum_sq_nonneg _ _
  by_cases ha : A = 0
  · have hx := Elementary.zero_of_sum_sq_zero x ha
    simp [hx]
  by_cases hb : B = 0
  · have hy := Elementary.zero_of_sum_sq_zero y hb
    simp [hy]
  let r := Real.sqrt A
  let s := Real.sqrt B
  have hr : r ^ 2 = A := Real.sq_sqrt hA
  have hs : s ^ 2 = B := Real.sq_sqrt hB
  have hr0 : r ≠ 0 := by intro h; simp [h] at hr; exact ha hr.symm
  have hs0 : s ≠ 0 := by intro h; simp [h] at hs; exact hb hs.symm
  let u := fun i => x i / r
  let v := fun i => y i / s
  have hu : ∑ i, u i ^ 2 = 1 := by
    simp only [u, div_pow, hr]
    simp only [div_eq_mul_inv, ← Finset.sum_mul]
    exact mul_inv_cancel₀ ha
  have hv : ∑ i, v i ^ 2 = 1 := by
    simp only [v, div_pow, hs]
    simp only [div_eq_mul_inv, ← Finset.sum_mul]
    exact mul_inv_cancel₀ hb
  let D := ∑ i, u i * v i
  have plus : ∑ i, (u i + v i) ^ 2 = 2 + 2 * D := by
    simp_rw [add_sq, Finset.sum_add_distrib]
    rw [hu, hv]
    simp only [D, Finset.mul_sum]
    have he : (∑ i, 2 * u i * v i) = ∑ i, 2 * (u i * v i) := by
      apply Finset.sum_congr rfl
      intro i _
      ring
    rw [he]
    ring
  have minus : ∑ i, (u i - v i) ^ 2 = 2 - 2 * D := by
    simp_rw [sub_sq, Finset.sum_add_distrib, Finset.sum_sub_distrib]
    rw [hu, hv]
    simp only [D, Finset.mul_sum]
    have he : (∑ i, 2 * u i * v i) = ∑ i, 2 * (u i * v i) := by
      apply Finset.sum_congr rfl
      intro i _
      ring
    rw [he]
    ring
  have hp := Elementary.sum_sq_nonneg Finset.univ (fun i => u i + v i)
  have hm := Elementary.sum_sq_nonneg Finset.univ (fun i => u i - v i)
  rw [plus] at hp
  rw [minus] at hm
  have lower : -1 ≤ D := by linarith
  have upper : D ≤ 1 := by linarith
  have hd : D ^ 2 ≤ 1 := by nlinarith [mul_nonneg (sub_nonneg.mpr upper) (by linarith : 0 ≤ D + 1)]
  have pairing : ∑ i, x i * y i = r * s * D := by
    simp only [D, u, v, Finset.mul_sum]
    apply Finset.sum_congr rfl
    intro i _
    field_simp
  rw [pairing]
  change (r * s * D) ^ 2 ≤ A * B
  have hscale := mul_le_mul_of_nonneg_left hd (mul_nonneg hA hB)
  simpa only [mul_pow, hr, hs, mul_one] using hscale

end TextbookBench

import Crouzeix.LoristSchwenninger.Dilation
import Crouzeix.LoristSchwenninger.NormAttainment

/-!
The Hilbert-space completed-square step in the Lorist--Schwenninger
perturbation recurrence.
-/

noncomputable section

open scoped InnerProduct

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {E : Type*} [NormedAddCommGroup E] [InnerProductSpace ℂ E]

/-- Completing the square with mathlib's convention that the inner product is
conjugate-linear in its first argument and linear in its second argument. -/
theorem completed_square_identity
    {a : ℝ} (ha : 0 < a) (u y : E) :
    a * ‖u - ((((2 * a)⁻¹ : ℝ) : ℂ) • y)‖ ^ 2 -
        ‖y‖ ^ 2 / (4 * a) =
      a * ‖u‖ ^ 2 - (inner ℂ y u).re := by
  have ha0 : a ≠ 0 := ne_of_gt ha
  have hscale_pos : 0 < (2 * a)⁻¹ := by positivity
  have hscale : ‖(((2 * a)⁻¹ : ℝ) : ℂ)‖ = (2 * a)⁻¹ := by
    simp only [Complex.norm_real, Real.norm_eq_abs, abs_of_pos hscale_pos]
  have hinner :
      RCLike.re ((((2 * a)⁻¹ : ℝ) : ℂ) * inner ℂ u y) =
        (2 * a)⁻¹ * (inner ℂ y u).re := by
    simp only [RCLike.re_to_complex, Complex.mul_re, Complex.ofReal_re,
      Complex.ofReal_im, zero_mul, sub_zero]
    congr 1
    exact @inner_re_symm ℂ E _ _ _ u y
  rw [@norm_sub_sq ℂ E _ _ _ _, inner_smul_right, norm_smul]
  rw [hscale]
  rw [hinner]
  field_simp [ha0]
  ring

/-- An exact pre-square identity gives the lower bound obtained by discarding
the nonnegative completed square. -/
theorem pre_square_lower_bound
    {a lhs : ℝ} (ha : 0 < a) (u y : E)
    (hpre : lhs = a * ‖u‖ ^ 2 - (inner ℂ y u).re) :
    -‖y‖ ^ 2 / (4 * a) ≤ lhs := by
  rw [hpre, ← completed_square_identity ha u y]
  have hsquare :
      0 ≤ a * ‖u - ((((2 * a)⁻¹ : ℝ) : ℂ) • y)‖ ^ 2 :=
    mul_nonneg ha.le (sq_nonneg _)
  rw [neg_div]
  simpa only [zero_sub] using
    sub_le_sub_right hsquare (‖y‖ ^ 2 / (4 * a))

namespace DilationData

variable {K : Type*}
variable [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

/-- The exact LS pre-square identity and the common-displacement defect bound
give the source's `n`-independent recurrence lower bound. -/
theorem recurrence_lower_bound_of_pre_square_identity
    (data : DilationData (E := E) (K := K))
    {κ : ℝ} (hκ : 1 < κ) (m : ℕ → ℝ) (n : ℕ) (x u : E)
    (hpre :
      κ * m n - m (n + 1) =
        (κ ^ 2 - κ) * ‖u‖ ^ 2 -
          (inner ℂ (data.adjacentPowerDefect κ n x) u).re) :
    -‖data.displacement κ x‖ ^ 2 / (κ ^ 2 - κ) ≤
      κ * m n - m (n + 1) := by
  have ha : 0 < κ ^ 2 - κ := by nlinarith
  have hcompleted := pre_square_lower_bound ha u
    (data.adjacentPowerDefect κ n x) hpre
  have hdefect := data.adjacentPowerDefect_norm_le κ n x
  have hsq :
      ‖data.adjacentPowerDefect κ n x‖ ^ 2 ≤
        (2 * ‖data.displacement κ x‖) ^ 2 :=
    pow_le_pow_left₀ (norm_nonneg _) hdefect 2
  have herror :
      -‖data.displacement κ x‖ ^ 2 / (κ ^ 2 - κ) ≤
        -‖data.adjacentPowerDefect κ n x‖ ^ 2 /
          (4 * (κ ^ 2 - κ)) := by
    rw [div_le_div_iff₀ ha (mul_pos (by norm_num) ha)]
    nlinarith
  exact herror.trans hcompleted

end DilationData

end LoristSchwenninger
end CrouzeixConjecture

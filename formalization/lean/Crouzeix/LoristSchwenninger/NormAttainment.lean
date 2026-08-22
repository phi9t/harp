import Mathlib.Analysis.InnerProductSpace.Rayleigh

/-!
Finite-dimensional norm attainment for the Lorist-Schwenninger route.

The operator norm of a continuous linear endomorphism is attained on the unit
sphere.  A maximizing vector is subsequently promoted to a top right singular
vector.
-/

noncomputable section

namespace CrouzeixConjecture
namespace LoristSchwenninger

open Metric

variable {E : Type*} [NormedAddCommGroup E] [InnerProductSpace ℂ E]

local instance finiteDimensionalCompleteSpace
    [FiniteDimensional ℂ E] : CompleteSpace E :=
  FiniteDimensional.complete ℂ E

/-- A continuous linear endomorphism of a nontrivial finite-dimensional complex
Hilbert space attains its operator norm on the unit sphere. -/
theorem exists_unit_norm_apply_eq_opNorm
    [FiniteDimensional ℂ E] [Nontrivial E] (T : E →L[ℂ] E) :
    ∃ x : E, ‖x‖ = 1 ∧ ‖T x‖ = ‖T‖ := by
  letI := FiniteDimensional.proper_rclike ℂ E
  have hsphere : IsCompact (sphere (0 : E) 1) := isCompact_sphere _ _
  have hsphere_nonempty : (sphere (0 : E) 1).Nonempty :=
    NormedSpace.sphere_nonempty.mpr zero_le_one
  obtain ⟨x, hx, hxmax⟩ :=
    hsphere.exists_isMaxOn hsphere_nonempty
      (continuous_norm.comp T.continuous).continuousOn
  have hxnorm : ‖x‖ = 1 := by
    simpa only [mem_sphere_zero_iff_norm] using hx
  refine ⟨x, hxnorm, le_antisymm ?_ ?_⟩
  · simpa only [hxnorm, mul_one] using T.le_opNorm x
  · refine T.opNorm_le_of_unit_norm (norm_nonneg (T x)) ?_
    intro y hy
    exact hxmax (mem_sphere_zero_iff_norm.mpr hy)

/-- A unit vector at which `T` attains its norm is a top right singular vector. -/
theorem adjoint_apply_apply_eq_opNorm_sq_smul_of_norm_attaining
    [CompleteSpace E] (T : E →L[ℂ] E) {x : E}
    (hx : ‖x‖ = 1) (hTx : ‖T x‖ = ‖T‖) :
    T.adjoint (T x) = ((‖T‖ ^ 2 : ℝ) : ℂ) • x := by
  let G : E →L[ℂ] E := T.adjoint ∘L T
  have hG : IsSelfAdjoint G := by
    rw [ContinuousLinearMap.isSelfAdjoint_iff']
    simp [G]
  have hmax : IsMaxOn G.reApplyInnerSelf (sphere (0 : E) ‖x‖) x := by
    intro y hy
    have hynorm : ‖y‖ = 1 := by
      simpa only [mem_sphere_zero_iff_norm, hx] using hy
    change G.reApplyInnerSelf y ≤ G.reApplyInnerSelf x
    rw [ContinuousLinearMap.reApplyInnerSelf_apply,
      ← T.apply_norm_sq_eq_inner_adjoint_left y]
    rw [ContinuousLinearMap.reApplyInnerSelf_apply,
      ← T.apply_norm_sq_eq_inner_adjoint_left x]
    apply pow_le_pow_left₀ (norm_nonneg (T y))
    calc
      ‖T y‖ ≤ ‖T‖ * ‖y‖ := T.le_opNorm y
      _ = ‖T x‖ := by rw [hynorm, mul_one, hTx]
  have heigen := hG.eq_smul_self_of_isLocalExtrOn (Or.inr hmax.localize)
  have hrayleigh : G.rayleighQuotient x = ‖T‖ ^ 2 := by
    rw [ContinuousLinearMap.rayleighQuotient]
    rw [ContinuousLinearMap.reApplyInnerSelf_apply,
      ← T.apply_norm_sq_eq_inner_adjoint_left x, hx, hTx]
    norm_num
  calc
    T.adjoint (T x) = G x := rfl
    _ = (G.rayleighQuotient x : ℂ) • x := heigen
    _ = ((‖T‖ ^ 2 : ℝ) : ℂ) • x := by rw [hrayleigh]

/-- A continuous linear endomorphism of a nontrivial finite-dimensional complex
Hilbert space has a unit top right singular vector. -/
theorem exists_unit_norm_attaining_and_adjoint_apply
    [FiniteDimensional ℂ E] [Nontrivial E] (T : E →L[ℂ] E) :
    ∃ x : E,
      ‖x‖ = 1 ∧
        ‖T x‖ = ‖T‖ ∧
          T.adjoint (T x) = ((‖T‖ ^ 2 : ℝ) : ℂ) • x := by
  obtain ⟨x, hx, hTx⟩ := exists_unit_norm_apply_eq_opNorm T
  exact ⟨x, hx, hTx,
    adjoint_apply_apply_eq_opNorm_sq_smul_of_norm_attaining T hx hTx⟩

end LoristSchwenninger
end CrouzeixConjecture

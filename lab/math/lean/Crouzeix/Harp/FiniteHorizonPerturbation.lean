import Crouzeix.Harp.FiniteHorizonOperatorRecurrence
import Crouzeix.LoristSchwenninger.NormAttainment
import Crouzeix.LoristSchwenninger.Scalar

/-!
Terminal perturbation theorem for horizon-indexed finite dilation witnesses.
-/

noncomputable section

namespace CrouzeixConjecture
namespace Harp

variable {E : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]

theorem norm_target_le_two_of_finiteHorizonDilationData
    {K : ℕ → Type*}
    [∀ N, NormedAddCommGroup (K N)]
    [∀ N, InnerProductSpace ℂ (K N)]
    [∀ N, CompleteSpace (K N)]
    (core : CommutingPerturbationData E)
    (data : ∀ N, FiniteHorizonDilationData core (K N) N) :
    ‖core.T‖ ≤ 2 := by
  by_cases hsmall : ‖core.T‖ ≤ 1
  · linarith
  · have hnorm : 1 < ‖core.T‖ := lt_of_not_ge hsmall
    obtain ⟨x, hx, _hTx, hsingular⟩ :=
      LoristSchwenninger.exists_unit_norm_attaining_and_adjoint_apply core.T
    let κ : ℝ := ‖core.T‖
    let m : ℕ → ℝ := core.recurrenceScalar x
    let C : ℝ := 2 * κ ^ 2 - κ * m 1 - κ ^ 3
    have hκ : 1 < κ := by simpa [κ] using hnorm
    have hsingular' :
        ContinuousLinearMap.adjoint core.T (core.T x) =
          ((κ ^ 2 : ℝ) : ℂ) • x := by
      simpa [κ] using hsingular
    have hfinite : ∀ N,
        (κ⁻¹) ^ N * m (N + 1) +
            (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) *
              (-C / (κ ^ 2 - κ)) ≤
          m 1 := by
      intro N
      have heq3 := (data N).equation_three_finite_lower_bound
        hκ x hsingular'
      have heq4 := (data N).displacementSq_le hx (by rfl) hsingular'
      have hb_le_C : (data N).displacementSq κ x ≤ C := by
        simpa [C, m] using heq4
      have hdenpos : 0 < κ ^ 2 - κ := by nlinarith
      have hcoeff :
          -C / (κ ^ 2 - κ) ≤
            -(data N).displacementSq κ x / (κ ^ 2 - κ) := by
        rw [div_le_div_iff₀ hdenpos hdenpos]
        nlinarith
      have hweight :
          0 ≤ ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) := by
        exact Finset.sum_nonneg fun i _hi =>
          pow_nonneg (inv_nonneg.mpr (le_trans zero_le_one hκ.le)) _
      calc
        (κ⁻¹) ^ N * m (N + 1) +
              (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) *
                (-C / (κ ^ 2 - κ))
            ≤ (κ⁻¹) ^ N * m (N + 1) +
                (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) *
                  (-(data N).displacementSq κ x / (κ ^ 2 - κ)) :=
              add_le_add le_rfl
                (mul_le_mul_of_nonneg_left hcoeff hweight)
        _ ≤ m 1 := by simpa [m] using heq3
    have hm : ∀ n, |m n| ≤ core.bound * (2 + core.bound) := by
      intro n
      dsimp [m]
      exact (data n).recurrenceScalar_abs_le hx (by omega)
    have hlimit := finite_weighted_inequalities_to_limit_lower_bound
      (κ := κ) (c := -C / (κ ^ 2 - κ))
      (M := core.bound * (2 + core.bound)) (m := m) hκ hm hfinite
    have hk0 : κ ≠ 0 := ne_of_gt (lt_trans zero_lt_one hκ)
    have hk1 : κ - 1 ≠ 0 := sub_ne_zero.mpr hκ.ne'
    have hden : κ ^ 2 - κ ≠ 0 := by
      rw [show κ ^ 2 - κ = κ * (κ - 1) by ring]
      exact mul_ne_zero hk0 hk1
    have halgebra :
        (-C / (κ ^ 2 - κ)) / (κ - 1) =
          -C / (κ * (κ - 1) ^ 2) := by
      field_simp [hk0, hk1, hden]
    have hlower :
        -C / (κ * (κ - 1) ^ 2) ≤ m 1 := by
      rw [← halgebra]
      exact hlimit
    have hb0_nonneg := (data 0).displacementSq_nonneg κ x
    have hb0_upper := (data 0).displacementSq_le hx (by rfl) hsingular'
    have hC_nonneg : 0 ≤ C := by
      exact hb0_nonneg.trans (by simpa [C, m] using hb0_upper)
    have hresult := LoristSchwenninger.scalar_endpoint_le_two
      (κ := κ) (b := C) (m := m 1) hC_nonneg
      (lt_trans zero_lt_one hκ) (sq_pos_of_pos (sub_pos.mpr hκ))
      hlower (by exact le_rfl)
    simpa [κ] using hresult

end Harp
end CrouzeixConjecture

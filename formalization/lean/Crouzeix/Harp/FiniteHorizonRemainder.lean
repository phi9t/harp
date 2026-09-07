import Crouzeix.Harp.FiniteHorizonRecurrence

/-! Scalar finite-horizon remainder estimates. A single estimate uses only its
terminal value and weighted inequality. The horizon existence result is
qualitative and does not specify a computable choice of horizon. -/

noncomputable section

open Filter Finset Topology

namespace CrouzeixConjecture.Harp

/-- Exact sum of the weights with source indices `1, …, N`, including `N = 0`. -/
theorem inverse_power_weight_sum_exact {κ : ℝ} (hκ : 1 < κ) (N : ℕ) :
    (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) =
      (1 - (κ⁻¹) ^ N) / (κ - 1) := by
  have hk0 : κ ≠ 0 := ne_of_gt (lt_trans zero_lt_one hκ)
  have hk1 : κ - 1 ≠ 0 := ne_of_gt (sub_pos.mpr hκ)
  induction N with
  | zero => simp
  | succ N ih =>
    rw [Finset.sum_range_succ, ih]
    simp only [pow_succ]
    field_simp
    ring

/-- The one terminal bound is sufficient to control the weighted terminal term. -/
theorem finite_terminal_abs_le {κ M : ℝ} {m : ℕ → ℝ} {N : ℕ}
    (hκ : 1 < κ) (hterminal : |m (N + 1)| ≤ M) :
    |(κ⁻¹) ^ N * m (N + 1)| ≤ M * (κ⁻¹) ^ N := by
  have hpow : 0 ≤ (κ⁻¹) ^ N :=
    pow_nonneg (inv_nonneg.mpr (le_trans zero_le_one hκ.le)) N
  rw [abs_mul, abs_of_nonneg hpow, mul_comm M]
  exact mul_le_mul_of_nonneg_left hterminal hpow

/-- The signed remainder needs no sign assumption on `c` or its coefficient. -/
theorem finite_lower_bound_with_remainder
    {κ c M : ℝ} {m : ℕ → ℝ} {N : ℕ}
    (hκ : 1 < κ) (hterminal : |m (N + 1)| ≤ M)
    (hfinite : (κ⁻¹) ^ N * m (N + 1) +
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1) :
    c / (κ - 1) - (κ⁻¹) ^ N * (M + c / (κ - 1)) ≤ m 1 := by
  have hterminalLower := (abs_le.mp (finite_terminal_abs_le hκ hterminal)).1
  rw [inverse_power_weight_sum_exact hκ] at hfinite
  have halgebra : (1 - (κ⁻¹) ^ N) / (κ - 1) * c =
      c / (κ - 1) - (κ⁻¹) ^ N * (c / (κ - 1)) := by ring
  rw [halgebra] at hfinite
  nlinarith

/-- Replacing `c` by `|c|` in the remainder gives a nonnegative error budget. -/
theorem finite_lower_bound_with_abs_remainder
    {κ c M : ℝ} {m : ℕ → ℝ} {N : ℕ}
    (hκ : 1 < κ) (_hM : 0 ≤ M) (hterminal : |m (N + 1)| ≤ M)
    (hfinite : (κ⁻¹) ^ N * m (N + 1) +
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1) :
    c / (κ - 1) - (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ m 1 := by
  have hsigned := finite_lower_bound_with_remainder hκ hterminal hfinite
  have hc := div_le_div_of_nonneg_right (le_abs_self c) (sub_pos.mpr hκ).le
  have hscaled := mul_le_mul_of_nonneg_left hc
    (pow_nonneg (inv_nonneg.mpr (le_trans zero_le_one hκ.le)) N)
  nlinarith

/-- Any certified upper bound on the nonnegative remainder is an error budget. -/
theorem finite_lower_bound_of_error_budget
    {κ c M ε : ℝ} {m : ℕ → ℝ} {N : ℕ}
    (hκ : 1 < κ) (hM : 0 ≤ M) (hterminal : |m (N + 1)| ≤ M)
    (hfinite : (κ⁻¹) ^ N * m (N + 1) +
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1)
    (_hε : 0 ≤ ε) (hbudget : (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ ε) :
    c / (κ - 1) - ε ≤ m 1 := by
  linarith [finite_lower_bound_with_abs_remainder hκ hM hterminal hfinite]

/-- Every positive tolerance eventually bounds the remainder uniformly in `N`.
This is existence, not an algorithm for selecting a horizon. -/
theorem exists_uniform_remainder_horizon {κ c M ε : ℝ}
    (hκ : 1 < κ) (hM : 0 ≤ M) (hε : 0 < ε) :
    ∃ N₀ : ℕ, ∀ N ≥ N₀, (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ ε := by
  let D := M + |c| / (κ - 1)
  have hD : 0 ≤ D := add_nonneg hM (div_nonneg (abs_nonneg c) (sub_pos.mpr hκ).le)
  by_cases hzero : D = 0
  · refine ⟨0, fun N _ => ?_⟩
    change (κ⁻¹) ^ N * D ≤ ε
    simpa [hzero] using hε.le
  · have hDpos : 0 < D := lt_of_le_of_ne hD (Ne.symm hzero)
    have hpow := tendsto_pow_atTop_nhds_zero_of_lt_one
      (inv_nonneg.mpr (le_trans zero_le_one hκ.le)) (inv_lt_one_of_one_lt₀ hκ)
    have hevent : ∀ᶠ N : ℕ in atTop, (κ⁻¹) ^ N < ε / D :=
      hpow.eventually (gt_mem_nhds (div_pos hε hDpos))
    obtain ⟨N₀, hN₀⟩ := eventually_atTop.mp hevent
    exact ⟨N₀, fun N hN => ((lt_div_iff₀ hDpos).mp (hN₀ N hN)).le⟩

/-- The finite remainder recovers the existing limit interface with its original
uniform bound. Nonnegativity of `M` follows from that bound. -/
theorem finite_weighted_inequalities_to_limit_lower_bound_via_remainder
    {κ c M : ℝ} {m : ℕ → ℝ}
    (hκ : 1 < κ) (hm : ∀ n, |m n| ≤ M)
    (hfinite : ∀ N, (κ⁻¹) ^ N * m (N + 1) +
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1) :
    c / (κ - 1) ≤ m 1 := by
  have hM : 0 ≤ M := le_trans (abs_nonneg (m 0)) (hm 0)
  by_contra h
  have hgap : 0 < (c / (κ - 1) - m 1) / 2 := by linarith
  obtain ⟨N, hN⟩ := exists_uniform_remainder_horizon (c := c) hκ hM hgap
  have hlower := finite_lower_bound_of_error_budget hκ hM (hm (N + 1))
    (hfinite N) hgap.le (hN N le_rfl)
  linarith

end CrouzeixConjecture.Harp

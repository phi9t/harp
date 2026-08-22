import Mathlib.Analysis.SpecificLimits.Basic
import Mathlib.Tactic.FieldSimp
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

/-!
Pure scalar iteration for the Lorist--Schwenninger recurrence.

The operator argument supplies real sequences `m` and `r` satisfying

```
κ * m n - m (n + 1) ≥ r n
```

for source indices `n ≥ 1`.  This module starts at `m 1`, performs the
finite weighted iteration, and isolates the bounded-terminal limit argument.
-/

noncomputable section

open Filter Finset Topology

namespace CrouzeixConjecture
namespace LoristSchwenninger

/-- A single recurrence inequality, divided by the positive scalar `κ`. -/
theorem recurrence_step_lower_bound
    {κ : ℝ} {m r : ℕ → ℝ} {n : ℕ}
    (hκ : 0 < κ)
    (hrec : r n ≤ κ * m n - m (n + 1)) :
    κ⁻¹ * m (n + 1) + κ⁻¹ * r n ≤ m n := by
  have hsum : m (n + 1) + r n ≤ κ * m n := by
    linarith
  calc
    κ⁻¹ * m (n + 1) + κ⁻¹ * r n = κ⁻¹ * (m (n + 1) + r n) := by ring
    _ ≤ κ⁻¹ * (κ * m n) :=
      mul_le_mul_of_nonneg_left hsum (inv_nonneg.mpr hκ.le)
    _ = m n := by field_simp [ne_of_gt hκ]

/-- Iterating the source recurrence from index `1` through index `N` gives
the finite weighted lower bound on `m 1`.  The `Finset.range N` summand with
index `i` represents the source term with index `i + 1`. -/
theorem recurrence_finite_iteration
    {κ : ℝ} {m r : ℕ → ℝ}
    (hκ : 0 < κ)
    (hrec : ∀ n, 1 ≤ n → r n ≤ κ * m n - m (n + 1))
    (N : ℕ) :
    (κ⁻¹) ^ N * m (N + 1) +
        ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) * r (i + 1) ≤
      m 1 := by
  have hq0 : 0 ≤ κ⁻¹ := inv_nonneg.mpr hκ.le
  induction N with
  | zero => simp
  | succ N ih =>
      have hstep := recurrence_step_lower_bound hκ
        (hrec (N + 1) (Nat.one_le_iff_ne_zero.mpr (Nat.succ_ne_zero N)))
      have hscaled :=
        mul_le_mul_of_nonneg_left hstep (pow_nonneg hq0 N)
      calc
        (κ⁻¹) ^ N.succ * m (N.succ + 1) +
              ∑ i ∈ Finset.range N.succ, (κ⁻¹) ^ (i + 1) * r (i + 1) =
            (κ⁻¹) ^ N *
                (κ⁻¹ * m ((N + 1) + 1) + κ⁻¹ * r (N + 1)) +
              ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) * r (i + 1) := by
          simp only [Nat.succ_eq_add_one, Finset.sum_range_succ, pow_succ]
          ring
        _ ≤ (κ⁻¹) ^ N * m (N + 1) +
              ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) * r (i + 1) :=
          add_le_add hscaled le_rfl
        _ ≤ m 1 := ih

/-- If every recurrence error is bounded below by the same scalar `c`, the
finite iteration is bounded below by the corresponding geometric sum. -/
theorem recurrence_finite_iteration_of_constant_error
    {κ c : ℝ} {m r : ℕ → ℝ}
    (hκ : 0 < κ)
    (hrec : ∀ n, 1 ≤ n → r n ≤ κ * m n - m (n + 1))
    (herror : ∀ n, 1 ≤ n → c ≤ r n)
    (N : ℕ) :
    (κ⁻¹) ^ N * m (N + 1) +
        (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤
      m 1 := by
  have hq0 : 0 ≤ κ⁻¹ := inv_nonneg.mpr hκ.le
  have hweighted :
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) * c) ≤
        ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) * r (i + 1) := by
    refine Finset.sum_le_sum fun i hi => ?_
    exact mul_le_mul_of_nonneg_left
      (herror (i + 1) (Nat.one_le_iff_ne_zero.mpr (Nat.succ_ne_zero i)))
      (pow_nonneg hq0 (i + 1))
  calc
    (κ⁻¹) ^ N * m (N + 1) +
          (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c =
        (κ⁻¹) ^ N * m (N + 1) +
          ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) * c := by
      rw [Finset.sum_mul]
    _ ≤ (κ⁻¹) ^ N * m (N + 1) +
          ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) * r (i + 1) :=
      add_le_add le_rfl hweighted
    _ ≤ m 1 := recurrence_finite_iteration hκ hrec N

/-- A uniformly bounded terminal sequence is killed by the inverse powers of
`κ` when `κ > 1`. -/
theorem bounded_recurrence_terminal_tendsto_zero
    {κ M : ℝ} {m : ℕ → ℝ}
    (hκ : 1 < κ)
    (hm : ∀ n, |m n| ≤ M) :
    Tendsto (fun N : ℕ => (κ⁻¹) ^ N * m (N + 1)) atTop (𝓝 0) := by
  have hq0 : 0 ≤ κ⁻¹ := inv_nonneg.mpr (le_trans zero_le_one hκ.le)
  have hq1 : κ⁻¹ < 1 := inv_lt_one_of_one_lt₀ hκ
  have hpow : Tendsto (fun N : ℕ => (κ⁻¹) ^ N) atTop (𝓝 0) :=
    tendsto_pow_atTop_nhds_zero_of_lt_one hq0 hq1
  rw [tendsto_zero_iff_abs_tendsto_zero]
  have hbound_tendsto :
      Tendsto (fun N : ℕ => (κ⁻¹) ^ N * M) atTop (𝓝 0) := by
    simpa using hpow.mul_const M
  refine squeeze_zero (fun N => abs_nonneg _) (fun N => ?_) hbound_tendsto
  calc
    |(κ⁻¹) ^ N * m (N + 1)| = (κ⁻¹) ^ N * |m (N + 1)| := by
      rw [abs_mul, abs_of_nonneg (pow_nonneg hq0 N)]
    _ ≤ (κ⁻¹) ^ N * M :=
      mul_le_mul_of_nonneg_left (hm (N + 1)) (pow_nonneg hq0 N)

/-- The source weights `κ⁻¹, ..., κ⁻ᴺ` converge to `1 / (κ - 1)`. -/
theorem inverse_power_weight_sum_tendsto
    {κ : ℝ} (hκ : 1 < κ) :
    Tendsto
      (fun N : ℕ => ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1))
      atTop (𝓝 ((κ - 1)⁻¹)) := by
  have hq0 : 0 ≤ κ⁻¹ := inv_nonneg.mpr (le_trans zero_le_one hκ.le)
  have hq1 : κ⁻¹ < 1 := inv_lt_one_of_one_lt₀ hκ
  have hseries :
      HasSum (fun i : ℕ => (κ⁻¹) ^ (i + 1))
        (κ⁻¹ * (1 - κ⁻¹)⁻¹) := by
    simpa [pow_succ'] using
      (hasSum_geometric_of_lt_one hq0 hq1).mul_left κ⁻¹
  have hvalue : κ⁻¹ * (1 - κ⁻¹)⁻¹ = (κ - 1)⁻¹ := by
    field_simp [ne_of_gt (lt_trans zero_lt_one hκ), sub_ne_zero.mpr hκ.ne]
  rw [← hvalue]
  exact hseries.tendsto_sum_nat

/-- Passing the finite constant-error bound to the limit gives the reusable
scalar recurrence endpoint `c / (κ - 1) ≤ m 1`. -/
theorem recurrence_limit_lower_bound_of_constant_error
    {κ c M : ℝ} {m r : ℕ → ℝ}
    (hκ : 1 < κ)
    (hm : ∀ n, |m n| ≤ M)
    (hrec : ∀ n, 1 ≤ n → r n ≤ κ * m n - m (n + 1))
    (herror : ∀ n, 1 ≤ n → c ≤ r n) :
    c / (κ - 1) ≤ m 1 := by
  have hterminal := bounded_recurrence_terminal_tendsto_zero hκ hm
  have hweights := inverse_power_weight_sum_tendsto hκ
  have hleft :
      Tendsto
        (fun N : ℕ =>
          (κ⁻¹) ^ N * m (N + 1) +
            (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c)
        atTop (𝓝 ((κ - 1)⁻¹ * c)) := by
    simpa using hterminal.add (hweights.mul_const c)
  have hlimit : (κ - 1)⁻¹ * c ≤ m 1 :=
    le_of_tendsto' hleft
      (recurrence_finite_iteration_of_constant_error
        (lt_trans zero_lt_one hκ) hrec herror)
  simpa [div_eq_mul_inv, mul_comm] using hlimit

/-- The constant error in the Lorist--Schwenninger proof yields its exact
source lower bound for `m 1`.  Nonnegativity of `b` is not needed for this
pure recurrence implication. -/
theorem recurrence_lower_bound
    {κ b M : ℝ} {m r : ℕ → ℝ}
    (hκ : 1 < κ)
    (hm : ∀ n, |m n| ≤ M)
    (hrec : ∀ n, 1 ≤ n → r n ≤ κ * m n - m (n + 1))
    (herror : ∀ n, 1 ≤ n → -b / (κ ^ 2 - κ) ≤ r n) :
    -b / (κ * (κ - 1) ^ 2) ≤ m 1 := by
  have hbound := recurrence_limit_lower_bound_of_constant_error
    hκ hm hrec herror
  have hk0 : κ ≠ 0 := ne_of_gt (lt_trans zero_lt_one hκ)
  have hk1 : κ - 1 ≠ 0 := sub_ne_zero.mpr hκ.ne'
  have hden : κ ^ 2 - κ ≠ 0 := by
    rw [show κ ^ 2 - κ = κ * (κ - 1) by ring]
    exact mul_ne_zero hk0 hk1
  have halgebra :
      (-b / (κ ^ 2 - κ)) / (κ - 1) =
        -b / (κ * (κ - 1) ^ 2) := by
    field_simp [hk0, hk1, hden]
  rw [← halgebra]
  exact hbound

end LoristSchwenninger
end CrouzeixConjecture

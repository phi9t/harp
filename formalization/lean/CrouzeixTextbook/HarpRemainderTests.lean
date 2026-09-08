import Crouzeix.Harp.FiniteHorizonRemainder

open CrouzeixConjecture.Harp

-- Public clients: these declarations must exist before the fixtures can compile.
#check inverse_power_weight_sum_exact
#check finite_terminal_abs_le
#check finite_lower_bound_with_remainder
#check finite_lower_bound_with_abs_remainder
#check finite_lower_bound_of_error_budget
#check exists_uniform_remainder_horizon

noncomputable section
namespace CrouzeixTextbook.HarpRemainderTests

theorem weight_empty {κ : ℝ} (hκ : 1 < κ) :
    (∑ i ∈ Finset.range 0, (κ⁻¹) ^ (i + 1)) = 0 := by
  calc
    _ = (1 - (κ⁻¹) ^ 0) / (κ - 1) := inverse_power_weight_sum_exact hκ 0
    _ = 0 := by simp

theorem weight_one : (∑ i ∈ Finset.range 1, ((2 : ℝ)⁻¹) ^ (i + 1)) = 1 / 2 := by
  rw [inverse_power_weight_sum_exact (by norm_num : (1 : ℝ) < 2)]
  norm_num

theorem weight_two : (∑ i ∈ Finset.range 2, ((2 : ℝ)⁻¹) ^ (i + 1)) = 3 / 4 := by
  rw [inverse_power_weight_sum_exact (by norm_num : (1 : ℝ) < 2)]
  norm_num

/-- At horizon zero the finite premise is reflexive; only the terminal bound matters. -/
theorem horizon_zero {κ c M : ℝ} {m : ℕ → ℝ}
    (hκ : 1 < κ) (hm : |m 1| ≤ M) :
    c / (κ - 1) - (M + c / (κ - 1)) ≤ m 1 := by
  simpa using finite_lower_bound_with_remainder (c := c) (N := 0) hκ hm (by simp)

theorem positive_c_one : (1 / 2 : ℝ) ≤ 1 / 2 := by
  have h := finite_lower_bound_with_abs_remainder (κ := 2) (c := 1) (M := 0)
    (m := fun n => if n = 1 then 1 / 2 else 0) (N := 1)
    (by norm_num) (by norm_num) (by norm_num) (by norm_num)
  convert h using 1 <;> norm_num

/-- The tighter coefficient is negative here; no nonnegative-coefficient premise is hidden. -/
theorem negative_coefficient_one : (-5 / 2 : ℝ) ≤ -5 / 2 := by
  have h := finite_lower_bound_with_remainder (κ := 2) (c := -5) (M := 0)
    (m := fun n => if n = 1 then -5 / 2 else 0) (N := 1)
    (by norm_num) (by norm_num) (by norm_num)
  convert h using 1 <;> norm_num

theorem zero_c_zero_M (N : ℕ) : (0 : ℝ) ≤ 0 := by
  have h := finite_lower_bound_with_abs_remainder (κ := 2) (c := 0) (M := 0)
    (m := fun _ => 0) (N := N) (by norm_num) (by norm_num) (by simp) (by simp)
  convert h using 1
  simp

theorem zero_error_budget {m : ℕ → ℝ} {N : ℕ}
    (ht : |m (N + 1)| ≤ 0)
    (hf : ((2 : ℝ)⁻¹) ^ N * m (N + 1) +
      (∑ i ∈ Finset.range N, ((2 : ℝ)⁻¹) ^ (i + 1)) * 0 ≤ m 1) :
    0 ≤ m 1 := by
  have h := finite_lower_bound_of_error_budget (by norm_num : (1 : ℝ) < 2)
    (by norm_num : (0 : ℝ) ≤ 0) ht hf (by norm_num : (0 : ℝ) ≤ 0) (by simp)
  simpa using h

theorem zero_budget_horizon {κ ε : ℝ} (hκ : 1 < κ) (hε : 0 < ε) :
    ∃ N₀ : ℕ, ∀ N ≥ N₀, (κ⁻¹) ^ N * (0 + |(0 : ℝ)| / (κ - 1)) ≤ ε :=
  exists_uniform_remainder_horizon hκ (by norm_num) hε

theorem positive_budget_horizon :
    ∃ N₀ : ℕ, ∀ N ≥ N₀, ((2 : ℝ)⁻¹) ^ N * (3 + |(-1 : ℝ)| / (2 - 1)) ≤ 1 / 8 :=
  exists_uniform_remainder_horizon (by norm_num) (by norm_num) (by norm_num)

def negativeExample (n : ℕ) : ℝ := if n = 1 then -3 / 2 else -3

theorem negative_example_terminal :
    |((2 : ℝ)⁻¹) ^ 2 * negativeExample (2 + 1)| ≤ 3 * ((2 : ℝ)⁻¹) ^ 2 :=
  finite_terminal_abs_le (by norm_num) (by norm_num [negativeExample])

theorem negative_example_finite :
    ((2 : ℝ)⁻¹) ^ 2 * negativeExample (2 + 1) +
      (∑ i ∈ Finset.range 2, ((2 : ℝ)⁻¹) ^ (i + 1)) * (-1) ≤ negativeExample 1 := by
  norm_num [negativeExample, Finset.sum_range_succ]

theorem negative_example_values :
    ((2 : ℝ)⁻¹) ^ 2 * (3 + |(-1 : ℝ)| / (2 - 1)) = 1 ∧
    (-1 : ℝ) / (2 - 1) - (2⁻¹ : ℝ) ^ 2 * (3 + (-1 : ℝ) / (2 - 1)) = -3 / 2 ∧
    (-1 : ℝ) / (2 - 1) - (2⁻¹ : ℝ) ^ 2 * (3 + |(-1 : ℝ)| / (2 - 1)) = -2 := by
  norm_num

theorem negative_example_tight : (-3 / 2 : ℝ) ≤ negativeExample 1 := by
  have h := finite_lower_bound_with_remainder (by norm_num : (1 : ℝ) < 2)
    (by norm_num [negativeExample] : |negativeExample (2 + 1)| ≤ 3)
    negative_example_finite
  norm_num at h
  simpa only [neg_div] using h

theorem negative_example_relaxed : (-2 : ℝ) ≤ negativeExample 1 := by
  have h := finite_lower_bound_with_abs_remainder (by norm_num : (1 : ℝ) < 2)
    (by norm_num : (0 : ℝ) ≤ 3)
    (by norm_num [negativeExample] : |negativeExample (2 + 1)| ≤ 3)
    negative_example_finite
  norm_num at h
  exact h

theorem negative_example_error_budget : (-2 : ℝ) ≤ negativeExample 1 := by
  have h := finite_lower_bound_of_error_budget (by norm_num : (1 : ℝ) < 2)
    (by norm_num : (0 : ℝ) ≤ 3)
    (by norm_num [negativeExample] : |negativeExample (2 + 1)| ≤ 3)
    negative_example_finite (by norm_num : (0 : ℝ) ≤ 1) (by norm_num)
  norm_num at h
  exact h

/-- Well-typed proposition variants isolate each omitted premise. -/
def missingKappaPremise : Prop :=
  ∀ (κ c M : ℝ) (m : ℕ → ℝ) (N : ℕ), 0 ≤ M → |m (N + 1)| ≤ M →
    (κ⁻¹) ^ N * m (N + 1) + (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1 →
    c / (κ - 1) - (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ m 1

def missingTerminalPremise : Prop :=
  ∀ (κ c M : ℝ) (m : ℕ → ℝ) (N : ℕ), 1 < κ → 0 ≤ M →
    (κ⁻¹) ^ N * m (N + 1) + (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1 →
    c / (κ - 1) - (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ m 1

theorem exact_contract_positive_control :
    ∀ (κ c M : ℝ) (m : ℕ → ℝ) (N : ℕ), 1 < κ → 0 ≤ M → |m (N + 1)| ≤ M →
      (κ⁻¹) ^ N * m (N + 1) + (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1 →
      c / (κ - 1) - (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ m 1 :=
  @finite_lower_bound_with_abs_remainder

theorem rejects_missing_kappa : True := by
  have _ := exact_contract_positive_control
  fail_if_success have _ : missingKappaPremise := @finite_lower_bound_with_abs_remainder
  trivial

theorem rejects_missing_terminal : True := by
  have _ := exact_contract_positive_control
  fail_if_success have _ : missingTerminalPremise := @finite_lower_bound_with_abs_remainder
  trivial

/-- With κ=2, c=M=0, m₁=-1 and m₂=0, the terminal bound holds but the
N=1 lower bound is false. The missing finite inequality would require 0 ≤ -1. -/
theorem missing_finite_counterexample :
    let m : ℕ → ℝ := fun n => if n = 1 then -1 else 0
    (1 : ℝ) < 2 ∧ (0 : ℝ) ≤ 0 ∧ |m (1 + 1)| ≤ 0 ∧
      ¬ (((2 : ℝ)⁻¹) ^ 1 * m (1 + 1) +
        (∑ i ∈ Finset.range 1, ((2 : ℝ)⁻¹) ^ (i + 1)) * 0 ≤ m 1) ∧
      ¬ ((0 : ℝ) / (2 - 1) - (2⁻¹ : ℝ) ^ 1 * (0 + |(0 : ℝ)| / (2 - 1)) ≤ m 1) := by
  norm_num

theorem comparison_old_interface {κ c M : ℝ} {m : ℕ → ℝ}
    (hκ : 1 < κ) (hm : ∀ n, |m n| ≤ M)
    (hf : ∀ N, (κ⁻¹) ^ N * m (N + 1) +
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1) :
    c / (κ - 1) ≤ m 1 :=
  finite_weighted_inequalities_to_limit_lower_bound_via_remainder hκ hm hf

#print axioms inverse_power_weight_sum_exact
#print axioms finite_terminal_abs_le
#print axioms finite_lower_bound_with_remainder
#print axioms finite_lower_bound_with_abs_remainder
#print axioms finite_lower_bound_of_error_budget
#print axioms exists_uniform_remainder_horizon
#print axioms finite_weighted_inequalities_to_limit_lower_bound_via_remainder
#print axioms weight_empty
#print axioms weight_one
#print axioms weight_two
#print axioms horizon_zero
#print axioms positive_c_one
#print axioms negative_coefficient_one
#print axioms zero_c_zero_M
#print axioms zero_error_budget
#print axioms zero_budget_horizon
#print axioms positive_budget_horizon
#print axioms negative_example_terminal
#print axioms negative_example_finite
#print axioms negative_example_values
#print axioms negative_example_tight
#print axioms negative_example_relaxed
#print axioms negative_example_error_budget
#print axioms exact_contract_positive_control
#print axioms rejects_missing_kappa
#print axioms rejects_missing_terminal
#print axioms missing_finite_counterexample
#print axioms comparison_old_interface

end CrouzeixTextbook.HarpRemainderTests

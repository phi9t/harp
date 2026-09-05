import Crouzeix.LoristSchwenninger.PerturbationLemma

namespace CrouzeixTextbook.Part06
open CrouzeixConjecture
open scoped InnerProduct

noncomputable section

section RecurrenceWorkshop

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

omit [Nontrivial E] in
/-- CFT-33-001: the exact denominator-free adjacent-power recurrence. -/
theorem recurrence_difference_identity
    (data : LoristSchwenninger.DilationData (E := E) (K := K))
    {κ : ℝ} (n : ℕ) (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint data.T (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    κ * LoristSchwenninger.recurrenceScalar data x n -
        LoristSchwenninger.recurrenceScalar data x (n + 1) =
      (κ ^ 2 - κ) * ‖(ContinuousLinearMap.adjoint data.T ^ n) x‖ ^ 2 -
        (inner ℂ (data.adjacentPowerDefect κ n x)
          ((ContinuousLinearMap.adjoint data.T ^ n) x)).re := by
  exact data.recurrence_difference_identity n x hsingular

/-- CFT-33-002: completing the square makes the recurrence uniform in `n`. -/
theorem recurrence_difference_lower_bound
    (data : LoristSchwenninger.DilationData (E := E) (K := K))
    {κ : ℝ} (hκ : 1 < κ) (n : ℕ) (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint data.T (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    -‖data.displacement κ x‖ ^ 2 / (κ ^ 2 - κ) ≤
      κ * LoristSchwenninger.recurrenceScalar data x n -
        LoristSchwenninger.recurrenceScalar data x (n + 1) := by
  exact data.recurrence_difference_lower_bound hκ n x hsingular

end RecurrenceWorkshop

set_option linter.defProp false in
def equation_three_lower_bound := @LoristSchwenninger.DilationData.equation_three_lower_bound

/-- CFT-33-004: source Equations (3) and (4) combine into one scalar inequality. -/
theorem scalar_combined_inequality
    {κ b m : ℝ}
    (hkpos : 0 < κ)
    (hkminus1sqpos : 0 < (κ - 1) ^ 2)
    (hlower : m ≥ -b / (κ * (κ - 1) ^ 2))
    (hupper : b ≤ 2 * κ ^ 2 - κ * m - κ ^ 3) :
    b * (1 - 1 / (κ - 1) ^ 2) ≤ 2 * κ ^ 2 - κ ^ 3 := by
  exact LoristSchwenninger.scalar_combined_inequality_of_recurrence_bounds
    hkpos hkminus1sqpos hlower hupper

/-- CFT-33-005: nonnegativity closes the real scalar contradiction at two. -/
theorem scalar_endpoint_two
    {κ b m : ℝ}
    (hb : 0 ≤ b)
    (hkpos : 0 < κ)
    (hkminus1sqpos : 0 < (κ - 1) ^ 2)
    (hlower : m ≥ -b / (κ * (κ - 1) ^ 2))
    (hupper : b ≤ 2 * κ ^ 2 - κ * m - κ ^ 3) :
    κ ≤ 2 := by
  exact LoristSchwenninger.scalar_endpoint_le_two
    hb hkpos hkminus1sqpos hlower hupper

set_option linter.defProp false in
def perturbation_lemma := @LoristSchwenninger.DilationData.norm_target_le_two

namespace Exercises.Chapter33

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

omit [Nontrivial E] in
/-- Starter idea: expose the successor power, then rewrite by `hsingular`. -/
theorem exercise_01_solution
    (data : LoristSchwenninger.DilationData (E := E) (K := K))
    {κ : ℝ} (n : ℕ) (x : E)
    (hsingular :
      ContinuousLinearMap.adjoint data.T (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    (ContinuousLinearMap.adjoint data.T ^ (n + 1)) (data.T x) =
      ((κ ^ 2 : ℝ) : ℂ) •
        ((ContinuousLinearMap.adjoint data.T ^ n) x) := by
  rw [pow_succ, mul_apply_eq_comp, hsingular, map_smul]

omit [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E] in
/-- Starter idea: first prove `0 < a`; the discarded term is a norm square. -/
theorem exercise_02_solution
    {a : ℝ} (ha : 0 < a) (u z : E) :
    -‖z‖ ^ 2 / (4 * a) ≤
      a * ‖u‖ ^ 2 - (inner ℂ z u).re := by
  exact LoristSchwenninger.pre_square_lower_bound ha u z rfl

omit [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E] in
/-- Starter idea: induct on `N`; expose `Finset.sum_range_succ` before
normalizing the two endpoint terms. -/
theorem exercise_03_solution
    (κ : ℝ) (m : ℕ → ℝ) (N : ℕ) :
    (∑ i ∈ Finset.range N,
        ((κ⁻¹) ^ (i + 1) * m (i + 1) -
          (κ⁻¹) ^ (i + 2) * m (i + 2))) =
      κ⁻¹ * m 1 - (κ⁻¹) ^ (N + 1) * m (N + 1) := by
  induction N with
  | zero => simp
  | succ N ih =>
      rw [Finset.sum_range_succ, ih]
      ring

/-- Starter idea: separate the three positivity facts before proving that their
product cannot vanish. -/
theorem exercise_04_solution
    {κ : ℝ} (hκ : 1 < κ) :
    0 < κ ∧ 0 < (κ - 1) ^ 2 ∧ κ * (κ - 1) ^ 2 ≠ 0 := by
  have hkpos : 0 < κ := lt_trans zero_lt_one hκ
  have hkminus1pos : 0 < κ - 1 := sub_pos.mpr hκ
  have hkminus1sqpos : 0 < (κ - 1) ^ 2 := sq_pos_of_pos hkminus1pos
  constructor
  · exact hkpos
  constructor
  · exact hkminus1sqpos
  · exact mul_ne_zero (ne_of_gt hkpos) (ne_of_gt hkminus1sqpos)

/-- Starter idea: assume `2 < κ`, prove the left side nonnegative and the
right side negative, then compare both signs through `hcombined`. -/
theorem exercise_05_solution
    {κ b : ℝ}
    (hb : 0 ≤ b)
    (hcombined : b * (1 - 1 / (κ - 1) ^ 2) ≤ 2 * κ ^ 2 - κ ^ 3) :
    κ ≤ 2 := by
  by_contra hnot
  have hk2 : 2 < κ := lt_of_not_ge hnot
  have hfactor_pos : 0 < 1 - 1 / (κ - 1) ^ 2 := by
    have hlt : 1 < (κ - 1) ^ 2 := by
      nlinarith [sq_pos_of_ne_zero (by linarith : κ - 2 ≠ 0)]
    have hden : 0 < (κ - 1) ^ 2 := by nlinarith
    rw [sub_pos, div_lt_one hden]
    exact hlt
  have hleft_nonneg : 0 ≤ b * (1 - 1 / (κ - 1) ^ 2) :=
    mul_nonneg hb (le_of_lt hfactor_pos)
  have hright_nonneg : 0 ≤ 2 * κ ^ 2 - κ ^ 3 :=
    hleft_nonneg.trans hcombined
  have hright_neg : 2 * κ ^ 2 - κ ^ 3 < 0 := by
    nlinarith [sq_pos_of_ne_zero (by linarith : κ ≠ 0)]
  linarith

/-- Starter idea: split on the named norm `κ`, obtain a top singular vector in
the large branch, derive Equations (3) and (4), and only then invoke the scalar
endpoint. -/
theorem exercise_06_solution
    (data : LoristSchwenninger.DilationData (E := E) (K := K))
    (κ : ℝ) (hκ : κ = ‖data.T‖) :
    ‖data.T‖ ≤ 2 := by
  by_cases hsmall : κ ≤ 1
  · calc
      ‖data.T‖ = κ := hκ.symm
      _ ≤ 2 := by linarith
  · have hnorm : 1 < κ := lt_of_not_ge hsmall
    obtain ⟨x, hx, _hTx, hsingularNorm⟩ :=
      LoristSchwenninger.exists_unit_norm_attaining_and_adjoint_apply data.T
    have hsingular :
        ContinuousLinearMap.adjoint data.T (data.T x) =
          ((κ ^ 2 : ℝ) : ℂ) • x := by
      simpa [hκ] using hsingularNorm
    let b : ℝ := data.displacementSq κ x
    let m : ℝ := data.firstPerturbationMoment x
    have hlower := data.equation_three_lower_bound hnorm hx hsingular
    have hlower' : -b / (κ * (κ - 1) ^ 2) ≤ m := by
      have hreal :
          (inner ℂ x (data.perturbation 1 (data.T x))).re =
            (inner ℂ (data.perturbation 1 (data.T x)) x).re :=
        @inner_re_symm ℂ E _ _ _ x (data.perturbation 1 (data.T x))
      simpa [b, m, LoristSchwenninger.DilationData.displacementSq,
        LoristSchwenninger.DilationData.firstPerturbationMoment] using
        hlower.trans_eq hreal
    have hupper : b ≤ 2 * κ ^ 2 - κ * m - κ ^ 3 := by
      simpa [b, m] using data.displacementSq_le hx hκ hsingular
    have hkpos : 0 < κ := lt_trans zero_lt_one hnorm
    have hkminus1sqpos : 0 < (κ - 1) ^ 2 :=
      sq_pos_of_pos (sub_pos.mpr hnorm)
    have hk : κ ≤ 2 := LoristSchwenninger.scalar_endpoint_le_two
      (by simpa [b] using data.displacementSq_nonneg κ x)
      hkpos hkminus1sqpos hlower' hupper
    simpa [hκ] using hk

end Exercises.Chapter33

end
end CrouzeixTextbook.Part06

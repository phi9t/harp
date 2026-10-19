import Crouzeix.LoristSchwenninger.Perturbation

/-!
Scalar endpoint for the Lorist-Schwenninger perturbation route.

The operator-theoretic part of the source proof reduces the case `κ > 1` to
real inequalities in two scalars:

* `b = ‖Q^* V T x - κ V x‖^2`, hence `0 ≤ b`;
* `m = Re ⟪E_1 T x, x⟫`;
* a recurrence lower bound on `m`;
* a Hilbert-space upper bound on `b`.

This module proves the final real-algebra step.  It does not prove the
operator recurrence or construct the dilation data; those remain separate
source-mapped LS obligations.
-/

noncomputable section

namespace CrouzeixConjecture
namespace LoristSchwenninger

/-- Combining the recurrence lower bound for `m` with the Hilbert-space upper
bound for `b` gives the displayed Lorist-Schwenninger contradiction inequality.
-/
theorem scalar_combined_inequality_of_recurrence_bounds
    {κ b m : ℝ}
    (hkpos : 0 < κ)
    (hkminus1sqpos : 0 < (κ - 1) ^ 2)
    (hlower : m ≥ -b / (κ * (κ - 1) ^ 2))
    (hupper : b ≤ 2 * κ ^ 2 - κ * m - κ ^ 3) :
    b * (1 - 1 / (κ - 1) ^ 2) ≤ 2 * κ ^ 2 - κ ^ 3 := by
  have hκminus1sq_ne : (κ - 1) ^ 2 ≠ 0 := ne_of_gt hkminus1sqpos
  have hκm_lower : -b / ((κ - 1) ^ 2) ≤ κ * m := by
    have hmul := mul_le_mul_of_nonneg_left hlower (le_of_lt hkpos)
    have hrewrite :
        κ * (-b / (κ * (κ - 1) ^ 2)) = -b / ((κ - 1) ^ 2) := by
      field_simp [ne_of_gt hkpos, hκminus1sq_ne]
    simpa [hrewrite] using hmul
  have hneg0 : -(κ * m) ≤ -(-b / ((κ - 1) ^ 2)) := neg_le_neg hκm_lower
  have hneg : -(κ * m) ≤ b / ((κ - 1) ^ 2) := by
    simpa [neg_div] using hneg0
  have hupper' : b ≤ 2 * κ ^ 2 - κ ^ 3 + b / ((κ - 1) ^ 2) := by
    nlinarith
  have hleft_le_right : b - b / ((κ - 1) ^ 2) ≤ 2 * κ ^ 2 - κ ^ 3 := by
    linarith
  have hrewrite : b * (1 - 1 / (κ - 1) ^ 2) = b - b / ((κ - 1) ^ 2) := by
    ring
  rw [hrewrite]
  exact hleft_le_right

/-- The terminal real-variable contradiction in the Lorist-Schwenninger
recurrence argument.  Once the source has supplied the combined inequality, the
case `κ > 2` is impossible. -/
theorem scalar_contradiction_le_two
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
    rw [sub_pos]
    rw [div_lt_one hden]
    exact hlt
  have hfactor_nonneg : 0 ≤ b * (1 - 1 / (κ - 1) ^ 2) :=
    mul_nonneg hb (le_of_lt hfactor_pos)
  have hright_nonneg : 0 ≤ 2 * κ ^ 2 - κ ^ 3 := hfactor_nonneg.trans hcombined
  have hright_neg : 2 * κ ^ 2 - κ ^ 3 < 0 := by
    nlinarith [sq_pos_of_ne_zero (by linarith : κ ≠ 0)]
  linarith

/-- Source-map endpoint for the scalar part of the LS route: the recurrence
lower bound and upper bound on the displacement scalar imply `κ ≤ 2`. -/
theorem scalar_endpoint_le_two
    {κ b m : ℝ}
    (hb : 0 ≤ b)
    (hkpos : 0 < κ)
    (hkminus1sqpos : 0 < (κ - 1) ^ 2)
    (hlower : m ≥ -b / (κ * (κ - 1) ^ 2))
    (hupper : b ≤ 2 * κ ^ 2 - κ * m - κ ^ 3) :
    κ ≤ 2 :=
  scalar_contradiction_le_two hb
    (scalar_combined_inequality_of_recurrence_bounds
      hkpos hkminus1sqpos hlower hupper)

end LoristSchwenninger
end CrouzeixConjecture

import Crouzeix.Harp.FiniteHorizonRemainder
import Crouzeix.Harp.FiniteHorizonOperatorRecurrence

/-! The scalar remainder applied to one finite-horizon dilation witness.
The perturbation core fixes the operator, scalar sequence, and uniform bound;
the norm and singular-vector hypotheses are supplied explicitly. -/

noncomputable section

namespace CrouzeixConjecture.Harp

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E] [CompleteSpace E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

/-- The common displacement envelope is nonnegative because it bounds the
squared displacement of this single witness. -/
theorem finite_horizon_displacement_envelope_nonneg
    {core : CommutingPerturbationData E} {N : ℕ}
    (data : FiniteHorizonDilationData core K N) {κ : ℝ} {x : E}
    (hx : ‖x‖ = 1) (hnorm : κ = ‖core.T‖)
    (hsingular : ContinuousLinearMap.adjoint core.T (core.T x) =
      ((κ ^ 2 : ℝ) : ℂ) • x) :
    0 ≤ 2 * κ ^ 2 - κ * core.recurrenceScalar x 1 - κ ^ 3 := by
  exact (data.displacementSq_nonneg κ x).trans
    (data.displacementSq_le hx hnorm hsingular)

/-- A single horizon witness gives the finite lower bound with its explicit
absolute remainder. No witnesses at other horizons or finite-dimensionality
assumption are needed when the unit singular vector is supplied. -/
theorem finite_horizon_scalar_lower_bound_with_remainder [Nontrivial E]
    {core : CommutingPerturbationData E} {N : ℕ}
    (data : FiniteHorizonDilationData core K N) {κ : ℝ} (hκ : 1 < κ)
    (hnorm : κ = ‖core.T‖) (x : E) (hx : ‖x‖ = 1)
    (hsingular : ContinuousLinearMap.adjoint core.T (core.T x) =
      ((κ ^ 2 : ℝ) : ℂ) • x) :
    let m := core.recurrenceScalar x
    let M := core.bound * (2 + core.bound)
    let C := 2 * κ ^ 2 - κ * m 1 - κ ^ 3
    let c := -C / (κ ^ 2 - κ)
    c / (κ - 1) - (κ⁻¹) ^ N * (M + |c| / (κ - 1)) ≤ m 1 := by
  let m := core.recurrenceScalar x
  let M := core.bound * (2 + core.bound)
  let C := 2 * κ ^ 2 - κ * m 1 - κ ^ 3
  let c := -C / (κ ^ 2 - κ)
  have hM : 0 ≤ M :=
    mul_nonneg core.bound_nonneg (add_nonneg (by norm_num) core.bound_nonneg)
  have hterminal : |m (N + 1)| ≤ M := data.recurrenceScalar_abs_le hx le_rfl
  have heq3 := data.equation_three_finite_lower_bound hκ x hsingular
  have hb_le_C : data.displacementSq κ x ≤ C :=
    data.displacementSq_le hx hnorm hsingular
  have hdenpos : 0 < κ ^ 2 - κ := by nlinarith
  have hcoeff : c ≤ -data.displacementSq κ x / (κ ^ 2 - κ) := by
    dsimp [c]
    exact div_le_div_of_nonneg_right (neg_le_neg hb_le_C) hdenpos.le
  have hweight : 0 ≤ ∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1) :=
    Finset.sum_nonneg fun i _ =>
      pow_nonneg (inv_nonneg.mpr (le_trans zero_le_one hκ.le)) _
  have hfinite : (κ⁻¹) ^ N * m (N + 1) +
      (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) * c ≤ m 1 := by
    calc
      _ ≤ (κ⁻¹) ^ N * m (N + 1) +
          (∑ i ∈ Finset.range N, (κ⁻¹) ^ (i + 1)) *
            (-data.displacementSq κ x / (κ ^ 2 - κ)) :=
        add_le_add le_rfl (mul_le_mul_of_nonneg_left hcoeff hweight)
      _ ≤ m 1 := heq3
  exact finite_lower_bound_with_abs_remainder hκ hM hterminal hfinite

end CrouzeixConjecture.Harp

import Crouzeix.LoristSchwenninger.Dilation
import Crouzeix.LoristSchwenninger.NormAttainment

/-!
The displacement upper bound in the Lorist--Schwenninger argument.

For a unit top right singular vector `x`, Equation (4) bounds the squared
displacement `‖Q† V (T x) - κ V x‖²` by the first perturbation moment.
-/

noncomputable section

open scoped InnerProduct

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

namespace DilationData

/-- The squared norm `b = ‖Q† V (T x) - κ V x‖²` from the source proof. -/
def displacementSq (data : DilationData (E := E) (K := K))
    (κ : ℝ) (x : E) : ℝ :=
  ‖data.displacement κ x‖ ^ 2

/-- The first perturbation moment
`m₁ = Re ⟪E₁ T x, x⟫`. -/
def firstPerturbationMoment
    (data : DilationData (E := E) (K := K)) (x : E) : ℝ :=
  (inner ℂ (data.perturbation 1 (data.T x)) x).re

omit [Nontrivial E] in
/-- The displacement scalar is nonnegative. -/
theorem displacementSq_nonneg
    (data : DilationData (E := E) (K := K)) (κ : ℝ) (x : E) :
    0 ≤ data.displacementSq κ x := by
  exact sq_nonneg _

omit [Nontrivial E] in
/-- Source Equation (4): the first perturbation identity and the contraction
estimate bound the displacement of a unit top right singular vector. -/
theorem displacementSq_le
    (data : DilationData (E := E) (K := K))
    {κ : ℝ} {x : E}
    (hx : ‖x‖ = 1)
    (hκ : κ = ‖data.T‖)
    (hsingular :
      ContinuousLinearMap.adjoint data.T (data.T x) =
        ((κ ^ 2 : ℝ) : ℂ) • x) :
    data.displacementSq κ x ≤
      2 * κ ^ 2 - κ * data.firstPerturbationMoment x - κ ^ 3 := by
  let A : K := ContinuousLinearMap.adjoint data.Q (data.V (data.T x))
  have hA_norm : ‖A‖ ≤ κ := by
    calc
      ‖A‖ ≤ ‖data.V (data.T x)‖ := data.Q_adjoint_apply_norm_le _
      _ = ‖data.T x‖ := data.V_apply_norm _
      _ ≤ ‖data.T‖ * ‖x‖ := data.T.le_opNorm x
      _ = κ := by rw [hx, mul_one, hκ]
  have hA_sq : ‖A‖ ^ 2 ≤ κ ^ 2 := by
    nlinarith [norm_nonneg A]
  have hE1 :
      data.perturbation 1 (data.T x) =
        (2 : ℂ) • ContinuousLinearMap.adjoint data.V A -
          ContinuousLinearMap.adjoint data.T (data.T x) := by
    have h := congrArg (fun S : E →L[ℂ] E => S (data.T x))
      (data.perturbation_eq 1)
    simpa [doubledCompressedAdjointPower, compressedAdjointPower, A] using h
  have hm :
      data.firstPerturbationMoment x =
        2 * (inner ℂ A (data.V x)).re - κ ^ 2 := by
    rw [firstPerturbationMoment, hE1, inner_sub_left, inner_smul_left,
      ContinuousLinearMap.adjoint_inner_left, hsingular, inner_smul_left,
      inner_self_eq_norm_sq_to_K, hx]
    norm_num [Complex.mul_re, pow_two]
  have hb :
      data.displacementSq κ x =
        ‖A‖ ^ 2 - 2 * κ * (inner ℂ A (data.V x)).re + κ ^ 2 := by
    rw [displacementSq, displacement, show
      ContinuousLinearMap.adjoint data.Q (data.V (data.T x)) = A by rfl,
      @norm_sub_sq ℂ, norm_smul, data.V_apply_norm, hx, mul_one,
      inner_smul_right]
    simp only [Complex.norm_real, Real.norm_eq_abs]
    rw [sq_abs]
    norm_num [Complex.mul_re]
    ring
  calc
    data.displacementSq κ x =
        ‖A‖ ^ 2 - 2 * κ * (inner ℂ A (data.V x)).re + κ ^ 2 := hb
    _ ≤ κ ^ 2 - 2 * κ * (inner ℂ A (data.V x)).re + κ ^ 2 := by
      linarith
    _ = 2 * κ ^ 2 - κ * data.firstPerturbationMoment x - κ ^ 3 := by
      rw [hm]
      ring

end DilationData
end LoristSchwenninger
end CrouzeixConjecture

import Crouzeix.LoristSchwenninger.Dilation

/-!
Finite-horizon dilation data for the Harp reconstruction of the
Lorist--Schwenninger argument.

The perturbation family is separated from the horizon-dependent dilation
witness.  A witness at horizon `N` is required to realize exactly the powers
through `N + 1`, which are the powers used by `N` adjacent recurrence steps.
-/

noncomputable section

open scoped InnerProduct

namespace CrouzeixConjecture
namespace Harp

/-- A uniformly bounded perturbation family commuting with its target. -/
structure CommutingPerturbationData (E : Type*)
    [NormedAddCommGroup E] [NormedSpace ℂ E] where
  T : E →L[ℂ] E
  perturbation : ℕ → E →L[ℂ] E
  bound : ℝ
  bound_nonneg : 0 ≤ bound
  perturbation_norm_le : ∀ n, ‖perturbation n‖ ≤ bound
  commutes_with_target : ∀ n, Commute (perturbation n) T

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

/-- A common isometric compression realizing the perturbation identities
exactly through power `N + 1`. -/
structure FiniteHorizonDilationData
    (core : CommutingPerturbationData E) (K : Type*) (N : ℕ)
    [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K] where
  V : E →L[ℂ] K
  Q : K →L[ℂ] K
  isometry : Isometry V
  contraction : ‖Q‖ ≤ 1
  perturbation_eq : ∀ k, k ≤ N + 1 →
    core.perturbation k =
      LoristSchwenninger.doubledCompressedAdjointPower V Q k -
        (ContinuousLinearMap.adjoint core.T) ^ k

namespace CommutingPerturbationData

/-- The real scalar sequence used in the recurrence, attached only to the
horizon-independent perturbation data. -/
def recurrenceScalar (core : CommutingPerturbationData E)
    (x : E) (n : ℕ) : ℝ :=
  Complex.re (inner ℂ x ((core.perturbation n * core.T ^ n) x))

end CommutingPerturbationData

namespace FiniteHorizonDilationData

omit [FiniteDimensional ℂ E] in
/-- On the realized horizon, a unit-vector recurrence scalar has the same
uniform absolute bound as in the unbounded dilation argument. -/
theorem recurrenceScalar_abs_le
    {core : CommutingPerturbationData E} {N : ℕ}
    (data : FiniteHorizonDilationData core K N) {x : E}
    (hx : ‖x‖ = 1) {n : ℕ} (hn : n ≤ N + 1) :
    |core.recurrenceScalar x n| ≤ core.bound * (2 + core.bound) := by
  have hV_apply (y : E) : ‖data.V y‖ = ‖y‖ :=
    data.isometry.norm_map_of_map_zero data.V.map_zero y
  have hV_norm : ‖data.V‖ = 1 := by
    let Vi : E →ₗᵢ[ℂ] K :=
      { data.V.toLinearMap with
        norm_map' := data.isometry.norm_map_of_map_zero data.V.map_zero }
    have hmap : Vi.toContinuousLinearMap = data.V := by
      ext y
      rfl
    rw [← hmap]
    exact Vi.norm_toContinuousLinearMap
  have hV_adjoint_norm : ‖ContinuousLinearMap.adjoint data.V‖ = 1 := by
    rw [(ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map, hV_norm]
  have hV_adjoint_apply (y : K) :
      ‖ContinuousLinearMap.adjoint data.V y‖ ≤ ‖y‖ := by
    calc
      ‖ContinuousLinearMap.adjoint data.V y‖
          ≤ ‖ContinuousLinearMap.adjoint data.V‖ * ‖y‖ :=
        (ContinuousLinearMap.adjoint data.V).le_opNorm y
      _ = ‖y‖ := by rw [hV_adjoint_norm, one_mul]
  have hQ_adjoint_norm : ‖ContinuousLinearMap.adjoint data.Q‖ ≤ 1 := by
    rw [(ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map]
    exact data.contraction
  have hQ_adjoint_power_apply (k : ℕ) (y : K) :
      ‖((ContinuousLinearMap.adjoint data.Q) ^ k) y‖ ≤ ‖y‖ := by
    induction k with
    | zero => simp
    | succ k ih =>
        rw [pow_succ', mul_apply_eq_comp]
        calc
          ‖ContinuousLinearMap.adjoint data.Q
              (((ContinuousLinearMap.adjoint data.Q) ^ k) y)‖
              ≤ ‖ContinuousLinearMap.adjoint data.Q‖ *
                  ‖((ContinuousLinearMap.adjoint data.Q) ^ k) y‖ :=
            (ContinuousLinearMap.adjoint data.Q).le_opNorm _
          _ ≤ 1 * ‖((ContinuousLinearMap.adjoint data.Q) ^ k) y‖ := by
            exact mul_le_mul_of_nonneg_right hQ_adjoint_norm (norm_nonneg _)
          _ ≤ ‖y‖ := by simpa using ih
  have hcompressed :
      ‖LoristSchwenninger.compressedAdjointPower data.V data.Q n‖ ≤ 1 := by
    refine ContinuousLinearMap.opNorm_le_bound _ zero_le_one fun y => ?_
    calc
      ‖LoristSchwenninger.compressedAdjointPower data.V data.Q n y‖
          ≤ ‖((ContinuousLinearMap.adjoint data.Q) ^ n) (data.V y)‖ :=
        hV_adjoint_apply _
      _ ≤ ‖data.V y‖ := hQ_adjoint_power_apply n _
      _ = ‖y‖ := hV_apply y
      _ = 1 * ‖y‖ := by rw [one_mul]
  have hdoubled :
      ‖LoristSchwenninger.doubledCompressedAdjointPower data.V data.Q n‖ ≤ 2 := by
    rw [LoristSchwenninger.doubledCompressedAdjointPower, norm_smul]
    have htwo : ‖(2 : ℂ)‖ = (2 : ℝ) := by norm_num
    rw [htwo]
    nlinarith [hcompressed,
      norm_nonneg (LoristSchwenninger.compressedAdjointPower data.V data.Q n)]
  have hadjoint_power :
      ‖(ContinuousLinearMap.adjoint core.T) ^ n‖ ≤ 2 + core.bound := by
    have hrewrite :
        (ContinuousLinearMap.adjoint core.T) ^ n =
          LoristSchwenninger.doubledCompressedAdjointPower data.V data.Q n -
            core.perturbation n := by
      rw [data.perturbation_eq n hn]
      abel
    rw [hrewrite]
    exact (norm_sub_le _ _).trans
      (add_le_add hdoubled (core.perturbation_norm_le n))
  have htarget_power : ‖core.T ^ n‖ ≤ 2 + core.bound := by
    rw [← (ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map (core.T ^ n),
      ← ContinuousLinearMap.star_eq_adjoint (core.T ^ n), star_pow,
      ContinuousLinearMap.star_eq_adjoint core.T]
    exact hadjoint_power
  have hproduct :
      ‖core.perturbation n * core.T ^ n‖ ≤
        core.bound * (2 + core.bound) := by
    exact (norm_mul_le _ _).trans
      (mul_le_mul (core.perturbation_norm_le n) htarget_power
        (norm_nonneg _) core.bound_nonneg)
  let A := core.perturbation n * core.T ^ n
  calc
    |core.recurrenceScalar x n| =
        |Complex.re (inner ℂ x (A x))| := rfl
    _ ≤ ‖inner ℂ x (A x)‖ := Complex.abs_re_le_norm _
    _ ≤ ‖x‖ * ‖A x‖ := norm_inner_le_norm _ _
    _ = ‖A x‖ := by rw [hx, one_mul]
    _ ≤ ‖A‖ * ‖x‖ := A.le_opNorm x
    _ = ‖A‖ := by rw [hx, mul_one]
    _ ≤ core.bound * (2 + core.bound) := hproduct

end FiniteHorizonDilationData
end Harp
end CrouzeixConjecture

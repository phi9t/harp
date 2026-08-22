import Mathlib.Analysis.InnerProductSpace.Adjoint

/-!
The concrete Hilbert-space dilation algebra used in the
Lorist--Schwenninger perturbation argument.

Unlike the matrix-level interface in `Perturbation.lean`, this file retains
the common dilation space `K`, the isometry `V`, and the contraction `Q`.
The perturbations are required to satisfy the source identity

```text
E n = 2 V† (Q†)^n V - (T†)^n
```

together with one uniform norm bound and commutation with `T`.
-/

noncomputable section

open scoped InnerProduct

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace ℂ E]
  [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace ℂ K] [CompleteSpace K]

/-- The compressed adjoint power `V† (Q†)^n V`. -/
def compressedAdjointPower (V : E →L[ℂ] K) (Q : K →L[ℂ] K)
    (n : ℕ) : E →L[ℂ] E :=
  (ContinuousLinearMap.adjoint V).comp
    (((ContinuousLinearMap.adjoint Q) ^ n).comp V)

/-- The source's operator `S_n† = 2 V† (Q†)^n V`. -/
def doubledCompressedAdjointPower (V : E →L[ℂ] K)
    (Q : K →L[ℂ] K) (n : ℕ) : E →L[ℂ] E :=
  (2 : ℂ) • compressedAdjointPower V Q n

/-- Source-faithful data for the Lorist--Schwenninger perturbation lemma. -/
structure DilationData [FiniteDimensional ℂ E] where
  T : E →L[ℂ] E
  V : E →L[ℂ] K
  Q : K →L[ℂ] K
  V_isometry : Isometry V
  Q_norm_le_one : ‖Q‖ ≤ 1
  perturbation : ℕ → E →L[ℂ] E
  perturbation_eq : ∀ n, perturbation n =
    doubledCompressedAdjointPower V Q n -
      (ContinuousLinearMap.adjoint T) ^ n
  bound : ℝ
  bound_nonneg : 0 ≤ bound
  perturbation_norm_le : ∀ n, ‖perturbation n‖ ≤ bound
  commutes_with_target : ∀ n, Commute (perturbation n) T

namespace DilationData

omit [Nontrivial E] in
/-- An isometric dilation map preserves vector norms. -/
theorem V_apply_norm (data : DilationData (E := E) (K := K)) (x : E) :
    ‖data.V x‖ = ‖x‖ := by
  exact data.V_isometry.norm_map_of_map_zero data.V.map_zero x

/-- The dilation map has operator norm one on a nontrivial domain. -/
theorem V_norm (data : DilationData (E := E) (K := K)) :
    ‖data.V‖ = 1 := by
  let Vi : E →ₗᵢ[ℂ] K :=
    { data.V.toLinearMap with
      norm_map' := data.V_isometry.norm_map_of_map_zero data.V.map_zero }
  have hmap : Vi.toContinuousLinearMap = data.V := by
    ext x
    rfl
  rw [← hmap]
  exact Vi.norm_toContinuousLinearMap

/-- Taking the Hilbert adjoint preserves the norm of the isometric dilation
map. -/
theorem V_adjoint_norm (data : DilationData (E := E) (K := K)) :
    ‖ContinuousLinearMap.adjoint data.V‖ = 1 := by
  rw [(ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map, data.V_norm]

/-- The adjoint `V†` is contractive on vectors. -/
theorem V_adjoint_apply_norm_le (data : DilationData (E := E) (K := K))
    (y : K) :
    ‖ContinuousLinearMap.adjoint data.V y‖ ≤ ‖y‖ := by
  calc
    ‖ContinuousLinearMap.adjoint data.V y‖
        ≤ ‖ContinuousLinearMap.adjoint data.V‖ * ‖y‖ :=
      (ContinuousLinearMap.adjoint data.V).le_opNorm y
    _ = ‖y‖ := by rw [data.V_adjoint_norm, one_mul]

omit [Nontrivial E] in
/-- The isometry law in adjoint-composition form. -/
theorem V_adjoint_comp_V (data : DilationData (E := E) (K := K)) :
    (ContinuousLinearMap.adjoint data.V).comp data.V = 1 :=
  data.V.isometry_iff_adjoint_comp_self.mp data.V_isometry

omit [Nontrivial E] in
/-- Applying `V† V` returns the original vector. -/
@[simp]
theorem V_adjoint_V_apply (data : DilationData (E := E) (K := K))
    (x : E) :
    ContinuousLinearMap.adjoint data.V (data.V x) = x := by
  have h := congrArg (fun A : E →L[ℂ] E => A x) data.V_adjoint_comp_V
  simpa using h

omit [Nontrivial E] in
/-- The adjoint of the contraction `Q` is again contractive in operator norm. -/
theorem Q_adjoint_norm_le_one (data : DilationData (E := E) (K := K)) :
    ‖ContinuousLinearMap.adjoint data.Q‖ ≤ 1 := by
  rw [(ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map]
  exact data.Q_norm_le_one

omit [Nontrivial E] in
/-- The adjoint of `Q` is contractive on vectors. -/
theorem Q_adjoint_apply_norm_le (data : DilationData (E := E) (K := K))
    (y : K) :
    ‖ContinuousLinearMap.adjoint data.Q y‖ ≤ ‖y‖ := by
  calc
    ‖ContinuousLinearMap.adjoint data.Q y‖
        ≤ ‖ContinuousLinearMap.adjoint data.Q‖ * ‖y‖ :=
      (ContinuousLinearMap.adjoint data.Q).le_opNorm y
    _ ≤ 1 * ‖y‖ := by
      exact mul_le_mul_of_nonneg_right data.Q_adjoint_norm_le_one (norm_nonneg y)
    _ = ‖y‖ := one_mul _

omit [Nontrivial E] in
/-- Every adjoint power of `Q` is contractive on vectors. -/
theorem Q_adjoint_power_apply_norm_le
    (data : DilationData (E := E) (K := K)) (n : ℕ) (y : K) :
    ‖((ContinuousLinearMap.adjoint data.Q) ^ n) y‖ ≤ ‖y‖ := by
  induction n with
  | zero => simp
  | succ n ih =>
      rw [pow_succ', mul_apply_eq_comp]
      exact (data.Q_adjoint_apply_norm_le _).trans ih

omit [Nontrivial E] in
/-- Concrete evaluation of a compressed adjoint power. -/
@[simp]
theorem compressedAdjointPower_apply
    (data : DilationData (E := E) (K := K)) (n : ℕ) (x : E) :
    compressedAdjointPower data.V data.Q n x =
      ContinuousLinearMap.adjoint data.V
        (((ContinuousLinearMap.adjoint data.Q) ^ n) (data.V x)) :=
  rfl

/-- Every compressed contraction power has norm at most one. -/
theorem compressedAdjointPower_norm_le_one
    (data : DilationData (E := E) (K := K)) (n : ℕ) :
    ‖compressedAdjointPower data.V data.Q n‖ ≤ 1 := by
  refine ContinuousLinearMap.opNorm_le_bound _ zero_le_one fun x => ?_
  calc
    ‖compressedAdjointPower data.V data.Q n x‖
        ≤ ‖((ContinuousLinearMap.adjoint data.Q) ^ n) (data.V x)‖ :=
      data.V_adjoint_apply_norm_le _
    _ ≤ ‖data.V x‖ := data.Q_adjoint_power_apply_norm_le n _
    _ = ‖x‖ := data.V_apply_norm x
    _ = 1 * ‖x‖ := by rw [one_mul]

/-- The source's leading factor two gives a uniform norm bound of two. -/
theorem doubledCompressedAdjointPower_norm_le_two
    (data : DilationData (E := E) (K := K)) (n : ℕ) :
    ‖doubledCompressedAdjointPower data.V data.Q n‖ ≤ 2 := by
  rw [doubledCompressedAdjointPower, norm_smul]
  have htwo : ‖(2 : ℂ)‖ = (2 : ℝ) := by norm_num
  rw [htwo]
  nlinarith [data.compressedAdjointPower_norm_le_one n,
    norm_nonneg (compressedAdjointPower data.V data.Q n)]

/-- The `n`-independent displacement appearing in every adjacent-power
defect. -/
def displacement (data : DilationData (E := E) (K := K))
    (κ : ℝ) (x : E) : K :=
  ContinuousLinearMap.adjoint data.Q (data.V (data.T x)) -
    (κ : ℂ) • data.V x

/-- The source's adjacent-power defect
`y_n = (S_{n+1}† T - κ S_n†)x`. -/
def adjacentPowerDefect (data : DilationData (E := E) (K := K))
    (κ : ℝ) (n : ℕ) (x : E) : E :=
  ((doubledCompressedAdjointPower data.V data.Q (n + 1)).comp data.T -
    (κ : ℂ) • doubledCompressedAdjointPower data.V data.Q n) x

omit [Nontrivial E] in
/-- Factoring adjacent powers isolates a displacement that is independent of
`n`: `y_n = 2 V† (Q†)^n (Q† V T x - κ V x)`. -/
theorem adjacentPowerDefect_factorization
    (data : DilationData (E := E) (K := K))
    (κ : ℝ) (n : ℕ) (x : E) :
    data.adjacentPowerDefect κ n x =
      (2 : ℂ) • ContinuousLinearMap.adjoint data.V
        (((ContinuousLinearMap.adjoint data.Q) ^ n)
          (data.displacement κ x)) := by
  simp only [adjacentPowerDefect, doubledCompressedAdjointPower,
    compressedAdjointPower, displacement, sub_apply,
    ContinuousLinearMap.comp_apply, smul_apply]
  rw [pow_succ, mul_apply_eq_comp]
  simp only [map_sub, map_smul]
  module

/-- The adjacent-power defect is bounded by twice the norm of the common
displacement. -/
theorem adjacentPowerDefect_norm_le
    (data : DilationData (E := E) (K := K))
    (κ : ℝ) (n : ℕ) (x : E) :
    ‖data.adjacentPowerDefect κ n x‖ ≤ 2 * ‖data.displacement κ x‖ := by
  rw [data.adjacentPowerDefect_factorization κ n x, norm_smul]
  have htwo : ‖(2 : ℂ)‖ = (2 : ℝ) := by norm_num
  rw [htwo]
  refine mul_le_mul_of_nonneg_left ?_ (by norm_num)
  exact (data.V_adjoint_apply_norm_le _).trans
    (data.Q_adjoint_power_apply_norm_le n _)

omit [Nontrivial E] in
/-- Adjoint powers and target powers have the same operator norm. -/
theorem target_power_norm_eq_adjoint_power_norm
    (data : DilationData (E := E) (K := K)) (n : ℕ) :
    ‖data.T ^ n‖ = ‖(ContinuousLinearMap.adjoint data.T) ^ n‖ := by
  rw [← (ContinuousLinearMap.adjoint (𝕜 := ℂ)).norm_map (data.T ^ n),
    ← ContinuousLinearMap.star_eq_adjoint (data.T ^ n), star_pow,
    ContinuousLinearMap.star_eq_adjoint data.T]

/-- Equation 1 bounds all adjoint powers of the target. -/
theorem adjoint_power_norm_le_two_add_bound
    (data : DilationData (E := E) (K := K)) (n : ℕ) :
    ‖(ContinuousLinearMap.adjoint data.T) ^ n‖ ≤ 2 + data.bound := by
  have hrewrite :
      (ContinuousLinearMap.adjoint data.T) ^ n =
        doubledCompressedAdjointPower data.V data.Q n - data.perturbation n := by
    rw [data.perturbation_eq n]
    abel
  rw [hrewrite]
  exact (norm_sub_le _ _).trans
    (add_le_add (data.doubledCompressedAdjointPower_norm_le_two n)
      (data.perturbation_norm_le n))

/-- Equation 1 therefore gives the same uniform bound on target powers. -/
theorem target_power_norm_le_two_add_bound
    (data : DilationData (E := E) (K := K)) (n : ℕ) :
    ‖data.T ^ n‖ ≤ 2 + data.bound := by
  rw [data.target_power_norm_eq_adjoint_power_norm n]
  exact data.adjoint_power_norm_le_two_add_bound n

/-- The terminal products `E_n T^n` are uniformly bounded by
`M (2 + M)`. -/
theorem perturbation_mul_target_power_norm_le
    (data : DilationData (E := E) (K := K)) (n : ℕ) :
    ‖data.perturbation n * data.T ^ n‖ ≤
      data.bound * (2 + data.bound) := by
  exact (norm_mul_le _ _).trans
    (mul_le_mul (data.perturbation_norm_le n)
      (data.target_power_norm_le_two_add_bound n)
      (norm_nonneg _) data.bound_nonneg)

omit [Nontrivial E] in
/-- The interface's commutation assumption, exposed as a named theorem. -/
theorem perturbation_commutes_with_target
    (data : DilationData (E := E) (K := K)) (n : ℕ) :
    Commute (data.perturbation n) data.T :=
  data.commutes_with_target n

end DilationData
end LoristSchwenninger
end CrouzeixConjecture

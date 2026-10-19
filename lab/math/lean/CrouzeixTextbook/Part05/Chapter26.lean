import CrouzeixConjecture.ParametricBoundary

namespace CrouzeixTextbook.Part05
open CrouzeixConjecture MeasureTheory
open scoped ComplexConjugate ComplexOrder InnerProductSpace Matrix
  Matrix.Norms.L2Operator MatrixOrder

noncomputable section

set_option linter.defProp false

/-- A supported boundary point of an open domain containing the numerical
range lies outside that numerical range, and every unit-vector Rayleigh
quotient obeys the associated scalar half-plane inequality. -/
theorem support_point_outside_numerical_range
    {n : Type*} [Fintype n] [DecidableEq n]
    {Omega : Set ℂ} {sigma nu : ℂ}
    (hgeom : OutwardBoundarySupport Omega sigma nu)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    sigma ∉ numericalRange B ∧
      ∀ x : EuclideanVector n, ‖x‖ = 1 → 0 ≤ RCLike.re
        (star nu * (sigma - ⟪x, euclideanOperator B x⟫_ℂ)) := by
  constructor
  · exact fun hsigma ↦ hgeom.sigma_not_mem (hWB hsigma)
  · intro x hx
    exact hgeom.support_inequality _ (hWB ⟨x, hx, rfl⟩)

/-- A supported boundary point belongs to the matrix resolvent set. -/
def support_resolvent_invertible :=
  @scalar_sub_matrix_isUnit_of_outwardBoundarySupport

/-- The Hermitian support-line matrix is positive semidefinite. -/
def double_layer_support_positive :=
  @doubleLayerSupportMatrix_posSemidef_of_outwardBoundarySupport

/-- The resolvent used in the double-layer construction. -/
def double_layer_resolvent := @doubleLayerResolvent

/-- Congruence by the supported resolvent turns the support-line matrix into
the unnormalised double-layer density. -/
def double_layer_congruence := @doubleLayerResolvent_congruence_density

/-- On a compact supported parametrized boundary, the pulled-back density is
integrable, its Bochner integral is positive semidefinite, and integration may
be read entry by entry. -/
theorem double_layer_density_positive
    {i n : Type*} [TopologicalSpace i] [CompactSpace i]
    [MeasurableSpace i] [OpensMeasurableSpace i]
    [Fintype n] [DecidableEq n] [Nonempty n]
    (mu : Measure i) [IsFiniteMeasure mu] {Omega : Set ℂ}
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega) :
    Integrable (parametricDoubleLayerDensity Gamma B) mu ∧
      (∫ x, parametricDoubleLayerDensity Gamma B x ∂mu).PosSemidef ∧
      ∀ a b, (∫ x, parametricDoubleLayerDensity Gamma B x ∂mu) a b =
        ∫ x, parametricDoubleLayerDensity Gamma B x a b ∂mu := by
  have hint : Integrable (parametricDoubleLayerDensity Gamma B) mu :=
    integrable_parametricDoubleLayerDensity Gamma B hWB
  refine ⟨hint, integral_posSemidef fun x ↦
    parametricDoubleLayerDensity_posSemidef Gamma B hWB x, ?_⟩
  intro a b
  let evalA : SquareMatrix n →L[ℂ] (n → ℂ) :=
    ContinuousLinearMap.proj a
  let evalB : (n → ℂ) →L[ℂ] ℂ :=
    ContinuousLinearMap.proj b
  change evalB (evalA (∫ x, parametricDoubleLayerDensity Gamma B x ∂mu)) =
    ∫ x, evalB (evalA (parametricDoubleLayerDensity Gamma B x)) ∂mu
  rw [← evalA.integral_comp_comm hint]
  rw [← evalB.integral_comp_comm (evalA.integrable_comp hint)]

namespace Exercises.Chapter26

/-- Transport numerical-range containment into the scalar supporting
half-plane inequality at a unit vector. -/
theorem exercise_01_solution
    (n : Type) [Fintype n] [DecidableEq n]
    {Omega : Set ℂ} {sigma nu : ℂ}
    (hgeom : OutwardBoundarySupport Omega sigma nu)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (x : EuclideanVector n) :
    ‖x‖ = 1 → 0 ≤ RCLike.re
      (star nu * (sigma - ⟪x, euclideanOperator B x⟫_ℂ)) := by
  intro hx
  exact hgeom.support_inequality _ (hWB ⟨x, hx, rfl⟩)

/-- Expand the support matrix against a unit vector and insert the scalar
support inequality. -/
theorem exercise_02_solution
    (n : Type) [Fintype n] [DecidableEq n]
    {Omega : Set ℂ} {sigma nu : ℂ}
    (hgeom : OutwardBoundarySupport Omega sigma nu)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (x : EuclideanVector n) :
    ‖x‖ = 1 → 0 ≤ (euclideanOperator
      (doubleLayerSupportMatrix B sigma nu)).reApplyInnerSelf x := by
  intro hx
  have hs := exercise_01_solution n hgeom B hWB x hx
  have heq :
      (euclideanOperator
          (doubleLayerSupportMatrix B sigma nu)).reApplyInnerSelf x =
        2 * RCLike.re
          (star nu * (sigma - ⟪x, euclideanOperator B x⟫_ℂ)) := by
    rw [ContinuousLinearMap.reApplyInnerSelf_apply, inner_re_symm]
    simp only [doubleLayerSupportMatrix, map_add, map_smul, map_sub, map_one,
      ContinuousLinearMap.add_apply, ContinuousLinearMap.smul_apply,
      ContinuousLinearMap.sub_apply, ContinuousLinearMap.one_apply,
      inner_add_right, inner_smul_right, inner_sub_right,
      euclideanOperator_conjTranspose, ContinuousLinearMap.adjoint_inner_right,
      inner_self_eq_norm_sq_to_K, hx, one_pow]
    rw [show ⟪euclideanOperator B x, x⟫_ℂ =
        star ⟪x, euclideanOperator B x⟫_ℂ by simp]
    simp only [mul_one]
    rw [show nu * (star sigma - star ⟪x, euclideanOperator B x⟫_ℂ) =
        star (star nu * (sigma - ⟪x, euclideanOperator B x⟫_ℂ)) by simp]
    simp
    have hreinner : (⟪euclideanOperator B x, x⟫_ℂ).re =
        (⟪x, euclideanOperator B x⟫_ℂ).re := by
      exact inner_re_symm (𝕜 := ℂ) (euclideanOperator B x) x
    have himinner : (⟪euclideanOperator B x, x⟫_ℂ).im =
        -(⟪x, euclideanOperator B x⟫_ℂ).im := by
      exact inner_im_symm (𝕜 := ℂ) (euclideanOperator B x) x
    rw [hreinner, himinner]
    ring
  rw [heq]
  exact mul_nonneg (by norm_num) hs

/-- A positive semidefinite matrix remains positive semidefinite after a
matrix congruence. -/
theorem exercise_03_solution
    (n : Type) [Fintype n] [DecidableEq n]
    (M B : SquareMatrix n) :
    M.PosSemidef → (Bᴴ * M * B).PosSemidef := by
  intro hM
  refine Matrix.PosSemidef.of_dotProduct_mulVec_nonneg ?_ fun x ↦ ?_
  · simp only [Matrix.IsHermitian, Matrix.conjTranspose_mul,
      Matrix.conjTranspose_conjTranspose, hM.1.eq, Matrix.mul_assoc]
  · simpa only [Matrix.star_mulVec, Matrix.dotProduct_mulVec,
      Matrix.vecMul_vecMul] using hM.dotProduct_mulVec_nonneg (B *ᵥ x)

/-- Expand both support terms, obtain the adjoint left inverse from the given
right inverse, and cancel all factors. -/
theorem exercise_04_solution
    (n : Type) [Fintype n] [DecidableEq n]
    (B R : SquareMatrix n) (sigma nu : ℂ) :
    (sigma • (1 : SquareMatrix n) - B) * R = 1 →
      Rᴴ * doubleLayerSupportMatrix B sigma nu * R =
        doubleLayerDensity R nu := by
  intro hR
  have hRstar : Rᴴ * (star sigma • (1 : SquareMatrix n) - Bᴴ) = 1 := by
    have h := congrArg Matrix.conjTranspose hR
    simpa only [Matrix.conjTranspose_mul, Matrix.conjTranspose_sub,
      Matrix.conjTranspose_smul, Matrix.conjTranspose_one, starRingEnd_apply,
      star_star] using h
  simp only [doubleLayerSupportMatrix, doubleLayerDensity, mul_add, add_mul,
    mul_smul_comm, smul_mul_assoc, mul_assoc, hR, hRstar, one_mul, mul_one]

/-- Convert a hypothetical spectral witness into a numerical-range witness,
contradict the open-domain boundary condition, and obtain matrix invertibility. -/
theorem exercise_05_solution
    (n : Type) [Fintype n] [DecidableEq n]
    {Omega : Set ℂ} {sigma nu : ℂ}
    (hgeom : OutwardBoundarySupport Omega sigma nu)
    (B : SquareMatrix n) :
    numericalRange B ⊆ Omega →
      IsUnit (sigma • (1 : SquareMatrix n) - B) := by
  intro hWB
  have hsigmaNumerical : sigma ∉ numericalRange B := fun hsigma ↦
    hgeom.sigma_not_mem (hWB hsigma)
  have hsigmaSpectrum : sigma ∉ matrixSpectrum B := fun hsigma ↦
    hsigmaNumerical (matrixSpectrum_subset_numericalRange B hsigma)
  change sigma ∉ spectrum ℂ B at hsigmaSpectrum
  have hunit := spectrum.notMem_iff.mp hsigmaSpectrum
  simpa only [Algebra.algebraMap_eq_smul_one] using hunit

/-- Reconstruct pointwise density positivity from support geometry, inversion,
the expanded congruence, and preservation of positivity under congruence. -/
theorem exercise_06_solution
    (n : Type) [Fintype n] [DecidableEq n]
    {Omega : Set ℂ} {sigma nu : ℂ}
    (hgeom : OutwardBoundarySupport Omega sigma nu)
    (B : SquareMatrix n) :
    numericalRange B ⊆ Omega →
      (doubleLayerDensity (doubleLayerResolvent B sigma) nu).PosSemidef := by
  intro hWB
  have hsupport : (doubleLayerSupportMatrix B sigma nu).PosSemidef := by
    change IsPositiveMatrix (doubleLayerSupportMatrix B sigma nu)
    rw [isPositiveMatrix_iff_euclideanOperator_isPositive]
    rw [ContinuousLinearMap.isPositive_def]
    refine ⟨Matrix.isHermitian_iff_isSymmetric.mp
      (doubleLayerSupportMatrix_isHermitian B sigma nu), ?_⟩
    intro x
    by_cases hx : x = 0
    · subst x
      simp [ContinuousLinearMap.reApplyInnerSelf_apply]
    · let y : EuclideanVector n := (‖x‖ : ℂ)⁻¹ • x
      have hxnorm : ‖x‖ ≠ 0 := norm_ne_zero_iff.mpr hx
      have hynorm : ‖y‖ = 1 := by
        simp [y, norm_smul, hxnorm]
      have hxy : (‖x‖ : ℂ) • y = x := by
        simp [y, smul_smul, hxnorm]
      rw [← hxy, ContinuousLinearMap.reApplyInnerSelf_smul]
      exact mul_nonneg (sq_nonneg _)
        (exercise_02_solution n hgeom B hWB y hynorm)
  have hunit := exercise_05_solution n hgeom B hWB
  have hdet : IsUnit (sigma • (1 : SquareMatrix n) - B).det :=
    (sigma • (1 : SquareMatrix n) - B).isUnit_iff_isUnit_det.mp hunit
  have hR : (sigma • (1 : SquareMatrix n) - B) *
      doubleLayerResolvent B sigma = 1 :=
    (sigma • (1 : SquareMatrix n) - B).mul_nonsing_inv hdet
  rw [← exercise_04_solution n B (doubleLayerResolvent B sigma) sigma nu hR]
  exact exercise_03_solution n (doubleLayerSupportMatrix B sigma nu)
    (doubleLayerResolvent B sigma) hsupport

end Exercises.Chapter26

end
end CrouzeixTextbook.Part05

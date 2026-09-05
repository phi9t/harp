import Crouzeix.LoristSchwenninger.ConcreteDilation
import Crouzeix.LoristSchwenninger.CompressionMoments
import Mathlib.MeasureTheory.Integral.CircleIntegral

namespace CrouzeixTextbook.Part06
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def boundary_dilation_data := @LoristSchwenninger.dilationDataOfParametricPolynomial
set_option linter.defProp false in
def boundary_compression_first_moment := @LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding
set_option linter.defProp false in
def boundary_compression_power_moments := @LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding
set_option linter.defProp false in
def boundary_multiplier_powers := @LoristSchwenninger.bcfMulL_pow
set_option linter.defProp false in
def boundary_multiplier_contractive := @LoristSchwenninger.bcfMulL_norm_le_one
set_option linter.defProp false in
def realization_norm_two := @LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary
open CrouzeixConjecture.LoristSchwenninger
open MeasureTheory
open scoped BoundedContinuousFunction ComplexOrder MatrixOrder Matrix.Norms.L2Operator

namespace Exercises.Chapter34

/-- Exercise 34.1: verify the isometry field from the normalized boundary
mass identity. -/
theorem exercise_01_solution {i n : Type*} [MeasurableSpace i]
    [Fintype n] [DecidableEq n] {mu : Measure i}
    (D : PositiveBoundaryDensity (n := n) mu) (x : EuclideanVector n) :
    ‖boundaryEmbeddingToLp D x‖ = ‖x‖ := by
  exact (norm_boundaryEmbeddingToLp D x).trans rfl

/-- Exercise 34.2: calculate the compressed multiplier on one vector by
expanding the boundary `L²` inner product. -/
theorem exercise_02_solution {i n : Type*} [MeasurableSpace i] [TopologicalSpace i]
    [BorelSpace i] [SecondCountableTopologyEither i ℂ]
    [Fintype n] [DecidableEq n] {mu : Measure i}
    (D : PositiveBoundaryDensity (n := n) mu) (h : i →ᵇ ℂ)
    (x : EuclideanVector n) :
    ((ContinuousLinearMap.adjoint (boundaryEmbedding D).toContinuousLinearMap).comp
        ((bcfMulL (mu := mu) (n := n) h).comp
          (boundaryEmbedding D).toContinuousLinearMap)) x =
      euclideanOperator (boundaryPhiCLM D h) x := by
  apply ext_inner_left ℂ
  intro y
  rw [ContinuousLinearMap.comp_apply, ContinuousLinearMap.adjoint_inner_right]
  rw [ContinuousLinearMap.comp_apply]
  rw [MeasureTheory.L2.inner_def]
  have hx :
      boundaryEmbedding D x =ᵐ[mu] boundaryEmbeddingField D x :=
    (boundaryEmbeddingField_memLp D x).coeFn_toLp
  have hy :
      boundaryEmbedding D y =ᵐ[mu] boundaryEmbeddingField D y :=
    (boundaryEmbeddingField_memLp D y).coeFn_toLp
  have hmul :
      bcfMulL h (boundaryEmbedding D x) =ᵐ[mu]
        fun z ↦ h z • boundaryEmbedding D x z :=
    bcfMulL_apply_ae h (boundaryEmbedding D x)
  have hcongr :
      (fun z ↦ inner ℂ (boundaryEmbedding D y z)
        (bcfMulL h (boundaryEmbedding D x) z)) =ᵐ[mu]
        (fun z ↦ (2 : ℂ)⁻¹ *
          inner ℂ y (euclideanOperator (h z • D.density z) x)) := by
    filter_upwards [hy, hmul, hx, boundarySquareRoot_mul_self_ae D]
      with z hyz hmz hxz hsq
    rw [hyz, hmz, hxz, boundaryEmbeddingField, boundaryEmbeddingField]
    have hsqrt :
        inner ℂ (euclideanOperator (boundarySquareRoot D z) y)
            (euclideanOperator (boundarySquareRoot D z) x) =
          inner ℂ y (euclideanOperator (D.density z) x) := by
      rw [← ContinuousLinearMap.adjoint_inner_right]
      rw [← euclideanOperator_conjTranspose,
        (boundarySquareRoot_posSemidef D z).isHermitian.eq]
      have happly :
          euclideanOperator (boundarySquareRoot D z)
              (euclideanOperator (boundarySquareRoot D z) x) =
            euclideanOperator (D.density z) x := by
        have hmatrix := congrArg
          (fun A : SquareMatrix n ↦ euclideanOperator A x) hsq
        simpa only [map_mul, mul_apply_eq_comp] using hmatrix
      rw [happly]
    rw [inner_smul_left, inner_smul_right, inner_smul_right, hsqrt]
    rw [map_smul, smul_apply, inner_smul_right]
    have hsqrt_ne : (Real.sqrt 2 : ℂ) ≠ 0 := by
      exact_mod_cast Real.sqrt_ne_zero'.mpr two_pos
    have hstar :
        (starRingEnd ℂ) ((Real.sqrt 2 : ℂ)⁻¹) =
          ((Real.sqrt 2 : ℂ)⁻¹) := by
      rw [map_inv₀]
      norm_num
    have hhalf :
        ((Real.sqrt 2 : ℂ)⁻¹) * ((Real.sqrt 2 : ℂ)⁻¹) =
          (2 : ℂ)⁻¹ := by
      field_simp [hsqrt_ne]
      have hsqrt_sq : Real.sqrt 2 ^ 2 = (2 : ℝ) :=
        Real.sq_sqrt (by norm_num)
      exact_mod_cast hsqrt_sq.symm
    rw [hstar]
    calc
      ((Real.sqrt 2 : ℂ)⁻¹) *
          (h z * (((Real.sqrt 2 : ℂ)⁻¹) *
            inner ℂ y (euclideanOperator (D.density z) x))) =
          (((Real.sqrt 2 : ℂ)⁻¹) * ((Real.sqrt 2 : ℂ)⁻¹)) *
            (h z * inner ℂ y (euclideanOperator (D.density z) x)) := by
              ring
      _ = (2 : ℂ)⁻¹ *
          (h z * inner ℂ y (euclideanOperator (D.density z) x)) := by
            rw [hhalf]
  change
    ∫ a : i, inner ℂ (((boundaryEmbedding D).toContinuousLinearMap) y a)
        (((bcfMulL h) (((boundaryEmbedding D).toContinuousLinearMap) x)) a) ∂mu =
      inner ℂ y (euclideanOperator (boundaryPhiCLM D h) x)
  have hcongr' :
      (fun z ↦ inner ℂ
        (((boundaryEmbedding D).toContinuousLinearMap) y z)
        (bcfMulL h (((boundaryEmbedding D).toContinuousLinearMap) x) z)) =ᵐ[mu]
        (fun z ↦ (2 : ℂ)⁻¹ *
          inner ℂ y (euclideanOperator (h z • D.density z) x)) := by
    filter_upwards [hcongr] with z hz
    change inner ℂ (boundaryEmbedding D y z)
      (bcfMulL h (boundaryEmbedding D x) z) = _
    exact hz
  rw [integral_congr_ae hcongr']
  have h_int :
      Integrable (fun z ↦ h z • D.density z) mu :=
    D.integrable_smul h
  let L :
      SquareMatrix n →L[ℂ] EuclideanVector n :=
    (ContinuousLinearMap.apply ℂ (EuclideanVector n) x).comp euclideanOperatorCLM
  have h_int_vec :
      Integrable (fun z ↦ euclideanOperator (h z • D.density z) x) mu :=
    L.integrable_comp h_int
  have h_integral :
      ∫ z, euclideanOperator (h z • D.density z) x ∂mu =
        euclideanOperator (∫ z, h z • D.density z ∂mu) x := by
    simpa [L, euclideanOperatorCLM_apply] using L.integral_comp_comm h_int
  calc
    ∫ z, (2 : ℂ)⁻¹ * inner ℂ y
        (euclideanOperator (h z • D.density z) x) ∂mu =
        (2 : ℂ)⁻¹ * ∫ z, inner ℂ y
          (euclideanOperator (h z • D.density z) x) ∂mu := by
            rw [integral_const_mul]
    _ = (2 : ℂ)⁻¹ * inner ℂ y
        (∫ z, euclideanOperator (h z • D.density z) x ∂mu) := by
          rw [integral_inner h_int_vec y]
    _ = inner ℂ y ((2 : ℂ)⁻¹ •
        (euclideanOperator (∫ z, h z • D.density z ∂mu) x)) := by
          rw [h_integral, inner_smul_right]
    _ = inner ℂ y (euclideanOperator (boundaryPhiCLM D h) x) := by
          rw [boundaryPhiCLM_apply, boundaryPhi]
          simp only [map_smul, smul_apply]

/-- Exercise 34.3: on the unit circle, the Cauchy kernel selects the
Fourier mode with exponent `-1`, while every other integer power integrates
to zero. Finite-sum linearity then selects the constant coefficient of a
polynomial after division by `z`. -/
theorem exercise_03_solution :
    ((∮ z in C((0 : ℂ), 1), z⁻¹) = 2 * Real.pi * Complex.I) ∧
      (∀ n : ℤ, n ≠ -1 → (∮ z in C((0 : ℂ), 1), z ^ n) = 0) ∧
      ∀ (N : ℕ) (a : ℕ → ℂ),
        (∮ z in C((0 : ℂ), 1),
            ∑ m ∈ Finset.range (N + 1), a m * z ^ ((m : ℤ) - 1)) =
          a 0 * (2 * Real.pi * Complex.I) := by
  have hInv : (∮ z in C((0 : ℂ), 1), z⁻¹) = 2 * Real.pi * Complex.I := by
    simpa using circleIntegral.integral_sub_inv_of_mem_ball
      (c := (0 : ℂ)) (w := (0 : ℂ)) (R := 1) (by simp)
  have hOther : ∀ n : ℤ, n ≠ -1 → (∮ z in C((0 : ℂ), 1), z ^ n) = 0 := by
    intro n hn
    simpa using circleIntegral.integral_sub_zpow_of_ne hn (0 : ℂ) (0 : ℂ) 1
  refine ⟨hInv, hOther, ?_⟩
  intro N a
  rw [circleIntegral.integral_fun_sum]
  · rw [Finset.sum_eq_single 0]
    · simpa using congrArg (a 0 * ·) hInv
    · intro m hm hm0
      rw [circleIntegral.integral_const_mul, hOther]
      · simp
      · omega
    · simp
  · intro m hm
    have hPower : CircleIntegrable (fun z : ℂ ↦ z ^ ((m : ℤ) - 1)) 0 1 := by
      simpa only [sub_zero] using
        (circleIntegrable_sub_zpow_iff (c := (0 : ℂ)) (w := (0 : ℂ))
          (R := 1) (n := (m : ℤ) - 1)).2 (by simp)
    simpa [smul_eq_mul] using hPower.const_fun_smul (a := a m)

/-- Exercise 34.4: derive the pointwise power-compression identity from the
multiplier power bridge and the first-moment compression theorem. -/
theorem exercise_04_solution {i n : Type*} [MeasurableSpace i] [TopologicalSpace i]
    [BorelSpace i] [SecondCountableTopologyEither i ℂ]
    [Fintype n] [DecidableEq n] {mu : Measure i}
    (D : PositiveBoundaryDensity (n := n) mu) (h : i →ᵇ ℂ) (k : ℕ)
    (x : EuclideanVector n) :
    ((ContinuousLinearMap.adjoint (boundaryEmbedding D).toContinuousLinearMap).comp
        (((bcfMulL (mu := mu) (n := n) h) ^ k).comp
          (boundaryEmbedding D).toContinuousLinearMap)) x =
      euclideanOperator (boundaryPhiCLM D (h ^ k)) x := by
  rw [← bcfMulL_pow (mu := mu) (n := n) h k]
  exact congrArg (fun L => L x)
    (boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding D (h ^ k))

/-- Exercise 34.5: derive the applied contraction estimate from the operator
norm estimate, without calling the parent unit-contraction theorem. -/
theorem exercise_05_solution {i n : Type*} [MeasurableSpace i] [TopologicalSpace i]
    [BorelSpace i] [SecondCountableTopologyEither i ℂ]
    [Fintype n] {mu : Measure i} (h : i →ᵇ ℂ) (hh : ‖h‖ ≤ 1)
    (f : i →₂[mu] EuclideanVector n) :
    ‖bcfMulL (mu := mu) (n := n) h f‖ ≤ ‖f‖ := by
  calc
    ‖bcfMulL (mu := mu) (n := n) h f‖
        ≤ ‖bcfMulL (mu := mu) (n := n) h‖ * ‖f‖ :=
      (bcfMulL (mu := mu) (n := n) h).le_opNorm f
    _ ≤ 1 * ‖f‖ := by
      exact mul_le_mul_of_nonneg_right ((bcfMulL_norm_le h).trans hh) (norm_nonneg f)
    _ = ‖f‖ := one_mul _

/-- Exercise 34.6: instantiate the concrete boundary realization, identify its
target, and assemble its factor-two endpoint. -/
theorem exercise_06_solution {i n : Type*} [TopologicalSpace i] [CompactSpace i]
    [MeasurableSpace i] [BorelSpace i] [OpensMeasurableSpace i]
    [SecondCountableTopologyEither i ℂ]
    [Fintype n] [DecidableEq n] [Nonempty n]
    {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Omega, ‖Polynomial.eval z q‖ ≤ 1)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B) :
    (dilationDataOfParametricPolynomial Gamma B hWB q hq hCauchy).T =
        euclideanOperator (polynomialEval q B) ∧
      ‖(dilationDataOfParametricPolynomial Gamma B hWB q hq hCauchy).T‖ ≤ 2 := by
  exact ⟨rfl,
    (dilationDataOfParametricPolynomial Gamma B hWB q hq hCauchy).norm_target_le_two⟩

end Exercises.Chapter34

end
end CrouzeixTextbook.Part06

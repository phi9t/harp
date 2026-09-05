import CrouzeixConjecture.Sharpness
import CrouzeixConjecture.NumericalRangeConvexity
import CrouzeixConjecture.Spectrum
import Mathlib.Analysis.CStarAlgebra.ContinuousLinearMap
import Mathlib.Analysis.CStarAlgebra.ContinuousFunctionalCalculus.Basic

namespace CrouzeixTextbook.Part06
open Set
open CrouzeixConjecture
open scoped ComplexConjugate InnerProductSpace Matrix Matrix.Norms.L2Operator

noncomputable section

private theorem jordanNumericalRange_subset_closedBall :
    numericalRange jordanNilpotentTwo ⊆ Metric.closedBall 0 ((1 : ℝ) / 2) := by
  rintro z ⟨x, hx, rfl⟩
  rw [Metric.mem_closedBall, dist_zero_right, inner_jordanNilpotentTwo,
    norm_mul, Complex.norm_conj]
  have hcoords := euclideanVector_finTwo_norm_sq x
  rw [hx] at hcoords
  norm_num at hcoords ⊢
  nlinarith [sq_nonneg (‖x 0‖ - ‖x 1‖)]

private theorem jordanSphere_subset_numericalRange :
    Metric.sphere (0 : ℂ) ((1 : ℝ) / 2) ⊆
      numericalRange jordanNilpotentTwo := by
  intro z hz
  have hzNorm : ‖z‖ = (1 : ℝ) / 2 := by
    simpa [Metric.mem_sphere, dist_zero_right] using hz
  let s : ℝ := Real.sqrt 2
  let x : EuclideanVector (Fin 2) :=
    WithLp.toLp 2 ![((s⁻¹ : ℝ) : ℂ), ((s : ℝ) : ℂ) * z]
  have hspos : 0 < s := by
    simpa only [s] using Real.sqrt_pos.2 (by norm_num : (0 : ℝ) < 2)
  have hssq : s ^ 2 = 2 := by
    simpa only [s] using Real.sq_sqrt (by norm_num : (0 : ℝ) ≤ 2)
  have hxNormSq : ‖x‖ ^ 2 = 1 := by
    rw [euclideanVector_finTwo_norm_sq]
    change ‖((s⁻¹ : ℝ) : ℂ)‖ ^ 2 + ‖((s : ℝ) : ℂ) * z‖ ^ 2 = 1
    rw [norm_mul, Complex.norm_real, Complex.norm_real, Real.norm_eq_abs,
      Real.norm_eq_abs, abs_of_pos hspos, abs_of_pos (inv_pos.2 hspos), hzNorm]
    field_simp [ne_of_gt hspos]
    nlinarith
  have hxNorm : ‖x‖ = 1 := by
    nlinarith [norm_nonneg x]
  refine ⟨x, hxNorm, ?_⟩
  rw [inner_jordanNilpotentTwo]
  change conj (((s⁻¹ : ℝ) : ℂ)) * (((s : ℝ) : ℂ) * z) = z
  rw [Complex.conj_ofReal, ← mul_assoc]
  norm_cast
  norm_num [ne_of_gt hspos]

private theorem jordanNumericalRange_eq_closedBall :
    numericalRange jordanNilpotentTwo = Metric.closedBall 0 ((1 : ℝ) / 2) := by
  apply Set.Subset.antisymm jordanNumericalRange_subset_closedBall
  rw [← convexHull_sphere_eq_closedBall (0 : ℂ) (by norm_num : (0 : ℝ) ≤ 1 / 2)]
  exact convexHull_min jordanSphere_subset_numericalRange
    (numericalRange_convex jordanNilpotentTwo)

private theorem scaledJordanNumericalRange_eq_unitDisk :
    numericalRange (((2 : ℂ) • jordanNilpotentTwo)) =
      Metric.closedBall 0 (1 : ℝ) := by
  ext z
  constructor
  · rintro ⟨x, hx, hzx⟩
    have hbase :
        ⟪x, (euclideanOperator jordanNilpotentTwo) x⟫_ℂ ∈
          Metric.closedBall (0 : ℂ) ((1 : ℝ) / 2) := by
      apply jordanNumericalRange_subset_closedBall
      exact ⟨x, hx, rfl⟩
    rw [Metric.mem_closedBall, dist_zero_right] at hbase ⊢
    have hscale :
        ⟪x, (euclideanOperator (((2 : ℂ) • jordanNilpotentTwo))) x⟫_ℂ =
          2 * ⟪x, (euclideanOperator jordanNilpotentTwo) x⟫_ℂ := by
      rw [map_smul, smul_apply, inner_smul_right]
    rw [← hzx, hscale, norm_mul]
    norm_num at hbase ⊢
    linarith
  · intro hz
    have hzNorm : ‖z‖ ≤ 1 := by
      simpa [Metric.mem_closedBall, dist_zero_right] using hz
    have hhalf : z / 2 ∈ Metric.closedBall (0 : ℂ) ((1 : ℝ) / 2) := by
      rw [Metric.mem_closedBall, dist_zero_right, norm_div]
      norm_num
      linarith
    rw [← jordanNumericalRange_eq_closedBall] at hhalf
    obtain ⟨x, hx, hzx⟩ := hhalf
    refine ⟨x, hx, ?_⟩
    calc
      ⟪x, (euclideanOperator (((2 : ℂ) • jordanNilpotentTwo))) x⟫_ℂ =
          2 * ⟪x, (euclideanOperator jordanNilpotentTwo) x⟫_ℂ := by
        rw [map_smul, smul_apply, inner_smul_right]
      _ = 2 * (z / 2) := by rw [hzx]
      _ = z := by ring

private theorem scaledJordanMaximum_eq_one :
    maxPolynomialModulusOnNumericalRange
        (((2 : ℂ) • jordanNilpotentTwo)) Polynomial.X = 1 := by
  let A : SquareMatrix (Fin 2) := ((2 : ℂ) • jordanNilpotentTwo)
  obtain ⟨z, hz, hmax⟩ :=
    exists_maxPolynomialModulusOnNumericalRange A Polynomial.X
  apply le_antisymm
  · rw [← hmax, Polynomial.eval_X]
    have hz' : z ∈ Metric.closedBall (0 : ℂ) (1 : ℝ) := by
      simpa only [A, scaledJordanNumericalRange_eq_unitDisk] using hz
    simpa [Metric.mem_closedBall, dist_zero_right] using hz'
  · have hone : (1 : ℂ) ∈ numericalRange A := by
      rw [show numericalRange A = Metric.closedBall 0 (1 : ℝ) by
        simpa only [A] using scaledJordanNumericalRange_eq_unitDisk]
      simp
    simpa only [Polynomial.eval_X, norm_one] using
      norm_polynomial_eval_le_maxOnNumericalRange A Polynomial.X hone

private theorem scaledJordanPolynomialNorm_eq_two :
    ‖polynomialEval Polynomial.X (((2 : ℂ) • jordanNilpotentTwo))‖ = 2 := by
  simp [polynomialEval, norm_smul, norm_jordanNilpotentTwo]

/-- Scaling a polynomial by the inverse of its positive numerical-range
maximum normalizes that maximum to one. -/
theorem normalized_max_polynomial_modulus
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ) (M : ℝ)
    (hM : 0 < M)
    (hmax : maxPolynomialModulusOnNumericalRange A p = M) :
    maxPolynomialModulusOnNumericalRange A (((M⁻¹ : ℝ) : ℂ) • p) = 1 := by
  let q : Polynomial ℂ := (((M⁻¹ : ℝ) : ℂ) • p)
  obtain ⟨z, hz, hpz⟩ := exists_maxPolynomialModulusOnNumericalRange A p
  obtain ⟨w, hw, hqw⟩ := exists_maxPolynomialModulusOnNumericalRange A q
  have hscalar : ‖(((M⁻¹ : ℝ) : ℂ))‖ = M⁻¹ := by
    rw [Complex.norm_real, Real.norm_eq_abs, abs_of_pos (inv_pos.2 hM)]
  apply le_antisymm
  · rw [← hqw]
    change ‖Polynomial.eval w ((((M⁻¹ : ℝ) : ℂ) • p))‖ ≤ 1
    rw [Polynomial.eval_smul, norm_smul, hscalar]
    apply (inv_mul_le_one₀ hM).mpr
    rw [← hmax]
    exact norm_polynomial_eval_le_maxOnNumericalRange A p hw
  · have hz_le :=
      norm_polynomial_eval_le_maxOnNumericalRange A q hz
    have hqz : ‖q.eval z‖ = 1 := by
      change ‖Polynomial.eval z ((((M⁻¹ : ℝ) : ℂ) • p))‖ = 1
      rw [Polynomial.eval_smul, norm_smul, hscalar, hpz, hmax]
      exact inv_mul_cancel₀ (ne_of_gt hM)
    simpa only [hqz] using hz_le

/-! The six stable theorem-card declarations. -/

/-- The maximum in the polynomial Crouzeix problem is attained, and the
attaining value bounds every other value on the numerical range. -/
theorem max_polynomial_modulus
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ) :
    ∃ z ∈ numericalRange A,
      ‖p.eval z‖ = maxPolynomialModulusOnNumericalRange A p ∧
      ∀ w ∈ numericalRange A,
        ‖p.eval w‖ ≤ maxPolynomialModulusOnNumericalRange A p := by
  obtain ⟨z, hz, hmax⟩ := exists_maxPolynomialModulusOnNumericalRange A p
  exact ⟨z, hz, hmax, fun w hw ↦ norm_polynomial_eval_le_maxOnNumericalRange A p hw⟩

/-- A supplied polynomial Crouzeix bound includes both its displayed
inequality and the logically separate zero-maximum edge case. -/
theorem polynomial_crouzeix_bound
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ)
    (hbound : PolynomialCrouzeixBound A p) :
    ‖polynomialEval p A‖ ≤
        2 * maxPolynomialModulusOnNumericalRange A p ∧
      (maxPolynomialModulusOnNumericalRange A p = 0 →
        polynomialEval p A = 0) := by
  refine ⟨hbound, fun hzero ↦ ?_⟩
  change ‖polynomialEval p A‖ ≤
    2 * maxPolynomialModulusOnNumericalRange A p at hbound
  apply norm_eq_zero.mp
  apply le_antisymm
  · simpa only [hzero, mul_zero] using hbound
  · exact norm_nonneg _

/-- The finite-matrix theorem surface is exactly the fully quantified family
of matrix-polynomial bounds; it is not the rational or Hilbert-space surface. -/
theorem main_theorem_statement :
    FiniteMatrixMainTheoremStatement ↔
      ∀ (d : ℕ) [Nonempty (Fin d)] (A : SquareMatrix (Fin d))
        (p : Polynomial ℂ), PolynomialCrouzeixBound A p := by
  rfl

/-- The scaled Jordan example has the exact unit-disk numerical range, and the
identity polynomial consequently has maximum modulus one. -/
theorem jordan_two_maximum :
    numericalRange (((2 : ℂ) • jordanNilpotentTwo)) =
        Metric.closedBall 0 (1 : ℝ) ∧
      maxPolynomialModulusOnNumericalRange
        (((2 : ℂ) • jordanNilpotentTwo)) Polynomial.X = 1 := by
  exact ⟨scaledJordanNumericalRange_eq_unitDisk, scaledJordanMaximum_eq_one⟩

/-- The identity polynomial has matrix norm two on the scaled Jordan block and
attains the sharp factor two relative to its unit numerical-range maximum. -/
theorem jordan_two_attains_two :
    ‖polynomialEval Polynomial.X (((2 : ℂ) • jordanNilpotentTwo))‖ = 2 ∧
      ‖polynomialEval Polynomial.X (((2 : ℂ) • jordanNilpotentTwo))‖ /
          maxPolynomialModulusOnNumericalRange
            (((2 : ℂ) • jordanNilpotentTwo)) Polynomial.X = 2 ∧
      ‖polynomialEval Polynomial.X (((2 : ℂ) • jordanNilpotentTwo))‖ =
        2 * maxPolynomialModulusOnNumericalRange
          (((2 : ℂ) • jordanNilpotentTwo)) Polynomial.X := by
  have hnorm := scaledJordanPolynomialNorm_eq_two
  refine ⟨hnorm, ?_, ?_⟩
  · rw [hnorm, scaledJordanMaximum_eq_one]
    norm_num
  · rw [hnorm, scaledJordanMaximum_eq_one, mul_one]

/-- Two is the least universal rational numerical-range constant even in
dimension two: the upper and lower halves of `IsLeast` are proved separately. -/
theorem constant_two_is_least :
    IsLeast {K : ℝ | RationalCrouzeixBoundOnFinTwo K} 2 := by
  refine ⟨?_, ?_⟩
  · intro A f hfree
    exact holomorphicCrouzeixRationalBound A f hfree
  · intro K hK
    let A0 : SquareMatrix (Fin 2) :=
      ((2 : ℂ) • jordanNilpotentTwo)
    let x : RatFunc ℂ :=
      algebraMap (Polynomial ℂ) (RatFunc ℂ) Polynomial.X
    have hbound :
        ‖rationalMatrixEval x A0‖ ≤
          K * maxRationalModulusOnNumericalRange A0 x :=
      hK A0 x
        (rationalPoleFreeOn_algebraMap_polynomial Polynomial.X
          (numericalRange A0))
    have hnorm : ‖rationalMatrixEval x A0‖ = 2 := by
      rw [show rationalMatrixEval x A0 = polynomialEval Polynomial.X A0 by
        simpa only [x] using rationalMatrixEval_algebraMap_polynomial Polynomial.X A0]
      simpa only [A0] using scaledJordanPolynomialNorm_eq_two
    have hmax :
        maxRationalModulusOnNumericalRange A0 x = 1 := by
      calc
        maxRationalModulusOnNumericalRange A0 x =
            maxPolynomialModulusOnNumericalRange A0 Polynomial.X := by
              simpa only [x] using
                maxRationalModulusOnNumericalRange_algebraMap_polynomial
                  A0 Polynomial.X
        _ = 1 := by simpa only [A0] using scaledJordanMaximum_eq_one
    rw [hnorm, hmax] at hbound
    nlinarith

namespace Exercises.Chapter29

/-- CFT-29-E01: the zero-maximum edge case. -/
theorem exercise_01_solution :
    ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n]
      (A : SquareMatrix n) (p : Polynomial ℂ),
      PolynomialCrouzeixBound A p →
      maxPolynomialModulusOnNumericalRange A p = 0 →
      polynomialEval p A = 0 := by
  intro n _ _ _ A p hbound hzero
  apply norm_eq_zero.mp
  apply le_antisymm
  · simpa only [PolynomialCrouzeixBound, hzero, mul_zero] using hbound
  · exact norm_nonneg _

/-- CFT-29-E02: the exact numerical range of the nilpotent Jordan block. -/
theorem exercise_02_solution :
    numericalRange jordanNilpotentTwo = Metric.closedBall 0 ((1 : ℝ) / 2) := by
  apply Set.Subset.antisymm jordanNumericalRange_subset_closedBall
  rw [← convexHull_sphere_eq_closedBall (0 : ℂ) (by norm_num : (0 : ℝ) ≤ 1 / 2)]
  exact convexHull_min jordanSphere_subset_numericalRange
    (numericalRange_convex jordanNilpotentTwo)

/-- CFT-29-E03: the Jordan-block sharpness ratio. -/
theorem exercise_03_solution :
    ‖polynomialEval Polynomial.X jordanNilpotentTwo‖ /
        maxPolynomialModulusOnNumericalRange jordanNilpotentTwo Polynomial.X = 2 := by
  rw [norm_polynomialEval_X_jordanNilpotentTwo,
    maxPolynomialModulusOnNumericalRange_X_jordanNilpotentTwo]
  norm_num

/-- CFT-29-E04: normal matrices have polynomial numerical-range constant one. -/
theorem exercise_04_solution :
    ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n]
      (A : SquareMatrix n) (p : Polynomial ℂ),
      A * Aᴴ = Aᴴ * A →
      ‖polynomialEval p A‖ ≤ maxPolynomialModulusOnNumericalRange A p := by
  intro n _ _ _ A p hnormal
  letI : IsStarNormal A := ⟨hnormal.symm⟩
  letI : IsStarNormal (euclideanOperator A) := inferInstance
  letI : ContinuousFunctionalCalculus ℂ
      (EuclideanVector n →L[ℂ] EuclideanVector n) IsStarNormal :=
    IsStarNormal.instContinuousFunctionalCalculus
  obtain ⟨z, hz, hmax⟩ := exists_maxPolynomialModulusOnNumericalRange A p
  have hnonneg : 0 ≤ maxPolynomialModulusOnNumericalRange A p := by
    rw [← hmax]
    exact norm_nonneg _
  rw [matrix_norm_eq_euclidean_operator_norm,
    euclideanOperator_polynomialEval, ← cfc_polynomial p (euclideanOperator A)]
  apply norm_cfc_le hnonneg
  intro w hw
  apply norm_polynomial_eval_le_maxOnNumericalRange A p
  apply matrixSpectrum_subset_numericalRange A
  simpa only [matrixSpectrum_eq_operatorSpectrum] using hw

/-- CFT-29-E05: positive normalization and rescaling of a polynomial bound. -/
theorem exercise_05_solution :
    ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n]
      (A : SquareMatrix n) (p : Polynomial ℂ) (M : ℝ),
      0 < M →
      maxPolynomialModulusOnNumericalRange A p = M →
      maxPolynomialModulusOnNumericalRange A (((M⁻¹ : ℝ) : ℂ) • p) = 1 →
      ‖polynomialEval (((M⁻¹ : ℝ) : ℂ) • p) A‖ ≤ 2 →
      ‖polynomialEval p A‖ ≤ 2 * M := by
  intro n _ _ _ A p M hM hmax hnormalizedMax hnormalized
  have hderivedMax := normalized_max_polynomial_modulus A p M hM hmax
  have hnormalizedCrouzeix :
      ‖polynomialEval (((M⁻¹ : ℝ) : ℂ) • p) A‖ ≤
        2 * maxPolynomialModulusOnNumericalRange A
          (((M⁻¹ : ℝ) : ℂ) • p) := by
    simpa only [hnormalizedMax, mul_one] using hnormalized
  have hnormalized' :
      ‖polynomialEval (((M⁻¹ : ℝ) : ℂ) • p) A‖ ≤ 2 := by
    simpa only [hderivedMax, mul_one] using hnormalizedCrouzeix
  have heval :
      polynomialEval (((M⁻¹ : ℝ) : ℂ) • p) A =
        (((M⁻¹ : ℝ) : ℂ) • polynomialEval p A) := by
    simp [polynomialEval, Algebra.smul_def]
  rw [heval, norm_smul, Complex.norm_real, Real.norm_eq_abs,
    abs_of_pos (inv_pos.2 hM)] at hnormalized'
  have hdiv : ‖polynomialEval p A‖ / M ≤ 2 := by
    simpa only [div_eq_mul_inv, mul_comm] using hnormalized'
  exact (div_le_iff₀ hM).mp hdiv

/-- CFT-29-E06: the expanded finite-index main-theorem surface. -/
theorem exercise_06_solution :
    ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n],
      MainTheoremStatement (n := n) ↔
        ∀ (A : SquareMatrix n) (p : Polynomial ℂ),
          PolynomialCrouzeixBound A p := by
  intro n _ _ _
  rfl

end Exercises.Chapter29
end
end CrouzeixTextbook.Part06
